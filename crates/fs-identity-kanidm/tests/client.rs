//! The routes, methods and bodies of the client, as recorded by the Kanidm stand-in.

mod common;

use std::collections::BTreeMap;

use common::FakeKanidm;
use fs_identity_kanidm::{Error, KanidmClient, Method};
use serde_json::json;

fn attrs(name: &str) -> BTreeMap<String, Vec<String>> {
    BTreeMap::from([
        ("name".to_string(), vec![name.to_string()]),
        ("description".to_string(), vec!["Display".to_string()]),
    ])
}

#[tokio::test]
async fn every_route_is_the_one_in_kanidms_source() {
    let k = FakeKanidm::new();
    let c = KanidmClient::new(k.clone());
    c.create_group(&attrs("g1")).await.unwrap();
    c.group("g1").await.unwrap().unwrap();
    c.search_groups("g").await.unwrap();
    c.set_attr("g1", "description", &["New".into()])
        .await
        .unwrap();
    c.add_attr("g1", "member", &["ada@idm.test".into()])
        .await
        .unwrap();
    c.remove_attr("g1", "member", Some(&["ada@idm.test".into()]))
        .await
        .unwrap();
    c.remove_attr("g1", "member", None).await.unwrap();
    assert!(c.delete_group("g1").await.unwrap());
    let calls = k.calls();
    let want = [
        ("POST", "/v1/group"),
        ("GET", "/v1/group/g1"),
        ("GET", "/v1/group/_search/g"),
        ("PUT", "/v1/group/g1/_attr/description"),
        ("POST", "/v1/group/g1/_attr/member"),
        ("DELETE", "/v1/group/g1/_attr/member"),
        ("DELETE", "/v1/group/g1/_attr/member"),
        ("DELETE", "/v1/group/g1"),
    ];
    assert_eq!(calls, want.map(|(m, p)| (m.to_string(), p.to_string())));
}

#[tokio::test]
async fn the_bodies_have_the_documented_shape() {
    let k = FakeKanidm::new();
    let c = KanidmClient::new(k.clone());
    c.create_group(&attrs("g1")).await.unwrap();
    c.add_attr("g1", "member", &["ada@idm.test".into()])
        .await
        .unwrap();
    c.remove_attr("g1", "member", None).await.unwrap();
    let log = k.log.lock().unwrap();
    assert_eq!(
        log[0].body,
        Some(json!({"attrs": {"name": ["g1"], "description": ["Display"]}}))
    );
    assert_eq!(
        log[1].body,
        Some(json!(["ada@idm.test"])),
        "a list of strings, as Vec<String>"
    );
    assert_eq!(log[2].method, Method::Delete);
    assert_eq!(log[2].body, None, "no body removes the whole attribute");
}

#[tokio::test]
async fn an_absent_group_reads_as_none_and_deleting_it_is_not_an_error() {
    let c = KanidmClient::new(FakeKanidm::new());
    assert!(c.group("nope").await.unwrap().is_none());
    assert!(!c.delete_group("nope").await.unwrap());
}

#[tokio::test]
async fn a_failure_status_is_an_error_with_the_route() {
    let c = KanidmClient::new(FakeKanidm::new());
    c.create_group(&attrs("g1")).await.unwrap();
    let Err(Error::Status { status, route, .. }) = c.create_group(&attrs("g1")).await else {
        panic!("expected the 409")
    };
    assert_eq!((status, route.as_str()), (409, "POST /v1/group"));
    assert!(matches!(
        c.set_attr("missing", "description", &[]).await,
        Err(Error::Status { status: 404, .. })
    ));
}

#[tokio::test]
async fn an_id_that_would_change_the_route_is_refused_before_sending() {
    let k = FakeKanidm::new();
    let c = KanidmClient::new(k.clone());
    for bad in ["", "a/b", "a?x=1", "a#b", "a b", "a%2fb"] {
        assert!(
            matches!(c.group(bad).await, Err(Error::BadId(_))),
            "{bad:?}"
        );
    }
    assert_eq!(
        k.calls(),
        [] as [(std::string::String, std::string::String); 0]
    );
}
