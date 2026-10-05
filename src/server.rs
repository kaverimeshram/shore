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
/// Launch caps, enforced before any quote is requested. One inbound swap and one outbound swap each
/// move at most 0.02 ZEC, and no outbound part exceeds 0.01 ZEC (the ZIP 318 floor), so the value on any one deposit
/// address is bounded. The service holds no funds and cannot raise these at runtime.
pub const MAX_INBOUND_ZAT: u64 = 2_000_000;
pub const MAX_PART_ZAT: u64 = 1_000_000;
pub const MAX_OUTBOUND_ZAT: u64 = 2_000_000;

/// Quotes stop at once when AIRLOCK_PAUSED is set on the machine. Health, tokens, plan and inspect keep working.
fn paused() -> Result<()> {
    if std::env::var_os("AIRLOCK_PAUSED").is_some() {
        bail!("quotes are paused");
    }
    Ok(())
}

pub struct App {
    tor: tor::Client,
    direct: Client,
    /// Bound concurrent Tor work, one pool per kind so status lookups cannot starve quotes.
    quote_slots: Semaphore,
    inspect_slots: Semaphore,
    cache: Mutex<HashMap<String, (Instant, Value)>>,
}

type S = State<Arc<App>>;
type Res = Result<Json<Value>, (StatusCode, Json<Value>)>;

fn bad(code: StatusCode, e: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    (code, Json(json!({ "error": e.to_string() })))
}

pub async fn serve(addr: &str, dir: PathBuf) -> Result<()> {
    let tor = Transport::bootstrap_tor(&dir.join("tor")).await?;
    let app = Arc::new(App { tor, direct: Client::new()?, quote_slots: Semaphore::new(4), inspect_slots: Semaphore::new(4), cache: Mutex::new(HashMap::new()) });
    let router = Router::new()
        .route("/api/health", get(|| async { Json(json!({ "ok": true })) }))
        .route("/api/tokens", get(tokens))
        .route("/api/plan", post(plan))
        .route("/api/inspect/{addr}", get(inspect))
        .route("/api/quote", post(quote))
        .route("/api/quote-in", post(quote_in))
        .layer(cors())
        .with_state(app);
    let l = tokio::net::TcpListener::bind(addr).await?;
    println!("airlock service on {}", l.local_addr()?);
    axum::serve(l, router).await?;
    Ok(())
}

/// Browsers on the static site (and local development) may call the service; other origins get no CORS header.
fn cors() -> tower_http::cors::CorsLayer {
    use axum::http::{header::CONTENT_TYPE, HeaderValue, Method};
    let allowed = |o: &HeaderValue, _: &axum::http::request::Parts| {
        let o = o.to_str().unwrap_or("");
        o == "https://airlock-zec.vercel.app" || o.starts_with("http://127.0.0.1:") || o.starts_with("http://localhost:")
    };
    tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::predicate(allowed))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE])
}

/// A permit, or an error after 20 s: a flood of one kind cannot queue the others without bound.
async fn slot(pool: &Semaphore) -> Result<tokio::sync::SemaphorePermit<'_>> {
    tokio::time::timeout(Duration::from_secs(20), pool.acquire()).await.map_err(|_| anyhow!("busy, try again"))?.map_err(|e| anyhow!("{e}"))
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

/// Base units to a decimal string: ("12971142", 6) is "12.971142".
fn units(raw: &str, decimals: u32) -> Option<String> {
    if raw.is_empty() || !raw.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let padded = format!("{raw:0>width$}", width = decimals as usize + 1);
    let (i, f) = padded.split_at(padded.len() - decimals as usize);
    let i = i.trim_start_matches('0');
    let i = if i.is_empty() { "0" } else { i };
    let f = f.trim_end_matches('0');
    Some(if f.is_empty() { i.to_string() } else { format!("{i}.{f}") })
}

async fn token_list(app: &App) -> Result<Value> {
    cached(app, "tokens".into(), Duration::from_secs(600), || async {
        let t = app.direct.tokens().await?;
        Ok(Value::Array(
            t.into_iter()
                .filter(|t| t.blockchain != "zec")
                .map(|t| json!({ "assetId": t.asset_id, "symbol": t.symbol, "blockchain": t.blockchain, "decimals": t.decimals, "price": t.price }))
                .collect(),
        ))
    })
    .await
}

