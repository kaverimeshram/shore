use airlock::swap::{quote_inbound, quote_outbound, read_parts, receipt, shared_count, watch, Swap};
use airlock::watchdog::{Config, Outcome};
use airlock::transport::Transport;
use airlock::oneclick::{deadline_in, Client, QuoteRequest, SwapType, ZEC_ASSET};
use airlock::wallet::{receivers, Wallet};
use anyhow::{bail, Result};
use std::path::Path;

/// Inbound origin assets for the dry-quote survey: (label, assetId, refund address on that
/// chain). Dry quotes only validate the refund format, no funds ever move to these.
const ORIGINS: [(&str, &str, &str); 5] = [
    ("USDC sol", "nep141:sol-5ce3bf3a31af18be40ba30f721101b4341690186.omft.near", "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"),
    ("USDC base", "nep141:base-0x833589fcd6edb6e08f4c7c32d4f71b54bda02913.omft.near", "0x2527D02599Ba641c19FEa793cD0F167589a0f10D"),
    ("USDC arb", "nep141:arb-0xaf88d065e77c8cc2239327c5edb3a432268e5831.omft.near", "0x2527D02599Ba641c19FEa793cD0F167589a0f10D"),
    ("USDC eth", "nep141:eth-0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48.omft.near", "0x2527D02599Ba641c19FEa793cD0F167589a0f10D"),
    ("BTC", "nep141:btc.omft.near", "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq"),
];

fn tor_dir() -> std::path::PathBuf {
    Path::new("work").join("tor")
}

