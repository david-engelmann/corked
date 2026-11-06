//! Pins for CK-17 and CK-18, the contested file and reading.
//!
//! The contested pins build their collections by hand, so
//! each one fails under the negation of the rule it names,
//! the contested rule of CK-9 read through the standing
//! helper, the order the set review states its contested
//! lines in, the no contested lines line, the whole or
//! nothing refusal, and the closing counts in their stated
//! relations to the sections and to the census over the
//! same collection. The golden pins hold the example of
//! CK-18 byte identical against its stored rendering, and
//! the hand checked pins hold its sections against the
//! contested section of the constructed set rendering and
//! the census golden. Every pin runs offline on recorded
//! and constructed entries, and no pin makes a live call.

use std::path::{Path, PathBuf};

use stickler::census::{census_of, load_census};
use stickler::cli::cli_main;
use stickler::contested::{
    contested_reading_of, load_contested, Contested, ContestedFlag, ContestedSet,
};
use stickler::json::parse;
use stickler::render::{render_census, render_contested};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn examples() -> PathBuf {
    root().join("examples")
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "stickler-contested-{}-{}-{}",
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

fn run_contested(dir: &Path, text: &str) -> (i32, String) {
    let path = write(dir, "contested.json", text);
    let (code, out, _err) = run(&["contested", &path]);
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
/// CK-17 and CK-12 share.
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

/// Two verdicts contesting the line 4 flag of FLAGS_DIFF,
/// the first carrying the given raw JSON text as one extra
/// field the verdict walk does not read, or no extra field
/// when the text is empty, in the shape of finding 1 of
/// the Round 8 red team review.
fn pair_verdicts(extra: &str) -> String {
    let field = if extra.is_empty() {
        String::new()
    } else {
        format!(",\"junk\":{extra}")
    };
    format!(
        "[{{\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":0.9{field}}},{{\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"openai/gpt-6-luna\",\"choice\":\"clean\",\"confidence\":0.9}}]"
    )
}

#[test]
fn the_example_matches_its_golden_through_the_command() {
    let path = examples().join("contested.json");
    let (code, out, _err) = run(&["contested", path.to_str().unwrap()]);
    assert_eq!(code, 0);
    let golden = std::fs::read_to_string(examples().join("contested.txt")).unwrap();
    assert_eq!(out, golden);
}

#[test]
fn the_example_matches_its_golden_through_the_library() {
    let text = std::fs::read_to_string(examples().join("contested.json")).unwrap();
    let value = parse(&text).unwrap();
    assert_eq!(load_contested(&value), Vec::<String>::new());
    let golden = std::fs::read_to_string(examples().join("contested.txt")).unwrap();
    assert_eq!(render_contested(&contested_reading_of(&value)), golden);
}

#[test]
fn the_example_holds_the_census_collection() {
    let contested_text = std::fs::read_to_string(examples().join("contested.json")).unwrap();
    let census_text = std::fs::read_to_string(examples().join("census.json")).unwrap();
    let contested_value = parse(&contested_text).unwrap();
    let census_value = parse(&census_text).unwrap();
    assert_eq!(load_census(&contested_value), Vec::<String>::new());
    assert_eq!(
        render_census(&census_of(&contested_value)),
        render_census(&census_of(&census_value)),
        "the contested example reads under the census as the census example itself"
    );
    let reading = contested_reading_of(&contested_value);
    let labels: Vec<&str> = reading.sets.iter().map(|set| set.label.as_str()).collect();
    assert_eq!(
        labels,
        vec!["round 2 set", "pilot set", "constructed round 4 example"]
    );
}

#[test]
fn the_example_sections_stand_hand_checked() {
    let golden = std::fs::read_to_string(examples().join("contested.txt")).unwrap();
    let blocks: Vec<&str> = golden.split("\n\n").collect();
    assert_eq!(blocks.len(), 5, "{golden}");
    assert_eq!(blocks[0], "Contested");
    assert_eq!(
        blocks[1],
        "Set, round 2 set\nNo contested lines\nFlags read, 0\nContested, 0"
    );
    assert_eq!(
        blocks[2],
        "Set, pilot set\nNo contested lines\nFlags read, 12\nContested, 0"
    );
    assert_eq!(
        blocks[3],
        "Set, constructed round 4 example\nledger.py, line 6, near placeholder name\ncache.py, line 15, near placeholder name\nFlags read, 6\nContested, 2"
    );
    assert_eq!(
        blocks[4],
        "Sets read, 3\nFlags read, 18\nContested flags, 2\nSets with contested flags, 1\n"
    );
    // The two named flags are the contested lines of the
    // constructed set review, in its stated order.
    let review = std::fs::read_to_string(examples().join("contested_review.txt")).unwrap();
    let section = review
        .split("Contested lines\n")
        .nth(1)
        .expect("the constructed review holds a contested section");
    assert!(
        section.contains("File, ledger.py\nBorderline 1, near placeholder name"),
        "{section}"
    );
    assert!(
        section.contains("File, cache.py\nBorderline 3, near placeholder name"),
        "{section}"
    );
}

#[test]
fn a_single_set_collection_stands_line_for_line() {
    let dir = temp_dir("single");
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]);
    let (code, out) = run_contested(&dir, &collection_json(&[("only set", set)]));
    assert_eq!(code, 0);
    let expected = "\
Contested

Set, only set
app.py, line 4, near placeholder name
Flags read, 2
Contested, 1

Sets read, 1
Flags read, 2
Contested flags, 1
Sets with contested flags, 1
";
    assert_eq!(out, expected);
}

