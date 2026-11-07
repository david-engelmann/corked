//! Pins for CK-11 and CK-12, the judging census.
//!
//! The census pins build their collections by hand, so each
//! one fails under the negation of the rule it names, the
//! absent verdict form, the standing orders of the shapes and
//! the patterns, the whole or nothing refusal, and the totals
//! read over the whole collection. The golden pin holds the
//! example census of CK-12 byte identical against its stored
//! rendering, and the cross pin holds the golden counts
//! against the three sets' own renderings. Every pin runs
//! offline on recorded and constructed entries, and no pin
//! makes a live call.

use std::path::{Path, PathBuf};

use stickler::census::{census_of, load_census, Census, CensusSet, Tally};
use stickler::changeset::review_changeset;
use stickler::cli::{cli_main, cli_main_units};
use stickler::json::{parse, JString};
use stickler::render::{render_census, render_set_review};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn examples() -> PathBuf {
    root().join("examples")
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "stickler-census-{}-{}-{}",
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

fn run_census(dir: &Path, text: &str) -> (i32, String) {
    let path = write(dir, "census.json", text);
    let (code, out, _err) = run(&["census", &path]);
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

/// One census text over the given set entries, each a pair of
/// a label and a change set JSON text.
fn census_json(sets: &[(&str, String)]) -> String {
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

const JUDGED_DIFF: &str =
    "--- a/app.py\n+++ b/app.py\n@@ -1,1 +1,3 @@\n+    print(total)\n+    except:\n+data = load_entries()\n";

const FLAGS_DIFF: &str =
    "--- a/app.py\n+++ b/app.py\n@@ -1,1 +1,3 @@\n+data = load_entries()\n+    except BaseException:\n";

#[test]
fn the_example_census_matches_its_golden_through_the_command() {
    let path = examples().join("census.json");
    let (code, out, _err) = run(&["census", path.to_str().unwrap()]);
    assert_eq!(code, 0);
    let golden = std::fs::read_to_string(examples().join("census.txt")).unwrap();
    assert_eq!(out, golden);
}

#[test]
fn the_example_census_matches_its_golden_through_the_library() {
    let text = std::fs::read_to_string(examples().join("census.json")).unwrap();
    let value = parse(&text).unwrap();
    assert_eq!(load_census(&value), Vec::<String>::new());
    let golden = std::fs::read_to_string(examples().join("census.txt")).unwrap();
    assert_eq!(render_census(&census_of(&value)), golden);
}

/// Read one count line of a set review summary or a census
/// block, in the form the line names.
fn count_of(lines: &[&str], head: &str) -> Option<String> {
    lines
        .iter()
        .find(|line| line.starts_with(head))
        .map(|line| line.to_string())
}

/// The summary lines of one stored set rendering, after its
/// Summary head.
fn summary_lines(path: &Path) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap();
    let tail = text.split("\nSummary\n").nth(1).expect("a summary");
    tail.lines().map(|line| line.to_string()).collect()
}

#[test]
fn the_example_counts_match_the_three_set_renderings() {
    let members = [
        ("round 2 set", "change_set_review.txt"),
        ("pilot set", "borderline_review.txt"),
        ("constructed round 4 example", "contested_review.txt"),
    ];
    let golden = std::fs::read_to_string(examples().join("census.txt")).unwrap();
    let blocks: Vec<&str> = golden.split("\n\n").collect();
    assert_eq!(blocks.len(), 5, "head, three sets, and the total");
    let total_block = blocks[4];
    let total_lines: Vec<&str> = total_block.lines().collect();
    let mut summed_judgements = 0i64;
    for (index, (label, rendering)) in members.iter().enumerate() {
        let summary = summary_lines(&examples().join(rendering));
        let summary_refs: Vec<&str> = summary.iter().map(|line| line.as_str()).collect();
        let block = blocks[index + 1];
        let block_lines: Vec<&str> = block.lines().collect();
        assert_eq!(block_lines[0], format!("Set, {label}"));
        assert_eq!(
            count_of(&block_lines, "Judgements,"),
            count_of(&summary_refs, "Judgements,"),
            "the {label} judgement count is its own rendering's"
        );
        for shape in [
            "bare except",
            "print call",
            "mutable default",
            "placeholder name",
            "call default",
            "class attribute",
        ] {
            let head = format!("Shape, {shape},");
            let in_summary =
                count_of(&summary_refs, &head).unwrap_or_else(|| format!("Shape, {shape}, 0"));
            assert_eq!(
                count_of(&block_lines, &head),
                Some(in_summary),
                "the {label} count of {shape} is its own rendering's"
            );
        }
        assert_eq!(
            count_of(&block_lines, "Borderline,"),
            count_of(&summary_refs, "Borderline,").or_else(|| Some("Borderline, 0".to_string())),
            "the {label} borderline count is its own rendering's"
        );
        match count_of(&summary_refs, "Verdict lines,") {
            Some(verdict_line) => {
                assert_eq!(count_of(&block_lines, "Verdict lines,"), Some(verdict_line));
                assert_eq!(
                    count_of(&block_lines, "Unjudged,"),
                    count_of(&summary_refs, "Unjudged,")
                );
                assert_eq!(
                    count_of(&block_lines, "Contested,"),
                    count_of(&summary_refs, "Contested,")
                );
            }
            None => {
                assert!(
                    block_lines.contains(&"Verdicts, none recorded"),
                    "the {label} block states the absent form"
                );
                assert_eq!(count_of(&block_lines, "Verdict lines,"), None);
            }
        }
        let judgements: i64 = count_of(&summary_refs, "Judgements,")
            .expect("a judgement count")
            .trim_start_matches("Judgements, ")
            .parse()
            .unwrap();
        summed_judgements += judgements;
    }
    assert_eq!(
        count_of(&total_lines, "Judgements,"),
        Some(format!("Judgements, {summed_judgements}")),
        "the total judgement count is the sum of the three renderings"
    );
}

#[test]
fn a_single_set_census_stands_line_for_line() {
    let dir = temp_dir("single");
    let set = change_set_json(&[entry_json("app.py", JUDGED_DIFF, "")]);
    let (code, out) = run_census(&dir, &census_json(&[("only set", set)]));
    assert_eq!(code, 0);
    let block = "\
Files, 1
Judgements, 2
Shape, bare except, 1
Shape, print call, 1
Shape, mutable default, 0
Shape, placeholder name, 0
Shape, call default, 0
Shape, class attribute, 0
Borderline, 1
Pattern, near placeholder name, 1
Pattern, broad handler, 0
Pattern, shared member default, 0
Verdicts, none recorded
";
    let expected = format!("Census\nSets, 1\n\nSet, only set\n{block}\nTotal\nSets, 1\n{block}");
    assert_eq!(out, expected);
}

#[test]
fn an_empty_collection_renders_zero_counts_and_the_absent_form() {
    let dir = temp_dir("empty");
    let (code, out) = run_census(&dir, "{\"sets\":[]}");
    assert_eq!(code, 0);
    let expected = "\
Census
Sets, 0

Total
Sets, 0
Files, 0
Judgements, 0
Shape, bare except, 0
Shape, print call, 0
Shape, mutable default, 0
Shape, placeholder name, 0
Shape, call default, 0
Shape, class attribute, 0
Borderline, 0
Pattern, near placeholder name, 0
Pattern, broad handler, 0
Pattern, shared member default, 0
Verdicts, none recorded
";
    assert_eq!(out, expected);
}

#[test]
fn a_set_holding_no_verdicts_contributes_its_flags_to_the_total_as_unjudged() {
    let dir = temp_dir("absent-total");
    let silent = change_set_json(&[entry_json("app.py", FLAGS_DIFF, "")]);
    let verdicts = "[{\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":0.9}]";
    let judged = change_set_json(&[entry_json("app.py", FLAGS_DIFF, verdicts)]);
    let (code, out) = run_census(
        &dir,
        &census_json(&[("silent set", silent), ("judged set", judged)]),
    );
    assert_eq!(code, 0);
    let blocks: Vec<&str> = out.split("\n\n").collect();
    let silent_block = blocks[1];
    assert!(silent_block.contains("Verdicts, none recorded"));
    assert!(!silent_block.contains("Verdict lines"));
    assert!(!silent_block.contains("Unjudged"));
    let judged_block = blocks[2];
    assert!(judged_block.contains("Verdict lines, stickler 1, clean 0"));
    assert!(judged_block.contains("Unjudged, 1"));
    let total_block = blocks[3];
    assert!(
        total_block.contains("Unjudged, 3"),
        "both flags of the silent set and the unjudged flag of the judged set stand in the total\n{total_block}"
    );
    assert!(total_block.contains("Verdict lines, stickler 1, clean 0"));
    assert!(total_block.contains("Borderline, 4"));
}

