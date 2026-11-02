"""Pins for CK-1 and CK-2. The verdict rule and the completeness rule."""

from corked.review import validate_review, verdict_for

COMPLETE = {
    "flaw": "The name says nothing about what the value holds",
    "standard": "A name says what its value holds",
    "rewrite": "orders = load_orders()",
    "category": "naming",
}


def review(objections):
    return {"diff_label": "change under review", "objections": objections}


def test_empty_objections_validate_clean_and_accept():
    assert validate_review(review([])) == []
    assert verdict_for(review([])) == "accepted"


def test_one_complete_objection_sends_back():
    assert validate_review(review([COMPLETE])) == []
    assert verdict_for(review([COMPLETE])) == "sent back"


def test_verdict_flips_on_one_objection_added_and_removed():
    assert verdict_for(review([])) == "accepted"
    assert verdict_for(review([COMPLETE])) == "sent back"
    assert verdict_for(review([])) == "accepted"


def test_missing_diff_label_is_named():
    assert validate_review({"objections": []}) == ["diff_label is missing"]


def test_blank_diff_label_is_named():
    bad = {"diff_label": "   ", "objections": []}
    assert validate_review(bad) == ["diff_label must be a non empty string"]


def test_non_string_diff_label_is_named():
    bad = {"diff_label": 7, "objections": []}
    assert validate_review(bad) == ["diff_label must be a non empty string"]


def test_a_diff_label_holding_an_unpaired_surrogate_is_named():
    bad = {"diff_label": "\ud800x", "objections": []}
    assert validate_review(bad) == ["diff_label holds an unpaired surrogate"]
    assert verdict_for(bad) == "invalid"


def test_each_text_field_holding_an_unpaired_surrogate_is_named():
    for field in ("flaw", "standard", "rewrite"):
        objection = dict(COMPLETE, **{field: "bad \ud800 text"})
        assert validate_review(review([objection])) == [
            f"objections[0].{field} holds an unpaired surrogate"
        ], field


def test_missing_objections_is_named():
    assert validate_review({"diff_label": "x"}) == ["objections is missing"]


def test_objections_of_wrong_type_is_named():
    bad = {"diff_label": "x", "objections": "none"}
    assert validate_review(bad) == ["objections must be a list"]


def test_non_object_review_is_named_and_never_raises():
    for handed in (None, 7, "review", ["objections"], True):
        assert validate_review(handed) == ["review must be a JSON object"]
        assert verdict_for(handed) == "invalid"


def test_each_missing_objection_field_is_named():
    for field in ("flaw", "standard", "rewrite", "category"):
        objection = {k: v for k, v in COMPLETE.items() if k != field}
        assert validate_review(review([objection])) == [
            f"objections[0].{field} is missing"
        ], field


def test_each_blank_text_field_is_named():
    for field in ("flaw", "standard", "rewrite"):
        objection = dict(COMPLETE, **{field: "  "})
        assert validate_review(review([objection])) == [
            f"objections[0].{field} must be a non empty string"
        ], field


def test_unknown_category_is_named():
    objection = dict(COMPLETE, category="style")
    assert validate_review(review([objection])) == [
        "objections[0].category must be one of naming, structure, smell"
    ]


def test_every_named_category_is_accepted():
    for category in ("naming", "structure", "smell"):
        objection = dict(COMPLETE, category=category)
        assert validate_review(review([objection])) == [], category


def test_incomplete_second_objection_is_named_with_its_position():
    objection = {k: v for k, v in COMPLETE.items() if k != "rewrite"}
    defects = validate_review(review([COMPLETE, objection]))
    assert defects == ["objections[1].rewrite is missing"]


def test_objection_of_wrong_type_is_named():
    assert validate_review(review(["not an objection"])) == [
        "objections[0] must be an object"
    ]


def test_incomplete_objection_makes_the_verdict_invalid_not_sent_back():
    objection = {k: v for k, v in COMPLETE.items() if k != "flaw"}
    assert verdict_for(review([objection])) == "invalid"
