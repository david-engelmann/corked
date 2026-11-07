//! Pins for CK-13 and CK-14, the rejudged set and ledger.
//!
//! The rejudged pins build their pairs by hand, so each one
//! fails under the negation of the rule it names, the
//! identity matching by file, line, and pattern with
//! occurrences matched in order, the held and changed words
//! read through the standing CK-8 outcomes, the whole or
//! nothing refusal, and the closing counts. The golden pins
//! hold the example pair of CK-14 byte identical against its
//! stored rendering, and the replay pins rebuild the
//! rejudged set of CK-13 from its kept envelopes through the
//! standing seat reader and recompute its run record from
//! the same envelopes. Every pin runs offline on recorded
//! and constructed entries, and no pin makes a live call.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use stickler::changeset::review_changeset;
use stickler::cli::cli_main;
use stickler::json::{parse, JsonNumber, JsonValue};
use stickler::rejudged::{load_rejudged, rejudged_of, ComparedInput, Rejudged};
use stickler::render::render_rejudged;
use stickler::verdicts::{
    build_verdict_entry, verdict_from_jev_envelope, verdict_outcome, VerdictEntry,
};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn examples() -> PathBuf {
    root().join("examples")
}

fn envelopes_rejudged() -> PathBuf {
    examples().join("verdicts").join("envelopes_rejudged")
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "stickler-rejudged-{}-{}-{}",
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

fn run_rejudged(dir: &Path, text: &str) -> (i32, String) {
    let path = write(dir, "rejudged.json", text);
    let (code, out, _err) = run(&["rejudged", &path]);
    (code, out)
}

fn load_json(path: &Path) -> JsonValue {
    let text = std::fs::read_to_string(path).unwrap();
    parse(&text).unwrap()
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

/// One recorded verdict as JSON text, the confidence given
/// raw so a pin can state one out of range.
fn verdict_json(line: usize, pattern: &str, choice: &str, confidence: &str) -> String {
    format!(
        "{{\"line\":{line},\"pattern\":\"{}\",\"model\":\"jev-1.13.0\",\"choice\":\"{choice}\",\"confidence\":{confidence}}}",
        jstr(pattern)
    )
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

fn change_set_json(entries: &[String]) -> String {
    format!("{{\"files\":[{}]}}", entries.join(","))
}

/// One rejudged pair text over two labelled change sets.
fn pair_json(first_label: &str, first: String, second_label: &str, second: String) -> String {
    format!(
        "{{\"sets\":[{{\"label\":\"{}\",\"change_set\":{}}},{{\"label\":\"{}\",\"change_set\":{}}}]}}",
        jstr(first_label),
        first,
        jstr(second_label),
        second
    )
}

/// A diff flagging the near placeholder name at line 4 and
/// the broad handler at line 5.
const FLAGS_DIFF: &str =
    "--- a/app.py\n+++ b/app.py\n@@ -1,1 +1,3 @@\n+data = load_entries()\n+    except BaseException:\n";

/// The same two flags with their lines exchanged, so line 4
/// carries the broad handler and line 5 the placeholder name.
const SWAPPED_DIFF: &str =
    "--- a/app.py\n+++ b/app.py\n@@ -1,1 +1,3 @@\n+    except BaseException:\n+data = load_entries()\n";

fn set_with(verdicts: &str) -> String {
    change_set_json(&[entry_json("app.py", FLAGS_DIFF, verdicts)])
}

#[test]
fn the_example_pair_matches_its_golden_through_the_command() {
    let path = examples().join("rejudged.json");
    let (code, out, _err) = run(&["rejudged", path.to_str().unwrap()]);
    assert_eq!(code, 0);
    let golden = std::fs::read_to_string(examples().join("rejudged.txt")).unwrap();
    assert_eq!(out, golden);
}

#[test]
fn the_example_pair_matches_its_golden_through_the_library() {
    let text = std::fs::read_to_string(examples().join("rejudged.json")).unwrap();
    let value = parse(&text).unwrap();
    assert_eq!(load_rejudged(&value), Vec::<String>::new());
    let golden = std::fs::read_to_string(examples().join("rejudged.txt")).unwrap();
    assert_eq!(render_rejudged(&rejudged_of(&value)), golden);
}

#[test]
fn the_example_pair_reads_the_stated_labels_and_counts() {
    let value = load_json(&examples().join("rejudged.json"));
    let reading = rejudged_of(&value);
    assert_eq!(reading.first_label, "pilot set");
    assert_eq!(reading.second_label, "rejudged set");
    assert_eq!(reading.inputs_read, 24);
    assert_eq!(reading.pairs.len(), 12);
    assert_eq!(reading.held, 10);
    assert_eq!(reading.changed, 2);
    assert_eq!(
        reading.held + reading.changed,
        reading.pairs.len(),
        "the closing counts close over the standing pairs"
    );
    let changed: Vec<(&str, usize)> = reading
        .pairs
        .iter()
        .filter(|pair| !pair.held)
        .map(|pair| (pair.file.as_str(), pair.line))
        .collect();
    assert_eq!(changed, vec![("harvest.py", 6), ("sorting.py", 11)]);
}

#[test]
fn the_contract_states_the_composition_of_the_example_pair() {
    // Stands under FIX-CK-18. The disclosure sentence of
    // CK-14 stands in the contract word for word, so the
    // held and changed counts of the example never stand
    // in the contract without the composition of the pair
    // beside them. The pin fails on the contract as first
    // written, which carried no such sentence.
    let text = std::fs::read_to_string(root().join("CONTRACT.md")).unwrap();
    let sentence = "The first asking's judgements rest on the verdicts of both \
        routes of the pilot run and the second asking's on the Jev route \
        alone, so a changed pair can record a route composition \
        difference or an abstention threshold crossing and not a \
        reversed choice.";
    assert!(
        text.contains(sentence),
        "the composition sentence stands in CK-14"
    );
}

#[test]
fn the_rejudged_set_replays_through_the_review_command_to_its_golden() {
    let path = examples().join("rejudged_set.json");
    let (code, out, _err) = run(&["review", path.to_str().unwrap()]);
    assert_eq!(code, 1, "the rejudged set stands borderline\n{out}");
    let golden = std::fs::read_to_string(examples().join("rejudged_set.txt")).unwrap();
    assert_eq!(out, golden);
}

/// The pilot flags in review order, each with the index of
/// its file, read from the pilot set exactly as recorded.
fn pilot_flags() -> Vec<(usize, String, stickler::checker::Flag)> {
    let value = load_json(&examples().join("borderline_set.json"));
    let mut flags = Vec::new();
    for (index, result) in review_changeset(&value).into_iter().enumerate() {
        for block in &result.blocks {
            flags.push((index + 1, result.name.clone(), block.flag.clone()));
        }
    }
    flags
}

fn envelope_name(file_index: usize, flag: &stickler::checker::Flag) -> String {
    format!("f{file_index}-l{:02}-jev.json", flag.line)
}

#[test]
fn the_rejudged_envelope_inventory_matches_the_twelve_flags() {
    let mut names: Vec<String> = std::fs::read_dir(envelopes_rejudged())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".json"))
        .collect();
    names.sort();
    let mut expected: Vec<String> = pilot_flags()
        .iter()
        .map(|(index, _name, flag)| envelope_name(*index, flag))
        .collect();
    expected.sort();
    assert_eq!(expected.len(), 12);
    assert_eq!(names, expected);
}

fn entry_of(value: &JsonValue) -> VerdictEntry {
    let line = match value.get("line").and_then(|v| v.as_number()) {
        Some(JsonNumber::Int(b)) => b.to_string().parse().unwrap(),
        _ => panic!("line stands as an integer"),
    };
    let field = |key: &str| {
        value
            .get(key)
            .and_then(|v| v.as_jstring())
            .and_then(|v| v.to_rust_string())
            .unwrap_or_else(|| panic!("{key} stands as a string"))
    };
    VerdictEntry {
        line,
        pattern: field("pattern"),
        model: field("model"),
        choice: field("choice"),
        confidence: value
            .get("confidence")
            .and_then(|v| v.as_number())
            .expect("confidence stands as a number")
            .clone(),
    }
}

#[test]
fn the_rejudged_set_rebuilds_from_its_envelopes() {
    let mut per_file: BTreeMap<String, Vec<VerdictEntry>> = BTreeMap::new();
    for (index, name, flag) in pilot_flags() {
        let envelope = load_json(&envelopes_rejudged().join(envelope_name(index, &flag)));
        let verdict = verdict_from_jev_envelope(&envelope)
            .expect("every call of the rejudging run recorded a verdict");
        per_file
            .entry(name)
            .or_default()
            .push(build_verdict_entry(&flag, &verdict));
    }
    let recorded_set = load_json(&examples().join("rejudged_set.json"));
    let files = recorded_set
        .get("files")
        .and_then(|files| files.as_array())
        .expect("files stands");
    assert_eq!(files.len(), 3);
    for held in files {
        let name = held
            .get("file")
            .and_then(|file| file.as_jstring())
            .and_then(|file| file.to_rust_string())
            .expect("file stands");
        let recorded: Vec<VerdictEntry> = held
            .get("verdicts")
            .and_then(|verdicts| verdicts.as_array())
            .map(|verdicts| verdicts.iter().map(entry_of).collect())
            .unwrap_or_default();
        let rebuilt = per_file.get(&name).cloned().unwrap_or_default();
        assert_eq!(recorded, rebuilt, "file {name}");
    }
}

fn int_of(value: &JsonValue) -> i64 {
    match value.as_number() {
        Some(JsonNumber::Int(b)) => b.to_string().parse().unwrap(),
        _ => panic!("an integer stands"),
    }
}

#[test]
fn the_rejudged_run_record_recomputes_from_the_envelopes() {
    let record = load_json(&examples().join("verdicts").join("run_record_rejudged.json"));
    let routes = record
        .get("routes")
        .and_then(|routes| routes.as_array())
        .expect("routes stands");
    assert_eq!(routes.len(), 1, "the run stood on the Jev route alone");
    let route = &routes[0];
    let model = route
        .get("model")
        .and_then(|model| model.as_jstring())
        .and_then(|model| model.to_rust_string())
        .expect("model stands");
    assert_eq!(model, "jev-1.13.0");
    assert!(
        route.get("cost_dollars").is_none(),
        "the Jev route stands unpriced, so the record holds no dollar figure"
    );
    let (mut attempted, mut recorded, mut failed, mut abstained) = (0i64, 0i64, 0i64, 0i64);
    let (mut tin, mut tout) = (0i64, 0i64);
    for (index, _name, flag) in pilot_flags() {
        let envelope = load_json(&envelopes_rejudged().join(envelope_name(index, &flag)));
        attempted += 1;
        match verdict_from_jev_envelope(&envelope) {
            None => failed += 1,
            Some(verdict) => {
                recorded += 1;
                let held = build_verdict_entry(&flag, &verdict);
                if verdict_outcome(&held) == "abstained" {
                    abstained += 1;
                }
                let usage = envelope.get("usage").expect("usage stands");
                tin += int_of(usage.get("input_tokens").expect("input stands"));
                tout += int_of(usage.get("output_tokens").expect("output stands"));
            }
        }
    }
    assert_eq!(attempted, 12);
    assert_eq!(int_of(route.get("calls_attempted").unwrap()), attempted);
    assert_eq!(int_of(route.get("verdicts_recorded").unwrap()), recorded);
    assert_eq!(int_of(route.get("calls_failed").unwrap()), failed);
    assert_eq!(int_of(route.get("abstentions").unwrap()), abstained);
    assert_eq!(int_of(route.get("input_tokens").unwrap()), tin);
    assert_eq!(int_of(route.get("output_tokens").unwrap()), tout);
}

#[test]
fn a_pair_judged_the_same_at_both_askings_stands_held_line_for_line() {
    let dir = temp_dir("held");
    let verdicts = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "stickler", "0.9")
    );
    let (code, out) = run_rejudged(
        &dir,
        &pair_json(
            "first asking",
            set_with(&verdicts),
            "second asking",
            set_with(&verdicts),
        ),
    );
    assert_eq!(code, 0);
    let expected = "\
Rejudged
First, first asking
Second, second asking

