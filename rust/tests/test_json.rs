//! Pins for the JSON value module at the surface the loaders
//! read. Each semantic stands against the behavior the Python
//! json reader shows on the same input, probed against the
//! interpreter. The module carries its own unit pins beside
//! these.

use num_bigint::BigInt;
use stickler::json::{parse, JsonNumber, JsonValue, ParseError};

#[test]
fn an_integer_literal_classifies_as_an_integer() {
    let value = parse("100").unwrap();
    assert!(matches!(value, JsonValue::Number(JsonNumber::Int(_))));
    let value = parse("1e2").unwrap();
    assert!(matches!(value, JsonValue::Number(JsonNumber::Float(_))));
    let value = parse("1.0").unwrap();
    assert!(matches!(value, JsonValue::Number(JsonNumber::Float(_))));
}

#[test]
fn duplicate_object_keys_keep_the_last_value() {
    let value = parse(r#"{"a": 1, "b": 2, "a": 3}"#).unwrap();
    let held = value.get("a").and_then(|v| v.as_number()).unwrap();
    assert_eq!(held.render(), "3");
    let pairs = value.as_object().unwrap();
    assert_eq!(pairs.len(), 2);
    assert!(pairs[0].0.equals("a"));
}

#[test]
fn an_overflowing_float_spelling_reads_as_not_finite() {
    let value = parse("1e400").unwrap();
    let held = value.as_number().unwrap();
    assert!(!held.is_finite());
    let value = parse("-1e400").unwrap();
    assert!(!value.as_number().unwrap().is_finite());
}

#[test]
fn the_finite_boundary_of_fix_ck_16_stands() {
    // The largest integer a finite double holds exactly, and
    // one more, spelled as integer literals.
    let largest = (BigInt::from((1u64 << 53) - 1) << 971usize).to_string();
    let value = parse(&largest).unwrap();
    assert!(value.as_number().unwrap().is_finite());
    let beyond: BigInt = (BigInt::from((1u64 << 53) - 1) << 971usize) + BigInt::from(1);
    let value = parse(&beyond.to_string()).unwrap();
    assert!(!value.as_number().unwrap().is_finite());
    let ten_308 = format!("1{}", "0".repeat(308));
    let value = parse(&ten_308).unwrap();
    assert!(value.as_number().unwrap().is_finite());
    let ten_309 = format!("1{}", "0".repeat(309));
    let value = parse(&ten_309).unwrap();
    assert!(!value.as_number().unwrap().is_finite());
}

#[test]
fn the_integer_digit_limit_matches_the_interpreter() {
    let longest = "1".repeat(4300);
    assert!(parse(&longest).is_ok());
    let too_long = "1".repeat(4301);
    assert_eq!(parse(&too_long), Err(ParseError::IntTooLong));
}

#[test]
fn the_nesting_limit_matches_the_interpreter() {
    let deepest = "[".repeat(9997) + &"]".repeat(9997);
    assert!(parse(&deepest).is_ok());
    let too_deep = "[".repeat(9998) + &"]".repeat(9998);
    assert_eq!(parse(&too_deep), Err(ParseError::TooDeep));
}

/// The boundary counts open containers in total, as the
/// console script measurements read it. A field nested in
/// arrays reaches the line one array earlier than the bare
/// shape. The object nested shape stands pinned at the
/// command level in the command line pins, and its disposal
/// stands pinned below. The disposal is iterative over a
/// work list, so a deepest object tree drops on a small stack
/// thread as a deepest array tree does.
#[test]
fn the_nesting_limit_counts_containers_on_the_field_shape() {
    let field_deepest = "{\"a\": ".to_string() + &"[".repeat(9996) + &"]".repeat(9996) + "}";
    assert!(parse(&field_deepest).is_ok());
    let field_too_deep = "{\"a\": ".to_string() + &"[".repeat(9997) + &"]".repeat(9997) + "}";
    assert_eq!(parse(&field_too_deep), Err(ParseError::TooDeep));
}

/// The deepest object tree is disposed of on a thread of the
/// stack size a test thread carries, two mebibytes. The pin
/// runs the reading and the dropping inside the spawned
/// thread and asserts the thread completes. Against the
/// library before the iterative disposal this pin aborts the
/// process, the derived disposal recursing to the depth of
/// the tree and overflowing the small stack.
#[test]
fn a_deepest_object_tree_drops_on_a_small_stack_thread() {
    let deepest = "{\"a\": ".repeat(9997) + "1" + &"}".repeat(9997);
    let handle = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let value = parse(&deepest).unwrap();
            drop(value);
        })
        .unwrap();
    handle.join().unwrap();
}

