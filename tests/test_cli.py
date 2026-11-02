"""Pins for CK-4. The two commands and the exit code per verdict."""

import json
import pathlib

from stickler.cli import main

ROOT = pathlib.Path(__file__).resolve().parent.parent


def write(tmp_path, name, text):
    path = tmp_path / name
    path.write_text(text, encoding="utf-8")
    return str(path)


def write_bytes(tmp_path, name, data):
    path = tmp_path / name
    path.write_bytes(data)
    return str(path)


SMELLY = (
    "--- a/app.py\n"
    "+++ b/app.py\n"
    "@@ -1,1 +1,2 @@\n"
    "+    print(total)\n"
)
CLEAN = (
    "--- a/app.py\n"
    "+++ b/app.py\n"
    "@@ -1,1 +1,2 @@\n"
    "+    logger.debug(total)\n"
)


def test_check_on_a_smelly_diff_sends_back_with_exit_1(tmp_path, capsys):
    code = main(["check", write(tmp_path, "change.diff", SMELLY)])
    out = capsys.readouterr().out
    assert code == 1
    assert "Verdict, sent back\n" in out
    assert "Objection 1, smell\n" in out


def test_check_on_a_clean_diff_accepts_with_exit_0(tmp_path, capsys):
    code = main(["check", write(tmp_path, "change.diff", CLEAN)])
    out = capsys.readouterr().out
    assert code == 0
    assert "Verdict, accepted\n" in out


def test_check_labels_the_review_with_the_file_name(tmp_path, capsys):
    main(["check", write(tmp_path, "change.diff", CLEAN)])
    out = capsys.readouterr().out
    assert out.startswith("Review of change.diff\n")


def test_check_on_a_missing_file_is_invalid_with_exit_2(tmp_path, capsys):
    code = main(["check", str(tmp_path / "gone.diff")])
    out = capsys.readouterr().out
    assert code == 2
    assert "Verdict, invalid\n" in out
    assert "Defect, the diff file could not be read\n" in out


def test_render_on_an_accepted_review_exits_0(tmp_path, capsys):
    path = write(tmp_path, "review.json",
                 json.dumps({"diff_label": "tidy", "objections": []}))
    code = main(["render", path])
    out = capsys.readouterr().out
    assert code == 0
    assert "Verdict, accepted\n" in out


def test_render_on_a_sent_back_review_exits_1(tmp_path, capsys):
    objection = {
        "flaw": "The name says nothing",
        "standard": "A name says what its value holds",
        "rewrite": "orders = load_orders()",
        "category": "naming",
    }
    path = write(tmp_path, "review.json",
                 json.dumps({"diff_label": "rough", "objections": [objection]}))
    code = main(["render", path])
    out = capsys.readouterr().out
    assert code == 1
    assert "Verdict, sent back\n" in out


def test_render_on_a_non_json_file_is_invalid_with_exit_2(tmp_path, capsys):
    code = main(["render", write(tmp_path, "review.json", "not json")])
    out = capsys.readouterr().out
    assert code == 2
    assert "Defect, the file does not hold a JSON review\n" in out


def test_render_on_an_incomplete_objection_exits_2_and_names_the_field(
        tmp_path, capsys):
    path = write(tmp_path, "review.json",
                 json.dumps({"diff_label": "rough",
                             "objections": [{"flaw": "x"}]}))
    code = main(["render", path])
    out = capsys.readouterr().out
    assert code == 2
    assert "Defect, objections[0].standard is missing\n" in out


def test_render_on_a_missing_file_is_invalid_with_exit_2(tmp_path, capsys):
    code = main(["render", str(tmp_path / "gone.json")])
    out = capsys.readouterr().out
    assert code == 2
    assert "Defect, the review file could not be read\n" in out


def test_check_on_a_non_utf8_diff_is_invalid_with_exit_2(tmp_path, capsys):
    data = (b"--- a/app.py\n+++ b/app.py\n@@ -1,1 +1,1 @@\n"
            b"+total = b'\xff\xfe'\n")
    code = main(["check", write_bytes(tmp_path, "change.diff", data)])
    out = capsys.readouterr().out
    assert code == 2
    assert "Verdict, invalid\n" in out
    assert "Defect, the diff file could not be read\n" in out


def test_render_on_a_non_utf8_review_is_invalid_with_exit_2(tmp_path, capsys):
    data = b'{"diff_label": "bad \xff\xfe", "objections": []}'
    code = main(["render", write_bytes(tmp_path, "review.json", data)])
    out = capsys.readouterr().out
    assert code == 2
    assert "Verdict, invalid\n" in out
    assert "Defect, the review file could not be read\n" in out


def test_usage_errors_exit_2(capsys):
    assert main([]) == 2
    assert main(["check"]) == 2
    assert main(["bogus", "x"]) == 2


def test_the_smelly_example_sends_back_and_the_clean_example_accepts(capsys):
    code = main(["check", str(ROOT / "examples" / "smelly.diff")])
    out = capsys.readouterr().out
    assert code == 1
    assert "Verdict, sent back\n" in out
    code = main(["check", str(ROOT / "examples" / "clean.diff")])
    out = capsys.readouterr().out
    assert code == 0
    assert "Verdict, accepted\n" in out


