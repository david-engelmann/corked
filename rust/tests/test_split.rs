//! Pins for CK-15 and CK-16, the split file and reading.
//!
//! The split pins build their files by hand, so each one
//! fails under the negation of the rule it names, the
//! route subsets read through the standing CK-8 outcome
//! helper, the order the file names the routes in, the
//! route membership defect, the whole or nothing
//! refusal, and the closing counts. The golden pins hold
//! the example of CK-16 byte identical against its
//! stored rendering, and the reconciliation pin holds
//! the joint judgements of the example against the
//! judgements the set review reads over the same pilot
//! set and against the summary of its recorded
//! rendering. Every pin runs offline on recorded and
//! constructed entries, and no pin makes a live call.

use std::path::{Path, PathBuf};

use stickler::changeset::review_changeset;
use stickler::cli::cli_main;
use stickler::json::{parse, JsonValue};
use stickler::render::render_split;
use stickler::split::{load_split, split_of};
use stickler::verdicts::line_outcome;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn examples() -> PathBuf {
    root().join("examples")
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "stickler-split-{}-{}-{}",
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

fn run_split(dir: &Path, text: &str) -> (i32, String) {
    let path = write(dir, "split.json", text);
    let (code, out, _err) = run(&["split", &path]);
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

/// One recorded verdict as JSON text under a named model,
/// the confidence given raw so a pin can state one at the
/// abstention boundary.
fn verdict_json(line: usize, pattern: &str, model: &str, choice: &str, confidence: &str) -> String {
    format!(
        "{{\"line\":{line},\"pattern\":\"{}\",\"model\":\"{}\",\"choice\":\"{choice}\",\"confidence\":{confidence}}}",
        jstr(pattern),
        jstr(model)
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

/// One split file text over two named routes and one set.
fn split_json(first: &str, second: &str, change_set: String) -> String {
    format!(
        "{{\"routes\":[\"{}\",\"{}\"],\"change_set\":{}}}",
        jstr(first),
        jstr(second),
        change_set
    )
}

/// A diff flagging the near placeholder name at line 4 and
/// the broad handler at line 5.
const FLAGS_DIFF: &str =
    "--- a/app.py\n+++ b/app.py\n@@ -1,1 +1,3 @@\n+data = load_entries()\n+    except BaseException:\n";

/// One entry over the two flags, verdicts appended raw.
fn set_with(verdicts: &str) -> String {
    change_set_json(&[entry_json("app.py", FLAGS_DIFF, verdicts)])
}

const JEV: &str = "jev-1.13.0";
const CHAT: &str = "openai/gpt-6-luna";

#[test]
fn the_example_matches_its_golden_through_the_command() {
    let path = examples().join("split.json");
    let (code, out, _err) = run(&["split", path.to_str().unwrap()]);
    assert_eq!(code, 0);
    let golden = std::fs::read_to_string(examples().join("split.txt")).unwrap();
    assert_eq!(out, golden);
}

#[test]
fn the_example_matches_its_golden_through_the_library() {
    let text = std::fs::read_to_string(examples().join("split.json")).unwrap();
    let value = parse(&text).unwrap();
    assert_eq!(load_split(&value), Vec::<String>::new());
    let golden = std::fs::read_to_string(examples().join("split.txt")).unwrap();
    assert_eq!(render_split(&split_of(&value)), golden);
}

#[test]
fn the_example_reads_the_stated_routes_counts_and_flags() {
    let value = load_json(&examples().join("split.json"));
    let reading = split_of(&value);
    assert_eq!(reading.first_route, JEV);
    assert_eq!(reading.second_route, CHAT);
    assert_eq!(reading.flags.len(), 12);
    assert_eq!(reading.routes_agree, 10);
    assert_eq!(reading.routes_differ, 2);
    assert_eq!(reading.joint_differs, 2);
    assert_eq!(
        reading.routes_agree + reading.routes_differ,
        reading.flags.len(),
        "the route counts close over the flags read"
    );
    let names: Vec<(&str, usize)> = reading
        .flags
        .iter()
        .map(|flag| (flag.file.as_str(), flag.line))
        .collect();
    assert_eq!(
        names,
        vec![
            ("harvest.py", 6),
            ("harvest.py", 7),
            ("harvest.py", 12),
            ("harvest.py", 18),
            ("windows.py", 6),
            ("windows.py", 10),
            ("windows.py", 14),
            ("windows.py", 18),
            ("sorting.py", 6),
            ("sorting.py", 11),
            ("sorting.py", 12),
            ("sorting.py", 15),
        ],
        "the flags stand in the order the set review states them"
    );
}

#[test]
fn the_composition_flags_carry_their_three_judgements() {
    let value = load_json(&examples().join("split.json"));
    let reading = split_of(&value);
    let judgements = |file: &str, line: usize| {
        let flag = reading
            .flags
            .iter()
            .find(|flag| flag.file == file && flag.line == line)
            .unwrap_or_else(|| panic!("the flag {file} line {line} stands"));
        (flag.first, flag.second, flag.joint)
    };
    // The flag the pilot Jev verdict abstains on at 0.2
    // while the comparison verdict chose stickler at
    // 0.88, so the joint judgement is a composition the
    // Jev route alone never reached.
    assert_eq!(
        judgements("sorting.py", 11),
        ("unjudged", "stickler", "stickler")
    );
    // The flag whose only verdict abstains and whose
    // comparison call failed, unjudged under all three.
    assert_eq!(
        judgements("harvest.py", 6),
        ("unjudged", "unjudged", "unjudged")
    );
    // The flag holding a verdict from the Jev route only,
    // judged by that route and jointly, and unjudged
    // under the route that holds no verdict on it.
    assert_eq!(
        judgements("harvest.py", 7),
        ("stickler", "unjudged", "stickler")
    );
}

#[test]
fn the_joint_judgements_reconcile_with_the_pilot_review() {
    let pilot = load_json(&examples().join("borderline_review.json"));
    let value = load_json(&examples().join("split.json"));
    let reading = split_of(&value);
    let mut joint_words: Vec<&str> = Vec::new();
    let mut at = 0;
    for result in review_changeset(&pilot) {
        for block in &result.blocks {
            let flag = &reading.flags[at];
            assert_eq!(flag.file, result.name);
            assert_eq!(flag.line, block.flag.line);
            assert_eq!(
                flag.joint,
                line_outcome(&block.verdicts),
                "the joint judgement of {} line {} is the judgement \
                 the set review reads over the same flag",
                result.name,
                block.flag.line
            );
            joint_words.push(flag.joint);
            at += 1;
        }
    }
    assert_eq!(at, reading.flags.len());
    let count = |word: &str| joint_words.iter().filter(|held| **held == word).count();
    // The counts the pilot review's own summary states
    // over the same twelve flags.
    let summary = std::fs::read_to_string(examples().join("borderline_review.txt")).unwrap();
    assert!(summary.contains("Verdict lines, stickler 11, clean 0"));
    assert!(summary.contains("Unjudged, 1"));
    assert_eq!(count("stickler"), 11);
    assert_eq!(count("clean"), 0);
    assert_eq!(count("unjudged"), 1);
}

#[test]
fn a_flag_with_verdicts_from_one_route_only_reads_unjudged_under_the_other() {
    let verdicts = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", JEV, "stickler", "0.9")
    );
    let text = split_json(JEV, CHAT, set_with(&verdicts));
    let value = parse(&text).unwrap();
    assert_eq!(load_split(&value), Vec::<String>::new());
    let reading = split_of(&value);
    assert_eq!(reading.flags.len(), 2);
    let judged = &reading.flags[0];
    assert_eq!(
        (judged.first, judged.second, judged.joint),
        ("stickler", "unjudged", "stickler")
    );
    // The flag holding no verdict at all reads unjudged
    // under each route and jointly, and the two routes
    // agree on it.
    let bare = &reading.flags[1];
    assert_eq!(
        (bare.first, bare.second, bare.joint),
        ("unjudged", "unjudged", "unjudged")
    );
    assert_eq!(reading.routes_agree, 1);
    assert_eq!(reading.routes_differ, 1);
    assert_eq!(reading.joint_differs, 1);
}

#[test]
fn the_routes_are_read_in_the_order_the_file_names_them() {
    let verdicts = format!(
        "[{},{}]",
        verdict_json(4, "near placeholder name", JEV, "clean", "0.9"),
        verdict_json(4, "near placeholder name", CHAT, "stickler", "0.9")
    );
    let forward = split_of(&parse(&split_json(JEV, CHAT, set_with(&verdicts))).unwrap());
    assert_eq!(
        (forward.flags[0].first, forward.flags[0].second),
        ("clean", "stickler"),
        "the first named route reads first"
    );
    let swapped = split_of(&parse(&split_json(CHAT, JEV, set_with(&verdicts))).unwrap());
    assert_eq!(
        (swapped.flags[0].first, swapped.flags[0].second),
        ("stickler", "clean"),
        "swapping the named routes swaps the two judgements"
    );
    assert_eq!(swapped.first_route, CHAT);
    assert_eq!(swapped.second_route, JEV);
}

#[test]
fn an_abstention_only_route_reads_unjudged_and_the_boundary_records() {
    // A confidence of exactly one half records as a
    // verdict under CK-7, and one below it abstains.
    let verdicts = format!(
        "[{},{}]",
        verdict_json(4, "near placeholder name", JEV, "clean", "0.5"),
        verdict_json(4, "near placeholder name", CHAT, "stickler", "0.49")
    );
    let value = parse(&split_json(JEV, CHAT, set_with(&verdicts))).unwrap();
    assert_eq!(load_split(&value), Vec::<String>::new());
    let flag = &split_of(&value).flags[0];
    assert_eq!(
        (flag.first, flag.second, flag.joint),
        ("clean", "unjudged", "clean")
    );
}

#[test]
fn a_third_model_verdict_is_a_defect_of_the_split_file() {
    let verdicts = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "third-model", "stickler", "0.9")
    );
    let value = parse(&split_json(JEV, CHAT, set_with(&verdicts))).unwrap();
    assert_eq!(
        load_split(&value),
        vec!["files[0].verdicts[0].model names no route of the split".to_string()]
    );
}

#[test]
fn a_third_model_verdict_naming_no_flag_draws_both_defects() {
    // The verdict stands in the form CK-8 states and is
    // read in its posture, so the membership defect
    // stands beside the defect the change set loader
    // names for a verdict naming no borderline line.
    let verdicts = format!(
        "[{}]",
        verdict_json(9, "broad handler", "third-model", "stickler", "0.9")
    );
    let value = parse(&split_json(JEV, CHAT, set_with(&verdicts))).unwrap();
    let defects = load_split(&value);
    assert!(
        defects.contains(&"files[0].verdicts[0] names no borderline line".to_string()),
        "the change set defect stands, {defects:?}"
    );
    assert!(
        defects.contains(&"files[0].verdicts[0].model names no route of the split".to_string()),
        "the membership defect stands, {defects:?}"
    );
}

#[test]
fn a_refused_verdict_draws_no_membership_defect() {
    // A verdict whose model the change set loader already
    // refuses draws no second defect from the route rule.
    let verdicts = "[{\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"\",\"choice\":\"stickler\",\"confidence\":0.9}]";
    let value = parse(&split_json(JEV, CHAT, set_with(verdicts))).unwrap();
    let defects = load_split(&value);
    assert_eq!(
        defects,
        vec!["files[0].verdicts[0].model must be a non empty string".to_string()]
    );
}

#[test]
fn a_verdict_refused_over_its_choice_draws_no_membership_defect() {
    // The choice stands outside the two labels, so the
    // change set loader already refuses the verdict and
    // the route rule names no second defect for it.
    let verdicts = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "third-model", "bogus", "0.9")
    );
    let value = parse(&split_json(JEV, CHAT, set_with(&verdicts))).unwrap();
    assert_eq!(
        load_split(&value),
        vec!["files[0].verdicts[0].choice must be one of stickler, clean".to_string()]
    );
}

