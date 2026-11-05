//! The set entry validation shared by the two collection
//! loaders. A census file under CK-12 and a rejudged file
//! under CK-14 both hold a sets list of entries in one
//! form, each entry an object holding a label and a change
//! set embedded whole, and the validation of one entry
//! stands here once, called by both loaders, so the two
//! surfaces can never drift apart on the defects they name
//! for an entry or for a member of an entry. The
//! obligations this module carries are CK-12 and CK-14 in
//! CONTRACT.md, in the Round 5 and Round 6 sections.

use crate::changeset::load_changeset;
use crate::json::JsonValue;

/// Name one defect of a member change set in a collection
/// list, in the form CK-12 states for the census.
///
/// The position of the set stands at the head of the defect
/// as a path, and the member defect naming the whole member
/// value stands without the member files word it does not
/// carry.
pub fn member_defect(index: usize, defect: &str) -> String {
    if defect == "change set must be a JSON object" {
        format!("sets[{index}].change_set must be a JSON object")
    } else {
        format!("sets[{index}].change_set.{defect}")
    }
}

/// Validate one entry of a sets list and push every defect
/// it draws onto the list it is handed.
///
/// An entry that is not an object draws its one defect and
/// is read no further. The label of an entry is held to one
/// line and to the non empty string form, and the change
/// set of an entry is validated through the change set
/// loader of CK-6, each of its defects named here under
/// the path of the set.
pub fn validate_entry(entry: &JsonValue, index: usize, defects: &mut Vec<String>) {
    let where_ = format!("sets[{index}]");
    if !entry.is_object() {
        defects.push(format!("{where_} must be an object"));
        return;
    }
    match entry.get("label") {
        None => defects.push(format!("{where_}.label is missing")),
        Some(label) => match label.as_jstring() {
            Some(text) if !text.is_blank() => {
                if text.holds_unpaired_surrogate() {
                    defects.push(format!("{where_}.label holds an unpaired surrogate"));
                } else if text.holds_line_break() {
                    defects.push(format!("{where_}.label holds a line break"));
                }
            }
            _ => defects.push(format!("{where_}.label must be a non empty string")),
        },
    }
    match entry.get("change_set") {
        None => defects.push(format!("{where_}.change_set is missing")),
        Some(member) => {
            for defect in load_changeset(member) {
                defects.push(member_defect(index, &defect));
            }
        }
    }
}
