"""The review core. A review is validated and given a verdict here."""

CATEGORIES = ("naming", "structure", "smell")
OBJECTION_FIELDS = ("flaw", "standard", "rewrite", "category")
TEXT_FIELDS = ("flaw", "standard", "rewrite")


def _is_blank(value):
    return not isinstance(value, str) or not value.strip()


def _holds_unpaired_surrogate(text):
    """Read a string for an unpaired surrogate. Such a string cannot
    be encoded as UTF-8 text and so cannot be printed."""
    try:
        text.encode("utf-8")
    except UnicodeEncodeError:
        return True
    return False


def _holds_line_break(text):
    return "\n" in text or "\r" in text


def validate_review(review):
    """Return the list of defects found in a review. Never raises."""
    defects = []
    if not isinstance(review, dict):
        return ["review must be a JSON object"]
    if "diff_label" not in review:
        defects.append("diff_label is missing")
    elif _is_blank(review["diff_label"]):
        defects.append("diff_label must be a non empty string")
    elif _holds_unpaired_surrogate(review["diff_label"]):
        defects.append("diff_label holds an unpaired surrogate")
    elif _holds_line_break(review["diff_label"]):
        defects.append("diff_label holds a line break")
    if "objections" not in review:
        defects.append("objections is missing")
        return defects
    objections = review["objections"]
    if not isinstance(objections, list):
        defects.append("objections must be a list")
        return defects
    for index, objection in enumerate(objections):
        where = f"objections[{index}]"
        if not isinstance(objection, dict):
            defects.append(f"{where} must be an object")
            continue
        for field in OBJECTION_FIELDS:
            if field not in objection:
                defects.append(f"{where}.{field} is missing")
        for field in TEXT_FIELDS:
            if field in objection and _is_blank(objection[field]):
                defects.append(f"{where}.{field} must be a non empty string")
            elif (
                field in objection
                and isinstance(objection[field], str)
                and _holds_unpaired_surrogate(objection[field])
            ):
                defects.append(
                    f"{where}.{field} holds an unpaired surrogate"
                )
        if "category" in objection and objection["category"] not in CATEGORIES:
            defects.append(
                f"{where}.category must be one of naming, structure, smell"
            )
    return defects


def verdict_for(review):
    """Return the verdict for a review. One of accepted, sent back, invalid."""
    if validate_review(review):
        return "invalid"
    if review["objections"]:
        return "sent back"
    return "accepted"
