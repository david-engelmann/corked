//! The stickler command line. Six commands, check, render,
//! review, census, rejudged, and split.
//!
//! The surface stands as two functions over two readings of
//! the arguments. The binary entry, run below, collects the
//! arguments as the operating system hands them over, decodes
//! them the way the interpreter decodes its argument strings,
//! and drives cli_main_units, which returns the exit code and
//! the two output texts as raw bytes, because a check label can
//! stand for a file name byte that no text encoding holds.
//! cli_main takes text arguments and returns texts, and stands
//! for the tests, which drive it the way the Python tests drive
//! the Python main. main.rs carries nothing but a forward to
//! run, which collects the arguments, prints the texts, and
//! exits with the code, all inside this module.

use std::io::Write;

use crate::census::{census_of, load_census};
use crate::changeset::{load_changeset, review_changeset};
use crate::checker::check_diff_text;
use crate::json::{self, JString, JsonValue, ParseError};
use crate::rejudged::{load_rejudged, rejudged_of};
use crate::render::{
    render_census, render_census_defects, render_defects, render_defects_units, render_rejudged,
    render_rejudged_defects, render_review, render_set_defects, render_set_review, render_split,
    render_split_defects,
};
use crate::review::verdict_for;
use crate::split::{load_split, split_of};
use crate::verdicts::line_outcome;

/// The usage text the command line answers an unrecognized
/// invocation with. No argument is echoed in it, under any
/// argument bytes.
const USAGE: &str = "Usage, stickler check DIFF_FILE\n\
                     Usage, stickler render REVIEW_JSON\n\
                     Usage, stickler review SET_JSON\n\
                     Usage, stickler census CENSUS_JSON\n\
                     Usage, stickler rejudged REJUDGED_JSON\n\
                     Usage, stickler split SPLIT_JSON\n";

