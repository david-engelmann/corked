//! The built in checker. It reads a unified diff as text and
//! raises the six shapes the contract names, as complete
//! objections, and flags the borderline lines CK-7 names beside
//! the judgements.
//!
//! The readings here mirror the Python pattern readings one for
//! one. The patterns are read by hand over the characters of a
//! line, because the readings rest on Python pattern behaviour
//! the port states as it goes, the space class that covers the
//! separator controls, the dollar that ends a line, and the print
//! call that reads a boundary behind it and a parenthesis ahead
//! of it.

use crate::pycompat::{is_decimal_digit, is_identifier, is_keyword, is_python_space, strip_str};

pub const PLACEHOLDERS: [&str; 6] = ["data2", "temp2", "foo", "bar", "stuff", "thing"];

pub const NEAR_PLACEHOLDERS: [&str; 3] = ["data", "temp", "tmp"];

pub const BORDERLINE_PATTERNS: [&str; 3] = [
    "near placeholder name",
    "broad handler",
    "shared member default",
];

pub const SHAPES: [&str; 6] = [
    "bare except",
    "print call",
    "mutable default",
    "placeholder name",
    "call default",
    "class attribute",
];

const CALL_REWRITE_END: &str =
    " and build each replaced default inside the body under a None check";

/// One objection the checker raises, complete in the form CK-2
/// states.
#[derive(Clone, Debug, PartialEq)]
pub struct Objection {
    pub flaw: String,
    pub standard: String,
    pub rewrite: String,
    pub category: String,
}

/// One borderline flag, holding the number of the flagged line
/// counting every line of the diff text from one, the pattern it
/// matched, the flagged line as written, and the reading the
/// checker computed for it.
#[derive(Clone, Debug, PartialEq)]
pub struct Flag {
    pub line: usize,
    pub pattern: String,
    pub content: String,
    pub reading: String,
}

fn chars_of(text: &str) -> Vec<char> {
    text.chars().collect()
}

fn string_of(chars: &[char]) -> String {
    chars.iter().collect()
}

/// Skip Python whitespace from an index. Returns the first index
/// at or after the start that holds no whitespace.
fn skip_spaces(chars: &[char], mut index: usize) -> usize {
    while index < chars.len() && is_python_space(chars[index]) {
        index += 1;
    }
    index
}

/// Strip Python whitespace from a character span, as str.strip
/// reads the text the span holds.
fn strip_chars(chars: &[char]) -> &[char] {
    let start = skip_spaces(chars, 0);
    let mut end = chars.len();
    while end > start && is_python_space(chars[end - 1]) {
        end -= 1;
    }
    &chars[start..end]
}

fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Read an identifier at an index, in the class the patterns
/// state. Returns the span of the name.
fn read_name(chars: &[char], start: usize) -> Option<(usize, usize)> {
    if start >= chars.len() || !is_name_start(chars[start]) {
        return None;
    }
    let mut end = start + 1;
    while end < chars.len() && is_name_char(chars[end]) {
        end += 1;
    }
    Some((start, end))
}

fn starts_with(chars: &[char], index: usize, literal: &str) -> bool {
    let literal: Vec<char> = literal.chars().collect();
    index + literal.len() <= chars.len() && chars[index..index + literal.len()] == literal[..]
}

