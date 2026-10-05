# Measured exposure of public 1Click ZEC swaps (read-only, 2026-10-03)

Nothing here is published. Per-swap rows (addresses) stay in gitignored files; only aggregates
are committed. Disclosure to NEAR waits for the orchestrator's OK.

## Method (all reads, no funds, no writes)
1. `grpcurl zec.rocks:443 .../GetBlockRange` with `poolTypes:[TRANSPARENT]`, blocks 3,502,581 to
   3,504,081 (1,501 blocks, tip 3,504,081). `extract.py` keeps P2PKH outputs: 7,669 t-addresses,
   5,799 with exactly one output of at least 0.01 ZEC (the shape of a one-time deposit address).
2. `probe.py`: `GET /v0/status?depositAddress=<t-addr>` with no key, 0.25 s apart (documented
   unauthenticated limit is 1,200 per minute; the run stayed near 150 per minute).
3. `analyze.py` over the 200 responses (`analysis.out`).

## Result
- 909 of 5,799 candidate t-addresses are live 1Click deposit addresses (HTTP 200); 4,890 returned 404.
  All 909 are ZEC-origin. Blind spot: the candidate filter (single output, at least 0.01 ZEC)
  misses deposits that arrive as several outputs; 909 is a floor for the window, not a census.
- Status: 896 SUCCESS, 8 REFUNDED (4 AMOUNT_LESS_THAN_MIN_AMOUNT_OUT, 2 NO_LIQUIDITY, 2
  PARTIAL_DEPOSIT), 4 PROCESSING, 1 INCOMPLETE_DEPOSIT. Latest `updatedAt` 2026-10-02T20:38:12Z.
  So 1Click is settling ZEC outbound swaps now: the pause reported on forum 57978 does not show
  on this route.
- Every response carries `quoteRequest.recipient` and `refundTo` in the clear.
- 558 of 909 swaps expose a shielded unified refund address; 350 expose a t-address; 1 other.
- 120 recipient addresses sit behind two or more deposit addresses (306 swaps linked by a
  shared destination). 50 refund addresses are reused across deposit addresses (131 swaps linked).
  The most reused recipient is behind 10 swaps, the most reused refund address behind 7.
- Destination chains: eth 241, sol 212, tron 203, arb 71, base 52, btc 44 (plus 26 nBTC, 14 wrap).
- 141 of 909 amounts are ZIP 318 canonical denominations; the rest are arbitrary and
  fingerprint the sender by amount.

## What AIRLOCK changes, against these numbers
- One refund UA and one destination per part: the shared-identifier count is 0 by construction
  and measured by `airlock audit` (B1, B2).
- The exposed shielded refund UA is fresh per part, so the public readout never joins two swaps.
- Canonical amounts draw each part's amount from the ZIP 318 denomination set instead of an arbitrary value (141 of 909 live swaps already use one). Whether that set is shared with other transparent outputs on chain is not measured.
- Not changed: the t-address and amount are public on chain, and the sum is recoverable by
  amount and timing correlation.
