# SHORE

**Every bridge says private. SHORE itemizes it.**

One Phantom approval moves the ZEC in your Solana wallet (the NEAR-bridged SPL token) into a shielded Zcash note.
Then SHORE hands you the receipt: every link the crossing left behind, who can read it, and a read-back from Zcash
itself, with your viewing key, that the note landed.

The receipt of the recorded mainnet crossing (10 Oct 2026, 84 s,
[Solana tx](https://solscan.io/tx/2LLVDBnVAqnG99PSbUVqapKqZFHjPPET9HsJhV2tqS29BHR4wNmUzYZbQzLf2BXjpFKMrRJCYxbVZq6ZJNQQ86au),
[Zcash tx](https://mainnet.zcashexplorer.app/transactions/fa8d4d6bd210c6150e98142ec1ae9a25706ec19bd591a775823f6616f95fa408)),
the same rows the app renders:

| Item | Who can read it |
|---|---|
| Solana sender `ia3MLukF…Vsa5` | anyone (Solana explorer) |
| Deposit address `DMDdCzum…SjbM` | anyone (Solana explorer) |
| Amount in and timing, 0.0014 ZEC | anyone (Solana explorer) |
| Recipient u1, via 1Click status | anyone holding the deposit address (unauthenticated `/v0/status`) |
| Amount entering the shielded pool, 0.00115 ZEC | anyone (the delivery tx has transparent inputs) |
| Note contents | 1Click/NEAR (the sender holds the outgoing view key) |
| Later spends of the note | nobody |

SHORE acts on what the receipt shows: by default each shield goes to a fresh Orchard-only u1, so 1Click status cannot
link one crossing to the next, and the API refuses the three moves that would leak more: a transparent address
(`TRANSPARENT_REFUSED`), a u1 you shielded to before (`ADDRESS_REUSED`), and a viewing key that does not cover the
address it is asked to prove (`KEY_MISMATCH`).

86,000 Solana wallets received ZEC since September, and bridging to Zcash from them is
["basically none"](https://forum.zcashcommunity.com/t/57788/10) (Dune numbers posted on the Zcash forum, 24 Sep 2026).
SHORE is the crossing for them, with the receipt attached.

- App: https://shore-zec.pages.dev/shore/ (opens on a replay of a real mainnet run; `?live=1` for mainnet)
- API: https://airlock-zec.fly.dev (Fly, one machine, no volume)
- Invite links: `https://shore-zec.pages.dev/shore/?invite=<code>&amount=<ZEC>&note=<text>`

## For judges

1. Open https://shore-zec.pages.dev/shore/ and press **Try it in 60s** (`?replay=1&tour=1&speed=3`). It plays the
   recorded mainnet run of 2026-10-10 hands-free. No wallet is touched and nothing is sent.
2. What you are seeing: connect, then a fresh Orchard-only u1 and a 1Click quote with every fee shown; one Phantom approval
   and the crossing on the tide band; then the view-key read-back of the landed note and "What's still public".
3. Check the proof yourself: the txids under "Mainnet proof" below, all on public explorers.
4. Live: `?live=1` with Phantom and your own Zcash wallet's u1 (capped at 0.02 ZEC). After it lands, paste that wallet's
   viewing key for the read-back, or open the wallet. Live audit of every step short of signing: "Feature status" below.
5. Invite links, safely: `?invite=zechub&amount=0.005&note=Welcome&replay=1` shows the invite banner exactly as a
   holder would see it, then plays the recorded run. It is marked "Judge preview", touches no wallet and makes no API or
   mainnet call. Without `replay=1` the same link opens the live flow (look at the banner, do not connect).
6. Code: `src/shore.rs` (API), `web/shore/` (app), `tests/shore_api.rs`, `tests/rpc_proxy.rs`, `web/tests/`.

## Business model

Holders will pay a small, transparent SHORE fee on each shield: it is built in through the 1Click `appFees` field
(`SHORE_APP_FEE_BPS`, 1 to 100 bps, paid to the NEAR account `SHORE_APP_FEE_RECIPIENT`) and shown as its own "SHORE fee"
line in the quote, and it is switched off today, so every shield so far paid only 1Click's fees. Wallets, communities
and bounty programs (ZecHub, zkSNARKs, Raydium reward campaigns) are the planned buyers of invite links that bring their
members through SHORE in one approval. SHORE never holds funds, so the fee is the whole business.

## Verified on production (2026-10-11 10:30 IST)

Checked by `web/scripts/live-audit.mjs` against https://shore-zec.pages.dev and https://airlock-zec.fly.dev, and by
the Rust and browser suites. The audit drives every step up to the wallet signature on production; the mainnet run
below carries the signature and the delivery.

| Feature | Status | Evidence |
| --- | --- | --- |
| Replay of the recorded mainnet run, 60s judge tour | working | Playwright on production |
| Phantom connect (live) | working | live audit: connect, real Solana address and ZEC balance |
| ZEC balance via `/api/rpc` (key server-side, cache, retry, fallback, read-only) | working | live audit: real balance 4,571 base units; `sendTransaction` refused (400); `tests/rpc_proxy.rs` |
| Pasted u1 validation (transparent / Sapling / no shielded receiver refused) | working | live audit: `TRANSPARENT_REFUSED`; Rust tests |
| Address reuse rejected (D1 used-address list) | working | live audit: second use 409 `ADDRESS_REUSED`, and the page shows it |
| Live mode never offers the demo wallet | working | live audit: no "fresh u1" button |
| Dry and non-dry 1Click quote, fee breakdown, one quote per shield | working | live audit: deposit address issued, deadline, repeat 409 `QUOTE_EXISTS` |
| Review and one approval | working (signature in the mainnet run) | live audit: "Approve 0.0014 ZEC in Phantom", deposit address shown, one sign request issued; signed and delivered in the mainnet run |
| Status polling | working | live audit: `PENDING_DEPOSIT` from 1Click |
| Read-back, demo wallet (UFVK scan) | working | recorded mainnet run, 115,000 zat |
| Read-back, your own address, with your viewing key | new, working | live audit: wrong wallet's key `KEY_MISMATCH`, a seed phrase `BAD_UFVK`, right key `WAITING`; Rust + Playwright end to end with a mocked payout |
| What's still public (exposure) | working | live audit: 7 rows |
| Invite links, judge preview `&replay=1` | working | Playwright on production, zero off-origin requests |
| SHORE fee via 1Click `appFees` | built, switched off | `shoreFeeBps: 0` in the live quote; Rust test that no fee is sent unless configured |
| Full live shield end to end | needs a funded wallet | the mainnet run of 2026-10-10 above; `scripts/live-run.sh` for the next one |

## Read-back for your own address

Fresh addresses from the demo wallet exist only in the replay, so on mainnet the note lands in your own wallet. After
it lands, you can paste that wallet's unified viewing key (`uview1...` or `uvf1...`) into the read-back card. The server
checks that the key covers the shield's u1, scans from the block that holds the payout into a throwaway database,
returns the decrypted value and deletes the database. The key is not stored or logged and cannot spend. The honest cost:
for that one request the SHORE server could see the wallet's incoming notes, so a cautious user can just open the wallet
instead. A fully in-browser scan (WebZjs) would remove that and is a next step.

## Do a live run yourself (kamal)

You need: Phantom with at least 0.00133334 ZEC on Solana (0.0014 suggested; the cap is 0.02) and about 0.003 SOL, and a
Zcash wallet (Zashi/Zodl or Ywallet) for a fresh receiving u1; for the in-page read-back, that wallet's viewing key.

1. `scripts/live-run.sh check <your Phantom address>` prints both balances and READY or what is missing.
2. Open https://shore-zec.pages.dev/shore/?live=1, connect Phantom, paste a new u1 from your Zcash wallet, enter 0.0014.
3. Get quote, read the fees, tick the mainnet box, approve once in Phantom.
4. Watch the crossing (about 90 seconds last time). `scripts/live-run.sh status <deposit address>` shows the same 1Click status.
5. Paste your viewing key in the read-back card, or open your Zcash wallet.

## Who it is for

People who were paid in ZEC on Solana and never held ZEC before.

- Raydium's CMO, Zcash forum: about 84,000 Solana wallets received ZEC (about 8,700 ZEC) from a ZEC-paired
  reward token; 38,000 of them had never held ZEC on Solana; about half the ZEC is still sitting in those
  wallets. https://forum.zcashcommunity.com/t/38-000-new-zec-holders-on-solana-in-six-weeks-now-what/57788
- Same thread: almost no one bridged; burns from these wallets total 78 ZEC.
  https://forum.zcashcommunity.com/t/57788
- Dune, ZEC distributed as rewards: https://dune.com/andrayondune/zec-distributed-rewards
- Zooko pointed those holders at the ZecHub HOWTO: https://x.com/zooko/status/2105044462397485491
- ZecHub guide "Got ZEC on Solana? Move it to shielded Zcash": https://zechub.wiki/using-zcash/solana-zec-to-shielded
  (PR https://github.com/ZecHub/zechub/pull/2245: "solswap has a Withdraw page too, but it didn't work for us").
  Today's path is 2 sites, 2+ approvals and about 8 minutes; the guide author's run took 507 s.
- The ecosystem pays for this onboarding: ZecHub bounty https://bounties.zechub.wiki/bounty/cmuishtwo0010cvegb8eexpye,
  zkSNARKs "zero to first shielded transaction" bounty https://x.com/zksnarks_/status/2104718302606205130

## Mainnet proof (2026-10-10, recorded in `web/shore/replay.json`)

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

## The receipt, explained

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

## Invite links

`/shore/?invite=<code>&amount=<ZEC>&note=<text>` lets an onboarder or bounty host share the flow. The link opens
the live flow with the amount pre-filled and the sender's note shown as plain text, labelled as the sender's words.
Nothing is stored and nothing is sent until the user connects Phantom and approves. No custody, no new trust.

- Code: `[A-Za-z0-9_-]`, up to 32 characters, shown only as a label and never looked up. An empty or unrecognized
  code still opens the flow as "Invite link".
- Amount: a suggestion the user can change. Below the 1Click minimum (0.00133334 ZEC) it is raised to it, above the
  launch cap (0.02 ZEC) it is lowered to it, and anything that is not a plain decimal is ignored; the banner says which.
- Note: rendered as text, never HTML; control, zero-width and bidi characters removed, whitespace collapsed, 140 characters.
- A query string over 1,024 characters is ignored as a whole.
- Add `&replay=1` for the judge preview: same banner, then the recorded run, no mainnet.

## Architecture

```
browser (Cloudflare Pages: shore-zec)       Fly: airlock-zec (Rust, axum)            outside
  web/shore  ── /api/shield/address ──────▶  src/shore.rs  ── Tor ──▶ 1Click quote/status
             ── /api/quote {id,amount} ───▶   Store trait  ── HTTPS ─▶ Pages Function /store ─▶ D1
             ── /api/rpc (4 read methods) ─▶   RPC proxy    ── HTTPS ─▶ Solana RPC (key server-side)
             ── /api/readback, /exposure ──▶   UFVK scan    ─────────▶ lightwalletd
  Phantom signs one SPL transfer to the 1Click deposit address
```

- Sessions, the ua-index counter and the used-address list live in Cloudflare D1 behind a token-protected Pages
  Function (`deploy/shore-store`); the index advances atomically, so it stays strictly increasing across machines.
- The Solana RPC key is only a Fly secret; the proxy allows `getTokenAccountsByOwner`, `getLatestBlockhash`,
  `getSignatureStatuses`, `getBalance`, nothing that writes.
  Balances are cached 8s per owner and blockhashes 2s; 429/5xx answers are retried with backoff (300ms, 900ms),
  then the public mainnet endpoint is tried once (`SHORE_SOLANA_RPC_FALLBACK`, `off` to disable). Errors never carry the provider URL.
- Contract: `docs/API.md`. Design record: `docs/design/`.

## Run and test

```
cargo test                                  # Rust, incl. tests/shore_api.rs
cargo run -- serve                          # API on 127.0.0.1:8787 (needs a local wallet in work/)
cd web && npm i && npx playwright test      # 20 browser tests, desktop + mobile
PW_BASE_URL=https://shore-zec.pages.dev npx playwright test           # against production
SHORE_LIVE_API=1 SHORE_TEST_U1=<u1> npx playwright test -g "real API"  # dry quote through the real API
sh scripts/deploy.sh                        # Fly; bash deploy/pages/build.sh then wrangler pages deploy for Pages
```

## Built before vs during the hackathon

Tag `baseline-pre-hackathon` (8e11db3, 2026-10-05) is AIRLOCK as it stood: the Rust 1Click client, Tor
isolation, wallet and UFVK scanning, exposure receipts, the AIRLOCK web app and the Fly app.
Built during (`git diff baseline-pre-hackathon`): the SHORE HTTP API, pluggable store with the D1 Pages
Function, the Solana RPC proxy, `airlock ufvk`, the SHORE web flow (replay, live, invite links), its tests,
the mainnet run, and the design work.


---

SHORE is built on AIRLOCK. AIRLOCK's original README follows.

# AIRLOCK

Swap into and out of shielded ZEC through NEAR Intents with the smallest public footprint, and
get a receipt that proves it.

A read-only scan of 1,501 Zcash mainnet blocks found 909 live 1Click deposit addresses. 558 of them
show a shielded refund address to anyone who asks the status endpoint, and 120 recipients tie 306
swaps together (`spikes/leak-scan/FINDINGS.md`). AIRLOCK closes that.

Every exit from shielded ZEC crosses one transparent deposit address. AIRLOCK makes that crossing
as small and as unlinkable as the protocol allows: the amount is split into ZIP 318 canonical
denominations, every part gets its own deposit address, its own fresh refund address and its own
destination, every quote and status read leaves the service on its own Tor circuit, and a watchdog drives every part to a
terminal state. The exposure receipt then recomputes, from live reads, what a chain observer, the
1Click status endpoint and the solver can link, and counts the identifiers two parts share. The
target is zero, and `airlock audit` prints the number.

## What it does

**Inbound, any token to shielded ZEC.** An `EXACT_OUTPUT` quote whose recipient is a fresh
Orchard-only unified address. No unshield step exists. A view-only watcher (UFVK, never a spending
key) scans lightwalletd and reports the note with its txid and mined height.

**Outbound, shielded ZEC to another chain.** `split` cuts the amount into `{1,2,5} x 10^k`
denominations capped per hold. One non-dry quote per part, each over its own isolated Tor
circuit, each with its own refund UA and destination, and each carrying a deadline (120 minutes) and slippage (2%) chosen from the values seen across the sampled live swaps, because both are public in `/v0/status`. A ZIP 318 transfer-delay schedule (mean 66
blocks, cap 576) suggests when to send each part.

**Exposure receipt.** Per swap, a table of what each observer can read, built from the live
`/v0/status` payloads, plus an unfiltered comparison of every field across parts so nothing is
hidden by an allow-list. The observers it lists are the chain, the status endpoint, the solver and the
AIRLOCK service itself; `airlock serve` runs the service on your own machine to remove it from the picture.

**Exposure check for any deposit address.** Paste a one-time deposit t-address and see every field
the unauthenticated status endpoint returns for it.

**Watchdog.** Polls every part to SUCCESS, REFUNDED, FAILED or expired, escalates a funded part
that stalls past its hold timeout, and gates the next funding on the value already in flight.

## Run it

```sh
cargo build
airlock dry                       # dry quotes, 5 origin assets into one Orchard-only UA
airlock outbound-quote 1.5 0.5 <destAssetId> <dest1,dest2,dest3>
airlock audit <swap-id>           # live reads over isolated circuits, receipt, shared count
airlock inspect <depositAddress>  # what /v0/status reveals for any deposit t-address
airlock watch <swap-id>           # watchdog until every part is terminal
airlock inbound 0.01 <originAssetId> <originRefundAddress>
airlock note test|main <min-zat>  # view-only note read-back from lightwalletd
airlock fund <test|main> <dest> <zats>  # signed through COUNTERSIGN, mined read-back
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
- Four isolated Tor handles reach 1Click from four different exit IP addresses.
- A 200,000 zat Ironwood note sent to a fresh Orchard-only UA on testnet is found by the view-only
  watcher at txid `a37a3dbc29b5093aee3f016278321e8ab999d0f1be223d6fbf8abd511c24cd39`, mined at height
  4,435,638 and confirmed by an independent `GetTransaction`.
- Shielded ZEC to a transparent address of the kind a 1Click deposit uses, signed through COUNTERSIGN's
  transparent-exit intent, proved and broadcast by `airlock fund`: testnet txids
  `49046be46d4ef114b3497897eefdb20290186f19c7c838fb6483f805576aabdf` (height 4,435,995) and, after
  NU7 activation, `a2f1c4325d9f066f7f6464fe146c84662aa2c01e36f60bbe50d845bf40dea266` (height
  4,466,716), each read back through lightwalletd.
- Per-hold caps in code: no quote the service opens exceeds 0.02 ZEC inbound or 0.01 ZEC per
  outbound part (`src/server.rs`, `MAX_*_ZAT`, enforced in `src/swap.rs`), and mainnet sends from
  `airlock fund` need an operator switch and the same part cap. Each quote is checked against the
  request 1Click recorded before its deposit address is shown.
- The web app is live at https://airlock-zec.vercel.app on a static export with no serverless
  functions; its service runs at https://airlock-zec.fly.dev.
- Dry quotes for USDC on Solana, Base, Arbitrum and Ethereum, and BTC, all resolve into one
  Orchard-only unified address.

## Provenance

AIRLOCK is built in a clean repository started on 2026-10-03. It shares no code with earlier
projects. Dependencies are pinned to the NU7 generation of the Zcash crates (`zcash_protocol`
0.11.0-pre.0 and its matching releases).
