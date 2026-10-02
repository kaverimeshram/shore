## 1. AIRLOCK (ZECATHON track: CROSS-CHAIN)

Swap into and out of shielded ZEC through NEAR Intents with the smallest possible public footprint:
inbound lands straight in a fresh Orchard-only UA (no unshield), outbound is split into ZIP 318
canonical denominations, each part with its own one-time deposit t-address, fresh refund UA,
destination and Tor-isolated quote/status circuit, and a per-swap watchdog that drives every part
to SUCCESS or REFUNDED and caps how much any single hold can freeze. Every swap ends with an
exposure receipt listing what a chain observer, the 1Click status endpoint and the solver can see.

Scoreboard: Zodl CrossPay and Zodl in-wallet swaps (one user, one wallet, live; zaddrnet reports a 1% send fee, DEMAND §8); Zwap (immabeyeet, DEMAND §3); Zypherpunk 2025 had no winner that hides the NEAR Intents exit leg (PRIOR-ART §1). L2Beat's NEAR Intents privacy page lists decryptable payouts and matchable entry and exit amounts (PRIOR-ART summary, item 8). No usage numbers are published for any of them
Bar to beat: Zodl's swap flow, which already uses a fresh shielded refund and recipient address (MECHANISMS §1.3). Our measurement: an unauthenticated `GET /v0/status?depositAddress=` returns `quoteRequest.recipient` and `refundTo` for any funded one-time t-address (MECHANISMS §1.5, status_probe.out). The bar is 0 shared identifiers across the parts of one outbound swap, both on chain and in /v0/status
Asset we will own: an exposure-measurement harness plus a dataset of mainnet 1Click ZEC swaps, recording linkable fields per swap and hold or refund latency per route, built from public /v0/status reads and lightwalletd. The tooling is a Rust 1Click client generated from oneclick-openapi.yaml (1.9 MB, 14 paths); only TypeScript and Go SDKs exist today
Off-platform buyer: ZEC holders exiting to stables who were frozen or exposed: timtech (about $589k USDT from 1,120 ZEC held 50 days, forum.zcashcommunity.com/t/57497/1); @Zpartan ("5 figs" frozen, x.com/Zpartan/status/2105623264865157577); brayo1st (unresolved partial swap, r/zec); nearpadtech (creator payouts in native ZEC); bentertainer of Raydium (84,000 wallets paid in ZEC, DEMAND §8)
Single entry: AIRLOCK, ZECATHON track Cross-Chain, account 1 of 3; CWF Zcash track
Verb the brief names: ZECATHON: "Swaps, bridges and intents that reach other chains without unshielding on the way through." CWF: "integrating ZEC into their products"
Our product performs that verb: planned. Inbound: 1Click quote with recipient = fresh Orchard-only UA (the validator accepts it, MECHANISMS §1.3), note read back from lightwalletd. Outbound: one unavoidable shielded-to-transparent crossing per part (MECHANISMS §1.5), minimised and measured rather than hidden. Code path: projects/airlock/src (not yet written)
Metric plan: mainnet swaps completed (terminal SUCCESS, both legs read back) and the shared-identifier count per outbound swap (target 0). Daily: at least 1 real swap from 2026-10-06; at least 10 swaps by outside users by 2026-10-20. Checked with `airlock audit <swap-id>` against /v0/status and lightwalletd
Live by: 2026-10-08 web app on a public URL with one mainnet inbound and one split outbound swap (CWF, 4 days before its deadline; the 7-day rule is missed by 3 days because the build starts 2026-10-03). ZECATHON hardening live by 2026-10-21
Deviation from research: MECHANISMS ranked X2 (outbound exposure accounting) first and X1 (pay in any token, settle shielded) second; AIRLOCK merges both, because the demand file's strongest asks are both exiting (5 askers) and delivering straight into the shielded pool (4 askers). Confidential Intents (X3) is deferred until the operator places a partner JWT
Ask it answers: https://x.com/Zpartan/status/2105623264865157577 ; https://forum.zcashcommunity.com/t/57497/1 ; ZCG wishlist "atomic swaps" (DEMAND §6)
Users who want it: timtech (forum 57497), @Zpartan (x.com/Zpartan/status/2105623264865157577), Firstmate and ZecMec (forum, DEMAND §3), brayo1st (r/zec), strahncryptography, opReturn, immabeyeet, mohbearded (deliver into the shielded pool, DEMAND §3)