app.py, line 4, near placeholder name
First, stickler
Second, stickler
Held

app.py, line 5, broad handler
First, unjudged
Second, unjudged
Held

Inputs read, 4
Standing in both, 2
Held, 2
Changed, 0
";
    assert_eq!(out, expected);
}

#[test]
fn a_pair_judged_differently_stands_changed_in_each_direction() {
    let dir = temp_dir("changed");
    let stickler_side = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "stickler", "0.9")
    );
    let clean_side = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "clean", "0.9")
    );
    let (code, out) = run_rejudged(
        &dir,
        &pair_json(
            "first asking",
            set_with(&stickler_side),
            "second asking",
            set_with(&clean_side),
        ),
    );
    assert_eq!(code, 0);
    assert!(
        out.contains(
            "app.py, line 4, near placeholder name\nFirst, stickler\nSecond, clean\nChanged"
        ),
        "{out}"
    );
    assert!(
        out.contains("Standing in both, 2\nHeld, 1\nChanged, 1\n"),
        "{out}"
    );
    let (code, out) = run_rejudged(
        &dir,
        &pair_json(
            "first asking",
            set_with(&clean_side),
            "second asking",
            set_with(&stickler_side),
        ),
    );
    assert_eq!(code, 0);
    assert!(
        out.contains(
            "app.py, line 4, near placeholder name\nFirst, clean\nSecond, stickler\nChanged"
        ),
        "{out}"
    );
    assert!(
        out.contains("Standing in both, 2\nHeld, 1\nChanged, 1\n"),
        "{out}"
    );
}

