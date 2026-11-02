"""The verdict seat over decision models, and its envelope readers.

The seat takes one borderline flag the checker raised and makes one
call asking for a choice between the labels stickler and clean. The
readers in this module take one recorded envelope, as parsed JSON,
and return the verdict the seat produced, or None when the call
failed. The readers are pure functions of their inputs and make no
call themselves. The obligations this module carries are CK-7 and
CK-8 in CONTRACT.md, in the Round 3 section.
"""

import json
import math
import sys
from fractions import Fraction

LABELS = ("stickler", "clean")

ABSTAIN_THRESHOLD = Fraction(1, 2)

PATTERN_READINGS = {
    "near placeholder name": (
        "the placeholder shape names exactly six names, and a name "
        "outside the six is one the checker accepts, and a short "
        "name can be honest in a small scope",
        "the name is the same shrug as the named placeholders and "
        "says nothing about what the value holds or what the "
        "function does",
    ),
    "broad handler": (
        "the handler names the exception it catches, and naming "
        "the expected failures is the standard the bare except "
        "shape states",
        "a handler on BaseException also catches the control "
        "exceptions a process is stopped and interrupted with, and "
        "those failures must still rise where they can be seen, so "
        "naming the handler does not cure the catch all harm",
    ),
    "shared member default": (
        "a tuple default is the standard immutable default, and "
        "neither the mutable default shape nor the call default "
        "shape names a tuple",
        "the list, dict, or set member is one object built when "
        "the definition is read and shared by every call that "
        "omits the argument, and that sharing is the harm the "
        "definition time shapes name",
    ),
}


def _holds_unpaired_surrogate(text):
    """Read a string for an unpaired surrogate. Such a string cannot
    be encoded as UTF-8 text and so cannot be printed."""
    try:
        text.encode("utf-8")
    except UnicodeEncodeError:
        return True
    return False


def _holds_line_break(text):
    return "\n" in text or "\r" in text


def _is_number(value):
    """Return whether a JSON value is a number and not a boolean."""
    return isinstance(value, (int, float)) and not isinstance(
        value, bool
    )


def _is_finite(value):
    """Return whether a number is finite, read without conversion.

    The value is a number and not a boolean, as _is_number reads
    it. A float is read by math.isfinite, which no float can
    overflow. An integer is read by comparison against the
    largest finite float, and the comparison converts nothing,
    so an integer with hundreds of digits is refused by the
    comparison instead of raising, under FIX-CK-16.
    """
    if isinstance(value, int):
        return -sys.float_info.max <= value <= sys.float_info.max
    return math.isfinite(value)


def _render_confidence(confidence):
    """Render a confidence figure as its shortest round trip text.

    An integer confidence renders with no decimal point, and a
    fractional one renders as the shortest decimal text that
    parses back to the same value.
    """
    if isinstance(confidence, int):
        return str(confidence)
    return repr(confidence)


def seat_state(file_name, flag):
    """Return the summary form the seat presents for one flag.

    The form names, in the order CK-7 states, the file the flagged
    line stands in, the pattern the line matched, the flagged line
    as written, the reading the flag carries, and the two readings
    of the pattern. The routes present this same text, and the
    chat route adds its reply form after it.
    """
    clean_reading, stickler_reading = PATTERN_READINGS[flag["pattern"]]
    return (
        "A strict code reviewer flagged one added line as "
        "borderline, sitting between two readings of its rules.\n"
        f"File: {file_name}\n"
        f"Pattern: {flag['pattern']}\n"
        f"Line: {flag['content']}\n"
        f"Shape reading: {flag['reading']}\n"
        f"Clean reading: {clean_reading}\n"
        f"Stickler reading: {stickler_reading}\n"
        "Choose stickler when the line falls short of the standard "
        "the pattern protects, and clean when it meets it."
    )


def _read_model(envelope):
    """Return the envelope model string, or None when unreadable."""
    if not isinstance(envelope, dict):
        return None
    model = envelope.get("model")
    if not isinstance(model, str) or model == "":
        return None
    if _holds_unpaired_surrogate(model):
        return None
    if _holds_line_break(model):
        return None
    return model


