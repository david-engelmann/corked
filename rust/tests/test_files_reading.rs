//! Pins for CK-21 and CK-22, the files file and reading.
//!
//! The files pins build their collections by hand, so
//! each one fails under the negation of the rule it
//! names, the sections in first appearance order across
//! the collection, a file standing once over its
//! standings in two sets, a file holding no flags
//! standing no section, the whole or nothing refusal,
//! and the closing counts in their stated relations to
//! the sections and to the census, contested, and
//! pattern closings over the same collection. The census
//! pins hold the reading over the shipped census
//! collection against its hand checked rendering and
//! against the three closings it is bound to. The
//! fencing pins hold the rendering to the one line rule
//! at the library surface under FIX-CK-22. Every pin
//! runs offline on recorded and constructed entries, and
//! no pin makes a live call.

use std::path::{Path, PathBuf};

use stickler::census::census_of;
use stickler::cli::cli_main;
use stickler::contested::contested_reading_of;
use stickler::files::{files_reading_of, load_files, FileFlag, FileSection, FilesReading};
use stickler::json::parse;
use stickler::pattern::pattern_reading_of;
use stickler::render::{render_files, render_files_defects};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn examples() -> PathBuf {
    root().join("examples")
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "stickler-files-{}-{}-{}",
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

fn run(args: &[&str]) -> (i32, String, String) {
    let owned: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    let result = cli_main(&owned);
    (result.code, result.stdout, result.stderr)
}

fn run_files(dir: &Path, text: &str) -> (i32, String) {
    let path = write(dir, "files.json", text);
    let (code, out, _err) = run(&["files", &path]);
    (code, out)
}

/// Escape a string as JSON string content.
fn jstr(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// One change set entry as JSON text, verdicts appended raw.
fn entry_json(file: &str, diff: &str, verdicts: &str) -> String {
    let verdict_part = if verdicts.is_empty() {
        String::new()
    } else {
        format!(",\"verdicts\":{verdicts}")
    };
    format!(
        "{{\"file\":\"{}\",\"diff\":\"{}\"{}}}",
        jstr(file),
        jstr(diff),
        verdict_part
    )
}

/// One collection text over the given set entries, each a
/// pair of a label and a change set JSON text, in the form
/// CK-21 and CK-12 share.
fn collection_json(sets: &[(&str, String)]) -> String {
    let parts: Vec<String> = sets
        .iter()
        .map(|(label, change_set)| {
            format!(
                "{{\"label\":\"{}\",\"change_set\":{}}}",
                jstr(label),
                change_set
            )
        })
        .collect();
    format!("{{\"sets\":[{}]}}", parts.join(","))
}

fn change_set_json(entries: &[String]) -> String {
    format!("{{\"files\":[{}]}}", entries.join(","))
}

/// A diff holding two borderline lines and no judgements,
/// the near placeholder name at line 4 and the broad
/// handler at line 5.
const FLAGS_DIFF: &str =
    "--- a/app.py\n+++ b/app.py\n@@ -1,1 +1,3 @@\n+data = load_entries()\n+    except BaseException:\n";

/// A diff holding one borderline line and no judgements,
/// the near placeholder name at line 4.
const ONE_FLAG_DIFF: &str = "--- a/app.py\n+++ b/app.py\n@@ -1,1 +1,2 @@\n+data = load_entries()\n";

/// A diff holding one borderline line and no judgements,
/// the shared member default at line 4.
const DEFAULT_DIFF: &str =
    "--- a/app.py\n+++ b/app.py\n@@ -1,1 +1,2 @@\n+def plan(start, cache=([],)):\n";

/// A diff holding one judgement and no borderline lines.
const JUDGED_DIFF: &str = "--- a/app.py\n+++ b/app.py\n@@ -1,1 +1,2 @@\n+    print(total)\n";

/// A diff holding no borderline lines and no judgements.
const CLEAN_DIFF: &str = "--- a/app.py\n+++ b/app.py\n@@ -1,1 +1,2 @@\n+    logger.debug(total)\n";

/// Verdicts contesting the line 4 flag of FLAGS_DIFF, one
/// stickler and one clean under different models, with the
/// line 5 flag holding an abstention only.
const CONTESTED_VERDICTS: &str = "[
    {\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":0.9},
    {\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"openai/gpt-6-luna\",\"choice\":\"clean\",\"confidence\":0.9},
    {\"line\":5,\"pattern\":\"broad handler\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":0.3}
]";

#[test]
fn a_value_that_is_not_an_object_is_refused() {
    let value = parse("[]").unwrap();
    assert_eq!(
        load_files(&value),
        vec!["files must be a JSON object".to_string()]
    );
}

#[test]
fn a_missing_sets_field_is_refused() {
    let value = parse("{}").unwrap();
    assert_eq!(load_files(&value), vec!["sets is missing".to_string()]);
}

#[test]
fn a_sets_field_that_is_not_a_list_is_refused() {
    let value = parse("{\"sets\": {}}").unwrap();
    assert_eq!(load_files(&value), vec!["sets must be a list".to_string()]);
}

#[test]
fn an_entry_defect_names_its_position() {
    let value = parse("{\"sets\": [{\"change_set\": {\"files\": []}}]}").unwrap();
    assert_eq!(
        load_files(&value),
        vec!["sets[0].label is missing".to_string()]
    );
}

#[test]
fn a_label_holding_a_line_break_is_refused() {
    let text = collection_json(&[("evil\nlabel", change_set_json(&[]))]);
    let value = parse(&text).unwrap();
    assert_eq!(
        load_files(&value),
        vec!["sets[0].label holds a line break".to_string()]
    );
}

#[test]
fn a_member_defect_stands_in_the_census_path_form() {
    let value = parse("{\"sets\": [{\"label\": \"one\", \"change_set\": {}}]}").unwrap();
    assert_eq!(
        load_files(&value),
        vec!["sets[0].change_set.files is missing".to_string()]
    );
}

#[test]
fn the_empty_collection_renders_its_closings_alone() {
    let value = parse("{\"sets\": []}").unwrap();
    assert_eq!(load_files(&value), Vec::<String>::new());
    assert_eq!(
        render_files(&files_reading_of(&value)),
        "Files\n\
         \n\
         Sets read, 0\n\
         Files read, 0\n\
         Flags read, 0\n\
         Contested flags, 0\n"
    );
}

#[test]
fn a_file_in_two_sets_stands_once_with_the_flags_of_both_standings() {
    let text = collection_json(&[
        (
            "first set",
            change_set_json(&[entry_json("app.py", FLAGS_DIFF, "")]),
        ),
        (
            "second set",
            change_set_json(&[entry_json("app.py", DEFAULT_DIFF, "")]),
        ),
    ]);
    let value = parse(&text).unwrap();
    let reading = files_reading_of(&value);
    assert_eq!(reading.sets_read, 2);
    assert_eq!(reading.sections.len(), 1);
    let section = &reading.sections[0];
    assert_eq!(section.file, "app.py");
    assert_eq!(
        section.flags,
        vec![
            FileFlag {
                line: 4,
                pattern: "near placeholder name".to_string(),
                set_label: "first set".to_string(),
                contested: false,
            },
            FileFlag {
                line: 5,
                pattern: "broad handler".to_string(),
                set_label: "first set".to_string(),
                contested: false,
            },
            FileFlag {
                line: 4,
                pattern: "shared member default".to_string(),
                set_label: "second set".to_string(),
                contested: false,
            },
        ]
    );
    assert_eq!(
        render_files(&reading),
        "Files\n\
         \n\
         File, app.py\n\
         line 4, near placeholder name, set first set\n\
         line 5, broad handler, set first set\n\
         line 4, shared member default, set second set\n\
         Flags read, 3\n\
         Contested, 0\n\
         \n\
         Sets read, 2\n\
         Files read, 1\n\
         Flags read, 3\n\
         Contested flags, 0\n"
    );
}

#[test]
fn the_sections_stand_in_first_appearance_order_across_the_collection() {
    let text = collection_json(&[
        (
            "first set",
            change_set_json(&[
                entry_json("zeta.py", ONE_FLAG_DIFF, ""),
                entry_json("mid.py", ONE_FLAG_DIFF, ""),
            ]),
        ),
        (
            "second set",
            change_set_json(&[
                entry_json("alpha.py", ONE_FLAG_DIFF, ""),
                entry_json("zeta.py", DEFAULT_DIFF, ""),
            ]),
        ),
    ]);
    let value = parse(&text).unwrap();
    let reading = files_reading_of(&value);
    let names: Vec<&str> = reading
        .sections
        .iter()
        .map(|section| section.file.as_str())
        .collect();
    assert_eq!(names, vec!["zeta.py", "mid.py", "alpha.py"]);
    assert_eq!(reading.sections[0].flags.len(), 2);
    assert_eq!(reading.flags_read, 4);
}

#[test]
fn a_file_holding_no_flags_stands_no_section() {
    let text = collection_json(&[(
        "only set",
        change_set_json(&[
            entry_json("judged.py", JUDGED_DIFF, ""),
            entry_json("clean.py", CLEAN_DIFF, ""),
            entry_json("flagged.py", ONE_FLAG_DIFF, ""),
        ]),
    )]);
    let value = parse(&text).unwrap();
    let reading = files_reading_of(&value);
    assert_eq!(reading.sections.len(), 1);
    assert_eq!(reading.sections[0].file, "flagged.py");
    let rendered = render_files(&reading);
    assert!(!rendered.contains("judged.py"), "{rendered}");
    assert!(!rendered.contains("clean.py"), "{rendered}");
}

#[test]
fn a_file_in_three_sets_stands_once_at_its_first_appearance() {
    let text = collection_json(&[
        (
            "first set",
            change_set_json(&[entry_json("app.py", ONE_FLAG_DIFF, "")]),
        ),
        (
            "second set",
            change_set_json(&[entry_json("other.py", ONE_FLAG_DIFF, "")]),
        ),
        (
            "third set",
            change_set_json(&[entry_json("app.py", DEFAULT_DIFF, "")]),
        ),
    ]);
    let value = parse(&text).unwrap();
    let reading = files_reading_of(&value);
    assert_eq!(reading.sets_read, 3);
    assert_eq!(reading.sections.len(), 2);
    assert_eq!(reading.sections[0].file, "app.py");
    assert_eq!(reading.sections[0].flags.len(), 2);
    assert_eq!(reading.sections[1].file, "other.py");
}

#[test]
fn the_closings_stand_in_their_relations_over_a_hand_built_collection() {
    let text = collection_json(&[
        (
            "first set",
            change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]),
        ),
        (
            "second set",
            change_set_json(&[entry_json("app.py", DEFAULT_DIFF, "")]),
        ),
    ]);
    let value = parse(&text).unwrap();
    let reading = files_reading_of(&value);
    assert_eq!(reading.flags_read, 3);
    assert_eq!(reading.contested_flags, 1);
    assert_eq!(reading.sections[0].contested, 1);
    assert_eq!(reading.flags_read, census_of(&value).total.borderline);
    assert_eq!(reading.flags_read, pattern_reading_of(&value).flags_read);
    assert_eq!(
        reading.contested_flags,
        contested_reading_of(&value).contested_flags
    );
    let section_sum: usize = reading.sections.iter().map(|s| s.flags.len()).sum();
    assert_eq!(reading.flags_read, section_sum);
}

#[test]
fn the_reading_is_deterministic() {
    let text = collection_json(&[(
        "only set",
        change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]),
    )]);
    let value = parse(&text).unwrap();
    assert_eq!(
        render_files(&files_reading_of(&value)),
        render_files(&files_reading_of(&value))
    );
}

