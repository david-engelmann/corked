"""Pins for CK-6. The change set loader and the set check."""

from stickler.changeset import (
    check_changeset,
    load_changeset,
    review_changeset,
)


def entry(name, *added_lines):
    lines = ["--- a/x.py", "+++ b/x.py", "@@ -1,1 +1,1 @@"]
    lines.extend("+" + line for line in added_lines)
    return {"file": name, "diff": "\n".join(lines) + "\n"}


def test_a_valid_change_set_loads_clean():
    value = {"files": [entry("a.py", "    logger.debug(total)")]}
    assert load_changeset(value) == []


def test_an_empty_files_list_loads_clean():
    assert load_changeset({"files": []}) == []
    assert check_changeset({"files": []}) == []


def test_a_non_object_change_set_is_named():
    assert load_changeset([1, 2]) == ["change set must be a JSON object"]
    assert load_changeset("nope") == ["change set must be a JSON object"]


def test_a_missing_files_field_is_named():
    assert load_changeset({}) == ["files is missing"]


def test_a_non_list_files_field_is_named():
    assert load_changeset({"files": "a.py"}) == ["files must be a list"]


def test_a_non_object_entry_is_named_with_its_position():
    value = {"files": [entry("a.py"), "oops"]}
    assert load_changeset(value) == ["files[1] must be an object"]


def test_a_missing_file_field_is_named_with_its_position():
    value = {"files": [{"diff": "x"}]}
    assert load_changeset(value) == ["files[0].file is missing"]


def test_a_blank_file_name_is_named():
    value = {"files": [{"file": "  ", "diff": ""}]}
    assert load_changeset(value) == [
        "files[0].file must be a non empty string"
    ]


def test_a_non_string_file_name_is_named():
    value = {"files": [{"file": ["a.py"], "diff": ""}]}
    assert load_changeset(value) == [
        "files[0].file must be a non empty string"
    ]


def test_a_file_name_holding_an_unpaired_surrogate_is_named():
    value = {"files": [{"file": "\ud800x.py", "diff": ""}]}
    assert load_changeset(value) == [
        "files[0].file holds an unpaired surrogate"
    ]


def test_a_file_name_holding_a_newline_is_named():
    value = {"files": [{"file": "evil\nForged, 99", "diff": ""}]}
    assert load_changeset(value) == [
        "files[0].file holds a line break"
    ]


def test_a_file_name_holding_a_carriage_return_is_named():
    value = {"files": [{"file": "evil\rForged, 99", "diff": ""}]}
    assert load_changeset(value) == [
        "files[0].file holds a line break"
    ]


def test_verdicts_are_not_read_when_the_file_name_holds_a_line_break():
    entry_with_verdicts = {
        "file": "evil\nForged, 99",
        "diff": "",
        "verdicts": "not a list",
    }
    assert load_changeset({"files": [entry_with_verdicts]}) == [
        "files[0].file holds a line break"
    ]


def test_a_diff_holding_an_unpaired_surrogate_is_named():
    value = {"files": [{"file": "a.py", "diff": "+print('\ud800')"}]}
    assert load_changeset(value) == [
        "files[0].diff holds an unpaired surrogate"
    ]


def test_a_missing_diff_field_is_named():
    value = {"files": [{"file": "a.py"}]}
    assert load_changeset(value) == ["files[0].diff is missing"]


def test_a_non_string_diff_is_named():
    value = {"files": [{"file": "a.py", "diff": 7}]}
    assert load_changeset(value) == ["files[0].diff must be a string"]


def test_an_empty_diff_is_a_valid_clean_entry():
    value = {"files": [{"file": "a.py", "diff": ""}]}
    assert load_changeset(value) == []
    assert check_changeset(value) == [("a.py", [])]


def test_the_loader_never_raises_on_hostile_values():
    for value in (None, 7, 2.5, True, [], {"files": None},
                  {"files": [None, {"file": [], "diff": {}}]}):
        defects = load_changeset(value)
        assert isinstance(defects, list)
        assert defects


def test_the_set_check_reports_judgements_per_entry_in_order():
    value = {"files": [
        entry("a.py", "    print(total)"),
        entry("b.py", "    logger.debug(total)"),
        entry("c.py", "    except:"),
    ]}
    results = check_changeset(value)
    assert [name for name, _judgements in results] == ["a.py", "b.py", "c.py"]
    assert [len(judgements) for _name, judgements in results] == [1, 0, 1]
    assert results[0][1][0][0] == "print call"
    assert results[2][1][0][0] == "bare except"


def test_the_set_check_carries_the_shape_names_of_the_new_shapes():
    diff = (
        "--- a/x.py\n+++ b/x.py\n@@ -1,2 +1,3 @@\n"
        "+def stamp(at=now()):\n"
    )
    value = {"files": [{"file": "a.py", "diff": diff}]}
    results = check_changeset(value)
    assert [shape for shape, _objection in results[0][1]] == ["call default"]


ONE_FLAG_DIFF = (
    "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,1 @@\n+data = load_entries()\n"
)


def one_flag_set(verdicts):
    return {
        "files": [
            {"file": "x.py", "diff": ONE_FLAG_DIFF, "verdicts": verdicts}
        ]
    }