#[test]
fn the_census_reads_contested_and_abstained_lines_through_the_standing_outcomes() {
    let dir = temp_dir("outcomes");
    let verdicts = "[
        {\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":0.9},
        {\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"openai/gpt-6-luna\",\"choice\":\"clean\",\"confidence\":0.9},
        {\"line\":5,\"pattern\":\"broad handler\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":0.3}
    ]";
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, verdicts)]);
    let (code, out) = run_census(&dir, &census_json(&[("outcome set", set)]));
    assert_eq!(code, 0);
    let blocks: Vec<&str> = out.split("\n\n").collect();
    let block = blocks[1];
    assert!(block.contains("Borderline, 2"), "{block}");
    assert!(
        block.contains("Verdict lines, stickler 1, clean 0"),
        "{block}"
    );
    assert!(block.contains("Unjudged, 1"), "{block}");
    assert!(block.contains("Contested, 1"), "{block}");
}

#[test]
fn the_pattern_and_shape_lines_stand_in_their_standing_order() {
    let dir = temp_dir("order");
    // The diff flags the shared member default before the near
    // placeholder name in line order, and raises the class
    // attribute and mutable default judgements after the print
    // call in the standing order of the shapes.
    let diff = "--- a/app.py\n+++ b/app.py\n@@ -1,1 +1,4 @@\n+def plan(start, cache=([],)):\n+data = load_entries()\n+def render(total, history=[]):\n+    print(total)\n";
    let set = change_set_json(&[entry_json("app.py", diff, "")]);
    let (code, out) = run_census(&dir, &census_json(&[("order set", set)]));
    assert_eq!(code, 0);
    let pattern_near = out.find("Pattern, near placeholder name").unwrap();
    let pattern_shared = out.find("Pattern, shared member default").unwrap();
    assert!(
        pattern_near < pattern_shared,
        "the pattern lines stand in the standing order, not the line order"
    );
    let shape_print = out.find("Shape, print call").unwrap();
    let shape_mutable = out.find("Shape, mutable default").unwrap();
    assert!(
        shape_print < shape_mutable,
        "the shape lines stand in the standing order, not the line order"
    );
}

