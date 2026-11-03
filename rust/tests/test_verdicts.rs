//! Pins for the verdict seat of CK-7.
//!
//! Both envelope shapes read through the one seat, a call the
//! seat cannot read records nothing, an abstention is a
//! confidence below one half, and the recorded reason is the
//! stated construction. The readings the seat presents stand
//! word for word in the contract under FIX-CK-14, and a pin
//! here holds them to it. Translated from
//! tests/test_verdicts.py.

use std::path::PathBuf;

use stickler::checker::Flag;
use stickler::json::{parse, JsonNumber, JsonValue};
use stickler::verdicts::{
    build_verdict_entry, is_abstention, line_outcome, reason_for, seat_state,
    verdict_from_chat_envelope, verdict_from_jev_envelope, verdict_outcome, Verdict, VerdictEntry,
    PATTERN_READINGS,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Build one jev envelope from raw JSON fragments.
fn jev_envelope(choice: &str, confidence: &str, model_json: &str) -> JsonValue {
    let text = String::new()
        + "{\"answers\": {\"decision\": {\"choice\": \""
        + choice
        + "\", \"confidence\": "
        + confidence
        + ", \"probabilities\": {\"stickler\": 0.78, \"clean\": 0.22}, \
           \"type\": \"choice\"}}, \"model\": "
        + model_json
        + ", \"usage\": {\"input_tokens\": 500, \"output_tokens\": 35}}";
    parse(&text).unwrap()
}

fn jev_default() -> JsonValue {
    jev_envelope("stickler", "0.78", "\"jev-1.13.0\"")
}

fn set_decision(envelope: &mut JsonValue, key: &str, value: JsonValue) {
    let JsonValue::Object(pairs) = envelope else {
        panic!("envelope is not an object");
    };
    let answers = pairs
        .iter_mut()
        .find(|(name, _)| name.equals("answers"))
        .map(|(_, value)| value)
        .expect("answers stands");
    let JsonValue::Object(answer_pairs) = answers else {
        panic!("answers is not an object");
    };
    let decision = answer_pairs
        .iter_mut()
        .find(|(name, _)| name.equals("decision"))
        .map(|(_, value)| value)
        .expect("decision stands");
    let JsonValue::Object(decision_pairs) = decision else {
        panic!("decision is not an object");
    };
    for (name, slot) in decision_pairs.iter_mut() {
        if name.equals(key) {
            *slot = value;
            return;
        }
    }
    decision_pairs.push((stickler::json::JString::from_text(key), value));
}

fn drop_decision(envelope: &mut JsonValue, key: &str) {
    let JsonValue::Object(pairs) = envelope else {
        panic!("envelope is not an object");
    };
    let answers = pairs
        .iter_mut()
        .find(|(name, _)| name.equals("answers"))
        .map(|(_, value)| value)
        .expect("answers stands");
    let JsonValue::Object(answer_pairs) = answers else {
        panic!("answers is not an object");
    };
    let decision = answer_pairs
        .iter_mut()
        .find(|(name, _)| name.equals("decision"))
        .map(|(_, value)| value)
        .expect("decision stands");
    let JsonValue::Object(decision_pairs) = decision else {
        panic!("decision is not an object");
    };
    decision_pairs.retain(|(name, _)| !name.equals(key));
}

/// Escape a string as JSON string content, for the chat
/// envelope contents.
fn jstr(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if (ch as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => out.push(ch),
        }
    }
    out
}

fn chat_envelope(content: Option<&str>, model: &str) -> JsonValue {
    let content_json = match content {
        Some(text) => format!("\"{}\"", jstr(text)),
        None => "null".to_string(),
    };
    let text = String::new()
        + "{\"model\": \""
        + &jstr(model)
        + "\", \"choices\": [{\"message\": {\"role\": \"assistant\", \
           \"content\": "
        + &content_json
        + "}}], \"usage\": {\"prompt_tokens\": 200, \
           \"completion_tokens\": 40}}";
    parse(&text).unwrap()
}

const GOOD_CHAT: &str = "{\"choice\": \"clean\", \"confidence\": 0.9, \
    \"probabilities\": {\"stickler\": 0.1, \"clean\": 0.9}}";

fn verdict(model: &str, choice: &str, confidence: JsonNumber) -> Verdict {
    Verdict {
        model: model.to_string(),
        choice: choice.to_string(),
        confidence,
    }
}

#[test]
fn jev_envelope_reads() {
    let read = verdict_from_jev_envelope(&jev_default());
    assert_eq!(
        read,
        Some(verdict("jev-1.13.0", "stickler", JsonNumber::Float(0.78)))
    );
}

#[test]
fn chat_envelope_reads() {
    let read = verdict_from_chat_envelope(&chat_envelope(Some(GOOD_CHAT), "openai/gpt-6-luna"));
    assert_eq!(
        read,
        Some(verdict(
            "openai/gpt-6-luna",
            "clean",
            JsonNumber::Float(0.9)
        ))
    );
}

