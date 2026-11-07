//! Pins for CK-19 and CK-20, the pattern file and reading.
//!
//! The pattern pins build their collections by hand, so
//! each one fails under the negation of the rule it names,
//! the pattern sections in the order CK-7 states, the
//! collection order of the flags inside one section, a
//! flag standing in exactly one section, the no flagged
//! lines line, the whole or nothing refusal, and the
//! closing counts in their stated relations to the
//! sections and to the census and the contested reading
//! over the same collection. The golden pins hold the
//! example of CK-20 byte identical against its stored
//! rendering, and the hand checked pins hold its sections
//! against the census golden and the contested golden.
//! The fencing pins hold the rendering to the one line
//! rule at the library surface under FIX-CK-21. Every pin
//! runs offline on recorded and constructed entries, and
//! no pin makes a live call.

use std::path::{Path, PathBuf};

use stickler::census::{census_of, load_census};
use stickler::cli::cli_main;
use stickler::contested::contested_reading_of;
use stickler::json::parse;
use stickler::pattern::{
    load_pattern, pattern_reading_of, PatternFlag, PatternReading, PatternSection,
};
use stickler::render::{render_census, render_pattern};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn examples() -> PathBuf {
    root().join("examples")
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "stickler-pattern-{}-{}-{}",
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

fn run_pattern(dir: &Path, text: &str) -> (i32, String) {
    let path = write(dir, "pattern.json", text);
    let (code, out, _err) = run(&["pattern", &path]);
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
/// CK-19 and CK-12 share.
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

/// Verdicts contesting the line 4 flag of FLAGS_DIFF, one
/// stickler and one clean under different models, with the
/// line 5 flag holding an abstention only.
const CONTESTED_VERDICTS: &str = "[
    {\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":0.9},
    {\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"openai/gpt-6-luna\",\"choice\":\"clean\",\"confidence\":0.9},
    {\"line\":5,\"pattern\":\"broad handler\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":0.3}
]";

/// Verdicts contesting nothing over FLAGS_DIFF, both labels
/// standing under agreement or under one flag alone.
const AGREEING_VERDICTS: &str = "[
    {\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":0.9},
    {\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"openai/gpt-6-luna\",\"choice\":\"stickler\",\"confidence\":0.8},
    {\"line\":5,\"pattern\":\"broad handler\",\"model\":\"openai/gpt-6-luna\",\"choice\":\"clean\",\"confidence\":0.9}
]";

#[test]
fn the_example_matches_its_golden_through_the_command() {
    let path = examples().join("pattern.json");
    let (code, out, _err) = run(&["pattern", path.to_str().unwrap()]);
    assert_eq!(code, 0);
    let golden = std::fs::read_to_string(examples().join("pattern.txt")).unwrap();
    assert_eq!(out, golden);
}

#[test]
fn the_example_matches_its_golden_through_the_library() {
    let text = std::fs::read_to_string(examples().join("pattern.json")).unwrap();
    let value = parse(&text).unwrap();
    assert_eq!(load_pattern(&value), Vec::<String>::new());
    let golden = std::fs::read_to_string(examples().join("pattern.txt")).unwrap();
    assert_eq!(render_pattern(&pattern_reading_of(&value)), golden);
}

#[test]
fn the_example_holds_the_census_collection() {
    let pattern_text = std::fs::read_to_string(examples().join("pattern.json")).unwrap();
    let census_text = std::fs::read_to_string(examples().join("census.json")).unwrap();
    let pattern_value = parse(&pattern_text).unwrap();
    let census_value = parse(&census_text).unwrap();
    assert_eq!(load_census(&pattern_value), Vec::<String>::new());
    assert_eq!(
        render_census(&census_of(&pattern_value)),
        render_census(&census_of(&census_value)),
        "the pattern example reads under the census as the census example itself"
    );
    assert_eq!(
        render_pattern(&pattern_reading_of(&pattern_value)),
        render_pattern(&pattern_reading_of(&census_value)),
        "the pattern reading of the pattern example is the pattern reading of the census example"
    );
}

#[test]
fn the_example_sections_stand_hand_checked() {
    let golden = std::fs::read_to_string(examples().join("pattern.txt")).unwrap();
    let blocks: Vec<&str> = golden.split("\n\n").collect();
    assert_eq!(blocks.len(), 5, "{golden}");
    assert_eq!(blocks[0], "Pattern");
    assert_eq!(
        blocks[1],
        "Pattern, near placeholder name\n\
         harvest.py, line 6, set pilot set\n\
         harvest.py, line 7, set pilot set\n\
         sorting.py, line 6, set pilot set\n\
         sorting.py, line 11, set pilot set\n\
         sorting.py, line 12, set pilot set\n\
         ledger.py, line 6, set constructed round 4 example\n\
         ledger.py, line 7, set constructed round 4 example\n\
         cache.py, line 15, set constructed round 4 example\n\
         cache.py, line 19, set constructed round 4 example\n\
         Flags read, 9\n\
         Contested, 2"
    );
    assert_eq!(
        blocks[2],
        "Pattern, broad handler\n\
         harvest.py, line 12, set pilot set\n\
         harvest.py, line 18, set pilot set\n\
         sorting.py, line 15, set pilot set\n\
         cache.py, line 10, set constructed round 4 example\n\
         Flags read, 4\n\
         Contested, 0"
    );
    assert_eq!(
        blocks[3],
        "Pattern, shared member default\n\
         windows.py, line 6, set pilot set\n\
         windows.py, line 10, set pilot set\n\
         windows.py, line 14, set pilot set\n\
         windows.py, line 18, set pilot set\n\
         cache.py, line 7, set constructed round 4 example\n\
         Flags read, 5\n\
         Contested, 0"
    );
    assert_eq!(
        blocks[4],
        "Sets read, 3\nFlags read, 18\nContested flags, 2\n"
    );
    // The section counts are the counts of the census
    // golden over the same collection, the pattern counts
    // of its total block and its total flagged count.
    let census = std::fs::read_to_string(examples().join("census.txt")).unwrap();
    let total = census
        .split("\nTotal\n")
        .nth(1)
        .expect("the census golden holds a total block");
    for (block, pattern) in blocks[1..4].iter().zip([
        "near placeholder name",
        "broad handler",
        "shared member default",
    ]) {
        let census_line = total
            .lines()
            .find(|line| line.starts_with(&format!("Pattern, {pattern}, ")))
            .expect("the census total names the pattern");
        let census_count: usize = census_line
            .trim_start_matches(&format!("Pattern, {pattern}, "))
            .parse()
            .unwrap();
        let section_count: usize = block
            .lines()
            .rev()
            .find(|line| line.starts_with("Flags read, "))
            .map(|line| line.trim_start_matches("Flags read, ").parse().unwrap())
            .unwrap();
        assert_eq!(section_count, census_count, "{pattern}");
    }
    assert!(total.contains("Borderline, 18\n"));
    // The contested counts are the counts of the contested
    // golden over the same collection, read per pattern
    // from the flags it names.
    let contested = std::fs::read_to_string(examples().join("contested.txt")).unwrap();
    for (block, pattern) in blocks[1..4].iter().zip([
        "near placeholder name",
        "broad handler",
        "shared member default",
    ]) {
        let named = contested
            .lines()
            .filter(|line| line.ends_with(&format!(", {pattern}")))
            .count();
        let section_contested: usize = block
            .lines()
            .rev()
            .find(|line| line.starts_with("Contested, "))
            .map(|line| line.trim_start_matches("Contested, ").parse().unwrap())
            .unwrap();
        assert_eq!(section_contested, named, "{pattern}");
    }
    assert!(contested.contains("Flags read, 18\n"));
    assert!(contested.contains("Contested flags, 2\n"));
}

#[test]
fn a_single_set_collection_stands_line_for_line() {
    let dir = temp_dir("single");
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]);
    let (code, out) = run_pattern(&dir, &collection_json(&[("only set", set)]));
    assert_eq!(code, 0);
    let expected = "\
Pattern

Pattern, near placeholder name
app.py, line 4, set only set
Flags read, 1
Contested, 1

Pattern, broad handler
app.py, line 5, set only set
Flags read, 1
Contested, 0

Pattern, shared member default
No flagged lines
Flags read, 0
Contested, 0

Sets read, 1
Flags read, 2
Contested flags, 1
";
    assert_eq!(out, expected);
}

#[test]
fn an_empty_collection_renders_every_pattern_with_zero_counts() {
    let dir = temp_dir("empty");
    let (code, out) = run_pattern(&dir, "{\"sets\":[]}");
    assert_eq!(code, 0);
    let expected = "\
Pattern

Pattern, near placeholder name
No flagged lines
Flags read, 0
Contested, 0

Pattern, broad handler
No flagged lines
Flags read, 0
Contested, 0

Pattern, shared member default
No flagged lines
Flags read, 0
Contested, 0

Sets read, 0
Flags read, 0
Contested flags, 0
";
    assert_eq!(out, expected);
}

#[test]
fn a_set_with_no_flags_leaves_every_pattern_absent() {
    let dir = temp_dir("noflags");
    let set = change_set_json(&[entry_json("app.py", JUDGED_DIFF, "")]);
    let (code, out) = run_pattern(&dir, &collection_json(&[("judged set", set)]));
    assert_eq!(code, 0);
    assert_eq!(out.matches("No flagged lines").count(), 3);
    assert!(out.contains("Sets read, 1\n"));
    assert!(out.contains("Contested flags, 0\n"));
}

#[test]
fn a_flag_stands_in_exactly_one_pattern_section() {
    let dir = temp_dir("onesection");
    // Two sets share one file name and one line number
    // under different patterns, the near placeholder name
    // at line 4 of the first set and the shared member
    // default at line 4 of the second.
    let first = change_set_json(&[entry_json("app.py", FLAGS_DIFF, "")]);
    let second = change_set_json(&[entry_json("app.py", DEFAULT_DIFF, "")]);
    let (code, out) = run_pattern(
        &dir,
        &collection_json(&[("first set", first), ("second set", second)]),
    );
    assert_eq!(code, 0);
    let blocks: Vec<&str> = out.split("\n\n").collect();
    assert!(
        blocks[1].contains("app.py, line 4, set first set\n"),
        "{out}"
    );
    assert!(!blocks[1].contains("set second set"), "{out}");
    assert!(
        blocks[2].contains("app.py, line 5, set first set\n"),
        "{out}"
    );
    assert!(
        blocks[3].contains("app.py, line 4, set second set\n"),
        "{out}"
    );
    assert!(!blocks[3].contains("set first set"), "{out}");
    let flag_lines = out.lines().filter(|line| line.contains(", line ")).count();
    assert_eq!(flag_lines, 3, "{out}");
}

#[test]
fn the_sections_follow_collection_order_across_sets() {
    let dir = temp_dir("order");
    // The file names stand against alphabetical order on
    // purpose, so a section grouped or sorted by flag
    // identity rather than read in collection order fails.
    let first = change_set_json(&[
        entry_json("z.py", ONE_FLAG_DIFF, ""),
        entry_json("m.py", ONE_FLAG_DIFF, ""),
    ]);
    let second = change_set_json(&[entry_json("a.py", ONE_FLAG_DIFF, "")]);
    let (code, out) = run_pattern(
        &dir,
        &collection_json(&[("first set", first), ("second set", second)]),
    );
    assert_eq!(code, 0);
    let blocks: Vec<&str> = out.split("\n\n").collect();
    assert_eq!(
        blocks[1],
        "Pattern, near placeholder name\n\
         z.py, line 4, set first set\n\
         m.py, line 4, set first set\n\
         a.py, line 4, set second set\n\
         Flags read, 3\n\
         Contested, 0"
    );
}

#[test]
fn the_closing_counts_stand_in_their_stated_relations() {
    let dir = temp_dir("relations");
    let contested = change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]);
    let agreeing = change_set_json(&[entry_json("app.py", FLAGS_DIFF, AGREEING_VERDICTS)]);
    let text = collection_json(&[("first set", contested), ("second set", agreeing)]);
    let path = write(&dir, "collection.json", &text);
    let (code, out, _err) = run(&["pattern", &path]);
    assert_eq!(code, 0);
    let closing = |head: &str| {
        out.lines()
            .rev()
            .find(|line| line.starts_with(head))
            .map(|line| line.trim_start_matches(head).parse::<usize>().unwrap())
            .unwrap()
    };
    let sections: Vec<&str> = out.split("\n\n").collect();
    let section_flags: usize = sections[1..4]
        .iter()
        .map(|block| {
            block
                .lines()
                .rev()
                .find(|line| line.starts_with("Flags read, "))
                .map(|line| {
                    line.trim_start_matches("Flags read, ")
                        .parse::<usize>()
                        .unwrap()
                })
                .unwrap()
        })
        .sum();
    let section_contested: usize = sections[1..4]
        .iter()
        .map(|block| {
            block
                .lines()
                .rev()
                .find(|line| line.starts_with("Contested, "))
                .map(|line| {
                    line.trim_start_matches("Contested, ")
                        .parse::<usize>()
                        .unwrap()
                })
                .unwrap()
        })
        .sum();
    assert_eq!(closing("Sets read, "), 2);
    assert_eq!(closing("Flags read, "), section_flags);
    assert_eq!(closing("Contested flags, "), section_contested);
    // The closing flags read count is the flagged count the
    // census reads over the same collection and the closing
    // flags read count of the contested reading over it.
    let value = parse(&text).unwrap();
    assert_eq!(closing("Flags read, "), census_of(&value).total.borderline);
    let contested_reading = contested_reading_of(&value);
    assert_eq!(contested_reading.flags_read, closing("Flags read, "));
    assert_eq!(
        contested_reading.contested_flags,
        closing("Contested flags, ")
    );
    // The contested count of one section is the count of
    // the flags the contested reading names under that
    // pattern.
    for (block, section) in sections[1..4]
        .iter()
        .zip(pattern_reading_of(&value).sections.iter())
    {
        let named = contested_reading
            .sets
            .iter()
            .flat_map(|set| set.contested.iter())
            .filter(|flag| flag.pattern == section.pattern)
            .count();
        let stated: usize = block
            .lines()
            .rev()
            .find(|line| line.starts_with("Contested, "))
            .map(|line| line.trim_start_matches("Contested, ").parse().unwrap())
            .unwrap();
        assert_eq!(stated, named, "{}", section.pattern);
    }
}

#[test]
fn one_malformed_member_refuses_the_whole_reading() {
    let dir = temp_dir("whole");
    let good = change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]);
    let bad_verdicts =
        "[{\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":2}]";
    let bad = change_set_json(&[entry_json("app.py", FLAGS_DIFF, bad_verdicts)]);
    let (code, out) = run_pattern(
        &dir,
        &collection_json(&[("good set", good), ("bad set", bad)]),
    );
    assert_eq!(code, 2);
    assert!(
        out.starts_with("Pattern\nVerdict, invalid\nDefects, 1\n"),
        "{out}"
    );
    assert!(
        out.contains(
            "Defect, sets[1].change_set.files[0].verdicts[0].confidence must be from 0 to 1"
        ),
        "{out}"
    );
    assert!(!out.contains("Flags read"), "no partial reading prints");
}

