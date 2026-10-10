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
use std::{collections::HashSet, path::PathBuf, sync::Arc};
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
    /// The gitignored work directory: `ua-index` (shared with the CLI) and `shore/`.
    pub dir: PathBuf,
    pub oneclick: Arc<dyn OneClickApi>,
    pub zcash: Arc<dyn ZcashApi>,
    pub addrs: Arc<dyn AddressSource>,
    lock: Mutex<()>,
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

impl Shore {
    pub fn new(dir: PathBuf, oneclick: Arc<dyn OneClickApi>, zcash: Arc<dyn ZcashApi>, addrs: Arc<dyn AddressSource>) -> Result<Self> {
        std::fs::create_dir_all(dir.join("shore/sessions"))?;
        Ok(Self { dir, oneclick, zcash, addrs, lock: Mutex::new(()) })
    }

    fn session_path(&self, id: &str) -> PathBuf {
        self.dir.join("shore/sessions").join(format!("{id}.json"))
    }

    pub fn save(&self, s: &Session) -> Result<()> {
        std::fs::write(self.session_path(&s.id), serde_json::to_vec_pretty(s)?)?;
        Ok(())
    }

    pub fn load(&self, id: &str) -> Result<Session, ApiErr> {
        if id.len() != 12 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(bad("BAD_ID", "not a shield id"));
        }
        let b = std::fs::read(self.session_path(id)).map_err(|_| ApiErr(StatusCode::NOT_FOUND, "NOT_FOUND", "no such shield".into()))?;
        serde_json::from_slice(&b).map_err(|e| internal(e.into()))
    }

    pub fn by_deposit(&self, deposit: &str) -> Option<Session> {
        std::fs::read_dir(self.dir.join("shore/sessions")).ok()?.flatten().find_map(|e| {
            let s: Session = serde_json::from_slice(&std::fs::read(e.path()).ok()?).ok()?;
            (s.deposit_address.as_deref() == Some(deposit)).then_some(s)
        })
    }

    fn used_path(&self) -> PathBuf {
        self.dir.join("shore/used-addresses.txt")
    }

    /// Every recipient any shield on this server has used.
    pub fn used(&self) -> HashSet<String> {
        std::fs::read_to_string(self.used_path()).unwrap_or_default().lines().map(str::to_string).collect()
    }

    fn mark_used(&self, u1: &str) -> Result<()> {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new().create(true).append(true).open(self.used_path())?;
        writeln!(f, "{u1}")?;
        Ok(())
    }

    /// Next never-used index. The index file is advanced before the address is returned, so a crash
    /// can skip an index but never hand one out twice.
    async fn fresh(&self) -> Result<Session> {
        let _g = self.lock.lock().await;
        let p = self.dir.join("ua-index");
        let i: u32 = match std::fs::read_to_string(&p) {
            Ok(s) => s.trim().parse().map_err(|_| anyhow!("ua-index is not a number"))?,
            Err(_) => 0,
        };
        std::fs::write(&p, (i + 1).to_string())?;
        let u1 = self.addrs.at(i)?;
        if self.used().contains(&u1) {
            anyhow::bail!("index {i} produced an address already used");
        }
        self.mark_used(&u1)?;
        let s = Session { id: hex::encode(crate::wallet::random_bytes::<6>()?), recipient: u1, source: "fresh".into(), ua_index: Some(i), ours: true, created_at: now(), deposit_address: None, sender: None, amount_in: None, quoted_out: None, deadline: None, solana_tx: None };
        self.save(&s)?;
        Ok(s)
    }

    async fn own(&self, addr: &str) -> Result<Session, ApiErr> {
        let (u1, _) = check_own_address(addr)?;
        let _g = self.lock.lock().await;
        if self.used().contains(&u1) {
            return Err(ApiErr(StatusCode::CONFLICT, "ADDRESS_REUSED", "this address already received a shield; reuse links the two deposits on the public 1Click status page. Use a fresh address".into()));
        }
        self.mark_used(&u1).map_err(internal)?;
        let ours = self.addrs.owns(&u1);
        let s = Session { id: hex::encode(crate::wallet::random_bytes::<6>().map_err(internal)?), recipient: u1, source: "own".into(), ua_index: None, ours, created_at: now(), deposit_address: None, sender: None, amount_in: None, quoted_out: None, deadline: None, solana_tx: None };
        self.save(&s).map_err(internal)?;
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

pub fn router(s: Arc<Shore>) -> Router {
    Router::new()
        .route("/api/shield/address", post(address))
        .route("/api/shield/{id}/tx", post(submit_tx))
        .route("/api/quote", post(quote))
        .route("/api/status/{deposit}", get(status))
        .route("/api/readback/{id}", get(readback))
        .route("/api/exposure/{id}", get(exposure))
        .with_state(s)
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
struct QuoteReq {
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
    }
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

async fn quote(State(s): St, Json(r): Json<QuoteReq>) -> R {
    if std::env::var_os("AIRLOCK_PAUSED").is_some() {
        return Err(ApiErr(StatusCode::SERVICE_UNAVAILABLE, "PAUSED", "quotes are paused".into()));
    }
    let mut sess = s.load(&r.id)?;
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
        s.save(&sess).map_err(internal)?;
    }
    Ok(Json(json!({
        "id": sess.id, "dry": r.dry, "depositAddress": qq.deposit_address, "mint": SOL_ZEC_MINT, "decimals": DECIMALS,
        "amountIn": qq.amount_in, "amountInFormatted": qq.amount_in_formatted,
        "amountOut": qq.amount_out, "amountOutFormatted": qq.amount_out_formatted, "minAmountOut": qq.min_amount_out,
        "fees": { "withdrawFee": qq.withdraw_fee, "refundFee": qq.refund_fee, "totalFeeBps": fee_bps },
        "minAmount": MIN_AMOUNT.to_string(), "deadline": qq.deadline, "timeEstimateSecs": qq.time_estimate,
        "recipient": sess.recipient, "correlationId": q.correlation_id,
    })))
}

#[derive(Deserialize)]
struct TxReq {
    #[serde(rename = "txHash")]
    tx_hash: String,
}

async fn submit_tx(State(s): St, UrlPath(id): UrlPath<String>, Json(r): Json<TxReq>) -> R {
    let mut sess = s.load(&id)?;
    let dep = sess.deposit_address.clone().ok_or_else(|| bad("NO_QUOTE", "quote first"))?;
    if !(64..=90).contains(&r.tx_hash.len()) || !r.tx_hash.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(bad("BAD_TX", "not a Solana signature"));
    }
    let v = s.oneclick.submit(&r.tx_hash, &dep).await.map_err(upstream)?;
    sess.solana_tx = Some(r.tx_hash);
    s.save(&sess).map_err(internal)?;
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
    let sess = s.load(&id)?;
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
    let sess = s.load(&id)?;
    let dep = sess.deposit_address.clone().ok_or_else(|| bad("NO_QUOTE", "quote first"))?;
    let r = read_status(&s, &dep).await?;
    let shape = match first_hash(&r.raw, "destinationChainTxHashes") {
        Some(t) => s.zcash.tx_shape(&t).await.unwrap_or(None),
        None => None,
    };
    // Reuse across shields: another session with the same recipient.
    let reused = std::fs::read_dir(s.dir.join("shore/sessions")).map_err(|e| internal(e.into()))?.flatten().filter_map(|e| serde_json::from_slice::<Session>(&std::fs::read(e.path()).ok()?).ok()).filter(|x| x.recipient == sess.recipient).count() > 1;
    Ok(Json(exposure_json(&sess, &r.raw, shape.as_ref(), reused)))
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
