//! Pins for the recorded borderline pilot of CK-8.
//!
//! The suite runs offline. These tests replay the recorded
//! envelopes through the same seat readers the live run used
//! and rebuild the pilot review exactly, recompute the run
//! record from the envelope fixtures, and hold the pilot
//! review against its stored rendering. Translated from
//! tests/test_pilot.py.

use std::collections::BTreeMap;
use std::path::PathBuf;

use num_bigint::BigInt;
use stickler::changeset::{load_changeset, review_changeset};
use stickler::checker::Flag;
use stickler::cli::cli_main;
use stickler::json::{parse, JsonNumber, JsonValue};
use stickler::render::render_set_review;
use stickler::verdicts::{
    build_verdict_entry, verdict_from_chat_envelope, verdict_from_jev_envelope, verdict_outcome,
    Verdict, VerdictEntry,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn examples() -> PathBuf {
    root().join("examples")
}

fn envelopes() -> PathBuf {
    examples().join("verdicts").join("envelopes")
}

fn load_json(path: &PathBuf) -> JsonValue {
    let text = std::fs::read_to_string(path).unwrap();
    parse(&text).unwrap()
}

type Reader = fn(&JsonValue) -> Option<Verdict>;

const ROUTES: [(&str, &str, Reader, &str, &str); 2] = [
    (
        "jev",
        "jev-1.13.0",
        verdict_from_jev_envelope,
        "input_tokens",
        "output_tokens",
    ),
    (
        "openai",
        "openai/gpt-6-luna",
        verdict_from_chat_envelope,
        "prompt_tokens",
        "completion_tokens",
    ),
];

fn pilot_flags() -> Vec<(usize, String, Flag)> {
    let value = load_json(&examples().join("borderline_set.json"));
    let mut flags = Vec::new();
    for (index, result) in review_changeset(&value).into_iter().enumerate() {
        for block in &result.blocks {
            flags.push((index + 1, result.name.clone(), block.flag.clone()));
        }
    }
    flags
}

fn envelope_name(file_index: usize, flag: &Flag, suffix: &str) -> String {
    format!("f{file_index}-l{:02}-{suffix}.json", flag.line)
}

type EntriesByFile = BTreeMap<String, Vec<VerdictEntry>>;
type OutcomesByLine = BTreeMap<(String, usize, String), Option<Verdict>>;

fn replay_entries() -> (EntriesByFile, OutcomesByLine) {
    let mut per_file: EntriesByFile = BTreeMap::new();
    let mut outcomes: OutcomesByLine = BTreeMap::new();
    for (file_index, name, flag) in pilot_flags() {
        for (suffix, _model, reader, _ki, _ko) in ROUTES {
            let envelope = load_json(&envelopes().join(envelope_name(file_index, &flag, suffix)));
            let verdict = reader(&envelope);
            let key = (name.clone(), flag.line, suffix.to_string());
            if let Some(held) = &verdict {
                per_file
                    .entry(name.clone())
                    .or_default()
                    .push(build_verdict_entry(&flag, held));
            }
            outcomes.insert(key, verdict);
        }
    }
    (per_file, outcomes)
}

fn entry_of(value: &JsonValue) -> VerdictEntry {
    let line = match value.get("line").and_then(|v| v.as_number()) {
        Some(JsonNumber::Int(b)) => b.to_string().parse().unwrap(),
        _ => panic!("line stands as an integer"),
    };
    let field = |key: &str| {
        value
            .get(key)
            .and_then(|v| v.as_jstring())
            .and_then(|v| v.to_rust_string())
            .unwrap_or_else(|| panic!("{key} stands as a string"))
    };
    VerdictEntry {
        line,
        pattern: field("pattern"),
        model: field("model"),
        choice: field("choice"),
        confidence: value
            .get("confidence")
            .and_then(|v| v.as_number())
            .expect("confidence stands as a number")
            .clone(),
    }
}

fn int_of(value: &JsonValue) -> i64 {
    match value.as_number() {
        Some(JsonNumber::Int(b)) => b.to_string().parse().unwrap(),
        _ => panic!("an integer stands"),
    }
}

/// Read a rendered float as a decimal pair, coefficient and
/// power of ten, the way the Python recompute reads
/// Decimal(str(cost)).
fn decimal_of(rendered: &str) -> (BigInt, i32) {
    let (mantissa, exponent) = match rendered.find(['e', 'E']) {
        Some(at) => (&rendered[..at], rendered[at + 1..].parse::<i32>().unwrap()),
        None => (rendered, 0),
    };
    let (digits, fraction) = match mantissa.find('.') {
        Some(at) => (
            format!("{}{}", &mantissa[..at], &mantissa[at + 1..]),
            (mantissa.len() - at - 1) as i32,
        ),
        None => (mantissa.to_string(), 0),
    };
    let coefficient = BigInt::parse_bytes(digits.as_bytes(), 10).expect("digits stand");
    (coefficient, exponent - fraction)
}

/// Sum rendered decimals and spell the total the way the
/// Python format(cost, "f") spells it.
fn decimal_sum(rendered_terms: &[String]) -> String {
    let terms: Vec<(BigInt, i32)> = rendered_terms.iter().map(|term| decimal_of(term)).collect();
    let floor = terms.iter().map(|(_, exp)| *exp).min().unwrap_or(0);
    let mut total = BigInt::from(0);
    for (coefficient, exp) in &terms {
        let mut scaled = coefficient.clone();
        for _ in 0..(exp - floor) {
            scaled *= 10;
        }
        total += scaled;
    }
    let digits = total.to_string();
    if floor >= 0 {
        let mut out = digits;
        for _ in 0..floor {
            out.push('0');
        }
        out
    } else {
        let places = (-floor) as usize;
        if digits.len() > places {
            let at = digits.len() - places;
            format!("{}.{}", &digits[..at], &digits[at..])
        } else {
            let mut out = "0.".to_string();
            for _ in 0..(places - digits.len()) {
                out.push('0');
            }
            out.push_str(&digits);
            out
        }
    }
}

#[test]
fn the_pilot_set_carries_twelve_flags_in_the_stated_split() {
    let value = load_json(&examples().join("borderline_set.json"));
    assert_eq!(load_changeset(&value), Vec::<String>::new());
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_judgements = 0;
    for result in review_changeset(&value) {
        total_judgements += result.judgements.len();
        for block in &result.blocks {
            *counts.entry(block.flag.pattern.clone()).or_insert(0) += 1;
        }
    }
    assert_eq!(total_judgements, 0);
    assert_eq!(counts.len(), 3);
    assert_eq!(counts["near placeholder name"], 5);
    assert_eq!(counts["broad handler"], 3);
    assert_eq!(counts["shared member default"], 4);
}

#[test]
fn the_envelope_inventory_matches_the_flags() {
    let mut names: Vec<String> = std::fs::read_dir(envelopes())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".json"))
        .collect();
    names.sort();
    let mut expected: Vec<String> = Vec::new();
    for (index, _name, flag) in pilot_flags() {
        for (suffix, _model, _reader, _ki, _ko) in ROUTES {
            expected.push(envelope_name(index, &flag, suffix));
        }
    }
    expected.sort();
    assert_eq!(names, expected);
}