#[test]
fn the_closing_counts_sum_the_sections() {
    let dir = temp_dir("sums");
    let contested = change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]);
    let agreeing = change_set_json(&[entry_json("app.py", FLAGS_DIFF, AGREEING_VERDICTS)]);
    let single = change_set_json(&[entry_json(
        "app.py",
        ONE_FLAG_DIFF,
        "[{\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"jev-1.13.0\",\"choice\":\"clean\",\"confidence\":0.9},\
          {\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"openai/gpt-6-luna\",\"choice\":\"stickler\",\"confidence\":0.9}]",
    )]);
    let (code, out) = run_contested(
        &dir,
        &collection_json(&[
            ("first set", contested),
            ("second set", agreeing),
            ("third set", single),
        ]),
    );
    assert_eq!(code, 0);
    let section_contested: usize = out
        .lines()
        .filter(|line| line.starts_with("Contested, "))
        .map(|line| {
            line.trim_start_matches("Contested, ")
                .parse::<usize>()
                .unwrap()
        })
        .sum();
    let closing = |head: &str| {
        out.lines()
            .rev()
            .find(|line| line.starts_with(head))
            .map(|line| line.trim_start_matches(head).parse::<usize>().unwrap())
            .unwrap()
    };
    assert_eq!(section_contested, closing("Contested flags, "));
    assert_eq!(section_contested, 2);
    assert_eq!(closing("Sets read, "), 3);
    assert_eq!(closing("Flags read, "), 5);
    assert_eq!(closing("Sets with contested flags, "), 2);
}

#[test]
fn the_closing_flags_read_matches_the_census_over_the_same_collection() {
    let dir = temp_dir("reconcile");
    let contested = change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]);
    let agreeing = change_set_json(&[entry_json("app.py", FLAGS_DIFF, AGREEING_VERDICTS)]);
    let text = collection_json(&[("first set", contested), ("second set", agreeing)]);
    let path = write(&dir, "collection.json", &text);
    let (code, contested_out, _err) = run(&["contested", &path]);
    assert_eq!(code, 0);
    let (code, census_out, _err) = run(&["census", &path]);
    assert_eq!(code, 0);
    let census_borderline = census_out
        .lines()
        .rev()
        .find(|line| line.starts_with("Borderline, "))
        .map(|line| line.trim_start_matches("Borderline, ").to_string())
        .unwrap();
    let closing_flags = contested_out
        .lines()
        .rev()
        .find(|line| line.starts_with("Flags read, "))
        .map(|line| line.trim_start_matches("Flags read, ").to_string())
        .unwrap();
    assert_eq!(closing_flags, census_borderline);
    assert_eq!(closing_flags, "4");
}

#[test]
fn a_set_whose_routes_agree_everywhere_reads_the_no_contest_line() {
    let dir = temp_dir("agreeing");
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, AGREEING_VERDICTS)]);
    let (code, out) = run_contested(&dir, &collection_json(&[("agreeing set", set)]));
    assert_eq!(code, 0);
    assert!(out.contains("Set, agreeing set\nNo contested lines\nFlags read, 2\nContested, 0\n"));
    assert!(out.contains("Sets with contested flags, 0\n"));
    assert!(!out.contains("line 4, near placeholder name"));
}

