//! Pins for CK-4. The two commands and the exit code per
//! verdict. Translated from tests/test_cli.py.

use std::path::{Path, PathBuf};

use stickler::cli::cli_main;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "stickler-cli-{}-{}-{}",
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

fn write(dir: &Path, name: &str, text: &str) -> String {
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path.to_string_lossy().into_owned()
}

fn write_bytes(dir: &Path, name: &str, data: &[u8]) -> String {
    let path = dir.join(name);
    std::fs::write(&path, data).unwrap();
    path.to_string_lossy().into_owned()
}

fn run(args: &[&str]) -> (i32, String, String) {
    let owned: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    let result = cli_main(&owned);
    (result.code, result.stdout, result.stderr)
}

const SMELLY: &str = "--- a/app.py\n+++ b/app.py\n@@ -1,1 +1,2 @@\n+    print(total)\n";
const CLEAN: &str = "--- a/app.py\n+++ b/app.py\n@@ -1,1 +1,2 @@\n+    logger.debug(total)\n";

/// Escape a string as JSON string content.
fn jstr(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if (ch as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", ch as u32));
            }
            ch => out.push(ch),
        }
    }
    out
}

fn set_entry_json(name: &str, added_lines: &[&str]) -> String {
    let mut lines = vec![
        "--- a/x.py".to_string(),
        "+++ b/x.py".to_string(),
        "@@ -1,1 +1,1 @@".to_string(),
    ];
    for line in added_lines {
        lines.push(format!("+{line}"));
    }
    let mut diff = lines.join("\n");
    diff.push('\n');
    format!(
        "{{\"file\": \"{}\", \"diff\": \"{}\"}}",
        jstr(name),
        jstr(&diff)
    )
}

fn change_set(entries: &[String]) -> String {
    format!("{{\"files\": [{}]}}", entries.join(", "))
}

#[test]
fn check_on_a_smelly_diff_sends_back_with_exit_1() {
    let dir = temp_dir("smelly");
    let path = write(&dir, "change.diff", SMELLY);
    let (code, out, _err) = run(&["check", &path]);
    assert_eq!(code, 1);
    assert!(out.contains("Verdict, sent back\n"));
    assert!(out.contains("Objection 1, smell\n"));
}

#[test]
fn check_on_a_clean_diff_accepts_with_exit_0() {
    let dir = temp_dir("clean");
    let path = write(&dir, "change.diff", CLEAN);
    let (code, out, _err) = run(&["check", &path]);
    assert_eq!(code, 0);
    assert!(out.contains("Verdict, accepted\n"));
}

#[test]
fn check_labels_the_review_with_the_file_name() {
    let dir = temp_dir("label");
    let path = write(&dir, "change.diff", CLEAN);
    let (_code, out, _err) = run(&["check", &path]);
    assert!(out.starts_with("Review of change.diff\n"));
}

#[test]
fn check_on_a_missing_file_is_invalid_with_exit_2() {
    let dir = temp_dir("gone");
    let missing = dir.join("gone.diff").to_string_lossy().into_owned();
    let (code, out, _err) = run(&["check", &missing]);
    assert_eq!(code, 2);
    assert!(out.contains("Verdict, invalid\n"));
    assert!(out.contains("Defect, the diff file could not be read\n"));
}

#[test]
fn render_on_an_accepted_review_exits_0() {
    let dir = temp_dir("render-ok");
    let path = write(
        &dir,
        "review.json",
        "{\"diff_label\": \"tidy\", \"objections\": []}",
    );
    let (code, out, _err) = run(&["render", &path]);
    assert_eq!(code, 0);
    assert!(out.contains("Verdict, accepted\n"));
}

#[test]
fn render_on_a_sent_back_review_exits_1() {
    let dir = temp_dir("render-back");
    let path = write(
        &dir,
        "review.json",
        "{\"diff_label\": \"rough\", \"objections\": [{\
         \"flaw\": \"The name says nothing\", \
         \"standard\": \"A name says what its value holds\", \
         \"rewrite\": \"orders = load_orders()\", \
         \"category\": \"naming\"}]}",
    );
    let (code, out, _err) = run(&["render", &path]);
    assert_eq!(code, 1);
    assert!(out.contains("Verdict, sent back\n"));
}