/// Read a brace literal as text. It is a dict when it is empty or
/// holds a colon at its top level, and a set literal otherwise.
/// The reading opens on the second character, as the Python
/// reading walks the text behind the opening brace.
fn is_dict_literal(chars: &[char]) -> bool {
    let mut depth: i64 = 0;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut seen_content = false;
    for ch in chars.iter().skip(1) {
        let ch = *ch;
        if quote.is_some() {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if Some(ch) == quote {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            seen_content = true;
        } else if matches!(ch, '{' | '[' | '(') {
            depth += 1;
            seen_content = true;
        } else if matches!(ch, '}' | ']' | ')') {
            if depth == 0 {
                return !seen_content;
            }
            depth -= 1;
        } else if ch == ':' && depth == 0 {
            return true;
        } else if !is_python_space(ch) {
            seen_content = true;
        }
    }
    !seen_content
}

/// Read a right hand side as one literal. It is one literal when
/// it opens with a bracket or a brace and the opener closes
/// exactly at the last character.
fn is_single_literal(chars: &[char]) -> bool {
    if chars.is_empty() || !matches!(chars[0], '[' | '{') {
        return false;
    }
    let mut depth: i64 = 0;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (index, ch) in chars.iter().enumerate() {
        let ch = *ch;
        if quote.is_some() {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if Some(ch) == quote {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if matches!(ch, '[' | '{' | '(') {
            depth += 1;
        } else if matches!(ch, ']' | '}' | ')') {
            depth -= 1;
            if depth == 0 {
                return index == chars.len() - 1;
            }
        }
    }
    false
}

/// Read a default text as one tuple literal. It is one when it
/// opens with a parenthesis and the opener closes exactly at the
/// last character.
fn is_tuple_literal(chars: &[char]) -> bool {
    if chars.is_empty() || chars[0] != '(' {
        return false;
    }
    let mut depth: i64 = 0;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (index, ch) in chars.iter().enumerate() {
        let ch = *ch;
        if quote.is_some() {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if Some(ch) == quote {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if matches!(ch, '(' | '[' | '{') {
            depth += 1;
        } else if matches!(ch, ')' | ']' | '}') {
            depth -= 1;
            if depth == 0 {
                return index == chars.len() - 1;
            }
        }
    }
    false
}

/// Split text at top level commas. Returns the spans of the
/// parts as offsets from base, so callers can rebuild the
/// original line.
fn split_top_level(chars: &[char], base: usize) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut depth: i64 = 0;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut start = 0usize;
    for (index, ch) in chars.iter().enumerate() {
        let ch = *ch;
        if quote.is_some() {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if Some(ch) == quote {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if matches!(ch, '(' | '[' | '{') {
            depth += 1;
        } else if matches!(ch, ')' | ']' | '}') {
            depth -= 1;
        } else if ch == ',' && depth == 0 {
            spans.push((base + start, base + index));
            start = index + 1;
        }
    }
    spans.push((base + start, base + chars.len()));
    spans
}

/// Find the span of the default text inside one parameter held
/// in chars[start..end]. Returns the span of the stripped
/// default, or None when the parameter holds no default.
fn default_span(chars: &[char], start: usize, end: usize) -> Option<(usize, usize)> {
    let mut depth: i64 = 0;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut index = start;
    while index < end {
        let ch = chars[index];
        if quote.is_some() {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if Some(ch) == quote {
                quote = None;
            }
            index += 1;
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if matches!(ch, '(' | '[' | '{') {
            depth += 1;
        } else if matches!(ch, ')' | ']' | '}') {
            depth -= 1;
        } else if ch == '=' && depth == 0 {
            if index + 1 < end && chars[index + 1] == '=' {
                index += 1;
                continue;
            }
            let raw = &chars[index + 1..end];
            let stripped = strip_chars(raw);
            if stripped.is_empty() {
                return None;
            }
            let mut lead = 0;
            while lead < raw.len() && is_python_space(raw[lead]) {
                lead += 1;
            }
            return Some((index + 1 + lead, index + 1 + lead + stripped.len()));
        }
        index += 1;
    }
    None
}

/// Find the closing parenthesis that matches the opening
/// parenthesis at open, reading quotes the way the shared
/// scanning reads them. Returns its index, or None.
fn find_close_paren(chars: &[char], open: usize) -> Option<usize> {
    let mut depth: i64 = 0;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (index, ch) in chars.iter().enumerate().skip(open) {
        let ch = *ch;
        if quote.is_some() {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if Some(ch) == quote {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

/// Read the head of a call text, a plain or dotted name followed
/// by an opening parenthesis. Returns the index just past the
/// parenthesis.
fn call_head(chars: &[char]) -> Option<usize> {
    let (_start, mut end) = read_name(chars, 0)?;
    loop {
        if end < chars.len()
            && chars[end] == '.'
            && end + 1 < chars.len()
            && is_name_start(chars[end + 1])
        {
            let (_s, next) = read_name(chars, end + 1)?;
            end = next;
        } else {
            break;
        }
    }
    let paren = skip_spaces(chars, end);
    if paren < chars.len() && chars[paren] == '(' {
        Some(paren + 1)
    } else {
        None
    }
}

/// Read a default text as a call. It is a call when it is a plain
/// or dotted name followed by a parenthesised argument list that
/// closes at the end of the text.
fn is_call_text(chars: &[char]) -> bool {
    let Some(after) = call_head(chars) else {
        return false;
    };
    let open = after - 1;
    let mut depth: i64 = 0;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (index, ch) in chars.iter().enumerate().skip(open) {
        let ch = *ch;
        if quote.is_some() {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if Some(ch) == quote {
                quote = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
        } else if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                return index == chars.len() - 1;
            }
        }
    }
    false
}

/// Read a mutable literal at an index, one of an empty list, an
/// empty dict, and an empty set call. Returns the index just
/// past the literal.
fn mutable_literal_end(chars: &[char], start: usize) -> Option<usize> {
    if start >= chars.len() {
        return None;
    }
    match chars[start] {
        '[' => {
            let end = skip_spaces(chars, start + 1);
            if end < chars.len() && chars[end] == ']' {
                Some(end + 1)
            } else {
                None
            }
        }
        '{' => {
            let end = skip_spaces(chars, start + 1);
            if end < chars.len() && chars[end] == '}' {
                Some(end + 1)
            } else {
                None
            }
        }
        's' => {
            if !starts_with(chars, start, "set") {
                return None;
            }
            let paren = skip_spaces(chars, start + 3);
            if paren >= chars.len() || chars[paren] != '(' {
                return None;
            }
            let end = skip_spaces(chars, paren + 1);
            if end < chars.len() && chars[end] == ')' {
                Some(end + 1)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Read a mutable default standing at an equals sign. Returns
/// the span of the whole reading and the end of its first group,
/// the equals sign and the spaces behind it.
fn mutable_default_at(chars: &[char], equals: usize) -> Option<(usize, usize, usize)> {
    if chars[equals] != '=' {
        return None;
    }
    let group_end = skip_spaces(chars, equals + 1);
    let end = mutable_literal_end(chars, group_end)?;
    Some((equals, group_end, end))
}

/// Find the first mutable default in a line.
fn find_mutable_default(chars: &[char]) -> Option<(usize, usize, usize)> {
    (0..chars.len()).find_map(|index| mutable_default_at(chars, index))
}

/// Replace every mutable default in a line with None, keeping
/// the equals sign and the spaces behind it at each reading.
fn sub_mutable_defaults(chars: &[char]) -> String {
    let mut out: Vec<char> = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '=' {
            if let Some((_start, group_end, end)) = mutable_default_at(chars, index) {
                out.extend_from_slice(&chars[index..group_end]);
                out.extend("None".chars());
                index = end;
                continue;
            }
        }
        out.push(chars[index]);
        index += 1;
    }
    string_of(&out)
}

/// Read a whole default text as a mutable default and nothing
/// else.
fn is_mutable_default_whole(chars: &[char]) -> bool {
    mutable_literal_end(chars, 0) == Some(chars.len()) && !chars.is_empty()
}

/// A def line reading, holding the span of the function name
/// and the index of its opening parenthesis.
struct DefMatch {
    name_start: usize,
    name_end: usize,
    paren: usize,
}

impl DefMatch {
    fn name(&self, chars: &[char]) -> String {
        string_of(&chars[self.name_start..self.name_end])
    }
}

/// Read a def line opening at a position. The word def stands
/// there, a name follows it, and an opening parenthesis follows
/// the name.
fn def_at(chars: &[char], start: usize) -> Option<DefMatch> {
    if !starts_with(chars, start, "def") {
        return None;
    }
    let after_def = skip_spaces(chars, start + 3);
    if after_def == start + 3 {
        return None;
    }
    let (name_start, name_end) = read_name(chars, after_def)?;
    let paren = skip_spaces(chars, name_end);
    if paren < chars.len() && chars[paren] == '(' {
        Some(DefMatch {
            name_start,
            name_end,
            paren,
        })
    } else {
        None
    }
}

/// Read a line as a def line. An optional async word stands
/// before the word def.
fn match_def_line(chars: &[char]) -> Option<DefMatch> {
    let start = skip_spaces(chars, 0);
    if starts_with(chars, start, "async") {
        let after = skip_spaces(chars, start + 5);
        if after > start + 5 {
            if let Some(found) = def_at(chars, after) {
                return Some(found);
            }
        }
    }
    def_at(chars, start)
}

/// An assignment reading, holding the span of the target name
/// and the index where the assigned text opens.
struct AssignMatch {
    name_start: usize,
    name_end: usize,
    rhs_start: usize,
}

impl AssignMatch {
    fn name(&self, chars: &[char]) -> String {
        string_of(&chars[self.name_start..self.name_end])
    }
}

/// Read a line as a plain assignment.
fn match_assignment(chars: &[char]) -> Option<AssignMatch> {
    let start = skip_spaces(chars, 0);
    let (name_start, name_end) = read_name(chars, start)?;
    let equals = skip_spaces(chars, name_end);
    if equals >= chars.len() || chars[equals] != '=' {
        return None;
    }
    if equals + 1 < chars.len() && chars[equals + 1] == '=' {
        return None;
    }
    let rhs_start = skip_spaces(chars, equals + 1);
    if rhs_start >= chars.len() {
        return None;
    }
    Some(AssignMatch {
        name_start,
        name_end,
        rhs_start,
    })
}

/// Read a line as an annotated assignment. The annotation is
/// read lazily, so the first equals sign the rest of the reading
/// accepts is the delimiter, and an equals sign inside the
/// annotation is swallowed by it as the Python reading swallows
/// it.
fn match_annotated(chars: &[char]) -> Option<AssignMatch> {
    let start = skip_spaces(chars, 0);
    let (name_start, name_end) = read_name(chars, start)?;
    let colon = skip_spaces(chars, name_end);
    if colon >= chars.len() || chars[colon] != ':' {
        return None;
    }
    let annotation = skip_spaces(chars, colon + 1);
    let mut probe = annotation + 1;
    while probe <= chars.len() {
        let equals = skip_spaces(chars, probe);
        if equals < chars.len() && chars[equals] == '=' {
            let doubled = equals + 1 < chars.len() && chars[equals + 1] == '=';
            if !doubled {
                let rhs_start = skip_spaces(chars, equals + 1);
                if rhs_start < chars.len() {
                    return Some(AssignMatch {
                        name_start,
                        name_end,
                        rhs_start,
                    });
                }
            }
        }
        probe += 1;
    }
    None
}

/// Read a line as a tuple assignment. Returns the target names
/// in order.
fn match_tuple_assignment(chars: &[char]) -> Option<Vec<String>> {
    let start = skip_spaces(chars, 0);
    let (_first_start, mut end) = read_name(chars, start)?;
    let mut names = vec![string_of(&chars[start..end])];
    loop {
        let comma = skip_spaces(chars, end);
        if comma < chars.len() && chars[comma] == ',' {
            let name_at = skip_spaces(chars, comma + 1);
            let (name_start, name_end) = read_name(chars, name_at)?;
            names.push(string_of(&chars[name_start..name_end]));
            end = name_end;
        } else {
            break;
        }
    }
    if names.len() < 2 {
        return None;
    }
    let equals = skip_spaces(chars, end);
    if equals >= chars.len() || chars[equals] != '=' {
        return None;
    }
    if equals + 1 < chars.len() && chars[equals + 1] == '=' {
        return None;
    }
    let rhs = skip_spaces(chars, equals + 1);
    if rhs >= chars.len() {
        return None;
    }
    Some(names)
}

/// Read a text as a simple call. Returns the callee name.
fn match_simple_call(chars: &[char]) -> Option<String> {
    let (name_start, name_end) = read_name(chars, 0)?;
    let paren = skip_spaces(chars, name_end);
    if paren >= chars.len() || chars[paren] != '(' {
        return None;
    }
    let mut end = chars.len();
    while end > 0 && is_python_space(chars[end - 1]) {
        end -= 1;
    }
    if end > paren && chars[end - 1] == ')' {
        Some(string_of(&chars[name_start..name_end]))
    } else {
        None
    }
}

/// Read a text as a number, an optional minus over digits with
/// an optional fractional part. The digits are the decimal
/// digits Python reads, the Nd category in any script.
fn is_number_literal(chars: &[char]) -> bool {
    let mut index = 0;
    if index < chars.len() && chars[index] == '-' {
        index += 1;
    }
    let digits_start = index;
    while index < chars.len() && is_decimal_digit(chars[index]) {
        index += 1;
    }
    if index == digits_start {
        return false;
    }
    if index < chars.len() && chars[index] == '.' {
        index += 1;
        let frac_start = index;
        while index < chars.len() && is_decimal_digit(chars[index]) {
            index += 1;
        }
        if index == frac_start {
            return false;
        }
    }
    index == chars.len()
}

/// Draw a name from the right hand side of an assignment.
///
/// Returns the drawn name, or None when no name can be drawn.
fn drawn_name(rhs: &[char]) -> Option<String> {
    let drawn = if let Some(callee) = match_simple_call(rhs) {
        let words: Vec<&str> = callee.split('_').collect();
        if words.len() > 1 {
            words[1..].join("_")
        } else {
            callee
        }
    } else if rhs[0] == '"' || rhs[0] == '\'' {
        "text".to_string()
    } else if rhs[0] == '[' {
        "items".to_string()
    } else if rhs[0] == '{' {
        if !is_dict_literal(rhs) {
            return None;
        }
        "mapping".to_string()
    } else if is_number_literal(rhs) {
        "number".to_string()
    } else {
        return None;
    };
    if drawn.is_empty() || !is_identifier(&drawn) {
        return None;
    }
    if PLACEHOLDERS.contains(&drawn.as_str()) || is_keyword(&drawn) {
        return None;
    }
    Some(drawn)
}

/// Build the placeholder objection for one name.
fn placeholder_objection(name: &str, drawn_line: Option<String>, holds_value: bool) -> Objection {
    let rewrite = if let Some(line) = drawn_line {
        line
    } else if holds_value {
        format!(
            "Rename {name} to a name that says what the value holds \
             and keep the rest of the line as written"
        )
    } else {
        format!(
            "Rename {name} to a name that says what the function does \
             and keep the signature as written"
        )
    };
    let (flaw, standard) = if holds_value {
        (
            format!(
                "The added line binds a value to the placeholder \
                 name {name}"
            ),
            "A name says what its value holds so the reader never \
             has to hold the assignment in memory to follow the code"
                .to_string(),
        )
    } else {
        (
            format!(
                "The added line gives a function the placeholder \
                 name {name}"
            ),
            "A function name says what the function does so a call \
             site reads as a statement of intent"
                .to_string(),
        )
    };
    Objection {
        flaw,
        standard,
        rewrite,
        category: "naming".to_string(),
    }
}

/// Redraw a def line with every call default replaced by None.
///
/// Returns the redrawn line, or None when the def line holds no
/// call default the shape can see.
fn call_default_rewrite(chars: &[char], def_match: &DefMatch) -> Option<String> {
    let open = def_match.paren;
    let close = find_close_paren(chars, open)?;
    let params = &chars[open + 1..close];
    let mut spans = Vec::new();
    for (start, end) in split_top_level(params, open + 1) {
        let Some(span) = default_span(chars, start, end) else {
            continue;
        };
        let default = &chars[span.0..span.1];
        if is_mutable_default_whole(default) {
            continue;
        }
        if is_call_text(default) {
            spans.push(span);
        }
    }
    if spans.is_empty() {
        return None;
    }
    let mut redrawn: Vec<char> = chars.to_vec();
    for (start, end) in spans.iter().rev() {
        let replacement: Vec<char> = "None".chars().collect();
        redrawn.splice(start..end, replacement);
    }
    Some(string_of(&redrawn) + CALL_REWRITE_END)
}

/// Raise the class attribute objection for an assignment line,
/// or return None when the line holds no single literal
/// assignment.
fn class_attribute_objection(chars: &[char]) -> Option<Objection> {
    let found = match_annotated(chars).or_else(|| match_assignment(chars))?;
    let (name, rhs): (String, &[char]) =
        (found.name(chars), strip_chars(&chars[found.rhs_start..]));
    if !is_single_literal(rhs) {
        return None;
    }
    let rhs = string_of(rhs);
    Some(Objection {
        flaw: format!(
            "The added line gives the class attribute {name} a \
             mutable literal that every instance shares"
        ),
        standard: "A mutable value that belongs to each instance \
                   is built in the initializer, so one instance \
                   changing it never changes it for every other \
                   instance"
            .to_string(),
        rewrite: format!(
            "Move {name} into the initializer as self.{name} = \
             {rhs} and remove the class level assignment"
        ),
        category: "structure".to_string(),
    })
}

/// Read the bare except rewrite of a line, when the line opens a
/// bare except clause. The rewrite keeps the leading spaces and
/// ends the clause naming Exception.
fn bare_except_rewrite(chars: &[char]) -> Option<String> {
    let lead = skip_spaces(chars, 0);
    if !starts_with(chars, lead, "except") {
        return None;
    }
    let colon = skip_spaces(chars, lead + 6);
    if colon < chars.len() && chars[colon] == ':' {
        let mut out: Vec<char> = chars[..lead].to_vec();
        out.extend("except Exception:".chars());
        out.extend_from_slice(&chars[colon + 1..]);
        Some(string_of(&out))
    } else {
        None
    }
}

/// Return whether a character is a word character, as the print
/// reading's boundary behind the name reads one.
fn is_word_char(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

/// Find a print call in a line. Returns the index of its name.
fn find_print_call(chars: &[char]) -> Option<usize> {
    (0..chars.len()).find(|&index| {
        if !starts_with(chars, index, "print") {
            return false;
        }
        if index > 0 && (is_word_char(chars[index - 1]) || chars[index - 1] == '.') {
            return false;
        }
        let paren = skip_spaces(chars, index + 5);
        paren < chars.len() && chars[paren] == '('
    })
}

/// Read a line as opening a broad handler, a handler on
/// BaseException with an optional binding.
fn match_broad_handler(chars: &[char]) -> bool {
    let start = skip_spaces(chars, 0);
    if !starts_with(chars, start, "except") {
        return false;
    }
    let name_at = skip_spaces(chars, start + 6);
    if name_at == start + 6 {
        return false;
    }
    if !starts_with(chars, name_at, "BaseException") {
        return false;
    }
    let after = name_at + "BaseException".len();
    // The binding form first, as the pattern tries it first.
    let bound = (|| {
        let as_at = skip_spaces(chars, after);
        if as_at == after || !starts_with(chars, as_at, "as") {
            return None;
        }
        let name_at = skip_spaces(chars, as_at + 2);
        if name_at == as_at + 2 {
            return None;
        }
        let (_s, name_end) = read_name(chars, name_at)?;
        Some(skip_spaces(chars, name_end))
    })();
    if let Some(colon) = bound {
        if colon < chars.len() && chars[colon] == ':' {
            return true;
        }
    }
    let colon = skip_spaces(chars, after);
    colon < chars.len() && chars[colon] == ':'
}

/// Return the reading for a def line holding a shared member
/// default, or None when no parameter default of the line is a
/// tuple literal holding a single list, dict, or set literal at
/// its top level, under the third borderline pattern of CK-7.
fn shared_member_reading(chars: &[char], def_match: &DefMatch) -> Option<String> {
    let open = def_match.paren;
    let close = find_close_paren(chars, open)?;
    let params = &chars[open + 1..close];
    for (start, end) in split_top_level(params, open + 1) {
        let Some(span) = default_span(chars, start, end) else {
            continue;
        };
        let default = &chars[span.0..span.1];
        if !is_tuple_literal(default) {
            continue;
        }
        let inner = &default[1..default.len() - 1];
        for (member_start, member_end) in split_top_level(inner, 0) {
            let member = strip_chars(&inner[member_start..member_end]);
            if member.is_empty() || !matches!(member[0], '[' | '{') {
                continue;
            }
            if !is_single_literal(member) {
                continue;
            }
            if member[0] == '[' {
                return Some("default holds a list literal".to_string());
            }
            if is_dict_literal(member) {
                return Some("default holds a dict literal".to_string());
            }
            return Some("default holds a set literal".to_string());
        }
    }
    None
}

/// Return the judgements one added line raises, as pairs of the
/// shape name and the objection, in the order the contract
/// states the shapes.
fn line_judgements(chars: &[char], in_class_body: bool) -> Vec<(&'static str, Objection)> {
    let mut judgements: Vec<(&'static str, Objection)> = Vec::new();

    if let Some(rewrite) = bare_except_rewrite(chars) {
        judgements.push((
            "bare except",
            Objection {
                flaw: "The added line catches every exception with \
                       a bare except clause"
                    .to_string(),
                standard: "A handler names the failures it expects \
                           so the failures nobody expected still \
                           rise where they can be seen"
                    .to_string(),
                rewrite,
                category: "smell".to_string(),
            },
        ));
    }

    if let Some(index) = find_print_call(chars) {
        let mut redrawn: Vec<char> = chars[..index].to_vec();
        redrawn.extend("logger.debug".chars());
        redrawn.extend_from_slice(&chars[index + 5..]);
        judgements.push((
            "print call",
            Objection {
                flaw: "The added line sends diagnostics through a \
                       print call"
                    .to_string(),
                standard: "Diagnostics travel through the logging \
                           module so they can be levelled, filtered, \
                           and silenced"
                    .to_string(),
                rewrite: string_of(&redrawn),
                category: "smell".to_string(),
            },
        ));
    }

    let def_match = match_def_line(chars);
    if def_match.is_some() && find_mutable_default(chars).is_some() {
        judgements.push((
            "mutable default",
            Objection {
                flaw: "The added def line gives a parameter a \
                       mutable default that every call shares"
                    .to_string(),
                standard: "A default that must be fresh on each \
                           call is written as None and the value is \
                           built inside the body"
                    .to_string(),
                rewrite: sub_mutable_defaults(chars),
                category: "structure".to_string(),
            },
        ));
    }

    if let Some(found) = &def_match {
        let name = found.name(chars);
        if PLACEHOLDERS.contains(&name.as_str()) {
            judgements.push((
                "placeholder name",
                placeholder_objection(&name, None, false),
            ));
        }
    } else {
        let annotated = match_annotated(chars);
        let assignment = if annotated.is_some() {
            None
        } else {
            match_assignment(chars)
        };
        let mut raised = false;
        if let Some(found) = &annotated {
            let name = found.name(chars);
            if PLACEHOLDERS.contains(&name.as_str()) {
                let rhs = strip_chars(&chars[found.rhs_start..]);
                let drawn = drawn_name(rhs);
                let drawn_line = drawn.as_ref().map(|drawn| {
                    let mut out: Vec<char> = chars[..found.name_start].to_vec();
                    out.extend(drawn.chars());
                    out.extend_from_slice(&chars[found.name_end..]);
                    string_of(&out)
                });
                judgements.push((
                    "placeholder name",
                    placeholder_objection(&name, drawn_line, true),
                ));
                raised = true;
            }
        } else if let Some(found) = &assignment {
            let name = found.name(chars);
            if PLACEHOLDERS.contains(&name.as_str()) {
                let rhs = strip_chars(&chars[found.rhs_start..]);
                let drawn = drawn_name(rhs);
                let drawn_line = drawn.as_ref().map(|drawn| {
                    let mut out: Vec<char> = chars[..found.name_start].to_vec();
                    out.extend(drawn.chars());
                    out.extend_from_slice(&chars[found.name_end..]);
                    string_of(&out)
                });
                judgements.push((
                    "placeholder name",
                    placeholder_objection(&name, drawn_line, true),
                ));
                raised = true;
            }
        }
        if !raised {
            if let Some(targets) = match_tuple_assignment(chars) {
                if let Some(held) = targets
                    .iter()
                    .find(|name| PLACEHOLDERS.contains(&name.as_str()))
                {
                    judgements.push(("placeholder name", placeholder_objection(held, None, true)));
                }
            }
        }
    }

    if let Some(found) = &def_match {
        if let Some(rewrite) = call_default_rewrite(chars, found) {
            judgements.push((
                "call default",
                Objection {
                    flaw: "The added def line gives a parameter a \
                           default that is a call, so the call runs \
                           once when the definition is read and \
                           every call that omits the argument shares \
                           the one result"
                        .to_string(),
                    standard: "A default that must be computed on \
                               each call is written as None and the \
                               value is built inside the body under \
                               a None check"
                        .to_string(),
                    rewrite,
                    category: "structure".to_string(),
                },
            ));
        }
    }

    if in_class_body && def_match.is_none() {
        if let Some(objection) = class_attribute_objection(chars) {
            judgements.push(("class attribute", objection));
        }
    }

    judgements
}

/// Return the borderline flag one added line raises, as a pair
/// of the pattern name and the reading the checker computed, or
/// None when the line matches no borderline pattern of CK-7.
///
/// The caller stands the flag only on a line that raised no
/// judgement, so a flag never stands beside a shape judgement.
fn line_flag(chars: &[char]) -> Option<(&'static str, String)> {
    let def_match = match_def_line(chars);

    if let Some(found) = &def_match {
        let name = found.name(chars);
        if NEAR_PLACEHOLDERS.contains(&name.as_str()) {
            return Some(("near placeholder name", "no drawn name".to_string()));
        }
    }
    if def_match.is_none() {
        let annotated = match_annotated(chars);
        let assignment = if annotated.is_some() {
            None
        } else {
            match_assignment(chars)
        };
        let mut target: Option<String> = None;
        let mut drawn: Option<String> = None;
        if let Some(found) = &annotated {
            let name = found.name(chars);
            if NEAR_PLACEHOLDERS.contains(&name.as_str()) {
                drawn = drawn_name(strip_chars(&chars[found.rhs_start..]));
                target = Some(name);
            }
        } else if let Some(found) = &assignment {
            let name = found.name(chars);
            if NEAR_PLACEHOLDERS.contains(&name.as_str()) {
                drawn = drawn_name(strip_chars(&chars[found.rhs_start..]));
                target = Some(name);
            }
        }
        if target.is_none() {
            if let Some(targets) = match_tuple_assignment(chars) {
                target = targets
                    .iter()
                    .find(|name| NEAR_PLACEHOLDERS.contains(&name.as_str()))
                    .cloned();
            }
        }
        if target.is_some() {
            let reading = match drawn {
                Some(drawn) => format!("drawn name {drawn}"),
                None => "no drawn name".to_string(),
            };
            return Some(("near placeholder name", reading));
        }
    }

    if match_broad_handler(chars) {
        return Some(("broad handler", "handler names BaseException".to_string()));
    }

    if let Some(found) = &def_match {
        if let Some(reading) = shared_member_reading(chars, found) {
            return Some(("shared member default", reading));
        }
    }

    None
}

/// One line the walk reads, with the number of the line counting
/// every line of the text from one, whether the line is an added
/// line, the content of the line without its marker, and whether
/// the line sits directly in a class body.
struct WalkLine {
    number: usize,
    is_added: bool,
    content: String,
    in_class_body: bool,
}

/// Walk the lines of a diff the way the checker reads them.
///
/// Removed lines and headers are never read, and the block stack
/// is kept under the rule CK-5 states.
fn walk(text: &str) -> Vec<WalkLine> {
    let mut blocks: Vec<(usize, bool)> = Vec::new();
    let mut lines = Vec::new();
    for (index, line) in crate::pycompat::split_lines(text).iter().enumerate() {
        let number = index + 1;
        if line.starts_with("@@") || line.starts_with("diff ") {
            blocks.clear();
            continue;
        }
        if line.starts_with("--- ") || line.starts_with("+++ ") {
            blocks.clear();
            continue;
        }
        if line.starts_with('-') || line.starts_with("+++") {
            continue;
        }
        if !line.starts_with('+') && !line.starts_with(' ') {
            continue;
        }
        let content = &line[1..];
        if crate::pycompat::is_blank_str(content) {
            continue;
        }
        let chars = chars_of(content);
        let indent = chars.iter().take_while(|c| **c == ' ').count();
        while blocks.last().is_some_and(|(held, _)| *held >= indent) {
            blocks.pop();
        }
        let in_class_body = blocks.last().is_some_and(|(_, is_class)| *is_class);
        lines.push(WalkLine {
            number,
            is_added: line.starts_with('+'),
            content: content.to_string(),
            in_class_body,
        });
        let stripped = strip_str(content);
        if stripped.starts_with("class ") && crate::pycompat::rstrip_str(content).ends_with(':') {
            blocks.push((indent, true));
        } else if match_def_line(&chars).is_some() {
            blocks.push((indent, false));
        }
    }
    lines
}

/// Return the judgements raised by a unified diff held as text,
/// as pairs of the shape name and the objection.
///
/// Only added lines are judged. Judgements come back in line
/// order and, on one line, in the order the contract states the
/// shapes. The class context is read from the context lines and
/// added lines of the diff itself, under the rule CK-5 states.
pub fn check_diff_detailed(text: &str) -> Vec<(&'static str, Objection)> {
    let mut judgements = Vec::new();
    for line in walk(text) {
        if line.is_added {
            judgements.extend(line_judgements(
                &chars_of(&line.content),
                line.in_class_body,
            ));
        }
    }
    judgements
}

/// Return the judgements and the borderline flags of a diff.
///
/// The judgements are the pairs check_diff_detailed returns. The
/// flags are one object per flagged line, in line order, holding
/// the number of the line in the diff text, the pattern it
/// matched, the line as written, and the reading the checker
/// computed for it, under CK-7. A flag stands only on a line that
/// raised no judgement.
pub fn check_diff_flagged(text: &str) -> (Vec<(&'static str, Objection)>, Vec<Flag>) {
    let mut judgements = Vec::new();
    let mut flags = Vec::new();
    for line in walk(text) {
        if !line.is_added {
            continue;
        }
        let chars = chars_of(&line.content);
        let found = line_judgements(&chars, line.in_class_body);
        judgements.extend(found.clone());
        if !found.is_empty() {
            continue;
        }
        if let Some((pattern, reading)) = line_flag(&chars) {
            flags.push(Flag {
                line: line.number,
                pattern: pattern.to_string(),
                content: line.content.clone(),
                reading,
            });
        }
    }
    (judgements, flags)
}

/// Return the objections raised by a unified diff held as text.
///
/// Only added lines are read. Objections come back in line order
/// and, on one line, in the order the contract states the shapes.
pub fn check_diff_text(text: &str) -> Vec<Objection> {
    check_diff_detailed(text)
        .into_iter()
        .map(|(_shape, objection)| objection)
        .collect()
}
