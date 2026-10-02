//! HTTP service for the static web app. Reads of public data go direct; everything that
//! touches a swap runs on isolated Tor circuits. No secrets live here: refund addresses
//! come from the caller's own wallet.
use crate::{
    exposure::identifiers,
    oneclick::Client,
    splitter::{is_canonical, schedule, split},
    swap::{quote_inbound, quote_part},
    transport::Transport,
    wallet::{receivers, to_revision_1},
};
use anyhow::{anyhow, bail, Result};
use axum::{
    extract::{Path as UrlPath, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use rand::SeedableRng;
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::{Duration, Instant}};
use tokio::sync::{Mutex, Semaphore};
use zcash_client_backend::tor;

const MAX_PARTS: usize = 16;

pub struct App {
    tor: tor::Client,
    direct: Client,
    /// Bounds concurrent Tor work so one caller cannot saturate the circuit pool.
    tor_slots: Semaphore,
    cache: Mutex<HashMap<String, (Instant, Value)>>,
}

type S = State<Arc<App>>;
type Res = Result<Json<Value>, (StatusCode, Json<Value>)>;

fn bad(code: StatusCode, e: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    (code, Json(json!({ "error": e.to_string() })))
}

pub async fn serve(addr: &str, dir: PathBuf) -> Result<()> {
    let tor = Transport::bootstrap_tor(&dir.join("tor")).await?;
    let app = Arc::new(App { tor, direct: Client::new()?, tor_slots: Semaphore::new(4), cache: Mutex::new(HashMap::new()) });
    let router = Router::new()
        .route("/api/health", get(|| async { Json(json!({ "ok": true })) }))
        .route("/api/tokens", get(tokens))
        .route("/api/plan", post(plan))
        .route("/api/inspect/{addr}", get(inspect))
        .route("/api/quote", post(quote))
        .route("/api/quote-in", post(quote_in))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(app);
    let l = tokio::net::TcpListener::bind(addr).await?;
    println!("airlock service on {}", l.local_addr()?);
    axum::serve(l, router).await?;
    Ok(())
}

async fn cached<F, Fut>(app: &App, key: String, ttl: Duration, f: F) -> Result<Value>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Value>>,
{
    if let Some((t, v)) = app.cache.lock().await.get(&key) {
        if t.elapsed() < ttl {
            return Ok(v.clone());
        }
    }
    let v = f().await?;
    app.cache.lock().await.insert(key, (Instant::now(), v.clone()));
    Ok(v)
}

async fn tokens(State(app): S) -> Res {
    let v = cached(&app, "tokens".into(), Duration::from_secs(600), || async {
        let t = app.direct.tokens().await?;
        Ok(Value::Array(
            t.into_iter()
                .filter(|t| t.blockchain != "zec")
                .map(|t| json!({ "assetId": t.asset_id, "symbol": t.symbol, "blockchain": t.blockchain, "decimals": t.decimals, "price": t.price }))
                .collect(),
        ))
    })
    .await
    .map_err(|e| bad(StatusCode::BAD_GATEWAY, e))?;
    Ok(Json(v))
}

fn zat(zec: &str) -> Result<u64> {
    let (i, f) = zec.split_once('.').unwrap_or((zec, ""));
    let digits = |x: &str| x.chars().all(|c| c.is_ascii_digit());
    if f.len() > 8 || (i.is_empty() && f.is_empty()) || !digits(i) || !digits(f) {
        bail!("amount {zec:?} is not a ZEC decimal with at most 8 places");
    }
    let whole = if i.is_empty() { 0 } else { i.parse::<u64>()? };
    whole.checked_mul(100_000_000).ok_or_else(|| anyhow!("amount too large"))?.checked_add(format!("{f:0<8}").parse::<u64>()?).ok_or_else(|| anyhow!("amount too large"))
}

#[derive(Deserialize)]
struct PlanReq {
    #[serde(rename = "amountZec")]
    amount_zec: String,
    #[serde(rename = "capZec")]
    cap_zec: String,
}

fn plan_json(amount: &str, cap: &str) -> Result<Value> {
    let s = split(zat(amount)?, zat(cap)?, MAX_PARTS)?;
    let mut rng = rand_chacha::ChaCha20Rng::try_from_rng(&mut rand::rngs::SysRng).map_err(|e| anyhow!("os rng: {e:?}"))?;
    let at = schedule(s.parts.len(), &mut rng);
    Ok(json!({
        // sendAtBlocks: suggested offsets from now, blocks of about 75 s (25 s after NU7)
        "parts": s.parts.iter().zip(&at).map(|(&p, &b)| json!({ "zat": p, "canonical": is_canonical(p), "sendAtBlocks": b })).collect::<Vec<_>>(),
        "residualZat": s.residual,
    }))
}

async fn plan(Json(r): Json<PlanReq>) -> Res {
    plan_json(&r.amount_zec, &r.cap_zec).map(Json).map_err(|e| bad(StatusCode::BAD_REQUEST, e))
}

/// A deposit address on any 1Click origin chain: letters and digits, 20 to 100 characters.
fn valid_deposit(a: &str) -> bool {
    (20..=100).contains(&a.len()) && a.chars().all(|c| c.is_ascii_alphanumeric())
}

async fn inspect(State(app): S, UrlPath(addr): UrlPath<String>) -> Res {
    if !valid_deposit(&addr) {
        return Err(bad(StatusCode::BAD_REQUEST, "not a deposit address"));
    }
    // Over Tor, one circuit per call: the caller's lookup does not hand 1Click a client address.
    let _slot = app.tor_slots.acquire().await.map_err(|e| bad(StatusCode::SERVICE_UNAVAILABLE, e))?;
    let r = match Client::with_transport(Transport::isolated(&app.tor)).status(&addr).await {
        Ok(r) => r,
        Err(e) if e.to_string().contains("HTTP 404") => return Err(bad(StatusCode::NOT_FOUND, "no 1Click swap uses this address")),
        Err(e) => return Err(bad(StatusCode::BAD_GATEWAY, e)),
    };
    let q = &r.raw["quoteResponse"]["quoteRequest"];
    let amount = q["amount"].as_str().and_then(|a| a.parse::<u64>().ok());
    Ok(Json(json!({
        "depositAddress": addr,
        "status": r.raw["status"],
        "updatedAt": r.raw["updatedAt"],
        "publicOnStatus": identifiers(&r.raw).into_iter().map(|(f, v)| json!({"field": f, "value": v})).collect::<Vec<_>>(),
        "referral": q["referral"],
        "originAsset": q["originAsset"],
        "destinationAsset": q["destinationAsset"],
        "amountZat": amount,
        "amountCanonical": amount.filter(|_| q["originAsset"] == crate::oneclick::ZEC_ASSET).map(is_canonical),
    })))
}

#[derive(Deserialize)]
struct QuoteReq {
    #[serde(rename = "amountZat")]
    amount_zat: u64,
    #[serde(rename = "destinationAsset")]
    destination_asset: String,
    destination: String,
    /// Orchard-only unified address from the caller's own wallet.
    refund: String,
}

/// One part. Stateless: the service never sees two parts of a swap in one request and writes nothing.
async fn quote(State(app): S, Json(r): Json<QuoteReq>) -> Res {
    let go = async {
        if !is_canonical(r.amount_zat) {
            bail!("{} zat is not a ZIP 318 canonical denomination", r.amount_zat);
        }
        let u1 = to_revision_1(r.refund.trim()).map_err(|_| anyhow!("refund address is not a unified address"))?;
        if receivers(&u1)? != ["orchard"] {
            bail!("refund address must be an Orchard-only unified address");
        }
        let _slot = app.tor_slots.acquire().await?;
        let p = quote_part(&app.tor, r.amount_zat, &r.destination_asset, r.destination.trim(), &u1).await?;
        Ok::<_, anyhow::Error>(json!({
            "amountZat": p.amount_zat, "depositAddress": p.deposit_address, "deadline": p.deadline,
            "destination": p.destination, "refund": p.refund_ua,
        }))
    };
    go.await.map(Json).map_err(|e| bad(StatusCode::BAD_REQUEST, e))
}

#[derive(Deserialize)]
struct InboundReq {
    #[serde(rename = "amountZec")]
    amount_zec: String,
    #[serde(rename = "originAsset")]
    origin_asset: String,
    /// Refund address on the origin chain.
    #[serde(rename = "originRefund")]
    origin_refund: String,
    /// Orchard-only unified address from the caller's own wallet.
    recipient: String,
}

async fn quote_in(State(app): S, Json(r): Json<InboundReq>) -> Res {
    let go = async {
        let _slot = app.tor_slots.acquire().await?;
        let ib = quote_inbound(&app.tor, zat(&r.amount_zec)?, &r.origin_asset, &r.origin_refund, &r.recipient).await?;
        Ok::<_, anyhow::Error>(json!({
            "originAsset": ib.origin_asset, "depositAddress": ib.deposit_address, "recipient": ib.recipient,
            "amountOutZat": ib.amount_out_zat, "amountIn": ib.amount_in, "minAmountIn": ib.min_amount_in, "deadline": ib.deadline,
        }))
    };
    go.await.map(Json).map_err(|e| bad(StatusCode::BAD_REQUEST, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_matches_the_splitter() {
        let p = plan_json("1.3", "0.5").unwrap();
        let z: Vec<u64> = p["parts"].as_array().unwrap().iter().map(|x| x["zat"].as_u64().unwrap()).collect();
        assert_eq!(z, vec![50_000_000, 50_000_000, 20_000_000, 10_000_000]);
        assert!(p["parts"].as_array().unwrap().iter().all(|x| x["canonical"] == true));
    }

    #[test]
    fn rejects_bad_amounts_and_addresses() {
        assert!(plan_json("abc", "0.5").is_err());
        assert!(plan_json("1.123456789", "0.5").is_err());
        assert!(!valid_deposit("t1../x"));
        assert!(!valid_deposit("short"));
        assert!(valid_deposit("t1UH6mFsvCCzTYvKgTgNCbNBW9t8Vd1NCoq"));
        assert!(valid_deposit("0x2527D02599Ba641c19FEa793cD0F167589a0f10D"));
    }
}
