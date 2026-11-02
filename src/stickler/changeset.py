"""The set level review. A change set holds named diffs, and the set
check runs the checker over every entry."""

from .checker import check_diff_detailed


def _holds_unpaired_surrogate(text):
    """Read a string for an unpaired surrogate. Such a string cannot
    be encoded as UTF-8 text and so cannot be printed."""
    try:
        text.encode("utf-8")
    except UnicodeEncodeError:
        return True
    return False


def load_changeset(value):
    """Return the list of defects found in a change set. Never raises."""
    defects = []
    if not isinstance(value, dict):
        return ["change set must be a JSON object"]
    if "files" not in value:
        return ["files is missing"]
    files = value["files"]
    if not isinstance(files, list):
        return ["files must be a list"]
    for index, entry in enumerate(files):
        where = f"files[{index}]"
        if not isinstance(entry, dict):
            defects.append(f"{where} must be an object")
            continue
        if "file" not in entry:
            defects.append(f"{where}.file is missing")
        elif not isinstance(entry["file"], str) or not entry["file"].strip():
            defects.append(f"{where}.file must be a non empty string")
        elif _holds_unpaired_surrogate(entry["file"]):
            defects.append(f"{where}.file holds an unpaired surrogate")
        if "diff" not in entry:
            defects.append(f"{where}.diff is missing")
        elif not isinstance(entry["diff"], str):
            defects.append(f"{where}.diff must be a string")
        elif _holds_unpaired_surrogate(entry["diff"]):
            defects.append(f"{where}.diff holds an unpaired surrogate")
    return defects


def check_changeset(value):
    """Run the checker over every entry of a valid change set.

    Returns one pair per entry, in the order given, of the file name
    and the judgements its diff raises as pairs of the shape name and
    the objection."""
    return [
        (entry["file"], check_diff_detailed(entry["diff"]))
        for entry in value["files"]
    ]
