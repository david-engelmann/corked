//! Pins for the contested line reading of CK-9 and its
//! example.
//!
//! A flagged line is contested when its verdicts hold both
//! labels without abstaining. An abstention never contests, a
//! single model never contests, and two verdicts from one
//! model are the defect of CK-8 and never reach the reading.
//! The example held here is the constructed example of CK-10,
//! named as constructed in the contract and here, because the
//! recorded pilot of CK-8 holds no contested line. No verdict
//! in this file is a recorded one. Translated from
//! tests/test_contested.py.

use std::collections::BTreeMap;
use std::path::PathBuf;

use stickler::changeset::{load_changeset, review_changeset};
use stickler::cli::cli_main;
use stickler::json::{parse, JString, JsonNumber, JsonValue};
use stickler::render::render_set_review;
use stickler::verdicts::{is_contested, line_outcome, VerdictEntry};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn examples() -> PathBuf {
    root().join("examples")
}

const JEV: &str = "jev-1.13.0";
const OPENAI: &str = "openai/gpt-6-luna";

const ONE_FLAG_DIFF: &str = "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,1 @@\n+data = load_entries()\n";

const TWO_FLAG_DIFF: &str = "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,3 @@\n\
    +data = load_entries()\n+temp = []\n";

fn entry(choice: &str, confidence: f64, model: &str, line: usize) -> VerdictEntry {
    VerdictEntry {
        line,
        pattern: "near placeholder name".to_string(),
        model: model.to_string(),
        choice: choice.to_string(),
        confidence: JsonNumber::Float(confidence),
    }
}

fn verdict_json(held: &VerdictEntry) -> JsonValue {
    JsonValue::Object(vec![
        (
            JString::from_text("line"),
            JsonValue::Number(JsonNumber::Int((held.line as i64).into())),
        ),
        (
            JString::from_text("pattern"),
            JsonValue::Str(JString::from_text(&held.pattern)),
        ),
        (
            JString::from_text("model"),
            JsonValue::Str(JString::from_text(&held.model)),
        ),
        (
            JString::from_text("choice"),
            JsonValue::Str(JString::from_text(&held.choice)),
        ),
        (
            JString::from_text("confidence"),
            JsonValue::Number(held.confidence.clone()),
        ),
    ])
}

fn flagged_set(verdicts: &[VerdictEntry], diff: &str, name: &str) -> JsonValue {
    JsonValue::Object(vec![(
        JString::from_text("files"),
        JsonValue::Array(vec![JsonValue::Object(vec![
            (
                JString::from_text("file"),
                JsonValue::Str(JString::from_text(name)),
            ),
            (
                JString::from_text("diff"),
                JsonValue::Str(JString::from_text(diff)),
            ),
            (
                JString::from_text("verdicts"),
                JsonValue::Array(verdicts.iter().map(verdict_json).collect()),
            ),
        ])]),
    )])
}

fn render(value: &JsonValue) -> String {
    render_set_review(&review_changeset(value))
}

#[test]
fn both_labels_without_abstaining_contest() {
    let entries = vec![
        entry("stickler", 0.81, JEV, 4),
        entry("clean", 0.77, OPENAI, 4),
    ];
    assert!(is_contested(&entries));
}

#[test]
fn a_confidence_of_exactly_one_half_holds_its_label() {
    assert!(is_contested(&[
        entry("stickler", 0.5, JEV, 4),
        entry("clean", 0.9, OPENAI, 4),
    ]));
    assert!(is_contested(&[
        entry("stickler", 0.5, JEV, 4),
        entry("clean", 0.5, OPENAI, 4),
    ]));
}

#[test]
fn one_label_alone_never_contests() {
    assert!(!is_contested(&[entry("stickler", 0.9, JEV, 4)]));
    assert!(!is_contested(&[entry("clean", 0.9, JEV, 4)]));
    assert!(!is_contested(&[]));
    assert!(!is_contested(&[
        entry("stickler", 0.9, JEV, 4),
        entry("stickler", 0.8, OPENAI, 4),
    ]));
    assert!(!is_contested(&[
        entry("clean", 0.9, JEV, 4),
        entry("clean", 0.8, OPENAI, 4),
    ]));
}

