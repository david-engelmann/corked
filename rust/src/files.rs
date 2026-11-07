//! The file reading. A file reading takes a collection
//! of recorded review inputs in the census form and
//! states, per file standing in the collection, every
//! flag of the collection standing on that file and the
//! counts of the collection, so which flags stand on
//! one file, across all sets, stands as one list rather
//! than being read out of one set review at a time. It
//! exists for the same recorded inputs as the census,
//! contested, and pattern readings, over the same
//! collection loading whole into all four. The loader
//! validates each entry through the shared set entry
//! validation and each member through the change set
//! loader, the flags and verdicts come from the set
//! review, and whether a flag is contested comes from
//! the helper of CK-9, so the reading holds no second
//! flagging, judging, or contesting logic of its own.
//! The obligations this module carries are CK-21 and
//! CK-22 in CONTRACT.md, in the Round 10 section.

use crate::changeset::review_changeset;
use crate::json::JsonValue;
use crate::sets::validate_entry;
use crate::verdicts::is_contested;

/// One flag of a file reading, holding the line number
/// of the flag, the pattern of the flag, and the label
/// of the set the flag stands in, with whether the flag
/// stands contested under CK-9.
#[derive(Clone, Debug, PartialEq)]
pub struct FileFlag {
    pub line: usize,
    pub pattern: String,
    pub set_label: String,
    pub contested: bool,
}

/// One section of a file reading, holding one file
/// standing in the collection, the flags of the
/// collection standing on that file in collection
/// order, and the count of those flags that stand
/// contested.
#[derive(Clone, Debug, PartialEq)]
pub struct FileSection {
    pub file: String,
    pub flags: Vec<FileFlag>,
    pub contested: usize,
}

/// A whole file reading, holding its sections in first
/// appearance order across the collection, the count of
/// the sets of the collection, and the closing counts
/// of the collection.
#[derive(Clone, Debug, PartialEq)]
pub struct FilesReading {
    pub sets_read: usize,
    pub sections: Vec<FileSection>,
    pub flags_read: usize,
    pub contested_flags: usize,
}

/// Return the list of defects found in a files file.
/// Never fails, whatever it is handed.
///
/// Each entry is validated through the shared set entry
/// validation, the one home the census, rejudged,
/// contested, and pattern loaders all call.
pub fn load_files(value: &JsonValue) -> Vec<String> {
    let mut defects = Vec::new();
    if !value.is_object() {
        return vec!["files must be a JSON object".to_string()];
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

/// Read a valid files file into its reading.
///
/// The flags of each set are gathered in collection
/// order and stand under their file in that order, a
/// file standing in more than one set standing once, at
/// its first appearance. A file standing in the
/// collection with no flag standing on it in any set
/// stands no section, because the reading states the
/// flags of a file. A file the loader refused reads as
/// the flags its members still yield, in the posture
/// the set readings hold, and the command line never
/// renders one.
pub fn files_reading_of(value: &JsonValue) -> FilesReading {
    let mut sections: Vec<FileSection> = Vec::new();
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
                for result in review_changeset(member) {
                    for block in &result.blocks {
                        let flag = FileFlag {
                            line: block.flag.line,
                            pattern: block.flag.pattern.clone(),
                            set_label: label.clone(),
                            contested: is_contested(&block.verdicts),
                        };
                        match sections
                            .iter_mut()
                            .find(|section| section.file == result.name)
                        {
                            Some(section) => section.flags.push(flag),
                            None => sections.push(FileSection {
                                file: result.name.clone(),
                                flags: vec![flag],
                                contested: 0,
                            }),
                        }
                    }
                }
            }
        }
    }
    for section in &mut sections {
        section.contested = section.flags.iter().filter(|flag| flag.contested).count();
    }
    let flags_read = sections.iter().map(|section| section.flags.len()).sum();
    let contested_flags = sections.iter().map(|section| section.contested).sum();
    FilesReading {
        sets_read,
        sections,
        flags_read,
        contested_flags,
    }
}
