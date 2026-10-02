//! Fresh Orchard-only unified addresses from one local seed. The seed lives in the
//! gitignored work/ directory and is never printed or logged.
use anyhow::{bail, Context, Result};
use std::path::Path;
use zcash_keys::keys::{UnifiedAddressRequest, UnifiedSpendingKey};
use zcash_protocol::consensus::MainNetwork;
use zip32::{AccountId, DiversifierIndex};

pub struct Wallet {
    seed: [u8; 32],
}

impl Wallet {
    /// Loads `<dir>/seed.hex`, creating a random one (mode 0600) on first use.
    pub fn open(dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(dir)?;
        let p = dir.join("seed.hex");
        if !p.exists() {
            let mut s = [0u8; 32];
            rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut s);
            std::fs::write(&p, hex::encode(s))?;
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
        let usk = UnifiedSpendingKey::from_seed(&MainNetwork, &self.seed, AccountId::ZERO)
            .map_err(|e| anyhow::anyhow!("usk: {e:?}"))?;
        let ufvk = usk.to_unified_full_viewing_key();
        let ua = ufvk
            .address(DiversifierIndex::from(i), UnifiedAddressRequest::ORCHARD)
            .map_err(|e| anyhow::anyhow!("ua at {i}: {e:?}"))?;
        Ok(ua.encode(&MainNetwork))
    }
}

/// Receiver kinds inside a mainnet UA string. Used to assert "Orchard-only" on what we
/// hand to 1Click.
pub fn receivers(ua: &str) -> Result<Vec<&'static str>> {
    use zcash_address::unified::{Address, Container, Encoding, Receiver};
    let (_, a) = Address::decode(ua).map_err(|e| anyhow::anyhow!("decode {e:?}"))?;
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
        assert!(a.starts_with("u1"));
        assert_eq!(receivers(&a).unwrap(), vec!["orchard"]);
        assert_eq!(receivers(&b).unwrap(), vec!["orchard"]);
        std::fs::remove_dir_all(dir).ok();
    }
}
