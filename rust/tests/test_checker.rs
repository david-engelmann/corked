//! Pins for CK-3 and CK-5. The checker shapes, their rewrites,
//! and their limits. Translated from tests/test_checker.py.

use stickler::checker::{check_diff_text, Objection, PLACEHOLDERS};
use stickler::json::{JString, JsonValue};
use stickler::review::{validate_review, verdict_for};

fn diff(lines: &[&str]) -> String {
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

fn owned(lines: &[&str]) -> Vec<String> {
    lines.iter().map(|line| line.to_string()).collect()
}

fn join_lines(lines: &[String]) -> String {
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

fn added(contents: &[&str]) -> String {
    let mut lines = owned(&["--- a/app.py", "+++ b/app.py", "@@ -1,1 +1,1 @@"]);
    for content in contents {
        lines.push(format!("+{content}"));
    }
    join_lines(&lines)
}

fn raw(lines: &[&str]) -> String {
    let mut all = owned(&["--- a/app.py", "+++ b/app.py", "@@ -1,1 +1,1 @@"]);
    all.extend(owned(lines));
    join_lines(&all)
}

fn categories(objections: &[Objection]) -> Vec<&str> {
    objections.iter().map(|o| o.category.as_str()).collect()
}

fn rewrites(objections: &[Objection]) -> Vec<&str> {
    objections.iter().map(|o| o.rewrite.as_str()).collect()
}

#[test]
fn bare_except_fires_with_a_handler_naming_exception() {
    let objections = check_diff_text(&added(&["    except:"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].category, "smell");
    assert_eq!(objections[0].rewrite, "    except Exception:");
}

#[test]
fn typed_except_does_not_fire() {
    assert!(check_diff_text(&added(&["    except ValueError:"])).is_empty());
}

#[test]
fn print_call_fires_with_a_logging_rewrite() {
    let objections = check_diff_text(&added(&["    print(total)"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].category, "smell");
    assert_eq!(objections[0].rewrite, "    logger.debug(total)");
}

#[test]
fn logging_call_does_not_fire() {
    assert!(check_diff_text(&added(&["    logger.debug(total)"])).is_empty());
}

#[test]
fn print_inside_a_longer_name_or_an_attribute_does_not_fire() {
    assert!(check_diff_text(&added(&["    sprint(total)"])).is_empty());
    assert!(check_diff_text(&added(&["    printer(total)"])).is_empty());
    assert!(check_diff_text(&added(&["    out.print(total)"])).is_empty());
}

#[test]
fn mutable_defaults_fire_with_the_none_sentinel_rewrite() {
    let cases = [
        ("def load(items=[]):", "def load(items=None):"),
        ("def load(cache={}):", "def load(cache=None):"),
        ("def load(seen=set()):", "def load(seen=None):"),
    ];
    for (line, rewrite) in cases {
        let objections = check_diff_text(&added(&[line]));
        assert_eq!(objections.len(), 1, "line {line}");
        assert_eq!(objections[0].category, "structure");
        assert_eq!(objections[0].rewrite, rewrite, "line {line}");
    }
}

#[test]
fn fresh_defaults_do_not_fire() {
    assert!(check_diff_text(&added(&["def load(items=None):"])).is_empty());
    assert!(check_diff_text(&added(&["def load(count=0):"])).is_empty());
    assert!(check_diff_text(&added(&["def load(name='x'):"])).is_empty());
}

#[test]
fn placeholder_assignment_draws_its_name_from_a_simple_call() {
    let objections = check_diff_text(&added(&["foo = load_orders()"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].category, "naming");
    assert_eq!(objections[0].rewrite, "orders = load_orders()");
}

#[test]
fn placeholder_assignment_drops_only_the_leading_word() {
    let objections = check_diff_text(&added(&["bar = fetch_user_records()"]));
    assert_eq!(objections[0].rewrite, "user_records = fetch_user_records()");
}

#[test]
fn placeholder_assignment_keeps_a_one_word_callee_as_the_name() {
    let objections = check_diff_text(&added(&["foo = load()"]));
    assert_eq!(objections[0].rewrite, "load = load()");
}

#[test]
fn placeholder_assignment_draws_its_name_from_literals() {
    let cases = [
        ("foo = \"hello\"", "text = \"hello\""),
        ("foo = [1, 2]", "items = [1, 2]"),
        ("foo = {'a': 1}", "mapping = {'a': 1}"),
        ("temp2 = 42", "number = 42"),
    ];
    for (line, rewrite) in cases {
        let objections = check_diff_text(&added(&[line]));
        assert_eq!(objections.len(), 1, "line {line}");
        assert_eq!(objections[0].rewrite, rewrite, "line {line}");
    }
}

#[test]
fn placeholder_assignment_draws_number_over_digits_of_any_script() {
    let cases = [
        ("temp2 = \u{0663}", "number = \u{0663}"),
        ("temp2 = \u{ff13}", "number = \u{ff13}"),
        ("temp2 = \u{0969}", "number = \u{0969}"),
        ("data2 = \u{0663}", "number = \u{0663}"),
        ("temp2 = \u{0663}.\u{0665}", "number = \u{0663}.\u{0665}"),
        ("temp2 = -\u{0663}", "number = -\u{0663}"),
        ("temp2 = \u{0664}\u{0662}", "number = \u{0664}\u{0662}"),
    ];
    for (line, rewrite) in cases {
        let objections = check_diff_text(&added(&[line]));
        assert_eq!(objections.len(), 1, "line {line}");
        assert_eq!(objections[0].rewrite, rewrite, "line {line}");
    }
}

#[test]
fn placeholder_assignment_draws_no_number_over_a_non_decimal_digit() {
    // The superscript two is category No, not Nd, and a digit
    // followed by a letter is no number either. Both draw no
    // name and the rewrite instructs the rename, as the Python
    // reads them.
    for line in ["temp2 = \u{00b2}", "temp2 = \u{0663}a"] {
        let objections = check_diff_text(&added(&[line]));
        assert_eq!(objections.len(), 1, "line {line}");
        assert!(
            objections[0].rewrite.starts_with("Rename temp2 to a name"),
            "line {line}"
        );
    }
}

#[test]
fn placeholder_rewrite_instructs_when_no_name_can_be_drawn() {
    let objections = check_diff_text(&added(&["foo = other"]));
    assert_eq!(
        objections[0].rewrite,
        "Rename foo to a name that says what the value holds \
         and keep the rest of the line as written"
    );
}

#[test]
fn placeholder_rewrite_instructs_when_the_drawn_name_is_a_placeholder() {
    let objections = check_diff_text(&added(&["foo = thing()"]));
    assert!(objections[0].rewrite.starts_with("Rename foo to a name"));
}

#[test]
fn placeholder_function_name_instructs_the_rename() {
    let objections = check_diff_text(&added(&["def foo():"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].category, "naming");
    assert_eq!(
        objections[0].rewrite,
        "Rename foo to a name that says what the function does \
         and keep the signature as written"
    );
}

#[test]
fn every_placeholder_fires_as_an_assignment_target() {
    for name in PLACEHOLDERS {
        let line = format!("{name} = load()");
        let objections = check_diff_text(&added(&[&line]));
        assert_eq!(objections.len(), 1, "name {name}");
        assert_eq!(objections[0].category, "naming", "name {name}");
    }
}

#[test]
fn descriptive_names_do_not_fire() {
    assert!(check_diff_text(&added(&["orders = load_orders()"])).is_empty());
    assert!(check_diff_text(&added(&["def load_orders():"])).is_empty());
}

#[test]
fn removed_lines_never_fire() {
    let text = diff(&[
        "--- a/app.py",
        "+++ b/app.py",
        "@@ -1,1 +1,1 @@",
        "-    print(total)",
        "-    except:",
        "-foo = load()",
    ]);
    assert!(check_diff_text(&text).is_empty());
}

#[test]
fn context_lines_never_fire() {
    let text = diff(&[
        "--- a/app.py",
        "+++ b/app.py",
        "@@ -1,1 +1,1 @@",
        "     print(total)",
        "     except:",
    ]);
    assert!(check_diff_text(&text).is_empty());
}

#[test]
fn the_three_plus_header_is_not_an_added_line() {
    let text = diff(&["--- a/app.py", "+++ b/print(total).py", "@@ -1,0 +1,0 @@"]);
    assert!(check_diff_text(&text).is_empty());
}

#[test]
fn a_shape_inside_a_string_literal_raises_like_code() {
    let objections = check_diff_text(&added(&["    label = \"print(total)\""]));
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].category, "smell");
}

#[test]
fn objections_come_back_in_line_order() {
    let objections = check_diff_text(&added(&["    print(total)", "    except:"]));
    assert_eq!(
        rewrites(&objections),
        vec!["    logger.debug(total)", "    except Exception:"]
    );
}

#[test]
fn one_line_raises_at_most_one_objection_per_shape() {
    let objections = check_diff_text(&added(&["def foo(items=[]):"]));
    assert_eq!(categories(&objections), vec!["structure", "naming"]);
}

#[test]
fn checker_output_is_always_a_complete_valid_review() {
    let text = added(&["    except:", "    print(total)", "foo = load_orders()"]);
    let objections = check_diff_text(&text);
    assert_eq!(objections.len(), 3);
    let built = JsonValue::Object(vec![
        (
            JString::from_text("diff_label"),
            JsonValue::Str(JString::from_text("checked change")),
        ),
        (
            JString::from_text("objections"),
            JsonValue::Array(
                objections
                    .iter()
                    .map(|objection| {
                        JsonValue::Object(vec![
                            (
                                JString::from_text("flaw"),
                                JsonValue::Str(JString::from_text(&objection.flaw)),
                            ),
                            (
                                JString::from_text("standard"),
                                JsonValue::Str(JString::from_text(&objection.standard)),
                            ),
                            (
                                JString::from_text("rewrite"),
                                JsonValue::Str(JString::from_text(&objection.rewrite)),
                            ),
                            (
                                JString::from_text("category"),
                                JsonValue::Str(JString::from_text(&objection.category)),
                            ),
                        ])
                    })
                    .collect(),
            ),
        ),
    ]);
    assert_eq!(validate_review(&built), Vec::<String>::new());
    assert_eq!(verdict_for(&built), "sent back");
}

#[test]
fn a_clean_diff_raises_nothing() {
    let text = added(&["    orders = load_orders()", "    logger.debug(orders)"]);
    assert!(check_diff_text(&text).is_empty());
}

#[test]
fn placeholder_trailing_underscore_callee_instructs_the_rename() {
    let objections = check_diff_text(&added(&["foo = load_()"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(
        objections[0].rewrite,
        "Rename foo to a name that says what the value holds \
         and keep the rest of the line as written"
    );
}

#[test]
fn placeholder_single_underscore_callee_instructs_the_rename() {
    let objections = check_diff_text(&added(&["foo = _()"]));
    assert_eq!(objections.len(), 1);
    assert!(objections[0].rewrite.starts_with("Rename foo to a name"));
}

#[test]
fn placeholder_keyword_stem_callee_instructs_the_rename() {
    let objections = check_diff_text(&added(&["foo = class_()"]));
    assert_eq!(objections.len(), 1);
    assert!(objections[0].rewrite.starts_with("Rename foo to a name"));
}

#[test]
fn placeholder_annotated_assignment_fires_and_keeps_the_annotation() {
    let objections = check_diff_text(&added(&["foo: int = load_orders()"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].category, "naming");
    assert_eq!(objections[0].rewrite, "orders: int = load_orders()");
}

#[test]
fn placeholder_annotated_assignment_keeps_a_one_word_callee() {
    let objections = check_diff_text(&added(&["foo: int = load()"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].rewrite, "load: int = load()");
}

#[test]
fn placeholder_in_a_tuple_target_instructs_the_rename() {
    let objections = check_diff_text(&added(&["foo, other = load(), 1"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].category, "naming");
    assert!(objections[0].rewrite.starts_with("Rename foo to a name"));
}

#[test]
fn placeholder_as_the_second_tuple_target_instructs_the_rename() {
    let objections = check_diff_text(&added(&["other, foo = 1, load()"]));
    assert_eq!(objections.len(), 1);
    assert!(objections[0].rewrite.starts_with("Rename foo to a name"));
}

#[test]
fn two_mutable_defaults_draw_one_objection_rewriting_both() {
    let objections = check_diff_text(&added(&["def load(a=[], b={}):"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].category, "structure");
    assert_eq!(objections[0].rewrite, "def load(a=None, b=None):");
}

#[test]
fn a_set_literal_instructs_the_rename() {
    let objections = check_diff_text(&added(&["foo = {1, 2}"]));
    assert_eq!(objections.len(), 1);
    assert!(objections[0].rewrite.starts_with("Rename foo to a name"));
}

#[test]
fn an_empty_dict_still_draws_the_dict_name() {
    let objections = check_diff_text(&added(&["foo = {}"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].rewrite, "mapping = {}");
}

#[test]
fn a_spaced_empty_set_call_fires_as_a_mutable_default() {
    let objections = check_diff_text(&added(&["def load(seen=set ( )):"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].category, "structure");
    assert_eq!(objections[0].rewrite, "def load(seen=None):");
}

const CALL_REWRITE_END: &str =
    " and build each replaced default inside the body under a None check";

#[test]
fn a_call_default_fires_with_the_none_check_rewrite() {
    let objections = check_diff_text(&added(&["def stamp(at=now()):"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].category, "structure");
    assert_eq!(
        objections[0].rewrite,
        format!("def stamp(at=None):{CALL_REWRITE_END}")
    );
}

#[test]
fn a_dotted_call_default_fires() {
    let objections = check_diff_text(&added(&["def stamp(at=datetime.now()):"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(
        objections[0].rewrite,
        format!("def stamp(at=None):{CALL_REWRITE_END}")
    );
}

#[test]
fn a_call_default_with_keyword_arguments_fires() {
    let objections = check_diff_text(&added(&["def load(at=now(tz=utc)):"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(
        objections[0].rewrite,
        format!("def load(at=None):{CALL_REWRITE_END}")
    );
}

#[test]
fn an_annotated_call_default_fires() {
    let objections = check_diff_text(&added(&["def stamp(at: str = now()):"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(
        objections[0].rewrite,
        format!("def stamp(at: str = None):{CALL_REWRITE_END}")
    );
}

#[test]
fn two_call_defaults_draw_one_objection_rewriting_both() {
    let objections = check_diff_text(&added(&["def stamp(a=now(), b=later()):"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(
        objections[0].rewrite,
        format!("def stamp(a=None, b=None):{CALL_REWRITE_END}")
    );
}

#[test]
fn a_mutable_default_and_a_call_default_fire_in_shape_order() {
    let objections = check_diff_text(&added(&["def load(items=[], at=now()):"]));
    assert_eq!(categories(&objections), vec!["structure", "structure"]);
    assert_eq!(objections[0].rewrite, "def load(items=None, at=now()):");
    assert_eq!(
        objections[1].rewrite,
        format!("def load(items=[], at=None):{CALL_REWRITE_END}")
    );
}

#[test]
fn the_empty_set_call_is_a_mutable_default_and_not_a_call_default() {
    let objections = check_diff_text(&added(&["def load(seen=set()):"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].rewrite, "def load(seen=None):");
}

#[test]
fn non_call_defaults_do_not_fire_the_call_default_shape() {
    assert!(check_diff_text(&added(&["def load(at=now):"])).is_empty());
    assert!(check_diff_text(&added(&["def load(at=lambda: 1):"])).is_empty());
    assert!(check_diff_text(&added(&["def load(at=now() + 1):"])).is_empty());
    assert!(check_diff_text(&added(&["def load(at=(now())):"])).is_empty());
}

#[test]
fn a_call_default_on_a_placeholder_def_stands_after_the_naming() {
    let objections = check_diff_text(&added(&["def foo(at=now()):"]));
    assert_eq!(categories(&objections), vec!["naming", "structure"]);
    assert_eq!(
        objections[1].rewrite,
        format!("def foo(at=None):{CALL_REWRITE_END}")
    );
}

#[test]
fn a_signature_split_across_lines_reads_no_call_default() {
    let text = added(&["def load(at=now(),", "        other=1):"]);
    assert!(check_diff_text(&text).is_empty());
}

#[test]
fn a_class_attribute_list_literal_fires_with_the_move_rewrite() {
    let text = raw(&[" class Config:", "+    seen = []"]);
    let objections = check_diff_text(&text);
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].category, "structure");
    assert_eq!(
        objections[0].flaw,
        "The added line gives the class attribute seen \
         a mutable literal that every instance shares"
    );
    assert_eq!(
        objections[0].rewrite,
        "Move seen into the initializer as self.seen = [] \
         and remove the class level assignment"
    );
}

#[test]
fn class_attribute_dict_and_set_literals_fire() {
    let cases = [
        ("    table = {'a': 1}", "self.table = {'a': 1}"),
        ("    tags = {1, 2}", "self.tags = {1, 2}"),
        ("    items = [1, 2]", "self.items = [1, 2]"),
    ];
    for (line, moved) in cases {
        let added_line = format!("+{line}");
        let text = raw(&[" class Config:", added_line.as_str()]);
        let objections = check_diff_text(&text);
        assert_eq!(objections.len(), 1, "line {line}");
        let name = line.split_whitespace().next().unwrap();
        assert_eq!(
            objections[0].rewrite,
            format!(
                "Move {name} into the initializer as {moved} \
                 and remove the class level assignment"
            ),
            "line {line}"
        );
    }
}

#[test]
fn an_annotated_class_attribute_fires() {
    let text = raw(&[" class Config:", "+    seen: list = []"]);
    let objections = check_diff_text(&text);
    assert_eq!(objections.len(), 1);
    assert_eq!(
        objections[0].rewrite,
        "Move seen into the initializer as self.seen = [] \
         and remove the class level assignment"
    );
}

#[test]
fn a_class_with_bases_still_opens_a_class_body() {
    let text = raw(&[" class Config(Base, Mixin):", "+    seen = []"]);
    assert_eq!(check_diff_text(&text).len(), 1);
}

#[test]
fn a_module_level_literal_assignment_draws_nothing() {
    assert!(check_diff_text(&added(&["seen = []"])).is_empty());
    assert!(check_diff_text(&added(&["table = {'a': 1}"])).is_empty());
}

#[test]
fn a_literal_assignment_inside_a_function_inside_a_class_draws_nothing() {
    let text = raw(&[
        " class Config:",
        "     def build(self):",
        "+        cache = []",
    ]);
    assert!(check_diff_text(&text).is_empty());
}

#[test]
fn a_literal_assignment_inside_a_class_inside_a_function_fires() {
    let text = raw(&[" def build():", "     class Inner:", "+        seen = []"]);
    let objections = check_diff_text(&text);
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].category, "structure");
}

#[test]
fn a_literal_assignment_inside_a_nested_class_fires() {
    let text = raw(&[" class Outer:", "     class Inner:", "+        seen = []"]);
    assert_eq!(check_diff_text(&text).len(), 1);
}

#[test]
fn a_literal_assignment_under_a_conditional_in_a_class_fires() {
    let text = raw(&[" class Config:", "     if DEBUG:", "+        seen = []"]);
    assert_eq!(check_diff_text(&text).len(), 1);
}

#[test]
fn a_second_hunk_without_the_class_line_carries_no_class_context() {
    let text = diff(&[
        "--- a/app.py",
        "+++ b/app.py",
        "@@ -1,2 +1,2 @@",
        " class Config:",
        "     value = 1",
        "@@ -20,1 +20,2 @@",
        "     other = 2",
        "+    seen = []",
    ]);
    assert!(check_diff_text(&text).is_empty());
}

#[test]
fn a_removed_class_line_opens_no_class_body() {
    let text = raw(&["-class Config:", "+    seen = []"]);
    assert!(check_diff_text(&text).is_empty());
}

#[test]
fn a_class_level_call_or_scalar_assignment_draws_nothing() {
    for line in [
        "    seen = list()",
        "    count = 0",
        "    name = 'x'",
        "    seen = [1] + [2]",
    ] {
        let added_line = format!("+{line}");
        let text = raw(&[" class Config:", added_line.as_str()]);
        assert!(check_diff_text(&text).is_empty(), "line {line}");
    }
}

#[test]
fn a_placeholder_in_a_class_body_draws_naming_then_structure() {
    let text = raw(&[" class Config:", "+    foo = []"]);
    let objections = check_diff_text(&text);
    assert_eq!(categories(&objections), vec!["naming", "structure"]);
    assert_eq!(objections[0].rewrite, "    items = []");
    assert_eq!(
        objections[1].rewrite,
        "Move foo into the initializer as self.foo = [] \
         and remove the class level assignment"
    );
}

#[test]
fn a_method_def_line_in_a_class_still_draws_the_def_shapes() {
    let text = raw(&[" class Config:", "+    def build(self, items=[]):"]);
    let objections = check_diff_text(&text);
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].rewrite, "    def build(self, items=None):");
}

#[test]
fn a_one_line_class_draws_nothing() {
    assert!(check_diff_text(&added(&["class Config: seen = []"])).is_empty());
}

#[test]
fn a_blank_context_line_inside_a_class_body_closes_nothing() {
    let text = diff(&[
        "--- a/app.py",
        "+++ b/app.py",
        "@@ -1,1 +1,1 @@",
        " class Config:",
        "     value = 1",
        " ",
        "+    items = []",
    ]);
    let objections = check_diff_text(&text);
    assert_eq!(objections.len(), 1);
    assert_eq!(
        objections[0].rewrite,
        "Move items into the initializer as self.items = [] \
         and remove the class level assignment"
    );
}

#[test]
fn an_added_blank_line_inside_a_class_body_closes_nothing() {
    let text = raw(&[" class Config:", "+", "+    items = []"]);
    assert_eq!(check_diff_text(&text).len(), 1);
}

#[test]
fn the_written_forms_of_a_blank_line_read_the_same_way() {
    let forms: [&[&str]; 3] = [
        &[" class Config:", "", "+    items = []"],
        &[" class Config:", " ", "+    items = []"],
        &[" class Config:", "   ", "+    items = []"],
    ];
    for lines in forms {
        assert_eq!(check_diff_text(&raw(lines)).len(), 1, "lines {lines:?}");
    }
}

#[test]
fn a_tab_only_context_line_inside_a_class_body_closes_nothing() {
    let text = raw(&[" class Config:", " \t", "+    items = []"]);
    let objections = check_diff_text(&text);
    assert_eq!(objections.len(), 1);
    assert_eq!(
        objections[0].rewrite,
        "Move items into the initializer as self.items = [] \
         and remove the class level assignment"
    );
}

#[test]
fn a_tab_only_added_line_inside_a_class_body_closes_nothing() {
    let text = raw(&[" class Config:", "+\t", "+    items = []"]);
    assert_eq!(check_diff_text(&text).len(), 1);
}

#[test]
fn a_call_default_holding_an_escaped_quote_fires() {
    let objections = check_diff_text(&added(&["def f(a=make(\"a\\\")b\")):"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(
        objections[0].rewrite,
        format!("def f(a=None):{CALL_REWRITE_END}")
    );
}

#[test]
fn a_call_default_after_a_string_holding_an_escaped_quote_fires() {
    let objections = check_diff_text(&added(&["def f(a=\"x\\\")y\", b=now()):"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(
        objections[0].rewrite,
        format!("def f(a=\"x\\\")y\", b=None):{CALL_REWRITE_END}")
    );
}

#[test]
fn a_class_attribute_literal_holding_an_escaped_quote_fires() {
    let text = raw(&[" class Config:", "+    x = [\"a\\\" ] \", \"b\"]"]);
    let objections = check_diff_text(&text);
    assert_eq!(objections.len(), 1);
    assert_eq!(
        objections[0].rewrite,
        "Move x into the initializer as self.x = \
         [\"a\\\" ] \", \"b\"] and remove the class level assignment"
    );
}

#[test]
fn a_placeholder_dict_holding_an_escaped_quote_draws_the_dict_name() {
    let objections = check_diff_text(&added(&["foo = {\"a\\\"b\": 1}"]));
    assert_eq!(objections.len(), 1);
    assert_eq!(objections[0].rewrite, "mapping = {\"a\\\"b\": 1}");
}

#[test]
fn a_plain_file_header_empties_the_class_stack() {
    let text = diff(&[
        "--- a/x.py",
        "+++ b/x.py",
        "@@ -1,1 +1,1 @@",
        " class Config:",
        "+    seen = []",
        "--- a/y.py",
        "+++ b/y.py",
        "+    items = []",
    ]);
    let objections = check_diff_text(&text);
    assert_eq!(objections.len(), 1);
    assert_eq!(
        objections[0].rewrite,
        "Move seen into the initializer as self.seen = [] \
         and remove the class level assignment"
    );
}