#[test]
fn an_abstention_beside_a_label_does_not_contest() {
    let dir = temp_dir("abstention");
    let verdicts = "[
        {\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":0.3},
        {\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"openai/gpt-6-luna\",\"choice\":\"clean\",\"confidence\":0.9},
        {\"line\":5,\"pattern\":\"broad handler\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":0.9},
        {\"line\":5,\"pattern\":\"broad handler\",\"model\":\"openai/gpt-6-luna\",\"choice\":\"stickler\",\"confidence\":0.4}
    ]";
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, verdicts)]);
    let (code, out) = run_contested(&dir, &collection_json(&[("abstaining set", set)]));
    assert_eq!(code, 0);
    assert!(out.contains("No contested lines\n"), "{out}");
    assert!(out.contains("Contested flags, 0\n"), "{out}");
}

#[test]
fn one_model_holding_both_labels_does_not_contest() {
    let verdicts = "[
        {\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":0.9},
        {\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"jev-1.13.0\",\"choice\":\"clean\",\"confidence\":0.9}
    ]";
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, verdicts)]);
    let text = collection_json(&[("split model set", set)]);
    let value = parse(&text).unwrap();
    // The loader refuses the set, two verdicts of one model
    // on one line being the defect CK-8 names.
    let defects = load_contested(&value);
    assert!(
        defects.iter().any(|defect| defect
            == "sets[0].change_set.files[0].verdicts[1] repeats a verdict for the same line and model"),
        "{defects:?}"
    );
    // The library reading over the same value still contests
    // nothing, the helper of CK-9 reading model identity as
    // part of the rule.
    let reading = contested_reading_of(&value);
    assert_eq!(reading.sets.len(), 1);
    assert!(reading.sets[0].contested.is_empty());
    assert_eq!(reading.contested_flags, 0);
}

#[test]
fn a_collection_where_every_set_is_contested_counts_every_set() {
    let dir = temp_dir("every");
    let first = change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]);
    let second = change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]);
    let (code, out) = run_contested(
        &dir,
        &collection_json(&[("first set", first), ("second set", second)]),
    );
    assert_eq!(code, 0);
    assert_eq!(
        out.matches("app.py, line 4, near placeholder name").count(),
        2
    );
    assert!(out.contains("Contested flags, 2\n"));
    assert!(out.contains("Sets with contested flags, 2\n"));
    assert!(!out.contains("No contested lines"));
}

#[test]
fn an_empty_collection_renders_zero_counts() {
    let dir = temp_dir("empty");
    let (code, out) = run_contested(&dir, "{\"sets\":[]}");
    assert_eq!(code, 0);
    let expected = "\
Contested

Sets read, 0
Flags read, 0
Contested flags, 0
Sets with contested flags, 0
";
    assert_eq!(out, expected);
}

#[test]
fn a_set_with_no_flags_reads_the_no_contest_line_with_zero_counts() {
    let dir = temp_dir("noflags");
    let set = change_set_json(&[entry_json("app.py", JUDGED_DIFF, "")]);
    let (code, out) = run_contested(&dir, &collection_json(&[("judged set", set)]));
    assert_eq!(code, 0);
    assert!(out.contains("Set, judged set\nNo contested lines\nFlags read, 0\nContested, 0\n"));
}

#[test]
fn one_malformed_member_refuses_the_whole_reading() {
    let dir = temp_dir("whole");
    let good = change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]);
    let bad_verdicts =
        "[{\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":2}]";
    let bad = change_set_json(&[entry_json("app.py", FLAGS_DIFF, bad_verdicts)]);
    let (code, out) = run_contested(
        &dir,
        &collection_json(&[("good set", good), ("bad set", bad)]),
    );
    assert_eq!(code, 2);
    assert!(
        out.starts_with("Contested\nVerdict, invalid\nDefects, 1\n"),
        "{out}"
    );
    assert!(
        out.contains(
            "Defect, sets[1].change_set.files[0].verdicts[0].confidence must be from 0 to 1"
        ),
        "{out}"
    );
    assert!(!out.contains("Set, good set"), "no partial reading prints");
    assert!(!out.contains("Sets read"));
}

