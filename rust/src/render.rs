//! The one deterministic rendering of a review, shared by every
//! command.

use crate::census::{Census, Tally};
use crate::changeset::{FileResult, Judgements};
use crate::checker::{BORDERLINE_PATTERNS, SHAPES};
use crate::contested::Contested;
use crate::json::{JString, JsonValue};
use crate::rejudged::Rejudged;
use crate::review::{validate_review, verdict_for};
use crate::split::Split;
use crate::verdicts::{is_contested, line_outcome, reason_for, verdict_outcome};

/// Read a text field of a valid review or entry as the string
/// it stands for.
fn field_text(value: &JsonValue, field: &str) -> String {
    value
        .get(field)
        .and_then(|found| found.as_jstring())
        .and_then(|text| text.to_rust_string())
        .unwrap_or_default()
}

/// The label a review renders under. A label that cannot be
/// printed, holding an unpaired surrogate or a line break, and
/// a review holding no usable label, render under the unnamed
/// review label.
fn label_for(review: &JsonValue) -> String {
    if let Some(text) = review
        .get("diff_label")
        .and_then(|value| value.as_jstring())
    {
        if !text.is_blank() && !text.holds_unpaired_surrogate() && !text.holds_line_break() {
            if let Some(label) = text.to_rust_string() {
                return label;
            }
        }
    }
    "unnamed review".to_string()
}

/// Render an invalid verdict with its defects named, as the
/// raw output bytes. The label reads from its code units under
/// the same handler the interpreter prints with, so a label
/// standing for a file name byte prints as that byte, as the
/// check command names an unreadable diff by the raw name the
/// caller handed over. Every other line of the rendering is
/// plain text.
pub fn render_defects_units(label: &JString, defects: &[String]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"Review of ");
    out.extend_from_slice(&label.to_surrogateescape());
    out.push(b'\n');
    let mut lines = vec![
        "Verdict, invalid".to_string(),
        format!("Defects, {}", defects.len()),
    ];
    lines.extend(defects.iter().map(|defect| format!("Defect, {defect}")));
    out.extend_from_slice((lines.join("\n") + "\n").as_bytes());
    out
}

/// Render an invalid verdict with its defects named.
pub fn render_defects(label: &str, defects: &[String]) -> String {
    let bytes = render_defects_units(&JString::from_text(label), defects);
    String::from_utf8(bytes).expect("a text label renders as text")
}

/// Render a review. The same review always renders byte
/// identical.
pub fn render_review(review: &JsonValue) -> String {
    let defects = validate_review(review);
    let label = label_for(review);
    if !defects.is_empty() {
        return render_defects(&label, &defects);
    }
    let verdict = verdict_for(review);
    let objections = review
        .get("objections")
        .and_then(|value| value.as_array())
        .map(|items| items.len())
        .unwrap_or(0);
    let mut lines = vec![
        format!("Review of {label}"),
        format!("Verdict, {verdict}"),
        format!("Objections, {objections}"),
    ];
    if let Some(items) = review.get("objections").and_then(|value| value.as_array()) {
        for (index, objection) in items.iter().enumerate() {
            lines.push(String::new());
            lines.push(format!(
                "Objection {}, {}",
                index + 1,
                field_text(objection, "category")
            ));
            lines.push(format!("Flaw, {}", field_text(objection, "flaw")));
            lines.push(format!("Standard, {}", field_text(objection, "standard")));
            lines.push(format!("Rewrite, {}", field_text(objection, "rewrite")));
        }
    }
    lines.join("\n") + "\n"
}

/// Render a malformed reading with its defects named, under
/// the head line its surface states.
fn render_defects_headed(head: &str, defects: &[String]) -> String {
    let mut lines = vec![
        head.to_string(),
        "Verdict, invalid".to_string(),
        format!("Defects, {}", defects.len()),
    ];
    lines.extend(defects.iter().map(|defect| format!("Defect, {defect}")));
    lines.join("\n") + "\n"
}

/// Render a malformed change set with its defects named.
pub fn render_set_defects(defects: &[String]) -> String {
    render_defects_headed("Change set review", defects)
}

