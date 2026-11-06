//! `fs-identity-kanidm`: the Kanidm adapter of the groups mirror interface (ADR 0047).
//!
//! `fs-groups` decides what groups are and who is in them; Kanidm is brought to that state through
//! its REST API v1 with a service-account token, via our own small client (no Kanidm crates). Programs
//! keep working without Kanidm (ADR 0024). Start with [`KanidmDirectory`], a `GroupDirectory` for
//! `fs_groups_core::Mirror::sync_to`. Kanidm's words (`description`, `member`, SPN) stay in this crate.
//!
//! Pattern: Facade — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
// Pedantic lints that would only add noise to this crate's small, documented API.
#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::module_name_repetitions
)]

mod client;
mod convert;
mod directory;
mod error;
mod http;
pub mod naming;
mod people;

pub use client::{Entry, KanidmClient};
pub use convert::FOREIGN;
pub use directory::KanidmDirectory;
pub use error::{Error, Result};
pub use http::{HttpSeam, Method, Request, ReqwestSeam, Response, Token};
pub use people::{PersonMap, StaticPersonMap};
