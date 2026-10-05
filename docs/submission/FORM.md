# AIRLOCK submission form answers

**One line.** Swap into and out of shielded ZEC through NEAR Intents with the smallest public
footprint, and print a receipt that proves it.

**Live.** https://airlock-zec.vercel.app (static export, no serverless functions); service
https://airlock-zec.fly.dev; repository: this repo.

**ZEC in the product.** ZEC is the asset on both legs. Inbound: any token to shielded ZEC in a fresh
Orchard-only unified address. Outbound: shielded ZEC split into ZIP 318 denominations and sent to
per-part deposit addresses. Testnet transactions read back through lightwalletd: heights 4,435,995
and 4,466,716 (deposit hop), 4,435,638 (note read-back).

**Prior work disclosure.** AIRLOCK is a clean repository whose first commit is dated 2026-10-03,
inside the event window. No code from an earlier project. Third-party libraries: librustzcash crates
(NU7 generation pins), arti (Tor), axum, tokio. The signing step uses the COUNTERSIGN crate, also
built during the window, as a path dependency.

**Evidence.** `cargo test` (floor and no failures), live audit of a three-part swap with 0 shared
identifiers, red and green runs recorded under `spikes/dry-quotes/`, leak dataset under
`spikes/leak-scan/`.

**Business.** Buyers are wallets and apps that route ZEC through NEAR Intents (Zodl in-wallet swaps,
Zwap, Vizor) and the holders who exit through them. Named users and what they said, verbatim sources:
timtech, who moved about 589,000 USDT worth of ZEC (1,120 ZEC held 50 days) through the rail
(https://forum.zcashcommunity.com/t/57497/1); @Zpartan, "5 figs" frozen on an exit
(https://x.com/Zpartan/status/2105623264865157577). Integration target: the exit flow of a wallet adds
the part planner and the receipt page. Revenue path: every 1Click quote carries an `appFees` field the
integrator can set; AIRLOCK's receipt and planner are the reason a wallet would route exits through it.
Distribution: receipts shared by users who audit their own swap, a how-to for ZEC exits, a link from
wallet exit flows.
