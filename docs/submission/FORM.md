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
(NU7 generation pins), arti (Tor), axum, tokio. Shielded sends are signed through the COUNTERSIGN
signing crate, written during the window by a separate team member under the same event; it is a path
dependency today and the form lists it so a reader can follow it.

**Evidence, with results.**
- `cargo test`: 33 library tests and 1 binary test pass; each acceptance was shown red with the guarded
  code broken, then green.
- Live three-part outbound swap `e89ed92b43d6`: `airlock audit` prints 0 shared identifiers over fresh
  Tor circuits and exits 0 (`spikes/dry-quotes/b2-audit-green.out`). A run with one refund address
  reused printed `SHARED refund_to` and exited 2 (`b2-audit-red.out`). The web form now refuses the reuse.
- Testnet transactions read back through lightwalletd: deposit hop `a2f1c432...ea266` at 4,466,716
  (after NU7 activation), `49046be4...abdf` at 4,435,995, funding `540762b0...594c` at 4,435,827,
  inbound note read-back `a37a3dbc...cd39` at 4,435,638.
- Leak dataset (`spikes/leak-scan/FINDINGS.md`): 909 live 1Click deposit addresses in 1,501 mainnet
  blocks, 558 showing a shielded refund address, 120 recipients tying 306 swaps together.
- Live: https://airlock-zec.vercel.app against https://airlock-zec.fly.dev. A real quote from the page for
  0.02 ZEC inbound returned a Base deposit address and 27.52 USDC.
- Mainnet round trip: MAINNET SLOT, filled from txids once it has run.

**Business.** Buyers are wallets and apps that route ZEC through NEAR Intents (Zodl in-wallet swaps,
Zwap, Vizor) and the holders who exit through them. Named users and what they said, verbatim sources:
timtech, who moved about 589,000 USDT worth of ZEC (1,120 ZEC held 50 days) through the rail
(https://forum.zcashcommunity.com/t/57497/1); @Zpartan, "5 figs" frozen on an exit
(https://x.com/Zpartan/status/2105623264865157577). Integration target: the exit flow of a wallet adds
the part planner and the receipt page. Revenue path: every 1Click quote carries an `appFees` field the
integrator can set; AIRLOCK's receipt and planner are the reason a wallet would route exits through it.
Distribution: receipts shared by users who audit their own swap, a how-to for ZEC exits, a link from
wallet exit flows.
