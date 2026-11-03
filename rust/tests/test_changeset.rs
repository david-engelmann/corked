//! Pins for CK-6. The change set loader and the set check.
//! Translated from tests/test_changeset.py.

use stickler::changeset::{check_changeset, load_changeset, review_changeset};
use stickler::json::{parse, JString, JsonNumber, JsonValue};
use stickler::verdicts::VerdictEntry;

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

fn entry(name: &str, added_lines: &[&str]) -> JsonValue {
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

const ONE_FLAG_DIFF: &str = "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,1 @@\n+data = load_entries()\n";

fn one_flag_set(verdicts: JsonValue) -> JsonValue {
    set_of(vec![obj(vec![
        ("file", text("x.py")),
        ("diff", text(ONE_FLAG_DIFF)),
        ("verdicts", verdicts),
    ])])
}

/// Build one verdict object from raw JSON fragments, so figures
/// the Rust literal syntax cannot state plainly, such as NaN
/// and an integer of 400 digits, stand as the Python states
/// them.
fn verdict_json(
    line: &str,
    pattern: &str,
    model: &str,
    choice: &str,
    confidence: &str,
) -> JsonValue {
    parse(&format!(
        "{{\"line\": {line}, \"pattern\": \"{pattern}\", \
         \"model\": \"{model}\", \"choice\": \"{choice}\", \
         \"confidence\": {confidence}}}"
    ))
    .unwrap()
}

fn good_verdict() -> JsonValue {
    verdict_json(
        "4",
        "near placeholder name",
        "jev-1.13.0",
        "stickler",
        "0.78",
    )
}

#[test]
fn a_valid_change_set_loads_clean() {
    let value = set_of(vec![entry("a.py", &["    logger.debug(total)"])]);
    assert_eq!(load_changeset(&value), Vec::<String>::new());
}

#[test]
fn an_empty_files_list_loads_clean() {
    let value = set_of(vec![]);
    assert_eq!(load_changeset(&value), Vec::<String>::new());
    let results = check_changeset(&value);
    assert!(results.is_empty());
}

#[test]
fn a_non_object_change_set_is_named() {
    let list = parse("[1, 2]").unwrap();
    assert_eq!(
        load_changeset(&list),
        vec!["change set must be a JSON object".to_string()]
    );
    assert_eq!(
        load_changeset(&text("nope")),
        vec!["change set must be a JSON object".to_string()]
    );
}

#[test]
fn a_missing_files_field_is_named() {
    assert_eq!(
        load_changeset(&obj(vec![])),
        vec!["files is missing".to_string()]
    );
}

#[test]
fn a_non_list_files_field_is_named() {
    let value = obj(vec![("files", text("a.py"))]);
    assert_eq!(
        load_changeset(&value),
        vec!["files must be a list".to_string()]
    );
}

#[test]
fn a_non_object_entry_is_named_with_its_position() {
    let value = set_of(vec![entry("a.py", &[]), text("oops")]);
    assert_eq!(
        load_changeset(&value),
        vec!["files[1] must be an object".to_string()]
    );
}

#[test]
fn a_missing_file_field_is_named_with_its_position() {
    let value = set_of(vec![obj(vec![("diff", text("x"))])]);
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].file is missing".to_string()]
    );
}

#[test]
fn a_blank_file_name_is_named() {
    let value = set_of(vec![obj(vec![("file", text("  ")), ("diff", text(""))])]);
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].file must be a non empty string".to_string()]
    );
}

#[test]
fn a_non_string_file_name_is_named() {
    let value = set_of(vec![obj(vec![
        ("file", JsonValue::Array(vec![text("a.py")])),
        ("diff", text("")),
    ])]);
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].file must be a non empty string".to_string()]
    );
}

#[test]
fn a_file_name_holding_an_unpaired_surrogate_is_named() {
    let name = parse(r#""\ud800x.py""#).unwrap();
    let value = set_of(vec![obj(vec![("file", name), ("diff", text(""))])]);
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].file holds an unpaired surrogate".to_string()]
    );
}

#[test]
fn a_file_name_holding_a_newline_is_named() {
    let value = set_of(vec![obj(vec![
        ("file", text("evil\nForged, 99")),
        ("diff", text("")),
    ])]);
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].file holds a line break".to_string()]
    );
}