#[test]
fn the_pilot_review_rebuilds_from_the_envelopes() {
    let (per_file, _outcomes) = replay_entries();
    let review = load_json(&examples().join("borderline_review.json"));
    assert_eq!(load_changeset(&review), Vec::<String>::new());
    let files = review
        .get("files")
        .and_then(|files| files.as_array())
        .expect("files stands");
    for held in files {
        let name = held
            .get("file")
            .and_then(|file| file.as_jstring())
            .and_then(|file| file.to_rust_string())
            .expect("file stands");
        let recorded: Vec<VerdictEntry> = held
            .get("verdicts")
            .and_then(|verdicts| verdicts.as_array())
            .map(|verdicts| verdicts.iter().map(entry_of).collect())
            .unwrap_or_default();
        let rebuilt = per_file.get(&name).cloned().unwrap_or_default();
        assert_eq!(recorded, rebuilt, "file {name}");
    }
}

#[test]
fn the_run_record_recomputes_from_the_envelopes() {
    let record = load_json(&examples().join("verdicts").join("run_record.json"));
    let (_per_file, outcomes) = replay_entries();
    let flags = pilot_flags();
    let routes = record
        .get("routes")
        .and_then(|routes| routes.as_array())
        .expect("routes stands");
    let mut by_model: BTreeMap<String, &JsonValue> = BTreeMap::new();
    for route in routes {
        let model = route
            .get("model")
            .and_then(|model| model.as_jstring())
            .and_then(|model| model.to_rust_string())
            .expect("model stands");
        by_model.insert(model, route);
    }
    assert_eq!(by_model.len(), 2);
    assert!(by_model.contains_key("jev-1.13.0"));
    assert!(by_model.contains_key("openai/gpt-6-luna"));
    for (suffix, model, _reader, key_in, key_out) in ROUTES {
        let route = by_model[model];
        let (mut attempted, mut recorded, mut failed, mut abstained) = (0i64, 0i64, 0i64, 0i64);
        let (mut tin, mut tout) = (0i64, 0i64);
        let mut costs: Vec<String> = Vec::new();
        for (file_index, name, flag) in &flags {
            let envelope = load_json(&envelopes().join(envelope_name(*file_index, flag, suffix)));
            attempted += 1;
            let verdict = &outcomes[&(name.clone(), flag.line, suffix.to_string())];
            let Some(verdict) = verdict else {
                failed += 1;
                continue;
            };
            recorded += 1;
            let held = build_verdict_entry(flag, verdict);
            if verdict_outcome(&held) == "abstained" {
                abstained += 1;
            }
            let usage = envelope.get("usage").expect("usage stands");
            tin += int_of(usage.get(key_in).expect("input stands"));
            tout += int_of(usage.get(key_out).expect("output stands"));
            if let Some(cost) = usage.get("cost") {
                let rendered = cost.as_number().expect("cost is a number").render();
                costs.push(rendered);
            }
        }
        assert_eq!(int_of(route.get("calls_attempted").unwrap()), attempted);
        assert_eq!(attempted, 12);
        assert_eq!(int_of(route.get("verdicts_recorded").unwrap()), recorded);
        assert_eq!(int_of(route.get("calls_failed").unwrap()), failed);
        assert_eq!(int_of(route.get("abstentions").unwrap()), abstained);
        assert_eq!(int_of(route.get("input_tokens").unwrap()), tin);
        assert_eq!(int_of(route.get("output_tokens").unwrap()), tout);
        if model.starts_with("openai") {
            let recorded_cost = route
                .get("cost_dollars")
                .and_then(|cost| cost.as_jstring())
                .and_then(|cost| cost.to_rust_string())
                .expect("cost_dollars stands");
            assert_eq!(recorded_cost, decimal_sum(&costs));
        } else {
            assert!(route.get("cost_dollars").is_none());
        }
    }
}

