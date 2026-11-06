//! The entry shape of REST v1: `{"attrs": {name: [values]}}`.
//!
//! Pattern: Value object — Kanidm's stable `v1::Entry` (`proto/src/v1/mod.rs`), read with serde. Only
//! this module and `groups.rs` know Kanidm's attribute names.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// An entry as Kanidm sends and accepts it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// Attribute name to its values.
    pub attrs: BTreeMap<String, Vec<String>>,
}

impl Entry {
    /// The values of an attribute (empty if absent).
    pub fn values(&self, attr: &str) -> &[String] {
        self.attrs.get(attr).map_or(&[], Vec::as_slice)
    }

    /// The first value of an attribute.
    pub fn first(&self, attr: &str) -> Option<&str> {
        self.values(attr).first().map(String::as_str)
    }
}