#[test]
fn one_malformed_member_refuses_the_whole_census() {
    let dir = temp_dir("whole");
    let good = change_set_json(&[entry_json("app.py", JUDGED_DIFF, "")]);
    let bad_verdicts =
        "[{\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":2}]";
    let bad = change_set_json(&[entry_json("app.py", FLAGS_DIFF, bad_verdicts)]);
    let (code, out) = run_census(&dir, &census_json(&[("good set", good), ("bad set", bad)]));
    assert_eq!(code, 2);
    assert!(
        out.starts_with("Census\nVerdict, invalid\nDefects, 1\n"),
        "{out}"
    );
    assert!(
        out.contains(
            "Defect, sets[1].change_set.files[0].verdicts[0].confidence must be from 0 to 1"
        ),
        "{out}"
    );
    assert!(!out.contains("Set, good set"), "no partial census prints");
    assert!(!out.contains("Total"));
}

#[test]
fn a_huge_integer_confidence_in_a_member_is_refused_by_range() {
    let dir = temp_dir("huge");
    let huge = "1".repeat(400);
    let verdicts = format!(
        "[{{\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":{huge}}}]"
    );
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, &verdicts)]);
    let (code, out) = run_census(&dir, &census_json(&[("huge set", set)]));
    assert_eq!(code, 2);
    assert!(
        out.contains(
            "Defect, sets[0].change_set.files[0].verdicts[0].confidence must be from 0 to 1"
        ),
        "{out}"
    );
}