#[test]
fn a_choice_outside_the_labels_records_nothing() {
    let envelope = jev_envelope("maybe", "0.78", "\"jev-1.13.0\"");
    assert_eq!(verdict_from_jev_envelope(&envelope), None);
}

#[test]
fn a_confidence_out_of_range_records_nothing() {
    for confidence in ["1.5", "-0.1"] {
        let envelope = jev_envelope("stickler", confidence, "\"jev-1.13.0\"");
        assert_eq!(
            verdict_from_jev_envelope(&envelope),
            None,
            "confidence {confidence}"
        );
    }
}

#[test]
fn a_non_finite_confidence_records_nothing() {
    let envelope = jev_envelope("stickler", "NaN", "\"jev-1.13.0\"");
    assert_eq!(verdict_from_jev_envelope(&envelope), None);
}

#[test]
fn an_oversized_integer_confidence_records_nothing() {
    let huge = "1".to_string() + &"0".repeat(400);
    for confidence in [huge.clone(), format!("-{huge}")] {
        let envelope = jev_envelope("stickler", &confidence, "\"jev-1.13.0\"");
        assert_eq!(verdict_from_jev_envelope(&envelope), None);
    }
}

#[test]
fn an_oversized_integer_confidence_in_a_chat_envelope() {
    let huge = "1".to_string() + &"0".repeat(400);
    let content = format!(
        "{{\"choice\": \"stickler\", \"confidence\": {huge}, \
         \"probabilities\": {{\"stickler\": 0.9, \"clean\": 0.1}}}}"
    );
    let envelope = chat_envelope(Some(&content), "openai/gpt-6-luna");
    assert_eq!(verdict_from_chat_envelope(&envelope), None);
}

#[test]
fn an_oversized_integer_probability_records_nothing() {
    let huge = "1".to_string() + &"0".repeat(400);
    let mut envelope = jev_default();
    set_decision(
        &mut envelope,
        "probabilities",
        parse(&format!("{{\"stickler\": {huge}, \"clean\": 0.1}}")).unwrap(),
    );
    assert_eq!(verdict_from_jev_envelope(&envelope), None);
    let mut envelope = jev_default();
    set_decision(
        &mut envelope,
        "probabilities",
        parse(&format!("{{\"stickler\": 0.9, \"clean\": -{huge}}}")).unwrap(),
    );
    assert_eq!(verdict_from_jev_envelope(&envelope), None);
}

#[test]
fn a_boolean_confidence_records_nothing() {
    let envelope = jev_envelope("stickler", "true", "\"jev-1.13.0\"");
    assert_eq!(verdict_from_jev_envelope(&envelope), None);
}

#[test]
fn missing_probabilities_record_nothing() {
    let mut envelope = jev_default();
    drop_decision(&mut envelope, "probabilities");
    assert_eq!(verdict_from_jev_envelope(&envelope), None);
}