#[test]
fn a_verdict_refused_over_its_confidence_draws_no_membership_defect() {
    // The confidence stands outside the range from 0 to
    // 1, so the change set loader already refuses the
    // verdict and the route rule names no second defect.
    let verdicts = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "third-model", "stickler", "1.5")
    );
    let value = parse(&split_json(JEV, CHAT, set_with(&verdicts))).unwrap();
    assert_eq!(
        load_split(&value),
        vec!["files[0].verdicts[0].confidence must be from 0 to 1".to_string()]
    );
}

#[test]
fn a_verdict_missing_its_choice_field_draws_no_membership_defect() {
    // The choice field is missing, so the change set
    // loader already refuses the verdict and the route
    // rule names no second defect for it.
    let verdicts = "[{\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"third-model\",\"confidence\":0.9}]";
    let value = parse(&split_json(JEV, CHAT, set_with(verdicts))).unwrap();
    assert_eq!(
        load_split(&value),
        vec!["files[0].verdicts[0].choice is missing".to_string()]
    );
}

#[test]
fn a_verdict_missing_its_confidence_field_draws_no_membership_defect() {
    // The confidence field is missing, so the change set
    // loader already refuses the verdict and the route
    // rule names no second defect for it.
    let verdicts = "[{\"line\":4,\"pattern\":\"near placeholder name\",\"model\":\"third-model\",\"choice\":\"stickler\"}]";
    let value = parse(&split_json(JEV, CHAT, set_with(verdicts))).unwrap();
    assert_eq!(
        load_split(&value),
        vec!["files[0].verdicts[0].confidence is missing".to_string()]
    );
}

