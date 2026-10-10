//! /api/rpc hardening: per-request cache, retry with backoff on 429/5xx, public fallback, read-only allowlist.
use airlock::shore::{rpc_ttl, RpcProxy};
use axum::{http::StatusCode, routing::post, Json, Router};
use serde_json::{json, Value};
use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};
use std::time::{Duration, Instant};

/// A fake RPC that answers with `script[n]` on the n-th hit (last entry repeats) and counts hits.
async fn fake(script: Vec<u16>, tag: &'static str) -> (String, Arc<AtomicUsize>) {
    let hits = Arc::new(AtomicUsize::new(0));
    let h = hits.clone();
    let app = Router::new().route("/", post(move |Json(v): Json<Value>| {
        let h = h.clone(); let script = script.clone();
        async move {
            let n = h.fetch_add(1, Ordering::SeqCst);
            let code = *script.get(n).unwrap_or(script.last().unwrap());
            if code == 200 { (StatusCode::OK, Json(json!({ "jsonrpc": "2.0", "id": v["id"], "result": { "from": tag, "hit": n } }))) }
            else { (StatusCode::from_u16(code).unwrap(), Json(json!({ "error": "x" }))) }
        }
    }));
    let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", l.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(l, app).await.unwrap() });
    (url, hits)
}
fn req(m: &str, id: u64) -> Value { json!({ "jsonrpc": "2.0", "id": id, "method": m, "params": ["ia3MLukFMa6zci4nyahxsKVT14yLkFzUkZs3EL5Vsa5"] }) }
const FAST: [Duration; 2] = [Duration::from_millis(20), Duration::from_millis(60)];

#[tokio::test]
async fn balance_is_cached_per_owner_and_keeps_the_callers_id() {
    let (p, hits) = fake(vec![200], "primary").await;
    let rpc = RpcProxy::new(Some(p), None, FAST.to_vec());
    let a = rpc.call(&req("getBalance", 1)).await.unwrap();
    let b = rpc.call(&req("getBalance", 2)).await.unwrap();
    assert_eq!(hits.load(Ordering::SeqCst), 1, "second read within the TTL is served from cache");
    assert_eq!((a["id"].clone(), b["id"].clone()), (json!(1), json!(2)));
    let mut other = req("getBalance", 3); other["params"] = json!(["Other1111111111111111111111111111111111111"]);
    rpc.call(&other).await.unwrap();
    assert_eq!(hits.load(Ordering::SeqCst), 2, "a different owner is a different cache entry");
    rpc.call(&req("getSignatureStatuses", 4)).await.unwrap();
    rpc.call(&req("getSignatureStatuses", 5)).await.unwrap();
    assert_eq!(hits.load(Ordering::SeqCst), 4, "signature statuses are never cached");
    assert_eq!(rpc_ttl("getBalance"), Some(Duration::from_secs(8)));
    assert_eq!(rpc_ttl("getTokenAccountsByOwner"), Some(Duration::from_secs(8)));
    assert_eq!(rpc_ttl("getLatestBlockhash"), Some(Duration::from_secs(2)));
}

#[tokio::test]
async fn retries_429_and_5xx_with_backoff_then_succeeds() {
    let (p, hits) = fake(vec![429, 503, 200], "primary").await;
    let rpc = RpcProxy::new(Some(p), None, FAST.to_vec());
    let t = Instant::now();
    let r = rpc.call(&req("getLatestBlockhash", 1)).await.unwrap();
    assert_eq!(r["result"]["from"], "primary");
    assert_eq!(hits.load(Ordering::SeqCst), 3);
    assert!(t.elapsed() >= Duration::from_millis(80), "both backoff steps were waited: {:?}", t.elapsed());
}

#[tokio::test]
async fn falls_back_to_public_after_retries_and_hides_the_provider_url() {
    let (p, hits) = fake(vec![429], "primary").await;
    let (f, fh) = fake(vec![200], "public").await;
    let rpc = RpcProxy::new(Some(p.clone()), Some(f), FAST.to_vec());
    let r = rpc.call(&req("getTokenAccountsByOwner", 7)).await.unwrap();
    assert_eq!((r["result"]["from"].as_str(), r["id"].clone()), (Some("public"), json!(7)));
    assert_eq!((hits.load(Ordering::SeqCst), fh.load(Ordering::SeqCst)), (3, 1), "3 primary tries, then 1 fallback");

    let (p2, _) = fake(vec![500], "primary").await;
    let (f2, _) = fake(vec![502], "public").await;
    let e = RpcProxy::new(Some(p2.clone()), Some(f2), FAST.to_vec()).call(&req("getBalance", 1)).await.unwrap_err();
    assert!(!e.2.contains("127.0.0.1") && !e.2.contains(&p2), "error must not carry a provider URL: {}", e.2);
    assert!(e.2.contains("HTTP 500") && e.2.contains("fallback HTTP 502"), "{}", e.2);
}

#[tokio::test]
async fn a_client_error_is_not_retried_and_writes_never_reach_a_provider() {
    let (p, hits) = fake(vec![400], "primary").await;
    let rpc = RpcProxy::new(Some(p), None, FAST.to_vec());
    assert!(rpc.call(&req("getBalance", 1)).await.is_err());
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    for m in ["sendTransaction", "requestAirdrop", "getProgramAccounts"] {
        assert_eq!(rpc.call(&req(m, 1)).await.unwrap_err().1, "RPC_METHOD");
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}