/// The deepest array tree stands under the same disposal pin
/// as the object tree. Its disposal already completed at
/// this stack size before the iterative disposal, and the
/// pin stands so the two shapes are read together.
#[test]
fn a_deepest_array_tree_drops_on_a_small_stack_thread() {
    let deepest = "[".repeat(9997) + &"]".repeat(9997);
    let handle = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let value = parse(&deepest).unwrap();
            drop(value);
        })
        .unwrap();
    handle.join().unwrap();
}

/// The deepest object tree is cloned on a thread of the
/// stack size a test thread carries, two mebibytes, under
/// FIX-CK-19. The pin runs the reading, the cloning, and
/// the dropping inside the spawned thread, walks the clone
/// to its full depth with a loop, and asserts the thread
/// completes. Against the library before the iterative
/// clone this pin aborts the process, the derived clone
/// recursing to the depth of the tree and overflowing the
/// small stack.
#[test]
fn a_deepest_object_tree_clones_on_a_small_stack_thread() {
    let deepest = "{\"a\": ".repeat(9997) + "1" + &"}".repeat(9997);
    let handle = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let value = parse(&deepest).unwrap();
            let cloned = value.clone();
            let mut depth = 0usize;
            let mut current = &cloned;
            while let Some(inner) = current.get("a") {
                depth += 1;
                current = inner;
            }
            assert_eq!(depth, 9997);
            drop(value);
            drop(cloned);
        })
        .unwrap();
    handle.join().unwrap();
}

/// The deepest array tree stands under the same clone pin
/// as the object tree, under FIX-CK-19. The walk counts
/// every array of the clone with a loop, so the pin reads
/// the depth of the clone without recursing itself.
#[test]
fn a_deepest_array_tree_clones_on_a_small_stack_thread() {
    let deepest = "[".repeat(9997) + &"]".repeat(9997);
    let handle = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let value = parse(&deepest).unwrap();
            let cloned = value.clone();
            let mut depth = 0usize;
            let mut current = &cloned;
            while let Some(items) = current.as_array() {
                depth += 1;
                match items.first() {
                    Some(inner) => current = inner,
                    None => break,
                }
            }
            assert_eq!(depth, 9997);
            drop(value);
            drop(cloned);
        })
        .unwrap();
    handle.join().unwrap();
}

#[test]
fn an_unpaired_surrogate_is_representable_and_detected() {
    let value = parse(r#""\ud800x""#).unwrap();
    let held = value.as_jstring().unwrap();
    assert!(held.holds_unpaired_surrogate());
    assert!(held.to_rust_string().is_none());
    let pair = parse(r#""\ud83d\ude00""#).unwrap();
    let held = pair.as_jstring().unwrap();
    assert!(!held.holds_unpaired_surrogate());
    assert_eq!(held.to_rust_string().unwrap(), "\u{1f600}");
}

#[test]
fn the_python_constant_spellings_read() {
    for (text, finite) in [("NaN", false), ("Infinity", false), ("-Infinity", false)] {
        let value = parse(text).unwrap();
        assert_eq!(value.as_number().unwrap().is_finite(), finite);
    }
}

#[test]
fn malformed_texts_are_refused() {
    for text in [
        "",
        "{",
        "[1,]",
        "{'a': 1}",
        "01",
        "+1",
        ".5",
        "1.",
        "1 2",
        "\"a\tb\"",
        "\u{feff}{}",
        "True",
        "None",
    ] {
        assert!(parse(text).is_err(), "text {text:?}");
    }
}

#[test]
fn float_spellings_render_as_the_interpreter_spells_them() {
    for (text, rendered) in [
        ("0.78", "0.78"),
        ("1.0", "1.0"),
        ("0.0001", "0.0001"),
        ("0.00001", "1e-05"),
        ("1e16", "1e+16"),
        ("1234567890123456.0", "1234567890123456.0"),
        ("2.5e-5", "2.5e-05"),
        ("-0.0", "-0.0"),
        ("100", "100"),
        ("-7", "-7"),
    ] {
        let value = parse(text).unwrap();
        assert_eq!(value.as_number().unwrap().render(), rendered, "text {text}");
    }
}

#[test]
fn object_equality_ignores_key_order_as_a_dict_does() {
    let left = parse(r#"{"a": 1, "b": [1, 2]}"#).unwrap();
    let right = parse(r#"{"b": [1, 2], "a": 1}"#).unwrap();
    assert_eq!(left, right);
    let different = parse(r#"{"b": [1, 2], "a": 2}"#).unwrap();
    assert_ne!(left, different);
}

#[test]
fn number_equality_crosses_the_integer_float_line_as_python_does() {
    let int = parse("1").unwrap();
    let float = parse("1.0").unwrap();
    assert_eq!(int, float);
    let text = parse(r#""1""#).unwrap();
    assert_ne!(int, text);
    let truth = parse("true").unwrap();
    assert_ne!(int, truth);
}
