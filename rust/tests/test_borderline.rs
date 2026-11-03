//! Pins for the borderline flagging of CK-7.
//!
//! A flagged line draws no judgement. The flag names the
//! pattern, carries the line as written, and carries the
//! reading the flag computes, and a line that raises any
//! judgement under the six shapes is never flagged.
//! Translated from tests/test_borderline.py.

use stickler::checker::{check_diff_detailed, check_diff_flagged};

fn flagged(
    added: &[&str],
) -> (
    Vec<(&'static str, stickler::checker::Objection)>,
    Vec<stickler::checker::Flag>,
) {
    let mut diff = "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,9 @@\n".to_string();
    for line in added {
        diff.push_str(&format!("+{line}\n"));
    }
    check_diff_flagged(&diff)
}

fn shapes(judgements: &[(&'static str, stickler::checker::Objection)]) -> Vec<&'static str> {
    judgements.iter().map(|(shape, _)| *shape).collect()
}

#[test]
fn assignment_near_name_flags_with_the_drawn_name() {
    let (_judgements, flags) = flagged(&["data = load_entries()"]);
    assert_eq!(flags.len(), 1);
    assert_eq!(flags[0].pattern, "near placeholder name");
    assert_eq!(flags[0].reading, "drawn name entries");
    assert_eq!(flags[0].content, "data = load_entries()");
}

#[test]
fn near_name_without_a_drawn_name() {
    let (_judgements, flags) = flagged(&["temp = left + right"]);
    assert_eq!(flags[0].reading, "no drawn name");
}

#[test]
fn function_name_near_placeholder() {
    let (_judgements, flags) = flagged(&["def tmp(rows):"]);
    assert_eq!(flags.len(), 1);
    assert_eq!(flags[0].pattern, "near placeholder name");
    assert_eq!(flags[0].reading, "no drawn name");
}

#[test]
fn tuple_member_near_placeholder() {
    let (_judgements, flags) = flagged(&["data, extra = left, right"]);
    assert_eq!(flags.len(), 1);
    assert_eq!(flags[0].pattern, "near placeholder name");
}

#[test]
fn annotated_target_near_placeholder() {
    let (_judgements, flags) = flagged(&["tmp: list = []"]);
    assert_eq!(flags.len(), 1);
    assert_eq!(flags[0].reading, "drawn name items");
}

#[test]
fn the_named_placeholders_are_judged_not_flagged() {
    for line in ["foo = 5", "data2 = 5", "thing = []"] {
        let (judgements, flags) = flagged(&[line]);
        assert_eq!(shapes(&judgements), vec!["placeholder name"], "line {line}");
        assert!(flags.is_empty(), "line {line}");
    }
}

#[test]
fn a_name_outside_both_lists_is_neither() {
    let (judgements, flags) = flagged(&["datum = 5"]);
    assert!(judgements.is_empty());
    assert!(flags.is_empty());
}

#[test]
fn broad_handler_flags() {
    let (_judgements, flags) = flagged(&["    except BaseException:"]);
    assert_eq!(flags.len(), 1);
    assert_eq!(flags[0].pattern, "broad handler");
    assert_eq!(flags[0].reading, "handler names BaseException");
}

#[test]
fn broad_handler_with_a_binding_flags() {
    let (_judgements, flags) = flagged(&["except BaseException as err:"]);
    assert_eq!(flags.len(), 1);
}

#[test]
fn a_named_handler_is_neither_judged_nor_flagged() {
    let (judgements, flags) = flagged(&["    except Exception:"]);
    assert!(judgements.is_empty());
    assert!(flags.is_empty());
}

#[test]
fn base_exception_group_is_not_the_broad_handler() {
    let (_judgements, flags) = flagged(&["    except BaseExceptionGroup:"]);
    assert!(flags.is_empty());
}

#[test]
fn shared_member_default_kinds() {
    let cases = [
        ("def f(cache=([],)):", "default holds a list literal"),
        (
            "def f(pair=({\"a\": 1}, 2)):",
            "default holds a dict literal",
        ),
        ("def f(seen=({1, 2},)):", "default holds a set literal"),
    ];
    for (line, reading) in cases {
        let (judgements, flags) = flagged(&[line]);
        assert!(judgements.is_empty(), "line {line}");
        assert_eq!(flags.len(), 1, "line {line}");
        assert_eq!(flags[0].pattern, "shared member default", "line {line}");
        assert_eq!(flags[0].reading, reading, "line {line}");
    }
}

#[test]
fn a_tuple_of_plain_values_is_not_flagged() {
    let (judgements, flags) = flagged(&["def f(span=(0, 30)):"]);
    assert!(judgements.is_empty());
    assert!(flags.is_empty());
}

#[test]
fn a_string_holding_brackets_is_not_a_member() {
    // A naive bracket count ends the tuple inside the string.
    // The walk is quote aware, so the member is the string and
    // no container member stands.
    let (judgements, flags) = flagged(&["def f(pair=(\"]\",)):"]);
    assert!(judgements.is_empty());
    assert!(flags.is_empty());
}

#[test]
fn a_tuple_call_default_is_judged_not_flagged() {
    let (judgements, flags) = flagged(&["def f(pair=tuple([1])):"]);
    assert_eq!(shapes(&judgements), vec!["call default"]);
    assert!(flags.is_empty());
}

#[test]
fn a_judged_line_is_never_flagged() {
    let (judgements, flags) = flagged(&["tmp = print(x)"]);
    assert_eq!(shapes(&judgements), vec!["print call"]);
    assert!(flags.is_empty());
}

#[test]
fn a_mutable_default_on_the_line_blocks_the_flag() {
    let (judgements, flags) = flagged(&["def f(a=[], pair=([],)):"]);
    assert_eq!(shapes(&judgements), vec!["mutable default"]);
    assert!(flags.is_empty());
}

#[test]
fn a_class_attribute_line_is_judged_not_flagged() {
    let diff = "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,2 @@\n class C:\n+    data = []\n";
    let (judgements, flags) = check_diff_flagged(diff);
    assert_eq!(shapes(&judgements), vec!["class attribute"]);
    assert!(flags.is_empty());
}

#[test]
fn at_most_one_flag_stands_on_a_line() {
    let (_judgements, flags) = flagged(&["def temp(cache=([],)):"]);
    assert_eq!(flags.len(), 1);
    // The name pattern is read first in the stated order.
    assert_eq!(flags[0].pattern, "near placeholder name");
}

#[test]
fn flags_stand_in_line_order() {
    let (_judgements, flags) = flagged(&["    except BaseException:", "data = load_entries()"]);
    let patterns: Vec<&str> = flags.iter().map(|flag| flag.pattern.as_str()).collect();
    assert_eq!(patterns, vec!["broad handler", "near placeholder name"]);
    assert!(flags[0].line < flags[1].line);
}

#[test]
fn flagged_judgements_match_the_detailed_walk() {
    let diff = "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,3 @@\n+print(x)\n+data = load_entries()\n";
    let (judgements, _flags) = check_diff_flagged(diff);
    assert_eq!(judgements, check_diff_detailed(diff));
}
