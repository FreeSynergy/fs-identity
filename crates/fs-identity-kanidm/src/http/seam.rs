//! The seam between the client and the network.
//!
//! Pattern: Strategy (+ Adapter for the real one) — the client talks to `dyn HttpSeam`; tests plug in
//! a Kanidm stand-in, production plugs in `ReqwestSeam`. A request carries no credentials: the
//! transport adds the token, so nothing the client logs or returns can hold it.

use async_trait::async_trait;
use serde_json::Value;

use crate::error::Result;

/// The HTTP methods Kanidm's group routes use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// Read.
    Get,
    /// Create, or append values.
    Post,
    /// Replace values.
    Put,
    /// Delete, or remove values.
    Delete,
}

impl Method {
    /// The method name.
    pub fn as_str(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
        }
    }
}

/// One request: a method, a path below the server root (`/v1/...`) and an optional JSON body.
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    /// The method.
    pub method: Method,
    /// Path with leading slash.
    pub path: String,
    /// JSON body, if any.
    pub body: Option<Value>,
}

impl Request {
    /// `"METHOD /path"`, for error messages.
    pub fn route(&self) -> String {
        format!("{} {}", self.method.as_str(), self.path)
    }
}

/// An answer: the status and the JSON body (`Null` when it had none).
#[derive(Clone, Debug, PartialEq)]
pub struct Response {
    /// HTTP status code.
    pub status: u16,
    /// Parsed body; `Null` if empty, a string with the text if it was not JSON.
    pub body: Value,
}

/// Something that sends a request to a Kanidm server.
#[async_trait]
pub trait HttpSeam: Send + Sync {
    /// Send the request. A non-2xx status is an `Ok(Response)`; only a failure to talk is an `Err`.
    async fn send(&self, request: Request) -> Result<Response>;
}