#[test]
fn an_abstention_only_second_asking_reads_unjudged_and_counts_changed() {
    let dir = temp_dir("abstained-second");
    let judged = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "stickler", "0.9")
    );
    let abstained = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "stickler", "0.3")
    );
    let (code, out) = run_rejudged(
        &dir,
        &pair_json(
            "first asking",
            set_with(&judged),
            "second asking",
            set_with(&abstained),
        ),
    );
    assert_eq!(code, 0);
    assert!(
        out.contains(
            "app.py, line 4, near placeholder name\nFirst, stickler\nSecond, unjudged\nChanged"
        ),
        "an abstention is no third verdict, the line reads unjudged\n{out}"
    );
    assert!(out.contains("Changed, 1\n"), "{out}");
}

#[test]
fn an_input_unjudged_at_both_askings_stands_held() {
    let dir = temp_dir("both-unjudged");
    let (code, out) = run_rejudged(
        &dir,
        &pair_json("first asking", set_with(""), "second asking", set_with("")),
    );
    assert_eq!(code, 0);
    assert!(
        out.contains(
            "app.py, line 4, near placeholder name\nFirst, unjudged\nSecond, unjudged\nHeld"
        ),
        "{out}"
    );
    assert!(
        out.contains("Standing in both, 2\nHeld, 2\nChanged, 0\n"),
        "{out}"
    );
}