#[test]
fn a_probability_missing_for_a_label_records_nothing() {
    let mut envelope = jev_default();
    set_decision(
        &mut envelope,
        "probabilities",
        parse(r#"{"stickler": 1.0}"#).unwrap(),
    );
    assert_eq!(verdict_from_jev_envelope(&envelope), None);
}

#[test]
fn a_model_holding_a_line_break_records_nothing() {
    let envelope = jev_envelope("stickler", "0.78", "\"jev\\n1.0\"");
    assert_eq!(verdict_from_jev_envelope(&envelope), None);
}

#[test]
fn a_model_holding_a_surrogate_records_nothing() {
    let envelope = jev_envelope("stickler", "0.78", "\"jev\\ud800\"");
    assert_eq!(verdict_from_jev_envelope(&envelope), None);
}

#[test]
fn missing_answers_record_nothing() {
    let bare = parse(r#"{"model": "jev"}"#).unwrap();
    assert_eq!(verdict_from_jev_envelope(&bare), None);
    let handed = parse(r#""not an envelope""#).unwrap();
    assert_eq!(verdict_from_jev_envelope(&handed), None);
    assert_eq!(verdict_from_jev_envelope(&JsonValue::Null), None);
}

#[test]
fn chat_prose_around_the_object_records_nothing() {
    let content = format!("Here is my verdict. {GOOD_CHAT}");
    let envelope = chat_envelope(Some(&content), "openai/gpt-6-luna");
    assert_eq!(verdict_from_chat_envelope(&envelope), None);
}

#[test]
fn chat_non_finite_confidence_records_nothing() {
    let content = "{\"choice\": \"clean\", \"confidence\": NaN, \
        \"probabilities\": {\"stickler\": 0.1, \"clean\": 0.9}}";
    let envelope = chat_envelope(Some(content), "openai/gpt-6-luna");
    assert_eq!(verdict_from_chat_envelope(&envelope), None);
}

#[test]
fn chat_missing_content_records_nothing() {
    let envelope = chat_envelope(None, "openai/gpt-6-luna");
    assert_eq!(verdict_from_chat_envelope(&envelope), None);
}

#[test]
fn chat_missing_choices_record_nothing() {
    let envelope = parse(r#"{"model": "openai/gpt-6-luna"}"#).unwrap();
    assert_eq!(verdict_from_chat_envelope(&envelope), None);
}

fn entry(model: &str, choice: &str, confidence: JsonNumber) -> VerdictEntry {
    VerdictEntry {
        line: 0,
        pattern: String::new(),
        model: model.to_string(),
        choice: choice.to_string(),
        confidence,
    }
}

#[test]
fn exactly_one_half_records_as_a_verdict() {
    let held = entry("m", "clean", JsonNumber::Float(0.5));
    assert!(!is_abstention(&held));
    assert_eq!(verdict_outcome(&held), "clean");
}

#[test]
fn below_one_half_abstains() {
    let held = entry("m", "stickler", JsonNumber::Float(0.49));
    assert!(is_abstention(&held));
    assert_eq!(verdict_outcome(&held), "abstained");
}

#[test]
fn an_abstention_keeps_its_choice_out_of_the_outcome() {
    let held = entry("m", "stickler", JsonNumber::Float(0.2));
    assert_eq!(verdict_outcome(&held), "abstained");
}

#[test]
fn the_reason_is_the_stated_construction() {
    let held = entry("m", "stickler", JsonNumber::Float(0.78));
    assert_eq!(reason_for(&held), "m chose stickler with confidence 0.78");
}

#[test]
fn the_abstention_reason_names_the_abstention() {
    let held = entry("m", "clean", JsonNumber::Float(0.2));
    assert_eq!(
        reason_for(&held),
        "m chose clean with confidence 0.2 and abstained"
    );
}

#[test]
fn a_whole_confidence_renders_without_a_point() {
    let held = entry("m", "stickler", JsonNumber::Int(1.into()));
    assert_eq!(reason_for(&held), "m chose stickler with confidence 1");
}

#[test]
fn the_seat_state_presents_the_case_in_order() {
    let flag = Flag {
        line: 6,
        pattern: "near placeholder name".to_string(),
        content: "data = load_entries()".to_string(),
        reading: "drawn name entries".to_string(),
    };
    let state = seat_state("harvest.py", &flag);
    let needles = [
        "harvest.py",
        "near placeholder name",
        "data = load_entries()",
        "drawn name entries",
        "Clean reading:",
        "Stickler reading:",
    ];
    let positions: Vec<usize> = needles
        .iter()
        .map(|needle| state.find(needle).expect("the needle stands"))
        .collect();
    let mut sorted = positions.clone();
    sorted.sort_unstable();
    assert_eq!(positions, sorted);
}

#[test]
fn the_verdict_entry_holds_exactly_five_fields() {
    let flag = Flag {
        line: 6,
        pattern: "broad handler".to_string(),
        content: "x".to_string(),
        reading: "handler names BaseException".to_string(),
    };
    let held = verdict("m", "clean", JsonNumber::Float(0.9));
    let built = build_verdict_entry(&flag, &held);
    assert_eq!(
        built,
        VerdictEntry {
            line: 6,
            pattern: "broad handler".to_string(),
            model: "m".to_string(),
            choice: "clean".to_string(),
            confidence: JsonNumber::Float(0.9),
        }
    );
}

#[test]
fn the_line_outcome_prefers_a_stickler_verdict() {
    let entries = vec![
        entry("a", "clean", JsonNumber::Float(0.9)),
        entry("b", "stickler", JsonNumber::Float(0.6)),
    ];
    assert_eq!(line_outcome(&entries), "stickler");
}

#[test]
fn a_line_with_only_abstentions_is_unjudged() {
    let entries = vec![
        entry("a", "stickler", JsonNumber::Float(0.2)),
        entry("b", "clean", JsonNumber::Float(0.1)),
    ];
    assert_eq!(line_outcome(&entries), "unjudged");
    assert_eq!(line_outcome(&[]), "unjudged");
}

#[test]
fn a_clean_verdict_beside_an_abstention_judges_clean() {
    let entries = vec![
        entry("a", "clean", JsonNumber::Float(0.9)),
        entry("b", "stickler", JsonNumber::Float(0.2)),
    ];
    assert_eq!(line_outcome(&entries), "clean");
}

#[test]
fn each_presented_reading_stands_verbatim_in_the_contract() {
    let text = std::fs::read_to_string(root().join("CONTRACT.md")).unwrap();
    for (pattern, readings) in PATTERN_READINGS {
        for reading in [readings.0, readings.1] {
            assert!(text.contains(reading), "pattern {pattern}");
        }
    }
}