#[test]
fn the_loader_names_the_top_level_defects() {
    assert_eq!(
        load_census(&parse("5").unwrap()),
        vec!["census must be a JSON object".to_string()]
    );
    assert_eq!(
        load_census(&parse("{}").unwrap()),
        vec!["sets is missing".to_string()]
    );
    assert_eq!(
        load_census(&parse("{\"sets\":{}}").unwrap()),
        vec!["sets must be a list".to_string()]
    );
    assert_eq!(
        load_census(&parse("{\"sets\":[5]}").unwrap()),
        vec!["sets[0] must be an object".to_string()]
    );
    assert_eq!(
        load_census(&parse("{\"sets\":[{\"change_set\":{\"files\":[]}}]}").unwrap()),
        vec!["sets[0].label is missing".to_string()]
    );
    assert_eq!(
        load_census(&parse("{\"sets\":[{\"label\":\"x\"}]}").unwrap()),
        vec!["sets[0].change_set is missing".to_string()]
    );
    assert_eq!(
        load_census(&parse("{\"sets\":[{\"label\":\"x\",\"change_set\":5}]}").unwrap()),
        vec!["sets[0].change_set must be a JSON object".to_string()]
    );
    assert_eq!(
        load_census(&parse("{\"sets\":[{\"label\":\"x\",\"change_set\":{}}]}").unwrap()),
        vec!["sets[0].change_set.files is missing".to_string()]
    );
    assert_eq!(
        load_census(
            &parse("{\"sets\":[{\"label\":\"  \",\"change_set\":{\"files\":[]}}]}").unwrap()
        ),
        vec!["sets[0].label must be a non empty string".to_string()]
    );
}

#[test]
fn a_label_holding_a_line_break_is_refused_and_never_printed() {
    let dir = temp_dir("label-break");
    let set = change_set_json(&[entry_json("app.py", JUDGED_DIFF, "")]);
    let text =
        format!("{{\"sets\":[{{\"label\":\"forged\\nSet, forged\",\"change_set\":{set}}}]}}");
    let (code, out) = run_census(&dir, &text);
    assert_eq!(code, 2);
    assert!(
        out.contains("Defect, sets[0].label holds a line break"),
        "{out}"
    );
    assert!(
        !out.lines().any(|line| line == "Set, forged"),
        "the refused label forges no printed line"
    );
}

#[test]
fn a_label_holding_an_unpaired_surrogate_is_refused() {
    let dir = temp_dir("label-surrogate");
    let set = change_set_json(&[entry_json("app.py", JUDGED_DIFF, "")]);
    let text = format!("{{\"sets\":[{{\"label\":\"bad \\ud800 label\",\"change_set\":{set}}}]}}");
    let (code, out) = run_census(&dir, &text);
    assert_eq!(code, 2);
    assert!(
        out.contains("Defect, sets[0].label holds an unpaired surrogate"),
        "{out}"
    );
}

#[test]
fn the_command_refuses_unreadable_and_foreign_files() {
    let dir = temp_dir("unreadable");
    let missing = dir.join("nowhere.json").to_string_lossy().into_owned();
    let (code, out, _err) = run(&["census", &missing]);
    assert_eq!(code, 2);
    assert_eq!(
        out,
        "Census\nVerdict, invalid\nDefects, 1\nDefect, the census file could not be read\n"
    );
    let foreign = write(&dir, "foreign.json", "this is not json");
    let (code, out, _err) = run(&["census", &foreign]);
    assert_eq!(code, 2);
    assert_eq!(
        out,
        "Census\nVerdict, invalid\nDefects, 1\nDefect, the file does not hold a JSON census\n"
    );
    let deep = write(&dir, "deep.json", &"[".repeat(10050));
    let (code, out, _err) = run(&["census", &deep]);
    assert_eq!(code, 2);
    assert!(
        out.contains("Defect, the file does not hold a JSON census"),
        "nesting past the boundary is a refusal, never a fall\n{out}"
    );
}

