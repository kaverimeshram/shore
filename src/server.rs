//! HTTP service for the static web app. Reads of public data go direct; everything that
//! touches a swap runs on isolated Tor circuits. No secrets live here: refund addresses
//! come from the caller's own wallet.
use crate::{
    exposure::identifiers,
    oneclick::Client,
    splitter::{is_canonical, split},
    swap::{quote_inbound, quote_outbound, read_parts, receipt_json, Inbound, Swap},
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
use std::{collections::HashMap, path::PathBuf, sync::Arc, time::{Duration, Instant}};
use tokio::sync::{Mutex, Semaphore};
use zcash_client_backend::tor;

const MAX_PARTS: usize = 16;

pub struct App {
    tor: tor::Client,
    dir: PathBuf,
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
    let app = Arc::new(App { tor, dir, direct: Client::new()?, tor_slots: Semaphore::new(4), cache: Mutex::new(HashMap::new()) });
    let router = Router::new()
        .route("/api/health", get(|| async { Json(json!({ "ok": true })) }))
        .route("/api/tokens", get(tokens))
        .route("/api/plan", post(plan))
        .route("/api/inspect/{addr}", get(inspect))
        .route("/api/inbound", post(create_inbound))
        .route("/api/inbound/{id}", get(get_inbound))
        .route("/api/swaps", post(create_swap))
        .route("/api/swaps/{id}", get(get_swap))
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
    Ok(json!({
        "parts": s.parts.iter().map(|&p| json!({ "zat": p, "canonical": is_canonical(p) })).collect::<Vec<_>>(),
        "residualZat": s.residual,
    }))
}

async fn plan(Json(r): Json<PlanReq>) -> Res {
    plan_json(&r.amount_zec, &r.cap_zec).map(Json).map_err(|e| bad(StatusCode::BAD_REQUEST, e))
}

fn valid_taddr(a: &str) -> bool {
    (a.starts_with("t1") || a.starts_with("t3")) && (26..=36).contains(&a.len()) && a.chars().all(|c| c.is_ascii_alphanumeric())
}

async fn inspect(State(app): S, UrlPath(addr): UrlPath<String>) -> Res {
    if !valid_taddr(&addr) {
        return Err(bad(StatusCode::BAD_REQUEST, "not a Zcash transparent address"));
    }
    let r = match app.direct.status(&addr).await {
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
        "amountCanonical": amount.map(is_canonical),
    })))
}

#[derive(Deserialize)]
struct SwapReq {
    #[serde(rename = "amountZec")]
    amount_zec: String,
    #[serde(rename = "capZec")]
    cap_zec: String,
    #[serde(rename = "destinationAsset")]
    destination_asset: String,
    destinations: Vec<String>,
    /// One Orchard-only unified address per part from the caller's own wallet.
    refunds: Vec<String>,
}

async fn create_swap(State(app): S, Json(r): Json<SwapReq>) -> Res {
    create_swap_inner(&app, r).await.map(Json).map_err(|e| bad(StatusCode::BAD_REQUEST, e))
}

async fn create_swap_inner(app: &App, r: SwapReq) -> Result<Value> {
    let plan = split(zat(&r.amount_zec)?, zat(&r.cap_zec)?, MAX_PARTS)?;
    if plan.parts.is_empty() {
        bail!("amount is below the smallest canonical denomination (0.01 ZEC)");
    }
    if r.refunds.len() != plan.parts.len() {
        bail!("the plan has {} parts: supply {} refund addresses, got {}", plan.parts.len(), plan.parts.len(), r.refunds.len());
    }
    let mut refunds = Vec::new();
    for u in &r.refunds {
        let u1 = to_revision_1(u.trim()).map_err(|_| anyhow!("refund address is not a unified address: {u}"))?;
        if receivers(&u1)? != ["orchard"] {
            bail!("refund address must be an Orchard-only unified address (1Click rejects Sapling-only and the point is a shielded refund): {u}");
        }
        refunds.push(u1);
    }
    if refunds.iter().collect::<std::collections::HashSet<_>>().len() != refunds.len() {
        bail!("refund addresses repeat: a shared refund address links the parts");
    }
    let _slot = app.tor_slots.acquire().await?;
    let mut it = refunds.into_iter();
    let swap = quote_outbound(&app.tor, || it.next().ok_or_else(|| anyhow!("refund list exhausted")), &app.dir, zat(&r.amount_zec)?, zat(&r.cap_zec)?, &r.destination_asset, &r.destinations).await?;
    Ok(json!({
        "swapId": swap.id,
        "residualZat": swap.residual_zat,
        "parts": swap.parts.iter().map(|p| json!({
            "amountZat": p.amount_zat, "depositAddress": p.deposit_address, "deadline": p.deadline,
            "destination": p.destination, "refund": p.refund_ua,
        })).collect::<Vec<_>>(),
    }))
}