#[test]
fn a_defective_collection_renders_its_defects_and_no_reading() {
    let dir = temp_dir("defects");
    let (code, out) = run_files(&dir, "{\"sets\": [{\"change_set\": {\"files\": []}}]}");
    assert_eq!(code, 2);
    assert_eq!(
        out,
        "Files\n\
         Verdict, invalid\n\
         Defects, 1\n\
         Defect, sets[0].label is missing\n"
    );
}

#[test]
fn a_malformed_files_rendering_names_its_defects() {
    let rendered = render_files_defects(&["sets is missing".to_string()]);
    assert_eq!(
        rendered,
        "Files\nVerdict, invalid\nDefects, 1\nDefect, sets is missing\n"
    );
}

#[test]
fn files_on_a_missing_file_is_malformed_with_exit_2() {
    let dir = temp_dir("gone");
    let missing = dir.join("gone.json").to_string_lossy().into_owned();
    let (code, out, _err) = run(&["files", &missing]);
    assert_eq!(code, 2);
    assert!(out.contains("Defect, the files file could not be read\n"));
}

#[test]
fn files_on_a_non_json_file_is_malformed_with_exit_2() {
    let dir = temp_dir("nonjson");
    let (code, out) = run_files(&dir, "not json");
    assert_eq!(code, 2);
    assert!(out.contains("Defect, the file does not hold a JSON files collection\n"));
}

