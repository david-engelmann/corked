//! The review core. A review is validated and given a verdict
//! here.

use crate::json::JsonValue;

pub const CATEGORIES: [&str; 3] = ["naming", "structure", "smell"];
const OBJECTION_FIELDS: [&str; 4] = ["flaw", "standard", "rewrite", "category"];
const TEXT_FIELDS: [&str; 3] = ["flaw", "standard", "rewrite"];

/// Read a value as blank, as the validator reads a field. A value
/// that is not a string, and a string holding nothing but
/// whitespace, are both blank.
fn is_blank(value: &JsonValue) -> bool {
    match value.as_jstring() {
        Some(text) => text.is_blank(),
        None => true,
    }
}

/// Return the list of defects found in a review. Never fails,
/// whatever it is handed.
pub fn validate_review(review: &JsonValue) -> Vec<String> {
    let mut defects = Vec::new();
    if !review.is_object() {
        return vec!["review must be a JSON object".to_string()];
    }
    match review.get("diff_label") {
        None => defects.push("diff_label is missing".to_string()),
        Some(label) => {
            if is_blank(label) {
                defects.push("diff_label must be a non empty string".to_string());
            } else if let Some(text) = label.as_jstring() {
                if text.holds_unpaired_surrogate() {
                    defects.push("diff_label holds an unpaired surrogate".to_string());
                } else if text.holds_line_break() {
                    defects.push("diff_label holds a line break".to_string());
                }
            }
        }
    }
    let objections = match review.get("objections") {
        None => {
            defects.push("objections is missing".to_string());
            return defects;
        }
        Some(value) => match value.as_array() {
            None => {
                defects.push("objections must be a list".to_string());
                return defects;
            }
            Some(items) => items,
        },
    };
    for (index, objection) in objections.iter().enumerate() {
        let where_ = format!("objections[{index}]");
        if !objection.is_object() {
            defects.push(format!("{where_} must be an object"));
            continue;
        }
        for field in OBJECTION_FIELDS {
            if !objection.contains_key(field) {
                defects.push(format!("{where_}.{field} is missing"));
            }
        }
        for field in TEXT_FIELDS {
            if let Some(value) = objection.get(field) {
                if is_blank(value) {
                    defects.push(format!("{where_}.{field} must be a non empty string"));
                } else if let Some(text) = value.as_jstring() {
                    if text.holds_unpaired_surrogate() {
                        defects.push(format!(
                            "{where_}.{field} holds an unpaired \
                             surrogate"
                        ));
                    }
                }
            }
        }
        if let Some(category) = objection.get("category") {
            let known = match category.as_jstring() {
                Some(text) => CATEGORIES.iter().any(|name| text.equals(name)),
                None => false,
            };
            if !known {
                defects.push(format!(
                    "{where_}.category must be one of naming, \
                     structure, smell"
                ));
            }
        }
    }
    defects
}

/// Return the verdict for a review. One of accepted, sent back,
/// and invalid.
pub fn verdict_for(review: &JsonValue) -> &'static str {
    if !validate_review(review).is_empty() {
        return "invalid";
    }
    match review.get("objections").and_then(|value| value.as_array()) {
        Some(items) if !items.is_empty() => "sent back",
        _ => "accepted",
    }
}
