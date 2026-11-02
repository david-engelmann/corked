"""The built in checker. It reads a unified diff as text and raises the
four shapes the contract names, as complete objections."""

import keyword
import re

PLACEHOLDERS = ("data2", "temp2", "foo", "bar", "stuff", "thing")

_BARE_EXCEPT = re.compile(r"^(\s*)except\s*:")
_PRINT_CALL = re.compile(r"(?<![\w.])print(?=\s*\()")
_MUTABLE_DEFAULT = re.compile(r"(=\s*)(\[\s*\]|\{\s*\}|set\s*\(\s*\))")
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


def _is_dict_literal(rhs):
    """Read a brace literal as text. It is a dict when it is empty or
    holds a colon at its top level, and a set literal otherwise."""
    depth = 0
    quote = None
    seen_content = False
    for ch in rhs[1:]:
        if quote is not None:
            if ch == quote:
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


def check_diff_text(text):
    """Return the objections raised by a unified diff held as text.

    Only added lines are read. Objections come back in line order and,
    on one line, in the order the contract states the shapes."""
    objections = []
    for line in text.splitlines():
        if not line.startswith("+") or line.startswith("+++"):
            continue
        content = line[1:]

        bare = _BARE_EXCEPT.match(content)
        if bare:
            objections.append(
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
                }
            )

        if _PRINT_CALL.search(content):
            objections.append(
                {
                    "flaw": "The added line sends diagnostics through "
                    "a print call",
                    "standard": "Diagnostics travel through the logging "
                    "module so they can be levelled, filtered, and silenced",
                    "rewrite": _PRINT_CALL.sub("logger.debug", content, count=1),
                    "category": "smell",
                }
            )

        def_match = _DEF_LINE.match(content)
        if def_match and _MUTABLE_DEFAULT.search(content):
            objections.append(
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
                }
            )

        if def_match and def_match.group(1) in PLACEHOLDERS:
            objections.append(
                _placeholder_objection(def_match.group(1), None, "function")
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
                objections.append(
                    _placeholder_objection(name, drawn_line, "value")
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
                objections.append(
                    _placeholder_objection(name, drawn_line, "value")
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
                        objections.append(
                            _placeholder_objection(held, None, "value")
                        )
    return objections
