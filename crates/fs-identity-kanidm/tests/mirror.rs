//! fs-groups' `sync_to` driving the Kanidm adapter: the whole path, without a server.

mod common;

use std::sync::Arc;

use async_trait::async_trait;
use common::{FakeKanidm, person};
use fs_groups_core::{AiMarking, GroupRepo, Groups, Member, Mirror};
use fs_identity_kanidm::naming::group_name;
use fs_objects::{Store, TursoStore};

/// Everybody in these tests is a person.
struct Humans;

#[async_trait]
impl AiMarking for Humans {
    async fn is_ai(&self, _person: &str) -> fs_groups_core::Result<Option<bool>> {
        Ok(Some(false))
    }
}

async fn repo() -> GroupRepo {
    let store: Arc<dyn Store> = Arc::new(TursoStore::open_in_memory().await.unwrap());
    GroupRepo::new(store, "https://example.org/users/owner").with_ai_marking(Arc::new(Humans))
}

#[tokio::test]
async fn groups_nested_and_filled_reach_kanidm_and_a_second_sync_is_quiet() {
    let (r, k) = (repo().await, FakeKanidm::new());
    let d = k.directory();
    let (dept, team) = (
        r.create("dept", "department").await.unwrap(),
        r.create("team", "team").await.unwrap(),
    );
    let request = r.request_parent(team.id, dept.id).await.unwrap();
    r.approve_parent(request.id).await.unwrap();
    r.add(team.id, Member::Person(person("ada")), "member")
        .await
        .unwrap();
    r.add(dept.id, Member::Person(person("bo")), "")
        .await
        .unwrap();

    let first = r.sync_to(&d).await.unwrap();
    assert_eq!(
        first.created,
        [team.id, dept.id],
        "children first, so the nested group exists"
    );
    assert!(first.failed.is_empty(), "{:?}", first.failed);
    assert_eq!(
        k.stored(&group_name(team.id)).unwrap().members,
        ["ada@idm.test"]
    );
    assert_eq!(
        k.stored(&group_name(dept.id))
            .unwrap()
            .description
            .as_deref(),
        Some("dept")
    );

    k.clear_log();
    let second = r.sync_to(&d).await.unwrap();
    assert!(second.is_quiet());
    assert_eq!(k.writes(), 0);
}

#[tokio::test]
#[allow(clippy::many_single_char_names)] // r, k, d, a, b: repo, fake Kanidm, directory and the two groups of one story
async fn a_hand_edit_is_healed_and_a_deleted_group_leaves_kanidm() {
    let (r, k) = (repo().await, FakeKanidm::new());
    let d = k.directory();
    let (a, b) = (
        r.create("a", "team").await.unwrap(),
        r.create("b", "team").await.unwrap(),
    );
    r.add(a.id, Member::Person(person("ada")), "")
        .await
        .unwrap();
    r.sync_to(&d).await.unwrap();

    k.groups
        .lock()
        .unwrap()
        .iter_mut()
        .find(|g| g.name == group_name(a.id))
        .unwrap()
        .members
        .clear();
    let healed = r.sync_to(&d).await.unwrap();
    assert_eq!(healed.updated, [a.id]);
    assert_eq!(
        k.stored(&group_name(a.id)).unwrap().members,
        ["ada@idm.test"]
    );

    Groups::delete(&r, b.id).await.unwrap();
    let removed = r.sync_to(&d).await.unwrap();
    assert_eq!(removed.removed, [b.id]);
    assert!(k.stored(&group_name(b.id)).is_none());
    assert!(k.stored(&group_name(a.id)).is_some());
}

#[tokio::test]
async fn a_group_made_by_hand_in_kanidm_is_never_touched() {
    let (r, k) = (repo().await, FakeKanidm::new());
    let d = k.directory();
    k.groups.lock().unwrap().push(common::Stored {
        name: "idm_admins".into(),
        uuid: "z".into(),
        description: None,
        members: vec![],
    });
    r.create("a", "team").await.unwrap();
    r.sync_to(&d).await.unwrap();
    r.sync_to(&d).await.unwrap();
    assert!(k.stored("idm_admins").is_some());
}

#[tokio::test]
async fn a_person_without_an_account_fails_that_group_only() {
    let (r, k) = (repo().await, FakeKanidm::new());
    let d = k.directory();
    let (good, bad) = (
        r.create("good", "team").await.unwrap(),
        r.create("bad", "team").await.unwrap(),
    );
    r.add(good.id, Member::Person(person("ada")), "")
        .await
        .unwrap();
    r.add(bad.id, Member::Person(person("ghost")), "")
        .await
        .unwrap();
    let report = r.sync_to(&d).await.unwrap();
    assert_eq!(report.created, [good.id]);
    assert_eq!(report.failed.len(), 1);
    assert_eq!(report.failed[0].0, bad.id);
    assert!(report.failed[0].1.contains("ghost"));
}
