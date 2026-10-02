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

## Proven (2026-10-03, later)
- B2 live: `airlock outbound-quote 1.5 0.5 <USDC base> <3 random EVM dests>` created swap e89ed92b43d6: three 0.5 ZEC parts (ZIP 318 greedy split), 3 non-dry quotes over isolated Tor handles, fresh Orchard-only refund UA per part (1Click accepted UA refundTo on non-dry). `airlock audit e89ed92b43d6` over fresh isolated circuits: shared identifiers 0, exit 0. Unfiltered leaf comparison lists only route constants (amount 0.5 ZEC identical by design, asset ids, fees, appFees recipient). spikes/dry-quotes/b2-audit-green.out.
- B2 red (live): mutated Wallet::fresh_ua to stop advancing the index, swap 460d6608ac09 (2 parts): audit reported `SHARED refund_to = u1czvd...` and exited 2. Mutation reverted, tests green (13 + 1).
- Leak re-measured: /v0/status with no key returns the shielded refund UA and EVM recipient for every part (receipt shows them).
- Tor: 4 isolated handles got 4 different exit IPs (check.torproject.org). 3 of 4 were in 192.42.116.0/24: different circuits, not independent operators.
- Both swap files and seed live in gitignored work/. The three unfunded quotes expire 2026-10-06; no funds sent.
- Tor exit hostname-lookup failures happen (seen once); transport retries on a fresh isolated circuit, 3 times.
- Operator rule (CLAUDE.md): testnet only, mainnet read-only. B3/B4 funded swaps on hold pending explicit OK. FUNDING.md not written.

## Next (re-planned)
- Leak dataset: read recent mainnet ZEC t-outputs read-only, call /v0/status per address, count what leaks. Nothing published until orchestrator OKs disclosure.
- Watchdog against recorded status transitions; web app (static frontend + Rust service).

## Proven (2026-10-03, evening)
- NU7 pins adopted (docs/PINS.md; Cargo.lock from countersign). Breaks hit: rand 0.10 (SysRng), 3-tuple UA decode, Rev 2 UA strings. `cargo test`: 21 + 1 green. B1 analyzer green, B2 live audit of swap e89ed92b43d6 re-run on the new pins: 0 shared identifiers, exit 0.
- 1Click REJECTS ZIP 316 Revision 2 UAs: dry quote with `zu1...` recipient returned 400 "recipient is not valid" for all 5 origins (2026-10-03 21:03 server time). `Wallet::orchard_ua` re-encodes to Revision 1 (`u1...`); index 0 is byte-identical to the pre-migration string; dry quotes 201 again for 5 origins. Test pins both encodings.
- Watchdog: 7 tests, red proven (timeout comparison mutated, stuck test FAILED), recorded live statuses (13 non-success + 5 success) parse and map to outcomes.
- Leak dataset (read-only, spikes/leak-scan/FINDINGS.md): 909 live 1Click deposit t-addresses in 1,501 mainnet blocks, all ZEC-origin, 896 SUCCESS (latest update 2026-10-02T20:38Z, so ZEC outbound is settling), 558 expose a shielded refund UA, 120 recipients link 306 swaps, 50 refund addresses link 131 swaps. Not published.
- Kill-gate evidence: ZEC-origin 1Click swaps settle now. Inbound (any token to shielded ZEC) delivery stays unproven: needs a funded swap, on hold by operator rule.

## Next
- `watch` driver run against a real terminal swap (public SUCCESS address) to exercise the loop.
- Web app: static frontend + Rust service (axum), exposure checker for any deposit address, outbound plan, receipt.

## Proven (service)
- `airlock serve` (axum): /api/health, /api/tokens, /api/plan, /api/inspect/{t-addr}, POST /api/swaps, GET /api/swaps/{id}. Live on 127.0.0.1:8787: plan 1.3/0.5 -> 4 canonical parts; inspect of an own deposit address returns the public fields; POST /api/swaps with repeated refund UA -> 400 (red), with 3 distinct refunds -> swap bd0c40d63886 with 3 real unfunded quotes over Tor (green); GET returns receipt JSON, shared identifiers [] and 26 shared constants, 11 s over Tor.
- Service holds no keys: refund UAs come from the caller's wallet, converted to Revision 1 server side.
- Blind spot: no funded swap, so deposit and delivery are unexercised.

## Proven (note read-back, testnet)
- `airlock note test 200000`: view-only scan (UFVK, no spending key) through testnet.zec.rocks found the Ironwood note of 200,000 zat sent by zc-countersign: txid a37a3dbc29b5093aee3f016278321e8ab999d0f1be223d6fbf8abd511c24cd39, mined height 4,435,638. Independent GetTransaction on the same txid returns height 4435638. Red: threshold 300,000 zat exits with "no note ... within 25s".
- This is the read-back half of B3. The swap half (1Click delivering to the UA) is on hold by operator rule.
- Inbound quote path: `airlock inbound 0.01 <USDC base> <0x refund>` returned a non-dry quote with a Base deposit address and a Revision 1 Orchard UA recipient (inbound 612fbdaab66d, unfunded). Endpoints POST /api/inbound, GET /api/inbound/{id} written, built, not yet exercised live.
- Jittered schedule: ZIP 318 transfer delay distribution (mean 66 blocks, cap 576), 2 tests, mean checked over 20,000 samples.

## Proven (COUNTERSIGN signing, testnet)
- `airlock fund <testnet UA> 50000`: view-only propose, COUNTERSIGN policy check + signature (`countersign_core::approve_and_sign`, path dep), prove, extract, broadcast, poll. Testnet tx 540762b04fcfe08e0c943c15cf90af4ba409fb1756ff245dd5e235b74a35594c mined at height 4,435,827 (handoff-tx and GetTransaction agree). 56 s end to end.
- Red: signer refuses with `FeeTooHigh { fee: 10000, max: 1 }` when the approved max fee is 1.
- Expiry: tx v6, expiry_height 4,435,848, mined 4,435,827. No airlock code sets an expiry (library default, 40 blocks past the target height); `grep expiry src` is empty. No tip+200.
- Mainnet funding is refused in code (`fund` returns an error unless the network is testnet).
- GAP: COUNTERSIGN's policy refuses any transparent output (`ForbiddenPool("transparent")`), so the shielded to deposit t-address hop cannot go through it yet. Asked zc-countersign for a transparent-output Intent.
- Static web app committed: Check, Split, Inbound, Swap pages; `uicraft gate` 0 high; zero console errors at 375 and 1440; fixed a doubled-decimal bug in the inbound amount.

## Proven (shielded to transparent deposit hop, testnet)
- `airlock fund tmHx6PGY... 40000`: COUNTERSIGN `Intent::to_transparent_once` (zc-countersign f8b9ab5), signed by the policy check, proved, broadcast. Testnet tx 49046be46d4ef114b3497897eefdb20290186f19c7c838fb6483f805576aabdf mined at height 4,435,995. This is exactly the hop a 1Click deposit address needs.
- Red: `fund ... 40000 1` refuses with FeeTooHigh (max fee 1).
- Judge runs (zecathon-judge: DISQUALIFY as shipped; cwf-judge-zcash: 32/60). Top finding shared by both: the service linked all parts and the copy said "nothing to link". Fixed: the service is stateless (POST /api/quote per part, GET /api/inspect over Tor, no swap storage), the browser holds the swap and spaces part quotes 4 to 25 s apart, the receipt lists the AIRLOCK service as an observer plus the timing caveat. CLI part quotes are also jittered (4 to 25 s).
- Pending: purge probe_full.out from history (filter-branch blocked by the web agent's unstaged edits), web rework verification.
