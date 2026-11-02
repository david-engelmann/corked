"""Pins for the verdict seat of CK-7.

Both envelope shapes read through the one seat, a call the seat
cannot read records nothing, an abstention is a confidence below
one half, and the recorded reason is the stated construction.
The readings the seat presents stand word for word in the
contract under FIX-CK-14, and a pin here holds them to it.
"""

import pathlib
import unittest

from stickler.verdicts import (
    PATTERN_READINGS,
    build_verdict_entry,
    is_abstention,
    line_outcome,
    reason_for,
    seat_state,
    verdict_from_chat_envelope,
    verdict_from_jev_envelope,
    verdict_outcome,
)

ROOT = pathlib.Path(__file__).resolve().parent.parent


def jev_envelope(choice="stickler", confidence=0.78, model="jev-1.13.0"):
    return {
        "answers": {
            "decision": {
                "choice": choice,
                "confidence": confidence,
                "probabilities": {"stickler": 0.78, "clean": 0.22},
                "type": "choice",
            }
        },
        "model": model,
        "usage": {"input_tokens": 500, "output_tokens": 35},
    }


def chat_envelope(content, model="openai/gpt-6-luna"):
    return {
        "model": model,
        "choices": [{"message": {"role": "assistant", "content": content}}],
        "usage": {"prompt_tokens": 200, "completion_tokens": 40},
    }


GOOD_CHAT = (
    '{"choice": "clean", "confidence": 0.9, '
    '"probabilities": {"stickler": 0.1, "clean": 0.9}}'
)


class SeatReaderTests(unittest.TestCase):
    def test_jev_envelope_reads(self):
        verdict = verdict_from_jev_envelope(jev_envelope())
        self.assertEqual(
            verdict,
            {
                "model": "jev-1.13.0",
                "choice": "stickler",
                "confidence": 0.78,
            },
        )

    def test_chat_envelope_reads(self):
        verdict = verdict_from_chat_envelope(chat_envelope(GOOD_CHAT))
        self.assertEqual(
            verdict,
            {
                "model": "openai/gpt-6-luna",
                "choice": "clean",
                "confidence": 0.9,
            },
        )

    def test_a_choice_outside_the_labels_records_nothing(self):
        self.assertIsNone(
            verdict_from_jev_envelope(jev_envelope(choice="maybe"))
        )

    def test_a_confidence_out_of_range_records_nothing(self):
        self.assertIsNone(
            verdict_from_jev_envelope(jev_envelope(confidence=1.5))
        )
        self.assertIsNone(
            verdict_from_jev_envelope(jev_envelope(confidence=-0.1))
        )

    def test_a_non_finite_confidence_records_nothing(self):
        self.assertIsNone(
            verdict_from_jev_envelope(
                jev_envelope(confidence=float("nan"))
            )
        )

    def test_a_boolean_confidence_records_nothing(self):
        self.assertIsNone(
            verdict_from_jev_envelope(jev_envelope(confidence=True))
        )

    def test_missing_probabilities_record_nothing(self):
        envelope = jev_envelope()
        del envelope["answers"]["decision"]["probabilities"]
        self.assertIsNone(verdict_from_jev_envelope(envelope))

    def test_a_probability_missing_for_a_label_records_nothing(self):
        envelope = jev_envelope()
        envelope["answers"]["decision"]["probabilities"] = {
            "stickler": 1.0
        }
        self.assertIsNone(verdict_from_jev_envelope(envelope))

    def test_a_model_holding_a_line_break_records_nothing(self):
        self.assertIsNone(
            verdict_from_jev_envelope(jev_envelope(model="jev\n1.0"))
        )

    def test_a_model_holding_a_surrogate_records_nothing(self):
        self.assertIsNone(
            verdict_from_jev_envelope(jev_envelope(model="jev\ud800"))
        )

    def test_missing_answers_record_nothing(self):
        self.assertIsNone(verdict_from_jev_envelope({"model": "jev"}))
        self.assertIsNone(verdict_from_jev_envelope("not an envelope"))
        self.assertIsNone(verdict_from_jev_envelope(None))

    def test_chat_prose_around_the_object_records_nothing(self):
        self.assertIsNone(
            verdict_from_chat_envelope(
                chat_envelope("Here is my verdict. " + GOOD_CHAT)
            )
        )

    def test_chat_non_finite_confidence_records_nothing(self):
        content = (
            '{"choice": "clean", "confidence": NaN, '
            '"probabilities": {"stickler": 0.1, "clean": 0.9}}'
        )
        self.assertIsNone(
            verdict_from_chat_envelope(chat_envelope(content))
        )

    def test_chat_missing_content_records_nothing(self):
        self.assertIsNone(
            verdict_from_chat_envelope(chat_envelope(None))
        )

    def test_chat_missing_choices_record_nothing(self):
        self.assertIsNone(
            verdict_from_chat_envelope({"model": "openai/gpt-6-luna"})
        )