def good_verdict(**overrides):
    verdict = {
        "line": 4,
        "pattern": "near placeholder name",
        "model": "jev-1.13.0",
        "choice": "stickler",
        "confidence": 0.78,
    }
    verdict.update(overrides)
    return verdict


def test_verdicts_stand_beside_file_and_diff():
    assert load_changeset(one_flag_set([good_verdict()])) == []


def test_two_models_may_stand_on_one_line():
    verdicts = [
        good_verdict(),
        good_verdict(model="openai/gpt-6-luna", choice="clean"),
    ]
    assert load_changeset(one_flag_set(verdicts)) == []


def test_fields_beyond_the_five_are_not_read():
    assert (
        load_changeset(
            one_flag_set([good_verdict(reason="invented words")])
        )
        == []
    )


def test_verdicts_must_be_a_list():
    assert load_changeset(one_flag_set("none")) == [
        "files[0].verdicts must be a list"
    ]


def test_a_verdict_must_be_an_object():
    assert load_changeset(one_flag_set([7])) == [
        "files[0].verdicts[0] must be an object"
    ]


def test_a_missing_verdict_field_is_named():
    verdict = good_verdict()
    del verdict["confidence"]
    assert load_changeset(one_flag_set([verdict])) == [
        "files[0].verdicts[0].confidence is missing"
    ]


def test_the_verdict_line_must_be_a_positive_integer():
    for bad in (0, -1, "4", True, 4.5):
        assert load_changeset(
            one_flag_set([good_verdict(line=bad)])
        ) == ["files[0].verdicts[0].line must be a positive integer"], bad


def test_the_verdict_pattern_must_be_a_borderline_pattern():
    assert load_changeset(
        one_flag_set([good_verdict(pattern="print call")])
    ) == [
        "files[0].verdicts[0].pattern must be one of "
        "near placeholder name, broad handler, shared member default"
    ]


def test_the_verdict_model_must_be_a_non_empty_string():
    for bad in ("", "   ", 7):
        assert load_changeset(
            one_flag_set([good_verdict(model=bad)])
        ) == [
            "files[0].verdicts[0].model must be a non empty string"
        ], bad


def test_the_verdict_model_must_hold_no_surrogate():
    assert load_changeset(
        one_flag_set([good_verdict(model="jev\ud800")])
    ) == ["files[0].verdicts[0].model holds an unpaired surrogate"]


def test_the_verdict_model_must_hold_no_line_break():
    assert load_changeset(
        one_flag_set([good_verdict(model="jev\nforged")])
    ) == ["files[0].verdicts[0].model holds a line break"]


def test_the_verdict_choice_must_be_a_label():
    assert load_changeset(one_flag_set([good_verdict(choice="maybe")])) == [
        "files[0].verdicts[0].choice must be one of stickler, clean"
    ]


def test_the_verdict_confidence_must_be_a_number_in_range():
    assert load_changeset(
        one_flag_set([good_verdict(confidence="high")])
    ) == ["files[0].verdicts[0].confidence must be a number"]
    assert load_changeset(
        one_flag_set([good_verdict(confidence=True)])
    ) == ["files[0].verdicts[0].confidence must be a number"]
    for bad in (1.5, -0.1, float("nan")):
        assert load_changeset(
            one_flag_set([good_verdict(confidence=bad)])
        ) == [
            "files[0].verdicts[0].confidence must be from 0 to 1"
        ], bad


def test_an_oversized_integer_confidence_is_refused_by_range():
    for bad in (10**400, -(10**400)):
        assert load_changeset(
            one_flag_set([good_verdict(confidence=bad)])
        ) == [
            "files[0].verdicts[0].confidence must be from 0 to 1"
        ], bad


def test_a_verdict_on_a_line_with_no_flag_is_named():
    assert load_changeset(one_flag_set([good_verdict(line=9)])) == [
        "files[0].verdicts[0] names no borderline line"
    ]


def test_a_verdict_with_the_wrong_pattern_is_named():
    assert load_changeset(
        one_flag_set([good_verdict(pattern="broad handler")])
    ) == ["files[0].verdicts[0] names no borderline line"]


def test_a_second_verdict_from_one_model_is_named():
    verdicts = [good_verdict(), good_verdict(choice="clean")]
    assert load_changeset(one_flag_set(verdicts)) == [
        "files[0].verdicts[1] repeats a verdict for the same "
        "line and model"
    ]


def test_verdicts_are_not_read_for_a_malformed_entry():
    value = {
        "files": [
            {
                "file": "",
                "diff": ONE_FLAG_DIFF,
                "verdicts": "not a list",
            }
        ]
    }
    assert load_changeset(value) == [
        "files[0].file must be a non empty string"
    ]


def test_review_changeset_groups_verdicts_onto_flags():
    verdicts = [
        good_verdict(),
        good_verdict(model="openai/gpt-6-luna", choice="clean"),
    ]
    results = review_changeset(one_flag_set(verdicts))
    name, judgements, blocks = results[0]
    assert name == "x.py"
    assert judgements == []
    assert len(blocks) == 1
    assert blocks[0]["flag"]["line"] == 4
    assert blocks[0]["verdicts"] == verdicts
