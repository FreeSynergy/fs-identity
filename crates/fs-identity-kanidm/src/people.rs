//! Which Kanidm account is which person.
//!
//! Pattern: Strategy — the program that knows both worlds (people in fs-people, accounts in Kanidm)
//! supplies the map; this crate guesses nothing. `account_of` gives the string sent in `member`
//! (an SPN `name@domain` or a UUID); `person_of` reads what Kanidm sends back, which is the SPN.

use std::collections::BTreeMap;

/// The mapping between person IRIs (as fs-groups holds them) and Kanidm accounts.
pub trait PersonMap: Send + Sync {
    /// The account for a person, as it should be sent to Kanidm. `None` if the person has none.
    fn account_of(&self, person: &str) -> Option<String>;
    /// The person for a member string that Kanidm sent back (an SPN). `None` if unknown.
    fn person_of(&self, member: &str) -> Option<String>;
}

/// A fixed map, built by the caller. Give the SPN (`ada@idm.example.org`), since that is what
/// Kanidm returns; a UUID here would never match what is read back.
#[derive(Clone, Debug, Default)]
pub struct StaticPersonMap {
    forward: BTreeMap<String, String>,
    backward: BTreeMap<String, String>,
}

impl StaticPersonMap {
    /// An empty map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one person and their account.
    #[must_use]
    pub fn with(mut self, person: impl Into<String>, account: impl Into<String>) -> Self {
        let (person, account) = (person.into(), account.into());
        self.backward.insert(account.clone(), person.clone());
        self.forward.insert(person, account);
        self
    }
}

impl PersonMap for StaticPersonMap {
    fn account_of(&self, person: &str) -> Option<String> {
        self.forward.get(person).cloned()
    }

    fn person_of(&self, member: &str) -> Option<String> {
        self.backward.get(member).cloned()
    }
}