#[test]
fn the_failed_calls_record_nothing() {
    let (_per_file, outcomes) = replay_entries();
    let failed: Vec<&(String, usize, String)> = outcomes
        .iter()
        .filter(|(_, verdict)| verdict.is_none())
        .map(|(key, _)| key)
        .collect();
    assert_eq!(failed.len(), 2);
    assert!(failed.iter().all(|(_, _, suffix)| suffix == "openai"));
}

#[test]
fn the_pilot_review_matches_its_stored_rendering() {
    let review = load_json(&examples().join("borderline_review.json"));
    let rendered = render_set_review(&review_changeset(&review));
    let golden = std::fs::read_to_string(examples().join("borderline_review.txt")).unwrap();
    assert_eq!(rendered, golden);
}

#[test]
fn the_pilot_review_replays_to_its_recorded_exit_code() {
    let path = examples().join("borderline_review.json");
    let result = cli_main(&["review".to_string(), path.to_string_lossy().into_owned()]);
    let golden = std::fs::read_to_string(examples().join("borderline_review.txt")).unwrap();
    assert_eq!(result.code, 1);
    assert_eq!(result.stdout, golden);
}

#[test]
fn the_pilot_summary_counts_zero_contested_lines() {
    let review = load_json(&examples().join("borderline_review.json"));
    let rendered = render_set_review(&review_changeset(&review));
    assert!(rendered.ends_with("Unjudged, 1\nContested, 0\n"));
    assert!(!rendered.contains("Contested lines"));
}
