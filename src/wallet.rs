//! Fresh Orchard-only unified addresses from one local seed. The seed lives in the
//! gitignored work/ directory and is never printed or logged.
use anyhow::{bail, Context, Result};
use std::path::Path;
use zcash_keys::keys::{UnifiedAddressRequest, UnifiedSpendingKey};
use zcash_protocol::consensus::MainNetwork;
use zip32::{AccountId, DiversifierIndex};

pub fn random_bytes<const N: usize>() -> Result<[u8; N]> {
    use rand::TryRng;
    let mut b = [0u8; N];
    rand::rngs::SysRng.try_fill_bytes(&mut b).map_err(|e| anyhow::anyhow!("os rng: {e:?}"))?;
    Ok(b)
}

pub struct Wallet {
    seed: [u8; 32],
}

impl Wallet {
    /// Loads `<dir>/seed.hex`, creating a random one (mode 0600) on first use.
    pub fn open(dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(dir)?;
        let p = dir.join("seed.hex");
        if !p.exists() {
            std::fs::write(&p, hex::encode(random_bytes::<32>()?))?;
            #[cfg(unix)]
            std::fs::set_permissions(&p, std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
        }
        let bytes = hex::decode(std::fs::read_to_string(&p)?.trim()).context("seed.hex is not hex")?;
        let Ok(seed) = <[u8; 32]>::try_from(bytes) else { bail!("seed.hex must be 32 bytes") };
        Ok(Self { seed })
    }

    /// Next never-used index from `<dir>/ua-index`, persisted so no UA is ever handed out twice.
    pub fn fresh_ua(&self, dir: &Path) -> Result<String> {
        let p = dir.join("ua-index");
        let i: u32 = match std::fs::read_to_string(&p) {
            Ok(s) => s.trim().parse().context("ua-index is not a number")?,
            Err(_) => 0,
        };
        let ua = self.orchard_ua(i)?;
        std::fs::write(&p, (i + 1).to_string())?;
        Ok(ua)
    }

    /// Orchard-only mainnet UA at diversifier index `i`. Each index is a distinct address
    /// that shares no receiver with any other index.
    pub fn orchard_ua(&self, i: u32) -> Result<String> {
        to_revision_1(&self.orchard_ua_on(&MainNetwork, i)?)
    }

    /// The account's unified full viewing key, for a view-only watcher. Never the spending key.
    pub fn ufvk<P: zcash_protocol::consensus::Parameters>(&self, params: &P) -> Result<String> {
        let usk = UnifiedSpendingKey::from_seed(params, &self.seed, AccountId::ZERO)
            .map_err(|e| anyhow::anyhow!("usk: {e:?}"))?;
        Ok(usk.to_unified_full_viewing_key().encode(params))
    }

    /// Orchard-only UA on any network, in the crate's native encoding (Revision 2).
    pub fn orchard_ua_on<P: zcash_protocol::consensus::Parameters>(&self, params: &P, i: u32) -> Result<String> {
        let usk = UnifiedSpendingKey::from_seed(params, &self.seed, AccountId::ZERO)
            .map_err(|e| anyhow::anyhow!("usk: {e:?}"))?;
        let ufvk = usk.to_unified_full_viewing_key();
        let ua = ufvk
            .address(DiversifierIndex::from(i), UnifiedAddressRequest::ORCHARD)
            .map_err(|e| anyhow::anyhow!("ua at {i}: {e:?}"))?;
        Ok(ua.encode(params))
    }
}

/// Re-encodes a unified address as ZIP 316 Revision 1 (`u1...`). The NU7-generation crates emit
/// Revision 2 (`zu1...`), which the 1Click validator rejects ("recipient is not valid",
/// measured 2026-10-03); the receivers are identical, only the string differs.
pub fn to_revision_1(ua: &str) -> Result<String> {
    use zcash_address::unified::{Address, Container, Encoding, Revision};
    let (net, _, a) = Address::decode(ua).map_err(|e| anyhow::anyhow!("decode {e:?}"))?;
    let r1 = Address::try_from_items(Revision::R0, a.items_as_parsed().to_vec())
        .map_err(|e| anyhow::anyhow!("revision 1 re-encode: {e:?}"))?;
    Ok(r1.encode(&net))
}

/// Receiver kinds inside a mainnet UA string. Used to assert "Orchard-only" on what we
/// hand to 1Click.
pub fn receivers(ua: &str) -> Result<Vec<&'static str>> {
    use zcash_address::unified::{Address, Container, Encoding, Receiver};
    let (_, _, a) = Address::decode(ua).map_err(|e| anyhow::anyhow!("decode {e:?}"))?;
    Ok(a.items()
        .iter()
        .map(|r| match r {
            Receiver::Orchard(_) => "orchard",
            Receiver::Sapling(_) => "sapling",
            Receiver::P2pkh(_) => "p2pkh",
            Receiver::P2sh(_) => "p2sh",
            Receiver::Unknown { .. } => "unknown",
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexes_are_orchard_only_and_distinct() {
        let dir = std::env::temp_dir().join(format!("airlock-wallet-{}", std::process::id()));
        let w = Wallet::open(&dir).unwrap();
        let (a, b) = (w.orchard_ua(0).unwrap(), w.orchard_ua(1).unwrap());
        assert_ne!(a, b);
        assert!(a.starts_with("u1"), "1Click needs Revision 1: {a}");
        let rev2 = zcash_keys::keys::UnifiedSpendingKey::from_seed(&zcash_protocol::consensus::MainNetwork, &[7u8; 32], zip32::AccountId::ZERO)
            .unwrap()
            .to_unified_full_viewing_key()
            .address(zip32::DiversifierIndex::from(0u32), zcash_keys::keys::UnifiedAddressRequest::ORCHARD)
            .unwrap()
            .encode(&zcash_protocol::consensus::MainNetwork);
        assert!(rev2.starts_with("zu1"), "crate default is Revision 2: {rev2}");
        let r1 = to_revision_1(&rev2).unwrap();
        assert!(r1.starts_with("u1") && r1 != rev2);
        assert_eq!(receivers(&r1).unwrap(), receivers(&rev2).unwrap());
        assert_eq!(receivers(&a).unwrap(), vec!["orchard"]);
        assert_eq!(receivers(&b).unwrap(), vec!["orchard"]);
        std::fs::remove_dir_all(dir).ok();
    }
}