#[test]
fn the_loader_names_the_top_level_defects() {
    assert_eq!(
        load_pattern(&parse("5").unwrap()),
        vec!["pattern must be a JSON object".to_string()]
    );
    assert_eq!(
        load_pattern(&parse("{}").unwrap()),
        vec!["sets is missing".to_string()]
    );
    assert_eq!(
        load_pattern(&parse("{\"sets\":{}}").unwrap()),
        vec!["sets must be a list".to_string()]
    );
    assert_eq!(
        load_pattern(&parse("{\"sets\":[5]}").unwrap()),
        vec!["sets[0] must be an object".to_string()]
    );
    assert_eq!(
        load_pattern(&parse("{\"sets\":[{\"change_set\":{\"files\":[]}}]}").unwrap()),
        vec!["sets[0].label is missing".to_string()]
    );
    assert_eq!(
        load_pattern(
            &parse("{\"sets\":[{\"label\":\"\",\"change_set\":{\"files\":[]}}]}").unwrap()
        ),
        vec!["sets[0].label must be a non empty string".to_string()]
    );
    assert_eq!(
        load_pattern(&parse("{\"sets\":[{\"label\":\"one set\"}]}").unwrap()),
        vec!["sets[0].change_set is missing".to_string()]
    );
    assert_eq!(
        load_pattern(&parse("{\"sets\":[{\"label\":\"one set\",\"change_set\":5}]}").unwrap()),
        vec!["sets[0].change_set must be a JSON object".to_string()]
    );
}

