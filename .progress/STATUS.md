# AIRLOCK status

## Proven (2026-10-03)
- B1 analyzer: `cargo test --lib analyzer` green (fresh identifiers -> 0 shared, reused refund UA -> 1 shared).
  Red proven by deleting the `refund_to` read: `analyzer_b1_reused_refund_ua_flagged` FAILED, restored, green.
  Blind spot: fixture is one real /v0/status payload (research/tech/status_probe.out) edited per part, not a live multi-part read; B2 covers live.
- `airlock dry`: 5 origin assets (USDC sol/base/arb/eth, BTC) quoted 201 into one Orchard-only UA. spikes/dry-quotes/FINDINGS.md.
- 1Click is up: /v0/tokens 200 (202 tokens), /v0/quote 201. Not paused for these routes.

## Next
- 10-04: ZIP 318 splitter, per-part fresh UA/refund/destination, Tor-isolated non-dry quotes, B2 live.

## Blocked
- Nothing. Funding not yet needed (B3 on 10-05 needs a small USDC balance on sol or base, see FUNDING.md once written).
- Plan constraint relayed by solana-18: CWF needs static-only Vercel frontend plus separately hosted Rust service.
