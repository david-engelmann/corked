//! Pins for CK-1 and CK-2. The verdict rule and the completeness
//! rule. Translated from tests/test_review.py.

use stickler::json::{parse, JsonValue};
use stickler::review::{validate_review, verdict_for};

const COMPLETE: &str = r#"{
    "flaw": "The name says nothing about what the value holds",
    "standard": "A name says what its value holds",
    "rewrite": "orders = load_orders()",
    "category": "naming"
}"#;

fn review(objections: &str) -> JsonValue {
    parse(&format!(
        r#"{{"diff_label": "change under review", "objections": {objections}}}"#
    ))
    .unwrap()
}

fn strings(defects: Vec<String>) -> Vec<String> {
    defects
}

#[test]
fn empty_objections_validate_clean_and_accept() {
    assert_eq!(validate_review(&review("[]")), strings(vec![]));
    assert_eq!(verdict_for(&review("[]")), "accepted");
}

#[test]
fn one_complete_objection_sends_back() {
    let value = review(&format!("[{COMPLETE}]"));
    assert_eq!(validate_review(&value), strings(vec![]));
    assert_eq!(verdict_for(&value), "sent back");
}

#[test]
fn verdict_flips_on_one_objection_added_and_removed() {
    assert_eq!(verdict_for(&review("[]")), "accepted");
    assert_eq!(verdict_for(&review(&format!("[{COMPLETE}]"))), "sent back");
    assert_eq!(verdict_for(&review("[]")), "accepted");
}

#[test]
fn missing_diff_label_is_named() {
    let value = parse(r#"{"objections": []}"#).unwrap();
    assert_eq!(
        validate_review(&value),
        vec!["diff_label is missing".to_string()]
    );
}

#[test]
fn blank_diff_label_is_named() {
    let value = parse(r#"{"diff_label": "   ", "objections": []}"#).unwrap();
    assert_eq!(
        validate_review(&value),
        vec!["diff_label must be a non empty string".to_string()]
    );
}

#[test]
fn non_string_diff_label_is_named() {
    let value = parse(r#"{"diff_label": 7, "objections": []}"#).unwrap();
    assert_eq!(
        validate_review(&value),
        vec!["diff_label must be a non empty string".to_string()]
    );
}

#[test]
fn a_diff_label_holding_an_unpaired_surrogate_is_named() {
    let value = parse(r#"{"diff_label": "\ud800x", "objections": []}"#).unwrap();
    assert_eq!(
        validate_review(&value),
        vec!["diff_label holds an unpaired surrogate".to_string()]
    );
    assert_eq!(verdict_for(&value), "invalid");
}

#[test]
fn a_diff_label_holding_a_line_break_is_named() {
    for broken in ["evil\nForged, 99", "evil\rForged, 99"] {
        let text = format!(
            "{{\"diff_label\": \"{}\", \"objections\": []}}",
            broken.replace('\n', "\\n").replace('\r', "\\r")
        );
        let value = parse(&text).unwrap();
        assert_eq!(
            validate_review(&value),
            vec!["diff_label holds a line break".to_string()]
        );
        assert_eq!(verdict_for(&value), "invalid");
    }
}

#[test]
fn each_text_field_holding_an_unpaired_surrogate_is_named() {
    for field in ["flaw", "standard", "rewrite"] {
        let mut fields: Vec<String> = Vec::new();
        for name in ["flaw", "standard", "rewrite", "category"] {
            let value = if name == field {
                "bad \\ud800 text".to_string()
            } else if name == "category" {
                "naming".to_string()
            } else {
                "x".to_string()
            };
            fields.push(format!("\"{name}\": \"{value}\""));
        }
        let value = review(&format!("[{{{}}}]", fields.join(", ")));
        assert_eq!(
            validate_review(&value),
            vec![format!("objections[0].{field} holds an unpaired surrogate")],
            "field {field}"
        );
    }
}

#[test]
fn missing_objections_is_named() {
    let value = parse(r#"{"diff_label": "x"}"#).unwrap();
    assert_eq!(
        validate_review(&value),
        vec!["objections is missing".to_string()]
    );
}

#[test]
fn objections_of_wrong_type_is_named() {
    let value = parse(r#"{"diff_label": "x", "objections": "none"}"#).unwrap();
    assert_eq!(
        validate_review(&value),
        vec!["objections must be a list".to_string()]
    );
}

#[test]
fn non_object_review_is_named_and_never_raises() {
    for handed in ["null", "7", "\"review\"", "[\"objections\"]", "true"] {
        let value = parse(handed).unwrap();
        assert_eq!(
            validate_review(&value),
            vec!["review must be a JSON object".to_string()],
            "handed {handed}"
        );
        assert_eq!(verdict_for(&value), "invalid");
    }
}

#[test]
fn each_missing_objection_field_is_named() {
    for field in ["flaw", "standard", "rewrite", "category"] {
        let kept: Vec<String> = ["flaw", "standard", "rewrite", "category"]
            .iter()
            .filter(|name| **name != field)
            .map(|name| {
                let value = if *name == "category" { "naming" } else { "x" };
                format!("\"{name}\": \"{value}\"")
            })
            .collect();
        let value = review(&format!("[{{{}}}]", kept.join(", ")));
        assert_eq!(
            validate_review(&value),
            vec![format!("objections[0].{field} is missing")],
            "field {field}"
        );
    }
}

#[test]
fn each_blank_text_field_is_named() {
    for field in ["flaw", "standard", "rewrite"] {
        let fields: Vec<String> = ["flaw", "standard", "rewrite", "category"]
            .iter()
            .map(|name| {
                let value = if *name == field {
                    "  "
                } else if *name == "category" {
                    "naming"
                } else {
                    "x"
                };
                format!("\"{name}\": \"{value}\"")
            })
            .collect();
        let value = review(&format!("[{{{}}}]", fields.join(", ")));
        assert_eq!(
            validate_review(&value),
            vec![format!("objections[0].{field} must be a non empty string")],
            "field {field}"
        );
    }
}

#[test]
fn unknown_category_is_named() {
    let objection = COMPLETE.replace("\"naming\"", "\"style\"");
    let value = review(&format!("[{objection}]"));
    assert_eq!(
        validate_review(&value),
        vec!["objections[0].category must be one of naming, structure, smell".to_string()]
    );
}

#[test]
fn every_named_category_is_accepted() {
    for category in ["naming", "structure", "smell"] {
        let objection = COMPLETE.replace("\"naming\"", &format!("\"{category}\""));
        let value = review(&format!("[{objection}]"));
        assert_eq!(
            validate_review(&value),
            Vec::<String>::new(),
            "category {category}"
        );
    }
}

#[test]
fn incomplete_second_objection_is_named_with_its_position() {
    let objection = r#"{
        "flaw": "x", "standard": "x", "category": "naming"
    }"#;
    let value = review(&format!("[{COMPLETE}, {objection}]"));
    assert_eq!(
        validate_review(&value),
        vec!["objections[1].rewrite is missing".to_string()]
    );
}

#[test]
fn objection_of_wrong_type_is_named() {
    let value = review(r#"["not an objection"]"#);
    assert_eq!(
        validate_review(&value),
        vec!["objections[0] must be an object".to_string()]
    );
}

#[test]
fn incomplete_objection_makes_the_verdict_invalid_not_sent_back() {
    let objection = r#"{
        "standard": "x", "rewrite": "x", "category": "naming"
    }"#;
    let value = review(&format!("[{objection}]"));
    assert_eq!(verdict_for(&value), "invalid");
}