#[test]
fn a_label_holding_a_line_break_is_refused_and_never_printed() {
    let dir = temp_dir("break");
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]);
    let text = collection_json(&[("Flags read, 99\nforged", set)]);
    let (code, out) = run_pattern(&dir, &text);
    assert_eq!(code, 2);
    assert!(
        out.contains("Defect, sets[0].label holds a line break"),
        "{out}"
    );
    assert!(!out.contains("forged"), "the label never prints\n{out}");
    assert!(!out.contains("Flags read, 99"));
}

#[test]
fn a_label_holding_an_unpaired_surrogate_is_refused() {
    let text = "{\"sets\":[{\"label\":\"bad\\ud800label\",\"change_set\":{\"files\":[]}}]}";
    let value = parse(text).unwrap();
    assert_eq!(
        load_pattern(&value),
        vec!["sets[0].label holds an unpaired surrogate".to_string()]
    );
}

#[test]
fn the_command_refuses_unreadable_and_foreign_files() {
    let dir = temp_dir("unreadable");
    let missing = dir.join("nowhere.json").to_string_lossy().into_owned();
    let (code, out, _err) = run(&["pattern", &missing]);
    assert_eq!(code, 2);
    assert_eq!(
        out,
        "Pattern\nVerdict, invalid\nDefects, 1\nDefect, the pattern file could not be read\n"
    );
    let foreign = write(&dir, "foreign.json", "this is not json");
    let (code, out, _err) = run(&["pattern", &foreign]);
    assert_eq!(code, 2);
    assert_eq!(
        out,
        "Pattern\nVerdict, invalid\nDefects, 1\nDefect, the file does not hold a JSON pattern collection\n"
    );
    let deep = write(&dir, "deep.json", &"[".repeat(10050));
    let (code, out, _err) = run(&["pattern", &deep]);
    assert_eq!(code, 2);
    assert!(
        out.contains("Defect, the file does not hold a JSON pattern collection"),
        "nesting past the boundary is a refusal, never a fall\n{out}"
    );
}