#[test]
fn a_verdict_with_a_float_line_draws_no_membership_defect() {
    // The line stands in float form, so the change set
    // loader already refuses the verdict and the route
    // rule names no second defect for it.
    let verdicts = "[{\"line\":4.0,\"pattern\":\"near placeholder name\",\"model\":\"third-model\",\"choice\":\"stickler\",\"confidence\":0.9}]";
    let value = parse(&split_json(JEV, CHAT, set_with(verdicts))).unwrap();
    assert_eq!(
        load_split(&value),
        vec!["files[0].verdicts[0].line must be a positive integer".to_string()]
    );
}

#[test]
fn a_verdict_refused_over_its_pattern_draws_no_membership_defect() {
    // The pattern stands outside the three names, so the
    // change set loader already refuses the verdict and
    // the route rule names no second defect for it.
    let verdicts = format!(
        "[{}]",
        verdict_json(4, "wide net", "third-model", "stickler", "0.9")
    );
    let value = parse(&split_json(JEV, CHAT, set_with(&verdicts))).unwrap();
    assert_eq!(
        load_split(&value),
        vec!["files[0].verdicts[0].pattern must be one of near placeholder name, broad handler, shared member default".to_string()]
    );
}

#[test]
fn a_repeated_foreign_verdict_draws_one_membership_defect() {
    // Two verdicts of one foreign model stand on one
    // line. The first stands in the form CK-8 states and
    // draws the membership defect, and the second is
    // refused by the change set loader as a repeat and
    // draws no second defect from the route rule, so the
    // file draws exactly the two defects named here.
    let one = verdict_json(4, "near placeholder name", "third-model", "stickler", "0.9");
    let verdicts = format!("[{one},{one}]");
    let value = parse(&split_json(JEV, CHAT, set_with(&verdicts))).unwrap();
    assert_eq!(
        load_split(&value),
        vec![
            "files[0].verdicts[1] repeats a verdict for the same line and model".to_string(),
            "files[0].verdicts[0].model names no route of the split".to_string(),
        ]
    );
}