def change_set(entries):
    return json.dumps({"files": entries})


def set_entry(name, *added_lines):
    lines = ["--- a/x.py", "+++ b/x.py", "@@ -1,1 +1,1 @@"]
    lines.extend("+" + line for line in added_lines)
    return {"file": name, "diff": "\n".join(lines) + "\n"}


def test_review_on_a_clean_set_exits_0(tmp_path, capsys):
    path = write(tmp_path, "set.json",
                 change_set([set_entry("a.py", "    logger.debug(total)")]))
    code = main(["review", path])
    out = capsys.readouterr().out
    assert code == 0
    assert "File, a.py\n" in out
    assert "Clean\n" in out
    assert "Judgements, 0\n" in out


def test_review_on_a_set_with_judgements_exits_1(tmp_path, capsys):
    path = write(tmp_path, "set.json",
                 change_set([set_entry("a.py", "    print(total)")]))
    code = main(["review", path])
    out = capsys.readouterr().out
    assert code == 1
    assert "Judgement 1, smell\n" in out
    assert "Worst, a.py, 1\n" in out


def test_review_on_a_non_json_file_is_malformed_with_exit_2(tmp_path, capsys):
    code = main(["review", write(tmp_path, "set.json", "not json")])
    out = capsys.readouterr().out
    assert code == 2
    assert "Verdict, invalid\n" in out
    assert "Defect, the file does not hold a JSON change set\n" in out


def test_review_on_a_set_with_a_loader_defect_exits_2_and_names_it(
        tmp_path, capsys):
    path = write(tmp_path, "set.json",
                 change_set([{"file": "a.py"}]))
    code = main(["review", path])
    out = capsys.readouterr().out
    assert code == 2
    assert "Defect, files[0].diff is missing\n" in out


def test_review_on_an_oversized_integer_confidence_exits_2_without_a_traceback(
        tmp_path, capsys):
    verdict = {
        "line": 4,
        "pattern": "near placeholder name",
        "model": "jev-1.13.0",
        "choice": "stickler",
        "confidence": 10**400,
    }
    entry = set_entry("a.py", "data = load_entries()")
    entry["verdicts"] = [verdict]
    path = write(tmp_path, "set.json", change_set([entry]))
    code = main(["review", path])
    captured = capsys.readouterr()
    assert code == 2
    assert (
        "Defect, files[0].verdicts[0].confidence must be from 0 to 1\n"
        in captured.out
    )
    assert "Traceback" not in captured.err


def test_review_on_a_missing_file_is_malformed_with_exit_2(tmp_path, capsys):
    code = main(["review", str(tmp_path / "gone.json")])
    out = capsys.readouterr().out
    assert code == 2
    assert "Defect, the change set file could not be read\n" in out


def test_review_on_a_non_utf8_file_is_malformed_with_exit_2(tmp_path, capsys):
    data = b'{"files": [{"file": "bad \xff\xfe", "diff": ""}]}'
    code = main(["review", write_bytes(tmp_path, "set.json", data)])
    out = capsys.readouterr().out
    assert code == 2
    assert "Defect, the change set file could not be read\n" in out


def test_review_on_a_deeply_nested_file_is_malformed_with_exit_2(
        tmp_path, capsys):
    data = ("[" * 100000 + "]" * 100000).encode("utf-8")
    code = main(["review", write_bytes(tmp_path, "set.json", data)])
    out = capsys.readouterr().out
    assert code == 2
    assert "Defect, the file does not hold a JSON change set\n" in out


def test_review_usage_errors_exit_2(capsys):
    assert main(["review"]) == 2
    assert main(["review", "a", "b"]) == 2


def test_review_on_a_surrogate_file_name_is_malformed_with_exit_2(
        tmp_path, capsys):
    path = write(tmp_path, "set.json",
                 change_set([{"file": "\ud800x.py", "diff": ""}]))
    code = main(["review", path])
    captured = capsys.readouterr()
    assert code == 2
    assert "Defect, files[0].file holds an unpaired surrogate\n" in captured.out
    assert "Traceback" not in captured.err
    captured.out.encode("utf-8")


def test_review_on_a_surrogate_in_diff_text_is_malformed_with_exit_2(
        tmp_path, capsys):
    path = write(tmp_path, "set.json",
                 change_set([{"file": "a.py", "diff": "+print('\ud800')"}]))
    code = main(["review", path])
    captured = capsys.readouterr()
    assert code == 2
    assert "Defect, files[0].diff holds an unpaired surrogate\n" in captured.out
    assert "Traceback" not in captured.err
    captured.out.encode("utf-8")


def test_render_on_a_surrogate_diff_label_is_invalid_with_exit_2(
        tmp_path, capsys):
    path = write(tmp_path, "review.json",
                 json.dumps({"diff_label": "\ud800x", "objections": []}))
    code = main(["render", path])
    captured = capsys.readouterr()
    assert code == 2
    assert captured.out.startswith("Review of unnamed review\n")
    assert "Defect, diff_label holds an unpaired surrogate\n" in captured.out
    assert "Traceback" not in captured.err
    captured.out.encode("utf-8")


