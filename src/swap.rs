//! Outbound swap: split, then one non-dry quote per part, each part with its own Tor
//! circuit, one-time deposit address (from the quote), fresh refund UA and destination.
use crate::{
    exposure::shared_identifiers,
    oneclick::{deadline_in, Client, QuoteRequest, StatusResponse, SwapType, ZEC_ASSET},
    splitter::{split, Split},
    transport::Transport,
    wallet::Wallet,
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

/// Creates the parts of one outbound swap. `destinations` must hold one distinct address per part.
pub async fn quote_outbound(
    tor: &tor::Client,
    wallet: &Wallet,
    dir: &Path,
    total_zat: u64,
    max_part: u64,
    destination_asset: &str,
    destinations: &[String],
) -> Result<Swap> {
    let Split { parts, residual } = split(total_zat, max_part, destinations.len())?;
    if parts.len() != destinations.len() {
        bail!("{} parts need {} distinct destinations, got {}", parts.len(), parts.len(), destinations.len());
    }
    if destinations.iter().collect::<HashSet<_>>().len() != destinations.len() {
        bail!("destinations repeat: a shared destination links the parts");
    }
    let mut out = Vec::new();
    for (amount, dest) in parts.iter().zip(destinations) {
        let refund_ua = wallet.fresh_ua(dir)?;
        let oc = Client::with_transport(Transport::isolated(tor));
        let q = oc
            .quote(&QuoteRequest {
                dry: false,
                deposit_mode: "SIMPLE",
                swap_type: SwapType::ExactInput,
                slippage_tolerance: 100,
                origin_asset: ZEC_ASSET.into(),
                deposit_type: "ORIGIN_CHAIN",
                destination_asset: destination_asset.into(),
                amount: amount.to_string(),
                refund_to: refund_ua.clone(),
                refund_type: "ORIGIN_CHAIN",
                recipient: dest.clone(),
                recipient_type: "DESTINATION_CHAIN",
                deadline: deadline_in(24 * 3600),
            })
            .await?;
        out.push(Part {
            amount_zat: *amount,
            deposit_address: q.quote.deposit_address.ok_or_else(|| anyhow::anyhow!("non-dry quote returned no depositAddress"))?,
            refund_ua,
            destination: dest.clone(),
            correlation_id: q.correlation_id,
            deadline: q.quote.deadline.unwrap_or_default(),
        });
    }
    let mut id = [0u8; 6];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut id);
    let swap = Swap { id: hex::encode(id), destination_asset: destination_asset.into(), parts: out, residual_zat: residual };
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
    let _ = writeln!(s, "not hidden: the sum is still recoverable by amount and timing correlation.");
    s
}
