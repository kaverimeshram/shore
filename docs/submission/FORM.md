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

**Business.** Buyers are wallets and apps that route ZEC through NEAR Intents and holders who exit
through them. Distribution: receipts shared by users who audit their own swaps, a how-to for ZEC
exits, and a link from wallet exit flows. Detail in the gtm folder of the workspace.
