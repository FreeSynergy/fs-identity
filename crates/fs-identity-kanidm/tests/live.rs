//! Against a real Kanidm server. Ignored by default; run with
//! `cargo test -p fs-identity-kanidm --test live -- --ignored`.
//!
//! Environment (all read here, none by the library):
//! * `FS_KANIDM_URL`: `https://localhost:18943`
//! * `FS_KANIDM_TOKEN`: a read-write service-account token in `idm_group_admins`
//! * `FS_KANIDM_CA_FILE`: PEM file of the CA that signed the server (optional)
//! * `FS_KANIDM_ACCOUNT`: SPN of an existing account to use as a member, e.g. `idm_admin@localhost`

use std::sync::Arc;

use fs_groups_core::{DirectoryGroup, GroupDirectory};
use fs_identity_kanidm::{
    KanidmClient, KanidmDirectory, ReqwestSeam, StaticPersonMap, Token, naming::group_name,
};
use fs_objects::ObjectId;

fn var(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("set {name} to run the live test"))
}

#[tokio::test]
#[ignore = "needs a running Kanidm; see the file header"]
async fn a_real_kanidm_mirrors_heals_and_cleans_up() {
    let ca = std::env::var("FS_KANIDM_CA_FILE")
        .ok()
        .map(|p| std::fs::read(p).unwrap());
    let seam = Arc::new(
        ReqwestSeam::new(
            &var("FS_KANIDM_URL"),
            Token::new(var("FS_KANIDM_TOKEN")),
            ca.as_deref(),
        )
        .unwrap(),
    );
    let account = var("FS_KANIDM_ACCOUNT");
    let person = "https://example.org/people/live".to_string();
    let people = StaticPersonMap::new().with(person.clone(), account.clone());
    let directory = KanidmDirectory::new(seam.clone(), Arc::new(people));
    let client = KanidmClient::new(seam);

    let (child, parent) = (ObjectId::new(), ObjectId::new());
    let child_g = DirectoryGroup {
        id: child,
        name: "Live child".into(),
        members: vec![person.clone()],
        nested: vec![],
    };
    let parent_g = DirectoryGroup {
        id: parent,
        name: "Live parent".into(),
        members: vec![],
        nested: vec![child],
    };

    directory.apply(&child_g).await.unwrap();
    directory.apply(&parent_g).await.unwrap();
    assert_eq!(directory.read(child).await.unwrap().unwrap(), child_g);
    assert_eq!(directory.read(parent).await.unwrap().unwrap(), parent_g);

    directory.apply(&parent_g).await.unwrap(); // idempotent
    assert_eq!(directory.read(parent).await.unwrap().unwrap(), parent_g);

    let listed: Vec<_> = directory
        .list()
        .await
        .unwrap()
        .into_iter()
        .map(|g| g.id)
        .collect();
    assert!(listed.contains(&child) && listed.contains(&parent));

    // a hand edit: empty the member list, rename the description
    client
        .set_attr(&group_name(child), "member", &[])
        .await
        .unwrap();
    client
        .set_attr(&group_name(child), "description", &["hacked".to_string()])
        .await
        .unwrap();
    assert_ne!(directory.read(child).await.unwrap().unwrap(), child_g);
    directory.apply(&child_g).await.unwrap();
    assert_eq!(directory.read(child).await.unwrap().unwrap(), child_g);

    directory.remove(parent).await.unwrap();
    directory.remove(child).await.unwrap();
    assert!(directory.read(parent).await.unwrap().is_none());
    directory.remove(child).await.unwrap(); // removing an absent group is fine

    // the same name again after a delete (Kanidm keeps deleted entries in a recycle bin first)
    directory.apply(&child_g).await.unwrap();
    assert_eq!(directory.read(child).await.unwrap().unwrap(), child_g);
    directory.remove(child).await.unwrap();
}