#[test]
fn render_on_a_non_json_file_is_invalid_with_exit_2() {
    let dir = temp_dir("render-nonjson");
    let path = write(&dir, "review.json", "not json");
    let (code, out, _err) = run(&["render", &path]);
    assert_eq!(code, 2);
    assert!(out.contains("Defect, the file does not hold a JSON review\n"));
}

#[test]
fn render_on_an_incomplete_objection_exits_2_and_names_the_field() {
    let dir = temp_dir("render-incomplete");
    let path = write(
        &dir,
        "review.json",
        "{\"diff_label\": \"rough\", \"objections\": [{\"flaw\": \"x\"}]}",
    );
    let (code, out, _err) = run(&["render", &path]);
    assert_eq!(code, 2);
    assert!(out.contains("Defect, objections[0].standard is missing\n"));
}

#[test]
fn render_on_a_missing_file_is_invalid_with_exit_2() {
    let dir = temp_dir("render-gone");
    let missing = dir.join("gone.json").to_string_lossy().into_owned();
    let (code, out, _err) = run(&["render", &missing]);
    assert_eq!(code, 2);
    assert!(out.contains("Defect, the review file could not be read\n"));
}

#[test]
fn check_on_a_non_utf8_diff_is_invalid_with_exit_2() {
    let dir = temp_dir("check-nonutf8");
    let data = b"--- a/app.py\n+++ b/app.py\n@@ -1,1 +1,1 @@\n+total = b'\xff\xfe'\n";
    let path = write_bytes(&dir, "change.diff", data);
    let (code, out, _err) = run(&["check", &path]);
    assert_eq!(code, 2);
    assert!(out.contains("Verdict, invalid\n"));
    assert!(out.contains("Defect, the diff file could not be read\n"));
}

#[test]
fn render_on_a_non_utf8_review_is_invalid_with_exit_2() {
    let dir = temp_dir("render-nonutf8");
    let data = b"{\"diff_label\": \"bad \xff\xfe\", \"objections\": []}";
    let path = write_bytes(&dir, "review.json", data);
    let (code, out, _err) = run(&["render", &path]);
    assert_eq!(code, 2);
    assert!(out.contains("Verdict, invalid\n"));
    assert!(out.contains("Defect, the review file could not be read\n"));
}

#[test]
fn usage_errors_exit_2() {
    assert_eq!(run(&[]).0, 2);
    assert_eq!(run(&["check"]).0, 2);
    assert_eq!(run(&["bogus", "x"]).0, 2);
}

#[test]
fn the_smelly_example_sends_back_and_the_clean_example_accepts() {
    let smelly = root()
        .join("examples")
        .join("smelly.diff")
        .to_string_lossy()
        .into_owned();
    let (code, out, _err) = run(&["check", &smelly]);
    assert_eq!(code, 1);
    assert!(out.contains("Verdict, sent back\n"));
    let clean = root()
        .join("examples")
        .join("clean.diff")
        .to_string_lossy()
        .into_owned();
    let (code, out, _err) = run(&["check", &clean]);
    assert_eq!(code, 0);
    assert!(out.contains("Verdict, accepted\n"));
}

#[test]
fn review_on_a_clean_set_exits_0() {
    let dir = temp_dir("review-clean");
    let path = write(
        &dir,
        "set.json",
        &change_set(&[set_entry_json("a.py", &["    logger.debug(total)"])]),
    );
    let (code, out, _err) = run(&["review", &path]);
    assert_eq!(code, 0);
    assert!(out.contains("File, a.py\n"));
    assert!(out.contains("Clean\n"));
    assert!(out.contains("Judgements, 0\n"));
}

#[test]
fn review_on_a_set_with_judgements_exits_1() {
    let dir = temp_dir("review-judged");
    let path = write(
        &dir,
        "set.json",
        &change_set(&[set_entry_json("a.py", &["    print(total)"])]),
    );
    let (code, out, _err) = run(&["review", &path]);
    assert_eq!(code, 1);
    assert!(out.contains("Judgement 1, smell\n"));
    assert!(out.contains("Worst, a.py, 1\n"));
}

#[test]
fn review_on_a_non_json_file_is_malformed_with_exit_2() {
    let dir = temp_dir("review-nonjson");
    let path = write(&dir, "set.json", "not json");
    let (code, out, _err) = run(&["review", &path]);
    assert_eq!(code, 2);
    assert!(out.contains("Verdict, invalid\n"));
    assert!(out.contains("Defect, the file does not hold a JSON change set\n"));
}