/// The census collection, read whole. The reading over
/// examples/census.json stands byte identical against
/// its hand checked rendering, the sections in first
/// appearance order, the round 2 files holding no flags
/// standing no section, and the closings in their
/// relations to the census, contested, and pattern
/// closings over the same collection.
#[test]
fn the_census_collection_reads_as_hand_checked() {
    let text = std::fs::read_to_string(examples().join("census.json")).unwrap();
    let value = parse(&text).unwrap();
    assert_eq!(load_files(&value), Vec::<String>::new());
    let reading = files_reading_of(&value);
    assert_eq!(
        render_files(&reading),
        "Files\n\
         \n\
         File, harvest.py\n\
         line 6, near placeholder name, set pilot set\n\
         line 7, near placeholder name, set pilot set\n\
         line 12, broad handler, set pilot set\n\
         line 18, broad handler, set pilot set\n\
         Flags read, 4\n\
         Contested, 0\n\
         \n\
         File, windows.py\n\
         line 6, shared member default, set pilot set\n\
         line 10, shared member default, set pilot set\n\
         line 14, shared member default, set pilot set\n\
         line 18, shared member default, set pilot set\n\
         Flags read, 4\n\
         Contested, 0\n\
         \n\
         File, sorting.py\n\
         line 6, near placeholder name, set pilot set\n\
         line 11, near placeholder name, set pilot set\n\
         line 12, near placeholder name, set pilot set\n\
         line 15, broad handler, set pilot set\n\
         Flags read, 4\n\
         Contested, 0\n\
         \n\
         File, ledger.py\n\
         line 6, near placeholder name, set constructed round 4 example\n\
         line 7, near placeholder name, set constructed round 4 example\n\
         Flags read, 2\n\
         Contested, 1\n\
         \n\
         File, cache.py\n\
         line 7, shared member default, set constructed round 4 example\n\
         line 10, broad handler, set constructed round 4 example\n\
         line 15, near placeholder name, set constructed round 4 example\n\
         line 19, near placeholder name, set constructed round 4 example\n\
         Flags read, 4\n\
         Contested, 1\n\
         \n\
         Sets read, 3\n\
         Files read, 5\n\
         Flags read, 18\n\
         Contested flags, 2\n"
    );
    assert_eq!(reading.flags_read, census_of(&value).total.borderline);
    assert_eq!(reading.flags_read, pattern_reading_of(&value).flags_read);
    assert_eq!(
        reading.contested_flags,
        contested_reading_of(&value).contested_flags
    );
    assert_eq!(reading.sections.len(), 5);
}

