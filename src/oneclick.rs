//! Typed client for the four 1Click endpoints AIRLOCK uses. Shapes follow
//! research/tech/oneclick-openapi.yaml (QuoteRequest, QuoteResponse, GetExecutionStatusResponse).
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use crate::transport::Transport;
use serde_json::Value;

pub const BASE: &str = "https://1click.chaindefuser.com";
pub const ZEC_ASSET: &str = "nep141:zec.omft.near";

#[derive(Debug, Clone, Deserialize)]
pub struct Token {
    #[serde(rename = "assetId")]
    pub asset_id: String,
    pub decimals: u32,
    pub blockchain: String,
    pub symbol: String,
    #[serde(default)]
    pub price: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SwapType {
    ExactInput,
    ExactOutput,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub dry: bool,
    pub deposit_mode: &'static str,
    pub swap_type: SwapType,
    pub slippage_tolerance: u32,
    pub origin_asset: String,
    pub deposit_type: &'static str,
    pub destination_asset: String,
    pub amount: String,
    pub refund_to: String,
    pub refund_type: &'static str,
    pub recipient: String,
    pub recipient_type: &'static str,
    pub deadline: String,
    /// Integrator fee (1Click `appFees`), omitted when empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub app_fees: Vec<AppFee>,
}

/// One 1Click integrator fee: `fee` in basis points of the input, paid to the NEAR account `recipient`.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AppFee {
    pub recipient: String,
    pub fee: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub amount_in: String,
    pub amount_in_formatted: String,
    pub min_amount_in: String,
    pub amount_out: String,
    pub amount_out_formatted: String,
    pub min_amount_out: String,
    pub time_estimate: Option<u64>,
    #[serde(default)]
    pub deposit_address: Option<String>,
    #[serde(default)]
    pub deadline: Option<String>,
    /// Fee 1Click reserves for the destination-chain delivery, in destination base units.
    #[serde(default)]
    pub withdraw_fee: Option<String>,
    #[serde(default)]
    pub refund_fee: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub quote: Quote,
    pub correlation_id: String,
    /// 1Click's copy of the request it recorded. A deposit address is only used when this copy
    /// equals what the caller asked for.
    #[serde(default)]
    pub quote_request: Value,
}

