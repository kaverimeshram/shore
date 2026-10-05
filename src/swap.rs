//! Outbound swap: split, then one non-dry quote per part, each part with its own Tor
//! circuit, one-time deposit address (from the quote), fresh refund UA and destination.
use crate::{
    watchdog::{Action, Config, Outcome, Tracker},
    exposure::shared_identifiers,
    oneclick::{deadline_in, parse_rfc3339, Client, QuoteRequest, StatusResponse, SwapType, ZEC_ASSET},
    splitter::{split, Split},
    transport::Transport,
};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::Path};
use zcash_client_backend::tor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    pub amount_zat: u64,
    pub deposit_address: String,
    pub refund_ua: String,
    pub destination: String,
    pub correlation_id: String,
    pub deadline: String,
    /// What 1Click expects to deliver, in the destination asset (informational; the minimum is lower by the slippage).
    #[serde(default)]
    pub amount_out_formatted: String,
    /// The least 1Click will deliver, in base units of the destination asset.
    #[serde(default)]
    pub min_amount_out: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Swap {
    pub id: String,
    pub destination_asset: String,
    pub parts: Vec<Part>,
    pub residual_zat: u64,
}

impl Swap {
    pub fn path(dir: &Path, id: &str) -> std::path::PathBuf {
        dir.join("swaps").join(format!("{id}.json"))
    }

    pub fn save(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir.join("swaps"))?;
        std::fs::write(Self::path(dir, &self.id), serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }

    pub fn load(dir: &Path, id: &str) -> Result<Self> {
        Ok(serde_json::from_slice(&std::fs::read(Self::path(dir, id))?)?)
    }
}

/// One part: a non-dry quote over its own isolated Tor circuit. Stateless, nothing is written.
pub async fn quote_part(tor: &tor::Client, amount_zat: u64, destination_asset: &str, destination: &str, refund_ua: &str) -> Result<Part> {
    if amount_zat > crate::server::MAX_PART_ZAT {
        bail!("launch cap: a part is at most {} zat", crate::server::MAX_PART_ZAT);
    }
    if destination_asset == ZEC_ASSET {
        bail!("the destination asset must not be ZEC");
    }
    let req = QuoteRequest {
            dry: false,
            deposit_mode: "SIMPLE",
            swap_type: SwapType::ExactInput,
            slippage_tolerance: crate::oneclick::SLIPPAGE_BPS,
            origin_asset: ZEC_ASSET.into(),
            deposit_type: "ORIGIN_CHAIN",
            destination_asset: destination_asset.into(),
            amount: amount_zat.to_string(),
            refund_to: refund_ua.into(),
            refund_type: "ORIGIN_CHAIN",
            recipient: destination.into(),
            recipient_type: "DESTINATION_CHAIN",
            deadline: deadline_in(crate::oneclick::REQUEST_DEADLINE_SECS),
    };
    let q = Client::with_transport(Transport::isolated(tor)).quote(&req).await?;
    q.verify_echo(&req)?;
    Ok(Part {
        amount_zat,
        amount_out_formatted: q.quote.amount_out_formatted.clone(),
        min_amount_out: q.quote.min_amount_out.clone(),
        deposit_address: q.quote.deposit_address.ok_or_else(|| anyhow::anyhow!("non-dry quote returned no depositAddress"))?,
        refund_ua: refund_ua.into(),
        destination: destination.into(),
        correlation_id: q.correlation_id,
        deadline: q.quote.deadline.unwrap_or_default(),
    })
}

/// Creates the parts of one outbound swap, `jitter_secs` (min, max) apart. `destinations` must hold one distinct address per part.
pub async fn quote_outbound(
    tor: &tor::Client,
    mut next_refund: impl FnMut() -> Result<String>,
    dir: &Path,
    total_zat: u64,
    max_part: u64,
    destination_asset: &str,
    destinations: &[String],
    jitter_secs: (u64, u64),
) -> Result<Swap> {
    let Split { parts, residual } = split(total_zat, max_part, destinations.len())?;
    if parts.len() != destinations.len() {
        bail!("{} parts need {} distinct destinations, got {}", parts.len(), parts.len(), destinations.len());
    }
    if destinations.iter().collect::<HashSet<_>>().len() != destinations.len() {
        bail!("destinations repeat: a shared destination links the parts");
    }
    let mut out = Vec::new();
    for (i, (amount, dest)) in parts.iter().zip(destinations).enumerate() {
        if i > 0 {
            // quotes created back to back would carry a timing pattern; space them out
            let secs = jitter_secs.0 + u64::from(u32::from_le_bytes(crate::wallet::random_bytes::<4>()?)) % (jitter_secs.1 - jitter_secs.0 + 1);
            tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
        }
        out.push(quote_part(tor, *amount, destination_asset, dest, &next_refund()?).await?);
    }
    let swap = Swap { id: hex::encode(crate::wallet::random_bytes::<6>()?), destination_asset: destination_asset.into(), parts: out, residual_zat: residual };
    swap.save(dir)?;
    Ok(swap)
}

