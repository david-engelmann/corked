//! Pins for the rendering shared by every command, and the
//! golden files. Translated from tests/test_render.py.

use std::path::PathBuf;

use stickler::changeset::{check_changeset, review_changeset};
use stickler::json::{parse, JString, JsonValue};
use stickler::render::{render_review, render_set_defects, render_set_review};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn obj(pairs: Vec<(&str, JsonValue)>) -> JsonValue {
    JsonValue::Object(
        pairs
            .into_iter()
            .map(|(key, value)| (JString::from_text(key), value))
            .collect(),
    )
}

fn text(value: &str) -> JsonValue {
    JsonValue::Str(JString::from_text(value))
}

const OBJECTION: &str = r#"{
    "flaw": "The name says nothing about what the value holds",
    "standard": "A name says what its value holds",
    "rewrite": "orders = load_orders()",
    "category": "naming"
}"#;

fn review(label: &str, objections: JsonValue) -> JsonValue {
    obj(vec![
        ("diff_label", text(label)),
        ("objections", objections),
    ])
}

#[test]
fn accepted_rendering_is_exact() {
    let value = review("tidy change", JsonValue::Array(vec![]));
    assert_eq!(
        render_review(&value),
        "Review of tidy change\nVerdict, accepted\nObjections, 0\n"
    );
}

#[test]
fn sent_back_rendering_is_exact() {
    let objection = parse(OBJECTION).unwrap();
    let value = review("rough change", JsonValue::Array(vec![objection]));
    assert_eq!(
        render_review(&value),
        "Review of rough change\n\
         Verdict, sent back\n\
         Objections, 1\n\
         \n\
         Objection 1, naming\n\
         Flaw, The name says nothing about what the value holds\n\
         Standard, A name says what its value holds\n\
         Rewrite, orders = load_orders()\n"
    );
}

#[test]
fn rendering_is_deterministic() {
    let objection = parse(OBJECTION).unwrap();
    let value = review("rough change", JsonValue::Array(vec![objection]));
    assert_eq!(render_review(&value), render_review(&value));
}

