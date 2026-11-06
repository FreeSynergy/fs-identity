//! Our own small client for Kanidm's group routes (REST v1).
//!
//! Pattern: Facade (declarations).

mod entry;
mod groups;

pub use entry::Entry;
pub use groups::KanidmClient;
