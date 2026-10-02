use airlock::swap::{quote_outbound, read_parts, receipt, shared_count, Swap};
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
            let [_, zec, cap, asset, dests] = &args[..] else { bail!("usage: outbound-quote <zec> <cap-zec> <destAssetId> <dest,dest,...>") };
            let tor = Transport::bootstrap_tor(&tor_dir()).await?;
            let w = Wallet::open(dir)?;
            let dests: Vec<String> = dests.split(',').map(String::from).collect();
            let swap = quote_outbound(&tor, &w, dir, zat(zec)?, zat(cap)?, asset, &dests).await?;
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
        _ => bail!("usage: airlock ua | dry [zatoshi] | circuits [n] | outbound-quote ... | audit <swap-id>"),
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
