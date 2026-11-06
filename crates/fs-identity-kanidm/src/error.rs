//! Error type of the Kanidm adapter.
//!
//! Pattern: none (plain error enum via `thiserror`) — one `Error` for the client and the directory,
//! converted into the groups core's `Directory` error at the trait boundary. No error text ever holds
//! the token: requests carry none, the transport adds it.

/// Result of every fallible operation of this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Everything that can go wrong talking to Kanidm.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The request did not reach Kanidm, or the answer did not come back.
    #[error("transport: {0}")]
    Transport(String),
    /// Kanidm answered with a status that means failure.
    #[error("kanidm answered {status} to {route}: {detail}")]
    Status {
        /// HTTP status.
        status: u16,
        /// Method and path that were asked.
        route: String,
        /// The start of the answer body.
        detail: String,
    },
    /// The answer was not the shape documented for the route.
    #[error("unexpected answer from {route}: {detail}")]
    Decode {
        /// Method and path that were asked.
        route: String,
        /// What was wrong.
        detail: String,
    },
    /// People of a group have no Kanidm account in the person map; nothing was written.
    #[error("no Kanidm account is mapped for: {}", .0.join(", "))]
    UnmappedPeople(Vec<String>),
    /// An id that cannot be put into a URL path.
    #[error("not usable in a path: {0:?}")]
    BadId(String),
    /// The base URL or the certificate given by the caller is not usable.
    #[error("setup: {0}")]
    Setup(String),
}

impl From<Error> for fs_groups_core::Error {
    fn from(e: Error) -> Self {
        fs_groups_core::Error::Directory(e.to_string())
    }
}