/// Live status reads, each over its own isolated circuit, plus the shared identifier count.
pub async fn read_parts(tor: &tor::Client, swap: &Swap) -> Result<Vec<StatusResponse>> {
    let mut out = Vec::new();
    for p in &swap.parts {
        out.push(Client::with_transport(Transport::isolated(tor)).status(&p.deposit_address).await?);
    }
    Ok(out)
}

pub fn shared_count(reads: &[StatusResponse]) -> usize {
    shared_identifiers(&reads.iter().map(|r| r.raw.clone()).collect::<Vec<_>>()).len()
}

/// Exposure receipt: what each observer can link, computed from the live status reads.
pub fn receipt(swap: &Swap, reads: &[StatusResponse]) -> String {
    use crate::exposure::{identifiers, shared_identifiers};
    use std::fmt::Write;
    let mut s = String::new();
    let _ = writeln!(s, "AIRLOCK exposure receipt, swap {}", swap.id);
    let _ = writeln!(s, "parts {} residual {} zat (stays shielded, never crosses)", swap.parts.len(), swap.residual_zat);
    for (i, (p, r)) in swap.parts.iter().zip(reads).enumerate() {
        let _ = writeln!(s, "\npart {i}: {} zat, status {:?}", p.amount_zat, r.status);
        let _ = writeln!(s, "  chain observer (Zcash): one-time t-address {} and the amount. Not the refund UA, not the destination.", p.deposit_address);
        let _ = writeln!(s, "  /v0/status, no key, anyone holding the t-address, returns:");
        for (field, v) in identifiers(&r.raw) {
            let _ = writeln!(s, "    {field} = {v}");
        }
        let _ = writeln!(s, "  solver / 1Click: the full quote request above; network source is a per-part Tor circuit.");
    }
    let shared = shared_identifiers(&reads.iter().map(|r| r.raw.clone()).collect::<Vec<_>>());
    let _ = writeln!(s, "\nshared identifiers across parts: {}", shared.len());
    for x in shared {
        let _ = writeln!(s, "  SHARED {} = {} in parts {:?}", x.field, x.value, x.parts);
    }
    let raws: Vec<_> = reads.iter().map(|r| r.raw.clone()).collect();
    let _ = writeln!(s, "\nunfiltered: leaf values identical across parts (asset ids and route constants repeat by design)");
    for (path, v, parts) in crate::exposure::shared_leaves(&raws) {
        let _ = writeln!(s, "  {path} = {v} in parts {parts:?}");
    }
    if let Ok(reference) = crate::profile::Reference::load(Path::new("data/quote-profiles.json")) {
        let _ = writeln!(s, "\nquote profile (public in /v0/status, an observer can group swaps by it):");
        for (i, r) in reads.iter().enumerate() {
            if let Some(p) = crate::profile::Profile::from_status(&r.raw) {
                let _ = writeln!(s, "  part {i}: referral {:?}, fees {:?}, slippage {} bps, deadline {} min, refund {}: shared by {} of {} sampled swaps", p.referral, p.fees, p.slippage_bps, p.deadline_min, p.refund, reference.matches(&p), reference.sample_size());
            }
        }
    }
    let _ = writeln!(s, "not hidden: the sum is still recoverable by amount and timing correlation.");
    s
}

/// Polls every part, each read on its own isolated circuit, until every part is terminal.
/// Returns the per-part outcomes. `on_event` receives each watchdog action as it happens.
pub async fn watch(
    tor: &tor::Client,
    swap: &Swap,
    cfg: Config,
    interval: std::time::Duration,
    mut on_event: impl FnMut(&Action),
) -> Result<Vec<Option<Outcome>>> {
    let deadlines: Vec<(u64, u64)> = swap
        .parts
        .iter()
        .map(|p| Ok((p.amount_zat, parse_rfc3339(&p.deadline).ok_or_else(|| anyhow::anyhow!("bad deadline {}", p.deadline))?)))
        .collect::<Result<_>>()?;
    let mut tracker = Tracker::new(cfg, &deadlines);
    while !tracker.all_done() {
        for (i, p) in swap.parts.iter().enumerate() {
            if tracker.outcomes()[i].is_some() {
                continue;
            }
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs();
            match Client::with_transport(Transport::isolated(tor)).status(&p.deposit_address).await {
                Ok(r) => tracker.observe(i, r.status, now).iter().for_each(&mut on_event),
                Err(e) => eprintln!("part {i}: status read failed, will retry: {e}"),
            }
        }
        if !tracker.all_done() {
            tokio::time::sleep(interval).await;
        }
    }
    Ok(tracker.outcomes())
}