def _verdict_from_decision(model, decision):
    """Build the verdict from one decision, or None.

    The decision must hold a choice that is one of the two labels,
    a finite confidence from zero to one, and probabilities
    holding a finite number for each of the two labels. Anything
    else is a call the seat cannot read, and the seat records
    nothing. A choice holding a line break or an unpaired
    surrogate also fails the call, so a verdict the seat produces
    always stands clean in a recorded review.
    """
    if not isinstance(decision, dict):
        return None
    choice = decision.get("choice")
    if choice not in LABELS:
        return None
    if _holds_unpaired_surrogate(choice):
        return None
    if _holds_line_break(choice):
        return None
    confidence = decision.get("confidence")
    if not _is_number(confidence):
        return None
    if not _is_finite(confidence):
        return None
    if confidence < 0 or confidence > 1:
        return None
    probabilities = decision.get("probabilities")
    if not isinstance(probabilities, dict):
        return None
    for label in LABELS:
        value = probabilities.get(label)
        if not _is_number(value) or not _is_finite(value):
            return None
    return {
        "model": model,
        "choice": choice,
        "confidence": confidence,
    }


def verdict_from_jev_envelope(envelope):
    """Return the verdict from one Typesafe System One envelope.

    The envelope holds the model string and an answers field whose
    decision field holds the decision. Returns None when the call
    failed, for the reasons stated in _verdict_from_decision.
    """
    model = _read_model(envelope)
    if model is None:
        return None
    answers = envelope.get("answers")
    if not isinstance(answers, dict):
        return None
    return _verdict_from_decision(model, answers.get("decision"))


def verdict_from_chat_envelope(envelope):
    """Return the verdict from one chat completion envelope.

    The envelope holds the model string and a choices field whose
    first entry holds a message whose content is one JSON object
    holding the decision. Content that does not parse as one JSON
    object is a failed call.
    """
    model = _read_model(envelope)
    if model is None:
        return None
    choices = envelope.get("choices")
    if not isinstance(choices, list) or not choices:
        return None
    first = choices[0]
    if not isinstance(first, dict):
        return None
    message = first.get("message")
    if not isinstance(message, dict):
        return None
    content = message.get("content")
    if not isinstance(content, str):
        return None
    try:
        decision = json.loads(content)
    except (ValueError, RecursionError):
        return None
    return _verdict_from_decision(model, decision)


def build_verdict_entry(flag, verdict):
    """Return the recorded verdict entry for one flag and verdict.

    The entry holds the line number and the pattern of the flag
    beside the model, the choice, and the confidence of the
    verdict, in the form CK-8 states.
    """
    return {
        "line": flag["line"],
        "pattern": flag["pattern"],
        "model": verdict["model"],
        "choice": verdict["choice"],
        "confidence": verdict["confidence"],
    }


def is_abstention(entry):
    """Return whether a recorded verdict is an abstention.

    A verdict is an abstention when its confidence is below one
    half, read exactly as a fraction, and a confidence of exactly
    one half records as a verdict.
    """
    return Fraction(entry["confidence"]) < ABSTAIN_THRESHOLD


def verdict_outcome(entry):
    """Return the outcome of one recorded verdict.

    One of stickler, clean, and abstained.
    """
    if is_abstention(entry):
        return "abstained"
    return entry["choice"]


def reason_for(entry):
    """Return the reason recorded beside a verdict.

    The reason is a construction and never text a model returned,
    built as the model, the word chose, the choice, the words with
    confidence, and the confidence in its shortest round trip
    text, with the words and abstained at the end when the verdict
    is an abstention.
    """
    reason = (
        f"{entry['model']} chose {entry['choice']} with confidence "
        f"{_render_confidence(entry['confidence'])}"
    )
    if is_abstention(entry):
        reason += " and abstained"
    return reason


def line_outcome(entries):
    """Return the judgement of one flagged line from its verdicts.

    One of stickler, clean, and unjudged, under CK-8. A line is
    judged stickler when at least one of its verdicts chose
    stickler without abstaining, judged clean when none did and
    at least one chose clean without abstaining, and unjudged
    otherwise.
    """
    outcomes = [verdict_outcome(entry) for entry in entries]
    if "stickler" in outcomes:
        return "stickler"
    if "clean" in outcomes:
        return "clean"
    return "unjudged"


def is_contested(entries):
    """Return whether one flagged line is contested by its verdicts.

    A line is contested under CK-9 when its verdicts hold both
    labels without abstaining, one verdict choosing stickler and
    one from another model choosing clean. The model strings
    are part of the rule, as FIX-CK-17 states, so a line whose
    two labels stand under one model only is not contested, and
    a line where one model is split and a second model holds
    either label is contested. An abstention holds no label for
    this reading, so a line where one side stands only as an
    abstention is not contested.
    """
    stickler = [
        entry
        for entry in entries
        if verdict_outcome(entry) == "stickler"
    ]
    clean = [
        entry
        for entry in entries
        if verdict_outcome(entry) == "clean"
    ]
    return any(
        one["model"] != other["model"]
        for one in stickler
        for other in clean
    )