/// Render a malformed census with its defects named.
pub fn render_census_defects(defects: &[String]) -> String {
    render_defects_headed("Census", defects)
}

/// Render a malformed rejudged pair with its defects named.
pub fn render_rejudged_defects(defects: &[String]) -> String {
    render_defects_headed("Rejudged", defects)
}

/// Render a rejudged reading under CK-14.
///
/// The reading names its two sets in asking order, states
/// each compared input in its own group with the judgement
/// of each asking and the word the pair stands under, and
/// closes with its counts. The same pair always renders
/// byte identical.
pub fn render_rejudged(reading: &Rejudged) -> String {
    let mut lines = vec![
        "Rejudged".to_string(),
        format!("First, {}", reading.first_label),
        format!("Second, {}", reading.second_label),
    ];
    for pair in &reading.pairs {
        lines.push(String::new());
        lines.push(format!(
            "{}, line {}, {}",
            pair.file, pair.line, pair.pattern
        ));
        lines.push(format!("First, {}", pair.first));
        lines.push(format!("Second, {}", pair.second));
        lines.push(if pair.held { "Held" } else { "Changed" }.to_string());
    }
    lines.push(String::new());
    lines.push(format!("Inputs read, {}", reading.inputs_read));
    lines.push(format!("Standing in both, {}", reading.pairs.len()));
    lines.push(format!("Held, {}", reading.held));
    lines.push(format!("Changed, {}", reading.changed));
    lines.join("\n") + "\n"
}

/// Render a malformed split file with its defects named.
pub fn render_split_defects(defects: &[String]) -> String {
    render_defects_headed("Split", defects)
}

/// Render a split reading under CK-16.
///
/// The reading names its two routes in the order the file
/// names them, states each flag in its own group with the
/// judgement of each route alone and the joint judgement,
/// and closes with its counts. The same file always
/// renders byte identical.
pub fn render_split(reading: &Split) -> String {
    let mut lines = vec![
        "Split".to_string(),
        format!("First route, {}", reading.first_route),
        format!("Second route, {}", reading.second_route),
    ];
    for flag in &reading.flags {
        lines.push(String::new());
        lines.push(format!(
            "{}, line {}, {}",
            flag.file, flag.line, flag.pattern
        ));
        lines.push(format!("First route, {}", flag.first));
        lines.push(format!("Second route, {}", flag.second));
        lines.push(format!("Joint, {}", flag.joint));
    }
    lines.push(String::new());
    lines.push(format!("Flags read, {}", reading.flags.len()));
    lines.push(format!("Routes agree, {}", reading.routes_agree));
    lines.push(format!("Routes differ, {}", reading.routes_differ));
    lines.push(format!(
        "Joint differs from a route, {}",
        reading.joint_differs
    ));
    lines.join("\n") + "\n"
}

/// Render a malformed contested file with its defects named.
pub fn render_contested_defects(defects: &[String]) -> String {
    render_defects_headed("Contested", defects)
}

/// Read a field the contested reading prints under the one
/// line rule of CK-18, as FIX-CK-20 closes that rule at the
/// library surface. A field holding a line break, the break
/// standing as the newline or the carriage return the
/// loaders name, prints as the empty field, the rendering
/// this reading already gives a label that stands for no
/// printable string through its reading function, so no
/// field the rendering is handed can split a line or mint
/// a structural line.
fn one_line_field(field: &str) -> &str {
    if field.contains('\n') || field.contains('\r') {
        ""
    } else {
        field
    }
}

