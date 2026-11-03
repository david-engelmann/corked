//! The verdict seat over decision models, and its envelope
//! readers.
//!
//! The seat takes one borderline flag the checker raised and
//! makes one call asking for a choice between the labels
//! stickler and clean. The readers in this module take one
//! recorded envelope, as parsed JSON, and return the verdict the
//! seat produced, or None when the call failed. The readers are
//! pure functions of their inputs and make no call themselves.
//! The obligations this module carries are CK-7 and CK-8 in
//! CONTRACT.md, in the Round 3 section.

use num_bigint::BigInt;
use num_rational::BigRational;

use crate::checker::Flag;
use crate::json::{parse_units, JsonNumber, JsonValue};

pub const LABELS: [&str; 2] = ["stickler", "clean"];

/// The two readings of each borderline pattern, clean first and
/// stickler second, as the seat presents them. The readings stand
/// verbatim in CONTRACT.md under FIX-CK-14, and a test holds the
/// seat to them.
pub const PATTERN_READINGS: [(&str, (&str, &str)); 3] = [
    (
        "near placeholder name",
        (
            "the placeholder shape names exactly six names, and a name \
             outside the six is one the checker accepts, and a short \
             name can be honest in a small scope",
            "the name is the same shrug as the named placeholders and \
             says nothing about what the value holds or what the \
             function does",
        ),
    ),
    (
        "broad handler",
        (
            "the handler names the exception it catches, and naming \
             the expected failures is the standard the bare except \
             shape states",
            "a handler on BaseException also catches the control \
             exceptions a process is stopped and interrupted with, and \
             those failures must still rise where they can be seen, so \
             naming the handler does not cure the catch all harm",
        ),
    ),
    (
        "shared member default",
        (
            "a tuple default is the standard immutable default, and \
             neither the mutable default shape nor the call default \
             shape names a tuple",
            "the list, dict, or set member is one object built when \
             the definition is read and shared by every call that \
             omits the argument, and that sharing is the harm the \
             definition time shapes name",
        ),
    ),
];

/// Return the two readings of one pattern, clean first.
pub fn pattern_readings(pattern: &str) -> Option<(&'static str, &'static str)> {
    PATTERN_READINGS
        .iter()
        .find(|(name, _readings)| *name == pattern)
        .map(|(_name, readings)| *readings)
}

/// A verdict the seat produced from one call.
#[derive(Clone, Debug, PartialEq)]
pub struct Verdict {
    pub model: String,
    pub choice: String,
    pub confidence: JsonNumber,
}

/// A recorded verdict entry, in the five field form CK-8 states.
#[derive(Clone, Debug, PartialEq)]
pub struct VerdictEntry {
    pub line: usize,
    pub pattern: String,
    pub model: String,
    pub choice: String,
    pub confidence: JsonNumber,
}

/// Return the summary form the seat presents for one flag.
///
/// The form names, in the order CK-7 states, the file the flagged
/// line stands in, the pattern the line matched, the flagged line
/// as written, the reading the flag carries, and the two readings
/// of the pattern. The routes present this same text, and the
/// chat route adds its reply form after it.
pub fn seat_state(file_name: &str, flag: &Flag) -> String {
    let (clean_reading, stickler_reading) =
        pattern_readings(&flag.pattern).expect("a flag always names one of the three patterns");
    format!(
        "A strict code reviewer flagged one added line as \
         borderline, sitting between two readings of its rules.\n\
         File: {file_name}\n\
         Pattern: {pattern}\n\
         Line: {content}\n\
         Shape reading: {reading}\n\
         Clean reading: {clean_reading}\n\
         Stickler reading: {stickler_reading}\n\
         Choose stickler when the line falls short of the standard \
         the pattern protects, and clean when it meets it.",
        pattern = flag.pattern,
        content = flag.content,
        reading = flag.reading,
    )
}

/// Return the envelope model string, or None when unreadable.
fn read_model(envelope: &JsonValue) -> Option<String> {
    let model = envelope.get("model")?.as_jstring()?;
    if model.is_empty() || model.holds_unpaired_surrogate() {
        return None;
    }
    if model.holds_line_break() {
        return None;
    }
    model.to_rust_string()
}

/// Return whether a confidence figure stands in the range from
/// zero to one, read exactly.
fn confidence_in_range(confidence: &JsonNumber) -> bool {
    match confidence {
        JsonNumber::Int(value) => *value >= BigInt::from(0) && *value <= BigInt::from(1),
        JsonNumber::Float(value) => *value >= 0.0 && *value <= 1.0,
    }
}

/// Build the verdict from one decision, or None.
///
/// The decision must hold a choice that is one of the two labels,
/// a finite confidence from zero to one, and probabilities
/// holding a finite number for each of the two labels. Anything
/// else is a call the seat cannot read, and the seat records
/// nothing. A choice holding a line break or an unpaired
/// surrogate also fails the call, so a verdict the seat produces
/// always stands clean in a recorded review. A choice equal to a
/// label holds neither, so the label reading settles it.
fn verdict_from_decision(model: &str, decision: Option<&JsonValue>) -> Option<Verdict> {
    let decision = decision?;
    if !decision.is_object() {
        return None;
    }
    let choice = decision.get("choice")?.as_jstring()?;
    if !LABELS.iter().any(|label| choice.equals(label)) {
        return None;
    }
    let choice = choice.to_rust_string()?;
    let confidence = decision.get("confidence")?.as_number()?;
    if !confidence.is_finite() || !confidence_in_range(confidence) {
        return None;
    }
    let probabilities = decision.get("probabilities")?;
    if !probabilities.is_object() {
        return None;
    }
    for label in LABELS {
        let value = probabilities.get(label)?.as_number()?;
        if !value.is_finite() {
            return None;
        }
    }
    Some(Verdict {
        model: model.to_string(),
        choice,
        confidence: confidence.clone(),
    })
}

