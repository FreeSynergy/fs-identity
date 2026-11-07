//! The Kanidm adapter of `GroupDirectory`, against the stand-in.

mod common;

use common::{FakeKanidm, dg, person, read_all};
use fs_groups_core::{Error, GroupDirectory};
use fs_identity_kanidm::{FOREIGN, naming::group_name};
use fs_objects::ObjectId;

#[tokio::test]
async fn apply_creates_the_group_under_its_derived_name_with_the_display_in_description() {
    let k = FakeKanidm::new();
    let d = k.directory();
    let id = ObjectId::new();
    d.apply(&dg(id, "Garden club", &["ada", "bo"], &[]))
        .await
        .unwrap();
    let s = k.stored(&group_name(id)).unwrap();
    assert_eq!(s.description.as_deref(), Some("Garden club"));
    assert_eq!(s.members, ["ada@idm.test", "bo@idm.test"]);
    assert_eq!(
        d.read(id).await.unwrap().unwrap(),
        dg(id, "Garden club", &["ada", "bo"], &[])
    );
}

#[tokio::test]
async fn nested_groups_are_members_by_their_derived_name() {
    let k = FakeKanidm::new();
    let d = k.directory();
    let (child, parent) = (ObjectId::new(), ObjectId::new());
    d.apply(&dg(child, "child", &["ada"], &[])).await.unwrap(); // children first
    d.apply(&dg(parent, "parent", &["bo"], &[child]))
        .await
        .unwrap();
    assert_eq!(
        k.stored(&group_name(parent)).unwrap().members,
        ["bo@idm.test", &format!("{}@idm.test", group_name(child))]
    );
    assert_eq!(d.read(parent).await.unwrap().unwrap().nested, [child]);
}

#[tokio::test]
async fn apply_twice_writes_nothing_the_second_time() {
    let k = FakeKanidm::new();
    let d = k.directory();
    let g = dg(ObjectId::new(), "Club", &["ada"], &[]);
    d.apply(&g).await.unwrap();
    k.clear_log();
    d.apply(&g).await.unwrap();
    assert_eq!(k.writes(), 0, "idempotent: only a read");
}

#[tokio::test]
async fn apply_changes_only_what_differs() {
    let k = FakeKanidm::new();
    let d = k.directory();
    let id = ObjectId::new();
    d.apply(&dg(id, "Club", &["ada"], &[])).await.unwrap();
    k.clear_log();
    d.apply(&dg(id, "Club renamed", &["ada"], &[]))
        .await
        .unwrap();
    assert_eq!(
        k.calls().last().unwrap(),
        &(
            "PUT".to_string(),
            format!("/v1/group/{}/_attr/description", group_name(id))
        )
    );
    assert_eq!(k.writes(), 1);
    d.apply(&dg(id, "Club renamed", &["bo", "cy"], &[]))
        .await
        .unwrap();
    assert_eq!(
        k.stored(&group_name(id)).unwrap().members,
        ["bo@idm.test", "cy@idm.test"]
    );
    d.apply(&dg(id, "Club renamed", &[], &[])).await.unwrap();
    assert!(k.stored(&group_name(id)).unwrap().members.is_empty());
}

#[tokio::test]
async fn a_hand_edit_in_kanidm_is_seen_and_put_back() {
    let k = FakeKanidm::new();
    let d = k.directory();
    let id = ObjectId::new();
    let want = dg(id, "Club", &["ada"], &[]);
    d.apply(&want).await.unwrap();
    {
        let mut groups = k.groups.lock().unwrap();
        groups[0].members.push("cy@idm.test".into()); // an account we know, added by hand
        groups[0].description = Some("hacked".into());
    }
    assert_ne!(
        d.read(id).await.unwrap().unwrap(),
        want,
        "read shows the difference"
    );
    d.apply(&want).await.unwrap();
    assert_eq!(d.read(id).await.unwrap().unwrap(), want);
}

#[tokio::test]
async fn an_account_nobody_mapped_shows_up_as_foreign_and_is_removed() {
    let k = FakeKanidm::new();
    let d = k.directory();
    let id = ObjectId::new();
    let want = dg(id, "Club", &["ada"], &[]);
    d.apply(&want).await.unwrap();
    k.groups.lock().unwrap()[0]
        .members
        .push("stranger@idm.test".into());
    let seen = d.read(id).await.unwrap().unwrap();
    assert!(
        seen.members
            .contains(&format!("{FOREIGN}stranger@idm.test"))
    );
    d.apply(&want).await.unwrap();
    assert_eq!(k.stored(&group_name(id)).unwrap().members, ["ada@idm.test"]);
}

#[tokio::test]
async fn people_without_an_account_fail_before_anything_is_written() {
    let k = FakeKanidm::new();
    let d = k.directory();
    let err = d
        .apply(&dg(ObjectId::new(), "Club", &["ada", "ghost"], &[]))
        .await
        .unwrap_err();
    assert!(
        matches!(&err, Error::Directory(m) if m.contains(&person("ghost"))),
        "{err}"
    );
    assert_eq!(k.writes(), 0);
    assert!(k.groups.lock().unwrap().is_empty());
}

#[tokio::test]
async fn remove_deletes_and_removing_an_absent_group_is_fine() {
    let k = FakeKanidm::new();
    let d = k.directory();
    let id = ObjectId::new();
    d.apply(&dg(id, "Club", &[], &[])).await.unwrap();
    d.remove(id).await.unwrap();
    assert!(d.read(id).await.unwrap().is_none());
    d.remove(id).await.unwrap();
}

#[tokio::test]
async fn list_returns_only_groups_we_made() {
    let k = FakeKanidm::new();
    let d = k.directory();
    let (a, b) = (ObjectId::new(), ObjectId::new());
    d.apply(&dg(a, "A", &[], &[])).await.unwrap();
    d.apply(&dg(b, "B", &["ada"], &[])).await.unwrap();
    k.groups.lock().unwrap().push(common::Stored {
        name: "idm_admins".into(),
        uuid: "x".into(),
        description: None,
        members: vec![],
    });
    k.groups.lock().unwrap().push(common::Stored {
        name: "my-fsg-handmade".into(),
        uuid: "y".into(),
        description: None,
        members: vec![],
    });
    let mut ids: Vec<_> = read_all(&d).await.into_iter().map(|g| g.id).collect();
    ids.sort();
    let mut want = vec![a, b];
    want.sort();
    assert_eq!(ids, want);
}

#[tokio::test]
async fn the_report_says_what_kanidm_does_not_do() {
    let d = FakeKanidm::new().directory();
    let c = d.capabilities();
    assert_eq!(c.provider(), "kanidm");
    assert!(c.is_supported("directory.apply") && c.is_supported("directory.read"));
    assert!(c.why_not("invite.create").is_some());
    assert!(c.why_not("right.grant").is_some());
}

#[tokio::test]
async fn a_server_failure_comes_back_as_a_directory_error_without_a_token() {
    let k = FakeKanidm::new();
    let d = k.directory();
    let id = ObjectId::new();
    let err = d
        .apply(&dg(id, "Parent", &[], &[ObjectId::new()]))
        .await
        .unwrap_err(); // nested group missing
    let Error::Directory(text) = err else {
        panic!("expected a directory error")
    };
    assert!(text.contains("POST /v1/group"), "{text}");
}