#[test]
fn a_file_name_holding_a_carriage_return_is_named() {
    let value = set_of(vec![obj(vec![
        ("file", text("evil\rForged, 99")),
        ("diff", text("")),
    ])]);
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].file holds a line break".to_string()]
    );
}

#[test]
fn verdicts_are_not_read_when_the_file_name_holds_a_line_break() {
    let entry_with_verdicts = obj(vec![
        ("file", text("evil\nForged, 99")),
        ("diff", text("")),
        ("verdicts", text("not a list")),
    ]);
    let value = set_of(vec![entry_with_verdicts]);
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].file holds a line break".to_string()]
    );
}

#[test]
fn a_diff_holding_an_unpaired_surrogate_is_named() {
    let diff = parse(r#""+print('\ud800')""#).unwrap();
    let value = set_of(vec![obj(vec![("file", text("a.py")), ("diff", diff)])]);
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].diff holds an unpaired surrogate".to_string()]
    );
}

#[test]
fn a_missing_diff_field_is_named() {
    let value = set_of(vec![obj(vec![("file", text("a.py"))])]);
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].diff is missing".to_string()]
    );
}

#[test]
fn a_non_string_diff_is_named() {
    let value = set_of(vec![obj(vec![
        ("file", text("a.py")),
        ("diff", JsonValue::Number(JsonNumber::Int(7.into()))),
    ])]);
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].diff must be a string".to_string()]
    );
}

#[test]
fn an_empty_diff_is_a_valid_clean_entry() {
    let value = set_of(vec![obj(vec![("file", text("a.py")), ("diff", text(""))])]);
    assert_eq!(load_changeset(&value), Vec::<String>::new());
    let results = check_changeset(&value);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "a.py");
    assert!(results[0].judgements.is_empty());
}

#[test]
fn the_loader_never_raises_on_hostile_values() {
    let hostile = [
        JsonValue::Null,
        parse("7").unwrap(),
        parse("2.5").unwrap(),
        JsonValue::Bool(true),
        parse("[]").unwrap(),
        obj(vec![("files", JsonValue::Null)]),
        obj(vec![(
            "files",
            JsonValue::Array(vec![
                JsonValue::Null,
                obj(vec![
                    ("file", JsonValue::Array(vec![])),
                    ("diff", obj(vec![])),
                ]),
            ]),
        )]),
    ];
    for value in &hostile {
        let defects = load_changeset(value);
        assert!(!defects.is_empty(), "value {value:?}");
    }
}

#[test]
fn the_set_check_reports_judgements_per_entry_in_order() {
    let value = set_of(vec![
        entry("a.py", &["    print(total)"]),
        entry("b.py", &["    logger.debug(total)"]),
        entry("c.py", &["    except:"]),
    ]);
    let results = check_changeset(&value);
    let names: Vec<&str> = results.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, vec!["a.py", "b.py", "c.py"]);
    let counts: Vec<usize> = results.iter().map(|r| r.judgements.len()).collect();
    assert_eq!(counts, vec![1, 0, 1]);
    assert_eq!(results[0].judgements[0].0, "print call");
    assert_eq!(results[2].judgements[0].0, "bare except");
}

#[test]
fn the_set_check_carries_the_shape_names_of_the_new_shapes() {
    let diff = "--- a/x.py\n+++ b/x.py\n@@ -1,2 +1,3 @@\n+def stamp(at=now()):\n";
    let value = set_of(vec![obj(vec![
        ("file", text("a.py")),
        ("diff", text(diff)),
    ])]);
    let results = check_changeset(&value);
    let shapes: Vec<&str> = results[0]
        .judgements
        .iter()
        .map(|(shape, _)| *shape)
        .collect();
    assert_eq!(shapes, vec!["call default"]);
}

#[test]
fn verdicts_stand_beside_file_and_diff() {
    let value = one_flag_set(JsonValue::Array(vec![good_verdict()]));
    assert_eq!(load_changeset(&value), Vec::<String>::new());
}