#[test]
fn the_example_matches_its_golden_through_the_command() {
    let path = examples().join("files.json");
    let (code, out, _err) = run(&["files", path.to_str().unwrap()]);
    assert_eq!(code, 0);
    let golden = std::fs::read_to_string(examples().join("files.txt")).unwrap();
    assert_eq!(out, golden);
}

#[test]
fn the_example_matches_its_golden_through_the_library() {
    let text = std::fs::read_to_string(examples().join("files.json")).unwrap();
    let value = parse(&text).unwrap();
    assert_eq!(load_files(&value), Vec::<String>::new());
    let golden = std::fs::read_to_string(examples().join("files.txt")).unwrap();
    assert_eq!(render_files(&files_reading_of(&value)), golden);
}

/// The example of CK-22. Its first set is the constructed
/// contested set of Round 4 embedded whole under its
/// standing label, and its second set is constructed for
/// this reading, the two sets sharing ledger.py standing
/// in both with differing flags over its two standings.
/// The golden stands hand checked against the census and
/// contested readings over the constructed set the
/// example embeds, whose standings the first section
/// states beside the second set's.
#[test]
fn the_example_sections_stand_hand_checked() {
    let golden = std::fs::read_to_string(examples().join("files.txt")).unwrap();
    let blocks: Vec<&str> = golden.split("\n\n").collect();
    assert_eq!(blocks.len(), 5, "{golden}");
    assert_eq!(blocks[0], "Files");
    assert_eq!(
        blocks[1],
        "File, ledger.py\n\
         line 6, near placeholder name, set constructed round 4 example\n\
         line 7, near placeholder name, set constructed round 4 example\n\
         line 6, shared member default, set constructed round 10 example\n\
         line 9, broad handler, set constructed round 10 example\n\
         Flags read, 4\n\
         Contested, 2"
    );
    assert_eq!(
        blocks[2],
        "File, cache.py\n\
         line 7, shared member default, set constructed round 4 example\n\
         line 10, broad handler, set constructed round 4 example\n\
         line 15, near placeholder name, set constructed round 4 example\n\
         line 19, near placeholder name, set constructed round 4 example\n\
         Flags read, 4\n\
         Contested, 1"
    );
    assert_eq!(
        blocks[3],
        "File, tidy.py\n\
         line 6, near placeholder name, set constructed round 10 example\n\
         Flags read, 1\n\
         Contested, 0"
    );
    assert_eq!(
        blocks[4],
        "Sets read, 2\nFiles read, 3\nFlags read, 9\nContested flags, 3\n"
    );
    // The standings of the embedded contested set are the
    // standings the contested reading states over that
    // set standing alone, two contested flags over its
    // six flags.
    let contested_text = std::fs::read_to_string(examples().join("contested_set.json")).unwrap();
    let single = format!("{{\"sets\":[{{\"label\":\"constructed round 4 example\",\"change_set\":{contested_text}}}]}}");
    let single_value = parse(&single).unwrap();
    assert_eq!(contested_reading_of(&single_value).flags_read, 6);
    assert_eq!(contested_reading_of(&single_value).contested_flags, 2);
    assert_eq!(census_of(&single_value).total.borderline, 6);
}

