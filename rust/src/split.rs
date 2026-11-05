//! The split reading. A split reading takes one recorded
//! set and the two route model strings its verdicts were
//! recorded under, and states per flag of the set the
//! judgement each route reads over its own verdicts alone
//! and the judgement the two routes read jointly, so the
//! composition of a joint judgement stands as figures
//! rather than as prose. The loader validates the member
//! through the change set loader, the flags and verdicts
//! come from the set review, and every judgement comes
//! from the outcome helper of CK-8 read over the named
//! subset, so the reading holds no second flagging or
//! judging logic of its own. The obligations this module
//! carries are CK-15 and CK-16 in CONTRACT.md, in the
//! Round 7 section.

use crate::changeset::{
    entry_verdicts_are_read, load_changeset, review_changeset, walk_verdicts, FileResult,
};
use crate::json::JsonValue;
use crate::verdicts::line_outcome;

/// One flag of a split reading, holding its name and the
/// three judgements the reading reads over it, the first
/// route alone, the second route alone, and both jointly.
#[derive(Clone, Debug, PartialEq)]
pub struct SplitFlag {
    pub file: String,
    pub line: usize,
    pub pattern: String,
    pub first: &'static str,
    pub second: &'static str,
    pub joint: &'static str,
}

/// A whole split reading, holding the two route strings
/// in the order the file names them, the flags of the set
/// in the order the set review states them, and its
/// closing counts.
#[derive(Clone, Debug, PartialEq)]
pub struct Split {
    pub first_route: String,
    pub second_route: String,
    pub flags: Vec<SplitFlag>,
    pub routes_agree: usize,
    pub routes_differ: usize,
    pub joint_differs: usize,
}

/// Name the recorded verdicts of a member change set whose
/// model is neither of the two route strings.
///
/// The check reads the verdicts of an entry in the
/// posture CK-8 states, over the entries whose verdicts
/// the change set loader reads, and it reads a verdict
/// only when the verdict stands in the full form CK-8
/// states, because a verdict the loader already refuses
/// draws no second defect from this rule. The entry
/// posture and the verdict standings both come from the
/// loader's own home in changeset.rs, the predicate
/// entry_verdicts_are_read and the walk walk_verdicts, so
/// this check holds no second copy of that posture.
fn route_defects(member: &JsonValue, routes: &[String]) -> Vec<String> {
    let mut defects = Vec::new();
    let Some(entries) = member.get("files").and_then(|files| files.as_array()) else {
        return defects;
    };
    for (index, entry) in entries.iter().enumerate() {
        if !entry.is_object() || !entry_verdicts_are_read(entry) {
            continue;
        }
        let Some(verdicts) = entry.get("verdicts").and_then(|list| list.as_array()) else {
            continue;
        };
        let (_named, standings) = walk_verdicts(index, entry);
        for (position, verdict) in verdicts.iter().enumerate() {
            if !standings[position] {
                continue;
            }
            let Some(text) = verdict
                .get("model")
                .and_then(|value| value.as_jstring())
                .and_then(|model| model.to_rust_string())
            else {
                continue;
            };
            if !routes.iter().any(|route| route == &text) {
                defects.push(format!(
                    "files[{index}].verdicts[{position}].model names no route of the split"
                ));
            }
        }
    }
    defects
}

/// Return the list of defects found in a split file.
/// Never fails, whatever it is handed.
///
/// The routes list and the member change set are each
/// validated as CK-15 states, the member through the
/// change set loader of CK-6, and the route membership
/// of every recorded verdict is checked over the member
/// the loader read.
pub fn load_split(value: &JsonValue) -> Vec<String> {
    let mut defects = Vec::new();
    if !value.is_object() {
        return vec!["split must be a JSON object".to_string()];
    }
    let mut routes: Vec<String> = Vec::new();
    match value.get("routes") {
        None => defects.push("routes is missing".to_string()),
        Some(routes_value) => match routes_value.as_array() {
            None => defects.push("routes must be a list".to_string()),
            Some(entries) => {
                if entries.len() != 2 {
                    defects.push("routes must hold exactly two routes".to_string());
                }
                for (index, entry) in entries.iter().enumerate() {
                    let where_ = format!("routes[{index}]");
                    match entry.as_jstring() {
                        Some(text) if !text.is_blank() => {
                            if text.holds_unpaired_surrogate() {
                                defects.push(format!("{where_} holds an unpaired surrogate"));
                            } else if text.holds_line_break() {
                                defects.push(format!("{where_} holds a line break"));
                            } else if let Some(text) = text.to_rust_string() {
                                routes.push(text);
                            }
                        }
                        _ => defects.push(format!("{where_} must be a non empty string")),
                    }
                }
                if routes.len() == 2 && routes[0] == routes[1] {
                    defects.push("routes must name two different model strings".to_string());
                }
            }
        },
    }
    match value.get("change_set") {
        None => defects.push("change_set is missing".to_string()),
        Some(member) => {
            for defect in load_changeset(member) {
                defects.push(defect);
            }
            if routes.len() == 2 {
                defects.extend(route_defects(member, &routes));
            }
        }
    }
    defects
}

/// Read the flags of one set from its set review results.
///
/// One flag stands per borderline block, in the order the
/// files stand in the review and the flags stand in each
/// file, and each of its three judgements is the
/// judgement CK-8 reads over the named subset of the
/// verdicts standing on the flag.
fn flags_of(results: &[FileResult], routes: &[String; 2]) -> Vec<SplitFlag> {
    let mut flags = Vec::new();
    for result in results {
        for block in &result.blocks {
            let first: Vec<_> = block
                .verdicts
                .iter()
                .filter(|verdict| verdict.model == routes[0])
                .cloned()
                .collect();
            let second: Vec<_> = block
                .verdicts
                .iter()
                .filter(|verdict| verdict.model == routes[1])
                .cloned()
                .collect();
            flags.push(SplitFlag {
                file: result.name.clone(),
                line: block.flag.line,
                pattern: block.flag.pattern.clone(),
                first: line_outcome(&first),
                second: line_outcome(&second),
                joint: line_outcome(&block.verdicts),
            });
        }
    }
    flags
}

/// Read a valid split file into its reading.
///
/// A file the loader refused reads as the flags its
/// member still yields under the routes its entries
/// still name, in the posture the set readings hold,
/// and the command line never renders one.
pub fn split_of(value: &JsonValue) -> Split {
    let mut routes = [String::new(), String::new()];
    if let Some(entries) = value.get("routes").and_then(|list| list.as_array()) {
        for (index, entry) in entries.iter().take(2).enumerate() {
            routes[index] = entry
                .as_jstring()
                .and_then(|text| text.to_rust_string())
                .unwrap_or_default();
        }
    }
    let mut flags = Vec::new();
    if let Some(member) = value.get("change_set") {
        flags = flags_of(&review_changeset(member), &routes);
    }
    let routes_agree = flags
        .iter()
        .filter(|flag| flag.first == flag.second)
        .count();
    let routes_differ = flags.len() - routes_agree;
    let joint_differs = flags
        .iter()
        .filter(|flag| flag.joint != flag.first || flag.joint != flag.second)
        .count();
    Split {
        first_route: routes[0].clone(),
        second_route: routes[1].clone(),
        flags,
        routes_agree,
        routes_differ,
        joint_differs,
    }
}