async fn get_swap(State(app): S, UrlPath(id): UrlPath<String>) -> Res {
    if id.len() != 12 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(bad(StatusCode::BAD_REQUEST, "swap ids are 12 hex characters"));
    }
    let swap = Swap::load(&app.dir, &id).map_err(|_| bad(StatusCode::NOT_FOUND, "unknown swap"))?;
    let v = cached(&app, format!("swap:{id}"), Duration::from_secs(20), || async {
        let _slot = app.tor_slots.acquire().await?;
        let reads = read_parts(&app.tor, &swap).await?;
        Ok(receipt_json(&swap, &reads))
    })
    .await
    .map_err(|e| bad(StatusCode::BAD_GATEWAY, e))?;
    Ok(Json(v))
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

async fn create_inbound(State(app): S, Json(r): Json<InboundReq>) -> Res {
    let go = async {
        let _slot = app.tor_slots.acquire().await?;
        let ib = quote_inbound(&app.tor, &app.dir, zat(&r.amount_zec)?, &r.origin_asset, &r.origin_refund, &r.recipient).await?;
        Ok::<_, anyhow::Error>(inbound_json(&ib))
    };
    go.await.map(Json).map_err(|e| bad(StatusCode::BAD_REQUEST, e))
}

fn inbound_json(ib: &Inbound) -> Value {
    json!({
        "inboundId": ib.id, "originAsset": ib.origin_asset, "depositAddress": ib.deposit_address,
        "recipient": ib.recipient, "amountOutZat": ib.amount_out_zat, "amountIn": ib.amount_in,
        "minAmountIn": ib.min_amount_in, "deadline": ib.deadline,
    })
}

async fn get_inbound(State(app): S, UrlPath(id): UrlPath<String>) -> Res {
    if id.len() != 12 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(bad(StatusCode::BAD_REQUEST, "inbound ids are 12 hex characters"));
    }
    let raw = std::fs::read(app.dir.join("inbound").join(format!("{id}.json"))).map_err(|_| bad(StatusCode::NOT_FOUND, "unknown inbound swap"))?;
    let ib: Inbound = serde_json::from_slice(&raw).map_err(|e| bad(StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let v = cached(&app, format!("inbound:{id}"), Duration::from_secs(20), || async {
        let _slot = app.tor_slots.acquire().await?;
        let s = Client::with_transport(Transport::isolated(&app.tor)).status(&ib.deposit_address).await?;
        let mut v = inbound_json(&ib);
        v["status"] = s.raw["status"].clone();
        v["updatedAt"] = s.raw["updatedAt"].clone();
        v["publicOnStatus"] = json!(identifiers(&s.raw).into_iter().map(|(f, x)| json!({"field": f, "value": x})).collect::<Vec<_>>());
        Ok(v)
    })
    .await
    .map_err(|e| bad(StatusCode::BAD_GATEWAY, e))?;
    Ok(Json(v))
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
        assert!(!valid_taddr("t1../x"));
        assert!(valid_taddr("t1UH6mFsvCCzTYvKgTgNCbNBW9t8Vd1NCoq"));
        assert!(!valid_taddr("0x2527D02599Ba641c19FEa793cD0F167589a0f10D"));
    }
}
