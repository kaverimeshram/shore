//! SHORE HTTP API against mocked 1Click and Zcash, plus a replay of the recorded mainnet run
//! (2026-10-10: 0.0014 ZEC from Phantom, 115,000 zat landed in Ironwood at 3,512,504).
use airlock::oneclick::{QuoteRequest, QuoteResponse, StatusResponse};
use airlock::shore::*;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::{collections::{HashMap, HashSet}, path::PathBuf, sync::{Arc, Mutex}};

const SENDER: &str = "ia3MLukFMa6zci4nyahxsKVT14yLkFzUkZs3EL5Vsa5";
const LIVE_U1: &str = "u1uufrl3tucq9vjgvssws978y0nm6d6cncqk75qs62htzctwytgpqqsdwlpe5jvl83xqyn2vtkptk4cw535n575yf3nrwpcrydpgrkpgmk";
const LIVE_DEPOSIT: &str = "DMDdCzumrZBaPPjKBP33o1RgDhW2F2BZPzyYZEk9SjbM";
const LIVE_ZTX: &str = "fa8d4d6bd210c6150e98142ec1ae9a25706ec19bd591a775823f6616f95fa408";

fn fixture(name: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(format!("{}/tests/fixtures/shore-live/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap()).unwrap()
}

#[derive(Default)]
struct MockOneClick {
    /// Recorded response to return instead of echoing; `tamper` rewrites the echo.
    recorded: Option<Value>,
    tamper: Option<(&'static str, &'static str)>,
    statuses: Mutex<HashMap<String, Value>>,
    quotes: Mutex<Vec<QuoteRequest>>,
    counter: Mutex<u32>,
}

#[async_trait]
impl OneClickApi for MockOneClick {
    async fn quote(&self, q: &QuoteRequest) -> Result<QuoteResponse> {
        self.quotes.lock().unwrap().push(q.clone());
        let v = match &self.recorded {
            Some(r) => r.clone(),
            None => {
                let mut n = self.counter.lock().unwrap();
                *n += 1;
                let mut echo = serde_json::to_value(q)?;
                if let Some((k, v)) = self.tamper {
                    echo[k] = v.into();
                }
                let out = q.amount.parse::<u64>()? - 32_000;
                json!({ "quote": {
                    "amountIn": q.amount, "amountInFormatted": "x", "minAmountIn": q.amount, "amountOut": out.to_string(), "amountOutFormatted": "y",
                    "minAmountOut": (out * 99 / 100).to_string(), "timeEstimate": 135, "withdrawFee": "32000", "refundFee": "844",
                    "deadline": "2026-10-13T06:47:48.000Z",
                    "depositAddress": if q.dry { Value::Null } else { format!("Dep{:041}", *n).into() } },
                    "correlationId": "c", "quoteRequest": echo })
            }
        };
        Ok(serde_json::from_value(v)?)
    }
    async fn status(&self, d: &str) -> Result<Option<StatusResponse>> {
        Ok(self.statuses.lock().unwrap().get(d).map(|raw| StatusResponse { status: serde_json::from_value(raw["status"].clone()).unwrap(), raw: raw.clone() }))
    }
    async fn submit(&self, _: &str, _: &str) -> Result<Value> {
        Ok(json!({ "status": "KNOWN_DEPOSIT_TX" }))
    }
}

#[derive(Default)]
struct MockZcash {
    heights: HashMap<String, u64>,
    notes: HashMap<String, Note>,
    shapes: HashMap<String, TxShape>,
}

#[async_trait]
impl ZcashApi for MockZcash {
    async fn mined_height(&self, t: &str) -> Result<Option<u64>> {
        Ok(self.heights.get(t).copied())
    }
    async fn readback(&self, t: &str) -> Result<(Option<Note>, u32)> {
        Ok((self.notes.get(t).cloned(), 3_512_512))
    }
    async fn tx_shape(&self, t: &str) -> Result<Option<TxShape>> {
        Ok(self.shapes.get(t).cloned())
    }
}

/// Deterministic test wallet (seed 7s), never the demo wallet.
fn test_addrs() -> Arc<dyn AddressSource> {
    let net = zcash_protocol::consensus::MainNetwork;
    let k = zcash_keys::keys::UnifiedSpendingKey::from_seed(&net, &[7u8; 32], zip32::AccountId::ZERO).unwrap().to_unified_full_viewing_key().encode(&net);
    Arc::new(UfvkAddresses::new(&k).unwrap())
}

struct SetAddrs(HashSet<String>);
impl AddressSource for SetAddrs {
    fn at(&self, _: u32) -> Result<String> {
        anyhow::bail!("replay issues no fresh address")
    }
    fn owns(&self, u: &str) -> bool {
        self.0.contains(u)
    }
}

fn tmp() -> PathBuf {
    let d = std::env::temp_dir().join(format!("shore-test-{}", hex::encode(airlock::wallet::random_bytes::<6>().unwrap())));
    std::fs::create_dir_all(&d).unwrap();
    d
}

struct Srv {
    base: String,
    http: reqwest::Client,
    dir: PathBuf,
}

impl Srv {
    async fn start(oc: Arc<dyn OneClickApi>, z: Arc<dyn ZcashApi>, a: Arc<dyn AddressSource>) -> Self {
        let dir = tmp();
        let shore = Arc::new(Shore::new(dir.clone(), oc, z, a).unwrap());
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", l.local_addr().unwrap());
        tokio::spawn(async move { axum::serve(l, standalone_router(shore)).await.unwrap() });
        Self { base, http: reqwest::Client::new(), dir }
    }
    async fn post(&self, path: &str, body: Value) -> (u16, Value) {
        let r = self.http.post(format!("{}{path}", self.base)).json(&body).send().await.unwrap();
        (r.status().as_u16(), r.json().await.unwrap())
    }
    async fn get(&self, path: &str) -> (u16, Value) {
        let r = self.http.get(format!("{}{path}", self.base)).send().await.unwrap();
        (r.status().as_u16(), r.json().await.unwrap())
    }
}

#[tokio::test]
async fn fresh_addresses_never_repeat_and_index_persists() {
    let s = Srv::start(Arc::new(MockOneClick::default()), Arc::new(MockZcash::default()), test_addrs()).await;
    std::fs::write(s.dir.join("ua-index"), "223").unwrap();
    let mut seen = HashSet::new();
    for i in 0..5u32 {
        let (c, v) = s.post("/api/shield/address", json!({ "mode": "fresh" })).await;
        assert_eq!(c, 200, "{v}");
        assert_eq!(v["uaIndex"], 223 + i);
        assert_eq!(v["receivers"], json!(["orchard"]));
        assert_eq!(v["readbackAvailable"], true);
        let u = v["recipient"].as_str().unwrap().to_string();
        assert!(u.starts_with("u1"));
        assert!(seen.insert(u), "address handed out twice");
    }
    assert_eq!(std::fs::read_to_string(s.dir.join("ua-index")).unwrap(), "228");
    // a fresh address offered back as "own" is refused: it is already used
    let used = seen.iter().next().unwrap().clone();
    let (c, v) = s.post("/api/shield/address", json!({ "mode": "own", "address": used })).await;
    assert_eq!((c, v["code"].as_str()), (409, Some("ADDRESS_REUSED")));
}

#[tokio::test]
async fn own_address_rules() {
    let s = Srv::start(Arc::new(MockOneClick::default()), Arc::new(MockZcash::default()), test_addrs()).await;
    let (c, v) = s.post("/api/shield/address", json!({ "mode": "own", "address": "t1UH6mFsvCCzTYvKgTgNCbNBW9t8Vd1NCoq" })).await;
    assert_eq!((c, v["code"].as_str()), (400, Some("TRANSPARENT_REFUSED")));
    let (c, v) = s.post("/api/shield/address", json!({ "mode": "own", "address": LIVE_U1 })).await;
    assert_eq!(c, 200, "{v}");
    assert_eq!((v["source"].as_str(), v["readbackAvailable"].as_bool()), (Some("own"), Some(false)));
    let (c, v) = s.post("/api/shield/address", json!({ "mode": "own", "address": format!(" {LIVE_U1} ") })).await;
    assert_eq!((c, v["code"].as_str()), (409, Some("ADDRESS_REUSED")));
    let (c, _) = s.post("/api/shield/address", json!({ "mode": "nope" })).await;
    assert_eq!(c, 400);
}

#[tokio::test]
async fn quote_rules_and_shape() {
    let oc = Arc::new(MockOneClick::default());
    let s = Srv::start(oc.clone(), Arc::new(MockZcash::default()), test_addrs()).await;
    let (_, a) = s.post("/api/shield/address", json!({ "mode": "fresh" })).await;
    let id = a["id"].as_str().unwrap();
    let (c, v) = s.post("/api/quote", json!({ "id": id, "amount": "133333", "sender": SENDER })).await;
    assert_eq!((c, v["code"].as_str()), (400, Some("BELOW_MINIMUM")));
    let (c, v) = s.post("/api/quote", json!({ "id": id, "amount": "140000", "sender": "0xabc" })).await;
    assert_eq!((c, v["code"].as_str()), (400, Some("BAD_SENDER")));
    let (c, v) = s.post("/api/quote", json!({ "id": "000000000000", "amount": "140000", "sender": SENDER })).await;
    assert_eq!(c, 404, "{v}");
    assert!(oc.quotes.lock().unwrap().is_empty(), "nothing reaches 1Click before validation passes");

    let (c, v) = s.post("/api/quote", json!({ "id": id, "amount": "140000", "sender": SENDER, "dry": true })).await;
    assert_eq!(c, 200, "{v}");
    assert!(v["depositAddress"].is_null());
    let (c, v) = s.post("/api/quote", json!({ "id": id, "amount": "140000", "sender": SENDER })).await;
    assert_eq!(c, 200, "{v}");
    assert_eq!(v["amountOut"], "108000");
    assert_eq!(v["fees"]["totalFeeBps"], 2285);
    assert_eq!(v["fees"]["withdrawFee"], "32000");
    assert_eq!(v["minAmount"], "133334");
    assert_eq!(v["mint"], SOL_ZEC_MINT);
    assert_eq!(v["recipient"], a["recipient"]);
    assert!(v["depositAddress"].as_str().unwrap().starts_with("Dep"));
    let q = oc.quotes.lock().unwrap().last().unwrap().clone();
    assert_eq!((q.origin_asset.as_str(), q.recipient.as_str(), q.refund_to.as_str()), (SOL_ZEC_ASSET, a["recipient"].as_str().unwrap(), SENDER));
    // a second real quote on the same shield would tie two deposits to one recipient
    let (c, v) = s.post("/api/quote", json!({ "id": id, "amount": "140000", "sender": SENDER })).await;
    assert_eq!((c, v["code"].as_str()), (409, Some("QUOTE_EXISTS")));
}

#[tokio::test]
async fn quote_with_tampered_echo_is_refused() {
    for field in ["recipient", "refundTo", "amount"] {
        let oc = Arc::new(MockOneClick { tamper: Some((field, "evil")), ..Default::default() });
        let s = Srv::start(oc, Arc::new(MockZcash::default()), test_addrs()).await;
        let (_, a) = s.post("/api/shield/address", json!({ "mode": "fresh" })).await;
        let (c, v) = s.post("/api/quote", json!({ "id": a["id"], "amount": "140000", "sender": SENDER })).await;
        assert_eq!((c, v["code"].as_str()), (502, Some("ECHO_MISMATCH")), "{field}");
    }
}

#[tokio::test]
async fn status_progresses_to_landed() {
    let oc = Arc::new(MockOneClick::default());
    let mut z = MockZcash::default();
    z.heights.insert("ab".repeat(32), 3_600_000);
    let s = Srv::start(oc.clone(), Arc::new(z), test_addrs()).await;
    let dep = "Dep00000000000000000000000000000000000000001";
    let (c, _) = s.get(&format!("/api/status/{dep}")).await;
    assert_eq!(c, 404);
    oc.statuses.lock().unwrap().insert(dep.into(), json!({ "status": "PENDING_DEPOSIT", "updatedAt": "t", "swapDetails": {} }));
    let (c, v) = s.get(&format!("/api/status/{dep}")).await;
    assert_eq!(c, 200);
    assert_eq!((v["status"].as_str(), v["terminal"].as_bool(), v["landed"].is_null()), (Some("PENDING_DEPOSIT"), Some(false), true));
    oc.statuses.lock().unwrap().insert(dep.into(), json!({ "status": "SUCCESS", "updatedAt": "t", "swapDetails": { "amountOut": "108000",
        "originChainTxHashes": [{ "hash": "sig", "explorerUrl": "" }], "destinationChainTxHashes": [{ "hash": "ab".repeat(32), "explorerUrl": "" }] } }));
    let (_, v) = s.get(&format!("/api/status/{dep}")).await;
    assert_eq!(v["terminal"], true);
    assert_eq!(v["landed"], json!({ "txid": "ab".repeat(32), "height": 3_600_000 }));
    let (c, _) = s.get("/api/status/bad!addr").await;
    assert_eq!(c, 400);
}

/// The recorded mainnet run, replayed through every endpoint.
#[tokio::test]
async fn replay_live_run() {
    let status = fixture("status-final.json");
    let oc = Arc::new(MockOneClick { recorded: Some(fixture("quote.json")), ..Default::default() });
    oc.statuses.lock().unwrap().insert(LIVE_DEPOSIT.into(), status);
    let mut z = MockZcash::default();
    z.heights.insert(LIVE_ZTX.into(), 3_512_504);
    // the note as the view-only scan read it: `airlock note main` -> LANDED pool=ironwood value_zat=115000
    z.notes.insert(LIVE_ZTX.into(), Note { pool: "ironwood".into(), value_zat: 115_000, height: Some(3_512_504) });
    let shape = shape_from_blockchair(&fixture("delivery-tx.json"), LIVE_ZTX).unwrap();
    assert_eq!(shape, TxShape { transparent_inputs: 2, shielded_value_in: 115_000 });
    z.shapes.insert(LIVE_ZTX.into(), shape);
    let s = Srv::start(oc, Arc::new(z), Arc::new(SetAddrs([LIVE_U1.to_string()].into()))).await;

    let (c, a) = s.post("/api/shield/address", json!({ "mode": "own", "address": LIVE_U1 })).await;
    assert_eq!(c, 200, "{a}");
    assert_eq!(a["readbackAvailable"], true);
    let id = a["id"].as_str().unwrap();
    let (c, q) = s.post("/api/quote", json!({ "id": id, "amount": "140000", "sender": SENDER })).await;
    assert_eq!(c, 200, "{q}");
    assert_eq!((q["depositAddress"].as_str(), q["amountOut"].as_str(), q["fees"]["withdrawFee"].as_str()), (Some(LIVE_DEPOSIT), Some("108000"), Some("32000")));

    let (_, st) = s.get(&format!("/api/status/{LIVE_DEPOSIT}")).await;
    assert_eq!(st["status"], "SUCCESS");
    assert_eq!(st["solanaTx"], "2LLVDBnVAqnG99PSbUVqapKqZFHjPPET9HsJhV2tqS29BHR4wNmUzYZbQzLf2BXjpFKMrRJCYxbVZq6ZJNQQ86au");
    assert_eq!(st["landed"], json!({ "txid": LIVE_ZTX, "height": 3_512_504 }));

    let (_, rb) = s.get(&format!("/api/readback/{id}")).await;
    assert_eq!(rb["state"], "LANDED");
    assert_eq!((rb["pool"].as_str(), rb["valueZat"].as_u64(), rb["quotedOutZat"].as_u64(), rb["deltaZat"].as_i64()), (Some("ironwood"), Some(115_000), Some(108_000), Some(7_000)));

    let (_, ex) = s.get(&format!("/api/exposure/{id}")).await;
    let rows = ex["rows"].as_array().unwrap();
    let find = |w: &str| rows.iter().find(|r| r["what"].as_str().unwrap().starts_with(w)).unwrap().clone();
    assert_eq!(find("Solana sender")["value"], SENDER);
    assert_eq!(find("Deposit address")["value"], LIVE_DEPOSIT);
    assert_eq!(find("Recipient u1")["value"], LIVE_U1);
    assert_eq!(find("Amount entering")["value"], 115_000);
    assert_eq!(find("Amount entering")["exposed"], true);
    assert_eq!(find("Later spends")["exposed"], false);
    assert_eq!(ex["addressReused"], false);
    let fields: Vec<_> = ex["publicOnStatus"].as_array().unwrap().iter().map(|x| x["field"].as_str().unwrap().to_string()).collect();
    for f in ["deposit_address", "refund_to", "recipient", "origin_tx", "destination_tx"] {
        assert!(fields.iter().any(|x| x == f), "{f} missing from {fields:?}");
    }
}

/// The real view-only wallet db from the live run, when present on this machine (never committed).
#[tokio::test]
async fn real_notes_db_holds_the_live_note() {
    let work = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("work");
    let db = work.join("notes-main.db");
    if !db.exists() {
        eprintln!("skipped: no work/notes-main.db");
        return;
    }
    let net = zcash_protocol::consensus::Network::MainNetwork;
    let ufvk = airlock::wallet::Wallet::open(&work).unwrap().ufvk(&net).unwrap();
    let nw = airlock::notewatch::NoteWatch::open(&db, net, &ufvk, 0).await.unwrap();
    let n = nw.landed().unwrap().into_iter().find(|n| n.txid == LIVE_ZTX).expect("live note in the db");
    assert_eq!((n.pool.as_str(), n.value_zat, n.mined_height), ("ironwood", 115_000, Some(3_512_504)));
}

#[tokio::test]
async fn rpc_proxy_allows_reads_only() {
    let s = Srv::start(Arc::new(MockOneClick::default()), Arc::new(MockZcash::default()), test_addrs()).await;
    for m in ["sendTransaction", "requestAirdrop", "getProgramAccounts"] {
        let (c, v) = s.post("/api/rpc", json!({ "jsonrpc": "2.0", "id": 1, "method": m, "params": [] })).await;
        assert_eq!((c, v["code"].as_str()), (400, Some("RPC_METHOD")), "{m}");
    }
    let (c, v) = s.post("/api/rpc", json!({ "jsonrpc": "2.0", "id": 1, "method": "getBalance", "params": [SENDER], "x": 1 })).await;
    assert_eq!((c, v["code"].as_str()), (400, Some("RPC_METHOD")));
}