#[test]
fn the_command_names_a_file_that_is_not_utf8_text() {
    let dir = temp_dir("notutf8");
    let path = dir.join("bytes.json");
    std::fs::write(&path, [0x80u8, 0x81, 0x82]).unwrap();
    let (code, out, _err) = run(&["pattern", path.to_str().unwrap()]);
    assert_eq!(code, 2);
    assert_eq!(
        out,
        "Pattern\nVerdict, invalid\nDefects, 1\nDefect, the pattern file could not be read\n"
    );
}

#[test]
fn duplicate_labels_and_unread_fields_stand() {
    let dir = temp_dir("duplicates");
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, AGREEING_VERDICTS)]);
    let text = format!(
        "{{\"sets\":[{{\"label\":\"same label\",\"change_set\":{set},\"extra\":1}},{{\"label\":\"same label\",\"change_set\":{set}}}]}}"
    );
    let (code, out) = run_pattern(&dir, &text);
    assert_eq!(code, 0);
    assert_eq!(out.matches("set same label").count(), 4);
    assert!(out.contains("Sets read, 2\n"));
}

#[test]
fn the_reading_writes_nothing() {
    let dir = temp_dir("writes");
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]);
    let path = write(&dir, "input.json", &collection_json(&[("only set", set)]));
    let before: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    let (code, _out, _err) = run(&["pattern", &path]);
    assert_eq!(code, 0);
    let after: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(before, after);
}

