//! The key: our group id <-> the Kanidm group name.
//!
//! Pattern: Adapter (pure functions) — Kanidm keeps no foreign id on a group and offers no custom
//! attributes (research note, section 3), so the id has to live in something Kanidm does keep. The
//! group *name* is the only unique, immutable-by-us, human-visible, API-addressable key
//! (`GET /v1/group/{name}`), so the name is derived from the id: `fsg-<uuid>`.
//!
//! Why this and not the alternatives: the Kanidm UUID is assigned by Kanidm (we would need our own
//! table of ids, which breaks when Kanidm is restored); `description` is free text that people edit
//! and is not unique; a name derived from the display name changes on rename and collides. The name
//! rule of Kanidm is `^[a-z][a-z0-9-_.]{0,63}$` (`server/lib/src/value.rs`, `INAME_RE`): `fsg-` plus a
//! hyphenated UUID is 40 characters and fits. The prefix lets `list` find our groups with one search
//! and keeps us away from groups made by hand. The display name goes to `description`.

use fs_store::ObjectId;

/// What every group name written by this crate starts with.
pub const PREFIX: &str = "fsg-";

/// The Kanidm group name of a group id.
pub fn group_name(id: ObjectId) -> String {
    format!("{PREFIX}{id}")
}

/// The group id behind a Kanidm name (`fsg-<uuid>`) or SPN (`fsg-<uuid>@domain`), if it is one of ours.
pub fn group_id(name_or_spn: &str) -> Option<ObjectId> {
    let name = name_or_spn.split('@').next()?;
    name.strip_prefix(PREFIX)?.parse().ok()
}
