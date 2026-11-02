"""Pins for the rendering shared by every command, and the golden files."""

import json
import pathlib

from stickler.changeset import check_changeset
from stickler.render import render_review, render_set_defects, render_set_review

ROOT = pathlib.Path(__file__).resolve().parent.parent

OBJECTION = {
    "flaw": "The name says nothing about what the value holds",
    "standard": "A name says what its value holds",
    "rewrite": "orders = load_orders()",
    "category": "naming",
}


def test_accepted_rendering_is_exact():
    review = {"diff_label": "tidy change", "objections": []}
    assert render_review(review) == (
        "Review of tidy change\n"
        "Verdict, accepted\n"
        "Objections, 0\n"
    )


def test_sent_back_rendering_is_exact():
    review = {"diff_label": "rough change", "objections": [OBJECTION]}
    assert render_review(review) == (
        "Review of rough change\n"
        "Verdict, sent back\n"
        "Objections, 1\n"
        "\n"
        "Objection 1, naming\n"
        "Flaw, The name says nothing about what the value holds\n"
        "Standard, A name says what its value holds\n"
        "Rewrite, orders = load_orders()\n"
    )


def test_rendering_is_deterministic():
    review = {"diff_label": "rough change", "objections": [OBJECTION]}
    assert render_review(review) == render_review(review)


def test_invalid_rendering_names_every_defect():
    review = {"objections": [{"flaw": "x"}]}
    rendered = render_review(review)
    assert "Verdict, invalid\n" in rendered
    assert "Defect, diff_label is missing\n" in rendered
    assert "Defect, objections[0].standard is missing\n" in rendered
    assert "Defect, objections[0].rewrite is missing\n" in rendered
    assert "Defect, objections[0].category is missing\n" in rendered


def test_a_review_whose_label_holds_a_surrogate_renders_unnamed():
    review = {"diff_label": "\ud800x", "objections": []}
    rendered = render_review(review)
    assert rendered.startswith("Review of unnamed review\n")
    assert "Verdict, invalid\n" in rendered
    assert "Defect, diff_label holds an unpaired surrogate\n" in rendered
    rendered.encode("utf-8")


def test_recorded_review_matches_its_golden_rendering():
    recorded = json.loads(
        (ROOT / "examples" / "recorded_review.json").read_text(encoding="utf-8")
    )
    golden = (ROOT / "examples" / "recorded_review.txt").read_text(encoding="utf-8")
    assert render_review(recorded) == golden


def set_entry(name, *added_lines):
    lines = ["--- a/x.py", "+++ b/x.py", "@@ -1,1 +1,1 @@"]
    lines.extend("+" + line for line in added_lines)
    return {"file": name, "diff": "\n".join(lines) + "\n"}


def test_a_clean_set_rendering_is_exact():
    results = check_changeset({"files": [set_entry("a.py", "    logger.debug(x)")]})
    assert render_set_review(results) == (
        "Change set review\n"
        "Files, 1\n"
        "\n"
        "File, a.py\n"
        "Clean\n"
        "\n"
        "Summary\n"
        "Judgements, 0\n"
    )


def test_a_set_rendering_is_exact_with_a_judgement():
    results = check_changeset({"files": [set_entry("a.py", "    print(total)")]})
    assert render_set_review(results) == (
        "Change set review\n"
        "Files, 1\n"
        "\n"
        "File, a.py\n"
        "Judgements, 1\n"
        "\n"
        "Judgement 1, smell\n"
        "Flaw, The added line sends diagnostics through a print call\n"
        "Standard, Diagnostics travel through the logging module so they "
        "can be levelled, filtered, and silenced\n"
        "Rewrite,     logger.debug(total)\n"
        "\n"
        "Summary\n"
        "Judgements, 1\n"
        "Shape, print call, 1\n"
        "Files with judgements, worst first\n"
        "Worst, a.py, 1\n"
    )


def test_the_summary_counts_by_shape_in_the_stated_shape_order():
    value = {"files": [
        set_entry("a.py", "def stamp(at=now()):", "    print(total)"),
        set_entry("b.py", "    except:"),
    ]}
    rendered = render_set_review(check_changeset(value))
    shape_lines = [
        line for line in rendered.splitlines() if line.startswith("Shape, ")
    ]
    assert shape_lines == [
        "Shape, bare except, 1",
        "Shape, print call, 1",
        "Shape, call default, 1",
    ]


def test_the_worst_first_order_breaks_ties_by_file_name():
    value = {"files": [
        set_entry("zeta.py", "    print(total)"),
        set_entry("mid.py", "    print(total)", "    except:"),
        set_entry("alpha.py", "    print(total)"),
        set_entry("clean.py", "    logger.debug(total)"),
    ]}
    rendered = render_set_review(check_changeset(value))
    worst_lines = [
        line for line in rendered.splitlines() if line.startswith("Worst, ")
    ]
    assert worst_lines == [
        "Worst, mid.py, 2",
        "Worst, alpha.py, 1",
        "Worst, zeta.py, 1",
    ]


def test_the_set_rendering_is_deterministic():
    value = {"files": [set_entry("a.py", "    print(total)")]}
    results = check_changeset(value)
    assert render_set_review(results) == render_set_review(results)


def test_the_example_change_set_matches_its_golden_rendering():
    value = json.loads(
        (ROOT / "examples" / "change_set.json").read_text(encoding="utf-8")
    )
    golden = (ROOT / "examples" / "change_set_review.txt").read_text(
        encoding="utf-8"
    )
    assert render_set_review(check_changeset(value)) == golden


def test_a_malformed_set_rendering_names_every_defect():
    rendered = render_set_defects(["files is missing", "files[0] must be an object"])
    assert rendered == (
        "Change set review\n"
        "Verdict, invalid\n"
        "Defects, 2\n"
        "Defect, files is missing\n"
        "Defect, files[0] must be an object\n"
    )