/// Render a contested reading under CK-18.
///
/// The reading states each set of the collection in its
/// own section in the order the collection states them,
/// naming the contested flags of the set or the no
/// contested lines line where it holds none, and closes
/// with the counts of the collection. The closing counts
/// print as the sums over the sections stated, in the
/// relations CK-18 states for them, and every field prints
/// under the one line rule, so the rendering holds both
/// rules for any reading it is handed, under FIX-CK-20.
/// The same collection always renders byte identical.
pub fn render_contested(reading: &Contested) -> String {
    let mut lines = vec!["Contested".to_string()];
    for set in &reading.sets {
        lines.push(String::new());
        lines.push(format!("Set, {}", one_line_field(&set.label)));
        if set.contested.is_empty() {
            lines.push("No contested lines".to_string());
        } else {
            for flag in &set.contested {
                lines.push(format!(
                    "{}, line {}, {}",
                    one_line_field(&flag.file),
                    flag.line,
                    one_line_field(&flag.pattern)
                ));
            }
        }
        lines.push(format!("Flags read, {}", set.flags_read));
        lines.push(format!("Contested, {}", set.contested.len()));
    }
    lines.push(String::new());
    lines.push(format!("Sets read, {}", reading.sets.len()));
    lines.push(format!(
        "Flags read, {}",
        reading.sets.iter().map(|set| set.flags_read).sum::<usize>()
    ));
    lines.push(format!(
        "Contested flags, {}",
        reading
            .sets
            .iter()
            .map(|set| set.contested.len())
            .sum::<usize>()
    ));
    lines.push(format!(
        "Sets with contested flags, {}",
        reading
            .sets
            .iter()
            .filter(|set| !set.contested.is_empty())
            .count()
    ));
    lines.join("\n") + "\n"
}

/// Push the three judged lines onto a rendering.
///
/// The forms are stated once here and shared by the set
/// review summary and the census tally, so the two surfaces
/// cannot drift apart.
fn push_judged_lines(
    lines: &mut Vec<String>,
    stickler: usize,
    clean: usize,
    unjudged: usize,
    contested: usize,
) {
    lines.push(format!("Verdict lines, stickler {stickler}, clean {clean}"));
    lines.push(format!("Unjudged, {unjudged}"));
    lines.push(format!("Contested, {contested}"));
}

/// Push the count lines of one tally onto the lines of a
/// census rendering.
///
/// The judged lines stand in the shared judged forms when a
/// verdict stands in the tally, and the none recorded line
/// stands in their place when none does, under CK-11.
fn push_tally(lines: &mut Vec<String>, tally: &Tally) {
    lines.push(format!("Files, {}", tally.files));
    lines.push(format!("Judgements, {}", tally.judgements));
    for (index, shape) in SHAPES.iter().enumerate() {
        lines.push(format!("Shape, {shape}, {}", tally.shapes[index]));
    }
    lines.push(format!("Borderline, {}", tally.borderline));
    for (index, pattern) in BORDERLINE_PATTERNS.iter().enumerate() {
        lines.push(format!("Pattern, {pattern}, {}", tally.patterns[index]));
    }
    if tally.verdicts_stand {
        push_judged_lines(
            lines,
            tally.stickler,
            tally.clean,
            tally.unjudged,
            tally.contested,
        );
    } else {
        lines.push("Verdicts, none recorded".to_string());
    }
}

/// Render a census reading under CK-11.
///
/// The reading states each set of the collection in its own
/// block in the order the collection states them, and a total
/// block after the last set. The same census always renders
/// byte identical.
pub fn render_census(census: &Census) -> String {
    let mut lines = vec!["Census".to_string(), format!("Sets, {}", census.sets.len())];
    for set in &census.sets {
        lines.push(String::new());
        lines.push(format!("Set, {}", set.label));
        push_tally(&mut lines, &set.tally);
    }
    lines.push(String::new());
    lines.push("Total".to_string());
    lines.push(format!("Sets, {}", census.sets.len()));
    push_tally(&mut lines, &census.total);
    lines.join("\n") + "\n"
}

