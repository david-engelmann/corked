//! The set level review. A change set holds named diffs, and
//! the set check runs the checker over every entry. An entry may
//! also hold recorded verdicts on its borderline lines, read
//! under CK-8.

use num_bigint::BigInt;

use crate::checker::{
    check_diff_detailed, check_diff_flagged, Flag, Objection, BORDERLINE_PATTERNS,
};
use crate::json::{JString, JsonNumber, JsonValue};
use crate::verdicts::{VerdictEntry, LABELS};

const VERDICT_FIELDS: [&str; 5] = ["line", "pattern", "model", "choice", "confidence"];

/// The judgements of one entry, as pairs of the shape name and
/// the objection.
pub type Judgements = Vec<(&'static str, Objection)>;

/// One borderline block of an entry, holding one flag and the
/// recorded verdicts standing on that flag, in the order the
/// verdicts stand in the entry.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub flag: Flag,
    pub verdicts: Vec<VerdictEntry>,
}

/// The review of one entry of a change set, holding its file
/// name, its judgements, and its borderline blocks.
#[derive(Clone, Debug, PartialEq)]
pub struct FileResult {
    pub name: String,
    pub judgements: Judgements,
    pub blocks: Vec<Block>,
}

/// Return whether a verdict confidence figure is refused.
///
/// The figure is a number and not a boolean when this reading
/// runs, and it is refused when it is not a finite number from 0
/// to 1. An integer figure is read by the range as it stands, so
/// an integer of any size is refused by the range itself and is
/// never converted to a float to test its finiteness, under
/// FIX-CK-16. A float figure is refused when it is not finite or
/// when it stands outside the range.
fn confidence_is_refused(confidence: &JsonNumber) -> bool {
    match confidence {
        JsonNumber::Int(value) => *value < BigInt::from(0) || *value > BigInt::from(1),
        JsonNumber::Float(value) => !value.is_finite() || !(0.0..=1.0).contains(value),
    }
}

/// Read a JSON string as the Rust string it stands for. A string
/// holding an unpaired surrogate stands for no printable string,
/// and reads as the lossy replacement, a path the review reaches
/// only for a value the loader already refused, never for a
/// valid change set.
fn text_of(text: &JString) -> String {
    text.to_rust_string()
        .unwrap_or_else(|| String::from_utf16_lossy(&text.0))
}

