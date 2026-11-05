//! The judging census. A census reads a collection of
//! recorded review inputs together and states, per set and in
//! total, the counts the standing set review paths produce.
//! The loader validates each member through the change set
//! loader, and the tally sums what the set review and the
//! outcome helpers return, holding no second flagging,
//! judging, or outcome logic. The obligations this module
//! carries are CK-11 and CK-12 in CONTRACT.md, in the Round 5
//! section.

use crate::changeset::{review_changeset, FileResult};
use crate::checker::{BORDERLINE_PATTERNS, SHAPES};
use crate::json::JsonValue;
use crate::sets::validate_entry;
use crate::verdicts::{is_contested, line_outcome};

/// The counts of one tally, taken over one set of a census or
/// over the whole collection. The shape counts stand in the
/// order the shapes are stated across CK-3 and CK-5, and the
/// pattern counts in the order the patterns are stated in
/// CK-7.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tally {
    pub files: usize,
    pub judgements: usize,
    pub shapes: [usize; 6],
    pub borderline: usize,
    pub patterns: [usize; 3],
    pub verdicts_stand: bool,
    pub stickler: usize,
    pub clean: usize,
    pub unjudged: usize,
    pub contested: usize,
}

/// One set of a census, holding its label and its tally.
#[derive(Clone, Debug, PartialEq)]
pub struct CensusSet {
    pub label: String,
    pub tally: Tally,
}

/// A whole census, holding its sets in the order the
/// collection states them and the tally of the collection.
#[derive(Clone, Debug, PartialEq)]
pub struct Census {
    pub sets: Vec<CensusSet>,
    pub total: Tally,
}

/// Take the tally of one run of the set review.
///
/// The judgements and flags come from the results as the set
/// review returns them, and the judged and contested readings
/// come from the outcome helpers of CK-8 and CK-9. Nothing in
/// this reading flags, judges, or contests a line itself.
fn tally(results: &[FileResult]) -> Tally {
    let mut tally = Tally {
        files: results.len(),
        ..Tally::default()
    };
    for result in results {
        tally.judgements += result.judgements.len();
        for (shape, _objection) in &result.judgements {
            if let Some(index) = SHAPES.iter().position(|name| name == shape) {
                tally.shapes[index] += 1;
            }
        }
        for block in &result.blocks {
            tally.borderline += 1;
            if let Some(index) = BORDERLINE_PATTERNS
                .iter()
                .position(|name| *name == block.flag.pattern)
            {
                tally.patterns[index] += 1;
            }
            if !block.verdicts.is_empty() {
                tally.verdicts_stand = true;
            }
            match line_outcome(&block.verdicts) {
                "stickler" => tally.stickler += 1,
                "clean" => tally.clean += 1,
                _ => tally.unjudged += 1,
            }
            if is_contested(&block.verdicts) {
                tally.contested += 1;
            }
        }
    }
    tally
}

/// Return the list of defects found in a census. Never fails,
/// whatever it is handed.
///
/// Each entry is validated through the shared set entry
/// validation, the one home the census and rejudged loaders
/// both call.
pub fn load_census(value: &JsonValue) -> Vec<String> {
    let mut defects = Vec::new();
    if !value.is_object() {
        return vec!["census must be a JSON object".to_string()];
    }
    let Some(sets) = value.get("sets") else {
        return vec!["sets is missing".to_string()];
    };
    let Some(entries) = sets.as_array() else {
        return vec!["sets must be a list".to_string()];
    };
    for (index, entry) in entries.iter().enumerate() {
        validate_entry(entry, index, &mut defects);
    }
    defects
}

/// Read a valid census into its sets and its total.
///
/// The tally of each set is taken over that set alone, and the
/// tally of the collection is taken over the sets together, so
/// the flagged lines of a set holding no verdicts stand in the
/// total among the unjudged lines. A census the loader refused
/// reads as the counts its members still yield, in the posture
/// the set readings hold, and the command line never renders
/// one.
pub fn census_of(value: &JsonValue) -> Census {
    let mut sets = Vec::new();
    let mut all: Vec<FileResult> = Vec::new();
    if let Some(entries) = value.get("sets").and_then(|sets| sets.as_array()) {
        for entry in entries {
            let label = entry
                .get("label")
                .and_then(|label| label.as_jstring())
                .and_then(|text| text.to_rust_string())
                .unwrap_or_default();
            let Some(member) = entry.get("change_set") else {
                continue;
            };
            let results = review_changeset(member);
            all.extend(results.iter().cloned());
            sets.push(CensusSet {
                label,
                tally: tally(&results),
            });
        }
    }
    Census {
        total: tally(&all),
        sets,
    }
}