#[test]
fn review_on_a_set_with_a_loader_defect_exits_2_and_names_it() {
    let dir = temp_dir("review-defect");
    let path = write(
        &dir,
        "set.json",
        &change_set(&["{\"file\": \"a.py\"}".to_string()]),
    );
    let (code, out, _err) = run(&["review", &path]);
    assert_eq!(code, 2);
    assert!(out.contains("Defect, files[0].diff is missing\n"));
}

#[test]
fn review_on_an_oversized_integer_confidence_exits_2_without_a_traceback() {
    let dir = temp_dir("review-huge");
    let huge = "1".to_string() + &"0".repeat(400);
    let mut entry = set_entry_json("a.py", &["data = load_entries()"]);
    entry.pop();
    entry.push_str(&format!(
        ", \"verdicts\": [{{\"line\": 4, \
         \"pattern\": \"near placeholder name\", \
         \"model\": \"jev-1.13.0\", \"choice\": \"stickler\", \
         \"confidence\": {huge}}}]}}"
    ));
    let path = write(&dir, "set.json", &change_set(&[entry]));
    let (code, out, err) = run(&["review", &path]);
    assert_eq!(code, 2);
    assert!(out.contains("Defect, files[0].verdicts[0].confidence must be from 0 to 1\n"));
    assert!(!err.contains("Traceback"));
}

#[test]
fn review_on_a_missing_file_is_malformed_with_exit_2() {
    let dir = temp_dir("review-gone");
    let missing = dir.join("gone.json").to_string_lossy().into_owned();
    let (code, out, _err) = run(&["review", &missing]);
    assert_eq!(code, 2);
    assert!(out.contains("Defect, the change set file could not be read\n"));
}

#[test]
fn review_on_a_non_utf8_file_is_malformed_with_exit_2() {
    let dir = temp_dir("review-nonutf8");
    let data = b"{\"files\": [{\"file\": \"bad \xff\xfe\", \"diff\": \"\"}]}";
    let path = write_bytes(&dir, "set.json", data);
    let (code, out, _err) = run(&["review", &path]);
    assert_eq!(code, 2);
    assert!(out.contains("Defect, the change set file could not be read\n"));
}

#[test]
fn review_on_a_deeply_nested_file_is_malformed_with_exit_2() {
    let dir = temp_dir("review-deep");
    let data = "[".repeat(100_000) + &"]".repeat(100_000);
    let path = write_bytes(&dir, "set.json", data.as_bytes());
    let (code, out, _err) = run(&["review", &path]);
    assert_eq!(code, 2);
    assert!(out.contains("Defect, the file does not hold a JSON change set\n"));
}

#[test]
fn review_usage_errors_exit_2() {
    assert_eq!(run(&["review"]).0, 2);
    assert_eq!(run(&["review", "a", "b"]).0, 2);
}

#[test]
fn review_on_a_surrogate_file_name_is_malformed_with_exit_2() {
    let dir = temp_dir("review-surrogate-name");
    // The file holds the escape, as json.dumps writes it.
    let path = write(
        &dir,
        "set.json",
        "{\"files\": [{\"file\": \"\\ud800x.py\", \"diff\": \"\"}]}",
    );
    let (code, out, err) = run(&["review", &path]);
    assert_eq!(code, 2);
    assert!(out.contains("Defect, files[0].file holds an unpaired surrogate\n"));
    assert!(!err.contains("Traceback"));
}

#[test]
fn review_on_a_surrogate_in_diff_text_is_malformed_with_exit_2() {
    let dir = temp_dir("review-surrogate-diff");
    let path = write(
        &dir,
        "set.json",
        "{\"files\": [{\"file\": \"a.py\", \
         \"diff\": \"+print('\\ud800')\"}]}",
    );
    let (code, out, err) = run(&["review", &path]);
    assert_eq!(code, 2);
    assert!(out.contains("Defect, files[0].diff holds an unpaired surrogate\n"));
    assert!(!err.contains("Traceback"));
}

