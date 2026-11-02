"""Pins for the recorded borderline pilot of CK-8.

The suite runs offline. These tests replay the recorded envelopes
through the same seat readers the live run used and rebuild the
pilot review exactly, recompute the run record from the envelope
fixtures, and hold the pilot review against its stored rendering.
"""

import json
import pathlib
from decimal import Decimal

from stickler.changeset import load_changeset, review_changeset
from stickler.cli import main
from stickler.render import render_set_review
from stickler.verdicts import (
    build_verdict_entry,
    verdict_from_chat_envelope,
    verdict_from_jev_envelope,
    verdict_outcome,
)

ROOT = pathlib.Path(__file__).resolve().parent.parent
EXAMPLES = ROOT / "examples"
ENVELOPES = EXAMPLES / "verdicts" / "envelopes"

ROUTES = (
    ("jev", "jev-1.13.0", verdict_from_jev_envelope,
     "input_tokens", "output_tokens"),
    ("openai", "openai/gpt-6-luna", verdict_from_chat_envelope,
     "prompt_tokens", "completion_tokens"),
)


def load_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def pilot_flags():
    value = load_json(EXAMPLES / "borderline_set.json")
    flags = []
    for index, (name, _judgements, blocks) in enumerate(
        review_changeset(value), start=1
    ):
        for block in blocks:
            flags.append((index, name, block["flag"]))
    return flags


def envelope_name(file_index, flag, suffix):
    return f"f{file_index}-l{flag['line']:02d}-{suffix}.json"


def replay_entries():
    """Rebuild the recorded verdicts from the envelopes alone."""
    per_file = {}
    outcomes = {}
    for file_index, name, flag in pilot_flags():
        for suffix, _model, reader, _ki, _ko in ROUTES:
            envelope = load_json(
                ENVELOPES / envelope_name(file_index, flag, suffix)
            )
            verdict = reader(envelope)
            key = (name, flag["line"], suffix)
            outcomes[key] = verdict
            if verdict is not None:
                per_file.setdefault(name, []).append(
                    build_verdict_entry(flag, verdict)
                )
    return per_file, outcomes


def test_the_pilot_set_carries_twelve_flags_in_the_stated_split():
    value = load_json(EXAMPLES / "borderline_set.json")
    assert load_changeset(value) == []
    counts = {}
    total_judgements = 0
    for _name, judgements, blocks in review_changeset(value):
        total_judgements += len(judgements)
        for block in blocks:
            pattern = block["flag"]["pattern"]
            counts[pattern] = counts.get(pattern, 0) + 1
    assert total_judgements == 0
    assert counts == {
        "near placeholder name": 5,
        "broad handler": 3,
        "shared member default": 4,
    }


def test_the_envelope_inventory_matches_the_flags():
    names = sorted(path.name for path in ENVELOPES.glob("*.json"))
    expected = sorted(
        envelope_name(index, flag, suffix)
        for index, _name, flag in pilot_flags()
        for suffix, _model, _reader, _ki, _ko in ROUTES
    )
    assert names == expected


def test_the_pilot_review_rebuilds_from_the_envelopes():
    per_file, _outcomes = replay_entries()
    review = load_json(EXAMPLES / "borderline_review.json")
    assert load_changeset(review) == []
    for entry in review["files"]:
        assert entry.get("verdicts", []) == per_file.get(
            entry["file"], []
        )


def test_the_run_record_recomputes_from_the_envelopes():
    record = load_json(EXAMPLES / "verdicts" / "run_record.json")
    _per_file, outcomes = replay_entries()
    flags = pilot_flags()
    by_model = {route["model"]: route for route in record["routes"]}
    assert set(by_model) == {"jev-1.13.0", "openai/gpt-6-luna"}
    for suffix, model, reader, key_in, key_out in ROUTES:
        route = by_model[model]
        attempted = recorded = failed = abstained = 0
        tin = tout = 0
        cost = Decimal(0)
        for file_index, name, flag in flags:
            envelope = load_json(
                ENVELOPES / envelope_name(file_index, flag, suffix)
            )
            attempted += 1
            verdict = outcomes[(name, flag["line"], suffix)]
            if verdict is None:
                failed += 1
                continue
            recorded += 1
            if verdict_outcome(verdict) == "abstained":
                abstained += 1
            usage = envelope["usage"]
            tin += usage[key_in]
            tout += usage[key_out]
            if "cost" in usage:
                cost += Decimal(str(usage["cost"]))
        assert route["calls_attempted"] == attempted == 12
        assert route["verdicts_recorded"] == recorded
        assert route["calls_failed"] == failed
        assert route["abstentions"] == abstained
        assert route["input_tokens"] == tin
        assert route["output_tokens"] == tout
        if model.startswith("openai"):
            assert route["cost_dollars"] == format(cost, "f")
        else:
            assert "cost_dollars" not in route


def test_the_failed_calls_record_nothing():
    _per_file, outcomes = replay_entries()
    failed = [
        key for key, verdict in outcomes.items() if verdict is None
    ]
    assert len(failed) == 2
    assert all(suffix == "openai" for _n, _l, suffix in failed)


def test_the_pilot_review_matches_its_stored_rendering():
    review = load_json(EXAMPLES / "borderline_review.json")
    rendered = render_set_review(review_changeset(review))
    golden = (EXAMPLES / "borderline_review.txt").read_text(
        encoding="utf-8"
    )
    assert rendered == golden


def test_the_pilot_review_replays_to_its_recorded_exit_code(capsys):
    code = main(["review", str(EXAMPLES / "borderline_review.json")])
    out = capsys.readouterr().out
    assert code == 1
    golden = (EXAMPLES / "borderline_review.txt").read_text(
        encoding="utf-8"
    )
    assert out == golden


def test_the_pilot_summary_counts_zero_contested_lines():
    review = load_json(EXAMPLES / "borderline_review.json")
    rendered = render_set_review(review_changeset(review))
    assert rendered.endswith("Unjudged, 1\nContested, 0\n")
    assert "Contested lines" not in rendered