/// Name the defects of one entry's recorded verdicts.
///
/// The verdicts are read only for an entry whose file and diff
/// fields drew no defect, so the flags of the entry can be
/// computed from its diff. No defect names the value it refuses,
/// only the position of the verdict and the rule it breaks.
fn validate_verdicts(index: usize, entry: &JsonValue, defects: &mut Vec<String>) {
    let where_ = format!("files[{index}].verdicts");
    let verdicts = entry.get("verdicts").expect("the caller read it");
    let Some(items) = verdicts.as_array() else {
        defects.push(format!("{where_} must be a list"));
        return;
    };
    let diff = entry
        .get("diff")
        .and_then(|value| value.as_jstring())
        .map(text_of)
        .unwrap_or_default();
    let (_judgements, flags) = check_diff_flagged(&diff);
    let mut seen: Vec<(BigInt, JString)> = Vec::new();
    for (position, verdict) in items.iter().enumerate() {
        let vwhere = format!("{where_}[{position}]");
        if !verdict.is_object() {
            defects.push(format!("{vwhere} must be an object"));
            continue;
        }
        for field in VERDICT_FIELDS {
            if !verdict.contains_key(field) {
                defects.push(format!("{vwhere}.{field} is missing"));
            }
        }
        let line = verdict.get("line");
        let line_int = match line.and_then(|value| value.as_number()) {
            Some(JsonNumber::Int(value)) => Some(value.clone()),
            _ => None,
        };
        let line_ok = match &line_int {
            Some(value) => *value >= BigInt::from(1),
            None => false,
        };
        if line.is_some() && !line_ok {
            defects.push(format!("{vwhere}.line must be a positive integer"));
        }
        let pattern = verdict.get("pattern");
        let pattern_ok = match pattern.and_then(|value| value.as_jstring()) {
            Some(text) => BORDERLINE_PATTERNS.iter().any(|name| text.equals(name)),
            None => false,
        };
        if pattern.is_some() && !pattern_ok {
            defects.push(format!(
                "{vwhere}.pattern must be one of near placeholder \
                 name, broad handler, shared member default"
            ));
        }
        let model = verdict.get("model");
        let mut model_ok = false;
        let mut model_text: Option<JString> = None;
        if let Some(model_value) = model {
            match model_value.as_jstring() {
                Some(text) if !text.is_blank() => {
                    model_ok = true;
                    model_text = Some(text.clone());
                }
                _ => {}
            }
            if !model_ok {
                defects.push(format!("{vwhere}.model must be a non empty string"));
            } else if model_text
                .as_ref()
                .is_some_and(|text| text.holds_unpaired_surrogate())
            {
                defects.push(format!("{vwhere}.model holds an unpaired surrogate"));
                model_ok = false;
            } else if model_text
                .as_ref()
                .is_some_and(|text| text.holds_line_break())
            {
                defects.push(format!("{vwhere}.model holds a line break"));
                model_ok = false;
            }
        }
        if let Some(choice) = verdict.get("choice") {
            let known = match choice.as_jstring() {
                Some(text) => LABELS.iter().any(|label| text.equals(label)),
                None => false,
            };
            if !known {
                defects.push(format!("{vwhere}.choice must be one of stickler, clean"));
            }
        }
        if let Some(confidence) = verdict.get("confidence") {
            match confidence.as_number() {
                None => {
                    defects.push(format!("{vwhere}.confidence must be a number"));
                }
                Some(number) => {
                    if confidence_is_refused(number) {
                        defects.push(format!("{vwhere}.confidence must be from 0 to 1"));
                    }
                }
            }
        }
        if line_ok && pattern_ok {
            let line_value = line_int.as_ref().expect("a sound line");
            let flag = flags
                .iter()
                .find(|flag| BigInt::from(flag.line) == *line_value);
            let pattern_text = pattern
                .and_then(|value| value.as_jstring())
                .expect("a sound pattern");
            let matched = match flag {
                Some(flag) => pattern_text.equals(&flag.pattern),
                None => false,
            };
            if !matched {
                defects.push(format!("{vwhere} names no borderline line"));
            } else if model_ok {
                let key = (
                    line_value.clone(),
                    model_text.clone().expect("a sound model"),
                );
                if seen.contains(&key) {
                    defects.push(format!(
                        "{vwhere} repeats a verdict for the same \
                         line and model"
                    ));
                } else {
                    seen.push(key);
                }
            }
        }
    }
}

/// Return the list of defects found in a change set. Never
/// fails, whatever it is handed.
pub fn load_changeset(value: &JsonValue) -> Vec<String> {
    let mut defects = Vec::new();
    if !value.is_object() {
        return vec!["change set must be a JSON object".to_string()];
    }
    let Some(files) = value.get("files") else {
        return vec!["files is missing".to_string()];
    };
    let Some(entries) = files.as_array() else {
        return vec!["files must be a list".to_string()];
    };
    for (index, entry) in entries.iter().enumerate() {
        let where_ = format!("files[{index}]");
        if !entry.is_object() {
            defects.push(format!("{where_} must be an object"));
            continue;
        }
        let before = defects.len();
        match entry.get("file") {
            None => defects.push(format!("{where_}.file is missing")),
            Some(file) => match file.as_jstring() {
                Some(text) if !text.is_blank() => {
                    if text.holds_unpaired_surrogate() {
                        defects.push(format!("{where_}.file holds an unpaired surrogate"));
                    } else if text.holds_line_break() {
                        defects.push(format!("{where_}.file holds a line break"));
                    }
                }
                _ => defects.push(format!("{where_}.file must be a non empty string")),
            },
        }
        match entry.get("diff") {
            None => defects.push(format!("{where_}.diff is missing")),
            Some(diff) => match diff.as_jstring() {
                None => defects.push(format!("{where_}.diff must be a string")),
                Some(text) => {
                    if text.holds_unpaired_surrogate() {
                        defects.push(format!("{where_}.diff holds an unpaired surrogate"));
                    }
                }
            },
        }
        if entry.contains_key("verdicts") && defects.len() == before {
            validate_verdicts(index, entry, &mut defects);
        }
    }
    defects
}

