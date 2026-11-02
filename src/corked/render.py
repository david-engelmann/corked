"""The one deterministic rendering of a review, shared by both commands."""

from .review import validate_review, verdict_for


def _label_for(review):
    if isinstance(review, dict):
        label = review.get("diff_label")
        if isinstance(label, str) and label.strip():
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