#[test]
fn one_model_holding_both_labels_never_contests() {
    assert!(!is_contested(&[
        entry("stickler", 0.9, JEV, 4),
        entry("clean", 0.8, JEV, 4),
    ]));
    assert!(!is_contested(&[
        entry("clean", 0.8, JEV, 4),
        entry("stickler", 0.9, JEV, 4),
    ]));
}

#[test]
fn a_second_model_holding_either_label_still_contests() {
    assert!(is_contested(&[
        entry("stickler", 0.9, JEV, 4),
        entry("clean", 0.8, JEV, 4),
        entry("stickler", 0.7, OPENAI, 4),
    ]));
    assert!(is_contested(&[
        entry("stickler", 0.9, JEV, 4),
        entry("clean", 0.8, JEV, 4),
        entry("clean", 0.7, OPENAI, 4),
    ]));
}

#[test]
fn one_model_holding_both_labels_renders_no_contested_section() {
    let rendered = render(&flagged_set(
        &[entry("stickler", 0.9, JEV, 4), entry("clean", 0.8, JEV, 4)],
        ONE_FLAG_DIFF,
        "x.py",
    ));
    assert!(!rendered.contains("Contested lines"));
    assert!(rendered.contains("Contested, 0"));
}

#[test]
fn an_abstention_never_contests() {
    assert!(!is_contested(&[
        entry("stickler", 0.9, JEV, 4),
        entry("clean", 0.49, OPENAI, 4),
    ]));
    assert!(!is_contested(&[
        entry("stickler", 0.49, JEV, 4),
        entry("clean", 0.9, OPENAI, 4),
    ]));
    assert!(!is_contested(&[
        entry("stickler", 0.2, JEV, 4),
        entry("clean", 0.3, OPENAI, 4),
    ]));
}

#[test]
fn an_abstention_beside_the_contenders_still_contests() {
    let entries = vec![
        entry("stickler", 0.9, JEV, 4),
        entry("clean", 0.8, OPENAI, 4),
        entry("stickler", 0.3, "a-third-model", 4),
    ];
    assert!(is_contested(&entries));
}

#[test]
fn a_contested_rendering_is_exact() {
    let value = flagged_set(
        &[
            entry("stickler", 0.81, JEV, 4),
            entry("clean", 0.77, OPENAI, 4),
        ],
        ONE_FLAG_DIFF,
        "x.py",
    );
    assert_eq!(
        render(&value),
        "Change set review\n\
         Files, 1\n\
         \n\
         File, x.py\n\
         \n\
         Borderline 1, near placeholder name\n\
         Line, data = load_entries()\n\
         Shape reading, drawn name entries\n\
         Verdict, jev-1.13.0, stickler\n\
         Reason, jev-1.13.0 chose stickler with confidence 0.81\n\
         Verdict, openai/gpt-6-luna, clean\n\
         Reason, openai/gpt-6-luna chose clean with confidence 0.77\n\
         \n\
         Contested lines\n\
         \n\
         File, x.py\n\
         Borderline 1, near placeholder name\n\
         Line, data = load_entries()\n\
         Verdict, jev-1.13.0, stickler\n\
         Verdict, openai/gpt-6-luna, clean\n\
         \n\
         Summary\n\
         Judgements, 0\n\
         Borderline, 1\n\
         Verdict lines, stickler 1, clean 0\n\
         Unjudged, 0\n\
         Contested, 1\n"
    );
}

#[test]
fn the_section_names_each_verdict_in_standing_order() {
    let value = flagged_set(
        &[
            entry("clean", 0.77, OPENAI, 4),
            entry("stickler", 0.81, JEV, 4),
            entry("stickler", 0.3, "a-third-model", 4),
        ],
        ONE_FLAG_DIFF,
        "x.py",
    );
    let rendered = render(&value);
    let section = rendered.split("Contested lines\n").nth(1).unwrap();
    let verdict_lines: Vec<&str> = section
        .lines()
        .filter(|line| line.starts_with("Verdict, "))
        .collect();
    assert_eq!(
        verdict_lines,
        vec![
            "Verdict, openai/gpt-6-luna, clean",
            "Verdict, jev-1.13.0, stickler",
            "Verdict, a-third-model, abstained",
        ]
    );
}