class AbstentionAndReasonTests(unittest.TestCase):
    def test_exactly_one_half_records_as_a_verdict(self):
        verdict = {"model": "m", "choice": "clean", "confidence": 0.5}
        self.assertFalse(is_abstention(verdict))
        self.assertEqual(verdict_outcome(verdict), "clean")

    def test_below_one_half_abstains(self):
        verdict = {"model": "m", "choice": "stickler", "confidence": 0.49}
        self.assertTrue(is_abstention(verdict))
        self.assertEqual(verdict_outcome(verdict), "abstained")

    def test_an_abstention_keeps_its_choice_out_of_the_outcome(self):
        verdict = {"model": "m", "choice": "stickler", "confidence": 0.2}
        self.assertEqual(verdict_outcome(verdict), "abstained")

    def test_the_reason_is_the_stated_construction(self):
        verdict = {"model": "m", "choice": "stickler", "confidence": 0.78}
        self.assertEqual(
            reason_for(verdict),
            "m chose stickler with confidence 0.78",
        )

    def test_the_abstention_reason_names_the_abstention(self):
        verdict = {"model": "m", "choice": "clean", "confidence": 0.2}
        self.assertEqual(
            reason_for(verdict),
            "m chose clean with confidence 0.2 and abstained",
        )

    def test_a_whole_confidence_renders_without_a_point(self):
        verdict = {"model": "m", "choice": "stickler", "confidence": 1}
        self.assertEqual(
            reason_for(verdict), "m chose stickler with confidence 1"
        )

    def test_the_seat_state_presents_the_case_in_order(self):
        flag = {
            "line": 6,
            "pattern": "near placeholder name",
            "content": "data = load_entries()",
            "reading": "drawn name entries",
        }
        state = seat_state("harvest.py", flag)
        positions = [
            state.index("harvest.py"),
            state.index("near placeholder name"),
            state.index("data = load_entries()"),
            state.index("drawn name entries"),
            state.index("Clean reading:"),
            state.index("Stickler reading:"),
        ]
        self.assertEqual(positions, sorted(positions))

    def test_the_verdict_entry_holds_exactly_five_fields(self):
        flag = {
            "line": 6,
            "pattern": "broad handler",
            "content": "x",
            "reading": "handler names BaseException",
        }
        verdict = {"model": "m", "choice": "clean", "confidence": 0.9}
        entry = build_verdict_entry(flag, verdict)
        self.assertEqual(
            entry,
            {
                "line": 6,
                "pattern": "broad handler",
                "model": "m",
                "choice": "clean",
                "confidence": 0.9,
            },
        )

    def test_the_line_outcome_prefers_a_stickler_verdict(self):
        entries = [
            {"model": "a", "choice": "clean", "confidence": 0.9},
            {"model": "b", "choice": "stickler", "confidence": 0.6},
        ]
        self.assertEqual(line_outcome(entries), "stickler")

    def test_a_line_with_only_abstentions_is_unjudged(self):
        entries = [
            {"model": "a", "choice": "stickler", "confidence": 0.2},
            {"model": "b", "choice": "clean", "confidence": 0.1},
        ]
        self.assertEqual(line_outcome(entries), "unjudged")
        self.assertEqual(line_outcome([]), "unjudged")

    def test_a_clean_verdict_beside_an_abstention_judges_clean(self):
        entries = [
            {"model": "a", "choice": "clean", "confidence": 0.9},
            {"model": "b", "choice": "stickler", "confidence": 0.2},
        ]
        self.assertEqual(line_outcome(entries), "clean")


class PresentedReadingsTests(unittest.TestCase):
    def test_each_presented_reading_stands_verbatim_in_the_contract(self):
        text = (ROOT / "CONTRACT.md").read_text(encoding="utf-8")
        for pattern, readings in PATTERN_READINGS.items():
            for reading in readings:
                self.assertIn(reading, text, pattern)


if __name__ == "__main__":
    unittest.main()
