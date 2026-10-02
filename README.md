# AIRLOCK

Swap into and out of shielded ZEC through NEAR Intents with the smallest public footprint, and
get a receipt that proves it.

Every exit from shielded ZEC crosses one transparent deposit address. AIRLOCK makes that crossing
as small and as unlinkable as the protocol allows: the amount is split into ZIP 318 canonical
denominations, every part gets its own deposit address, its own fresh refund address and its own
destination, every request leaves on its own Tor circuit, and a watchdog drives every part to a
terminal state. The exposure receipt then recomputes, from live reads, what a chain observer, the
1Click status endpoint and the solver can link, and counts the identifiers two parts share. The
target is zero, and `airlock audit` prints the number.

## What it does

**Inbound, any token to shielded ZEC.** An `EXACT_OUTPUT` quote whose recipient is a fresh
Orchard-only unified address. No unshield step exists. A view-only watcher (UFVK, never a spending
key) scans lightwalletd and reports the note with its txid and mined height.

**Outbound, shielded ZEC to another chain.** `split` cuts the amount into `{1,2,5} x 10^k`
denominations capped per hold. One non-dry quote per part, each over its own isolated Tor
circuit, each with its own refund UA and destination. A ZIP 318 transfer-delay schedule (mean 66
blocks, cap 576) suggests when to send each part.

**Exposure receipt.** Per swap, a table of what each observer can read, built from the live
`/v0/status` payloads, plus an unfiltered comparison of every field across parts so nothing is
hidden by an allow-list.

**Exposure check for any deposit address.** Paste a one-time deposit t-address and see every field
the unauthenticated status endpoint returns for it.

**Watchdog.** Polls every part to SUCCESS, REFUNDED, FAILED or expired, escalates a funded part
that stalls past its hold timeout, and gates the next funding on the value already in flight.

## Run it

```sh
export CARGO_TARGET_DIR=<shared target>
cargo build
airlock dry                       # dry quotes, 5 origin assets into one Orchard-only UA
airlock outbound-quote 1.5 0.5 <destAssetId> <dest1,dest2,dest3>
airlock audit <swap-id>           # live reads over isolated circuits, receipt, shared count
airlock inspect <depositAddress>  # what /v0/status reveals for any deposit t-address
airlock watch <swap-id>           # watchdog until every part is terminal
airlock inbound 0.01 <originAssetId> <originRefundAddress>
airlock note test|main <min-zat>  # view-only note read-back from lightwalletd
airlock serve                     # HTTP service for the web app
```

The web app is static files in `web/` (no build step, no serverless functions). It calls the Rust
service named in `web/config.js`.

## Evidence

- `cargo test`: the analyzer flags a reused refund UA and reports zero for fresh ones; the
  splitter conserves value and only emits canonical denominations; the watchdog escalates a stalled
  part exactly once. Each acceptance was broken on purpose and shown red before it was shown green.
- A live three-part outbound swap audited over fresh isolated circuits reports 0 shared identifiers;
  the same flow with one refund UA reused reports `SHARED refund_to` and exits non-zero
  (`spikes/dry-quotes/b2-audit-green.out`, `b2-audit-red.out`).
- Four isolated Tor handles reach 1Click through four different exit relays.
- A 200,000 zat Ironwood note sent to a fresh Orchard-only UA on testnet is found by the view-only
  watcher at txid `a37a3dbc29b5093aee3f016278321e8ab999d0f1be223d6fbf8abd511c24cd39`, mined at height
  4,435,638 and confirmed by an independent `GetTransaction`.
- Dry quotes for USDC on Solana, Base, Arbitrum and Ethereum, and BTC, all resolve into one
  Orchard-only unified address.

## Provenance

AIRLOCK is built in a clean repository started on 2026-10-03. It shares no code with earlier
projects. Dependencies are pinned to the NU7 generation of the Zcash crates (`zcash_protocol`
0.11.0-pre.0 and its matching releases).