/// The receipt as data, for the web service. Same facts as [`receipt`].
pub fn receipt_json(swap: &Swap, reads: &[StatusResponse]) -> serde_json::Value {
    use crate::exposure::{identifiers, shared_identifiers, shared_leaves};
    let raws: Vec<_> = reads.iter().map(|r| r.raw.clone()).collect();
    let parts: Vec<_> = swap
        .parts
        .iter()
        .zip(reads)
        .enumerate()
        .map(|(i, (p, r))| {
            serde_json::json!({
                "index": i,
                "amountZat": p.amount_zat,
                "status": r.raw["status"],
                "depositAddress": p.deposit_address,
                "deadline": p.deadline,
                "publicOnStatus": identifiers(&r.raw).into_iter().map(|(f, v)| serde_json::json!({"field": f, "value": v})).collect::<Vec<_>>(),
            })
        })
        .collect();
    serde_json::json!({
        "swapId": swap.id,
        "residualZat": swap.residual_zat,
        "parts": parts,
        "sharedIdentifiers": shared_identifiers(&raws).into_iter().map(|x| serde_json::json!({"field": x.field, "value": x.value, "parts": x.parts})).collect::<Vec<_>>(),
        "sharedConstants": shared_leaves(&raws).into_iter().map(|(p, v, i)| serde_json::json!({"path": p, "value": v, "parts": i})).collect::<Vec<_>>(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inbound {
    pub id: String,
    pub origin_asset: String,
    pub deposit_address: String,
    pub recipient: String,
    pub amount_out_zat: u64,
    pub amount_in: String,
    pub min_amount_in: String,
    pub deadline: String,
}

/// Inbound swap: pay in `origin_asset`, land as shielded ZEC in `recipient` (Orchard-only UA).
/// EXACT_OUTPUT so the note that lands is the amount asked for. No unshield step exists.
pub async fn quote_inbound(
    tor: &tor::Client,
    amount_out_zat: u64,
    origin_asset: &str,
    origin_refund: &str,
    recipient: &str,
) -> Result<Inbound> {
    if amount_out_zat > crate::server::MAX_INBOUND_ZAT {
        bail!("launch cap: an inbound swap is at most {} zat", crate::server::MAX_INBOUND_ZAT);
    }
    let recipient = crate::wallet::to_revision_1(recipient)?;
    if crate::wallet::receivers(&recipient)? != ["orchard"] {
        bail!("recipient must be an Orchard-only unified address");
    }
    let req = QuoteRequest {
            dry: false,
            deposit_mode: "SIMPLE",
            swap_type: SwapType::ExactOutput,
            slippage_tolerance: crate::oneclick::SLIPPAGE_BPS,
            origin_asset: origin_asset.into(),
            deposit_type: "ORIGIN_CHAIN",
            destination_asset: ZEC_ASSET.into(),
            amount: amount_out_zat.to_string(),
            refund_to: origin_refund.into(),
            refund_type: "ORIGIN_CHAIN",
            recipient: recipient.clone(),
            recipient_type: "DESTINATION_CHAIN",
            deadline: deadline_in(crate::oneclick::REQUEST_DEADLINE_SECS),
    };
    let q = Client::with_transport(Transport::isolated(tor)).quote(&req).await?;
    q.verify_echo(&req)?;
    let inbound = Inbound {
        id: hex::encode(crate::wallet::random_bytes::<6>()?),
        origin_asset: origin_asset.into(),
        deposit_address: q.quote.deposit_address.ok_or_else(|| anyhow::anyhow!("non-dry quote returned no depositAddress"))?,
        recipient,
        amount_out_zat,
        amount_in: q.quote.amount_in_formatted,
        min_amount_in: q.quote.min_amount_in,
        deadline: q.quote.deadline.unwrap_or_default(),
    };
    Ok(inbound)
}

impl Inbound {
    pub fn save(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir.join("inbound"))?;
        std::fs::write(dir.join("inbound").join(format!("{}.json", self.id)), serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }
}
