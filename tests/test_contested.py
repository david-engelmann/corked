"""Pins for the contested line reading of CK-9 and its example.

A flagged line is contested when its verdicts hold both labels
without abstaining. An abstention never contests, a single model
never contests, and two verdicts from one model are the defect
of CK-8 and never reach the reading. The example held here is
the constructed example of CK-10, named as constructed in the
contract and here, because the recorded pilot of CK-8 holds no
contested line. No verdict in this file is a recorded one.
"""

import json
import pathlib

from stickler.changeset import load_changeset, review_changeset
from stickler.cli import main
from stickler.render import render_set_review
from stickler.verdicts import is_contested, line_outcome

ROOT = pathlib.Path(__file__).resolve().parent.parent
EXAMPLES = ROOT / "examples"

JEV = "jev-1.13.0"
OPENAI = "openai/gpt-6-luna"
MODELS = {JEV, OPENAI}

ONE_FLAG_DIFF = (
    "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,1 @@\n+data = load_entries()\n"
)

TWO_FLAG_DIFF = (
    "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,3 @@\n"
    "+data = load_entries()\n+temp = []\n"
)


def entry(choice, confidence, model=JEV, line=4):
    return {
        "line": line,
        "pattern": "near placeholder name",
        "model": model,
        "choice": choice,
        "confidence": confidence,
    }


def flagged_set(verdicts, diff=ONE_FLAG_DIFF, name="x.py"):
    return {
        "files": [{"file": name, "diff": diff, "verdicts": verdicts}]
    }


def render(value):
    return render_set_review(review_changeset(value))


def test_both_labels_without_abstaining_contest():
    entries = [entry("stickler", 0.81), entry("clean", 0.77, OPENAI)]
    assert is_contested(entries)


def test_a_confidence_of_exactly_one_half_holds_its_label():
    assert is_contested([entry("stickler", 0.5), entry("clean", 0.9, OPENAI)])
    assert is_contested([entry("stickler", 0.5), entry("clean", 0.5, OPENAI)])


def test_one_label_alone_never_contests():
    assert not is_contested([entry("stickler", 0.9)])
    assert not is_contested([entry("clean", 0.9)])
    assert not is_contested([])
    assert not is_contested(
        [entry("stickler", 0.9), entry("stickler", 0.8, OPENAI)]
    )
    assert not is_contested(
        [entry("clean", 0.9), entry("clean", 0.8, OPENAI)]
    )


def test_one_model_holding_both_labels_never_contests():
    assert not is_contested(
        [entry("stickler", 0.9), entry("clean", 0.8)]
    )
    assert not is_contested(
        [entry("clean", 0.8), entry("stickler", 0.9)]
    )


def test_a_second_model_holding_either_label_still_contests():
    assert is_contested(
        [
            entry("stickler", 0.9),
            entry("clean", 0.8),
            entry("stickler", 0.7, OPENAI),
        ]
    )
    assert is_contested(
        [
            entry("stickler", 0.9),
            entry("clean", 0.8),
            entry("clean", 0.7, OPENAI),
        ]
    )


def test_one_model_holding_both_labels_renders_no_contested_section():
    rendered = render(
        flagged_set([entry("stickler", 0.9), entry("clean", 0.8)])
    )
    assert "Contested lines" not in rendered
    assert "Contested, 0" in rendered


def test_an_abstention_never_contests():
    assert not is_contested(
        [entry("stickler", 0.9), entry("clean", 0.49, OPENAI)]
    )
    assert not is_contested(
        [entry("stickler", 0.49), entry("clean", 0.9, OPENAI)]
    )
    assert not is_contested(
        [entry("stickler", 0.2), entry("clean", 0.3, OPENAI)]
    )


def test_an_abstention_beside_the_contenders_still_contests():
    entries = [
        entry("stickler", 0.9),
        entry("clean", 0.8, OPENAI),
        entry("stickler", 0.3, "a-third-model"),
    ]
    assert is_contested(entries)


def test_a_contested_rendering_is_exact():
    value = flagged_set(
        [entry("stickler", 0.81), entry("clean", 0.77, OPENAI)]
    )
    assert render(value) == (
        "Change set review\n"
        "Files, 1\n"
        "\n"
        "File, x.py\n"
        "\n"
        "Borderline 1, near placeholder name\n"
        "Line, data = load_entries()\n"
        "Shape reading, drawn name entries\n"
        "Verdict, jev-1.13.0, stickler\n"
        "Reason, jev-1.13.0 chose stickler with confidence 0.81\n"
        "Verdict, openai/gpt-6-luna, clean\n"
        "Reason, openai/gpt-6-luna chose clean with confidence 0.77\n"
        "\n"
        "Contested lines\n"
        "\n"
        "File, x.py\n"
        "Borderline 1, near placeholder name\n"
        "Line, data = load_entries()\n"
        "Verdict, jev-1.13.0, stickler\n"
        "Verdict, openai/gpt-6-luna, clean\n"
        "\n"
        "Summary\n"
        "Judgements, 0\n"
        "Borderline, 1\n"
        "Verdict lines, stickler 1, clean 0\n"
        "Unjudged, 0\n"
        "Contested, 1\n"
    )


def test_the_section_names_each_verdict_in_standing_order():
    value = flagged_set(
        [
            entry("clean", 0.77, OPENAI),
            entry("stickler", 0.81),
            entry("stickler", 0.3, "a-third-model"),
        ]
    )
    rendered = render(value)
    section = rendered.split("Contested lines\n", 1)[1]
    verdict_lines = [
        line for line in section.splitlines()
        if line.startswith("Verdict, ")
    ]
    assert verdict_lines == [
        "Verdict, openai/gpt-6-luna, clean",
        "Verdict, jev-1.13.0, stickler",
        "Verdict, a-third-model, abstained",
    ]