#[test]
fn a_valid_census_exits_zero_even_when_judgements_stand() {
    let dir = temp_dir("exit-zero");
    let set = change_set_json(&[entry_json("app.py", JUDGED_DIFF, "")]);
    let (code, out) = run_census(&dir, &census_json(&[("judged set", set)]));
    assert_eq!(
        code, 0,
        "the census reports counts and judges nothing\n{out}"
    );
    assert!(out.contains("Judgements, 2"));
}

#[test]
fn duplicate_labels_and_unread_fields_stand() {
    let dir = temp_dir("duplicates");
    let set = change_set_json(&[entry_json("app.py", JUDGED_DIFF, "")]);
    let text = format!(
        "{{\"sets\":[{{\"label\":\"same\",\"change_set\":{set},\"note\":\"unread\"}},{{\"label\":\"same\",\"change_set\":{set}}}]}}"
    );
    let (code, out) = run_census(&dir, &text);
    assert_eq!(code, 0);
    assert_eq!(out.matches("Set, same").count(), 2);
    assert!(out.contains("Sets, 2"));
}

#[test]
fn the_census_command_reads_a_raw_byte_file_name() {
    use std::os::unix::ffi::OsStringExt;
    let dir = temp_dir("raw-byte");
    let set = change_set_json(&[entry_json("app.py", JUDGED_DIFF, "")]);
    let text = census_json(&[("byte set", set)]);
    let mut bytes = dir.as_os_str().as_encoded_bytes().to_vec();
    bytes.push(b'/');
    bytes.extend_from_slice(b"bad\xffname.json");
    let path = std::ffi::OsString::from_vec(bytes.clone());
    std::fs::write(path, text).unwrap();
    let owned: Vec<JString> = [b"census".as_slice(), bytes.as_slice()]
        .iter()
        .map(|arg| JString::from_surrogateescape(arg))
        .collect();
    let result = cli_main_units(&owned);
    assert_eq!(result.code, 0);
    let out = String::from_utf8(result.stdout).unwrap();
    assert!(out.contains("Set, byte set"), "{out}");
    assert!(out.contains("Judgements, 2"), "{out}");
}

#[test]
fn the_census_command_names_a_file_that_is_not_utf8_text() {
    let dir = temp_dir("not-utf8");
    let path = dir.join("census.json");
    std::fs::write(&path, [0xff, 0xfe, 0xfd]).unwrap();
    let (code, out, _err) = run(&["census", path.to_str().unwrap()]);
    assert_eq!(code, 2);
    assert_eq!(
        out,
        "Census\nVerdict, invalid\nDefects, 1\nDefect, the census file could not be read\n"
    );
}

#[test]
fn the_census_of_a_member_matches_its_set_review_counts() {
    // The census holds no second logic, so the tally of one
    // member equals the counts the set review of that member
    // renders, read from the rendering itself.
    let set_text = format!("{{\"files\":[{}]}}", entry_json("app.py", FLAGS_DIFF, ""));
    let value = parse(&set_text).unwrap();
    let rendered = render_set_review(&review_changeset(&value));
    assert!(rendered.contains("Borderline, 2"));
    let census_text = census_json(&[("member", set_text)]);
    let census_value = parse(&census_text).unwrap();
    let census = census_of(&census_value);
    assert_eq!(census.sets.len(), 1);
    assert_eq!(census.sets[0].tally.borderline, 2);
    assert_eq!(census.sets[0].tally.patterns, [1, 1, 0]);
    assert_eq!(census.total.borderline, 2);
}

/// The fencing of FIX-CK-21 on the census label. A set
/// label holding a line break and a head line's own text
/// is refused by the loader, and read through the library
/// the rendering prints the label as the empty field, so
/// the forged head line never stands beside the true one.
/// Before the closure the rendering printed the label raw
/// and the forged line stood.
#[test]
fn a_line_break_label_mints_no_head_line_through_the_library() {
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, "")]);
    let text = census_json(&[("evil\nSets, 99", set)]);
    let value = parse(&text).unwrap();
    assert!(!load_census(&value).is_empty());
    let out = render_census(&census_of(&value));
    assert!(out.lines().any(|line| line == "Set, "), "{out}");
    assert!(!out.lines().any(|line| line == "Sets, 99"), "{out}");
    let heads: Vec<&str> = out
        .lines()
        .filter(|line| line.starts_with("Sets, "))
        .collect();
    assert_eq!(heads, ["Sets, 1", "Sets, 1"], "{out}");
}

