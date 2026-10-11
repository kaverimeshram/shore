//! Prints a throwaway mainnet wallet's Orchard u1 at index N and its UFVK (random seed, never stored), for live
//! tests of user-owned addresses. Nothing is ever sent to it. `cargo run --example foreign_wallet -- <index>`
use airlock::shore::{AddressSource, UfvkAddresses};
fn main() {
    let net = zcash_protocol::consensus::MainNetwork;
    let seed: [u8; 32] = airlock::wallet::random_bytes::<32>().unwrap();
    let k = zcash_keys::keys::UnifiedSpendingKey::from_seed(&net, &seed, zip32::AccountId::ZERO).unwrap().to_unified_full_viewing_key().encode(&net);
    let i: u32 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(0);
    println!("{}\n{}", UfvkAddresses::new(&k).unwrap().at(i).unwrap(), k);
}