#[test]
fn two_models_may_stand_on_one_line() {
    let verdicts = JsonValue::Array(vec![
        good_verdict(),
        verdict_json(
            "4",
            "near placeholder name",
            "openai/gpt-6-luna",
            "clean",
            "0.78",
        ),
    ]);
    assert_eq!(
        load_changeset(&one_flag_set(verdicts)),
        Vec::<String>::new()
    );
}

#[test]
fn fields_beyond_the_five_are_not_read() {
    let mut verdict = good_verdict();
    if let JsonValue::Object(pairs) = &mut verdict {
        pairs.push((JString::from_text("reason"), text("invented words")));
    }
    let value = one_flag_set(JsonValue::Array(vec![verdict]));
    assert_eq!(load_changeset(&value), Vec::<String>::new());
}

#[test]
fn verdicts_must_be_a_list() {
    let value = one_flag_set(text("none"));
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].verdicts must be a list".to_string()]
    );
}

#[test]
fn a_verdict_must_be_an_object() {
    let value = one_flag_set(JsonValue::Array(vec![JsonValue::Number(JsonNumber::Int(
        7.into(),
    ))]));
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].verdicts[0] must be an object".to_string()]
    );
}

#[test]
fn a_missing_verdict_field_is_named() {
    let verdict = parse(
        r#"{"line": 4, "pattern": "near placeholder name",
            "model": "jev-1.13.0", "choice": "stickler"}"#,
    )
    .unwrap();
    let value = one_flag_set(JsonValue::Array(vec![verdict]));
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].verdicts[0].confidence is missing".to_string()]
    );
}

#[test]
fn the_verdict_line_must_be_a_positive_integer() {
    for bad in ["0", "-1", "\"4\"", "true", "4.5"] {
        let verdict = verdict_json(
            bad,
            "near placeholder name",
            "jev-1.13.0",
            "stickler",
            "0.78",
        );
        let value = one_flag_set(JsonValue::Array(vec![verdict]));
        assert_eq!(
            load_changeset(&value),
            vec!["files[0].verdicts[0].line must be a positive integer".to_string()],
            "bad {bad}"
        );
    }
}

#[test]
fn the_verdict_pattern_must_be_a_borderline_pattern() {
    let verdict = verdict_json("4", "print call", "jev-1.13.0", "stickler", "0.78");
    let value = one_flag_set(JsonValue::Array(vec![verdict]));
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].verdicts[0].pattern must be one of \
             near placeholder name, broad handler, shared member default"
            .to_string()]
    );
}

#[test]
fn the_verdict_model_must_be_a_non_empty_string() {
    for bad in ["\"\"", "\"   \"", "7"] {
        let verdict = parse(&format!(
            "{{\"line\": 4, \"pattern\": \"near placeholder name\", \
             \"model\": {bad}, \"choice\": \"stickler\", \
             \"confidence\": 0.78}}"
        ))
        .unwrap();
        let value = one_flag_set(JsonValue::Array(vec![verdict]));
        assert_eq!(
            load_changeset(&value),
            vec!["files[0].verdicts[0].model must be a non empty string".to_string()],
            "bad {bad}"
        );
    }
}

#[test]
fn the_verdict_model_must_hold_no_surrogate() {
    let verdict = parse(
        r#"{"line": 4, "pattern": "near placeholder name",
            "model": "jev\ud800", "choice": "stickler",
            "confidence": 0.78}"#,
    )
    .unwrap();
    let value = one_flag_set(JsonValue::Array(vec![verdict]));
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].verdicts[0].model holds an unpaired surrogate".to_string()]
    );
}

#[test]
fn the_verdict_model_must_hold_no_line_break() {
    let verdict = verdict_json(
        "4",
        "near placeholder name",
        "jev\\nforged",
        "stickler",
        "0.78",
    );
    // The fragment above spells the model through a JSON escape,
    // and parsing it puts a real line break in the name.
    let value = one_flag_set(JsonValue::Array(vec![verdict]));
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].verdicts[0].model holds a line break".to_string()]
    );
}

#[test]
fn the_verdict_choice_must_be_a_label() {
    let verdict = verdict_json("4", "near placeholder name", "jev-1.13.0", "maybe", "0.78");
    let value = one_flag_set(JsonValue::Array(vec![verdict]));
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].verdicts[0].choice must be one of stickler, clean".to_string()]
    );
}