#[test]
fn a_huge_integer_confidence_in_a_member_is_refused_by_range() {
    let dir = temp_dir("huge");
    let huge = "1".repeat(400);
    let verdicts = format!(
        "[{{\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"jev-1.13.0\",\"choice\":\"stickler\",\"confidence\":{huge}}}]"
    );
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, &verdicts)]);
    let (code, out) = run_contested(&dir, &collection_json(&[("huge set", set)]));
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
        load_contested(&parse("5").unwrap()),
        vec!["contested must be a JSON object".to_string()]
    );
    assert_eq!(
        load_contested(&parse("{}").unwrap()),
        vec!["sets is missing".to_string()]
    );
    assert_eq!(
        load_contested(&parse("{\"sets\":{}}").unwrap()),
        vec!["sets must be a list".to_string()]
    );
    assert_eq!(
        load_contested(&parse("{\"sets\":[5]}").unwrap()),
        vec!["sets[0] must be an object".to_string()]
    );
    assert_eq!(
        load_contested(&parse("{\"sets\":[{\"change_set\":{\"files\":[]}}]}").unwrap()),
        vec!["sets[0].label is missing".to_string()]
    );
    assert_eq!(
        load_contested(
            &parse("{\"sets\":[{\"label\":\"\",\"change_set\":{\"files\":[]}}]}").unwrap()
        ),
        vec!["sets[0].label must be a non empty string".to_string()]
    );
    assert_eq!(
        load_contested(&parse("{\"sets\":[{\"label\":\"one set\"}]}").unwrap()),
        vec!["sets[0].change_set is missing".to_string()]
    );
    assert_eq!(
        load_contested(&parse("{\"sets\":[{\"label\":\"one set\",\"change_set\":5}]}").unwrap()),
        vec!["sets[0].change_set must be a JSON object".to_string()]
    );
}

#[test]
fn a_label_holding_a_line_break_is_refused_and_never_printed() {
    let dir = temp_dir("break");
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]);
    let text = collection_json(&[("Sets read, 99\nforged", set)]);
    let (code, out) = run_contested(&dir, &text);
    assert_eq!(code, 2);
    assert!(
        out.contains("Defect, sets[0].label holds a line break"),
        "{out}"
    );
    assert!(!out.contains("forged"), "the label never prints\n{out}");
    assert!(!out.contains("Sets read, 99"));
}

#[test]
fn a_label_holding_an_unpaired_surrogate_is_refused() {
    let text = "{\"sets\":[{\"label\":\"bad\\ud800label\",\"change_set\":{\"files\":[]}}]}";
    let value = parse(text).unwrap();
    assert_eq!(
        load_contested(&value),
        vec!["sets[0].label holds an unpaired surrogate".to_string()]
    );
}

#[test]
fn a_structural_label_prints_behind_the_set_word_only() {
    let dir = temp_dir("fencing");
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]);
    let (code, out) = run_contested(&dir, &collection_json(&[("Sets read, 99", set)]));
    assert_eq!(code, 0);
    assert!(out.contains("Set, Sets read, 99\n"), "{out}");
    let closing: Vec<&str> = out
        .lines()
        .filter(|line| line.starts_with("Sets read, "))
        .collect();
    assert_eq!(closing, vec!["Sets read, 1"]);
}

#[test]
fn the_command_refuses_unreadable_and_foreign_files() {
    let dir = temp_dir("unreadable");
    let missing = dir.join("nowhere.json").to_string_lossy().into_owned();
    let (code, out, _err) = run(&["contested", &missing]);
    assert_eq!(code, 2);
    assert_eq!(
        out,
        "Contested\nVerdict, invalid\nDefects, 1\nDefect, the contested file could not be read\n"
    );
    let foreign = write(&dir, "foreign.json", "this is not json");
    let (code, out, _err) = run(&["contested", &foreign]);
    assert_eq!(code, 2);
    assert_eq!(
        out,
        "Contested\nVerdict, invalid\nDefects, 1\nDefect, the file does not hold a JSON contested collection\n"
    );
    let deep = write(&dir, "deep.json", &"[".repeat(10050));
    let (code, out, _err) = run(&["contested", &deep]);
    assert_eq!(code, 2);
    assert!(
        out.contains("Defect, the file does not hold a JSON contested collection"),
        "nesting past the boundary is a refusal, never a fall\n{out}"
    );
}

