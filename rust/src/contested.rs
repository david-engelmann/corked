//! The contested reading. A contested reading takes a
//! collection of recorded review inputs in the census form
//! and states, per set, the flags of the set that are
//! contested under CK-9 and the counts of the collection,
//! so which flags are contested, in which sets, stands as
//! figures rather than being read out of one set review at
//! a time. The loader validates each entry through the
//! shared set entry validation and each member through the
//! change set loader, the flags and verdicts come from the
//! set review, and whether a flag is contested comes from
//! the helper of CK-9, so the reading holds no second
//! flagging, judging, or contesting logic of its own. The
//! obligations this module carries are CK-17 and CK-18 in
//! CONTRACT.md, in the Round 8 section.

use crate::changeset::{review_changeset, FileResult};
use crate::json::JsonValue;
use crate::sets::validate_entry;
use crate::verdicts::is_contested;

/// One contested flag of a set, holding the name a flag is
/// named by, the file field of its entry, the line number
/// of the flag, and the pattern of the flag.
#[derive(Clone, Debug, PartialEq)]
pub struct ContestedFlag {
    pub file: String,
    pub line: usize,
    pub pattern: String,
}

/// One set of a contested reading, holding its label, the
/// contested flags of the set in the order the set review
/// states its contested lines, and the count of the
/// flagged lines of the set.
#[derive(Clone, Debug, PartialEq)]
pub struct ContestedSet {
    pub label: String,
    pub contested: Vec<ContestedFlag>,
    pub flags_read: usize,
}

/// A whole contested reading, holding its sets in the
/// order the collection states them and the closing
/// counts of the collection.
#[derive(Clone, Debug, PartialEq)]
pub struct Contested {
    pub sets: Vec<ContestedSet>,
    pub flags_read: usize,
    pub contested_flags: usize,
    pub sets_with_contested: usize,
}

/// Read the contested flags and the flag count of one set
/// from its set review results.
///
/// One contested flag stands per borderline block whose
/// verdicts contest it under the helper of CK-9, in the
/// order the files stand in the review and the flags
/// stand in each file, the order the contested section of
/// CK-9 states its lines in. Nothing in this reading
/// flags or contests a line itself.
fn contested_of(results: &[FileResult]) -> (Vec<ContestedFlag>, usize) {
    let mut contested = Vec::new();
    let mut flags_read = 0usize;
    for result in results {
        for block in &result.blocks {
            flags_read += 1;
            if is_contested(&block.verdicts) {
                contested.push(ContestedFlag {
                    file: result.name.clone(),
                    line: block.flag.line,
                    pattern: block.flag.pattern.clone(),
                });
            }
        }
    }
    (contested, flags_read)
}

/// Return the list of defects found in a contested file.
/// Never fails, whatever it is handed.
///
/// Each entry is validated through the shared set entry
/// validation, the one home the census and rejudged
/// loaders both call.
pub fn load_contested(value: &JsonValue) -> Vec<String> {
    let mut defects = Vec::new();
    if !value.is_object() {
        return vec!["contested must be a JSON object".to_string()];
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

/// Read a valid contested file into its reading.
///
/// The contested flags of each set are taken over that set
/// alone, and the closing counts are summed over the
/// sections the reading states, so the closing contested
/// flags count is the sum of the contested counts of the
/// sections by construction. A file the loader refused
/// reads as the flags its members still yield, in the
/// posture the set readings hold, and the command line
/// never renders one.
pub fn contested_reading_of(value: &JsonValue) -> Contested {
    let mut sets = Vec::new();
    if let Some(entries) = value.get("sets").and_then(|sets| sets.as_array()) {
        for entry in entries {
            let label = entry
                .get("label")
                .and_then(|label| label.as_jstring())
                .and_then(|text| text.to_rust_string())
                .unwrap_or_default();
            let (contested, flags_read) = match entry.get("change_set") {
                Some(member) => contested_of(&review_changeset(member)),
                None => (Vec::new(), 0),
            };
            sets.push(ContestedSet {
                label,
                contested,
                flags_read,
            });
        }
    }
    let flags_read = sets.iter().map(|set| set.flags_read).sum();
    let contested_flags = sets.iter().map(|set| set.contested.len()).sum();
    let sets_with_contested = sets.iter().filter(|set| !set.contested.is_empty()).count();
    Contested {
        sets,
        flags_read,
        contested_flags,
        sets_with_contested,
    }
}