/// Return the verdict from one Typesafe System One envelope.
///
/// The envelope holds the model string and an answers field
/// whose decision field holds the decision. Returns None when
/// the call failed, for the reasons stated in
/// verdict_from_decision.
pub fn verdict_from_jev_envelope(envelope: &JsonValue) -> Option<Verdict> {
    let model = read_model(envelope)?;
    let answers = envelope.get("answers")?;
    if !answers.is_object() {
        return None;
    }
    verdict_from_decision(&model, answers.get("decision"))
}

/// Return the verdict from one chat completion envelope.
///
/// The envelope holds the model string and a choices field whose
/// first entry holds a message whose content is one JSON object
/// holding the decision. Content that does not parse as one JSON
/// object is a failed call.
pub fn verdict_from_chat_envelope(envelope: &JsonValue) -> Option<Verdict> {
    let model = read_model(envelope)?;
    let choices = envelope.get("choices")?.as_array()?;
    let first = choices.first()?;
    if !first.is_object() {
        return None;
    }
    let message = first.get("message")?;
    if !message.is_object() {
        return None;
    }
    let content = message.get("content")?.as_jstring()?;
    let decision = parse_units(&content.0).ok()?;
    verdict_from_decision(&model, Some(&decision))
}

/// Return the recorded verdict entry for one flag and verdict.
///
/// The entry holds the line number and the pattern of the flag
/// beside the model, the choice, and the confidence of the
/// verdict, in the form CK-8 states.
pub fn build_verdict_entry(flag: &Flag, verdict: &Verdict) -> VerdictEntry {
    VerdictEntry {
        line: flag.line,
        pattern: flag.pattern.clone(),
        model: verdict.model.clone(),
        choice: verdict.choice.clone(),
        confidence: verdict.confidence.clone(),
    }
}

/// Return whether a recorded verdict is an abstention.
///
/// A verdict is an abstention when its confidence is below one
/// half, read exactly as a fraction, and a confidence of exactly
/// one half records as a verdict.
pub fn is_abstention(entry: &VerdictEntry) -> bool {
    match entry.confidence.to_rational() {
        Some(value) => value < BigRational::new(BigInt::from(1), BigInt::from(2)),
        None => match &entry.confidence {
            JsonNumber::Float(value) => *value < 0.5,
            JsonNumber::Int(_) => false,
        },
    }
}

/// Return the outcome of one recorded verdict.
///
/// One of stickler, clean, and abstained.
pub fn verdict_outcome(entry: &VerdictEntry) -> &str {
    if is_abstention(entry) {
        "abstained"
    } else {
        &entry.choice
    }
}

/// Return the reason recorded beside a verdict.
///
/// The reason is a construction and never text a model returned,
/// built as the model, the word chose, the choice, the words with
/// confidence, and the confidence in its shortest round trip
/// text, with the words and abstained at the end when the verdict
/// is an abstention.
pub fn reason_for(entry: &VerdictEntry) -> String {
    let mut reason = format!(
        "{} chose {} with confidence {}",
        entry.model,
        entry.choice,
        entry.confidence.render()
    );
    if is_abstention(entry) {
        reason.push_str(" and abstained");
    }
    reason
}

/// Return the judgement of one flagged line from its verdicts.
///
/// One of stickler, clean, and unjudged, under CK-8. A line is
/// judged stickler when at least one of its verdicts chose
/// stickler without abstaining, judged clean when none did and
/// at least one chose clean without abstaining, and unjudged
/// otherwise.
pub fn line_outcome(entries: &[VerdictEntry]) -> &'static str {
    let mut saw_clean = false;
    for entry in entries {
        match verdict_outcome(entry) {
            "stickler" => return "stickler",
            "clean" => saw_clean = true,
            _ => {}
        }
    }
    if saw_clean {
        "clean"
    } else {
        "unjudged"
    }
}

/// Return whether one flagged line is contested by its verdicts.
///
/// A line is contested under CK-9 when its verdicts hold both
/// labels without abstaining, one verdict choosing stickler and
/// one from another model choosing clean. The model strings are
/// part of the rule, as FIX-CK-17 states, so a line whose two
/// labels stand under one model only is not contested, and a
/// line where one model is split and a second model holds either
/// label is contested. An abstention holds no label for this
/// reading, so a line where one side stands only as an
/// abstention is not contested.
pub fn is_contested(entries: &[VerdictEntry]) -> bool {
    let sticklers = entries
        .iter()
        .filter(|entry| verdict_outcome(entry) == "stickler");
    let cleans: Vec<&VerdictEntry> = entries
        .iter()
        .filter(|entry| verdict_outcome(entry) == "clean")
        .collect();
    for one in sticklers {
        for other in &cleans {
            if one.model != other.model {
                return true;
            }
        }
    }
    false
}