#[test]
fn an_input_standing_in_one_set_only_is_read_and_stands_in_no_group() {
    let dir = temp_dir("one-set-only");
    let wider = change_set_json(&[
        entry_json("app.py", FLAGS_DIFF, ""),
        entry_json("extra.py", FLAGS_DIFF, ""),
    ]);
    let (code, out) = run_rejudged(
        &dir,
        &pair_json("first asking", set_with(""), "second asking", wider),
    );
    assert_eq!(code, 0);
    assert!(out.contains("Inputs read, 6\n"), "{out}");
    assert!(out.contains("Standing in both, 2\n"), "{out}");
    assert!(
        !out.contains("extra.py"),
        "an input standing in one set only stands in no group\n{out}"
    );
}

#[test]
fn the_same_file_and_line_under_another_pattern_is_never_matched() {
    let dir = temp_dir("pattern-split");
    let swapped = change_set_json(&[entry_json("app.py", SWAPPED_DIFF, "")]);
    let (code, out) = run_rejudged(
        &dir,
        &pair_json("first asking", set_with(""), "second asking", swapped),
    );
    assert_eq!(code, 0);
    assert!(out.contains("Inputs read, 4\n"), "{out}");
    assert!(
        out.contains("Standing in both, 0\nHeld, 0\nChanged, 0\n"),
        "the pattern is part of the name, so no input stands in both\n{out}"
    );
    assert!(!out.contains("app.py, line"), "no group prints\n{out}");
}