/// The fencing of FIX-CK-21 on this reading's own fields.
/// A set label holding a line break and the closing
/// line's own text is refused by the loader, and read
/// through the library the flag line prints the label as
/// the empty field behind the word set, so the forged
/// closing line never stands beside the true one.
#[test]
fn a_line_break_label_mints_no_closing_line_through_the_library() {
    let set = change_set_json(&[entry_json("app.py", ONE_FLAG_DIFF, "")]);
    let text = collection_json(&[("evil\nFlags read, 99", set)]);
    let value = parse(&text).unwrap();
    assert!(!load_pattern(&value).is_empty());
    let out = render_pattern(&pattern_reading_of(&value));
    assert!(
        out.lines().any(|line| line == "app.py, line 4, set "),
        "{out}"
    );
    assert!(!out.lines().any(|line| line == "Flags read, 99"), "{out}");
    let closing: Vec<&str> = out
        .lines()
        .filter(|line| line.starts_with("Flags read"))
        .collect();
    assert_eq!(
        closing,
        [
            "Flags read, 1",
            "Flags read, 0",
            "Flags read, 0",
            "Flags read, 1"
        ],
        "{out}"
    );
}

/// The fencing of FIX-CK-21 on the flag file name. A file
/// name holding a line break and closing text is refused
/// by the loader, and read through the library the flag
/// line prints the file as the empty field, so the line
/// does not split and no closing text stands at a line
/// start.
#[test]
fn a_line_break_file_name_splits_no_flag_line_through_the_library() {
    let set = change_set_json(&[entry_json("evil.py\nFlags read, 99", ONE_FLAG_DIFF, "")]);
    let text = collection_json(&[("only set", set)]);
    let value = parse(&text).unwrap();
    assert!(!load_pattern(&value).is_empty());
    let out = render_pattern(&pattern_reading_of(&value));
    assert!(
        out.lines().any(|line| line == ", line 4, set only set"),
        "{out}"
    );
    assert!(!out.lines().any(|line| line == "Flags read, 99"), "{out}");
}