fn zat(zec: &str) -> Result<u64> {
    let (i, f) = zec.split_once('.').unwrap_or((zec, ""));
    if f.len() > 8 {
        bail!("{zec} has more than 8 decimals");
    }
    Ok(i.parse::<u64>()? * 100_000_000 + format!("{f:0<8}").parse::<u64>()?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let dir = Path::new("work");
    let args: Vec<String> = std::env::args().skip(1).collect();
    let oc = Client::new()?;
    match args.first().map(String::as_str) {
        Some("ua-test") => {
            // airlock ua-test <index>   testnet Orchard-only UA at an index (Revision 2)
            let i: u32 = args.get(1).map_or(Ok(0), |s| s.parse())?;
            println!("{}", Wallet::open(dir)?.orchard_ua_on(&zcash_protocol::consensus::Network::TestNetwork, i)?);
        }
        Some("ua") => {
            let ua = Wallet::open(dir)?.fresh_ua(dir)?;
            println!("{ua}\nreceivers: {:?}", receivers(&ua)?);
        }
        Some("dry") => {
            // airlock dry [zatoshi]   one EXACT_OUTPUT dry quote per origin asset into one Orchard-only UA
            let zat = args.get(1).cloned().unwrap_or_else(|| "1000000".into());
            let ua = Wallet::open(dir)?.fresh_ua(dir)?;
            println!("recipient {ua}\nreceivers {:?}", receivers(&ua)?);
            for (label, asset, refund) in ORIGINS {
                let q = QuoteRequest {
                    dry: true,
                    deposit_mode: "SIMPLE",
                    swap_type: SwapType::ExactOutput,
                    slippage_tolerance: 100,
                    origin_asset: asset.into(),
                    deposit_type: "ORIGIN_CHAIN",
                    destination_asset: ZEC_ASSET.into(),
                    amount: zat.clone(),
                    refund_to: refund.into(),
                    refund_type: "ORIGIN_CHAIN",
                    recipient: ua.clone(),
                    recipient_type: "DESTINATION_CHAIN",
                    deadline: deadline_in(7200),
                };
                match oc.quote(&q).await {
                    Ok(r) => println!(
                        "{label:10} in {} (min {}) -> out {} ZEC, est {}s",
                        r.quote.amount_in_formatted,
                        r.quote.min_amount_in,
                        r.quote.amount_out_formatted,
                        r.quote.time_estimate.unwrap_or(0)
                    ),
                    Err(e) => println!("{label:10} ERR {e}"),
                }
            }
        }
        Some("outbound-quote") => {
            // airlock outbound-quote <zec> <cap-zec> <destAssetId> <dest1,dest2,...>
            let (Some(zec), Some(cap), Some(asset), Some(dests)) = (args.get(1), args.get(2), args.get(3), args.get(4)) else { bail!("usage: outbound-quote <zec> <cap-zec> <destAssetId> <dest,dest,...> [min-secs max-secs]") };
            // quote times are public in /v0/status: space them out (default 5 to 45 minutes)
            let lo: u64 = args.get(5).map_or(Ok(300), |s| s.parse())?;
            let hi: u64 = args.get(6).map_or(Ok(2700), |s| s.parse())?;
            if lo > hi {
                bail!("min-secs must not exceed max-secs");
            }
            let tor = Transport::bootstrap_tor(&tor_dir()).await?;
            let w = Wallet::open(dir)?;
            let dests: Vec<String> = dests.split(',').map(String::from).collect();
            let swap = quote_outbound(&tor, || w.fresh_ua(dir), dir, zat(zec)?, zat(cap)?, asset, &dests, (lo, hi)).await?;
            println!("swap {}", swap.id);
            for p in &swap.parts {
                println!("  {} zat -> deposit {} deadline {}", p.amount_zat, p.deposit_address, p.deadline);
            }
        }
        Some("circuits") => {
            // airlock circuits [n]   exit IP seen by a server for n isolated handles, to show circuits differ
            let n: usize = args.get(1).map_or(Ok(3), |s| s.parse())?;
            let tor = Transport::bootstrap_tor(&tor_dir()).await?;
            for i in 0..n {
                let (_, body) = Transport::isolated(&tor).get("https://check.torproject.org/api/ip").await?;
                println!("handle {i}: {body}");
            }
        }
        Some("inbound") => {
            // airlock inbound <zec> <originAssetId> <originRefundAddress>   unfunded non-dry quote into a fresh Orchard-only UA
            let [_, zec, asset, refund] = &args[..] else { bail!("usage: inbound <zec> <originAssetId> <originRefundAddress>") };
            let tor = Transport::bootstrap_tor(&tor_dir()).await?;
            let w = Wallet::open(dir)?;
            let ib = quote_inbound(&tor, zat(zec)?, asset, refund, &w.fresh_ua(dir)?).await?;
            ib.save(dir)?;
            println!("inbound {}\n  send {} (min {}) of {}\n  to {}\n  lands in {}\n  deadline {}", ib.id, ib.amount_in, ib.min_amount_in, ib.origin_asset, ib.deposit_address, ib.recipient, ib.deadline);
        }
        Some("fund") => {
            // airlock fund <dest-unified-address> <zats> [max-fee]   testnet only: signed through COUNTERSIGN, mined read-back
            use airlock::notewatch::{tip, NoteWatch};
            use zcash_protocol::consensus::Network;
            let (Some(dest), Some(z)) = (args.get(1), args.get(2)) else { bail!("usage: fund <dest> <zats> [max-fee]") };
            let max_fee: u64 = args.get(3).map_or(Ok(20_000), |s| s.parse())?;
            let w = Wallet::open(dir)?;
            let net = Network::TestNetwork;
            let birthday = tip(net).await?.saturating_sub(10);
            let mut nw = NoteWatch::open(&dir.join("notes-test.db"), net, &w.ufvk(&net)?, birthday).await?;
            let (txid, h) = nw.fund(&w, dest, z.parse()?, max_fee, 900).await?;
            println!("MINED {txid} at height {h}");
        }
        Some("note") => {
            // airlock note <test|main> [min-zat] [timeout-secs]   wait until a note of at least min-zat lands in the wallet (view-only scan)
            use airlock::notewatch::{endpoint, tip, NoteWatch};
            use zcash_protocol::consensus::Network;
            let net = match args.get(1).map(String::as_str) {
                Some("main") => Network::MainNetwork,
                Some("test") => Network::TestNetwork,
                _ => bail!("usage: airlock note <test|main> [min-zat] [timeout-secs]"),
            };
            let min: u64 = args.get(2).map_or(Ok(1), |s| s.parse())?;
            let timeout: u64 = args.get(3).map_or(Ok(600), |s| s.parse())?;
            let w = Wallet::open(dir)?;
            let db = dir.join(format!("notes-{}.db", args[1]));
            let birthday = tip(net).await?.saturating_sub(10);
            let mut nw = NoteWatch::open(&db, net, &w.ufvk(&net)?, birthday).await?;
            println!("lightwalletd {}\nreceive at (Orchard-only UA, index 0):\n  revision 2: {}\n  revision 1: {}", endpoint(net), w.orchard_ua_on(&net, 0)?, airlock::wallet::to_revision_1(&w.orchard_ua_on(&net, 0)?)?);
            let start = std::time::Instant::now();
            loop {
                let scanned = nw.sync().await?;
                let hit: Vec<_> = nw.landed()?.into_iter().filter(|n| n.value_zat >= min && n.mined_height.is_some()).collect();
                println!("scanned to {scanned}, notes >= {min} zat: {}", hit.len());
                if let Some(n) = hit.first() {
                    println!("LANDED {n:?}");
                    break;
                }
                if start.elapsed().as_secs() > timeout {
                    bail!("no note of at least {min} zat within {timeout}s");
                }
                tokio::time::sleep(std::time::Duration::from_secs(20)).await;
            }
        }
        Some("serve") => {
            // airlock serve [addr]   HTTP service for the web app
            airlock::server::serve(args.get(1).map_or("127.0.0.1:8787", String::as_str), dir.to_path_buf()).await?;
        }
        Some("watch") => {
            // airlock watch <swap-id> [interval-secs] [hold-timeout-secs]
            let Some(id) = args.get(1) else { bail!("usage: airlock watch <swap-id> [interval-secs] [hold-timeout-secs]") };
            let interval: u64 = args.get(2).map_or(Ok(30), |s| s.parse())?;
            let hold_timeout: u64 = args.get(3).map_or(Ok(1800), |s| s.parse())?;
            let swap = Swap::load(dir, id)?;
            let tor = Transport::bootstrap_tor(&tor_dir()).await?;
            let outcomes = watch(&tor, &swap, Config { hold_timeout }, std::time::Duration::from_secs(interval), |a| println!("{a:?}")).await?;
            println!("outcomes {outcomes:?}");
            if outcomes.iter().any(|o| *o != Some(Outcome::Success)) {
                std::process::exit(1);
            }
        }
        Some("inspect") => {
            // airlock inspect <depositAddress>...   what /v0/status reveals for any funded one-time t-address
            if args.len() < 2 {
                bail!("usage: airlock inspect <depositAddress>...");
            }
            let mut raws = Vec::new();
            for d in &args[1..] {
                let s = oc.status(d).await?;
                println!("{d} {:?}", s.status);
                for (field, v) in airlock::exposure::identifiers(&s.raw) {
                    println!("  {field} = {v}");
                }
                raws.push(s.raw);
            }
            println!("shared identifiers across addresses: {}", airlock::exposure::shared_identifiers(&raws).len());
        }
        Some("audit") => {
            // airlock audit <swap-id>   live /v0/status reads over isolated circuits, receipt, shared count
            let Some(id) = args.get(1) else { bail!("usage: airlock audit <swap-id>") };
            let swap = Swap::load(dir, id)?;
            let tor = Transport::bootstrap_tor(&tor_dir()).await?;
            let reads = read_parts(&tor, &swap).await?;
            print!("{}", receipt(&swap, &reads));
            if shared_count(&reads) > 0 {
                std::process::exit(2);
            }
        }
        _ => bail!("usage: airlock ua | dry [zatoshi] | circuits [n] | outbound-quote ... | inspect <depositAddress>... | inbound <zec> <asset> <refund> | fund <dest> <zats> | note <test|main> [min-zat] [timeout] | serve [addr] | watch <swap-id> | audit <swap-id>"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn zat_parses_decimals() {
        assert_eq!(super::zat("1.3").unwrap(), 130_000_000);
        assert_eq!(super::zat("0.00000001").unwrap(), 1);
        assert_eq!(super::zat("2").unwrap(), 200_000_000);
        assert!(super::zat("0.123456789").is_err());
    }
}