#[test]
fn the_verdict_confidence_must_be_a_number_in_range() {
    for (fragment, defect) in [
        (
            "\"high\"",
            "files[0].verdicts[0].confidence must be a number",
        ),
        ("true", "files[0].verdicts[0].confidence must be a number"),
        ("1.5", "files[0].verdicts[0].confidence must be from 0 to 1"),
        (
            "-0.1",
            "files[0].verdicts[0].confidence must be from 0 to 1",
        ),
        ("NaN", "files[0].verdicts[0].confidence must be from 0 to 1"),
    ] {
        let verdict = verdict_json(
            "4",
            "near placeholder name",
            "jev-1.13.0",
            "stickler",
            fragment,
        );
        let value = one_flag_set(JsonValue::Array(vec![verdict]));
        assert_eq!(
            load_changeset(&value),
            vec![defect.to_string()],
            "fragment {fragment}"
        );
    }
}

#[test]
fn an_oversized_integer_confidence_is_refused_by_range() {
    let huge = "1".to_string() + &"0".repeat(400);
    for fragment in [huge.clone(), format!("-{huge}")] {
        let verdict = verdict_json(
            "4",
            "near placeholder name",
            "jev-1.13.0",
            "stickler",
            &fragment,
        );
        let value = one_flag_set(JsonValue::Array(vec![verdict]));
        assert_eq!(
            load_changeset(&value),
            vec!["files[0].verdicts[0].confidence must be from 0 to 1".to_string()],
            "fragment {fragment}"
        );
    }
}

#[test]
fn a_verdict_on_a_line_with_no_flag_is_named() {
    let verdict = verdict_json(
        "9",
        "near placeholder name",
        "jev-1.13.0",
        "stickler",
        "0.78",
    );
    let value = one_flag_set(JsonValue::Array(vec![verdict]));
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].verdicts[0] names no borderline line".to_string()]
    );
}

#[test]
fn a_verdict_with_the_wrong_pattern_is_named() {
    let verdict = verdict_json("4", "broad handler", "jev-1.13.0", "stickler", "0.78");
    let value = one_flag_set(JsonValue::Array(vec![verdict]));
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].verdicts[0] names no borderline line".to_string()]
    );
}

#[test]
fn a_second_verdict_from_one_model_is_named() {
    let verdicts = JsonValue::Array(vec![
        good_verdict(),
        verdict_json("4", "near placeholder name", "jev-1.13.0", "clean", "0.78"),
    ]);
    let value = one_flag_set(verdicts);
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].verdicts[1] repeats a verdict for the same \
             line and model"
            .to_string()]
    );
}

#[test]
fn verdicts_are_not_read_for_a_malformed_entry() {
    let bad_entry = obj(vec![
        ("file", text("")),
        ("diff", text(ONE_FLAG_DIFF)),
        ("verdicts", text("not a list")),
    ]);
    let value = set_of(vec![bad_entry]);
    assert_eq!(
        load_changeset(&value),
        vec!["files[0].file must be a non empty string".to_string()]
    );
}

#[test]
fn review_changeset_groups_verdicts_onto_flags() {
    let verdicts = JsonValue::Array(vec![
        good_verdict(),
        verdict_json(
            "4",
            "near placeholder name",
            "openai/gpt-6-luna",
            "clean",
            "0.78",
        ),
    ]);
    let value = one_flag_set(verdicts);
    let results = review_changeset(&value);
    let result = &results[0];
    assert_eq!(result.name, "x.py");
    assert!(result.judgements.is_empty());
    assert_eq!(result.blocks.len(), 1);
    assert_eq!(result.blocks[0].flag.line, 4);
    let expected = vec![
        VerdictEntry {
            line: 4,
            pattern: "near placeholder name".to_string(),
            model: "jev-1.13.0".to_string(),
            choice: "stickler".to_string(),
            confidence: JsonNumber::Float(0.78),
        },
        VerdictEntry {
            line: 4,
            pattern: "near placeholder name".to_string(),
            model: "openai/gpt-6-luna".to_string(),
            choice: "clean".to_string(),
            confidence: JsonNumber::Float(0.78),
        },
    ];
    assert_eq!(result.blocks[0].verdicts, expected);
}
