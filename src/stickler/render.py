"""The one deterministic rendering of a review, shared by every command."""

from .checker import SHAPES
from .review import (
    _holds_line_break,
    _holds_unpaired_surrogate,
    validate_review,
    verdict_for,
)
from .verdicts import is_contested, line_outcome, reason_for, verdict_outcome


def _label_for(review):
    if isinstance(review, dict):
        label = review.get("diff_label")
        if isinstance(label, str) and label.strip():
            if _holds_unpaired_surrogate(label) or _holds_line_break(label):
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
    """Render a set review from the per entry results.

    The results are pairs of a file name and the judgements its diff
    raised, as pairs of the shape name and the objection, or triples
    whose third element holds the borderline blocks of the entry
    under CK-8. When a flagged line is contested under CK-9 a
    contested section stands before the summary and the summary
    counts the contested lines. The same results always render byte
    identical, and results holding no flags render exactly as pairs
    alone render.
    """
    lines = ["Change set review", f"Files, {len(results)}"]
    total = 0
    counts = {}
    worst = []
    flag_total = 0
    judged_lines = {"stickler": 0, "clean": 0, "unjudged": 0}
    contested = []
    for result in results:
        name, judgements = result[0], result[1]
        blocks = result[2] if len(result) > 2 else []
        lines.append("")
        lines.append(f"File, {name}")
        if not judgements and not blocks:
            lines.append("Clean")
            continue
        if judgements:
            lines.append(f"Judgements, {len(judgements)}")
            total += len(judgements)
            worst.append((name, len(judgements)))
            for shape, objection in judgements:
                counts[shape] = counts.get(shape, 0) + 1
            for index, (_shape, objection) in enumerate(
                judgements, start=1
            ):
                lines.append("")
                lines.append(
                    f"Judgement {index}, {objection['category']}"
                )
                lines.append(f"Flaw, {objection['flaw']}")
                lines.append(f"Standard, {objection['standard']}")
                lines.append(f"Rewrite, {objection['rewrite']}")
        for index, block in enumerate(blocks, start=1):
            flag = block["flag"]
            entries = block["verdicts"]
            flag_total += 1
            outcome = line_outcome(entries)
            judged_lines[outcome] += 1
            if is_contested(entries):
                contested.append((name, index, flag, entries))
            lines.append("")
            lines.append(f"Borderline {index}, {flag['pattern']}")
            lines.append(f"Line, {flag['content']}")
            lines.append(f"Shape reading, {flag['reading']}")
            for entry in entries:
                word = verdict_outcome(entry)
                if word == "abstained":
                    lines.append(
                        f"Verdict, {entry['model']}, abstained"
                    )
                else:
                    lines.append(
                        f"Verdict, {entry['model']}, {entry['choice']}"
                    )
                lines.append(f"Reason, {reason_for(entry)}")
            if outcome == "unjudged":
                lines.append("Unjudged")
    if contested:
        lines.append("")
        lines.append("Contested lines")
        for name, index, flag, entries in contested:
            lines.append("")
            lines.append(f"File, {name}")
            lines.append(f"Borderline {index}, {flag['pattern']}")
            lines.append(f"Line, {flag['content']}")
            for entry in entries:
                lines.append(
                    f"Verdict, {entry['model']}, "
                    f"{verdict_outcome(entry)}"
                )
    lines.append("")
    lines.append("Summary")
    lines.append(f"Judgements, {total}")
    for shape in SHAPES:
        if counts.get(shape):
            lines.append(f"Shape, {shape}, {counts[shape]}")
    if flag_total:
        lines.append(f"Borderline, {flag_total}")
        lines.append(
            f"Verdict lines, stickler {judged_lines['stickler']}, "
            f"clean {judged_lines['clean']}"
        )
        lines.append(f"Unjudged, {judged_lines['unjudged']}")
        lines.append(f"Contested, {len(contested)}")
    if worst:
        lines.append("Files with judgements, worst first")
        ordered = sorted(worst, key=lambda item: (-item[1], item[0]))
        for name, count in ordered:
            lines.append(f"Worst, {name}, {count}")
    return "\n".join(lines) + "\n"
