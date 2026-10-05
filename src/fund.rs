//! Funds a deposit: proposes with the view-only wallet, has COUNTERSIGN's policy check and sign
//! the signer view, proves, extracts, broadcasts and polls to a mined height. Testnet only: the
//! operator rule keeps mainnet read-only unless AIRLOCK_MAINNET_FUND is set, and then every send is
//! held to the launch cap.
use crate::{notewatch::NoteWatch, wallet::Wallet};
use anyhow::{anyhow, bail, Result};
use pczt::roles::{prover::Prover, signer::Signer};
use rand::{rngs::SysRng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use zcash_client_backend::{
    data_api::{
        wallet::{
            create_pczt_from_proposal, extract_and_store_transaction_from_pczt, input_selection::{GreedyInputSelector, SpendPolicy},
            propose_transfer, redact_pczt_for_signer, ConfirmationsPolicy, SignerView,
        },
        WalletRead,
    },
    fees::zip317::SingleOutputChangeStrategy,
    proto::service::{RawTransaction, TxFilter},
    wallet::OvkPolicy,
};
use zcash_client_sqlite::util::SystemClock;
use zcash_primitives::transaction::{builder::BundlePadding, fees::zip317::FeeRule};
use zcash_protocol::{consensus::Network, value::Zatoshis, ShieldedPool};

fn os_rng() -> Result<ChaCha20Rng> {
    ChaCha20Rng::try_from_rng(&mut SysRng).map_err(|e| anyhow!("os rng: {e:?}"))
}

/// Mainnet sends need the operator's switch and stay under the part cap of the service (the same
/// constant, so the two cannot drift apart). Testnet is unrestricted.
fn check_launch(net: Network, zats: u64, mainnet_enabled: bool) -> Result<()> {
    if net == Network::MainNetwork {
        if !mainnet_enabled {
            bail!("mainnet funding is off: the operator sets AIRLOCK_MAINNET_FUND");
        }
        if zats > crate::server::MAX_PART_ZAT {
            bail!("launch cap: a mainnet send is at most {} zat", crate::server::MAX_PART_ZAT);
        }
    }
    Ok(())
}

impl NoteWatch {
    /// Sends `zats` to `dest` and returns (txid, mined height). `max_fee` is the most the signer
    /// will approve. The transaction keeps the library's default expiry.
    pub async fn fund(&mut self, wallet: &Wallet, dest: &str, zats: u64, max_fee: u64, timeout_s: u64) -> Result<(String, u64)> {
        check_launch(self.network, zats, std::env::var_os("AIRLOCK_MAINNET_FUND").is_some())?;
        self.sync().await?;
        let net = self.network;
        let id = *self.db.get_account_ids().map_err(|e| anyhow!("{e}"))?.first().ok_or_else(|| anyhow!("no account"))?;
        let recipient: zcash_address::ZcashAddress = dest.parse().map_err(|e| anyhow!("dest: {e}"))?;
        let amount = Zatoshis::from_u64(zats).map_err(|e| anyhow!("{e:?}"))?;
        let request = zip321::TransactionRequest::new(vec![zip321::Payment::without_memo(recipient, amount)]).map_err(|e| anyhow!("{e:?}"))?;
        let change = SingleOutputChangeStrategy::new(FeeRule::standard(), None, ShieldedPool::Orchard, Default::default());
        let proposal = propose_transfer::<_, _, _, _, std::convert::Infallible>(
            &mut self.db, &net, id, &GreedyInputSelector::new(), &change, request,
            ConfirmationsPolicy::MIN, &SpendPolicy::default(), None, None,
        )
        .map_err(|e| anyhow!("propose_transfer: {e}"))?;
        let full = create_pczt_from_proposal::<_, _, std::convert::Infallible, _, zcash_primitives::transaction::fees::zip317::FeeError, _>(
            &mut self.db, &net, &SystemClock, &mut os_rng()?, id, OvkPolicy::Sender, &proposal, None, BundlePadding::DEFAULT,
        )
        .map_err(|e| anyhow!("create_pczt_from_proposal: {e}"))?;
        let view = redact_pczt_for_signer(&full, SignerView::Compact).serialize().map_err(|e| anyhow!("serialize view: {e:?}"))?;

        // A transparent destination (a 1Click deposit address) takes COUNTERSIGN's opt-in transparent
        // intent: one P2PKH output of exactly this amount, no memo, change back to the signer.
        let intent = if dest.starts_with('t') {
            countersign_core::Intent::to_transparent_once(dest, zats, max_fee)
        } else {
            countersign_core::Intent::from_unified_address(dest, zats, None, max_fee)
        }
        .map_err(|e| anyhow!(e))?;
        let sigs = wallet.approve_and_sign(&view, &intent, if net == Network::MainNetwork { countersign_core::Network::Main } else { countersign_core::Network::Test })?;

        let mut signer = Signer::new(full).map_err(|e| anyhow!("Signer::new: {e:?}"))?;
        for s in sigs {
            let s: pczt::roles::signer::SpendAuthSignature = s.into();
            signer.apply_orchard_spend_auth_signature(&s).map_err(|e| anyhow!("apply signature: {e:?}"))?;
        }
        let mut prover = Prover::new(signer.finish());
        let pk = orchard::circuit::ProvingKey::build(orchard::circuit::OrchardCircuitVersion::PostNu6_3);
        let mut rng = os_rng()?;
        if prover.requires_ironwood_proof() {
            prover = prover.create_ironwood_proof(&mut rng, &pk).map_err(|e| anyhow!("ironwood proof: {e:?}"))?;
        }
        if prover.requires_orchard_proof() {
            prover = prover.create_orchard_proof(&mut rng, &pk).map_err(|e| anyhow!("orchard proof: {e:?}"))?;
        }
        if prover.requires_sapling_proofs() {
            bail!("sapling proofs required");
        }
        let txid = extract_and_store_transaction_from_pczt::<_, zcash_client_sqlite::ReceivedNoteId>(&mut self.db, &SystemClock, &mut rng, prover.finish(), None, None)
            .map_err(|e| anyhow!("extract: {e}"))?;
        let raw = self.db.get_transaction(txid).map_err(|e| anyhow!("{e}"))?.ok_or_else(|| anyhow!("tx not stored"))?;
        let mut bytes = Vec::new();
        raw.write(&mut bytes)?;
        let mut c = crate::notewatch::client(self.network).await?;
        let r = c.send_transaction(RawTransaction { data: bytes, height: 0 }).await?.into_inner();
        if r.error_code != 0 {
            bail!("lightwalletd rejected: {} {}", r.error_code, r.error_message);
        }
        let txid_hex = txid.to_string();
        let mut wire = hex::decode(&txid_hex)?;
        wire.reverse();
        let start = std::time::Instant::now();
        loop {
            if let Ok(r) = c.get_transaction(TxFilter { block: None, index: 0, hash: wire.clone() }).await {
                let tx = r.into_inner();
                if tx.height > 0 && tx.height != u64::MAX {
                    return Ok((txid_hex, tx.height));
                }
            }
            if start.elapsed().as_secs() > timeout_s {
                bail!("{txid_hex} broadcast but not mined within {timeout_s}s");
            }
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mainnet_needs_the_switch_and_the_cap() {
        let m = Network::MainNetwork;
        assert!(check_launch(m, 1_000_000, false).is_err());
        assert!(check_launch(m, 1_000_001, true).is_err());
        assert!(check_launch(m, 1_000_000, true).is_ok());
        assert!(check_launch(Network::TestNetwork, 50_000_000, false).is_ok());
    }
}