#[test]
fn the_command_names_a_file_that_is_not_utf8_text() {
    let dir = temp_dir("notutf8");
    let path = dir.join("bytes.json");
    std::fs::write(&path, [0x80u8, 0x81, 0x82]).unwrap();
    let (code, out, _err) = run(&["contested", path.to_str().unwrap()]);
    assert_eq!(code, 2);
    assert_eq!(
        out,
        "Contested\nVerdict, invalid\nDefects, 1\nDefect, the contested file could not be read\n"
    );
}

#[test]
fn duplicate_labels_and_unread_fields_stand() {
    let dir = temp_dir("duplicates");
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, AGREEING_VERDICTS)]);
    let text = format!(
        "{{\"sets\":[{{\"label\":\"same label\",\"change_set\":{set},\"extra\":1}},{{\"label\":\"same label\",\"change_set\":{set}}}]}}"
    );
    let (code, out) = run_contested(&dir, &text);
    assert_eq!(code, 0);
    assert_eq!(out.matches("Set, same label").count(), 2);
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
    let (code, _out, _err) = run(&["contested", &path]);
    assert_eq!(code, 0);
    let after: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(before, after);
}

/// Finding 1 of the Round 8 red team review, closed by
/// FIX-CK-19. A verdict carrying one ignored field nested
/// to a depth the reader accepts is a sound member, and
/// the contested, census, and review readings over it
/// render exactly as the same member without the field
/// renders, with no command dying. Before the repair the
/// derived clone of the parsed value walked the tree by
/// recursion and each command died on this file with a
/// stack overflow.
#[test]
fn a_verdict_carrying_a_deeply_nested_ignored_field_renders() {
    let dir = temp_dir("deepjunk");
    let junk = "[".repeat(9500) + "1" + &"]".repeat(9500);
    let deep = collection_json(&[(
        "only set",
        change_set_json(&[entry_json("app.py", FLAGS_DIFF, &pair_verdicts(&junk))]),
    )]);
    let plain = collection_json(&[(
        "only set",
        change_set_json(&[entry_json("app.py", FLAGS_DIFF, &pair_verdicts(""))]),
    )]);
    let value = parse(&deep).unwrap();
    assert!(load_contested(&value).is_empty());
    assert_eq!(contested_reading_of(&value).contested_flags, 1);
    let deep_path = write(&dir, "deep.json", &deep);
    let plain_path = write(&dir, "plain.json", &plain);
    for command in ["contested", "census"] {
        let (deep_code, deep_out, _err) = run(&[command, &deep_path]);
        let (plain_code, plain_out, _err) = run(&[command, &plain_path]);
        assert_eq!(deep_code, 0, "{command}");
        assert_eq!(deep_code, plain_code, "{command}");
        assert_eq!(deep_out, plain_out, "{command}");
    }
    let (code, out, _err) = run(&["contested", &deep_path]);
    assert_eq!(code, 0);
    assert!(out.contains("Contested flags, 1"), "{out}");
    let deep_set_path = write(
        &dir,
        "deep_set.json",
        &change_set_json(&[entry_json("app.py", FLAGS_DIFF, &pair_verdicts(&junk))]),
    );
    let plain_set_path = write(
        &dir,
        "plain_set.json",
        &change_set_json(&[entry_json("app.py", FLAGS_DIFF, &pair_verdicts(""))]),
    );
    let (deep_code, deep_out, _err) = run(&["review", &deep_set_path]);
    let (plain_code, plain_out, _err) = run(&["review", &plain_set_path]);
    assert_eq!(deep_code, plain_code);
    assert_eq!(deep_out, plain_out);
    assert!(deep_out.contains("Contested lines"), "{deep_out}");
}

/// Finding 2 of the Round 8 red team review, closed by
/// FIX-CK-20. A set label holding a line break and the
/// closing line's own text is refused by the loader, and
/// read through the library the rendering prints the
/// label as the empty field, so the forged closing line
/// never stands beside the true one.
#[test]
fn a_line_break_label_mints_no_closing_line_through_the_library() {
    let set = change_set_json(&[entry_json("app.py", FLAGS_DIFF, CONTESTED_VERDICTS)]);
    let text = collection_json(&[("evil\nSets read, 99", set)]);
    let value = parse(&text).unwrap();
    assert!(!load_contested(&value).is_empty());
    let out = render_contested(&contested_reading_of(&value));
    assert!(out.lines().any(|line| line == "Set, "), "{out}");
    assert!(!out.lines().any(|line| line == "Sets read, 99"), "{out}");
    let closing: Vec<&str> = out
        .lines()
        .filter(|line| line.starts_with("Sets read"))
        .collect();
    assert_eq!(closing, ["Sets read, 1"], "{out}");
}