#[test]
fn duplicate_names_pair_their_occurrences_in_order() {
    let dir = temp_dir("occurrences");
    let stickler_side = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "stickler", "0.9")
    );
    let clean_side = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "clean", "0.9")
    );
    let first = set_with(&stickler_side);
    let second = change_set_json(&[
        entry_json("app.py", FLAGS_DIFF, &clean_side),
        entry_json("app.py", FLAGS_DIFF, &stickler_side),
    ]);
    let (code, out) = run_rejudged(
        &dir,
        &pair_json("first asking", first, "second asking", second),
    );
    assert_eq!(code, 0);
    assert!(out.contains("Inputs read, 6\n"), "{out}");
    assert!(
        out.contains(
            "app.py, line 4, near placeholder name\nFirst, stickler\nSecond, clean\nChanged"
        ),
        "the first occurrence pairs with the first occurrence\n{out}"
    );
    assert!(
        out.contains("Standing in both, 2\nHeld, 1\nChanged, 1\n"),
        "{out}"
    );
}

#[test]
fn a_valid_pair_exits_zero_even_when_judgements_stand() {
    let dir = temp_dir("exit-zero");
    let verdicts = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "stickler", "0.9")
    );
    let (code, out) = run_rejudged(
        &dir,
        &pair_json(
            "first asking",
            set_with(&verdicts),
            "second asking",
            set_with(&verdicts),
        ),
    );
    assert_eq!(
        code, 0,
        "the ledger compares judgements and judges nothing itself\n{out}"
    );
    assert!(out.contains("First, stickler"));
}

#[test]
fn a_file_named_as_a_structural_word_forges_no_standalone_line() {
    let dir = temp_dir("hostile-name");
    let verdicts = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "stickler", "0.9")
    );
    let set = change_set_json(&[entry_json("Held", FLAGS_DIFF, &verdicts)]);
    let (code, out) = run_rejudged(
        &dir,
        &pair_json("first asking", set.clone(), "second asking", set),
    );
    assert_eq!(code, 0);
    assert!(
        out.contains("Held, line 4, near placeholder name"),
        "the identity stands fenced on its group line\n{out}"
    );
    assert_eq!(
        out.lines().filter(|line| *line == "Held").count(),
        2,
        "the only standalone Held lines are the two pair words\n{out}"
    );
    assert!(
        out.contains("Held, 2\n"),
        "the closing count stands apart\n{out}"
    );
}

#[test]
fn one_malformed_member_refuses_the_whole_pair() {
    let dir = temp_dir("whole");
    let good = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "stickler", "0.9")
    );
    let bad = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "stickler", "2")
    );
    let (code, out) = run_rejudged(
        &dir,
        &pair_json("good set", set_with(&good), "bad set", set_with(&bad)),
    );
    assert_eq!(code, 2);
    assert!(
        out.starts_with("Rejudged\nVerdict, invalid\nDefects, 1\n"),
        "{out}"
    );
    assert!(
        out.contains(
            "Defect, sets[1].change_set.files[0].verdicts[0].confidence must be from 0 to 1"
        ),
        "{out}"
    );
    assert!(!out.contains("Inputs read"), "no partial ledger prints");
    assert!(!out.contains("Standing in both"));
}

