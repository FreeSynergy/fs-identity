//! A stand-in for a Kanidm server that behaves as the source of v1.12.0 reads: names, SPNs and UUIDs
//! all address a group, `member` values are resolved and answered as SPNs, a missing group on delete
//! is 404, a name is unique (409). It records every request, so tests can check routes and bodies.

#![allow(dead_code)] // each test file uses a different subset

use fs_groups_core::{DirectoryGroup, GroupDirectory};
use fs_identity_kanidm::{KanidmDirectory, StaticPersonMap};

mod fake;
#[allow(unused_imports)]
pub use fake::{FakeKanidm, Stored};

pub const REALM: &str = "idm.test";

pub fn person(n: &str) -> String {
    format!("https://example.org/people/{n}")
}

/// A map in which `ada`, `bo` and `cy` have accounts `ada@idm.test`, ...
pub fn people() -> StaticPersonMap {
    ["ada", "bo", "cy"]
        .into_iter()
        .fold(StaticPersonMap::new(), |m, n| {
            m.with(person(n), format!("{n}@{REALM}"))
        })
}

pub fn dg(
    id: fs_objects::ObjectId,
    name: &str,
    members: &[&str],
    nested: &[fs_objects::ObjectId],
) -> DirectoryGroup {
    DirectoryGroup {
        id,
        name: name.into(),
        members: members.iter().map(|m| person(m)).collect(),
        nested: nested.to_vec(),
    }
    .normalised()
}

pub async fn read_all(d: &KanidmDirectory) -> Vec<DirectoryGroup> {
    d.list().await.unwrap()
}