/// Run the checker over every entry of a valid change set.
///
/// Returns one result per entry, in the order given, holding the
/// file name and the judgements its diff raises.
pub fn check_changeset(value: &JsonValue) -> Vec<FileResult> {
    entries_of(value)
        .into_iter()
        .map(|entry| FileResult {
            name: entry.name,
            judgements: check_diff_detailed(&entry.diff),
            blocks: Vec::new(),
        })
        .collect()
}

/// One entry of a change set read for the set readings, holding
/// its file name, its diff as text, and its recorded verdicts as
/// raw values.
struct RawEntry {
    name: String,
    diff: String,
    verdicts: Vec<JsonValue>,
}

fn entries_of(value: &JsonValue) -> Vec<RawEntry> {
    let Some(entries) = value.get("files").and_then(|files| files.as_array()) else {
        return Vec::new();
    };
    entries
        .iter()
        .map(|entry| RawEntry {
            name: entry
                .get("file")
                .and_then(|file| file.as_jstring())
                .map(text_of)
                .unwrap_or_default(),
            diff: entry
                .get("diff")
                .and_then(|diff| diff.as_jstring())
                .map(text_of)
                .unwrap_or_default(),
            verdicts: entry
                .get("verdicts")
                .and_then(|verdicts| verdicts.as_array())
                .cloned()
                .unwrap_or_default(),
        })
        .collect()
}

/// Read one recorded verdict value standing on a flag into its
/// entry form. The line reading crosses the integer and float
/// forms, as the Python equality crosses them.
fn standing_entry(flag: &Flag, verdict: &JsonValue) -> Option<VerdictEntry> {
    let line_matches = match verdict.get("line")?.as_number()? {
        JsonNumber::Int(value) => *value == BigInt::from(flag.line),
        JsonNumber::Float(value) => *value == flag.line as f64,
    };
    if !line_matches {
        return None;
    }
    Some(VerdictEntry {
        line: flag.line,
        pattern: verdict
            .get("pattern")
            .and_then(|value| value.as_jstring())
            .map(text_of)
            .unwrap_or_default(),
        model: verdict
            .get("model")
            .and_then(|value| value.as_jstring())
            .map(text_of)
            .unwrap_or_default(),
        choice: verdict
            .get("choice")
            .and_then(|value| value.as_jstring())
            .map(text_of)
            .unwrap_or_default(),
        confidence: verdict
            .get("confidence")
            .and_then(|value| value.as_number())
            .cloned()
            .unwrap_or(JsonNumber::Int(BigInt::from(0))),
    })
}

/// Review every entry of a valid change set, verdicts included.
///
/// Returns one result per entry, in the order given, holding the
/// file name, the judgements its diff raises, and its borderline
/// blocks. A block holds one flag of the entry and the recorded
/// verdicts standing on that flag, in the order the verdicts
/// stand in the entry, under CK-8.
pub fn review_changeset(value: &JsonValue) -> Vec<FileResult> {
    let mut results = Vec::new();
    for entry in entries_of(value) {
        let (judgements, flags) = check_diff_flagged(&entry.diff);
        let mut blocks = Vec::new();
        for flag in flags {
            let standing: Vec<VerdictEntry> = entry
                .verdicts
                .iter()
                .filter_map(|verdict| standing_entry(&flag, verdict))
                .collect();
            blocks.push(Block {
                flag,
                verdicts: standing,
            });
        }
        results.push(FileResult {
            name: entry.name,
            judgements,
            blocks,
        });
    }
    results
}