#[test]
fn render_on_a_surrogate_diff_label_is_invalid_with_exit_2() {
    let dir = temp_dir("render-surrogate-label");
    let path = write(
        &dir,
        "review.json",
        "{\"diff_label\": \"\\ud800x\", \"objections\": []}",
    );
    let (code, out, err) = run(&["render", &path]);
    assert_eq!(code, 2);
    assert!(out.starts_with("Review of unnamed review\n"));
    assert!(out.contains("Defect, diff_label holds an unpaired surrogate\n"));
    assert!(!err.contains("Traceback"));
}

#[test]
fn render_on_a_surrogate_objection_field_is_invalid_with_exit_2() {
    let dir = temp_dir("render-surrogate-field");
    let path = write(
        &dir,
        "review.json",
        "{\"diff_label\": \"rough\", \"objections\": [{\
         \"flaw\": \"bad \\ud800 text\", \
         \"standard\": \"A name says what its value holds\", \
         \"rewrite\": \"orders = load_orders()\", \
         \"category\": \"naming\"}]}",
    );
    let (code, out, err) = run(&["render", &path]);
    assert_eq!(code, 2);
    assert!(out.contains("Defect, objections[0].flaw holds an unpaired surrogate\n"));
    assert!(!err.contains("Traceback"));
}

const BROAD_HANDLER_DIFF: &str =
    "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,1 @@\n+    except BaseException:\n";

#[test]
fn review_on_a_line_break_file_name_is_malformed_with_exit_2() {
    let dir = temp_dir("review-linebreak-name");
    let entry = format!(
        "{{\"file\": \"{}\", \"diff\": \"{}\", \"verdicts\": [\
         {{\"line\": 4, \"pattern\": \"broad handler\", \
         \"model\": \"jev-1.13.0\", \"choice\": \"stickler\", \
         \"confidence\": 0.78}}]}}",
        jstr("evil\nForged, 99"),
        jstr(BROAD_HANDLER_DIFF)
    );
    let path = write(&dir, "set.json", &change_set(&[entry]));
    let (code, out, err) = run(&["review", &path]);
    assert_eq!(code, 2);
    assert!(out.contains("Defect, files[0].file holds a line break\n"));
    assert!(!out.contains("Forged"));
    assert!(!err.contains("Traceback"));
}

#[test]
fn render_on_a_line_break_diff_label_is_invalid_with_exit_2() {
    let dir = temp_dir("render-linebreak-label");
    let path = write(
        &dir,
        "review.json",
        "{\"diff_label\": \"evil\\nForged, 99\", \"objections\": []}",
    );
    let (code, out, err) = run(&["render", &path]);
    assert_eq!(code, 2);
    assert!(out.starts_with("Review of unnamed review\n"));
    assert!(out.contains("Defect, diff_label holds a line break\n"));
    assert!(!out.contains("Forged"));
    assert!(!err.contains("Traceback"));
}

const BORDERLINE_DIFF: &str = "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,1 @@\n+data = load_entries()\n";

fn verdict_set_json(verdicts: &[String]) -> String {
    format!(
        "{{\"files\": [{{\"file\": \"x.py\", \"diff\": \"{}\", \
         \"verdicts\": [{}]}}]}}",
        jstr(BORDERLINE_DIFF),
        verdicts.join(", ")
    )
}

fn one_verdict_json(choice: &str, confidence: &str) -> String {
    format!(
        "{{\"line\": 4, \"pattern\": \"near placeholder name\", \
         \"model\": \"jev-1.13.0\", \"choice\": \"{choice}\", \
         \"confidence\": {confidence}}}"
    )
}

#[test]
fn review_with_a_stickler_verdict_exits_1() {
    let dir = temp_dir("review-stickler-verdict");
    let path = write(
        &dir,
        "set.json",
        &verdict_set_json(&[one_verdict_json("stickler", "0.78")]),
    );
    let (code, out, _err) = run(&["review", &path]);
    assert_eq!(code, 1);
    assert!(out.contains("Judgements, 0"));
    assert!(out.contains("Borderline 1, near placeholder name"));
    assert!(out.contains("Line, data = load_entries()"));
    assert!(out.contains("Shape reading, drawn name entries"));
    assert!(out.contains("Verdict, jev-1.13.0, stickler"));
    assert!(out.contains("Reason, jev-1.13.0 chose stickler with confidence 0.78"));
    assert!(out.contains("Borderline, 1"));
    assert!(out.contains("Verdict lines, stickler 1, clean 0"));
    assert!(out.contains("Unjudged, 0"));
}

