# SHORE submission form answers

**Name.** SHORE

**One line.** Got ZEC in Phantom from rewards? Shield it in one approval: SHORE moves ZEC on Solana into a shielded
Zcash balance with one Phantom approval, reads the note back, and says plainly what stays public.

**Live.** App https://shore-zec.pages.dev/shore/ (Cloudflare Pages) with the API at https://airlock-zec.fly.dev (Fly,
one machine, Cloudflare D1 store). Repository: private GitHub repo, access on request.

**For judges.**
- 60 seconds, hands-free, nothing sent: https://shore-zec.pages.dev/shore/?replay=1&tour=1&speed=3 (the recorded mainnet run).
- Invite preview: https://shore-zec.pages.dev/shore/?invite=zechub&amount=0.005&note=Welcome&replay=1 (the banner a
  holder would see, then the replay; no wallet, no API or mainnet call).
- Live: https://shore-zec.pages.dev/shore/?live=1 with Phantom and your own Zcash wallet's u1, capped at 0.02 ZEC.

**ZEC in the product.** ZEC on both sides: the NEAR-bridged SPL ZEC in Phantom goes through NEAR Intents 1Click into a
fresh Orchard-only u1 on Zcash mainnet, and SHORE decrypts the landed note with a viewing key (never a spending key).
For your own address, you can paste your wallet's viewing key for a one-time read-back; it is not stored.

**Feature audit (production, 2026-10-11).** Every live step short of signing was run against the production app and
API (`web/scripts/live-audit.mjs`, results `docs/submission/live-audit.json`). Nothing was signed or spent; a stand-in
wallet supplies a real Solana address because a Phantom extension cannot run headless.

| Feature | Status | Evidence |
| --- | --- | --- |
| Replay of the recorded mainnet run, 60s judge tour | working | Playwright on production |
| Phantom connect (live) | working, stand-in wallet in tests | live audit: connect, real address |
| ZEC balance via `/api/rpc` (key server-side, cache, retry, fallback, read-only) | working | live audit: real balance 4,571 base units; `sendTransaction` refused (400); `tests/rpc_proxy.rs` |
| Pasted u1 validation (transparent / Sapling / no shielded receiver refused) | working | live audit: `TRANSPARENT_REFUSED`; Rust tests |
| Address reuse rejected (D1 used-address list) | working | live audit: second use 409 `ADDRESS_REUSED`, and the page shows it |
| Live mode never offers the demo wallet | working | live audit: no "fresh u1" button |
| Dry and non-dry 1Click quote, fee breakdown, one quote per shield | working | live audit: deposit address issued, deadline, repeat 409 `QUOTE_EXISTS` |
| Review and one approval | working up to signing | live audit: "Approve 0.0014 ZEC in Phantom", deposit address shown, one sign request, refused by the stand-in |
| Status polling | working | live audit: `PENDING_DEPOSIT` from 1Click |
| Read-back, demo wallet (UFVK scan) | working | recorded mainnet run, 115,000 zat |
| Read-back, your own address, with your viewing key | new, working | live audit: wrong wallet's key `KEY_MISMATCH`, a seed phrase `BAD_UFVK`, right key `WAITING`; Rust + Playwright end to end with a mocked payout |
| What's still public (exposure) | working | live audit: 7 rows |
| Invite links, judge preview `&replay=1` | working | Playwright on production, zero off-origin requests |
| SHORE fee via 1Click `appFees` | built, switched off | `shoreFeeBps: 0` in the live quote; Rust test that no fee is sent unless configured |
| Full live shield end to end | needs a funded wallet | the mainnet run of 2026-10-10 above; `scripts/live-run.sh` for the next one |

