//! Mirror one group into a Kanidm server.
//!
//! `FS_KANIDM_URL=https://idm.example.org FS_KANIDM_TOKEN=... [FS_KANIDM_CA_FILE=ca.pem] \
//!  FS_KANIDM_ACCOUNT=ada@idm.example.org cargo run -p fs-identity-kanidm --example mirror`

use std::sync::Arc;

use fs_groups_core::{DirectoryGroup, GroupDirectory};
use fs_identity_kanidm::{KanidmDirectory, ReqwestSeam, StaticPersonMap, Token};
use fs_objects::ObjectId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (Ok(url), Ok(token), Ok(account)) = (
        std::env::var("FS_KANIDM_URL"),
        std::env::var("FS_KANIDM_TOKEN"),
        std::env::var("FS_KANIDM_ACCOUNT"),
    ) else {
        println!("set FS_KANIDM_URL, FS_KANIDM_TOKEN and FS_KANIDM_ACCOUNT (see the file header)");
        return Ok(());
    };
    let ca = std::env::var("FS_KANIDM_CA_FILE")
        .ok()
        .map(std::fs::read)
        .transpose()?;
    let seam = ReqwestSeam::new(&url, Token::new(token), ca.as_deref())?;
    let ada = "https://example.org/people/ada".to_string();
    let people = StaticPersonMap::new().with(ada.clone(), account);
    let directory = KanidmDirectory::new(Arc::new(seam), Arc::new(people));

    let group = DirectoryGroup {
        id: ObjectId::new(),
        name: "Garden club".into(),
        members: vec![ada],
        nested: vec![],
    };
    directory.apply(&group).await?;
    println!("mirrored: {:?}", directory.read(group.id).await?);
    directory.remove(group.id).await?;
    Ok(())
}
