//! Pins for SETUP-3 as amended by FIX-CK-7.
//!
//! Translated from tests/test_scan.py, and rewritten with the
//! removal slice of PORT-9 to read the library scan directly
//! instead of running the Python script as a subprocess, because
//! the scan lives in the crate now. Each case plants a real tree
//! on disk and reads the counts the library returns for it, so
//! each case can fail for a real reason in either direction. The
//! exit status the old pins read belonged to the script process
//! and belongs to the thin codepoint_scan binary now.

use std::path::PathBuf;

use stickler::scan::{report_line, scan_tree, CodepointCounts};

const EM_DASH: &str = "\u{2014}";
const EN_DASH: &str = "\u{2013}";

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "stickler-scan-{}-{}-{}",
        tag,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn planted_tree(tag: &str, files: &[(&str, String)]) -> CodepointCounts {
    let dir = temp_dir(tag);
    for (relative, text) in files {
        let path = dir.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
    }
    let counts = scan_tree(&dir);
    let _ = std::fs::remove_dir_all(&dir);
    counts
}

#[test]
fn a_dash_in_an_extensionless_file_fails_the_scan() {
    let counts = planted_tree(
        "extensionless",
        &[("NOTES", format!("a note {EM_DASH} held back\n"))],
    );
    assert!(!counts.is_clean());
    assert_eq!(counts.em_dashes, 1);
    assert_eq!(report_line(&counts), "codepoint scan U+2014=1 U+2013=0");
}

#[test]
fn a_dash_in_a_python_file_fails_the_scan() {
    let counts = planted_tree(
        "python-file",
        &[("app.py", format!("# a note {EN_DASH} held\n"))],
    );
    assert!(!counts.is_clean());
    assert_eq!(counts.en_dashes, 1);
    assert_eq!(report_line(&counts), "codepoint scan U+2014=0 U+2013=1");
}

#[test]
fn dashes_only_under_an_ignored_directory_pass_the_scan() {
    let counts = planted_tree(
        "ignored",
        &[(
            ".venv/notes.md",
            format!("a note {EM_DASH} and {EN_DASH}\n"),
        )],
    );
    assert!(counts.is_clean());
    assert_eq!(counts.em_dashes, 0);
    assert_eq!(counts.en_dashes, 0);
}

#[test]
fn a_clean_tree_passes_the_scan() {
    let counts = planted_tree(
        "clean",
        &[
            ("app.py", "total = 1\n".to_string()),
            ("README.md", "plain words\n".to_string()),
        ],
    );
    assert!(counts.is_clean());
    assert_eq!(counts.em_dashes, 0);
    assert_eq!(counts.en_dashes, 0);
    assert_eq!(report_line(&counts), "codepoint scan U+2014=0 U+2013=0");
}

#[test]
fn a_dash_in_a_rust_file_fails_the_scan() {
    let counts = planted_tree(
        "rust-file",
        &[("lib.rs", format!("// a note {EM_DASH} held\n"))],
    );
    assert!(!counts.is_clean());
    assert_eq!(counts.em_dashes, 1);
    assert_eq!(report_line(&counts), "codepoint scan U+2014=1 U+2013=0");
}