#[test]
fn the_third_model_defect_refuses_the_reading_through_the_command() {
    let dir = temp_dir("third-model");
    let verdicts = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", "third-model", "stickler", "0.9")
    );
    let (code, out) = run_split(&dir, &split_json(JEV, CHAT, set_with(&verdicts)));
    assert_eq!(code, 2);
    assert!(out.starts_with("Split\nVerdict, invalid\n"), "{out}");
    assert!(
        out.contains("Defect, files[0].verdicts[0].model names no route of the split"),
        "{out}"
    );
    assert!(!out.contains("Flags read"), "no reading renders, {out}");
}

#[test]
fn the_loader_names_the_split_level_defects() {
    let not_object = parse("[1]").unwrap();
    assert_eq!(
        load_split(&not_object),
        vec!["split must be a JSON object".to_string()]
    );
    let no_routes = parse("{\"change_set\":{\"files\":[]}}").unwrap();
    assert_eq!(
        load_split(&no_routes),
        vec!["routes is missing".to_string()]
    );
    let routes_not_list = parse("{\"routes\":4,\"change_set\":{\"files\":[]}}").unwrap();
    assert_eq!(
        load_split(&routes_not_list),
        vec!["routes must be a list".to_string()]
    );
    let set = set_with("");
    let one_route = parse(&format!("{{\"routes\":[\"{JEV}\"],\"change_set\":{set}}}")).unwrap();
    assert_eq!(
        load_split(&one_route),
        vec!["routes must hold exactly two routes".to_string()]
    );
    let blank_route = parse(&split_json("", CHAT, set.clone())).unwrap();
    assert_eq!(
        load_split(&blank_route),
        vec!["routes[0] must be a non empty string".to_string()]
    );
    let broken_route = parse(&split_json(JEV, "openai/gpt-6-luna\nnext", set.clone())).unwrap();
    assert_eq!(
        load_split(&broken_route),
        vec!["routes[1] holds a line break".to_string()]
    );
    let same_route = parse(&split_json(JEV, JEV, set.clone())).unwrap();
    assert_eq!(
        load_split(&same_route),
        vec!["routes must name two different model strings".to_string()]
    );
    let no_set = parse(&format!("{{\"routes\":[\"{JEV}\",\"{CHAT}\"]}}")).unwrap();
    assert_eq!(
        load_split(&no_set),
        vec!["change_set is missing".to_string()]
    );
    let foreign_set = parse(&split_json(JEV, CHAT, "42".to_string())).unwrap();
    assert_eq!(
        load_split(&foreign_set),
        vec!["change set must be a JSON object".to_string()],
        "the member defect stands as the change set loader states it"
    );
}

