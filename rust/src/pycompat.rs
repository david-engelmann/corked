//! Small pieces of Python semantics the port depends on.
//!
//! Each helper mirrors one behaviour of the Python runtime that the
//! specification rests on, so the domain modules read plainly. The
//! semantics were probed against Python 3.12 on this machine, and
//! the probes stand in the unit tests below.

/// Return whether a code point is Python whitespace.
///
/// Python str.strip, str.isspace, and the whitespace the text
/// readers honour cover the ASCII controls from tab to unit
/// separator in part, the space, and the Unicode space characters.
/// The set differs from the Unicode White_Space property in the
/// file, group, record, and unit separators, which Python counts
/// and Unicode does not.
pub fn is_python_space(c: char) -> bool {
    matches!(
        c,
        '\u{0009}'..='\u{000d}'
            | '\u{001c}'..='\u{001f}'
            | '\u{0020}'
            | '\u{0085}'
            | '\u{00a0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200a}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202f}'
            | '\u{205f}'
            | '\u{3000}'
    )
}

/// Return whether a UTF-16 code unit is a Python whitespace
/// character. Every Python whitespace character stands in the
/// basic multilingual plane, so a code unit read on its own
/// settles the question, and a surrogate unit never counts.
pub fn is_python_space_unit(unit: u16) -> bool {
    char::from_u32(unit as u32).is_some_and(is_python_space)
}

/// The ranges of the Unicode Nd category, the decimal digits.
/// The table was generated from the unicodedata module of the
/// Python 3.12.3 on this machine, whose database is Unicode
/// 15.0.0, by reading the category of every code point. The
/// digit class of a Python regular expression reads exactly
/// this category, so a drawn name over digits of any script is
/// drawn as the Python draws it.
const ND_RANGES: &[(u32, u32)] = &[
    (0x0030, 0x0039),
    (0x0660, 0x0669),
    (0x06f0, 0x06f9),
    (0x07c0, 0x07c9),
    (0x0966, 0x096f),
    (0x09e6, 0x09ef),
    (0x0a66, 0x0a6f),
    (0x0ae6, 0x0aef),
    (0x0b66, 0x0b6f),
    (0x0be6, 0x0bef),
    (0x0c66, 0x0c6f),
    (0x0ce6, 0xcef),
    (0x0d66, 0x0d6f),
    (0x0de6, 0x0def),
    (0x0e50, 0x0e59),
    (0x0ed0, 0x0ed9),
    (0x0f20, 0x0f29),
    (0x1040, 0x1049),
    (0x1090, 0x1099),
    (0x17e0, 0x17e9),
    (0x1810, 0x1819),
    (0x1946, 0x194f),
    (0x19d0, 0x19d9),
    (0x1a80, 0x1a89),
    (0x1a90, 0x1a99),
    (0x1b50, 0x1b59),
    (0x1bb0, 0x1bb9),
    (0x1c40, 0x1c49),
    (0x1c50, 0x1c59),
    (0xa620, 0xa629),
    (0xa8d0, 0xa8d9),
    (0xa900, 0xa909),
    (0xa9d0, 0xa9d9),
    (0xa9f0, 0xa9f9),
    (0xaa50, 0xaa59),
    (0xabf0, 0xabf9),
    (0xff10, 0xff19),
    (0x104a0, 0x104a9),
    (0x10d30, 0x10d39),
    (0x11066, 0x1106f),
    (0x110f0, 0x110f9),
    (0x11136, 0x1113f),
    (0x111d0, 0x111d9),
    (0x112f0, 0x112f9),
    (0x11450, 0x11459),
    (0x114d0, 0x114d9),
    (0x11650, 0x11659),
    (0x116c0, 0x116c9),
    (0x11730, 0x11739),
    (0x118e0, 0x118e9),
    (0x11950, 0x11959),
    (0x11c50, 0x11c59),
    (0x11d50, 0x11d59),
    (0x11da0, 0x11da9),
    (0x11f50, 0x11f59),
    (0x16a60, 0x16a69),
    (0x16ac0, 0x16ac9),
    (0x16b50, 0x16b59),
    (0x1d7ce, 0x1d7ff),
    (0x1e140, 0x1e149),
    (0x1e2f0, 0x1e2f9),
    (0x1e4f0, 0x1e4f9),
    (0x1e950, 0x1e959),
    (0x1fbf0, 0x1fbf9),
];