**Mainnet proof (2026-10-10).**
| Leg | Transaction | Time (IST) |
| --- | --- | --- |
| Solana, one Phantom approval, 0.0014 ZEC | [2LLVDBnV…QQ86au](https://solscan.io/tx/2LLVDBnVAqnG99PSbUVqapKqZFHjPPET9HsJhV2tqS29BHR4wNmUzYZbQzLf2BXjpFKMrRJCYxbVZq6ZJNQQ86au) | 09:22:15 |
| NEAR Intents 1Click | [BARJAuxG…tkqC53](https://nearblocks.io/txns/BARJAuxGT1tXY8XAK9LsUCroQsA3V1AgErm9DhtkqC53) | |
| Zcash, shielded note, height 3,512,504 | [fa8d4d6b…5fa408](https://mainnet.zcashexplorer.app/transactions/fa8d4d6bd210c6150e98142ec1ae9a25706ec19bd591a775823f6616f95fa408) | 09:23:39 |

84 seconds from send to land. 115,000 zat landed in the Ironwood pool (v6 transaction, Orchard receiver),
read back by a view-only key (UFVK, no spending key). 1Click's status said 108,000 zat; the decrypted note holds
7,000 zat more, and SHORE shows the decrypted value. Cost: $1.70 in, $1.31 out at quote time, so about
39 cents, most of it 1Click's fixed 0.00032 ZEC Zcash withdraw fee (22.9% at this tiny size; it falls fast as the
amount grows).

**SHORE fee.** Wired but off: the quote request can carry a 1Click `appFees` entry (`SHORE_APP_FEE_BPS`,
`SHORE_APP_FEE_RECIPIENT`) and the quote shows it as its own "SHORE fee" line; today it is not set, so `shoreFeeBps` is 0.

**Business.**
Holders will pay a small, transparent SHORE fee on each shield: it is built in through the 1Click `appFees` field
(`SHORE_APP_FEE_BPS`, 1 to 100 bps, paid to the NEAR account `SHORE_APP_FEE_RECIPIENT`) and shown as its own "SHORE fee"
line in the quote, and it is switched off today, so every shield so far paid only 1Click's fees. Wallets, communities
and bounty programs (ZecHub, zkSNARKs, Raydium reward campaigns) are the planned buyers of invite links that bring their
members through SHORE in one approval; none has paid yet. SHORE never holds funds, so the fee is the whole business.

**Honest risks (what's still public).**
- **The hop in is public.** Your Phantom address, the deposit address, the amount and the time are on Solana.
- **The route is linkable.** 1Click's unauthenticated `/v0/status` maps the deposit to the recipient u1, and
  L2Beat reports that NEAR Intents Zcash payouts are decryptable by anyone
  (https://l2beat.com/privacy/projects/zcash-near-intents). So the Phantom address is linked to that one u1
  and amount. Use a fresh address for every shield, so the link does not extend to your other shields. What
  stays private is everything after: spends from the note.
- **Counterparty risk.** The hop runs through an operated solver network that can delay, refund or freeze funds.
  Precedent: about $589,000 of USDT from 1,120 shielded ZEC was held by NEAR Intents for 50 days
  (https://forum.zcashcommunity.com/t/57497/1). SHORE never holds funds but cannot overrule 1Click.
- **You need a Zcash wallet.** On mainnet the ZEC goes only to an address you paste from your own wallet
  (Zashi/Zodl, Ywallet). SHORE's "fresh address" button is the demo wallet and exists only in the replay;
  live mode never offers it.
- Launch cap 0.02 ZEC per shield; 1Click minimum 0.00133334 ZEC.

**Built before vs during.**
Tag `baseline-pre-hackathon` (8e11db3, 2026-10-05) is AIRLOCK as it stood: the Rust 1Click client, Tor
isolation, wallet and UFVK scanning, exposure receipts, the AIRLOCK web app and the Fly app.
Built during (`git diff baseline-pre-hackathon`): the SHORE HTTP API, pluggable store with the D1 Pages
Function, the Solana RPC proxy, `airlock ufvk`, the SHORE web flow (replay, live, invite links), its tests,
the mainnet run, and the design work. About 1,310 lines added in `src/` (921) and `tests/` (393), and about 1,480 in the SHORE web flow,
its browser tests, scripts and deploy files (measured with `git diff --shortstat baseline-pre-hackathon`, 2026-10-11).

SHORE pivoted from AIRLOCK, the earlier exposure-receipt tool in the same repository, and reuses its 1Click client,
Tor isolation and UFVK scanning. Third-party libraries: librustzcash crates, arti (Tor), axum, tokio; the `countersign-core`
path crate (a sibling repo) is used only by AIRLOCK's own shielded sends, not by SHORE.
