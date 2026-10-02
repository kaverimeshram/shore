//! Exposure analyzer: which identifiers an observer of the unauthenticated `/v0/status`
//! endpoint (or the chains) can use to link the parts of one swap together.
use serde_json::Value;
use std::collections::BTreeMap;

/// Identifier classes read from one status payload. Asset ids are not identifiers: every
/// swap on a route shares them. The 1Click appFees recipient is identical for every swap
/// of every user, so it links nothing and is skipped.
pub fn identifiers(status: &Value) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    let qr = &status["quoteResponse"]["quoteRequest"];
    let mut put = |field: &'static str, v: &Value| {
        if let Some(s) = v.as_str().filter(|s| !s.is_empty()) {
            out.push((field, s.to_string()));
        }
    };
    put("deposit_address", &status["quoteResponse"]["quote"]["depositAddress"]);
    put("refund_to", &qr["refundTo"]);
    put("recipient", &qr["recipient"]);
    put("deposit_memo", &status["quoteResponse"]["quote"]["depositMemo"]);
    for (field, path) in [
        ("origin_tx", "originChainTxHashes"),
        ("destination_tx", "destinationChainTxHashes"),
        ("near_tx", "nearTxHashes"),
        ("intent_hash", "intentHashes"),
    ] {
        for h in status["swapDetails"][path].as_array().into_iter().flatten() {
            // entries are bare strings or {hash, explorerUrl}
            put(field, if h.is_object() { &h["hash"] } else { h });
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shared {
    pub field: &'static str,
    pub value: String,
    pub parts: Vec<usize>,
}

/// Identifier values carried by two or more parts. Count of the returned list is the
/// "shared identifiers" number the receipt reports (target 0).
pub fn shared_identifiers(parts: &[Value]) -> Vec<Shared> {
    let mut seen: BTreeMap<(&'static str, String), Vec<usize>> = BTreeMap::new();
    for (i, p) in parts.iter().enumerate() {
        for id in identifiers(p) {
            let e = seen.entry(id).or_default();
            if !e.contains(&i) {
                e.push(i);
            }
        }
    }
    seen.into_iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|((field, value), parts)| Shared { field, value, parts })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Value {
        serde_json::from_str(include_str!("../tests/fixtures/status_pending.json")).unwrap()
    }

    fn part(deposit: &str, refund: &str, recipient: &str) -> Value {
        let mut v = base();
        v["quoteResponse"]["quote"]["depositAddress"] = deposit.into();
        v["quoteResponse"]["quoteRequest"]["refundTo"] = refund.into();
        v["quoteResponse"]["quoteRequest"]["recipient"] = recipient.into();
        v
    }

    #[test]
    fn extracts_the_leaked_fields() {
        let ids = identifiers(&base());
        assert!(ids.contains(&("refund_to", "t1Hsc1LR8yKnbbe3twRp88p6vFfC5t7DLbs".into())));
        assert!(ids.contains(&("recipient", "0x2527D02599Ba641c19FEa793cD0F167589a0f10D".into())));
        assert!(ids.contains(&("deposit_address", "t1UH6mFsvCCzTYvKgTgNCbNBW9t8Vd1NCoq".into())));
    }

    /// B1 green: fresh deposit, refund and destination per part share nothing.
    #[test]
    fn analyzer_b1_fresh_identifiers_count_zero() {
        let parts = [part("d1", "r1", "x1"), part("d2", "r2", "x2"), part("d3", "r3", "x3")];
        assert_eq!(shared_identifiers(&parts), vec![]);
    }

    /// B1 red: two parts reuse one refund UA, the analyzer must flag it.
    #[test]
    fn analyzer_b1_reused_refund_ua_flagged() {
        let parts = [part("d1", "uREUSED", "x1"), part("d2", "uREUSED", "x2"), part("d3", "r3", "x3")];
        let s = shared_identifiers(&parts);
        assert_eq!(s.len(), 1);
        assert_eq!((s[0].field, s[0].value.as_str(), s[0].parts.clone()), ("refund_to", "uREUSED", vec![0, 1]));
    }

    #[test]
    fn tx_hash_objects_and_strings_both_read() {
        let mut a = part("d1", "r1", "x1");
        let mut b = part("d2", "r2", "x2");
        a["swapDetails"]["destinationChainTxHashes"] = serde_json::json!([{"hash": "0xabc", "explorerUrl": "u"}]);
        b["swapDetails"]["destinationChainTxHashes"] = serde_json::json!(["0xabc"]);
        let s = shared_identifiers(&[a, b]);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].field, "destination_tx");
    }
}