/// Finding 2, the flag line. A file name holding a line
/// break and closing text is refused by the loader, and
/// read through the library the flag line prints the file
/// as the empty field, so the line does not split and no
/// closing text stands at a line start.
#[test]
fn a_line_break_file_name_splits_no_flag_line_through_the_library() {
    let set = change_set_json(&[entry_json(
        "evil.py\nSets read, 99",
        FLAGS_DIFF,
        CONTESTED_VERDICTS,
    )]);
    let text = collection_json(&[("only set", set)]);
    let value = parse(&text).unwrap();
    assert!(!load_contested(&value).is_empty());
    let out = render_contested(&contested_reading_of(&value));
    assert!(
        out.lines()
            .any(|line| line == ", line 4, near placeholder name"),
        "{out}"
    );
    assert!(
        !out.lines()
            .any(|line| line == "Sets read, 99, line 4, near placeholder name"),
        "{out}"
    );
    let closing: Vec<&str> = out
        .lines()
        .filter(|line| line.starts_with("Sets read"))
        .collect();
    assert_eq!(closing, ["Sets read, 1"], "{out}");
}

/// Finding 2, the handed pattern. A pattern holding a
/// line break, standing on a reading built by hand, prints
/// as the empty field, so the flag line does not split.
#[test]
fn a_line_break_pattern_splits_no_flag_line_in_a_hand_built_reading() {
    let reading = Contested {
        sets: vec![ContestedSet {
            label: "only set".to_string(),
            contested: vec![ContestedFlag {
                file: "app.py".to_string(),
                line: 4,
                pattern: "near placeholder name\nSets read, 99".to_string(),
            }],
            flags_read: 1,
        }],
        flags_read: 1,
        contested_flags: 1,
        sets_with_contested: 1,
    };
    let out = render_contested(&reading);
    assert!(out.lines().any(|line| line == "app.py, line 4, "), "{out}");
    assert!(!out.lines().any(|line| line == "Sets read, 99"), "{out}");
}

/// Finding 2, the handed label. A label holding a line
/// break, standing on a reading built by hand, prints as
/// the empty field, so no closing line is minted beside
/// the true one.
#[test]
fn a_line_break_label_mints_no_closing_line_in_a_hand_built_reading() {
    let reading = Contested {
        sets: vec![ContestedSet {
            label: "evil\nSets read, 99".to_string(),
            contested: Vec::new(),
            flags_read: 0,
        }],
        flags_read: 0,
        contested_flags: 0,
        sets_with_contested: 0,
    };
    let out = render_contested(&reading);
    assert!(out.lines().any(|line| line == "Set, "), "{out}");
    let closing: Vec<&str> = out
        .lines()
        .filter(|line| line.starts_with("Sets read"))
        .collect();
    assert_eq!(closing, ["Sets read, 1"], "{out}");
}

/// Finding 2, the handed counts. A reading built by hand
/// prints its closing counts as the sums over the sections
/// it states, under FIX-CK-20, and the counts it was
/// handed never stand.
#[test]
fn a_hand_built_reading_prints_the_counts_of_its_sections() {
    let reading = Contested {
        sets: vec![ContestedSet {
            label: "only set".to_string(),
            contested: vec![ContestedFlag {
                file: "app.py".to_string(),
                line: 4,
                pattern: "near placeholder name".to_string(),
            }],
            flags_read: 2,
        }],
        flags_read: 555,
        contested_flags: 666,
        sets_with_contested: 777,
    };
    let out = render_contested(&reading);
    assert!(!out.contains("555"), "{out}");
    assert!(!out.contains("666"), "{out}");
    assert!(!out.contains("777"), "{out}");
    let lines: Vec<&str> = out.lines().collect();
    let closing = &lines[lines.len() - 4..];
    assert_eq!(
        closing,
        [
            "Sets read, 1",
            "Flags read, 2",
            "Contested flags, 1",
            "Sets with contested flags, 1"
        ],
        "{out}"
    );
}