/// The handed total of FIX-CK-21. CK-11 states the total
/// block as every count of the set blocks summed over the
/// collection, and a census built by hand prints its
/// total block in that relation, so the total it was
/// handed never stands. Before the closure the rendering
/// printed the handed total whole.
#[test]
fn a_hand_built_census_prints_the_total_of_its_set_blocks() {
    let census = Census {
        sets: vec![
            CensusSet {
                label: "one".to_string(),
                tally: Tally {
                    files: 1,
                    judgements: 2,
                    shapes: [1, 0, 0, 0, 0, 0],
                    borderline: 3,
                    patterns: [1, 1, 1],
                    verdicts_stand: true,
                    stickler: 1,
                    clean: 1,
                    unjudged: 1,
                    contested: 1,
                },
            },
            CensusSet {
                label: "two".to_string(),
                tally: Tally {
                    files: 2,
                    judgements: 0,
                    shapes: [0, 1, 0, 0, 0, 0],
                    borderline: 1,
                    patterns: [0, 0, 1],
                    verdicts_stand: false,
                    stickler: 0,
                    clean: 0,
                    unjudged: 1,
                    contested: 0,
                },
            },
        ],
        total: Tally {
            files: 555,
            judgements: 555,
            shapes: [555; 6],
            borderline: 555,
            patterns: [555; 3],
            verdicts_stand: false,
            stickler: 555,
            clean: 555,
            unjudged: 555,
            contested: 555,
        },
    };
    let out = render_census(&census);
    assert!(!out.contains("555"), "{out}");
    let total = out
        .split("\nTotal\n")
        .nth(1)
        .expect("the rendering holds a total block");
    assert!(
        total.starts_with("Sets, 2\nFiles, 3\nJudgements, 2\n"),
        "{out}"
    );
    assert!(total.contains("Shape, bare except, 1\n"), "{out}");
    assert!(total.contains("Shape, print call, 1\n"), "{out}");
    assert!(total.contains("Borderline, 4\n"), "{out}");
    assert!(
        total.contains("Pattern, near placeholder name, 1\n"),
        "{out}"
    );
    assert!(total.contains("Pattern, broad handler, 1\n"), "{out}");
    assert!(
        total.contains("Pattern, shared member default, 2\n"),
        "{out}"
    );
    // The judged lines stand in the total because a
    // verdict stands in one of its set blocks, though the
    // handed total said none did.
    assert!(
        total.contains("Verdict lines, stickler 1, clean 1\nUnjudged, 2\nContested, 1\n"),
        "{out}"
    );
}

/// The other direction of the same derivation. Where no
/// set block holds a verdict, the total block states the
/// none recorded line even when the handed total claims
/// judged counts.
#[test]
fn a_hand_built_census_derives_the_none_recorded_form_of_its_total() {
    let census = Census {
        sets: vec![CensusSet {
            label: "one".to_string(),
            tally: Tally {
                files: 1,
                judgements: 0,
                shapes: [0; 6],
                borderline: 2,
                patterns: [1, 1, 0],
                verdicts_stand: false,
                stickler: 0,
                clean: 0,
                unjudged: 2,
                contested: 0,
            },
        }],
        total: Tally {
            files: 1,
            judgements: 0,
            shapes: [0; 6],
            borderline: 2,
            patterns: [1, 1, 0],
            verdicts_stand: true,
            stickler: 2,
            clean: 0,
            unjudged: 0,
            contested: 0,
        },
    };
    let out = render_census(&census);
    let total = out
        .split("\nTotal\n")
        .nth(1)
        .expect("the rendering holds a total block");
    assert!(total.contains("Verdicts, none recorded\n"), "{out}");
    assert!(!total.contains("Verdict lines"), "{out}");
}
