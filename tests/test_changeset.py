"""Pins for CK-6. The change set loader and the set check."""

from stickler.changeset import check_changeset, load_changeset


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