impl QuoteResponse {
    /// Refuses a quote whose recorded recipient, refund, amount or assets differ from the request.
    pub fn verify_echo(&self, q: &QuoteRequest) -> Result<()> {
        let want = [
            ("recipient", &q.recipient),
            ("refundTo", &q.refund_to),
            ("amount", &q.amount),
            ("originAsset", &q.origin_asset),
            ("destinationAsset", &q.destination_asset),
        ];
        for (k, v) in want {
            let got = self.quote_request.get(k).and_then(Value::as_str);
            if got != Some(v.as_str()) {
                anyhow::bail!("1Click recorded {k} {got:?}, not the {v:?} that was requested");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SwapStatus {
    KnownDepositTx,
    PendingDeposit,
    IncompleteDeposit,
    Processing,
    Success,
    Refunded,
    Failed,
}

impl SwapStatus {
    pub fn terminal(self) -> bool {
        matches!(self, Self::Success | Self::Refunded | Self::Failed)
    }
}

/// The raw status body is kept: the exposure analyzer reads every field the
/// unauthenticated endpoint returns, not only the ones typed here.
#[derive(Debug, Clone)]
pub struct StatusResponse {
    pub status: SwapStatus,
    pub raw: Value,
}

#[derive(Clone)]
pub struct Client {
    http: Transport,
    base: String,
}

impl Client {
    pub fn new() -> Result<Self> {
        Ok(Self::with_transport(Transport::direct()?))
    }

    /// Per-part isolation: pass a transport on its own Tor circuit.
    pub fn with_transport(http: Transport) -> Self {
        Self { http, base: BASE.into() }
    }

    pub async fn tokens(&self) -> Result<Vec<Token>> {
        decode(self.http.get(&format!("{}/v0/tokens", self.base)).await?)
    }

    pub async fn quote(&self, q: &QuoteRequest) -> Result<QuoteResponse> {
        decode(self.http.post_json(&format!("{}/v0/quote", self.base), &serde_json::to_value(q)?).await?)
    }

    pub async fn status(&self, deposit_address: &str) -> Result<StatusResponse> {
        let raw: Value = decode(self.http.get(&format!("{}/v0/status?depositAddress={deposit_address}", self.base)).await?)?;
        let status = serde_json::from_value(raw.get("status").cloned().context("status field missing")?)?;
        Ok(StatusResponse { status, raw })
    }

    pub async fn submit_deposit(&self, tx_hash: &str, deposit_address: &str) -> Result<Value> {
        let body = serde_json::json!({ "txHash": tx_hash, "depositAddress": deposit_address });
        decode(self.http.post_json(&format!("{}/v0/deposit/submit", self.base), &body).await?)
    }
}

fn decode<T: serde::de::DeserializeOwned>((code, body): (u16, String)) -> Result<T> {
    if !(200..300).contains(&code) {
        bail!("1click HTTP {code}: {body}");
    }
    serde_json::from_str(&body).with_context(|| format!("decode 1click body: {body}"))
}

/// Requested quote deadline. Public in /v0/status, so it must blend in: of 909 live ZEC swaps read
/// on 2026-10-03, 519 requested 120 minutes, 276 requested 60 and none requested 24 hours.
pub const REQUEST_DEADLINE_SECS: u64 = 2 * 3600;

/// Slippage tolerance in basis points, public in /v0/status. 659 of the 909 live swaps read on
/// 2026-10-03 used 200; matching the majority keeps our parts inside the crowd.
pub const SLIPPAGE_BPS: u32 = 200;

/// RFC3339 UTC `now + secs`, the only date arithmetic the client needs.
pub fn deadline_in(secs: u64) -> String {
    let t = std::time::SystemTime::now() + std::time::Duration::from_secs(secs);
    let d = t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let (days, rem) = (d / 86400, d % 86400);
    let (y, m, dd) = civil_from_days(days as i64);
    format!("{y:04}-{m:02}-{dd:02}T{:02}:{:02}:{:02}.000Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

/// Unix seconds from an RFC3339 UTC stamp like `2026-10-06T18:58:48.000Z`.
pub fn parse_rfc3339(s: &str) -> Option<u64> {
    let n = |r: std::ops::Range<usize>| s.get(r)?.parse::<i64>().ok();
    let (y, m, d) = (n(0..4)?, n(5..7)?, n(8..10)?);
    let (y, mp) = if m <= 2 { (y - 1, m + 9) } else { (y, m - 3) };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let doy = (153 * mp + 2) / 5 + d - 1;
    let days = era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719468;
    u64::try_from(days * 86400 + n(11..13)? * 3600 + n(14..16)? * 60 + n(17..19)?).ok()
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_deadline_matches_the_common_value() {
        assert_eq!(REQUEST_DEADLINE_SECS, 7200);
    }

    #[test]
    fn deadline_shape() {
        let d = deadline_in(0);
        assert_eq!(d.len(), 24);
        assert!(d.starts_with("20") && d.ends_with(".000Z"));
    }

    #[test]
    fn rfc3339_roundtrips_with_deadline_in() {
        assert_eq!(parse_rfc3339("2026-10-03T00:00:00.000Z"), Some(20729 * 86400));
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        assert_eq!(parse_rfc3339(&deadline_in(3600)), Some(now + 3600));
    }

    #[test]
    fn civil_known_date() {
        // 2026-10-03 is day 20729 since the epoch
        assert_eq!(civil_from_days(20729), (2026, 10, 3));
    }
}

#[cfg(test)]
mod echo_tests {
    use super::*;

    fn req() -> QuoteRequest {
        QuoteRequest {
            dry: false, deposit_mode: "SIMPLE", swap_type: SwapType::ExactInput, slippage_tolerance: 200,
            origin_asset: ZEC_ASSET.into(), deposit_type: "ORIGIN_CHAIN", destination_asset: "nep141:base".into(),
            amount: "1000000".into(), refund_to: "u1refund".into(), refund_type: "ORIGIN_CHAIN",
            recipient: "0xabc".into(), recipient_type: "DESTINATION_CHAIN", deadline: "2026-10-05T10:00:00.000Z".into(), app_fees: vec![],
        }
    }

    fn resp(echo: Value) -> QuoteResponse {
        serde_json::from_value(serde_json::json!({
            "quote": { "amountIn": "1", "amountInFormatted": "1", "minAmountIn": "1", "amountOut": "1", "amountOutFormatted": "1", "minAmountOut": "1", "timeEstimate": 1 },
            "correlationId": "c", "quoteRequest": echo,
        })).unwrap()
    }

    #[test]
    fn echo_must_equal_the_request() {
        let ok = serde_json::json!({ "recipient": "0xabc", "refundTo": "u1refund", "amount": "1000000", "originAsset": ZEC_ASSET, "destinationAsset": "nep141:base" });
        assert!(resp(ok.clone()).verify_echo(&req()).is_ok());
        for (k, v) in [("recipient", "0xevil"), ("refundTo", "t1evil"), ("amount", "9000000"), ("destinationAsset", "nep141:zec.omft.near")] {
            let mut bad = ok.clone();
            bad[k] = v.into();
            assert!(resp(bad).verify_echo(&req()).is_err(), "{k} swap must be refused");
        }
        assert!(resp(serde_json::json!({})).verify_echo(&req()).is_err(), "missing echo must be refused");
    }
}
