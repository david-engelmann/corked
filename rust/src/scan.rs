//! The codepoint scan over a tree of text files.
//!
//! The scan counts the em dash U+2014 and the en dash U+2013 in
//! the files of a tree, under the scope SETUP-3 states as amended
//! by FIX-CK-7 and widened to the Rust tree with the port. A file
//! stands in scope when its suffix is one of the listed suffixes
//! or it carries no suffix at all, and a file standing under an
//! ignored directory stands out of scope whatever its suffix. The
//! reading mirrors the Python scan script this module replaces
//! under PORT-9, so a tree reads the same counts under the
//! library as it read under the script.

use std::path::Path;

/// The suffixes the scan reads, without their leading dot. A file
/// with no suffix stands in scope beside these.
const SCANNED_SUFFIXES: [&str; 8] = ["md", "txt", "py", "json", "toml", "yml", "yaml", "rs"];

/// The directory names the scan never enters. A directory whose
/// name ends in .egg-info stands ignored beside these.
const IGNORED_DIRS: [&str; 7] = [
    ".git",
    ".venv",
    "venv",
    "dist",
    "build",
    "node_modules",
    "__pycache__",
];

const EM_DASH: char = '\u{2014}';
const EN_DASH: char = '\u{2013}';

/// The counts one scan of a tree produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodepointCounts {
    pub em_dashes: usize,
    pub en_dashes: usize,
}

impl CodepointCounts {
    /// A tree is clean when neither dash stands anywhere in it.
    pub fn is_clean(&self) -> bool {
        self.em_dashes == 0 && self.en_dashes == 0
    }
}

/// The one line the scan prints, in the form the Python script
/// printed it.
pub fn report_line(counts: &CodepointCounts) -> String {
    format!(
        "codepoint scan U+2014={} U+2013={}",
        counts.em_dashes, counts.en_dashes
    )
}

fn is_ignored_dir(name: &str) -> bool {
    IGNORED_DIRS.contains(&name) || name.ends_with(".egg-info")
}

fn in_scope(path: &Path) -> bool {
    match path.extension().and_then(|suffix| suffix.to_str()) {
        None => true,
        Some(suffix) => SCANNED_SUFFIXES.contains(&suffix),
    }
}

fn count_file(path: &Path, counts: &mut CodepointCounts) {
    let Ok(bytes) = std::fs::read(path) else {
        return;
    };
    let text = String::from_utf8_lossy(&bytes);
    for ch in text.chars() {
        if ch == EM_DASH {
            counts.em_dashes += 1;
        } else if ch == EN_DASH {
            counts.en_dashes += 1;
        }
    }
}

fn walk(dir: &Path, counts: &mut CodepointCounts) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            let name = entry.file_name();
            if !is_ignored_dir(&name.to_string_lossy()) {
                walk(&path, counts);
            }
        } else if std::fs::metadata(&path).is_ok_and(|meta| meta.is_file()) && in_scope(&path) {
            count_file(&path, counts);
        }
    }
}

/// Scan the tree standing at the root and return its counts. The
/// walk prunes an ignored directory wherever it stands in the
/// tree, reads each file in scope with undecodable bytes
/// replaced, and counts the two dashes in the text it reads, so
/// the counts are a pure function of the tree on disk.
pub fn scan_tree(root: &Path) -> CodepointCounts {
    let mut counts = CodepointCounts {
        em_dashes: 0,
        en_dashes: 0,
    };
    walk(root, &mut counts);
    counts
}