def test_render_on_a_surrogate_objection_field_is_invalid_with_exit_2(
        tmp_path, capsys):
    objection = {
        "flaw": "bad \ud800 text",
        "standard": "A name says what its value holds",
        "rewrite": "orders = load_orders()",
        "category": "naming",
    }
    path = write(tmp_path, "review.json",
                 json.dumps({"diff_label": "rough",
                             "objections": [objection]}))
    code = main(["render", path])
    captured = capsys.readouterr()
    assert code == 2
    assert "Defect, objections[0].flaw holds an unpaired surrogate\n" in (
        captured.out
    )
    assert "Traceback" not in captured.err
    captured.out.encode("utf-8")


BROAD_HANDLER_DIFF = (
    "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,1 @@\n"
    "+    except BaseException:\n"
)


def test_review_on_a_line_break_file_name_is_malformed_with_exit_2(
        tmp_path, capsys):
    entry = {
        "file": "evil\nForged, 99",
        "diff": BROAD_HANDLER_DIFF,
        "verdicts": [
            {
                "line": 4,
                "pattern": "broad handler",
                "model": "jev-1.13.0",
                "choice": "stickler",
                "confidence": 0.78,
            }
        ],
    }
    path = write(tmp_path, "set.json", change_set([entry]))
    code = main(["review", path])
    captured = capsys.readouterr()
    assert code == 2
    assert "Defect, files[0].file holds a line break\n" in captured.out
    assert "Forged" not in captured.out
    assert "Traceback" not in captured.err
    captured.out.encode("utf-8")


def test_render_on_a_line_break_diff_label_is_invalid_with_exit_2(
        tmp_path, capsys):
    path = write(tmp_path, "review.json",
                 json.dumps({"diff_label": "evil\nForged, 99",
                             "objections": []}))
    code = main(["render", path])
    captured = capsys.readouterr()
    assert code == 2
    assert captured.out.startswith("Review of unnamed review\n")
    assert "Defect, diff_label holds a line break\n" in captured.out
    assert "Forged" not in captured.out
    assert "Traceback" not in captured.err
    captured.out.encode("utf-8")


BORDERLINE_DIFF = (
    "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,1 @@\n+data = load_entries()\n"
)


def verdict_set(verdicts):
    return {
        "files": [
            {
                "file": "x.py",
                "diff": BORDERLINE_DIFF,
                "verdicts": verdicts,
            }
        ]
    }


def one_verdict(**overrides):
    verdict = {
        "line": 4,
        "pattern": "near placeholder name",
        "model": "jev-1.13.0",
        "choice": "stickler",
        "confidence": 0.78,
    }
    verdict.update(overrides)
    return verdict


def test_review_with_a_stickler_verdict_exits_1(tmp_path, capsys):
    path = write(
        tmp_path, "set.json", json.dumps(verdict_set([one_verdict()]))
    )
    code = main(["review", path])
    out = capsys.readouterr().out
    assert code == 1
    assert "Judgements, 0" in out
    assert "Borderline 1, near placeholder name" in out
    assert "Line, data = load_entries()" in out
    assert "Shape reading, drawn name entries" in out
    assert "Verdict, jev-1.13.0, stickler" in out
    assert (
        "Reason, jev-1.13.0 chose stickler with confidence 0.78" in out
    )
    assert "Borderline, 1" in out
    assert "Verdict lines, stickler 1, clean 0" in out
    assert "Unjudged, 0" in out


def test_review_with_only_clean_verdicts_exits_0(tmp_path, capsys):
    path = write(
        tmp_path,
        "set.json",
        json.dumps(verdict_set([one_verdict(choice="clean")])),
    )
    code = main(["review", path])
    out = capsys.readouterr().out
    assert code == 0
    assert "Verdict lines, stickler 0, clean 1" in out


def test_review_with_only_an_abstention_exits_0_unjudged(
    tmp_path, capsys
):
    path = write(
        tmp_path,
        "set.json",
        json.dumps(verdict_set([one_verdict(confidence=0.2)])),
    )
    code = main(["review", path])
    out = capsys.readouterr().out
    assert code == 0
    assert "Verdict, jev-1.13.0, abstained" in out
    assert "and abstained" in out
    assert "\nUnjudged\n" in out
    assert "Unjudged, 1" in out


def test_review_with_a_malformed_verdict_exits_2(tmp_path, capsys):
    path = write(
        tmp_path,
        "set.json",
        json.dumps(verdict_set([one_verdict(choice="maybe")])),
    )
    code = main(["review", path])
    out = capsys.readouterr().out
    assert code == 2
    assert (
        "files[0].verdicts[0].choice must be one of stickler, clean"
        in out
    )


def test_review_of_a_flagged_file_draws_no_clean_line(
    tmp_path, capsys
):
    path = write(tmp_path, "set.json", json.dumps(verdict_set([])))
    code = main(["review", path])
    out = capsys.readouterr().out
    assert code == 0
    assert "File, x.py\n\nBorderline 1, near placeholder name" in out
    assert "Unjudged, 1" in out
