//! The bearer token of a service account.
//!
//! Pattern: Newtype — a string that cannot be printed by accident: `Debug` hides it and there is no
//! `Display`. The caller passes it in; this crate never reads it from a file or the environment.

/// A Kanidm API token (service account, read-write).
#[derive(Clone)]
pub struct Token(String);

impl Token {
    /// Wrap a token the caller holds.
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    /// The secret text. Only the transport that sends it should call this.
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Token(***)")
    }
}