#[test]
fn review_with_only_clean_verdicts_exits_0() {
    let dir = temp_dir("review-clean-verdict");
    let path = write(
        &dir,
        "set.json",
        &verdict_set_json(&[one_verdict_json("clean", "0.78")]),
    );
    let (code, out, _err) = run(&["review", &path]);
    assert_eq!(code, 0);
    assert!(out.contains("Verdict lines, stickler 0, clean 1"));
}

#[test]
fn review_with_only_an_abstention_exits_0_unjudged() {
    let dir = temp_dir("review-abstention");
    let path = write(
        &dir,
        "set.json",
        &verdict_set_json(&[one_verdict_json("stickler", "0.2")]),
    );
    let (code, out, _err) = run(&["review", &path]);
    assert_eq!(code, 0);
    assert!(out.contains("Verdict, jev-1.13.0, abstained"));
    assert!(out.contains("and abstained"));
    assert!(out.contains("\nUnjudged\n"));
    assert!(out.contains("Unjudged, 1"));
}

#[test]
fn review_with_a_malformed_verdict_exits_2() {
    let dir = temp_dir("review-malformed-verdict");
    let path = write(
        &dir,
        "set.json",
        &verdict_set_json(&[one_verdict_json("maybe", "0.78")]),
    );
    let (code, out, _err) = run(&["review", &path]);
    assert_eq!(code, 2);
    assert!(out.contains("files[0].verdicts[0].choice must be one of stickler, clean"));
}

#[test]
fn review_of_a_flagged_file_draws_no_clean_line() {
    let dir = temp_dir("review-flagged-clean");
    let path = write(&dir, "set.json", &verdict_set_json(&[]));
    let (code, out, _err) = run(&["review", &path]);
    assert_eq!(code, 0);
    assert!(out.contains("File, x.py\n\nBorderline 1, near placeholder name"));
    assert!(out.contains("Unjudged, 1"));
}

// The pins below stand for the argument byte repair. The
// interpreter decodes its arguments under the surrogateescape
// handler, so a file name carrying a raw byte runs under every
// command, and the port carries the arguments as code units to
// stand the same. The expected texts are the byte strings the
// Python command line itself prints on this machine.

use std::os::unix::ffi::OsStringExt;

use stickler::cli::{cli_main_units, ByteOutcome};
use stickler::json::JString;

/// The full path of a file standing under a raw byte name, as
/// bytes.
fn raw_path(dir: &Path, name: &[u8]) -> Vec<u8> {
    let mut bytes = dir.as_os_str().as_encoded_bytes().to_vec();
    bytes.push(b'/');
    bytes.extend_from_slice(name);
    bytes
}

fn write_raw(dir: &Path, name: &[u8], text: &str) -> Vec<u8> {
    let bytes = raw_path(dir, name);
    let path = std::ffi::OsString::from_vec(bytes.clone());
    std::fs::write(path, text).unwrap();
    bytes
}

fn run_units(args: &[&[u8]]) -> ByteOutcome {
    let owned: Vec<JString> = args
        .iter()
        .map(|arg| JString::from_surrogateescape(arg))
        .collect();
    cli_main_units(&owned)
}

// Adjusted with the Round 6 slice, as the named mechanical
// consequence of CK-14 adding the rejudged command to the
// command line. The usage text gains the rejudged line, so
// the pinned bytes gain it too, and the three assertions
// standing on this constant hold the new text. The constant
// was adjusted before with the Round 5 slice, when CK-12
// added the census command in the same way.
const USAGE_BYTES: &[u8] = b"Usage, stickler check DIFF_FILE\n\
    Usage, stickler render REVIEW_JSON\n\
    Usage, stickler review SET_JSON\n\
    Usage, stickler census CENSUS_JSON\n\
    Usage, stickler rejudged REJUDGED_JSON\n";

#[test]
fn check_names_a_readable_raw_byte_diff_by_its_surrogate_defect() {
    let dir = temp_dir("raw-byte-readable");
    let path = write_raw(&dir, b"bad\xffname.diff", "@@ -0,0 +1 @@\n+print('x')\n");
    let result = run_units(&[b"check", &path]);
    assert_eq!(result.code, 2);
    assert_eq!(
        result.stdout,
        b"Review of unnamed review\nVerdict, invalid\nDefects, 1\n\
          Defect, diff_label holds an unpaired surrogate\n"
            .to_vec()
    );
    assert!(result.stderr.is_empty());
}

