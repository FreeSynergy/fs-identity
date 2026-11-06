//! [`KanidmDirectory`]: Kanidm as a `GroupDirectory` (ADR 0047).
//!
//! Pattern: Adapter (+ Strategy) — the groups core says what should be; this brings Kanidm there over
//! REST v1 through the HTTP seam. `apply` reads first and writes only what differs, so a repeat is
//! quiet and a hand edit in Kanidm is put back. Cycles are refused in fs-groups, not here.

use std::sync::Arc;

use async_trait::async_trait;
use fs_groups_core::{DirectoryGroup, GroupDirectory, Result};
use fs_interfaces::Capabilities;
use fs_store::ObjectId;

use crate::client::KanidmClient;
use crate::convert::{from_entry, member_values, to_attrs};
use crate::http::HttpSeam;
use crate::naming::{PREFIX, group_name};
use crate::people::PersonMap;

/// Kanidm as a directory that mirrors our groups.
#[derive(Clone)]
pub struct KanidmDirectory {
    client: KanidmClient,
    people: Arc<dyn PersonMap>,
}

impl KanidmDirectory {
    /// A directory over a transport and a person map.
    pub fn new(seam: Arc<dyn HttpSeam>, people: Arc<dyn PersonMap>) -> Self {
        Self {
            client: KanidmClient::new(seam),
            people,
        }
    }
}

#[async_trait]
impl GroupDirectory for KanidmDirectory {
    fn capabilities(&self) -> Capabilities {
        let mut c = Capabilities::new("kanidm");
        c.supported("directory.apply").supported("directory.read");
        c.unsupported(
            "invite.create",
            "Kanidm has no invitations; fs-groups keeps them (kanidm#4659)",
        );
        c.unsupported(
            "right.grant",
            "Kanidm delegation is all or nothing; fs-groups keeps the finer rights (kanidm#4641)",
        );
        c
    }

    async fn read(&self, id: ObjectId) -> Result<Option<DirectoryGroup>> {
        let entry = self.client.group(&group_name(id)).await?;
        Ok(entry.and_then(|e| from_entry(&e, self.people.as_ref())))
    }

    async fn list(&self) -> Result<Vec<DirectoryGroup>> {
        let found = self.client.search_groups(PREFIX).await?;
        Ok(found
            .iter()
            .filter_map(|e| from_entry(e, self.people.as_ref()))
            .collect())
    }

    async fn apply(&self, group: &DirectoryGroup) -> Result<()> {
        let wanted = group.clone().normalised();
        let name = group_name(wanted.id);
        let attrs = to_attrs(&wanted, self.people.as_ref())?; // fails before any write
        let Some(entry) = self.client.group(&name).await? else {
            return Ok(self.client.create_group(&attrs).await?);
        };
        let Some(current) = from_entry(&entry, self.people.as_ref()) else {
            return Ok(()); // cannot happen: the name is ours; nothing to compare
        };
        if current.name != wanted.name {
            self.client
                .set_attr(&name, "description", std::slice::from_ref(&wanted.name))
                .await?;
        }
        if current.members != wanted.members || current.nested != wanted.nested {
            let members = member_values(&wanted, self.people.as_ref())?;
            self.client.set_attr(&name, "member", &members).await?;
        }
        Ok(())
    }

    async fn remove(&self, id: ObjectId) -> Result<()> {
        self.client.delete_group(&group_name(id)).await?;
        Ok(())
    }
}