#[test]
fn the_loader_names_the_top_level_defects() {
    assert_eq!(
        load_rejudged(&parse("5").unwrap()),
        vec!["rejudged must be a JSON object".to_string()]
    );
    assert_eq!(
        load_rejudged(&parse("{}").unwrap()),
        vec!["sets is missing".to_string()]
    );
    assert_eq!(
        load_rejudged(&parse("{\"sets\":{}}").unwrap()),
        vec!["sets must be a list".to_string()]
    );
    let member = "{\"label\":\"x\",\"change_set\":{\"files\":[]}}";
    assert_eq!(
        load_rejudged(&parse(&format!("{{\"sets\":[{member}]}}")).unwrap()),
        vec!["sets must hold exactly two sets".to_string()],
        "one asking is no pair"
    );
    assert_eq!(
        load_rejudged(&parse(&format!("{{\"sets\":[{member},{member},{member}]}}")).unwrap()),
        vec!["sets must hold exactly two sets".to_string()],
        "three askings are no pair either"
    );
    assert_eq!(
        load_rejudged(&parse(&format!("{{\"sets\":[5,{member}]}}")).unwrap()),
        vec!["sets[0] must be an object".to_string()]
    );
    assert_eq!(
        load_rejudged(
            &parse(&format!(
                "{{\"sets\":[{{\"change_set\":{{\"files\":[]}}}},{member}]}}"
            ))
            .unwrap()
        ),
        vec!["sets[0].label is missing".to_string()]
    );
    assert_eq!(
        load_rejudged(&parse(&format!("{{\"sets\":[{{\"label\":\"x\"}},{member}]}}")).unwrap()),
        vec!["sets[0].change_set is missing".to_string()]
    );
    assert_eq!(
        load_rejudged(
            &parse(&format!(
                "{{\"sets\":[{{\"label\":\"x\",\"change_set\":5}},{member}]}}"
            ))
            .unwrap()
        ),
        vec!["sets[0].change_set must be a JSON object".to_string()]
    );
    assert_eq!(
        load_rejudged(
            &parse(&format!(
                "{{\"sets\":[{{\"label\":\"x\",\"change_set\":{{}}}},{member}]}}"
            ))
            .unwrap()
        ),
        vec!["sets[0].change_set.files is missing".to_string()]
    );
    assert_eq!(
        load_rejudged(
            &parse(&format!(
                "{{\"sets\":[{{\"label\":\"  \",\"change_set\":{{\"files\":[]}}}},{member}]}}"
            ))
            .unwrap()
        ),
        vec!["sets[0].label must be a non empty string".to_string()]
    );
}

#[test]
fn a_label_holding_a_line_break_is_refused_and_never_printed() {
    let dir = temp_dir("label-break");
    let text = format!(
        "{{\"sets\":[{{\"label\":\"forged\\nSecond, forged\",\"change_set\":{}}},{{\"label\":\"second asking\",\"change_set\":{}}}]}}",
        set_with(""),
        set_with("")
    );
    let (code, out) = run_rejudged(&dir, &text);
    assert_eq!(code, 2);
    assert!(
        out.contains("Defect, sets[0].label holds a line break"),
        "{out}"
    );
    assert!(
        !out.lines().any(|line| line == "Second, forged"),
        "the refused label forges no printed line"
    );
}

#[test]
fn a_label_holding_an_unpaired_surrogate_is_refused() {
    let dir = temp_dir("label-surrogate");
    let text = format!(
        "{{\"sets\":[{{\"label\":\"bad \\ud800 label\",\"change_set\":{}}},{{\"label\":\"second asking\",\"change_set\":{}}}]}}",
        set_with(""),
        set_with("")
    );
    let (code, out) = run_rejudged(&dir, &text);
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
    let (code, out, _err) = run(&["rejudged", &missing]);
    assert_eq!(code, 2);
    assert_eq!(
        out,
        "Rejudged\nVerdict, invalid\nDefects, 1\nDefect, the rejudged file could not be read\n"
    );
    let foreign = write(&dir, "foreign.json", "this is not json");
    let (code, out, _err) = run(&["rejudged", &foreign]);
    assert_eq!(code, 2);
    assert_eq!(
        out,
        "Rejudged\nVerdict, invalid\nDefects, 1\nDefect, the file does not hold a JSON rejudged pair\n"
    );
    let deep = write(&dir, "deep.json", &"[".repeat(10050));
    let (code, out, _err) = run(&["rejudged", &deep]);
    assert_eq!(code, 2);
    assert!(
        out.contains("Defect, the file does not hold a JSON rejudged pair"),
        "nesting past the boundary is a refusal, never a fall\n{out}"
    );
}

