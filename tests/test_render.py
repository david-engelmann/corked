"""Pins for the rendering shared by both commands, and the golden file."""

import json
import pathlib

from corked.render import render_review

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


def test_recorded_review_matches_its_golden_rendering():
    recorded = json.loads(
        (ROOT / "examples" / "recorded_review.json").read_text(encoding="utf-8")
    )
    golden = (ROOT / "examples" / "recorded_review.txt").read_text(encoding="utf-8")
    assert render_review(recorded) == golden