/// Render a set review from the per entry results.
///
/// The results hold one entry per file, with the judgements its
/// diff raised and its borderline blocks under CK-8. When a
/// flagged line is contested under CK-9 a contested section
/// stands before the summary and the summary counts the
/// contested lines. The same results always render byte
/// identical, and results holding no flags render exactly as
/// judgement only results render.
pub fn render_set_review(results: &[FileResult]) -> String {
    let mut lines = vec![
        "Change set review".to_string(),
        format!("Files, {}", results.len()),
    ];
    let mut total = 0usize;
    let mut counts: Vec<(&'static str, usize)> = Vec::new();
    let mut worst: Vec<(&str, usize)> = Vec::new();
    let mut flag_total = 0usize;
    let mut judged_stickler = 0usize;
    let mut judged_clean = 0usize;
    let mut judged_unjudged = 0usize;
    let mut contested: Vec<(
        &str,
        usize,
        &crate::checker::Flag,
        &[crate::verdicts::VerdictEntry],
    )> = Vec::new();
    for result in results {
        let name = &result.name;
        let judgements: &Judgements = &result.judgements;
        let blocks = &result.blocks;
        lines.push(String::new());
        lines.push(format!("File, {name}"));
        if judgements.is_empty() && blocks.is_empty() {
            lines.push("Clean".to_string());
            continue;
        }
        if !judgements.is_empty() {
            lines.push(format!("Judgements, {}", judgements.len()));
            total += judgements.len();
            worst.push((name, judgements.len()));
            for (shape, _objection) in judgements {
                match counts.iter_mut().find(|(held, _)| held == shape) {
                    Some((_held, count)) => *count += 1,
                    None => counts.push((shape, 1)),
                }
            }
            for (index, (_shape, objection)) in judgements.iter().enumerate() {
                lines.push(String::new());
                lines.push(format!("Judgement {}, {}", index + 1, objection.category));
                lines.push(format!("Flaw, {}", objection.flaw));
                lines.push(format!("Standard, {}", objection.standard));
                lines.push(format!("Rewrite, {}", objection.rewrite));
            }
        }
        for (index, block) in blocks.iter().enumerate() {
            let flag = &block.flag;
            let entries = &block.verdicts;
            flag_total += 1;
            let outcome = line_outcome(entries);
            match outcome {
                "stickler" => judged_stickler += 1,
                "clean" => judged_clean += 1,
                _ => judged_unjudged += 1,
            }
            if is_contested(entries) {
                contested.push((name, index + 1, flag, entries));
            }
            lines.push(String::new());
            lines.push(format!("Borderline {}, {}", index + 1, flag.pattern));
            lines.push(format!("Line, {}", flag.content));
            lines.push(format!("Shape reading, {}", flag.reading));
            for entry in entries {
                if verdict_outcome(entry) == "abstained" {
                    lines.push(format!("Verdict, {}, abstained", entry.model));
                } else {
                    lines.push(format!("Verdict, {}, {}", entry.model, entry.choice));
                }
                lines.push(format!("Reason, {}", reason_for(entry)));
            }
            if outcome == "unjudged" {
                lines.push("Unjudged".to_string());
            }
        }
    }
    if !contested.is_empty() {
        lines.push(String::new());
        lines.push("Contested lines".to_string());
        for (name, index, flag, entries) in &contested {
            lines.push(String::new());
            lines.push(format!("File, {name}"));
            lines.push(format!("Borderline {index}, {}", flag.pattern));
            lines.push(format!("Line, {}", flag.content));
            for entry in *entries {
                lines.push(format!(
                    "Verdict, {}, {}",
                    entry.model,
                    verdict_outcome(entry)
                ));
            }
        }
    }
    lines.push(String::new());
    lines.push("Summary".to_string());
    lines.push(format!("Judgements, {total}"));
    for shape in SHAPES {
        if let Some((_held, count)) = counts.iter().find(|(held, _)| *held == shape) {
            if *count > 0 {
                lines.push(format!("Shape, {shape}, {count}"));
            }
        }
    }
    if flag_total > 0 {
        lines.push(format!("Borderline, {flag_total}"));
        push_judged_lines(
            &mut lines,
            judged_stickler,
            judged_clean,
            judged_unjudged,
            contested.len(),
        );
    }
    if !worst.is_empty() {
        lines.push("Files with judgements, worst first".to_string());
        let mut ordered = worst.clone();
        ordered.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(right.0)));
        for (name, count) in ordered {
            lines.push(format!("Worst, {name}, {count}"));
        }
    }
    lines.join("\n") + "\n"
}