#[test]
fn a_malformed_member_verdict_refuses_the_whole_reading() {
    let dir = temp_dir("whole-or-nothing");
    let verdicts = format!(
        "[{}]",
        verdict_json(4, "near placeholder name", JEV, "stickler", "1.5")
    );
    let (code, out) = run_split(&dir, &split_json(JEV, CHAT, set_with(&verdicts)));
    assert_eq!(code, 2);
    assert!(
        out.contains("Defect, files[0].verdicts[0].confidence must be from 0 to 1"),
        "{out}"
    );
    assert!(
        !out.contains("Flags read"),
        "no partial split prints, {out}"
    );
    assert!(!out.contains("Joint,"), "no flag group prints, {out}");
}

#[test]
fn the_split_command_refuses_an_unreadable_file_and_a_foreign_file() {
    let dir = temp_dir("refusals");
    let missing = dir.join("nope.json").to_string_lossy().into_owned();
    let (code, out, _err) = run(&["split", &missing]);
    assert_eq!(code, 2);
    assert!(
        out.contains("Defect, the split file could not be read"),
        "{out}"
    );
    let (code, out) = run_split(&dir, "not json at all");
    assert_eq!(code, 2);
    assert!(
        out.contains("Defect, the file does not hold a JSON split"),
        "{out}"
    );
}

#[test]
fn a_route_string_carrying_a_structural_line_mints_no_line() {
    // The route strings stand behind the fixed words of
    // the head lines, so a route named as a closing line
    // of the reading prints only there and the closing
    // lines stand once, with their true counts.
    let verdicts = format!(
        "[{},{}]",
        verdict_json(
            4,
            "near placeholder name",
            "Flags read, 12",
            "stickler",
            "0.9"
        ),
        verdict_json(
            4,
            "near placeholder name",
            "Routes agree, 10",
            "clean",
            "0.9"
        )
    );
    let text = split_json("Flags read, 12", "Routes agree, 10", set_with(&verdicts));
    let value = parse(&text).unwrap();
    assert_eq!(load_split(&value), Vec::<String>::new());
    let rendered = render_split(&split_of(&value));
    assert!(
        rendered.contains("First route, Flags read, 12\n"),
        "{rendered}"
    );
    assert_eq!(
        rendered
            .lines()
            .filter(|line| *line == "Flags read, 2")
            .count(),
        1,
        "the true closing line stands once, {rendered}"
    );
    assert!(
        !rendered.lines().any(|line| line == "Flags read, 12"),
        "no minted closing line stands, {rendered}"
    );
}

#[test]
fn the_reading_writes_nothing() {
    let dir = temp_dir("writes-nothing");
    let (code, _out) = run_split(&dir, &split_json(JEV, CHAT, set_with("")));
    assert_eq!(code, 0);
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, vec!["split.json".to_string()]);
}

#[test]
fn the_contract_states_the_split_obligations() {
    let text = std::fs::read_to_string(root().join("CONTRACT.md")).unwrap();
    for marker in [
        "CK-15. This obligation states the split file and its loader.",
        "CK-16. This obligation states the split reading, its rendering, its command, and its example.",
        "model names no route of the split",
        "Joint differs from a route",
    ] {
        assert!(text.contains(marker), "the contract states {marker}");
    }
}