#[test]
fn invalid_rendering_names_every_defect() {
    let value = parse(r#"{"objections": [{"flaw": "x"}]}"#).unwrap();
    let rendered = render_review(&value);
    assert!(rendered.contains("Verdict, invalid\n"));
    assert!(rendered.contains("Defect, diff_label is missing\n"));
    assert!(rendered.contains("Defect, objections[0].standard is missing\n"));
    assert!(rendered.contains("Defect, objections[0].rewrite is missing\n"));
    assert!(rendered.contains("Defect, objections[0].category is missing\n"));
}

#[test]
fn a_review_whose_label_holds_a_surrogate_renders_unnamed() {
    let value = parse(r#"{"diff_label": "\ud800x", "objections": []}"#).unwrap();
    let rendered = render_review(&value);
    assert!(rendered.starts_with("Review of unnamed review\n"));
    assert!(rendered.contains("Verdict, invalid\n"));
    assert!(rendered.contains("Defect, diff_label holds an unpaired surrogate\n"));
}

#[test]
fn a_review_whose_label_holds_a_line_break_renders_unnamed() {
    let value = obj(vec![
        ("diff_label", text("evil\nForged, 99")),
        ("objections", JsonValue::Array(vec![])),
    ]);
    let rendered = render_review(&value);
    assert!(rendered.starts_with("Review of unnamed review\n"));
    assert!(rendered.contains("Verdict, invalid\n"));
    assert!(rendered.contains("Defect, diff_label holds a line break\n"));
    assert!(!rendered.contains("Forged"));
}

#[test]
fn recorded_review_matches_its_golden_rendering() {
    let recorded_text =
        std::fs::read_to_string(root().join("examples").join("recorded_review.json")).unwrap();
    let golden =
        std::fs::read_to_string(root().join("examples").join("recorded_review.txt")).unwrap();
    let recorded = parse(&recorded_text).unwrap();
    assert_eq!(render_review(&recorded), golden);
}

fn set_entry(name: &str, added_lines: &[&str]) -> JsonValue {
    let mut lines = vec![
        "--- a/x.py".to_string(),
        "+++ b/x.py".to_string(),
        "@@ -1,1 +1,1 @@".to_string(),
    ];
    for line in added_lines {
        lines.push(format!("+{line}"));
    }
    let mut diff = lines.join("\n");
    diff.push('\n');
    obj(vec![("file", text(name)), ("diff", text(&diff))])
}

fn set_of(entries: Vec<JsonValue>) -> JsonValue {
    obj(vec![("files", JsonValue::Array(entries))])
}

#[test]
fn a_clean_set_rendering_is_exact() {
    let value = set_of(vec![set_entry("a.py", &["    logger.debug(x)"])]);
    let results = check_changeset(&value);
    assert_eq!(
        render_set_review(&results),
        "Change set review\n\
         Files, 1\n\
         \n\
         File, a.py\n\
         Clean\n\
         \n\
         Summary\n\
         Judgements, 0\n"
    );
}

#[test]
fn a_set_rendering_is_exact_with_a_judgement() {
    let value = set_of(vec![set_entry("a.py", &["    print(total)"])]);
    let results = check_changeset(&value);
    assert_eq!(
        render_set_review(&results),
        "Change set review\n\
         Files, 1\n\
         \n\
         File, a.py\n\
         Judgements, 1\n\
         \n\
         Judgement 1, smell\n\
         Flaw, The added line sends diagnostics through a print call\n\
         Standard, Diagnostics travel through the logging module so they \
         can be levelled, filtered, and silenced\n\
         Rewrite,     logger.debug(total)\n\
         \n\
         Summary\n\
         Judgements, 1\n\
         Shape, print call, 1\n\
         Files with judgements, worst first\n\
         Worst, a.py, 1\n"
    );
}

#[test]
fn the_summary_counts_by_shape_in_the_stated_shape_order() {
    let value = set_of(vec![
        set_entry("a.py", &["def stamp(at=now()):", "    print(total)"]),
        set_entry("b.py", &["    except:"]),
    ]);
    let rendered = render_set_review(&check_changeset(&value));
    let shape_lines: Vec<&str> = rendered
        .lines()
        .filter(|line| line.starts_with("Shape, "))
        .collect();
    assert_eq!(
        shape_lines,
        vec![
            "Shape, bare except, 1",
            "Shape, print call, 1",
            "Shape, call default, 1",
        ]
    );
}

#[test]
fn the_worst_first_order_breaks_ties_by_file_name() {
    let value = set_of(vec![
        set_entry("zeta.py", &["    print(total)"]),
        set_entry("mid.py", &["    print(total)", "    except:"]),
        set_entry("alpha.py", &["    print(total)"]),
        set_entry("clean.py", &["    logger.debug(total)"]),
    ]);
    let rendered = render_set_review(&check_changeset(&value));
    let worst_lines: Vec<&str> = rendered
        .lines()
        .filter(|line| line.starts_with("Worst, "))
        .collect();
    assert_eq!(
        worst_lines,
        vec![
            "Worst, mid.py, 2",
            "Worst, alpha.py, 1",
            "Worst, zeta.py, 1"
        ]
    );
}

#[test]
fn the_set_rendering_is_deterministic() {
    let value = set_of(vec![set_entry("a.py", &["    print(total)"])]);
    let results = check_changeset(&value);
    assert_eq!(render_set_review(&results), render_set_review(&results));
}

#[test]
fn the_example_change_set_matches_its_golden_rendering() {
    let set_text =
        std::fs::read_to_string(root().join("examples").join("change_set.json")).unwrap();
    let golden =
        std::fs::read_to_string(root().join("examples").join("change_set_review.txt")).unwrap();
    let value = parse(&set_text).unwrap();
    assert_eq!(render_set_review(&check_changeset(&value)), golden);
}

#[test]
fn a_contested_flag_gains_the_section_and_the_count() {
    let diff = "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,1 @@\n+data = load_entries()\n";
    let verdicts = JsonValue::Array(vec![
        parse(
            r#"{"line": 4, "pattern": "near placeholder name",
                "model": "jev-1.13.0", "choice": "stickler",
                "confidence": 0.81}"#,
        )
        .unwrap(),
        parse(
            r#"{"line": 4, "pattern": "near placeholder name",
                "model": "openai/gpt-6-luna", "choice": "clean",
                "confidence": 0.77}"#,
        )
        .unwrap(),
    ]);
    let value = set_of(vec![obj(vec![
        ("file", text("x.py")),
        ("diff", text(diff)),
        ("verdicts", verdicts),
    ])]);
    let rendered = render_set_review(&review_changeset(&value));
    let section_at = rendered.find("Contested lines").unwrap();
    let summary_at = rendered.find("Summary").unwrap();
    assert!(section_at < summary_at);
    assert!(rendered.ends_with("Contested, 1\n"));
}

#[test]
fn a_malformed_set_rendering_names_every_defect() {
    let rendered = render_set_defects(&[
        "files is missing".to_string(),
        "files[0] must be an object".to_string(),
    ]);
    assert_eq!(
        rendered,
        "Change set review\n\
         Verdict, invalid\n\
         Defects, 2\n\
         Defect, files is missing\n\
         Defect, files[0] must be an object\n"
    );
}