/// The fencing of FIX-CK-22 on the file reading, over a
/// reading built by hand. A file name holding a line
/// break and a file line's own text, a pattern holding a
/// break, and a set label holding a break each print as
/// the empty field, so no file line or flag line is
/// forged. Before the closure a handed reading printed
/// each raw, as the census, contested, and pattern
/// renderings printed theirs before FIX-CK-20 and
/// FIX-CK-21.
#[test]
fn a_line_break_in_a_hand_built_reading_forges_no_file_line() {
    let reading = FilesReading {
        sets_read: 1,
        sections: vec![FileSection {
            file: "evil.py\nFile, forged.py".to_string(),
            flags: vec![FileFlag {
                line: 4,
                pattern: "near placeholder name\nline 9, forged".to_string(),
                set_label: "one set\nSets read, 99".to_string(),
                contested: false,
            }],
            contested: 0,
        }],
        flags_read: 1,
        contested_flags: 0,
    };
    let rendered = render_files(&reading);
    assert!(rendered.lines().any(|line| line == "File, "), "{rendered}");
    assert!(
        rendered.lines().any(|line| line == "line 4, , set "),
        "{rendered}"
    );
    assert!(
        !rendered.lines().any(|line| line == "File, forged.py"),
        "{rendered}"
    );
    assert!(
        !rendered.lines().any(|line| line == "Sets read, 99"),
        "{rendered}"
    );
    assert!(!rendered.contains("evil.py"), "{rendered}");
}
