//! Directory group <-> Kanidm entry.
//!
//! Pattern: Adapter — the one place that maps our vocabulary (name, members, nested) to Kanidm's
//! attributes (`name`, `description`, `member`) and back. Kanidm vocabulary stops here.

use std::collections::BTreeMap;

use fs_groups_core::DirectoryGroup;

use crate::client::Entry;
use crate::error::{Error, Result};
use crate::naming::{group_id, group_name};
use crate::people::PersonMap;

/// What `read` returns for a member Kanidm has that we cannot map to a person: the raw value after
/// this prefix. It never equals a wanted member, so the difference is seen and healed.
pub const FOREIGN: &str = "kanidm:";

/// The attributes to send for a group. Fails, before anything is written, if a person has no account.
pub fn to_attrs(
    group: &DirectoryGroup,
    people: &dyn PersonMap,
) -> Result<BTreeMap<String, Vec<String>>> {
    let (mut accounts, mut missing) = (vec![], vec![]);
    for person in &group.members {
        match people.account_of(person) {
            Some(a) => accounts.push(a),
            None => missing.push(person.clone()),
        }
    }
    if !missing.is_empty() {
        return Err(Error::UnmappedPeople(missing));
    }
    accounts.extend(group.nested.iter().map(|id| group_name(*id)));
    let mut attrs = BTreeMap::new();
    attrs.insert("name".to_string(), vec![group_name(group.id)]);
    attrs.insert("description".to_string(), vec![group.name.clone()]);
    attrs.insert("member".to_string(), accounts);
    Ok(attrs)
}

/// The member values to send for a group (the same as in [`to_attrs`]).
pub fn member_values(group: &DirectoryGroup, people: &dyn PersonMap) -> Result<Vec<String>> {
    Ok(to_attrs(group, people)?
        .remove("member")
        .unwrap_or_default())
}

/// Read a Kanidm entry as a directory group; `None` if it is not one of ours (name without our prefix).
pub fn from_entry(entry: &Entry, people: &dyn PersonMap) -> Option<DirectoryGroup> {
    let id = group_id(entry.first("name").or_else(|| entry.first("spn"))?)?;
    let mut group = DirectoryGroup {
        id,
        name: entry.first("description").unwrap_or_default().to_string(),
        members: vec![],
        nested: vec![],
    };
    for value in entry.values("member") {
        if let Some(child) = group_id(value) {
            group.nested.push(child);
        } else if let Some(person) = people.person_of(value) {
            group.members.push(person);
        } else {
            group.members.push(format!("{FOREIGN}{value}"));
        }
    }
    Some(group.normalised())
}
