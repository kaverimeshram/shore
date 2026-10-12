# SHORE hostile-judge pass (2026-10-12, 11:20 to 11:40 IST)

Walked https://shore-zec.pages.dev/shore/ as a skeptical judge at 1440 and 390: the judge path
`?replay=1&tour=1&speed=3`, and `?live=1` with a stand-in Phantom (wallet `ia3M…Vsa5`, real balance, nothing signed).
Then audited every number in README, the UI, PITCH-SCRIPT, DEMO-SCRIPT, FORM and JUDGE-CHECKLIST against its source,
and every holder-facing error path. Pre-fix shots: `docs/submission/hostile-judge/`.

## Findings

| # | Finding | Severity | Fix | Evidence |
|---|---|---|---|---|
| 1 | README: "the guide author's run took 507 s". The PR's own test shows the withdrawal taking 7 min 57 s (477 s); no source gives 507. | High (wrong number) | Replaced with the PR's timestamps: signed 15:59:28, Zcash block 3,498,141 at 16:07:25. | https://github.com/ZecHub/zechub/pull/2245 |
| 2 | UI and replay data cited `shore-live-run.json` as the source. That file is not in the repo or reachable by a judge. | High (unsourced) | The UI's Source note now points to the public Solana, NEAR and Zcash explorer links shown above it; `replay.json` points to `tests/fixtures/shore-live/`. README "Mainnet proof" now gives the raw fixture values and block times. | `tests/fixtures/shore-live/quote.json`, `status-final.json` |
| 3 | "What's still public" said only "1Click/NEAR" can read the payout note. L2Beat says payouts use an all-zero outgoing viewing key, so anyone can decrypt them, and the README already said so. | High (understated exposure) | Row and help text now say anyone can decrypt the payout address and amount (L2Beat). Replay/client rows only: the Fly API's live row still has the old text and needs a Fly redeploy (blocked by the machine limit). | https://l2beat.com/privacy/projects/zcash-near-intents |
| 4 | Live mode, address step: the hint said "A new Orchard-only u1 address is derived for this transfer", but live mode never derives one. | Medium (wrong state) | Hint now reads "Every shield goes to a shielded address (u1…) that has never been used before." | `web/shore/index.html` |
| 5 | Live: the real wallet holds 0.00004571 ZEC, below the 1Click minimum, and nothing said so until the quote. | Medium | The connect step now says the balance is below the minimum and how much ZEC to add. | live walk |
| 6 | Replay: "Confirmed … 10 Oct, 09:24 IST" next to "landed in 84s" after a 09:22 send, so it looked like 2 minutes. The block is at 09:23:39. | Medium (wrong number) | Times now 09:22:15 and 09:23:39 IST. | Solana RPC blockTime 03:52:15 UTC; Blockchair block 3,512,504 at 03:53:39 UTC |
| 7 | Error copy came straight from the API in lowercase jargon, with no next step: 409 reuse, BAD_UFVK, KEY_MISMATCH, below minimum, above cap, NOT_UNIFIED. Below minimum and above cap also only failed after a round trip to the API. | Medium | Plain-words copy with a next step for every code; the code stays in brackets. Minimum, cap and balance are now checked before any quote request. | `tests/errors.spec.js` |
| 8 | "More than this wallet holds." gave no balance and no next step. | Low | Now states the balance, the amount asked and what to do. | `tests/errors.spec.js` |
| 9 | "7000 zat more than 1Click's status amountOut": no thousands separator, and API jargon. | Low | "7,000 zat more than 1Click's status said would land (amountOut)". | UI |
| 10 | "0.00115ZEC" missing a space; the hero label "Confirmed, send to land" was confusing. | Low | Space added; label now "Landed, send to land". | UI |
| 11 | README demand numbers are from 20 Sep; the same thread updated them on 24 Sep (86k, 39k, 53% still hold). | Low | README notes the date and the update. The pitch's "about half" still holds. | forum t/57788 |
| 12 | README live-run step said "about 90 seconds last time"; the run was 84 s. | Low | Now "84 seconds last time; 1Click quotes about 135". | `replay.json` |

## Claims audit (all now sourced)

| Claim | Where | Source, checked 2026-10-12 |
|---|---|---|
| 84,000 wallets, about 8,700 ZEC, 38,000 first-time, about half still held, almost none bridged (78 ZEC burned) | README, UI, pitch, demo | Raydium CMO, forum t/57788 (20 Sep and 24 Sep posts) |
| 2 sites, 2+ approvals, about 8 minutes | README, UI, pitch, demo | ZecHub PR #2245 (solswap deposit, near.com withdraw, Sign Message; withdrawal 15:59:28 to 16:07:25) |
| Zooko pointed holders at the HOWTO; zkSNARKs onboarding bounty | README, UI | x.com/zooko/status/2105044462397485491, x.com/zksnarks_/status/2104718302606205130 |
| 84 s send to land | all | Solana tx 2LLVDBnV… blockTime 03:52:15 UTC; Zcash block 3,512,504 at 03:53:39 UTC |
| 115,000 zat landed, 108,000 quoted, 7,000 more | all | Zcash tx fa8d4d6b…: 7,960,000 in, 7,830,000 change, 130,000 into Ironwood plus fee; `status-final.json` amountOut 108000; read-back via UFVK |
| $1.70 in, $1.31 out, about 39 cents, 0.00032 ZEC withdraw fee, 22.9% | README, FORM, pitch, demo, UI | `quote.json`: amountInUsd 1.6974, amountOutUsd 1.3094, withdrawFee 32000 of 140000 = 22.86% |
| 1Click minimum 0.00133334 ZEC | all | `quote-too-small.json`: "try at least 133334"; `MIN_AMOUNT` in `src/shore.rs` |
| 0.02 ZEC launch cap | all | `MAX_INBOUND_ZAT = 2_000_000` in `src/server.rs`, enforced by the API |
| 909 live deposit addresses, 558 leak a shielded refund address, 1,501 blocks | README, pitch | `spikes/leak-scan/FINDINGS.md` (commit 3c1eed2) |
| $589,000 USDT held 50 days | README, UI, pitch, demo, FORM, checklist | forum t/57497 (8 Sep 2026) |
| Payouts decryptable by anyone | README, UI | L2Beat zcash-near-intents |

## Error paths (production, plain words plus next step, all tested)

Bad u1, transparent address, Sapling address, reused address (client-side and server 409), below minimum, above cap,
balance too low, BAD_UFVK (client-side and server), KEY_MISMATCH: `web/tests/errors.spec.js` and `web/tests/shore.spec.js`.

## Result

Deployed to production Pages (shore-zec, deploy c7429780) at 11:34 IST. Full Playwright suite on
PW_BASE_URL=https://shore-zec.pages.dev: 43 passed, 7 skipped, 0 failed, overlap at 1440/768/390 included.
Open: the live exposure row from the Fly API (finding 3) changes only after a Fly redeploy, which is blocked by the
machine limit.
