//! The rejudged ledger. A rejudged reading takes two
//! recorded sets, the first asking and the second asking of
//! one collection of review inputs, and states per input
//! standing in both sets the judgement each set reads over
//! it and whether the judgement held or changed between the
//! askings. The loader validates each member through the
//! change set loader, the flags and verdicts come from the
//! set review, and the judgements come from the outcome
//! helper of CK-8, so the ledger holds no second flagging
//! or judging logic of its own. The obligations this module
//! carries are CK-13 and CK-14 in CONTRACT.md, in the
//! Round 6 section.

use crate::changeset::{review_changeset, FileResult};
use crate::json::JsonValue;
use crate::sets::validate_entry;
use crate::verdicts::line_outcome;

/// One input of a rejudged reading, a borderline flag of one
/// entry of one set, holding its name and the judgement its
/// set reads over it.
#[derive(Clone, Debug, PartialEq)]
struct Input {
    file: String,
    line: usize,
    pattern: String,
    judgement: &'static str,
}

/// One compared input of the reading, standing in both sets
/// and holding the judgement of each asking and whether the
/// two judgements agree.
#[derive(Clone, Debug, PartialEq)]
pub struct ComparedInput {
    pub file: String,
    pub line: usize,
    pub pattern: String,
    pub first: &'static str,
    pub second: &'static str,
    pub held: bool,
}

/// A whole rejudged reading, holding the labels of its two
/// sets in asking order, the compared inputs in the order
/// the first set states them, and its closing counts.
#[derive(Clone, Debug, PartialEq)]
pub struct Rejudged {
    pub first_label: String,
    pub second_label: String,
    pub pairs: Vec<ComparedInput>,
    pub inputs_read: usize,
    pub held: usize,
    pub changed: usize,
}

/// Read the inputs of one set from its set review results.
///
/// One input stands per borderline block, in the order the
/// files stand in the review and the flags stand in each
/// file, and its judgement is the judgement CK-8 reads over
/// the verdicts standing on its flag.
fn inputs_of(results: &[FileResult]) -> Vec<Input> {
    let mut inputs = Vec::new();
    for result in results {
        for block in &result.blocks {
            inputs.push(Input {
                file: result.name.clone(),
                line: block.flag.line,
                pattern: block.flag.pattern.clone(),
                judgement: line_outcome(&block.verdicts),
            });
        }
    }
    inputs
}

/// Return the list of defects found in a rejudged pair.
/// Never fails, whatever it is handed.
///
/// Each entry is validated through the shared set entry
/// validation, the one home the census and rejudged loaders
/// both call.
pub fn load_rejudged(value: &JsonValue) -> Vec<String> {
    let mut defects = Vec::new();
    if !value.is_object() {
        return vec!["rejudged must be a JSON object".to_string()];
    }
    let Some(sets) = value.get("sets") else {
        return vec!["sets is missing".to_string()];
    };
    let Some(entries) = sets.as_array() else {
        return vec!["sets must be a list".to_string()];
    };
    if entries.len() != 2 {
        defects.push("sets must hold exactly two sets".to_string());
    }
    for (index, entry) in entries.iter().enumerate() {
        validate_entry(entry, index, &mut defects);
    }
    defects
}

/// Read a valid rejudged pair into its reading.
///
/// The inputs of the two sets are matched by name, file,
/// line, and pattern, occurrences of one name matched in
/// order, so every input stands in at most one pair. A pair
/// stands held when its two judgements are the same word.
/// A pair the loader refused reads as the inputs its
/// members still yield, in the posture the set readings
/// hold, and the command line never renders one.
pub fn rejudged_of(value: &JsonValue) -> Rejudged {
    let mut labels = [String::new(), String::new()];
    let mut sides: Vec<Vec<Input>> = vec![Vec::new(), Vec::new()];
    if let Some(entries) = value.get("sets").and_then(|sets| sets.as_array()) {
        for (index, entry) in entries.iter().take(2).enumerate() {
            labels[index] = entry
                .get("label")
                .and_then(|label| label.as_jstring())
                .and_then(|text| text.to_rust_string())
                .unwrap_or_default();
            if let Some(member) = entry.get("change_set") {
                sides[index] = inputs_of(&review_changeset(member));
            }
        }
    }
    let inputs_read = sides[0].len() + sides[1].len();
    let mut used = vec![false; sides[1].len()];
    let mut pairs = Vec::new();
    let mut held = 0usize;
    for first in &sides[0] {
        let found = sides[1].iter().enumerate().find(|(at, second)| {
            !used[*at]
                && second.file == first.file
                && second.line == first.line
                && second.pattern == first.pattern
        });
        if let Some((at, second)) = found {
            used[at] = true;
            let stands_held = first.judgement == second.judgement;
            if stands_held {
                held += 1;
            }
            pairs.push(ComparedInput {
                file: first.file.clone(),
                line: first.line,
                pattern: first.pattern.clone(),
                first: first.judgement,
                second: second.judgement,
                held: stands_held,
            });
        }
    }
    let changed = pairs.len() - held;
    Rejudged {
        first_label: labels[0].clone(),
        second_label: labels[1].clone(),
        pairs,
        inputs_read,
        held,
        changed,
    }
}