def test_the_section_uses_the_blocks_own_numbers():
    verdicts = [
        entry("stickler", 0.9, line=4),
        entry("clean", 0.8, line=5),
        entry("stickler", 0.7, OPENAI, line=5),
    ]
    rendered = render(flagged_set(verdicts, diff=TWO_FLAG_DIFF))
    section = rendered.split("Contested lines\n", 1)[1]
    assert "Borderline 2, near placeholder name" in section
    assert "Borderline 1, near placeholder name" not in section
    assert "Contested, 1" in rendered


def test_flags_without_a_contest_count_zero_and_stand_no_section():
    rendered = render(flagged_set([entry("stickler", 0.9)]))
    assert "Contested, 0" in rendered
    assert "Contested lines" not in rendered


def test_an_abstention_beside_a_stickler_verdict_is_not_contested():
    rendered = render(
        flagged_set(
            [entry("stickler", 0.9), entry("clean", 0.3, OPENAI)]
        )
    )
    assert "Contested lines" not in rendered
    assert "Verdict lines, stickler 1, clean 0" in rendered
    assert "Contested, 0" in rendered


def test_an_abstention_beside_a_clean_verdict_is_not_contested():
    rendered = render(
        flagged_set(
            [entry("stickler", 0.3), entry("clean", 0.9, OPENAI)]
        )
    )
    assert "Contested lines" not in rendered
    assert "Verdict lines, stickler 0, clean 1" in rendered
    assert "Contested, 0" in rendered


def test_a_flagless_set_renders_no_contested_text():
    diff = (
        "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,1 @@\n"
        "+    print(total)\n"
    )
    value = {"files": [{"file": "x.py", "diff": diff}]}
    assert "Contested" not in render(value)


def test_two_verdicts_from_one_model_are_a_defect_and_never_render():
    value = flagged_set(
        [entry("stickler", 0.9), entry("clean", 0.8)]
    )
    defects = load_changeset(value)
    assert defects == [
        "files[0].verdicts[1] repeats a verdict for the same "
        "line and model"
    ]


def test_two_verdicts_from_one_model_exit_2(tmp_path, capsys):
    value = flagged_set(
        [entry("stickler", 0.9), entry("clean", 0.8)]
    )
    path = tmp_path / "set.json"
    path.write_text(json.dumps(value), encoding="utf-8")
    code = main(["review", str(path)])
    out = capsys.readouterr().out
    assert code == 2
    assert "Verdict, invalid" in out
    assert "Contested" not in out


def load_example():
    return json.loads(
        (EXAMPLES / "contested_set.json").read_text(encoding="utf-8")
    )


def example_blocks():
    blocks = []
    for name, _judgements, file_blocks in review_changeset(
        load_example()
    ):
        for block in file_blocks:
            blocks.append((name, block))
    return blocks


def test_the_constructed_example_is_a_valid_change_set():
    value = load_example()
    assert load_changeset(value) == []
    judgements = 0
    counts = {}
    for _name, file_judgements, _blocks in review_changeset(value):
        judgements += len(file_judgements)
        for block in _blocks:
            pattern = block["flag"]["pattern"]
            counts[pattern] = counts.get(pattern, 0) + 1
    assert judgements == 0
    assert counts == {
        "near placeholder name": 4,
        "broad handler": 1,
        "shared member default": 1,
    }


def test_the_constructed_example_outcomes_match_the_hand_count():
    outcomes = {"stickler": 0, "clean": 0, "unjudged": 0}
    contested = []
    for name, block in example_blocks():
        entries = block["verdicts"]
        outcomes[line_outcome(entries)] += 1
        if is_contested(entries):
            contested.append((name, block["flag"]["line"]))
    assert outcomes == {"stickler": 3, "clean": 2, "unjudged": 1}
    assert contested == [("ledger.py", 6), ("cache.py", 15)]


def test_the_constructed_example_entries_carry_the_recorded_models():
    total = 0
    for entry in load_example()["files"]:
        for verdict in entry.get("verdicts", []):
            total += 1
            assert verdict["model"] in MODELS
            assert set(verdict) == {
                "line", "pattern", "model", "choice", "confidence",
            }
    assert total == 10


def test_the_constructed_example_matches_its_golden_rendering():
    rendered = render(load_example())
    golden = (EXAMPLES / "contested_review.txt").read_text(
        encoding="utf-8"
    )
    assert rendered == golden


def test_the_constructed_example_replays_to_exit_1(capsys):
    code = main(["review", str(EXAMPLES / "contested_set.json")])
    out = capsys.readouterr().out
    golden = (EXAMPLES / "contested_review.txt").read_text(
        encoding="utf-8"
    )
    assert code == 1
    assert out == golden


def test_the_recorded_pilot_holds_no_contested_line():
    pilot = json.loads(
        (EXAMPLES / "borderline_review.json").read_text(
            encoding="utf-8"
        )
    )
    flag_count = 0
    for _name, _judgements, blocks in review_changeset(pilot):
        for block in blocks:
            flag_count += 1
            assert not is_contested(block["verdicts"])
    assert flag_count == 12
    rendered = render_set_review(review_changeset(pilot))
    assert rendered.endswith("Unjudged, 1\nContested, 0\n")
    assert "Contested lines" not in rendered