/// The outcome of one command line run, holding the exit code
/// and the text for each of the two output streams.
pub struct CliOutcome {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// The outcome of one command line run over code unit
/// arguments, holding the exit code and the raw bytes for each
/// of the two output streams.
pub struct ByteOutcome {
    pub code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

fn byte_outcome(code: i32, stdout: Vec<u8>) -> ByteOutcome {
    ByteOutcome {
        code,
        stdout,
        stderr: Vec::new(),
    }
}

fn text_outcome(code: i32, stdout: String) -> ByteOutcome {
    byte_outcome(code, stdout.into_bytes())
}

fn exit_code(verdict: &str) -> i32 {
    match verdict {
        "accepted" => 0,
        "sent back" => 1,
        _ => 2,
    }
}

/// The raw bytes of one operating system argument.
#[cfg(unix)]
fn argument_bytes(arg: &std::ffi::OsString) -> Vec<u8> {
    std::os::unix::ffi::OsStrExt::as_bytes(arg.as_os_str()).to_vec()
}

/// The raw bytes of one operating system argument. Off Unix
/// the lossy text reading is the closest the platform offers,
/// and the gate the port stands under is a Unix one.
#[cfg(not(unix))]
fn argument_bytes(arg: &std::ffi::OsString) -> Vec<u8> {
    arg.to_string_lossy().into_owned().into_bytes()
}

/// The path a code unit argument names on this platform, under
/// the same handler the interpreter encodes its paths with, so
/// a name decoded from raw argument bytes opens the very file
/// the caller named.
#[cfg(unix)]
fn path_os(path: &JString) -> std::ffi::OsString {
    std::os::unix::ffi::OsStringExt::from_vec(path.to_surrogateescape())
}

/// The path a code unit argument names on this platform.
#[cfg(not(unix))]
fn path_os(path: &JString) -> std::ffi::OsString {
    std::ffi::OsString::from(path.to_rust_string().unwrap_or_default())
}

/// Read a file as UTF-8 text, as the Python readers read it.
/// Input that cannot be read, and input that cannot be decoded
/// as UTF-8 text, are the same failure to the commands.
fn read_text(path: &JString) -> Result<String, ()> {
    let bytes = std::fs::read(path_os(path)).map_err(|_| ())?;
    String::from_utf8(bytes).map_err(|_| ())
}

/// The name a path stands under, as the Python pathlib reads
/// it. The path splits on the forward slash, empty parts and
/// parts holding a single dot drop out, and the last part
/// standing is the name, so the name of a/.. and of a/b/../..
/// alike is the two dots. A path with no part standing names
/// nothing.
fn file_name(path: &JString) -> JString {
    let mut name: &[u16] = &[];
    for part in path.0.split(|unit| *unit == 0x2f) {
        if !part.is_empty() && part != [0x2e] {
            name = part;
        }
    }
    JString(name.to_vec())
}

fn check(path: &JString) -> ByteOutcome {
    let name = file_name(path);
    let text = match read_text(path) {
        Err(()) => {
            return byte_outcome(
                2,
                render_defects_units(&name, &["the diff file could not be read".to_string()]),
            );
        }
        Ok(text) => text,
    };
    let objections: Vec<JsonValue> = check_diff_text(&text)
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
        .collect();
    let review = JsonValue::Object(vec![
        (JString::from_text("diff_label"), JsonValue::Str(name)),
        (
            JString::from_text("objections"),
            JsonValue::Array(objections),
        ),
    ]);
    let code = exit_code(verdict_for(&review));
    text_outcome(code, render_review(&review))
}

fn render_command(path: &JString) -> ByteOutcome {
    let text = match read_text(path) {
        Err(()) => {
            return text_outcome(
                2,
                render_defects(
                    "unnamed review",
                    &["the review file could not be read".to_string()],
                ),
            );
        }
        Ok(text) => text,
    };
    let review = match json::parse(&text) {
        Err(ParseError::TooDeep) => {
            // The Python reader raises its recursion failure out
            // of this command uncaught, and the console entry
            // falls with it. The port mirrors the fall, an exit
            // code of one with nothing on the standard output.
            return ByteOutcome {
                code: 1,
                stdout: Vec::new(),
                stderr: "maximum recursion depth exceeded while \
                         reading the review file\n"
                    .as_bytes()
                    .to_vec(),
            };
        }
        Err(_) => {
            return text_outcome(
                2,
                render_defects(
                    "unnamed review",
                    &["the file does not hold a JSON review".to_string()],
                ),
            );
        }
        Ok(value) => value,
    };
    let code = exit_code(verdict_for(&review));
    text_outcome(code, render_review(&review))
}

fn review_command(path: &JString) -> ByteOutcome {
    let text = match read_text(path) {
        Err(()) => {
            return text_outcome(
                2,
                render_set_defects(&["the change set file could not be read".to_string()]),
            );
        }
        Ok(text) => text,
    };
    let value = match json::parse(&text) {
        Err(_) => {
            return text_outcome(
                2,
                render_set_defects(&["the file does not hold a JSON change set".to_string()]),
            );
        }
        Ok(value) => value,
    };
    let defects = load_changeset(&value);
    if !defects.is_empty() {
        return text_outcome(2, render_set_defects(&defects));
    }
    let results = review_changeset(&value);
    let rendered = render_set_review(&results);
    if results.iter().any(|result| !result.judgements.is_empty()) {
        return text_outcome(1, rendered);
    }
    for result in &results {
        for block in &result.blocks {
            if line_outcome(&block.verdicts) == "stickler" {
                return text_outcome(1, rendered);
            }
        }
    }
    text_outcome(0, rendered)
}

fn census_command(path: &JString) -> ByteOutcome {
    let text = match read_text(path) {
        Err(()) => {
            return text_outcome(
                2,
                render_census_defects(&["the census file could not be read".to_string()]),
            );
        }
        Ok(text) => text,
    };
    let value = match json::parse(&text) {
        Err(_) => {
            return text_outcome(
                2,
                render_census_defects(&["the file does not hold a JSON census".to_string()]),
            );
        }
        Ok(value) => value,
    };
    let defects = load_census(&value);
    if !defects.is_empty() {
        return text_outcome(2, render_census_defects(&defects));
    }
    text_outcome(0, render_census(&census_of(&value)))
}

fn rejudged_command(path: &JString) -> ByteOutcome {
    let text = match read_text(path) {
        Err(()) => {
            return text_outcome(
                2,
                render_rejudged_defects(&["the rejudged file could not be read".to_string()]),
            );
        }
        Ok(text) => text,
    };
    let value = match json::parse(&text) {
        Err(_) => {
            return text_outcome(
                2,
                render_rejudged_defects(&[
                    "the file does not hold a JSON rejudged pair".to_string()
                ]),
            );
        }
        Ok(value) => value,
    };
    let defects = load_rejudged(&value);
    if !defects.is_empty() {
        return text_outcome(2, render_rejudged_defects(&defects));
    }
    text_outcome(0, render_rejudged(&rejudged_of(&value)))
}

fn split_command(path: &JString) -> ByteOutcome {
    let text = match read_text(path) {
        Err(()) => {
            return text_outcome(
                2,
                render_split_defects(&["the split file could not be read".to_string()]),
            );
        }
        Ok(text) => text,
    };
    let value = match json::parse(&text) {
        Err(_) => {
            return text_outcome(
                2,
                render_split_defects(&["the file does not hold a JSON split".to_string()]),
            );
        }
        Ok(value) => value,
    };
    let defects = load_split(&value);
    if !defects.is_empty() {
        return text_outcome(2, render_split_defects(&defects));
    }
    text_outcome(0, render_split(&split_of(&value)))
}

/// Run the command line over code unit arguments and return
/// the outcome as raw bytes.
pub fn cli_main_units(args: &[JString]) -> ByteOutcome {
    if args.len() == 2 {
        if args[0].equals("check") {
            return check(&args[1]);
        }
        if args[0].equals("render") {
            return render_command(&args[1]);
        }
        if args[0].equals("review") {
            return review_command(&args[1]);
        }
        if args[0].equals("census") {
            return census_command(&args[1]);
        }
        if args[0].equals("rejudged") {
            return rejudged_command(&args[1]);
        }
        if args[0].equals("split") {
            return split_command(&args[1]);
        }
    }
    ByteOutcome {
        code: 2,
        stdout: Vec::new(),
        stderr: USAGE.as_bytes().to_vec(),
    }
}

/// Run the command line over text arguments and return the
/// outcome. The output texts over text arguments are always
/// text, because a label standing for raw bytes enters only
/// through the operating system route, so the lossy read of
/// the byte outcome is exact here.
pub fn cli_main(args: &[String]) -> CliOutcome {
    let units: Vec<JString> = args.iter().map(|arg| JString::from_text(arg)).collect();
    let result = cli_main_units(&units);
    CliOutcome {
        code: result.code,
        stdout: String::from_utf8_lossy(&result.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&result.stderr).into_owned(),
    }
}

/// Write one output stream and return whether the stream
/// stands closed under the write. Only a broken pipe counts,
/// the closed stream the interpreter's own exit stands on.
fn write_stream(stream: &mut impl Write, bytes: &[u8]) -> bool {
    let mut broken = false;
    for attempt in [stream.write_all(bytes), stream.flush()] {
        if let Err(error) = attempt {
            if error.kind() == std::io::ErrorKind::BrokenPipe {
                broken = true;
            }
        }
    }
    broken
}

/// Run the command line for the binary. Returns the exit code
/// the process exits with.
///
/// The arguments are read as the raw bytes the operating
/// system hands over and decoded the way the interpreter
/// decodes them, so a file name carrying a byte that is no
/// UTF-8 text runs as the Python runs it, and the run never
/// falls on the argument itself. A stream closed under the
/// final write fails the run with the exit code 120, whatever
/// code the run itself earned, as the interpreter exits 120
/// when its output stream stands closed at its shutdown.
pub fn run() -> i32 {
    let args: Vec<JString> = std::env::args_os()
        .skip(1)
        .map(|arg| JString::from_surrogateescape(&argument_bytes(&arg)))
        .collect();
    let result = cli_main_units(&args);
    let mut stdout = std::io::stdout().lock();
    let stdout_broken = write_stream(&mut stdout, &result.stdout);
    let mut stderr = std::io::stderr().lock();
    let stderr_broken = write_stream(&mut stderr, &result.stderr);
    if stdout_broken || stderr_broken {
        120
    } else {
        result.code
    }
}
