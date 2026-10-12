//! SHORE: ZEC held on Solana becomes shielded Zcash with one Phantom approval.
//! HTTP API for the one-page flow (contract in docs/API.md). Every outside service sits behind a
//! trait so the handlers run against mocks in tests and against 1Click, lightwalletd and a
//! public block explorer in production. The server holds a UFVK at most, never a spending key.
use crate::{
    exposure::identifiers,
    oneclick::{deadline_in, QuoteRequest, QuoteResponse, StatusResponse, SwapType, ZEC_ASSET},
    wallet::{receivers, to_revision_1},
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use axum::{
    extract::{Path as UrlPath, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

/// ZEC on Solana, the SPL mint 1Click lists as `1cs_v1:sol:spl:<mint>`.
pub const SOL_ZEC_MINT: &str = "A7bdiYdS5GjqGFtxf17ppRHtDKPkkRqbKtR27dxvQXaS";
pub const SOL_ZEC_ASSET: &str = "1cs_v1:sol:spl:A7bdiYdS5GjqGFtxf17ppRHtDKPkkRqbKtR27dxvQXaS";
pub const DECIMALS: u32 = 8;
/// Smallest input 1Click accepted on this route (sol-zec-quote-too-small.json, 2026-10-10).
pub const MIN_AMOUNT: u64 = 133_334;
/// Launch cap, same as AIRLOCK inbound: 0.02 ZEC.
pub const MAX_AMOUNT: u64 = crate::server::MAX_INBOUND_ZAT;

// ---------- outside services ----------

#[async_trait]
pub trait OneClickApi: Send + Sync {
    async fn quote(&self, q: &QuoteRequest) -> Result<QuoteResponse>;
    /// `Ok(None)` when 1Click knows no swap for this deposit address (HTTP 404).
    async fn status(&self, deposit: &str) -> Result<Option<StatusResponse>>;
    async fn submit(&self, tx_hash: &str, deposit: &str) -> Result<Value>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub pool: String,
    pub value_zat: u64,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxShape {
    pub transparent_inputs: u32,
    /// Value entering the shielded pools, readable by anyone when the inputs are transparent.
    pub shielded_value_in: i64,
}

#[async_trait]
pub trait ZcashApi: Send + Sync {
    /// Mined height, `None` while in the mempool or unknown.
    async fn mined_height(&self, txid: &str) -> Result<Option<u64>>;
    /// View-key scan; the note our wallet received in `txid`, if any, and the scanned height.
    async fn readback(&self, txid: &str) -> Result<(Option<Note>, u32)>;
    async fn tx_shape(&self, txid: &str) -> Result<Option<TxShape>>;
    /// Read-back with a viewing key the user supplies for this request only: scans from `from_height` into a
    /// throwaway db that is deleted afterwards. The key can view, never spend; it is not stored or logged.
    async fn readback_with(&self, _ufvk: &str, _from_height: u32, _txid: &str) -> Result<(Option<Note>, u32)> {
        anyhow::bail!("user viewing-key read-back is not available on this server")
    }
}

/// The demo wallet's addresses, from its UFVK.
pub trait AddressSource: Send + Sync {
    fn at(&self, index: u32) -> Result<String>;
    fn owns(&self, u1: &str) -> bool;
}

// ---------- errors ----------

#[derive(Debug)]
pub struct ApiErr(pub StatusCode, pub &'static str, pub String);

impl IntoResponse for ApiErr {
    fn into_response(self) -> Response {
        (self.0, Json(json!({ "error": self.2, "code": self.1 }))).into_response()
    }
}

fn bad(code: &'static str, m: impl std::fmt::Display) -> ApiErr {
    ApiErr(StatusCode::BAD_REQUEST, code, m.to_string())
}
fn upstream(e: anyhow::Error) -> ApiErr {
    ApiErr(StatusCode::BAD_GATEWAY, "UPSTREAM", e.to_string())
}
fn internal(e: anyhow::Error) -> ApiErr {
    ApiErr(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL", e.to_string())
}

// ---------- sessions ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub recipient: String,
    pub source: String,
    pub ua_index: Option<u32>,
    pub ours: bool,
    pub created_at: u64,
    #[serde(default)]
    pub deposit_address: Option<String>,
    #[serde(default)]
    pub sender: Option<String>,
    #[serde(default)]
    pub amount_in: Option<String>,
    #[serde(default)]
    pub quoted_out: Option<String>,
    #[serde(default)]
    pub deadline: Option<String>,
    #[serde(default)]
    pub solana_tx: Option<String>,
}

pub struct Shore {
    /// The gitignored work directory (the file store and the read-back db live here).
    pub dir: PathBuf,
    pub oneclick: Arc<dyn OneClickApi>,
    pub zcash: Arc<dyn ZcashApi>,
    pub addrs: Arc<dyn AddressSource>,
    pub store: Arc<dyn Store>,
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Where shields, the address index and the used-address set persist. Both operations that guard
/// privacy are atomic in every implementation: `next_index` never returns a value twice and `claim`
/// succeeds for an address at most once, even with several API machines.
#[async_trait]
pub trait Store: Send + Sync {
    async fn next_index(&self, start: u32) -> Result<u32>;
    /// True when `u1` was not claimed before (and is claimed now).
    async fn claim(&self, u1: &str) -> Result<bool>;
    async fn put(&self, s: &Session) -> Result<()>;
    async fn get(&self, id: &str) -> Result<Option<Session>>;
    async fn by_deposit(&self, deposit: &str) -> Result<Option<Session>>;
    async fn recipient_count(&self, u1: &str) -> Result<usize>;
}

/// Files under the work directory, one process. `ua-index` is the file the CLI also advances.
pub struct FileStore {
    dir: PathBuf,
    lock: Mutex<()>,
}

impl FileStore {
    pub fn new(dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(dir.join("shore/sessions"))?;
        Ok(Self { dir, lock: Mutex::new(()) })
    }
    fn sessions(&self) -> Vec<Session> {
        std::fs::read_dir(self.dir.join("shore/sessions")).into_iter().flatten().flatten().filter_map(|e| serde_json::from_slice(&std::fs::read(e.path()).ok()?).ok()).collect()
    }
}

#[async_trait]
impl Store for FileStore {
    async fn next_index(&self, start: u32) -> Result<u32> {
        let _g = self.lock.lock().await;
        let p = self.dir.join("ua-index");
        let i: u32 = match std::fs::read_to_string(&p) {
            Ok(s) => s.trim().parse().map_err(|_| anyhow!("ua-index is not a number"))?,
            Err(_) => start,
        };
        // advanced before the address is used: a crash can skip an index, never repeat one
        std::fs::write(&p, (i + 1).to_string())?;
        Ok(i)
    }
    async fn claim(&self, u1: &str) -> Result<bool> {
        use std::io::Write;
        let _g = self.lock.lock().await;
        let p = self.dir.join("shore/used-addresses.txt");
        if std::fs::read_to_string(&p).unwrap_or_default().lines().any(|l| l == u1) {
            return Ok(false);
        }
        writeln!(std::fs::OpenOptions::new().create(true).append(true).open(p)?, "{u1}")?;
        Ok(true)
    }
    async fn put(&self, s: &Session) -> Result<()> {
        std::fs::write(self.dir.join("shore/sessions").join(format!("{}.json", s.id)), serde_json::to_vec_pretty(s)?)?;
        Ok(())
    }
    async fn get(&self, id: &str) -> Result<Option<Session>> {
        match std::fs::read(self.dir.join("shore/sessions").join(format!("{id}.json"))) {
            Ok(b) => Ok(Some(serde_json::from_slice(&b)?)),
            Err(_) => Ok(None),
        }
    }
    async fn by_deposit(&self, d: &str) -> Result<Option<Session>> {
        Ok(self.sessions().into_iter().find(|s| s.deposit_address.as_deref() == Some(d)))
    }
    async fn recipient_count(&self, u1: &str) -> Result<usize> {
        Ok(self.sessions().iter().filter(|s| s.recipient == u1).count())
    }
}

/// A Cloudflare Worker over D1 (deploy/shore-store): atomic `UPDATE .. RETURNING` for the index and a
/// primary key on the used-address table. Free tier; the bearer token is a Fly secret.
pub struct HttpStore {
    base: String,
    token: String,
    http: reqwest::Client,
}

impl HttpStore {
    pub fn new(base: String, token: String) -> Self {
        Self { base: base.trim_end_matches('/').into(), token, http: reqwest::Client::new() }
    }
    async fn call(&self, method: reqwest::Method, path: &str, body: Option<Value>) -> Result<Value> {
        let mut r = self.http.request(method, format!("{}{path}", self.base)).bearer_auth(&self.token).timeout(std::time::Duration::from_secs(10));
        if let Some(b) = body {
            r = r.json(&b);
        }
        let r = r.send().await.map_err(|e| anyhow!("store: {}", e.without_url()))?;
        let code = r.status();
        let v: Value = r.json().await.unwrap_or(Value::Null);
        if !code.is_success() {
            anyhow::bail!("store HTTP {code}: {v}");
        }
        Ok(v)
    }
}

#[async_trait]
impl Store for HttpStore {
    async fn next_index(&self, start: u32) -> Result<u32> {
        let v = self.call(reqwest::Method::POST, "/index", Some(json!({ "start": start }))).await?;
        v["index"].as_u64().and_then(|i| u32::try_from(i).ok()).ok_or_else(|| anyhow!("store returned no index"))
    }
    async fn claim(&self, u1: &str) -> Result<bool> {
        Ok(self.call(reqwest::Method::POST, "/claim", Some(json!({ "u1": u1 }))).await?["claimed"] == true)
    }
    async fn put(&self, s: &Session) -> Result<()> {
        self.call(reqwest::Method::PUT, &format!("/session/{}", s.id), Some(serde_json::to_value(s)?)).await.map(|_| ())
    }
    async fn get(&self, id: &str) -> Result<Option<Session>> {
        let v = self.call(reqwest::Method::GET, &format!("/session/{id}"), None).await?;
        Ok(if v["session"].is_null() { None } else { Some(serde_json::from_value(v["session"].clone())?) })
    }
    async fn by_deposit(&self, d: &str) -> Result<Option<Session>> {
        let v = self.call(reqwest::Method::GET, &format!("/deposit/{d}"), None).await?;
        Ok(if v["session"].is_null() { None } else { Some(serde_json::from_value(v["session"].clone())?) })
    }
    async fn recipient_count(&self, u1: &str) -> Result<usize> {
        Ok(self.call(reqwest::Method::GET, &format!("/recipient/{u1}"), None).await?["count"].as_u64().unwrap_or(0) as usize)
    }
}

const REUSED: &str = "this address already received a shield; reuse links the two deposits on the public 1Click status page. Use a fresh address";

impl Shore {
    pub fn new(dir: PathBuf, oneclick: Arc<dyn OneClickApi>, zcash: Arc<dyn ZcashApi>, addrs: Arc<dyn AddressSource>) -> Result<Self> {
        let store = Arc::new(FileStore::new(dir.clone())?);
        Ok(Self { dir, oneclick, zcash, addrs, store })
    }

    pub fn with_store(mut self, store: Arc<dyn Store>) -> Self {
        self.store = store;
        self
    }

    pub async fn save(&self, s: &Session) -> Result<()> {
        self.store.put(s).await
    }

    pub async fn load(&self, id: &str) -> Result<Session, ApiErr> {
        if id.len() != 12 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(bad("BAD_ID", "not a shield id"));
        }
        self.store.get(id).await.map_err(upstream)?.ok_or_else(|| ApiErr(StatusCode::NOT_FOUND, "NOT_FOUND", "no such shield".into()))
    }

    pub async fn by_deposit(&self, deposit: &str) -> Option<Session> {
        self.store.by_deposit(deposit).await.ok().flatten()
    }

    async fn fresh(&self) -> Result<Session> {
        // A new store (a deploy) starts in its own index range so it never repeats a local address.
        let start = std::env::var("SHORE_INDEX_START").ok().and_then(|x| x.parse().ok()).unwrap_or(0);
        let i = self.store.next_index(start).await?;
        let u1 = self.addrs.at(i)?;
        if !self.store.claim(&u1).await? {
            anyhow::bail!("index {i} produced an address already used");
        }
        let s = Session { id: hex::encode(crate::wallet::random_bytes::<6>()?), recipient: u1, source: "fresh".into(), ua_index: Some(i), ours: true, created_at: now(), deposit_address: None, sender: None, amount_in: None, quoted_out: None, deadline: None, solana_tx: None };
        self.save(&s).await?;
        Ok(s)
    }

    async fn own(&self, addr: &str) -> Result<Session, ApiErr> {
        let (u1, _) = check_own_address(addr)?;
        if !self.store.claim(&u1).await.map_err(upstream)? {
            return Err(ApiErr(StatusCode::CONFLICT, "ADDRESS_REUSED", REUSED.into()));
        }
        let ours = self.addrs.owns(&u1);
        let s = Session { id: hex::encode(crate::wallet::random_bytes::<6>().map_err(internal)?), recipient: u1, source: "own".into(), ua_index: None, ours, created_at: now(), deposit_address: None, sender: None, amount_in: None, quoted_out: None, deadline: None, solana_tx: None };
        self.save(&s).await.map_err(internal)?;
        Ok(s)
    }
}

/// A user-supplied recipient: mainnet unified address with a shielded receiver, re-encoded as `u1`.
pub fn check_own_address(a: &str) -> Result<(String, Vec<&'static str>), ApiErr> {
    let a = a.trim();
    if a.starts_with("t1") || a.starts_with("t3") || a.starts_with("tex1") || a.starts_with("tm") {
        return Err(bad("TRANSPARENT_REFUSED", "transparent addresses are public; SHORE only delivers to a shielded unified address"));
    }
    if a.starts_with("zs1") || a.starts_with("zc") {
        return Err(bad("NOT_UNIFIED", "give a unified address (u1...), not a Sapling-only address"));
    }
    if !(a.starts_with("u1") || a.starts_with("zu1")) {
        return Err(bad("NOT_UNIFIED", "not a mainnet unified address"));
    }
    let u1 = to_revision_1(a).map_err(|_| bad("NOT_UNIFIED", "address does not decode as a unified address"))?;
    if !u1.starts_with("u1") {
        return Err(bad("NOT_UNIFIED", "not a mainnet unified address"));
    }
    let r = receivers(&u1).map_err(|_| bad("NOT_UNIFIED", "address does not decode"))?;
    if !r.iter().any(|k| *k == "orchard" || *k == "sapling") {
        return Err(bad("NO_SHIELDED_RECEIVER", "this unified address has no shielded receiver"));
    }
    Ok((u1, r))
}

fn valid_solana(a: &str) -> bool {
    (32..=44).contains(&a.len()) && a.chars().all(|c| c.is_ascii_alphanumeric() && !"0OIl".contains(c))
}

fn valid_deposit(a: &str) -> bool {
    (20..=100).contains(&a.len()) && a.chars().all(|c| c.is_ascii_alphanumeric())
}

fn formatted(raw: &str) -> String {
    let padded = format!("{raw:0>9}");
    let (i, f) = padded.split_at(padded.len() - 8);
    let i = i.trim_start_matches('0');
    let f = f.trim_end_matches('0');
    match (i.is_empty(), f.is_empty()) {
        (true, true) => "0".into(),
        (_, true) => i.into(),
        (true, _) => format!("0.{f}"),
        _ => format!("{i}.{f}"),
    }
}

// ---------- handlers ----------

type St = State<Arc<Shore>>;
type R = Result<Json<Value>, ApiErr>;

/// Routes SHORE owns. `POST /api/quote` is shared with AIRLOCK's outbound quote; the server sends
/// bodies carrying a shield `id` to [`quote`] (see `server::quote_any`).
pub fn router(s: Arc<Shore>) -> Router {
    Router::new()
        .route("/api/shield/address", post(address))
        .route("/api/shield/{id}/tx", post(submit_tx))
        .route("/api/status/{deposit}", get(status))
        .route("/api/readback/{id}", get(readback).post(readback_key))
        .route("/api/exposure/{id}", get(exposure))
        .route("/api/rpc", post(rpc))
        .with_state(s)
}

/// SHORE alone, `/api/quote` included (tests and a SHORE-only deploy).
pub fn standalone_router(s: Arc<Shore>) -> Router {
    router(s.clone()).merge(Router::new().route("/api/quote", post(quote)).with_state(s))
}

#[derive(Deserialize)]
struct AddrReq {
    mode: String,
    #[serde(default)]
    address: Option<String>,
}

async fn address(State(s): St, Json(r): Json<AddrReq>) -> R {
    let sess = match r.mode.as_str() {
        "fresh" => s.fresh().await.map_err(internal)?,
        "own" => s.own(r.address.as_deref().ok_or_else(|| bad("MISSING_ADDRESS", "mode own needs address"))?).await?,
        _ => return Err(bad("BAD_MODE", "mode is fresh or own")),
    };
    let rec = receivers(&sess.recipient).map_err(internal)?;
    let mut v = json!({ "id": sess.id, "recipient": sess.recipient, "receivers": rec, "source": sess.source, "readbackAvailable": sess.ours });
    if let Some(i) = sess.ua_index {
        v["uaIndex"] = i.into();
    }
    Ok(Json(v))
}

#[derive(Deserialize)]
pub struct QuoteReq {
    id: String,
    amount: String,
    sender: String,
    #[serde(default)]
    dry: bool,
}

pub fn quote_request(recipient: &str, sender: &str, amount: u64, dry: bool) -> QuoteRequest {
    QuoteRequest {
        dry,
        deposit_mode: "SIMPLE",
        swap_type: SwapType::ExactInput,
        slippage_tolerance: crate::oneclick::SLIPPAGE_BPS,
        origin_asset: SOL_ZEC_ASSET.into(),
        deposit_type: "ORIGIN_CHAIN",
        destination_asset: ZEC_ASSET.into(),
        amount: amount.to_string(),
        refund_to: sender.into(),
        refund_type: "ORIGIN_CHAIN",
        recipient: recipient.into(),
        recipient_type: "DESTINATION_CHAIN",
        deadline: deadline_in(crate::oneclick::REQUEST_DEADLINE_SECS),
        app_fees: app_fee().into_iter().collect(),
    }
}

/// The SHORE shield fee, if configured: SHORE_APP_FEE_BPS (1..=100) paid to the NEAR account SHORE_APP_FEE_RECIPIENT.
/// Unset (the default today) means no SHORE fee; the quote then carries only 1Click's own fees.
pub fn app_fee() -> Option<crate::oneclick::AppFee> {
    let bps: u32 = std::env::var("SHORE_APP_FEE_BPS").ok()?.parse().ok().filter(|b| (1..=100).contains(b))?;
    let recipient = std::env::var("SHORE_APP_FEE_RECIPIENT").ok().filter(|r| !r.is_empty() && r.len() <= 64)?;
    Some(crate::oneclick::AppFee { recipient, fee: bps })
}

/// Validates the amount: base units, at least [`MIN_AMOUNT`], at most [`MAX_AMOUNT`].
pub fn check_amount(a: &str) -> Result<u64, ApiErr> {
    let a = a.trim();
    if a.is_empty() || a.len() > 18 || !a.chars().all(|c| c.is_ascii_digit()) {
        return Err(bad("BAD_AMOUNT", "amount is base units (8 decimals) as a digit string"));
    }
    let n: u64 = a.parse().map_err(|_| bad("BAD_AMOUNT", "amount too large"))?;
    if n < MIN_AMOUNT {
        return Err(bad("BELOW_MINIMUM", format!("1Click's minimum on this route is {MIN_AMOUNT} ({} ZEC)", formatted(&MIN_AMOUNT.to_string()))));
    }
    if n > MAX_AMOUNT {
        return Err(bad("ABOVE_CAP", format!("launch cap is {MAX_AMOUNT} ({} ZEC)", formatted(&MAX_AMOUNT.to_string()))));
    }
    Ok(n)
}

pub async fn quote(State(s): St, Json(r): Json<QuoteReq>) -> R {
    if std::env::var_os("AIRLOCK_PAUSED").is_some() {
        return Err(ApiErr(StatusCode::SERVICE_UNAVAILABLE, "PAUSED", "quotes are paused".into()));
    }
    let mut sess = s.load(&r.id).await?;
    let amount = check_amount(&r.amount)?;
    let sender = r.sender.trim();
    if !valid_solana(sender) {
        return Err(bad("BAD_SENDER", "sender is a Solana public key"));
    }
    if !r.dry && sess.deposit_address.is_some() {
        return Err(ApiErr(StatusCode::CONFLICT, "QUOTE_EXISTS", "this shield already has a deposit address; start a new shield with a fresh address".into()));
    }
    let req = quote_request(&sess.recipient, sender, amount, r.dry);
    let q = s.oneclick.quote(&req).await.map_err(upstream)?;
    q.verify_echo(&req).map_err(|e| ApiErr(StatusCode::BAD_GATEWAY, "ECHO_MISMATCH", e.to_string()))?;
    let qq = &q.quote;
    let fee_bps = qq.amount_in.parse::<u64>().ok().zip(qq.amount_out.parse::<u64>().ok()).filter(|(i, _)| *i > 0).map(|(i, o)| i.saturating_sub(o) * 10_000 / i);
    if !r.dry {
        let dep = qq.deposit_address.clone().ok_or_else(|| upstream(anyhow!("non-dry quote returned no depositAddress")))?;
        sess.deposit_address = Some(dep);
        sess.sender = Some(sender.into());
        sess.amount_in = Some(qq.amount_in.clone());
        sess.quoted_out = Some(qq.amount_out.clone());
        sess.deadline = qq.deadline.clone();
        s.save(&sess).await.map_err(internal)?;
    }
    Ok(Json(json!({
        "id": sess.id, "dry": r.dry, "depositAddress": qq.deposit_address, "mint": SOL_ZEC_MINT, "decimals": DECIMALS,
        "amountIn": qq.amount_in, "amountInFormatted": qq.amount_in_formatted,
        "amountOut": qq.amount_out, "amountOutFormatted": qq.amount_out_formatted, "minAmountOut": qq.min_amount_out,
        "fees": { "withdrawFee": qq.withdraw_fee, "refundFee": qq.refund_fee, "totalFeeBps": fee_bps, "shoreFeeBps": req.app_fees.first().map_or(0, |f| f.fee) },
        "minAmount": MIN_AMOUNT.to_string(), "deadline": qq.deadline, "timeEstimateSecs": qq.time_estimate,
        "recipient": sess.recipient, "correlationId": q.correlation_id,
        "amountInUsd": qq.amount_in_usd, "amountOutUsd": qq.amount_out_usd,
        "usdSource": "1Click quote", "quotedAt": chrono_now(),
    })))
}

#[derive(Deserialize)]
struct TxReq {
    #[serde(rename = "txHash")]
    tx_hash: String,
}

async fn submit_tx(State(s): St, UrlPath(id): UrlPath<String>, Json(r): Json<TxReq>) -> R {
    let mut sess = s.load(&id).await?;
    let dep = sess.deposit_address.clone().ok_or_else(|| bad("NO_QUOTE", "quote first"))?;
    if !(64..=90).contains(&r.tx_hash.len()) || !r.tx_hash.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(bad("BAD_TX", "not a Solana signature"));
    }
    let v = s.oneclick.submit(&r.tx_hash, &dep).await.map_err(upstream)?;
    sess.solana_tx = Some(r.tx_hash);
    s.save(&sess).await.map_err(internal)?;
    Ok(Json(json!({ "ok": true, "status": v["status"] })))
}

fn first_hash(raw: &Value, key: &str) -> Option<String> {
    let h = raw["swapDetails"][key].as_array()?.first()?;
    let h = if h.is_object() { &h["hash"] } else { h };
    h.as_str().filter(|s| !s.is_empty()).map(str::to_string)
}

async fn read_status(s: &Shore, deposit: &str) -> Result<StatusResponse, ApiErr> {
    s.oneclick.status(deposit).await.map_err(upstream)?.ok_or_else(|| ApiErr(StatusCode::NOT_FOUND, "NOT_FOUND", "1Click has no swap for this deposit address yet".into()))
}

/// Status body for the UI, from one 1Click status payload and the Zcash height of its delivery.
pub fn status_json(deposit: &str, r: &StatusResponse, landed_height: Option<u64>) -> Value {
    let raw = &r.raw;
    let ztx = first_hash(raw, "destinationChainTxHashes");
    json!({
        "depositAddress": deposit, "status": raw["status"], "terminal": r.status.terminal(), "updatedAt": raw["updatedAt"],
        "solanaTx": first_hash(raw, "originChainTxHashes"), "zcashTx": ztx,
        "landed": landed_height.zip(ztx.clone()).map(|(h, t)| json!({ "txid": t, "height": h })),
        "quotedOut": raw["swapDetails"]["amountOut"].as_str().or(raw["quoteResponse"]["quote"]["amountOut"].as_str()),
        "refundReason": raw["swapDetails"]["refundReason"],
    })
}

async fn status(State(s): St, UrlPath(deposit): UrlPath<String>) -> R {
    if !valid_deposit(&deposit) {
        return Err(bad("BAD_DEPOSIT", "not a deposit address"));
    }
    let r = read_status(&s, &deposit).await?;
    let h = match first_hash(&r.raw, "destinationChainTxHashes") {
        Some(t) => s.zcash.mined_height(&t).await.map_err(upstream)?,
        None => None,
    };
    Ok(Json(status_json(&deposit, &r, h)))
}

async fn readback(State(s): St, UrlPath(id): UrlPath<String>) -> R {
    let sess = s.load(&id).await?;
    let base = json!({ "id": sess.id, "recipient": sess.recipient });
    if !sess.ours {
        return Ok(Json(merge(base, json!({ "state": "UNAVAILABLE", "reason": "the address is not the demo wallet's; read the note in your own wallet" }))));
    }
    let Some(dep) = sess.deposit_address.clone() else { return Ok(Json(merge(base, json!({ "state": "WAITING", "reason": "no quote yet" })))) };
    let r = read_status(&s, &dep).await?;
    let Some(txid) = first_hash(&r.raw, "destinationChainTxHashes") else { return Ok(Json(merge(base, json!({ "state": "WAITING", "status": r.raw["status"] })))) };
    let (note, scanned) = s.zcash.readback(&txid).await.map_err(upstream)?;
    let quoted: Option<u64> = r.raw["swapDetails"]["amountOut"].as_str().or(sess.quoted_out.as_deref()).and_then(|x| x.parse().ok());
    Ok(Json(merge(base, match note {
        Some(n) => json!({
            "state": "LANDED", "pool": n.pool, "valueZat": n.value_zat, "height": n.height, "txid": txid,
            "quotedOutZat": quoted, "deltaZat": quoted.map(|q| n.value_zat as i64 - q as i64), "scannedTo": scanned,
        }),
        None => json!({ "state": "WAITING", "txid": txid, "scannedTo": scanned }),
    })))
}

#[derive(Deserialize)]
struct KeyReq {
    ufvk: String,
}

/// Read-back for a user-owned address: the user pastes their wallet's viewing key (uview1... or uvf1...). It is checked to
/// cover the session's u1, used for this one scan (from the block that holds the payout), then dropped.
async fn readback_key(State(s): St, UrlPath(id): UrlPath<String>, Json(r): Json<KeyReq>) -> R {
    let sess = s.load(&id).await?;
    let k = r.ufvk.trim();
    if k.len() > 2048 || !(k.starts_with("uview1") || k.starts_with("uvf1")) {
        return Err(bad("BAD_UFVK", "paste a mainnet unified full viewing key (uview1... or uvf1...), never a seed phrase or spending key"));
    }
    let owns = UfvkAddresses::new(k).map_err(|_| bad("BAD_UFVK", "this is not a valid unified viewing key"))?.owns(&sess.recipient);
    if !owns {
        return Err(bad("KEY_MISMATCH", "this viewing key does not cover the shield's address"));
    }
    let base = json!({ "id": sess.id, "recipient": sess.recipient, "source": "your-key" });
    let Some(dep) = sess.deposit_address.clone() else { return Ok(Json(merge(base, json!({ "state": "WAITING", "reason": "no quote yet" })))) };
    let st = read_status(&s, &dep).await?;
    let Some(txid) = first_hash(&st.raw, "destinationChainTxHashes") else { return Ok(Json(merge(base, json!({ "state": "WAITING", "status": st.raw["status"] })))) };
    let Some(h) = s.zcash.mined_height(&txid).await.map_err(upstream)? else { return Ok(Json(merge(base, json!({ "state": "WAITING", "txid": txid, "reason": "payout not mined yet" })))) };
    let (note, scanned) = s.zcash.readback_with(k, h.saturating_sub(1) as u32, &txid).await.map_err(upstream)?;
    let quoted: Option<u64> = st.raw["swapDetails"]["amountOut"].as_str().or(sess.quoted_out.as_deref()).and_then(|x| x.parse().ok());
    Ok(Json(merge(base, match note {
        Some(n) => json!({ "state": "LANDED", "pool": n.pool, "valueZat": n.value_zat, "height": n.height, "txid": txid,
            "quotedOutZat": quoted, "deltaZat": quoted.map(|q| n.value_zat as i64 - q as i64), "scannedTo": scanned }),
        None => json!({ "state": "WAITING", "txid": txid, "scannedTo": scanned }),
    })))
}

fn merge(mut a: Value, b: Value) -> Value {
    if let (Some(a), Value::Object(b)) = (a.as_object_mut(), b) {
        a.extend(b);
    }
    a
}

/// "What's still public" for one shield, from its session, the unauthenticated 1Click status
/// payload and the shape of the Zcash delivery transaction.
pub fn exposure_json(sess: &Session, raw: &Value, shape: Option<&TxShape>, reused: bool) -> Value {
    let row = |what: &str, value: Value, who: &str, source: &str, exposed: bool| json!({ "what": what, "value": value, "whoCanSee": who, "source": source, "exposed": exposed });
    let qr = &raw["quoteResponse"]["quoteRequest"];
    let sender = qr["refundTo"].as_str().map(str::to_string).or(sess.sender.clone());
    let dep = raw["quoteResponse"]["quote"]["depositAddress"].as_str().map(str::to_string).or(sess.deposit_address.clone());
    let amount_in = raw["swapDetails"]["amountIn"].as_str().or(raw["swapDetails"]["depositedAmount"].as_str()).map(str::to_string).or(sess.amount_in.clone());
    let solana_tx = first_hash(raw, "originChainTxHashes").or(sess.solana_tx.clone());
    let zcash_tx = first_hash(raw, "destinationChainTxHashes");
    let recipient = qr["recipient"].as_str().unwrap_or(&sess.recipient).to_string();
    let mut rows = vec![
        row("Solana sender (Phantom wallet)", json!(sender), "anyone: it signs the SPL transfer and is the refund address", "Solana chain + 1Click status refundTo", true),
        row("Deposit address", json!(dep), "anyone: the SPL transfer's destination", "Solana chain", true),
        row("Amount in and timing", json!({ "amountIn": amount_in, "solanaTx": solana_tx, "updatedAt": raw["updatedAt"] }), "anyone: amount and block time of the SPL transfer", "Solana chain", true),
        row("Recipient u1 via 1Click status", json!(recipient), "anyone holding the deposit address: /v0/status needs no key and returns the recipient", "1Click /v0/status", true),
    ];
    match shape {
        Some(t) if t.transparent_inputs > 0 => rows.push(row("Amount entering the shielded pool", json!(t.shielded_value_in), "anyone: the delivery tx spends transparent inputs, so its value balance is public", "Zcash chain", true)),
        Some(t) => rows.push(row("Amount entering the shielded pool", json!(t.shielded_value_in), "nobody outside: fully shielded delivery", "Zcash chain", false)),
        None => rows.push(row("Amount entering the shielded pool", json!(zcash_tx), "pending: delivery tx not read yet", "Zcash chain", true)),
    }
    rows.push(row("Note contents to the payer", json!(zcash_tx), "1Click / NEAR solver: the sender of a shielded output can decrypt it with its outgoing viewing key", "protocol", true));
    rows.push(row("Later spends of the note", Value::Null, "nobody: spending a shielded note reveals no link to this delivery", "protocol", false));
    json!({
        "id": sess.id, "rows": rows, "addressReused": reused,
        "publicOnStatus": identifiers(raw).into_iter().map(|(f, v)| json!({ "field": f, "value": v })).collect::<Vec<_>>(),
    })
}

async fn exposure(State(s): St, UrlPath(id): UrlPath<String>) -> R {
    let sess = s.load(&id).await?;
    let dep = sess.deposit_address.clone().ok_or_else(|| bad("NO_QUOTE", "quote first"))?;
    let r = read_status(&s, &dep).await?;
    let shape = match first_hash(&r.raw, "destinationChainTxHashes") {
        Some(t) => s.zcash.tx_shape(&t).await.unwrap_or(None),
        None => None,
    };
    // Reuse across shields: another session with the same recipient.
    let reused = s.store.recipient_count(&sess.recipient).await.map_err(upstream)? > 1;
    Ok(Json(exposure_json(&sess, &r.raw, shape.as_ref(), reused)))
}

/// Solana JSON-RPC methods the page needs: the ZEC balance and a blockhash for the one transfer.
/// Phantom broadcasts the signed transaction itself, so nothing that writes passes through here.
pub const RPC_METHODS: [&str; 4] = ["getTokenAccountsByOwner", "getLatestBlockhash", "getSignatureStatuses", "getBalance"];

/// The public mainnet endpoint, used only after the keyed provider has failed every retry.
pub const PUBLIC_SOLANA_RPC: &str = "https://api.mainnet-beta.solana.com";

/// Read-only Solana RPC proxy: a short per-request cache, retry with backoff on 429/5xx/network errors,
/// then one try on a public fallback. Provider URLs (which carry the API key) never appear in errors or logs.
pub struct RpcProxy {
    primary: Option<String>,
    fallback: Option<String>,
    backoff: Vec<std::time::Duration>,
    http: reqwest::Client,
    cache: std::sync::Mutex<std::collections::HashMap<String, (std::time::Instant, Value)>>,
}

/// How long a successful answer is reused. Signature statuses are never cached: the page polls them for progress.
pub fn rpc_ttl(method: &str) -> Option<std::time::Duration> {
    use std::time::Duration;
    match method {
        "getBalance" | "getTokenAccountsByOwner" => Some(Duration::from_secs(8)),
        "getLatestBlockhash" => Some(Duration::from_secs(2)),
        _ => None,
    }
}

enum Try { Ok(Value), Retry(String), Fail(String) }

impl RpcProxy {
    pub fn new(primary: Option<String>, fallback: Option<String>, backoff: Vec<std::time::Duration>) -> Self {
        Self { primary, fallback, backoff, http: reqwest::Client::builder().timeout(std::time::Duration::from_secs(10)).build().expect("http client"), cache: Default::default() }
    }
    /// SHORE_SOLANA_RPC (keyed provider), SHORE_SOLANA_RPC_FALLBACK (default: public mainnet; "off" disables).
    pub fn from_env() -> Self {
        let fallback = match std::env::var("SHORE_SOLANA_RPC_FALLBACK") { Ok(v) if v == "off" => None, Ok(v) => Some(v), Err(_) => Some(PUBLIC_SOLANA_RPC.into()) };
        use std::time::Duration;
        Self::new(std::env::var("SHORE_SOLANA_RPC").ok(), fallback, vec![Duration::from_millis(300), Duration::from_millis(900)])
    }

    async fn once(&self, url: &str, v: &Value) -> Try {
        match self.http.post(url).json(v).send().await {
            Err(e) => Try::Retry(format!("network: {}", e.without_url())),
            Ok(r) => {
                let st = r.status();
                if st.as_u16() == 429 || st.is_server_error() { return Try::Retry(format!("HTTP {}", st.as_u16())); }
                if !st.is_success() { return Try::Fail(format!("HTTP {}", st.as_u16())); }
                match r.json::<Value>().await {
                    Ok(j) if j.get("error").and_then(|e| e["code"].as_i64()) == Some(429) => Try::Retry("rate limited".into()),
                    Ok(j) => Try::Ok(j),
                    Err(e) => Try::Retry(format!("bad body: {}", e.without_url())),
                }
            }
        }
    }

    pub async fn call(&self, v: &Value) -> Result<Value, ApiErr> {
        let m = v["method"].as_str().unwrap_or("");
        if !RPC_METHODS.contains(&m) || v.as_object().map_or(true, |o| o.keys().any(|k| !["jsonrpc", "id", "method", "params"].contains(&k.as_str()))) {
            return Err(bad("RPC_METHOD", format!("only {} are proxied", RPC_METHODS.join(", "))));
        }
        if serde_json::to_vec(v).map_or(true, |b| b.len() > 4096) {
            return Err(bad("RPC_SIZE", "request too large"));
        }
        let key = format!("{m}|{}", v["params"]);
        let ttl = rpc_ttl(m);
        if let Some(t) = ttl {
            let c = self.cache.lock().unwrap();
            if let Some((at, res)) = c.get(&key) { if at.elapsed() < t { let mut r = res.clone(); r["id"] = v["id"].clone(); return Ok(r); } }
        }
        let mut last = String::from("no provider configured");
        let mut got = None;
        if let Some(p) = &self.primary {
            for i in 0..=self.backoff.len() {
                match self.once(p, v).await {
                    Try::Ok(j) => { got = Some(j); break; }
                    Try::Fail(e) => { last = format!("provider {e}"); break; }
                    Try::Retry(e) => { last = format!("provider {e}"); if let Some(d) = self.backoff.get(i) { tokio::time::sleep(*d).await; } }
                }
            }
        }
        if got.is_none() {
            if let Some(f) = &self.fallback {
                match self.once(f, v).await {
                    Try::Ok(j) => got = Some(j),
                    Try::Retry(e) | Try::Fail(e) => last = format!("{last}; fallback {e}"),
                }
            }
        }
        let Some(mut j) = got else { return Err(upstream(anyhow!("solana rpc: {last}"))) };
        if let (Some(t), true) = (ttl, j.get("result").is_some()) {
            let mut c = self.cache.lock().unwrap();
            if c.len() > 2000 { c.retain(|_, (at, _)| at.elapsed() < t); }
            c.insert(key, (std::time::Instant::now(), j.clone()));
        }
        j["id"] = v["id"].clone();
        Ok(j)
    }
}

static RPC: std::sync::OnceLock<RpcProxy> = std::sync::OnceLock::new();

/// Proxies allowed read methods so the provider key stays server-side.
async fn rpc(Json(v): Json<Value>) -> Result<Json<Value>, ApiErr> {
    let p = RPC.get_or_init(RpcProxy::from_env);
    if p.primary.is_none() && p.fallback.is_none() {
        return Err(ApiErr(StatusCode::SERVICE_UNAVAILABLE, "RPC_UNSET", "SHORE_SOLANA_RPC is not set".into()));
    }
    p.call(&v).await.map(Json)
}

// ---------- production implementations ----------

/// Addresses from a UFVK only. Index `i` is the Orchard-only diversified address at `i`.
pub struct UfvkAddresses {
    ufvk: zcash_keys::keys::UnifiedFullViewingKey,
}

impl UfvkAddresses {
    pub fn new(ufvk: &str) -> Result<Self> {
        let ufvk = zcash_keys::keys::UnifiedFullViewingKey::decode(&zcash_protocol::consensus::MainNetwork, ufvk).map_err(|e| anyhow!("ufvk: {e}"))?;
        Ok(Self { ufvk })
    }
}

impl AddressSource for UfvkAddresses {
    fn at(&self, i: u32) -> Result<String> {
        let ua = self.ufvk.address(zip32::DiversifierIndex::from(i), zcash_keys::keys::UnifiedAddressRequest::ORCHARD).map_err(|e| anyhow!("ua at {i}: {e:?}"))?;
        to_revision_1(&ua.encode(&zcash_protocol::consensus::MainNetwork))
    }

    fn owns(&self, u1: &str) -> bool {
        use zcash_address::unified::{Address, Container, Encoding, Receiver};
        let (Ok((_, _, a)), Some(fvk)) = (Address::decode(u1), self.ufvk.orchard()) else { return false };
        a.items().iter().any(|r| match r {
            Receiver::Orchard(b) => Option::<orchard::Address>::from(orchard::Address::from_raw_address_bytes(b)).is_some_and(|o| fvk.scope_for_address(&o).is_some()),
            _ => false,
        })
    }
}

pub struct LiveOneClick {
    pub tor: zcash_client_backend::tor::Client,
}

#[async_trait]
impl OneClickApi for LiveOneClick {
    async fn quote(&self, q: &QuoteRequest) -> Result<QuoteResponse> {
        crate::oneclick::Client::with_transport(crate::transport::Transport::isolated(&self.tor)).quote(q).await
    }
    async fn status(&self, deposit: &str) -> Result<Option<StatusResponse>> {
        match crate::oneclick::Client::with_transport(crate::transport::Transport::isolated(&self.tor)).status(deposit).await {
            Ok(r) => Ok(Some(r)),
            Err(e) if e.to_string().contains("HTTP 404") => Ok(None),
            Err(e) => Err(e),
        }
    }
    async fn submit(&self, tx: &str, deposit: &str) -> Result<Value> {
        crate::oneclick::Client::with_transport(crate::transport::Transport::isolated(&self.tor)).submit_deposit(tx, deposit).await
    }
}

/// lightwalletd for heights and the view-key scan, a public explorer for the delivery tx shape.
pub struct LiveZcash {
    pub ufvk: String,
    pub db: PathBuf,
    pub birthday: u32,
    pub http: reqwest::Client,
    watch: Mutex<Option<crate::notewatch::NoteWatch>>,
}

impl LiveZcash {
    pub fn new(ufvk: String, db: PathBuf, birthday: u32) -> Self {
        Self { ufvk, db, birthday, http: reqwest::Client::new(), watch: Mutex::new(None) }
    }
}

const NET: zcash_protocol::consensus::Network = zcash_protocol::consensus::Network::MainNetwork;

#[async_trait]
impl ZcashApi for LiveZcash {
    async fn mined_height(&self, txid: &str) -> Result<Option<u64>> {
        match crate::notewatch::mined_height(NET, txid).await {
            Ok(h) => Ok(Some(h)),
            Err(e) if e.to_string().contains("not mined") || e.to_string().contains("NotFound") => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn readback(&self, txid: &str) -> Result<(Option<Note>, u32)> {
        let mut g = self.watch.lock().await;
        if g.is_none() {
            *g = Some(crate::notewatch::NoteWatch::open(&self.db, NET, &self.ufvk, self.birthday).await?);
        }
        let nw = g.as_mut().unwrap();
        let scanned = nw.sync().await?;
        let note = nw.landed()?.into_iter().find(|n| n.txid == txid).map(|n| Note { pool: n.pool, value_zat: n.value_zat, height: n.mined_height });
        Ok((note, scanned))
    }

    async fn readback_with(&self, ufvk: &str, from_height: u32, txid: &str) -> Result<(Option<Note>, u32)> {
        let dir = std::env::temp_dir().join(format!("shore-rb-{}", hex::encode(crate::wallet::random_bytes::<8>()?)));
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("view.sqlite");
        let r = async {
            let mut nw = crate::notewatch::NoteWatch::open(&path, NET, ufvk, from_height).await?;
            let scanned = nw.sync().await?;
            let note = nw.landed()?.into_iter().find(|n| n.txid == txid).map(|n| Note { pool: n.pool, value_zat: n.value_zat, height: n.mined_height });
            Ok::<_, anyhow::Error>((note, scanned))
        }
        .await;
        let _ = std::fs::remove_dir_all(&dir);
        r.map_err(|e| anyhow!("{}", e.to_string().replace(ufvk, "<ufvk>")))
    }

    async fn tx_shape(&self, txid: &str) -> Result<Option<TxShape>> {
        if txid.len() != 64 || !txid.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("bad txid");
        }
        let v: Value = self.http.get(format!("https://api.blockchair.com/zcash/raw/transaction/{txid}")).send().await?.json().await?;
        Ok(shape_from_blockchair(&v, txid))
    }
}

/// Transparent input count and value entering the shielded pools, from a blockchair raw-tx body.
pub fn shape_from_blockchair(v: &Value, txid: &str) -> Option<TxShape> {
    let t = &v["data"][txid]["decoded_raw_transaction"];
    let vin = t["vin"].as_array()?;
    let bal = |x: &Value| x.as_i64().unwrap_or(0);
    let sum = bal(&t["valueBalanceZat"]) + bal(&t["orchard"]["valueBalanceZat"]) + bal(&t["ironwood"]["valueBalanceZat"]);
    Some(TxShape { transparent_inputs: vin.iter().filter(|i| i.get("coinbase").is_none()).count() as u32, shielded_value_in: -sum })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amounts() {
        assert_eq!(check_amount("140000").unwrap(), 140_000);
        assert_eq!(check_amount("133334").unwrap(), 133_334);
        assert_eq!(check_amount("133333").unwrap_err().1, "BELOW_MINIMUM");
        assert_eq!(check_amount("2000001").unwrap_err().1, "ABOVE_CAP");
        for b in ["", "1.5", "-1", "1e6", "99999999999999999999"] {
            assert!(check_amount(b).is_err(), "{b}");
        }
    }

    #[test]
    fn formatting() {
        assert_eq!(formatted("133334"), "0.00133334");
        assert_eq!(formatted("2000000"), "0.02");
        assert_eq!(formatted("100000000"), "1");
    }

    #[test]
    fn transparent_and_sapling_refused() {
        assert_eq!(check_own_address("t1UH6mFsvCCzTYvKgTgNCbNBW9t8Vd1NCoq").unwrap_err().1, "TRANSPARENT_REFUSED");
        assert_eq!(check_own_address("t3Vz22vK5z2LcKEdg16Yv4FFneEL1zg9ojd").unwrap_err().1, "TRANSPARENT_REFUSED");
        assert_eq!(check_own_address("tex1s2rt77ggv6q989lr49rkgzmh5slsksa9khdgte").unwrap_err().1, "TRANSPARENT_REFUSED");
        assert_eq!(check_own_address("zs1z7rejlpsa98s2rrrfkwmaxu53e4ue0ulcrw0h4x5g8jl04tak0d3mm47vdtahatqrlkngh9sly").unwrap_err().1, "NOT_UNIFIED");
        assert_eq!(check_own_address("u1notreal").unwrap_err().1, "NOT_UNIFIED");
    }

    #[test]
    fn solana_keys() {
        assert!(valid_solana("ia3MLukFMa6zci4nyahxsKVT14yLkFzUkZs3EL5Vsa5"));
        assert!(!valid_solana("0x2527D02599Ba641c19FEa793cD0F167589a0f10D"));
        assert!(!valid_solana("short"));
    }

    #[test]
    fn ufvk_source_owns_its_addresses_only() {
        let k = |seed: u8| zcash_keys::keys::UnifiedSpendingKey::from_seed(&zcash_protocol::consensus::MainNetwork, &[seed; 32], zip32::AccountId::ZERO).unwrap().to_unified_full_viewing_key().encode(&zcash_protocol::consensus::MainNetwork);
        let (a, b) = (UfvkAddresses::new(&k(1)).unwrap(), UfvkAddresses::new(&k(2)).unwrap());
        let (a0, a5) = (a.at(0).unwrap(), a.at(5).unwrap());
        assert!(a0.starts_with("u1") && a0 != a5);
        assert_eq!(receivers(&a5).unwrap(), vec!["orchard"]);
        assert!(a.owns(&a0) && a.owns(&a5));
        assert!(!b.owns(&a0));
        assert_eq!(check_own_address(&a0).unwrap().0, a0);
    }
}

/// RFC 3339 UTC timestamp for when a quote was fetched (labels the USD values).
fn chrono_now() -> String {
    let d = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let (days, rem) = (d.div_euclid(86400), d.rem_euclid(86400));
    // civil-from-days (Howard Hinnant)
    let z = days + 719468; let era = z.div_euclid(146097); let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); let mp = (5 * doy + 2) / 153;
    let dd = doy - (153 * mp + 2) / 5 + 1; let m = if mp < 10 { mp + 3 } else { mp - 9 };
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", if m <= 2 { y + 1 } else { y }, m, dd, rem / 3600, rem % 3600 / 60, rem % 60)
}