/// Return whether a character is a decimal digit in the Python
/// reading, a member of the Unicode Nd category.
pub fn is_decimal_digit(c: char) -> bool {
    let point = c as u32;
    ND_RANGES
        .binary_search_by(|range| {
            if point < range.0 {
                std::cmp::Ordering::Greater
            } else if point > range.1 {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

/// Return whether a string is blank under Python str.strip.
///
/// A string is blank when it holds nothing but Python whitespace,
/// which is the reading the validators give a blank field.
pub fn is_blank_str(text: &str) -> bool {
    text.chars().all(is_python_space)
}

/// Strip Python whitespace from both ends of a string, as
/// Python str.strip does.
pub fn strip_str(text: &str) -> &str {
    text.trim_matches(is_python_space)
}

/// Strip Python whitespace from the start of a string.
pub fn lstrip_str(text: &str) -> &str {
    text.trim_start_matches(is_python_space)
}

/// Strip Python whitespace from the end of a string.
pub fn rstrip_str(text: &str) -> &str {
    text.trim_end_matches(is_python_space)
}

/// The line boundaries Python str.splitlines splits on.
fn is_line_boundary(c: char) -> bool {
    matches!(
        c,
        '\n' | '\r'
            | '\u{000b}'
            | '\u{000c}'
            | '\u{001c}'
            | '\u{001d}'
            | '\u{001e}'
            | '\u{0085}'
            | '\u{2028}'
            | '\u{2029}'
    )
}

/// Split a string into lines the way Python str.splitlines does.
///
/// The boundaries are the newline, the carriage return, the pair
/// of the two read as one boundary, the vertical tab, the form
/// feed, the file, group, and record separators, the next line
/// character, and the two Unicode line and paragraph separators.
/// A trailing boundary leaves no empty final line, and an empty
/// string holds no lines.
pub fn split_lines(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut lines = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i < chars.len() {
        if is_line_boundary(chars[i]) {
            lines.push(chars[start..i].iter().collect());
            if chars[i] == '\r' && i + 1 < chars.len() && chars[i + 1] == '\n' {
                i += 1;
            }
            i += 1;
            start = i;
        } else {
            i += 1;
        }
    }
    if start < chars.len() {
        lines.push(chars[start..].iter().collect());
    }
    lines
}

/// The hard keywords of Python 3.12, as keyword.iskeyword reads.
const KEYWORDS: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class", "continue",
    "def", "del", "elif", "else", "except", "finally", "for", "from", "global", "if", "import",
    "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while",
    "with", "yield",
];

/// Return whether a name is a Python keyword.
pub fn is_keyword(name: &str) -> bool {
    KEYWORDS.contains(&name)
}

/// Return whether a name is a usable Python identifier.
///
/// The names read through this helper are drawn from callee names
/// the checker already read as ASCII identifier characters, so the
/// ASCII reading of the identifier rule settles them. A name is an
/// identifier when it is non empty, opens with a letter or an
/// underscore, and holds only letters, digits, and underscores.
pub fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        None => false,
        Some(first) => {
            (first.is_ascii_alphabetic() || first == '_')
                && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
    }
}

/// Render a float the way Python repr renders it.
///
/// The digits are the shortest decimal digits that parse back to
/// the same value, which both runtimes compute. The form follows
/// the Python rule. A decimal point position past the digits by
/// more than sixteen, or four or more places before the first
/// digit, reads in scientific form with an exponent of at least
/// two digits, and every other value reads in fixed form with a
/// trailing point and zero when it is whole.
pub fn repr_float(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value < 0.0 {
            "-inf".to_string()
        } else {
            "inf".to_string()
        };
    }
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0.0".to_string()
        } else {
            "0.0".to_string()
        };
    }
    let negative = value < 0.0;
    // The shortest digits and their decimal exponent, read from
    // the scientific form the standard library prints. The value
    // stands as the first digit, a point, the remaining digits,
    // and a power of ten.
    let sci = format!("{:e}", value.abs());
    let (mantissa, exponent) = sci
        .split_once('e')
        .expect("scientific form always holds an exponent");
    let exponent: i32 = exponent.parse().expect("exponent is an integer");
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    // The decimal point position, counting the value as the digits
    // read behind a leading zero and a point.
    let decpt = exponent + 1;
    let mut out = String::new();
    if negative {
        out.push('-');
    }
    if decpt <= -4 || decpt > 16 {
        out.push_str(&digits[..1]);
        if digits.len() > 1 {
            out.push('.');
            out.push_str(&digits[1..]);
        }
        out.push('e');
        let exp = decpt - 1;
        if exp < 0 {
            out.push('-');
        } else {
            out.push('+');
        }
        let exp_digits = format!("{}", exp.abs());
        if exp_digits.len() < 2 {
            out.push('0');
        }
        out.push_str(&exp_digits);
        return out;
    }
    if decpt <= 0 {
        out.push_str("0.");
        for _ in 0..(-decpt) {
            out.push('0');
        }
        out.push_str(&digits);
        return out;
    }
    let point = decpt as usize;
    if point >= digits.len() {
        out.push_str(&digits);
        for _ in 0..(point - digits.len()) {
            out.push('0');
        }
        out.push_str(".0");
        return out;
    }
    out.push_str(&digits[..point]);
    out.push('.');
    out.push_str(&digits[point..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_space_covers_the_separator_controls() {
        assert!(is_python_space('\u{001c}'));
        assert!(is_python_space('\u{001f}'));
        assert!(is_python_space('\u{0085}'));
        assert!(is_python_space('\u{00a0}'));
        assert!(!is_python_space('x'));
    }

    #[test]
    fn split_lines_reads_every_boundary() {
        assert_eq!(split_lines(""), Vec::<String>::new());
        assert_eq!(split_lines("a\n"), vec!["a"]);
        assert_eq!(split_lines("a\n\n"), vec!["a", ""]);
        assert_eq!(split_lines("a\r\nb"), vec!["a", "b"]);
        assert_eq!(split_lines("a\rb"), vec!["a", "b"]);
        assert_eq!(split_lines("a\u{000b}b"), vec!["a", "b"]);
        assert_eq!(split_lines("a\u{0085}b"), vec!["a", "b"]);
        assert_eq!(split_lines("a\u{2028}b"), vec!["a", "b"]);
        assert_eq!(split_lines("a\u{001c}b"), vec!["a", "b"]);
        assert_eq!(split_lines("abc"), vec!["abc"]);
    }

    #[test]
    fn repr_float_matches_the_python_probes() {
        let cases = [
            (0.78, "0.78"),
            (0.5, "0.5"),
            (1.0, "1.0"),
            (0.0, "0.0"),
            (0.0001, "0.0001"),
            (0.00001, "1e-05"),
            (0.1, "0.1"),
            (0.30000000000000004, "0.30000000000000004"),
            (1e16, "1e+16"),
            (1234567890123456.0, "1234567890123456.0"),
            (2.5e-5, "2.5e-05"),
            (0.32, "0.32"),
            (0.66, "0.66"),
            (-0.0, "-0.0"),
            (5.23e-5, "5.23e-05"),
            (5.55e-5, "5.55e-05"),
        ];
        for (value, expected) in cases {
            assert_eq!(repr_float(value), expected, "value {value}");
        }
    }

    #[test]
    fn decimal_digits_read_across_scripts() {
        assert!(is_decimal_digit('5'));
        assert!(is_decimal_digit('\u{0663}'));
        assert!(is_decimal_digit('\u{ff13}'));
        assert!(is_decimal_digit('\u{0969}'));
        assert!(is_decimal_digit('\u{104a5}'));
        assert!(!is_decimal_digit('a'));
        assert!(!is_decimal_digit('\u{00b2}'));
        assert!(!is_decimal_digit('\u{066a}'));
        assert!(!is_decimal_digit('\u{0964}'));
    }

    #[test]
    fn identifiers_and_keywords_read_as_python_reads() {
        assert!(is_identifier("orders"));
        assert!(is_identifier("_x"));
        assert!(!is_identifier(""));
        assert!(!is_identifier("2fast"));
        assert!(is_keyword("class"));
        assert!(!is_keyword("orders"));
    }
}