async fn tokens(State(app): S) -> Res {
    token_list(&app).await.map(Json).map_err(|e| bad(StatusCode::BAD_GATEWAY, e))
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
    let (total, cap) = (zat(amount)?, zat(cap)?);
    if total > MAX_OUTBOUND_ZAT || cap > MAX_PART_ZAT {
        bail!("launch caps: at most {} ZEC per swap and {} ZEC per part", MAX_OUTBOUND_ZAT as f64 / 1e8, MAX_PART_ZAT as f64 / 1e8);
    }
    let s = split(total, cap, MAX_PARTS)?;
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
    let _slot = slot(&app.inspect_slots).await.map_err(|e| bad(StatusCode::SERVICE_UNAVAILABLE, e))?;
    let r = match Client::with_transport(Transport::isolated(&app.tor)).status(&addr).await {
        Ok(r) => r,
        Err(e) if e.to_string().contains("HTTP 404") => return Err(bad(StatusCode::NOT_FOUND, "no 1Click swap uses this address")),
        Err(e) => return Err(bad(StatusCode::BAD_GATEWAY, e)),
    };
    let q = &r.raw["quoteResponse"]["quoteRequest"];
    let amount = q["amount"].as_str().and_then(|a| a.parse::<u64>().ok());
    let profile = crate::profile::Profile::from_status(&r.raw);
    let crowd = profile.as_ref().and_then(|p| {
        let r = crate::profile::Reference::load(std::path::Path::new("data/quote-profiles.json")).ok()?;
        Some(json!({ "matches": r.matches(p), "sample": r.sample_size() }))
    });
    Ok(Json(json!({
        "quoteProfile": profile.map(|p| json!({
            "referral": p.referral, "fees": p.fees, "slippageBps": p.slippage_bps, "deadlineMin": p.deadline_min, "refund": p.refund,
        })),
        "profileCrowd": crowd,
        "depositAddress": addr,
        "status": r.raw["status"],
        "updatedAt": r.raw["updatedAt"],
        "publicOnStatus": identifiers(&r.raw).into_iter().map(|(f, v)| json!({"field": f, "value": v})).collect::<Vec<_>>(),
        "referral": q["referral"],
        "originAsset": q["originAsset"],
        "destinationAsset": q["destinationAsset"],
        "amountZat": amount.filter(|_| q["originAsset"] == crate::oneclick::ZEC_ASSET),
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
        paused()?;
        if !is_canonical(r.amount_zat) {
            bail!("{} zat is not a ZIP 318 canonical denomination", r.amount_zat);
        }
        if r.amount_zat > MAX_PART_ZAT {
            bail!("launch cap: a part is at most {} zat", MAX_PART_ZAT);
        }
        let u1 = to_revision_1(r.refund.trim()).map_err(|_| anyhow!("refund address is not a unified address"))?;
        if receivers(&u1)? != ["orchard"] {
            bail!("refund address must be an Orchard-only unified address");
        }
        let _slot = slot(&app.quote_slots).await?;
        let p = quote_part(&app.tor, r.amount_zat, &r.destination_asset, r.destination.trim(), &u1).await?;
        let decimals = token_list(&app).await.ok().and_then(|t| t.as_array()?.iter().find(|x| x["assetId"] == r.destination_asset.as_str())?["decimals"].as_u64());
        let min_out = decimals.and_then(|d| units(&p.min_amount_out, d as u32));
        Ok::<_, anyhow::Error>(json!({
            "minAmountOutFormatted": min_out,
            "amountZat": p.amount_zat, "depositAddress": p.deposit_address, "deadline": p.deadline,
            "destination": p.destination, "refund": p.refund_ua, "amountOutFormatted": p.amount_out_formatted,
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
        paused()?;
        let amount = zat(&r.amount_zec)?;
        if amount > MAX_INBOUND_ZAT {
            bail!("launch cap: an inbound swap is at most {} zat", MAX_INBOUND_ZAT);
        }
        let _slot = slot(&app.quote_slots).await?;
        let ib = quote_inbound(&app.tor, amount, &r.origin_asset, &r.origin_refund, &r.recipient).await?;
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
        let p = plan_json("0.02", "0.01").unwrap();
        let z: Vec<u64> = p["parts"].as_array().unwrap().iter().map(|x| x["zat"].as_u64().unwrap()).collect();
        assert_eq!(z, vec![1_000_000, 1_000_000]);
        assert!(p["parts"].as_array().unwrap().iter().all(|x| x["canonical"] == true));
    }

    #[test]
    fn units_format_base_units() {
        assert_eq!(units("12971142", 6).unwrap(), "12.971142");
        assert_eq!(units("12000000", 6).unwrap(), "12");
        assert_eq!(units("500", 6).unwrap(), "0.0005");
        assert_eq!(units("0", 6).unwrap(), "0");
        assert!(units("1e5", 6).is_none());
    }

    #[test]
    fn pause_switch_stops_quotes() {
        std::env::remove_var("AIRLOCK_PAUSED");
        assert!(paused().is_ok());
        std::env::set_var("AIRLOCK_PAUSED", "1");
        assert!(paused().is_err());
        std::env::remove_var("AIRLOCK_PAUSED");
    }

    #[test]
    fn plan_refuses_over_the_launch_caps() {
        assert!(plan_json("0.02", "0.01").is_ok());
        assert!(plan_json("0.0200001", "0.01").is_err());
        assert!(plan_json("0.02", "0.0100001").is_err());
        assert!(plan_json("1.3", "0.5").is_err());
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
