//! The group routes of Kanidm REST v1, checked against `server/core/src/https/v1.rs` at v1.12.0.
//!
//! Pattern: Facade over the HTTP seam — one small type that knows the routes, the bodies and what a
//! status means. The routes used:
//!
//! | Call | Route |
//! |---|---|
//! | create | `POST /v1/group` with `{"attrs": {...}}` |
//! | read | `GET /v1/group/{id}` (200 with `null` when absent; `id` is name, SPN or UUID) |
//! | search | `GET /v1/group/_search/{fragment}` (names that contain the fragment) |
//! | delete | `DELETE /v1/group/{id}` (404 when absent) |
//! | attribute | `PUT` replaces, `POST` appends, `DELETE` removes (values in the body, or all) on `/v1/group/{id}/_attr/{attr}` |

use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::{Value, json};

use super::Entry;
use crate::error::{Error, Result};
use crate::http::{HttpSeam, Method, Request, Response};

/// Kanidm's group routes over a transport.
#[derive(Clone)]
pub struct KanidmClient {
    seam: Arc<dyn HttpSeam>,
}

impl KanidmClient {
    /// A client over `seam`.
    pub fn new(seam: Arc<dyn HttpSeam>) -> Self {
        Self { seam }
    }

    /// `POST /v1/group`: create a group with these attributes (`name`, `description`, `member`).
    pub async fn create_group(&self, attrs: &BTreeMap<String, Vec<String>>) -> Result<()> {
        let body = json!({ "attrs": attrs });
        self.call(Method::Post, "/v1/group".into(), Some(body))
            .await
            .map(|_| ())
    }

    /// `GET /v1/group/{id}`: the group, or `None` if there is none.
    pub async fn group(&self, id: &str) -> Result<Option<Entry>> {
        let route = format!("/v1/group/{}", segment(id)?);
        let answer = self.call(Method::Get, route.clone(), None).await?;
        decode(&format!("GET {route}"), answer.body)
    }

    /// `GET /v1/group/_search/{fragment}`: groups whose name contains the fragment.
    pub async fn search_groups(&self, fragment: &str) -> Result<Vec<Entry>> {
        let route = format!("/v1/group/_search/{}", segment(fragment)?);
        let answer = self.call(Method::Get, route.clone(), None).await?;
        decode(&format!("GET {route}"), answer.body)
    }

    /// `DELETE /v1/group/{id}`: `true` if it was deleted, `false` if it was not there.
    pub async fn delete_group(&self, id: &str) -> Result<bool> {
        let route = format!("/v1/group/{}", segment(id)?);
        match self.call(Method::Delete, route, None).await {
            Ok(_) => Ok(true),
            Err(Error::Status { status: 404, .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// `PUT /v1/group/{id}/_attr/{attr}`: replace all values of the attribute.
    pub async fn set_attr(&self, id: &str, attr: &str, values: &[String]) -> Result<()> {
        self.attr_call(Method::Put, id, attr, Some(values)).await
    }

    /// `POST /v1/group/{id}/_attr/{attr}`: add values to the attribute.
    pub async fn add_attr(&self, id: &str, attr: &str, values: &[String]) -> Result<()> {
        self.attr_call(Method::Post, id, attr, Some(values)).await
    }

    /// `DELETE /v1/group/{id}/_attr/{attr}`: remove these values, or all of them when `None`.
    pub async fn remove_attr(&self, id: &str, attr: &str, values: Option<&[String]>) -> Result<()> {
        self.attr_call(Method::Delete, id, attr, values).await
    }

    async fn attr_call(
        &self,
        method: Method,
        id: &str,
        attr: &str,
        values: Option<&[String]>,
    ) -> Result<()> {
        let route = format!("/v1/group/{}/_attr/{}", segment(id)?, segment(attr)?);
        self.call(method, route, values.map(|v| json!(v)))
            .await
            .map(|_| ())
    }

    async fn call(&self, method: Method, path: String, body: Option<Value>) -> Result<Response> {
        let request = Request { method, path, body };
        let route = request.route();
        let answer = self.seam.send(request).await?;
        if (200..300).contains(&answer.status) {
            return Ok(answer);
        }
        let mut detail = answer.body.to_string();
        detail.truncate(300);
        Err(Error::Status {
            status: answer.status,
            route,
            detail,
        })
    }
}

/// An id or attribute name as one path segment; refuses what would change the route.
fn segment(s: &str) -> Result<&str> {
    let bad = s.is_empty()
        || s.chars()
            .any(|c| matches!(c, '/' | '?' | '#' | '%') || c.is_whitespace() || c.is_control());
    if bad {
        Err(Error::BadId(s.into()))
    } else {
        Ok(s)
    }
}

fn decode<T: serde::de::DeserializeOwned>(route: &str, body: Value) -> Result<T> {
    serde_json::from_value(body).map_err(|e| Error::Decode {
        route: route.into(),
        detail: e.to_string(),
    })
}