#[test]
fn check_names_a_missing_raw_byte_diff_by_the_raw_byte() {
    let dir = temp_dir("raw-byte-missing");
    let path = raw_path(&dir, b"no\xffpe.diff");
    let result = run_units(&[b"check", &path]);
    assert_eq!(result.code, 2);
    assert_eq!(
        result.stdout,
        b"Review of no\xffpe.diff\nVerdict, invalid\nDefects, 1\n\
          Defect, the diff file could not be read\n"
            .to_vec()
    );
}

#[test]
fn render_names_an_unreadable_raw_byte_review_by_the_constant_label() {
    let dir = temp_dir("raw-byte-render");
    let path = raw_path(&dir, b"no\xffpe.diff");
    let result = run_units(&[b"render", &path]);
    assert_eq!(result.code, 2);
    assert_eq!(
        result.stdout,
        b"Review of unnamed review\nVerdict, invalid\nDefects, 1\n\
          Defect, the review file could not be read\n"
            .to_vec()
    );
}

#[test]
fn review_renders_a_valid_set_under_a_raw_byte_name() {
    let dir = temp_dir("raw-byte-review");
    let path = write_raw(&dir, b"set\xffname.json", "{\"files\": []}");
    let result = run_units(&[b"review", &path]);
    assert_eq!(result.code, 0);
    assert_eq!(
        result.stdout,
        b"Change set review\nFiles, 0\n\nSummary\nJudgements, 0\n".to_vec()
    );
}

#[test]
fn a_command_word_with_a_raw_byte_draws_the_usage() {
    let dir = temp_dir("raw-byte-word");
    let path = write_raw(&dir, b"bad\xffname.diff", SMELLY);
    let result = run_units(&[b"ch\xffeck", &path]);
    assert_eq!(result.code, 2);
    assert!(result.stdout.is_empty());
    assert_eq!(result.stderr, USAGE_BYTES.to_vec());
}

#[test]
fn an_extra_argument_with_a_raw_byte_draws_the_usage() {
    let dir = temp_dir("raw-byte-extra");
    let path = write_raw(&dir, b"bad\xffname.diff", SMELLY);
    let result = run_units(&[b"check", &path, b"ex\xfftra"]);
    assert_eq!(result.code, 2);
    assert!(result.stdout.is_empty());
    assert_eq!(result.stderr, USAGE_BYTES.to_vec());
}

#[test]
fn check_names_a_missing_dotdot_path_by_the_dotdot() {
    for path in ["..", "a/..", "a/../", "../", "a/b/../..", "a//..//"] {
        let (code, out, _err) = run(&["check", path]);
        assert_eq!(code, 2, "path {path}");
        assert!(
            out.starts_with("Review of ..\n"),
            "path {path} gave {out:?}"
        );
        assert!(out.contains("Defect, the diff file could not be read"));
    }
}

#[test]
fn check_names_a_missing_dot_path_as_pathlib_names_it() {
    for (path, first) in [
        (".", "Review of \n"),
        ("./", "Review of \n"),
        ("a/.", "Review of a\n"),
        ("a/b/.", "Review of b\n"),
    ] {
        let (code, out, _err) = run(&["check", path]);
        assert_eq!(code, 2, "path {path}");
        assert!(out.starts_with(first), "path {path} gave {out:?}");
    }
}

/// Spawn the built binary with raw byte arguments and return
/// its exit code and its two output streams as bytes.
fn run_binary(args: &[&[u8]]) -> (i32, Vec<u8>, Vec<u8>) {
    let owned: Vec<std::ffi::OsString> = args
        .iter()
        .map(|arg| std::ffi::OsString::from_vec(arg.to_vec()))
        .collect();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_stickler"))
        .args(&owned)
        .output()
        .unwrap();
    (output.status.code().unwrap(), output.stdout, output.stderr)
}

#[test]
fn the_binary_runs_a_readable_raw_byte_diff() {
    let dir = temp_dir("binary-raw-byte");
    let path = write_raw(&dir, b"bad\xffname.diff", "@@ -0,0 +1 @@\n+print('x')\n");
    let (code, out, err) = run_binary(&[b"check", &path]);
    assert_eq!(code, 2);
    assert_eq!(
        out,
        b"Review of unnamed review\nVerdict, invalid\nDefects, 1\n\
          Defect, diff_label holds an unpaired surrogate\n"
            .to_vec()
    );
    assert!(err.is_empty());
}

