//! The real transport: `reqwest` with rustls.
//!
//! Pattern: Adapter — `reqwest` behind our `HttpSeam`. HTTPS only (the token must not travel in the
//! clear); a private CA is added as a root certificate, certificate checks are never switched off.

use async_trait::async_trait;
use reqwest::{Certificate, Client};
use serde_json::Value;

use super::{HttpSeam, Method, Request, Response, Token};
use crate::error::{Error, Result};

/// Talks to one Kanidm server with one service-account token.
pub struct ReqwestSeam {
    client: Client,
    base: String,
    token: Token,
}

impl ReqwestSeam {
    /// A transport for `base_url` (`https://idm.example.org`). `ca_pem` adds a private CA to the
    /// trusted roots, for a server with its own certificate authority.
    pub fn new(base_url: &str, token: Token, ca_pem: Option<&[u8]>) -> Result<Self> {
        let base = base_url.trim_end_matches('/').to_string();
        if !base.starts_with("https://") {
            return Err(Error::Setup(
                "the server URL must start with https://".into(),
            ));
        }
        let mut builder = Client::builder().timeout(std::time::Duration::from_secs(30));
        if let Some(pem) = ca_pem {
            if !String::from_utf8_lossy(pem).contains("-----BEGIN CERTIFICATE-----") {
                return Err(Error::Setup(
                    "CA certificate: no PEM certificate found".into(),
                ));
            }
            let cert = Certificate::from_pem(pem)
                .map_err(|e| Error::Setup(format!("CA certificate: {e}")))?;
            builder = builder.add_root_certificate(cert);
        }
        let client = builder.build().map_err(|e| Error::Setup(e.to_string()))?;
        Ok(Self {
            client,
            base,
            token,
        })
    }
}

impl std::fmt::Debug for ReqwestSeam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReqwestSeam")
            .field("base", &self.base)
            .field("token", &self.token)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl HttpSeam for ReqwestSeam {
    async fn send(&self, request: Request) -> Result<Response> {
        let url = format!("{}{}", self.base, request.path);
        let method = match request.method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
        };
        let mut call = self
            .client
            .request(method, url)
            .bearer_auth(self.token.expose());
        if let Some(body) = &request.body {
            call = call.json(body);
        }
        // reqwest errors can quote the URL, never the header; they hold no token.
        let answer = call
            .send()
            .await
            .map_err(|e| Error::Transport(e.without_url().to_string()))?;
        let status = answer.status().as_u16();
        let bytes = answer
            .bytes()
            .await
            .map_err(|e| Error::Transport(e.without_url().to_string()))?;
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
        };
        Ok(Response { status, body })
    }
}
