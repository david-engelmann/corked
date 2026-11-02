"""The set level review. A change set holds named diffs, and the set
check runs the checker over every entry. An entry may also hold
recorded verdicts on its borderline lines, read under CK-8."""

import math

from .checker import BORDERLINE_PATTERNS, check_diff_detailed, check_diff_flagged
from .verdicts import LABELS

VERDICT_FIELDS = ("line", "pattern", "model", "choice", "confidence")


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


def _confidence_is_refused(confidence):
    """Return whether a verdict confidence figure is refused.

    The figure is a number and not a boolean when this reading
    runs, and it is refused when it is not a finite number from
    0 to 1. An integer figure is read by the range as it stands,
    so an integer of any size is refused by the range itself and
    is never converted to a float to test its finiteness, under
    FIX-CK-16. A float figure is refused when it is not finite
    or when it stands outside the range.
    """
    if isinstance(confidence, int):
        return not 0 <= confidence <= 1
    return not math.isfinite(confidence) or not (
        0 <= confidence <= 1
    )


def _validate_verdicts(index, entry, defects):
    """Name the defects of one entry's recorded verdicts.

    The verdicts are read only for an entry whose file and diff
    fields drew no defect, so the flags of the entry can be
    computed from its diff. No defect names the value it refuses,
    only the position of the verdict and the rule it breaks.
    """
    where = f"files[{index}].verdicts"
    verdicts = entry["verdicts"]
    if not isinstance(verdicts, list):
        defects.append(f"{where} must be a list")
        return
    _judgements, flags = check_diff_flagged(entry["diff"])
    by_line = {flag["line"]: flag for flag in flags}
    seen = []
    for position, verdict in enumerate(verdicts):
        vwhere = f"{where}[{position}]"
        if not isinstance(verdict, dict):
            defects.append(f"{vwhere} must be an object")
            continue
        for field in VERDICT_FIELDS:
            if field not in verdict:
                defects.append(f"{vwhere}.{field} is missing")
        line = verdict.get("line")
        line_ok = (
            isinstance(line, int)
            and not isinstance(line, bool)
            and line >= 1
        )
        if "line" in verdict and not line_ok:
            defects.append(f"{vwhere}.line must be a positive integer")
        pattern = verdict.get("pattern")
        pattern_ok = pattern in BORDERLINE_PATTERNS
        if "pattern" in verdict and not pattern_ok:
            defects.append(
                f"{vwhere}.pattern must be one of "
                "near placeholder name, broad handler, "
                "shared member default"
            )
        model = verdict.get("model")
        model_ok = isinstance(model, str) and bool(model.strip())
        if "model" in verdict:
            if not model_ok:
                defects.append(
                    f"{vwhere}.model must be a non empty string"
                )
            elif _holds_unpaired_surrogate(model):
                defects.append(
                    f"{vwhere}.model holds an unpaired surrogate"
                )
                model_ok = False
            elif _holds_line_break(model):
                defects.append(f"{vwhere}.model holds a line break")
                model_ok = False
        if "choice" in verdict and verdict["choice"] not in LABELS:
            defects.append(
                f"{vwhere}.choice must be one of stickler, clean"
            )
        if "confidence" in verdict:
            confidence = verdict["confidence"]
            if (
                not isinstance(confidence, (int, float))
                or isinstance(confidence, bool)
            ):
                defects.append(f"{vwhere}.confidence must be a number")
            elif _confidence_is_refused(confidence):
                defects.append(
                    f"{vwhere}.confidence must be from 0 to 1"
                )
        if line_ok and pattern_ok:
            flag = by_line.get(line)
            if flag is None or flag["pattern"] != pattern:
                defects.append(f"{vwhere} names no borderline line")
            elif model_ok:
                key = (line, model)
                if key in seen:
                    defects.append(
                        f"{vwhere} repeats a verdict for the same "
                        "line and model"
                    )
                else:
                    seen.append(key)


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
        before = len(defects)
        if "file" not in entry:
            defects.append(f"{where}.file is missing")
        elif not isinstance(entry["file"], str) or not entry["file"].strip():
            defects.append(f"{where}.file must be a non empty string")
        elif _holds_unpaired_surrogate(entry["file"]):
            defects.append(f"{where}.file holds an unpaired surrogate")
        elif _holds_line_break(entry["file"]):
            defects.append(f"{where}.file holds a line break")
        if "diff" not in entry:
            defects.append(f"{where}.diff is missing")
        elif not isinstance(entry["diff"], str):
            defects.append(f"{where}.diff must be a string")
        elif _holds_unpaired_surrogate(entry["diff"]):
            defects.append(f"{where}.diff holds an unpaired surrogate")
        if "verdicts" in entry and len(defects) == before:
            _validate_verdicts(index, entry, defects)
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


def review_changeset(value):
    """Review every entry of a valid change set, verdicts included.

    Returns one triple per entry, in the order given, of the file
    name, the judgements its diff raises, and its borderline
    blocks. A block holds one flag of the entry and the recorded
    verdicts standing on that flag, in the order the verdicts
    stand in the entry, under CK-8.
    """
    results = []
    for entry in value["files"]:
        judgements, flags = check_diff_flagged(entry["diff"])
        verdicts = entry.get("verdicts") or []
        blocks = []
        for flag in flags:
            standing = [
                verdict
                for verdict in verdicts
                if verdict["line"] == flag["line"]
            ]
            blocks.append({"flag": flag, "verdicts": standing})
        results.append((entry["file"], judgements, blocks))
    return results