/// The handed counts of FIX-CK-21 on this reading. A
/// reading built by hand prints its section counts as the
/// counts of the flags it states and its closing counts
/// as the sums over the sections it states, and the
/// counts it was handed never stand, nor does a handed
/// pattern name holding a line break print.
#[test]
fn a_hand_built_reading_prints_the_counts_of_its_sections() {
    let reading = PatternReading {
        sets_read: 1,
        sections: vec![PatternSection {
            pattern: "near placeholder name\nFlags read, 99".to_string(),
            flags: vec![PatternFlag {
                file: "app.py".to_string(),
                line: 4,
                set_label: "only set".to_string(),
                contested: true,
            }],
            contested: 777,
        }],
        flags_read: 555,
        contested_flags: 666,
    };
    let out = render_pattern(&reading);
    assert!(!out.contains("555"), "{out}");
    assert!(!out.contains("666"), "{out}");
    assert!(!out.contains("777"), "{out}");
    assert!(!out.lines().any(|line| line == "Flags read, 99"), "{out}");
    assert!(out.lines().any(|line| line == "Pattern, "), "{out}");
    let lines: Vec<&str> = out.lines().collect();
    let closing = &lines[lines.len() - 3..];
    assert_eq!(
        closing,
        ["Sets read, 1", "Flags read, 1", "Contested flags, 1"],
        "{out}"
    );
}