#[test]
fn the_section_uses_the_blocks_own_numbers() {
    let verdicts = vec![
        entry("stickler", 0.9, JEV, 4),
        entry("clean", 0.8, JEV, 5),
        entry("stickler", 0.7, OPENAI, 5),
    ];
    let rendered = render(&flagged_set(&verdicts, TWO_FLAG_DIFF, "x.py"));
    let section = rendered.split("Contested lines\n").nth(1).unwrap();
    assert!(section.contains("Borderline 2, near placeholder name"));
    assert!(!section.contains("Borderline 1, near placeholder name"));
    assert!(rendered.contains("Contested, 1"));
}

#[test]
fn flags_without_a_contest_count_zero_and_stand_no_section() {
    let rendered = render(&flagged_set(
        &[entry("stickler", 0.9, JEV, 4)],
        ONE_FLAG_DIFF,
        "x.py",
    ));
    assert!(rendered.contains("Contested, 0"));
    assert!(!rendered.contains("Contested lines"));
}

#[test]
fn an_abstention_beside_a_stickler_verdict_is_not_contested() {
    let rendered = render(&flagged_set(
        &[
            entry("stickler", 0.9, JEV, 4),
            entry("clean", 0.3, OPENAI, 4),
        ],
        ONE_FLAG_DIFF,
        "x.py",
    ));
    assert!(!rendered.contains("Contested lines"));
    assert!(rendered.contains("Verdict lines, stickler 1, clean 0"));
    assert!(rendered.contains("Contested, 0"));
}

#[test]
fn an_abstention_beside_a_clean_verdict_is_not_contested() {
    let rendered = render(&flagged_set(
        &[
            entry("stickler", 0.3, JEV, 4),
            entry("clean", 0.9, OPENAI, 4),
        ],
        ONE_FLAG_DIFF,
        "x.py",
    ));
    assert!(!rendered.contains("Contested lines"));
    assert!(rendered.contains("Verdict lines, stickler 0, clean 1"));
    assert!(rendered.contains("Contested, 0"));
}

#[test]
fn a_flagless_set_renders_no_contested_text() {
    let diff = "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,1 @@\n+    print(total)\n";
    let value = JsonValue::Object(vec![(
        JString::from_text("files"),
        JsonValue::Array(vec![JsonValue::Object(vec![
            (
                JString::from_text("file"),
                JsonValue::Str(JString::from_text("x.py")),
            ),
            (
                JString::from_text("diff"),
                JsonValue::Str(JString::from_text(diff)),
            ),
        ])]),
    )]);
    assert!(!render(&value).contains("Contested"));
}

