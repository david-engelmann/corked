//! The pattern reading. A pattern reading takes a
//! collection of recorded review inputs in the census form
//! and states, per borderline pattern of CK-7, every flag
//! of the collection standing under that pattern and the
//! counts of the collection, so which flags stand under
//! one rule, across all sets, stands as one list rather
//! than being summed out of the census or read out of one
//! set review at a time. The loader validates each entry
//! through the shared set entry validation and each member
//! through the change set loader, the flags and verdicts
//! come from the set review, and whether a flag is
//! contested comes from the helper of CK-9, so the reading
//! holds no second flagging, judging, or contesting logic
//! of its own. The obligations this module carries are
//! CK-19 and CK-20 in CONTRACT.md, in the Round 9 section.

use crate::changeset::{review_changeset, FileResult};
use crate::checker::BORDERLINE_PATTERNS;
use crate::json::JsonValue;
use crate::sets::validate_entry;
use crate::verdicts::is_contested;

/// One flag of a pattern reading, holding the name a flag
/// is named by, the file field of its entry, the line
/// number of the flag, and the label of the set the flag
/// stands in, with whether the flag stands contested
/// under CK-9.
#[derive(Clone, Debug, PartialEq)]
pub struct PatternFlag {
    pub file: String,
    pub line: usize,
    pub set_label: String,
    pub contested: bool,
}

/// One section of a pattern reading, holding one pattern
/// of CK-7, the flags of the collection standing under
/// it in collection order, and the count of those flags
/// that stand contested.
#[derive(Clone, Debug, PartialEq)]
pub struct PatternSection {
    pub pattern: String,
    pub flags: Vec<PatternFlag>,
    pub contested: usize,
}

/// A whole pattern reading, holding its sections in the
/// order CK-7 states the patterns, the count of the sets
/// of the collection, and the closing counts of the
/// collection.
#[derive(Clone, Debug, PartialEq)]
pub struct PatternReading {
    pub sets_read: usize,
    pub sections: Vec<PatternSection>,
    pub flags_read: usize,
    pub contested_flags: usize,
}

/// One flag of the collection, gathered before the
/// sections are cut, holding the position of its pattern
/// among the patterns of CK-7 beside the flag itself.
struct Gathered {
    pattern: usize,
    flag: PatternFlag,
}

/// Gather the flags of one set from its set review
/// results, in the order the files stand in the review
/// and the flags stand in each file.
///
/// One gathered flag stands per borderline block, and
/// whether it stands contested comes from the helper of
/// CK-9 over the verdicts standing on its flag. Nothing
/// in this reading flags or contests a line itself.
fn flags_of(results: &[FileResult], set_label: &str) -> Vec<Gathered> {
    let mut gathered = Vec::new();
    for result in results {
        for block in &result.blocks {
            let Some(pattern) = BORDERLINE_PATTERNS
                .iter()
                .position(|name| *name == block.flag.pattern)
            else {
                continue;
            };
            gathered.push(Gathered {
                pattern,
                flag: PatternFlag {
                    file: result.name.clone(),
                    line: block.flag.line,
                    set_label: set_label.to_string(),
                    contested: is_contested(&block.verdicts),
                },
            });
        }
    }
    gathered
}

/// Return the list of defects found in a pattern file.
/// Never fails, whatever it is handed.
///
/// Each entry is validated through the shared set entry
/// validation, the one home the census, rejudged, and
/// contested loaders all call.
pub fn load_pattern(value: &JsonValue) -> Vec<String> {
    let mut defects = Vec::new();
    if !value.is_object() {
        return vec!["pattern must be a JSON object".to_string()];
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

/// Read a valid pattern file into its reading.
///
/// The flags of each set are gathered over the whole
/// collection in collection order and then stand under
/// their pattern in that order, so a flag stands in
/// exactly one section, the section of its pattern. A
/// file the loader refused reads as the flags its
/// members still yield, in the posture the set readings
/// hold, and the command line never renders one.
pub fn pattern_reading_of(value: &JsonValue) -> PatternReading {
    let mut gathered = Vec::new();
    let mut sets_read = 0usize;
    if let Some(entries) = value.get("sets").and_then(|sets| sets.as_array()) {
        for entry in entries {
            sets_read += 1;
            let label = entry
                .get("label")
                .and_then(|label| label.as_jstring())
                .and_then(|text| text.to_rust_string())
                .unwrap_or_default();
            if let Some(member) = entry.get("change_set") {
                gathered.extend(flags_of(&review_changeset(member), &label));
            }
        }
    }
    let mut sections = Vec::new();
    for (index, pattern) in BORDERLINE_PATTERNS.iter().enumerate() {
        let flags: Vec<PatternFlag> = gathered
            .iter()
            .filter(|held| held.pattern == index)
            .map(|held| held.flag.clone())
            .collect();
        let contested = flags.iter().filter(|flag| flag.contested).count();
        sections.push(PatternSection {
            pattern: pattern.to_string(),
            flags,
            contested,
        });
    }
    let flags_read = sections.iter().map(|section| section.flags.len()).sum();
    let contested_flags = sections.iter().map(|section| section.contested).sum();
    PatternReading {
        sets_read,
        sections,
        flags_read,
        contested_flags,
    }
}
