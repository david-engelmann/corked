"""The built in checker. It reads a unified diff as text and raises the
six shapes the contract names, as complete objections, and flags the
borderline lines CK-7 names beside the judgements."""

import keyword
import re

PLACEHOLDERS = ("data2", "temp2", "foo", "bar", "stuff", "thing")

NEAR_PLACEHOLDERS = ("data", "temp", "tmp")

BORDERLINE_PATTERNS = (
    "near placeholder name",
    "broad handler",
    "shared member default",
)

SHAPES = (
    "bare except",
    "print call",
    "mutable default",
    "placeholder name",
    "call default",
    "class attribute",
)

_BARE_EXCEPT = re.compile(r"^(\s*)except\s*:")
_BROAD_HANDLER = re.compile(
    r"^\s*except\s+BaseException(?:\s+as\s+[A-Za-z_][A-Za-z0-9_]*)?\s*:"
)
_PRINT_CALL = re.compile(r"(?<![\w.])print(?=\s*\()")
_MUTABLE_DEFAULT = re.compile(r"(=\s*)(\[\s*\]|\{\s*\}|set\s*\(\s*\))")
_MUTABLE_DEFAULT_WHOLE = re.compile(r"^(\[\s*\]|\{\s*\}|set\s*\(\s*\))$")
_DEF_LINE = re.compile(r"^\s*(?:async\s+)?def\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(")
_ASSIGNMENT = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=(?![=])\s*(\S.*)$")
_ANNOTATED_ASSIGNMENT = re.compile(
    r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*:\s*(.+?)\s*=(?![=])\s*(\S.*)$"
)
_TUPLE_ASSIGNMENT = re.compile(
    r"^\s*([A-Za-z_][A-Za-z0-9_]*(?:\s*,\s*[A-Za-z_][A-Za-z0-9_]*)+)"
    r"\s*=(?![=])\s*\S"
)
_SIMPLE_CALL = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\s*\(.*\)\s*$")
_NUMBER = re.compile(r"^-?\d+(\.\d+)?$")
_CALL_HEAD = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*)\s*\(")


def _is_dict_literal(rhs):
    """Read a brace literal as text. It is a dict when it is empty or
    holds a colon at its top level, and a set literal otherwise."""
    depth = 0
    quote = None
    escaped = False
    seen_content = False
    for ch in rhs[1:]:
        if quote is not None:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == quote:
                quote = None
            continue
        if ch in "\"'":
            quote = ch
            seen_content = True
        elif ch in "{[(":
            depth += 1
            seen_content = True
        elif ch in "}])":
            if depth == 0:
                return not seen_content
            depth -= 1
        elif ch == ":" and depth == 0:
            return True
        elif not ch.isspace():
            seen_content = True
    return not seen_content


def _is_single_literal(rhs):
    """Read a right hand side as one literal. It is one literal when it
    opens with a bracket or a brace and the opener closes exactly at
    the last character."""
    if not rhs or rhs[0] not in "[{":
        return False
    depth = 0
    quote = None
    escaped = False
    for index, ch in enumerate(rhs):
        if quote is not None:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == quote:
                quote = None
            continue
        if ch in "\"'":
            quote = ch
        elif ch in "[{(":
            depth += 1
        elif ch in "]})":
            depth -= 1
            if depth == 0:
                return index == len(rhs) - 1
    return False


def _drawn_name(rhs):
    """Draw a name from the right hand side of an assignment.

    Returns the drawn name, or None when no name can be drawn."""
    call = _SIMPLE_CALL.match(rhs)
    if call:
        callee = call.group(1)
        words = callee.split("_")
        drawn = "_".join(words[1:]) if len(words) > 1 else callee
    elif rhs.startswith(('"', "'")):
        drawn = "text"
    elif rhs.startswith("["):
        drawn = "items"
    elif rhs.startswith("{"):
        if not _is_dict_literal(rhs):
            return None
        drawn = "mapping"
    elif _NUMBER.match(rhs):
        drawn = "number"
    else:
        return None
    if not drawn or not drawn.isidentifier():
        return None
    if drawn in PLACEHOLDERS or keyword.iskeyword(drawn):
        return None
    return drawn


def _placeholder_objection(name, drawn_line, holds):
    if drawn_line is not None:
        rewrite = drawn_line
    elif holds == "value":
        rewrite = (
            f"Rename {name} to a name that says what the value holds "
            "and keep the rest of the line as written"
        )
    else:
        rewrite = (
            f"Rename {name} to a name that says what the function does "
            "and keep the signature as written"
        )
    if holds == "value":
        flaw = f"The added line binds a value to the placeholder name {name}"
        standard = (
            "A name says what its value holds so the reader never has to "
            "hold the assignment in memory to follow the code"
        )
    else:
        flaw = f"The added line gives a function the placeholder name {name}"
        standard = (
            "A function name says what the function does so a call site "
            "reads as a statement of intent"
        )
    return {
        "flaw": flaw,
        "standard": standard,
        "rewrite": rewrite,
        "category": "naming",
    }


def _split_top_level(text, base):
    """Split text at top level commas. Returns the spans of the parts
    as offsets from base, so callers can rebuild the original line."""
    spans = []
    depth = 0
    quote = None
    escaped = False
    start = 0
    for index, ch in enumerate(text):
        if quote is not None:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == quote:
                quote = None
            continue
        if ch in "\"'":
            quote = ch
        elif ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        elif ch == "," and depth == 0:
            spans.append((base + start, base + index))
            start = index + 1
    spans.append((base + start, base + len(text)))
    return spans


def _default_span(text, start, end):
    """Find the span of the default text inside one parameter held in
    text[start:end]. Returns the span of the stripped default, or None
    when the parameter holds no default."""
    depth = 0
    quote = None
    escaped = False
    for index in range(start, end):
        ch = text[index]
        if quote is not None:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == quote:
                quote = None
            continue
        if ch in "\"'":
            quote = ch
        elif ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        elif ch == "=" and depth == 0:
            if index + 1 < end and text[index + 1] == "=":
                continue
            raw = text[index + 1:end]
            stripped = raw.strip()
            if not stripped:
                return None
            lead = len(raw) - len(raw.lstrip())
            return (index + 1 + lead, index + 1 + lead + len(stripped))
    return None


def _is_call_text(text):
    """Read a default text as a call. It is a call when it is a plain or
    dotted name followed by a parenthesised argument list that closes
    at the end of the text."""
    head = _CALL_HEAD.match(text)
    if not head:
        return False
    depth = 0
    quote = None
    escaped = False
    for index in range(head.end() - 1, len(text)):
        ch = text[index]
        if quote is not None:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == quote:
                quote = None
            continue
        if ch in "\"'":
            quote = ch
        elif ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                return index == len(text) - 1
    return False


def _call_default_rewrite(content, def_match):
    """Redraw a def line with every call default replaced by None.

    Returns the redrawn line, or None when the def line holds no call
    default the shape can see."""
    open_index = def_match.end() - 1
    depth = 0
    quote = None
    escaped = False
    close_index = None
    for index in range(open_index, len(content)):
        ch = content[index]
        if quote is not None:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == quote:
                quote = None
            continue
        if ch in "\"'":
            quote = ch
        elif ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                close_index = index
                break
    if close_index is None:
        return None
    params = content[open_index + 1:close_index]
    spans = []
    for start, end in _split_top_level(params, open_index + 1):
        span = _default_span(content, start, end)
        if span is None:
            continue
        default = content[span[0]:span[1]]
        if _MUTABLE_DEFAULT_WHOLE.match(default):
            continue
        if _is_call_text(default):
            spans.append(span)
    if not spans:
        return None
    redrawn = content
    for start, end in reversed(spans):
        redrawn = redrawn[:start] + "None" + redrawn[end:]
    return (
        redrawn
        + " and build each replaced default inside the body "
        "under a None check"
    )


def _class_attribute_objection(content):
    """Raise the class attribute objection for an assignment line, or
    return None when the line holds no single literal assignment."""
    annotated = _ANNOTATED_ASSIGNMENT.match(content)
    if annotated:
        name, rhs = annotated.group(1), annotated.group(3).strip()
    else:
        assignment = _ASSIGNMENT.match(content)
        if not assignment:
            return None
        name, rhs = assignment.group(1), assignment.group(2).strip()
    if not _is_single_literal(rhs):
        return None
    return {
        "flaw": f"The added line gives the class attribute {name} "
        "a mutable literal that every instance shares",
        "standard": "A mutable value that belongs to each instance is "
        "built in the initializer, so one instance changing it never "
        "changes it for every other instance",
        "rewrite": f"Move {name} into the initializer as "
        f"self.{name} = {rhs} and remove the class level assignment",
        "category": "structure",
    }


def _line_judgements(content, in_class_body):
    """Return the judgements one added line raises, as pairs of the
    shape name and the objection, in the order the contract states
    the shapes."""
    judgements = []

    bare = _BARE_EXCEPT.match(content)
    if bare:
        judgements.append(
            (
                "bare except",
                {
                    "flaw": "The added line catches every exception with "
                    "a bare except clause",
                    "standard": "A handler names the failures it expects "
                    "so the failures nobody expected still rise where "
                    "they can be seen",
                    "rewrite": _BARE_EXCEPT.sub(
                        r"\g<1>except Exception:", content, count=1
                    ),
                    "category": "smell",
                },
            )
        )

    if _PRINT_CALL.search(content):
        judgements.append(
            (
                "print call",
                {
                    "flaw": "The added line sends diagnostics through "
                    "a print call",
                    "standard": "Diagnostics travel through the logging "
                    "module so they can be levelled, filtered, and silenced",
                    "rewrite": _PRINT_CALL.sub("logger.debug", content, count=1),
                    "category": "smell",
                },
            )
        )

    def_match = _DEF_LINE.match(content)
    if def_match and _MUTABLE_DEFAULT.search(content):
        judgements.append(
            (
                "mutable default",
                {
                    "flaw": "The added def line gives a parameter a "
                    "mutable default that every call shares",
                    "standard": "A default that must be fresh on each "
                    "call is written as None and the value is built "
                    "inside the body",
                    "rewrite": _MUTABLE_DEFAULT.sub(
                        lambda m: m.group(1) + "None", content
                    ),
                    "category": "structure",
                },
            )
        )

    if def_match and def_match.group(1) in PLACEHOLDERS:
        judgements.append(
            (
                "placeholder name",
                _placeholder_objection(def_match.group(1), None, "function"),
            )
        )
    elif not def_match:
        annotated = _ANNOTATED_ASSIGNMENT.match(content)
        assignment = None if annotated else _ASSIGNMENT.match(content)
        if annotated and annotated.group(1) in PLACEHOLDERS:
            name = annotated.group(1)
            rhs = annotated.group(3).strip()
            drawn = _drawn_name(rhs)
            drawn_line = None
            if drawn is not None:
                drawn_line = re.sub(
                    r"^(\s*)" + re.escape(name) + r"(?=\s*:)",
                    r"\g<1>" + drawn,
                    content,
                    count=1,
                )
            judgements.append(
                ("placeholder name", _placeholder_objection(name, drawn_line, "value"))
            )
        elif assignment and assignment.group(1) in PLACEHOLDERS:
            name = assignment.group(1)
            rhs = assignment.group(2).strip()
            drawn = _drawn_name(rhs)
            drawn_line = None
            if drawn is not None:
                drawn_line = re.sub(
                    r"^(\s*)" + re.escape(name) + r"(?=\s*=)",
                    r"\g<1>" + drawn,
                    content,
                    count=1,
                )
            judgements.append(
                ("placeholder name", _placeholder_objection(name, drawn_line, "value"))
            )
        else:
            tuple_match = _TUPLE_ASSIGNMENT.match(content)
            if tuple_match:
                targets = [
                    target.strip()
                    for target in tuple_match.group(1).split(",")
                ]
                held = next(
                    (t for t in targets if t in PLACEHOLDERS), None
                )
                if held is not None:
                    judgements.append(
                        (
                            "placeholder name",
                            _placeholder_objection(held, None, "value"),
                        )
                    )

    if def_match:
        rewrite = _call_default_rewrite(content, def_match)
        if rewrite is not None:
            judgements.append(
                (
                    "call default",
                    {
                        "flaw": "The added def line gives a parameter a "
                        "default that is a call, so the call runs once "
                        "when the definition is read and every call that "
                        "omits the argument shares the one result",
                        "standard": "A default that must be computed on "
                        "each call is written as None and the value is "
                        "built inside the body under a None check",
                        "rewrite": rewrite,
                        "category": "structure",
                    },
                )
            )

    if in_class_body and not def_match:
        objection = _class_attribute_objection(content)
        if objection is not None:
            judgements.append(("class attribute", objection))

    return judgements


def _indent_of(content):
    return len(content) - len(content.lstrip(" "))


def _is_tuple_literal(text):
    """Read a default text as one tuple literal. It is one when it
    opens with a parenthesis and the opener closes exactly at the
    last character."""
    if not text or text[0] != "(":
        return False
    depth = 0
    quote = None
    escaped = False
    for index, ch in enumerate(text):
        if quote is not None:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == quote:
                quote = None
            continue
        if ch in "\"'":
            quote = ch
        elif ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
            if depth == 0:
                return index == len(text) - 1
    return False


def _shared_member_reading(content, def_match):
    """Return the reading for a def line holding a shared member
    default, or None when no parameter default of the line is a
    tuple literal holding a single list, dict, or set literal at
    its top level, under the third borderline pattern of CK-7."""
    open_index = def_match.end() - 1
    depth = 0
    quote = None
    escaped = False
    close_index = None
    for index in range(open_index, len(content)):
        ch = content[index]
        if quote is not None:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == quote:
                quote = None
            continue
        if ch in "\"'":
            quote = ch
        elif ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                close_index = index
                break
    if close_index is None:
        return None
    params = content[open_index + 1:close_index]
    for start, end in _split_top_level(params, open_index + 1):
        span = _default_span(content, start, end)
        if span is None:
            continue
        default = content[span[0]:span[1]]
        if not _is_tuple_literal(default):
            continue
        inner = default[1:-1]
        for member_start, member_end in _split_top_level(inner, 0):
            member = inner[member_start:member_end].strip()
            if not member or member[0] not in "[{":
                continue
            if not _is_single_literal(member):
                continue
            if member[0] == "[":
                return "default holds a list literal"
            if _is_dict_literal(member):
                return "default holds a dict literal"
            return "default holds a set literal"
    return None


def _line_flag(content):
    """Return the borderline flag one added line raises, as a pair of
    the pattern name and the reading the checker computed, or None
    when the line matches no borderline pattern of CK-7.

    The caller stands the flag only on a line that raised no
    judgement, so a flag never stands beside a shape judgement."""
    def_match = _DEF_LINE.match(content)

    if def_match and def_match.group(1) in NEAR_PLACEHOLDERS:
        return ("near placeholder name", "no drawn name")
    if not def_match:
        annotated = _ANNOTATED_ASSIGNMENT.match(content)
        assignment = None if annotated else _ASSIGNMENT.match(content)
        target = None
        drawn = None
        if annotated and annotated.group(1) in NEAR_PLACEHOLDERS:
            target = annotated.group(1)
            drawn = _drawn_name(annotated.group(3).strip())
        elif assignment and assignment.group(1) in NEAR_PLACEHOLDERS:
            target = assignment.group(1)
            drawn = _drawn_name(assignment.group(2).strip())
        else:
            tuple_match = _TUPLE_ASSIGNMENT.match(content)
            if tuple_match:
                targets = [
                    target.strip()
                    for target in tuple_match.group(1).split(",")
                ]
                if any(t in NEAR_PLACEHOLDERS for t in targets):
                    target = next(
                        t for t in targets if t in NEAR_PLACEHOLDERS
                    )
        if target is not None:
            reading = (
                f"drawn name {drawn}" if drawn is not None
                else "no drawn name"
            )
            return ("near placeholder name", reading)

    if _BROAD_HANDLER.match(content):
        return ("broad handler", "handler names BaseException")

    if def_match:
        reading = _shared_member_reading(content, def_match)
        if reading is not None:
            return ("shared member default", reading)

    return None


def _walk(text):
    """Walk the lines of a diff the way the checker reads them.

    Yields, for every line the walk reads, the number of the line
    counting every line of the text from one, whether the line is an
    added line, the content of the line without its marker, and
    whether the line sits directly in a class body. Removed lines
    and headers are never read, and the block stack is kept under
    the rule CK-5 states."""
    blocks = []
    for number, line in enumerate(text.splitlines(), start=1):
        if line.startswith("@@") or line.startswith("diff "):
            blocks = []
            continue
        if line.startswith("--- ") or line.startswith("+++ "):
            blocks = []
            continue
        if line.startswith("-") or line.startswith("+++"):
            continue
        if not line.startswith(("+", " ")):
            continue
        content = line[1:]
        if not content.strip():
            continue
        indent = _indent_of(content)
        while blocks and blocks[-1][0] >= indent:
            blocks.pop()
        in_class_body = bool(blocks) and blocks[-1][1] == "class"
        yield number, line.startswith("+"), content, in_class_body
        stripped = content.strip()
        if stripped.startswith("class ") and content.rstrip().endswith(":"):
            blocks.append((indent, "class"))
        elif _DEF_LINE.match(content):
            blocks.append((indent, "def"))


def check_diff_detailed(text):
    """Return the judgements raised by a unified diff held as text, as
    pairs of the shape name and the objection.

    Only added lines are judged. Judgements come back in line order
    and, on one line, in the order the contract states the shapes.
    The class context is read from the context lines and added lines
    of the diff itself, under the rule CK-5 states."""
    judgements = []
    for _number, is_added, content, in_class_body in _walk(text):
        if is_added:
            judgements.extend(_line_judgements(content, in_class_body))
    return judgements


def check_diff_flagged(text):
    """Return the judgements and the borderline flags of a diff.

    The judgements are the pairs check_diff_detailed returns. The
    flags are one object per flagged line, in line order, holding
    the number of the line in the diff text, the pattern it
    matched, the line as written, and the reading the checker
    computed for it, under CK-7. A flag stands only on a line that
    raised no judgement."""
    judgements = []
    flags = []
    for number, is_added, content, in_class_body in _walk(text):
        if not is_added:
            continue
        line_judgements = _line_judgements(content, in_class_body)
        judgements.extend(line_judgements)
        if line_judgements:
            continue
        flag = _line_flag(content)
        if flag is not None:
            pattern, reading = flag
            flags.append(
                {
                    "line": number,
                    "pattern": pattern,
                    "content": content,
                    "reading": reading,
                }
            )
    return judgements, flags


def check_diff_text(text):
    """Return the objections raised by a unified diff held as text.

    Only added lines are read. Objections come back in line order and,
    on one line, in the order the contract states the shapes."""
    return [objection for _shape, objection in check_diff_detailed(text)]
