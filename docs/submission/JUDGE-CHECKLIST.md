# SHORE judge checklist

Colosseum World's Fair, Zcash track. Deadline Tue 2026-10-13 12:29 IST. Status as of 2026-10-11 01:30 IST.

## Links
- Live app: https://shore-zec.pages.dev/shore/ (opens on a replay of the real mainnet run; `?live=1` for mainnet, capped at 0.02 ZEC)
- API: https://airlock-zec.fly.dev (Fly, one machine, no volume; D1 store)
- 60s judge path: https://shore-zec.pages.dev/shore/?replay=1&tour=1&speed=3 (button "Try it in 60s"; hands-free, no wallet, nothing sent)
- Invite preview: https://shore-zec.pages.dev/shore/?invite=zechub&amount=0.005&note=Welcome&replay=1
  (banner marked "Judge preview", then the recorded run; verified 2026-10-11 01:26 IST with zero off-origin requests,
  screenshot `docs/submission/invite-preview-run.png`, script `web/scripts/invite-preview-check.mjs`)
- Demo video: `docs/pitch/demo-capture.mp4` (and `.webm`), 1440x900, 2:39, no narration yet
- Pitch script: `docs/pitch/pitch-script.md`; demo walkthrough: `docs/pitch/demo-walkthrough.md`
- Repo / code: `/Users/kamal/Desktop/solana/cwf/zcash/projects/airlock` on kamal's Mac. API `src/shore.rs`, app `web/shore/`,
  tests `tests/shore_api.rs`, `tests/rpc_proxy.rs`, `web/tests/`. README.md has "For judges".

## Business model
Holders pay a small, transparent SHORE fee on each shield, set through the 1Click `appFees` field and shown as its own
line in the quote (today it is zero). Wallets, communities and bounty programs pay for invite links that bring their
members through SHORE in one approval. SHORE never holds funds, so the fee is the whole business.

## Built before vs during
- Tag `baseline-pre-hackathon` (8e11db3, 2026-10-05): AIRLOCK (Rust 1Click client, Tor isolation, wallet and UFVK
  scanning, exposure receipts, AIRLOCK web app, Fly app).
- Built during (`git diff baseline-pre-hackathon`): SHORE API, D1 store, hardened Solana RPC proxy, SHORE web flow
  (replay, live, invite links, judge tour), tests, mainnet run, design. About 1,310 lines in `src/` (921) and `tests/`
  (393); about 1,480 in the web flow, browser tests, scripts and deploy files.

## Mainnet proof (2026-10-10)
- Solana, one Phantom approval, 0.0014 ZEC, 09:22:15 IST: https://solscan.io/tx/2LLVDBnVAqnG99PSbUVqapKqZFHjPPET9HsJhV2tqS29BHR4wNmUzYZbQzLf2BXjpFKMrRJCYxbVZq6ZJNQQ86au
- NEAR Intents 1Click: https://nearblocks.io/txns/BARJAuxGT1tXY8XAK9LsUCroQsA3V1AgErm9DhtkqC53
- Zcash shielded note, height 3,512,504, 09:23:39 IST: https://mainnet.zcashexplorer.app/transactions/fa8d4d6bd210c6150e98142ec1ae9a25706ec19bd591a775823f6616f95fa408
- 84 s send to land; 115,000 zat landed (Ironwood, Orchard receiver), read back with a viewing key; about 39 cents cost.

## Honest risks
- The hop in is public on Solana (address, deposit, amount, time).
- The route is linkable: 1Click's unauthenticated status maps deposit to recipient u1, and NEAR Intents Zcash payouts
  are decryptable (L2Beat). A fresh u1 per shield limits the link to that one shield.
- Counterparty risk: the solver network can delay, refund or freeze funds (precedent: about $589,000 held for 50 days).
- The user needs a Zcash wallet (Zashi/Zodl, Ywallet). Launch cap 0.02 ZEC; 1Click minimum 0.00133334 ZEC.

## Tests (2026-10-11)
- Rust: all green, including 4 RPC proxy tests (cache, backoff, fallback, allowlist).
- Playwright against production: 28 passed, 4 skipped (opt-in live-API tests; live dry flow passed 2/2 on 2026-10-10).

## Open items (need kamal)
- [ ] Narration for the demo video (script in `docs/pitch/demo-walkthrough.md`).
- [ ] X posts (`docs/pitch/x-posts.md`): drafts, awaiting kamal approval. Nothing posted.
- [ ] Outreach (`docs/pitch/outreach.md`: @zksnarks_, forum t/57788, holder communities): drafts, awaiting kamal approval. Nothing sent.
- [ ] Confirm the team is listed as solo on the submission form.
- [ ] Post-deadline: shielded receive in the browser (noted, not built).
