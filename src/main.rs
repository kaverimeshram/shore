use airlock::exposure::shared_identifiers;
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

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let oc = Client::new()?;
    match args.first().map(String::as_str) {
        Some("ua") => {
            let w = Wallet::open(Path::new("work"))?;
            let i: u32 = args.get(1).map_or(Ok(0), |s| s.parse())?;
            let ua = w.orchard_ua(i)?;
            println!("{ua}\nreceivers: {:?}", receivers(&ua)?);
        }
        Some("dry") => {
            // airlock dry [zatoshi]   one EXACT_OUTPUT dry quote per origin asset into one Orchard-only UA
            let zat = args.get(1).cloned().unwrap_or_else(|| "1000000".into());
            let ua = Wallet::open(Path::new("work"))?.orchard_ua(0)?;
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
        Some("audit") => {
            // airlock audit <depositAddress>...   live /v0/status reads, shared identifier count
            if args.len() < 2 {
                bail!("usage: airlock audit <depositAddress>...");
            }
            let mut raws = Vec::new();
            for d in &args[1..] {
                let s = oc.status(d).await?;
                println!("{d} {:?}", s.status);
                raws.push(s.raw);
            }
            let shared = shared_identifiers(&raws);
            for s in &shared {
                println!("SHARED {} = {} in parts {:?}", s.field, s.value, s.parts);
            }
            println!("shared identifiers: {}", shared.len());
        }
        _ => bail!("usage: airlock ua [n] | dry [zatoshi] | audit <depositAddress>..."),
    }
    Ok(())
}
