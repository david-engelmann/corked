//! Pins for SETUP-3 as amended by FIX-CK-7. The scan runs as
//! a subprocess against a planted tree, so each case can fail
//! for a real reason in either direction. Translated from
//! tests/test_scan.py.

use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

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

fn planted_tree(tag: &str, files: &[(&str, String)]) -> (i32, String) {
    let dir = temp_dir(tag);
    let scripts = dir.join("scripts");
    std::fs::create_dir_all(&scripts).unwrap();
    std::fs::copy(
        root().join("scripts").join("codepoint_scan.py"),
        scripts.join("codepoint_scan.py"),
    )
    .unwrap();
    for (relative, text) in files {
        let path = dir.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, text).unwrap();
    }
    let output = Command::new("python3")
        .arg(scripts.join("codepoint_scan.py"))
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let code = output.status.code().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    (code, stdout)
}

#[test]
fn a_dash_in_an_extensionless_file_fails_the_scan() {
    let (code, stdout) = planted_tree(
        "extensionless",
        &[("NOTES", format!("a note {EM_DASH} held back\n"))],
    );
    assert_eq!(code, 1);
    assert!(stdout.contains("U+2014=1"));
}

#[test]
fn a_dash_in_a_python_file_fails_the_scan() {
    let (code, stdout) = planted_tree(
        "python-file",
        &[("app.py", format!("# a note {EN_DASH} held\n"))],
    );
    assert_eq!(code, 1);
    assert!(stdout.contains("U+2013=1"));
}

#[test]
fn dashes_only_under_an_ignored_directory_pass_the_scan() {
    let (code, stdout) = planted_tree(
        "ignored",
        &[(
            ".venv/notes.md",
            format!("a note {EM_DASH} and {EN_DASH}\n"),
        )],
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("U+2014=0"));
    assert!(stdout.contains("U+2013=0"));
}

#[test]
fn a_clean_tree_passes_the_scan() {
    let (code, stdout) = planted_tree(
        "clean",
        &[
            ("app.py", "total = 1\n".to_string()),
            ("README.md", "plain words\n".to_string()),
        ],
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("U+2014=0"));
    assert!(stdout.contains("U+2013=0"));
}

#[test]
fn a_dash_in_a_rust_file_fails_the_scan() {
    let (code, stdout) = planted_tree(
        "rust-file",
        &[("lib.rs", format!("// a note {EM_DASH} held\n"))],
    );
    assert_eq!(code, 1);
    assert!(stdout.contains("U+2014=1"));
}
