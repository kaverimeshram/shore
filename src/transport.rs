//! HTTP transports for 1Click. `Tor` runs every request over its own isolated Arti circuit
//! so two swap parts never share an exit or a connection.
use anyhow::{anyhow, Result};
use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, header, Uri};
use std::path::Path;
use zcash_client_backend::tor;

/// Exit failures (hostname lookup, stream launch) are transient; a retry takes a new circuit
/// that is still isolated from every other part.
const RETRIES: u8 = 3;

fn retry(r: Result<hyper::StatusCode, &tor::Error>) -> Option<tor::http::Retry> {
    r.is_err().then_some(tor::http::Retry::Isolated)
}

#[derive(Clone)]
pub enum Transport {
    Direct(reqwest::Client),
    Tor(tor::Client),
}

impl Transport {
    pub fn direct() -> Result<Self> {
        Ok(Self::Direct(reqwest::Client::builder().timeout(std::time::Duration::from_secs(30)).build()?))
    }

    /// Bootstraps Arti with state under `dir` (created if missing).
    pub async fn bootstrap_tor(dir: &Path) -> Result<tor::Client> {
        std::fs::create_dir_all(dir)?;
        tor::Client::create(dir, |_| {}).await.map_err(|e| anyhow!("tor bootstrap: {e}"))
    }

    /// A transport on a fresh circuit that shares no stream with any other handle.
    pub fn isolated(base: &tor::Client) -> Self {
        Self::Tor(base.isolated_client())
    }

    pub async fn get(&self, url: &str) -> Result<(u16, String)> {
        match self {
            Self::Direct(c) => {
                let r = c.get(url).send().await?;
                Ok((r.status().as_u16(), r.text().await?))
            }
            Self::Tor(c) => {
                let uri: Uri = url.parse()?;
                let r = c
                    .http_get(uri, |b| b.header(header::ACCEPT, "application/json"), collect, RETRIES, retry)
                    .await
                    .map_err(|e| anyhow!("tor get: {e}"))?;
                Ok((r.status().as_u16(), String::from_utf8(r.into_body().to_vec())?))
            }
        }
    }

    pub async fn post_json(&self, url: &str, body: &serde_json::Value) -> Result<(u16, String)> {
        match self {
            Self::Direct(c) => {
                let r = c.post(url).json(body).send().await?;
                Ok((r.status().as_u16(), r.text().await?))
            }
            Self::Tor(c) => {
                let uri: Uri = url.parse()?;
                let r = c
                    .http_post(
                        uri,
                        |b| b.header(header::CONTENT_TYPE, "application/json").header(header::ACCEPT, "application/json"),
                        Full::new(Bytes::from(serde_json::to_vec(body)?)),
                        collect,
                        RETRIES,
                        retry,
                    )
                    .await
                    .map_err(|e| anyhow!("tor post: {e}"))?;
                Ok((r.status().as_u16(), String::from_utf8(r.into_body().to_vec())?))
            }
        }
    }
}

async fn collect(body: hyper::body::Incoming) -> Result<Bytes, tor::Error> {
    Ok(body
        .collect()
        .await
        .map_err(|e| tor::Error::Http(tor::http::HttpError::from(e)))?
        .to_bytes())
}