#[test]
fn the_binary_draws_the_usage_for_a_raw_byte_command_word() {
    let (code, out, err) = run_binary(&[b"ch\xffeck", b"whatever.diff"]);
    assert_eq!(code, 2);
    assert!(out.is_empty());
    assert_eq!(err, USAGE_BYTES.to_vec());
}

/// Spawn the built binary with the named stream standing on a
/// socket whose peer is dropped before the spawn, so the
/// stream stands closed from the binary's first moment, as a
/// closed pipe stands for the Python probes, and return the
/// exit code.
fn run_binary_with_closed_stream(args: &[&str], closed: &str) -> i32 {
    use std::os::unix::io::{FromRawFd, IntoRawFd};
    let (peer, stream_end) = std::os::unix::net::UnixStream::pair().unwrap();
    let file = unsafe { std::fs::File::from_raw_fd(stream_end.into_raw_fd()) };
    drop(peer);
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_stickler"));
    command.args(args);
    if closed == "stdout" {
        command
            .stdout(std::process::Stdio::from(file))
            .stderr(std::process::Stdio::null());
    } else {
        command
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::from(file));
    }
    let mut child = command.spawn().unwrap();
    child.wait().unwrap().code().unwrap()
}

/// The nesting boundary on the object shape, both sides,
/// driven through the built binary, whose main thread carries
/// the disposal of the deep tree as the console process
/// carries it. The figures are the console script
/// measurements, 9997 open containers accepted and the 9998th
/// refused, on the render and review surfaces alike.
#[test]
fn the_binary_holds_the_nesting_boundary_on_the_object_shape() {
    let dir = temp_dir("binary-depth");
    let deepest = "{\"a\": ".repeat(9997) + "1" + &"}".repeat(9997);
    let deep_path = write(&dir, "deep.json", &deepest);
    let too_deep = "{\"a\": ".repeat(9998) + "1" + &"}".repeat(9998);
    let too_path = write(&dir, "too-deep.json", &too_deep);
    let deep = deep_path.as_bytes();
    let too = too_path.as_bytes();

    let (code, out, _err) = run_binary(&[b"render", deep]);
    assert_eq!(code, 2);
    assert_eq!(
        out,
        b"Review of unnamed review\nVerdict, invalid\nDefects, 2\n\
          Defect, diff_label is missing\nDefect, objections is missing\n"
            .to_vec()
    );

    let (code, out, _err) = run_binary(&[b"review", deep]);
    assert_eq!(code, 2);
    assert_eq!(
        out,
        b"Change set review\nVerdict, invalid\nDefects, 1\n\
          Defect, files is missing\n"
            .to_vec()
    );

    let (code, out, err) = run_binary(&[b"render", too]);
    assert_eq!(code, 1);
    assert!(out.is_empty());
    assert_eq!(
        err,
        b"maximum recursion depth exceeded while reading \
          the review file\n"
            .to_vec()
    );

    let (code, out, _err) = run_binary(&[b"review", too]);
    assert_eq!(code, 2);
    assert_eq!(
        out,
        b"Change set review\nVerdict, invalid\nDefects, 1\n\
          Defect, the file does not hold a JSON change set\n"
            .to_vec()
    );
}

#[test]
fn a_closed_stdout_pipe_exits_120() {
    let dir = temp_dir("closed-stdout");
    let path = write(&dir, "clean.diff", CLEAN);
    let code = run_binary_with_closed_stream(&["check", &path], "stdout");
    assert_eq!(code, 120);
}

#[test]
fn a_closed_stderr_pipe_on_the_usage_path_exits_120() {
    let code = run_binary_with_closed_stream(&[], "stderr");
    assert_eq!(code, 120);
}

#[test]
fn a_closed_stderr_pipe_with_nothing_to_write_keeps_the_code() {
    let dir = temp_dir("closed-stderr-idle");
    let path = write(&dir, "clean.diff", CLEAN);
    let code = run_binary_with_closed_stream(&["check", &path], "stderr");
    assert_eq!(code, 0);
}