#[test]
fn two_verdicts_from_one_model_are_a_defect_and_never_render() {
    let value = flagged_set(
        &[entry("stickler", 0.9, JEV, 4), entry("clean", 0.8, JEV, 4)],
        ONE_FLAG_DIFF,
        "x.py",
    );
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].verdicts[1] repeats a verdict for the same \
             line and model"
            .to_string()]
    );
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "stickler-test-{}-{}-{}",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Serialize a value as JSON text, the way the Python tests
/// hand a set to the command line through json.dumps.
fn to_json_text(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(true) => "true".to_string(),
        JsonValue::Bool(false) => "false".to_string(),
        JsonValue::Number(number) => number.render(),
        JsonValue::Str(text) => {
            let plain = text.to_rust_string().expect("plain text");
            let mut out = String::from("\"");
            for ch in plain.chars() {
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
            out.push('"');
            out
        }
        JsonValue::Array(items) => {
            let parts: Vec<String> = items.iter().map(to_json_text).collect();
            format!("[{}]", parts.join(", "))
        }
        JsonValue::Object(pairs) => {
            let parts: Vec<String> = pairs
                .iter()
                .map(|(key, item)| {
                    let key_text = JsonValue::Str(key.clone());
                    format!("{}: {}", to_json_text(&key_text), to_json_text(item))
                })
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
    }
}

#[test]
fn two_verdicts_from_one_model_exit_2() {
    let value = flagged_set(
        &[entry("stickler", 0.9, JEV, 4), entry("clean", 0.8, JEV, 4)],
        ONE_FLAG_DIFF,
        "x.py",
    );
    let dir = temp_dir("contested-exit");
    let path = dir.join("set.json");
    std::fs::write(&path, to_json_text(&value)).unwrap();
    let result = cli_main(&["review".to_string(), path.to_string_lossy().into_owned()]);
    assert_eq!(result.code, 2);
    assert!(result.stdout.contains("Verdict, invalid"));
    assert!(!result.stdout.contains("Contested"));
    let _ = std::fs::remove_dir_all(&dir);
}

fn load_example() -> JsonValue {
    let text = std::fs::read_to_string(examples().join("contested_set.json")).unwrap();
    parse(&text).unwrap()
}

#[test]
fn the_constructed_example_is_a_valid_change_set() {
    let value = load_example();
    assert_eq!(load_changeset(&value), Vec::<String>::new());
    let mut judgements = 0;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for result in review_changeset(&value) {
        judgements += result.judgements.len();
        for block in &result.blocks {
            *counts.entry(block.flag.pattern.clone()).or_insert(0) += 1;
        }
    }
    assert_eq!(judgements, 0);
    assert_eq!(counts.len(), 3);
    assert_eq!(counts["near placeholder name"], 4);
    assert_eq!(counts["broad handler"], 1);
    assert_eq!(counts["shared member default"], 1);
}

#[test]
fn the_constructed_example_outcomes_match_the_hand_count() {
    let value = load_example();
    let mut outcomes: BTreeMap<&str, usize> = BTreeMap::new();
    let mut contested: Vec<(String, usize)> = Vec::new();
    for result in review_changeset(&value) {
        for block in &result.blocks {
            *outcomes.entry(line_outcome(&block.verdicts)).or_insert(0) += 1;
            if is_contested(&block.verdicts) {
                contested.push((result.name.clone(), block.flag.line));
            }
        }
    }
    assert_eq!(outcomes["stickler"], 3);
    assert_eq!(outcomes["clean"], 2);
    assert_eq!(outcomes["unjudged"], 1);
    assert_eq!(
        contested,
        vec![("ledger.py".to_string(), 6), ("cache.py".to_string(), 15),]
    );
}

#[test]
fn the_constructed_example_entries_carry_the_recorded_models() {
    let value = load_example();
    let mut total = 0;
    let files = value
        .get("files")
        .and_then(|files| files.as_array())
        .expect("files stands as a list");
    for held in files {
        let Some(verdicts) = held.get("verdicts").and_then(|v| v.as_array()) else {
            continue;
        };
        for verdict in verdicts {
            total += 1;
            let verdict_pairs = verdict.as_object().expect("a verdict is an object");
            let mut keys: Vec<String> = verdict_pairs
                .iter()
                .map(|(name, _)| name.to_rust_string().expect("a plain key"))
                .collect();
            keys.sort_unstable();
            assert_eq!(
                keys,
                vec![
                    "choice".to_string(),
                    "confidence".to_string(),
                    "line".to_string(),
                    "model".to_string(),
                    "pattern".to_string(),
                ]
            );
            let model = verdict
                .get("model")
                .and_then(|model| model.as_jstring())
                .and_then(|model| model.to_rust_string())
                .expect("model stands as a string");
            assert!(model == JEV || model == OPENAI, "model {model}");
        }
    }
    assert_eq!(total, 10);
}

#[test]
fn the_constructed_example_matches_its_golden_rendering() {
    let rendered = render(&load_example());
    let golden = std::fs::read_to_string(examples().join("contested_review.txt")).unwrap();
    assert_eq!(rendered, golden);
}

#[test]
fn the_constructed_example_replays_to_exit_1() {
    let path = examples().join("contested_set.json");
    let result = cli_main(&["review".to_string(), path.to_string_lossy().into_owned()]);
    let golden = std::fs::read_to_string(examples().join("contested_review.txt")).unwrap();
    assert_eq!(result.code, 1);
    assert_eq!(result.stdout, golden);
}

#[test]
fn the_recorded_pilot_holds_no_contested_line() {
    let text = std::fs::read_to_string(examples().join("borderline_review.json")).unwrap();
    let pilot = parse(&text).unwrap();
    let mut flag_count = 0;
    for result in review_changeset(&pilot) {
        for block in &result.blocks {
            flag_count += 1;
            assert!(!is_contested(&block.verdicts));
        }
    }
    assert_eq!(flag_count, 12);
    let rendered = render_set_review(&review_changeset(&pilot));
    assert!(rendered.ends_with("Unjudged, 1\nContested, 0\n"));
    assert!(!rendered.contains("Contested lines"));
}