/// The fencing of FIX-CK-21 on the rejudged labels. A set
/// label holding a line break and a naming line's own
/// text is refused by the loader, and read through the
/// library the rendering prints the label as the empty
/// field, so the forged naming line never stands beside
/// the true one. Before the closure the rendering printed
/// the labels raw and the forged line stood.
#[test]
fn line_break_labels_print_as_empty_fields_through_the_library() {
    let text = pair_json(
        "evil\nFirst, forged",
        set_with(""),
        "evil\nSecond, forged",
        set_with(""),
    );
    let value = parse(&text).unwrap();
    assert!(!load_rejudged(&value).is_empty());
    let out = render_rejudged(&rejudged_of(&value));
    assert!(out.lines().any(|line| line == "First, "), "{out}");
    assert!(out.lines().any(|line| line == "Second, "), "{out}");
    assert!(!out.lines().any(|line| line == "First, forged"), "{out}");
    assert!(!out.lines().any(|line| line == "Second, forged"), "{out}");
    assert!(!out.contains("evil"), "{out}");
}

/// The fencing of FIX-CK-21 on the pair file name. A file
/// name holding a line break and the held word is refused
/// by the loader, and read through the library the pair
/// line prints the file as the empty field, so the line
/// does not split and no held word stands on its own
/// line from the name. Before the closure the pair line
/// split at the name.
#[test]
fn a_line_break_file_name_splits_no_pair_line_through_the_library() {
    let verdicts = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "stickler", "0.9")
    );
    let set = change_set_json(&[entry_json("evil.py\nHeld", FLAGS_DIFF, &verdicts)]);
    let text = pair_json("first set", set.clone(), "second set", set);
    let value = parse(&text).unwrap();
    assert!(!load_rejudged(&value).is_empty());
    let out = render_rejudged(&rejudged_of(&value));
    assert!(
        out.lines()
            .any(|line| line == ", line 4, near placeholder name"),
        "{out}"
    );
    // Both flags of the set stand as held pairs, so the
    // held word stands as a line exactly twice, once per
    // pair, and the name mints no third standing of it.
    assert_eq!(
        out.lines().filter(|line| *line == "Held").count(),
        2,
        "the held word stands only as the pairs' own lines\n{out}"
    );
}

/// The handed counts of FIX-CK-21 on the rejudged
/// rendering. CK-14 states the held and changed counts as
/// the counts of the pairs the reading states, and a
/// reading built by hand prints them in that relation, so
/// the counts it was handed never stand. The inputs read
/// count counts the inputs of the two sets and not the
/// pairs stated, so it stands as handed. Before the
/// closure the rendering printed the handed held and
/// changed counts whole.
#[test]
fn a_hand_built_rejudged_prints_the_counts_of_its_pairs() {
    let pair = |held: bool| ComparedInput {
        file: "app.py".to_string(),
        line: 4,
        pattern: "near placeholder name".to_string(),
        first: "stickler",
        second: if held { "stickler" } else { "clean" },
        held,
    };
    let reading = Rejudged {
        first_label: "first set".to_string(),
        second_label: "second set".to_string(),
        pairs: vec![pair(true), pair(false)],
        inputs_read: 4,
        held: 555,
        changed: 666,
    };
    let out = render_rejudged(&reading);
    assert!(!out.contains("555"), "{out}");
    assert!(!out.contains("666"), "{out}");
    assert!(out.contains("Inputs read, 4\n"), "{out}");
    assert!(out.contains("Standing in both, 2\n"), "{out}");
    assert!(out.contains("Held, 1\n"), "{out}");
    assert!(out.contains("Changed, 1\n"), "{out}");
}
