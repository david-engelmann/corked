"""The one deterministic rendering of a review, shared by every command."""

from .checker import SHAPES
from .review import _holds_unpaired_surrogate, validate_review, verdict_for


def _label_for(review):
    if isinstance(review, dict):
        label = review.get("diff_label")
        if isinstance(label, str) and label.strip():
            if _holds_unpaired_surrogate(label):
                return "unnamed review"
            return label
    return "unnamed review"


def render_defects(label, defects):
    """Render an invalid verdict with its defects named."""
    lines = [f"Review of {label}", "Verdict, invalid"]
    lines.append(f"Defects, {len(defects)}")
    lines.extend(f"Defect, {defect}" for defect in defects)
    return "\n".join(lines) + "\n"


def render_review(review):
    """Render a review. The same review always renders byte identical."""
    defects = validate_review(review)
    label = _label_for(review)
    if defects:
        return render_defects(label, defects)
    verdict = verdict_for(review)
    objections = review["objections"]
    lines = [f"Review of {label}", f"Verdict, {verdict}"]
    lines.append(f"Objections, {len(objections)}")
    for index, objection in enumerate(objections, start=1):
        lines.append("")
        lines.append(f"Objection {index}, {objection['category']}")
        lines.append(f"Flaw, {objection['flaw']}")
        lines.append(f"Standard, {objection['standard']}")
        lines.append(f"Rewrite, {objection['rewrite']}")
    return "\n".join(lines) + "\n"


def render_set_defects(defects):
    """Render a malformed change set with its defects named."""
    lines = ["Change set review", "Verdict, invalid"]
    lines.append(f"Defects, {len(defects)}")
    lines.extend(f"Defect, {defect}" for defect in defects)
    return "\n".join(lines) + "\n"


def render_set_review(results):
    """Render a set review from the per entry judgements.

    The results are pairs of a file name and the judgements its diff
    raised, as pairs of the shape name and the objection. The same
    results always render byte identical."""
    lines = ["Change set review", f"Files, {len(results)}"]
    total = 0
    counts = {}
    worst = []
    for name, judgements in results:
        lines.append("")
        lines.append(f"File, {name}")
        if not judgements:
            lines.append("Clean")
            continue
        lines.append(f"Judgements, {len(judgements)}")
        total += len(judgements)
        worst.append((name, len(judgements)))
        for shape, objection in judgements:
            counts[shape] = counts.get(shape, 0) + 1
        for index, (_shape, objection) in enumerate(judgements, start=1):
            lines.append("")
            lines.append(f"Judgement {index}, {objection['category']}")
            lines.append(f"Flaw, {objection['flaw']}")
            lines.append(f"Standard, {objection['standard']}")
            lines.append(f"Rewrite, {objection['rewrite']}")
    lines.append("")
    lines.append("Summary")
    lines.append(f"Judgements, {total}")
    for shape in SHAPES:
        if counts.get(shape):
            lines.append(f"Shape, {shape}, {counts[shape]}")
    if worst:
        lines.append("Files with judgements, worst first")
        ordered = sorted(worst, key=lambda item: (-item[1], item[0]))
        for name, count in ordered:
            lines.append(f"Worst, {name}, {count}")
    return "\n".join(lines) + "\n"
