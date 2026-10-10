# SHORE demo walkthrough (2:50)

Shot list for the screen recording. One browser window, 1440 wide, on https://shore-zec.pages.dev/shore/ .
The page opens in replay mode by default: the recorded mainnet run of 2026-10-10 (`web/shore/replay.json`),
played back 6x faster, so the 84 second crossing takes about 14 seconds on screen. Nothing is signed or sent
in replay. Every number below is from README.md or replay.json.

Notes for the recorder:
- The site has no button labelled "Try it in 60s" (checked at commit 7f03498). The replay that opens by default
  is that path. If such a button ships before recording, click it in beat 1 instead.
- The site has no on-page demand section. Beat 2 uses a second tab with the Zcash forum thread.
- Keep the mode badge ("Replay of a mainnet run") visible in the header whenever possible, so no one mistakes
  the replay for a live send.
- Second tab to preload: https://forum.zcashcommunity.com/t/38-000-new-zec-holders-on-solana-in-six-weeks-now-what/57788
- Third tab to preload (beat 10): https://shore-zec.pages.dev/shore/?invite=zechub&amount=0.005&note=Welcome%20to%20shielded%20ZEC

| # | Time | On screen (action) | Narration |
|---|------|--------------------|-----------|
| 1 | 0:00 to 0:15 | **Landing.** Load https://shore-zec.pages.dev/shore/ . Hold on the headline "Got ZEC in Phantom from rewards? Shield it in one approval." Cursor rests on the mode badge "Replay of a mainnet run", then on the "84s, last mainnet run, 115,000 zat landed" line in the crossing tracker. | "This is SHORE. If you got ZEC in Phantom from rewards, it moves that ZEC into a shielded Zcash balance with one approval. What you're watching is a replay of a real mainnet run." |
| 2 | 0:15 to 0:35 | **Demand.** Switch to the forum tab. Scroll to the Raydium CMO's numbers. Then back to SHORE. | "Raydium's CMO counted about 84,000 Solana wallets that got ZEC from a reward token. 38,000 had never held ZEC. About half that ZEC is still sitting there, and almost no one bridged. Today's guide takes two sites, two or more approvals and about eight minutes." |
| 3 | 0:35 to 0:50 | **Connect.** Click "Connect recorded wallet". The step closes into a receipt: "0.0014 ZEC on Solana" with the shortened wallet link. The ledger hero shows "Balance on Solana". | "Step one, connect Phantom. It shares the address and the ZEC balance. Nothing is signed yet. Here it's the recorded wallet, holding 0.0014 ZEC." |
| 4 | 0:50 to 1:05 | **Fresh u1.** Click "Make a fresh u1". Receipt shows the shortened u1 address, "Orchard receiver, never used before". Hover the help line about reuse. | "Step two, a fresh shielded address. Every shield gets its own u1, because reusing one would link this Phantom wallet to every shield sent there. On mainnet you paste a fresh address from your own Zcash wallet, like Zashi or Ywallet." |
| 5 | 1:05 to 1:25 | **Quote with fees.** Amount is fixed at 0.0014. Click "Get quote". Hold on "0.0014 → 0.00108 ZEC", the fee percentage and the line "Most of it is the fixed 0.00032 ZEC Zcash withdraw fee". Open "Fee breakdown (6)" for two seconds. | "Step three, NEAR Intents 1Click prices the hop. 0.0014 in, 0.00108 quoted out. About $1.70 in, $1.31 out, so roughly 39 cents. Most of that is a fixed withdraw fee, so the percentage falls fast as the amount grows. Every fee is shown before you approve." |
| 6 | 1:25 to 1:40 | **One approval.** Show the review card: one SPL transfer to the 1Click deposit address, refunds go to your wallet. Click "Approve in Phantom (replay)". "Approved (simulated, nothing is sent)." appears, then the receipt "Signed" with the Solana tx link. | "Step four. Phantom shows one transfer. You approve it once. That's the only signature in the whole flow." |
| 7 | 1:40 to 1:55 | **The crossing tide band.** Let the tide band fill across Solana, Bridge (NEAR Intents), Shielded (Zcash). The elapsed counter runs (recorded time, replayed faster) and the step reads "Crossing now. Nothing else to sign." | "Now it crosses: Solana, through NEAR Intents, into Zcash. Nothing else to sign. On mainnet this took 84 seconds from send to land." |
| 8 | 1:55 to 2:15 | **Read-back card.** The read-back card counts up to 115,000 zat: "Shielded, read back from Zcash". Point at the height 3,512,504 link, "Orchard receiver, Ironwood pool (v6 transaction)" and the +7,000 zat difference from the quote. | "This is the moment. SHORE reads the note back from Zcash with a view-only key, no spending key. 115,000 zat landed in the Ironwood pool. 1Click's status said 108,000. The decrypted note holds 7,000 more, so SHORE shows what actually landed." |
| 9 | 2:15 to 2:35 | **What's still public.** Open the "What's still public" panel. Show the rows: the hop in is on Solana, 1Click status links your Phantom address to your u1, NEAR can decrypt the payout. Then the "Counterparty risk" paragraph. | "And it's honest about what stays public. The hop in is on Solana. 1Click's status links your Phantom address to that one u1, which is why every shield gets a fresh address. The solver network can delay, refund or freeze funds in flight. What's private is everything after: spends from the note." |
| 10 | 2:35 to 2:50 | **Invite link.** Switch to the invite tab. Show the URL bar, then the banner: "Invite zechub", "The sender wrote: ...", "Suggested amount: 0.005 ZEC. You can change it." Do not connect (this opens the live mainnet flow). End on the SHORE wordmark. | "Onboarders and bounty hosts can share an invite link with a suggested amount and a note. Nothing is stored and nothing is sent until the user approves. SHORE: one approval from Phantom to shielded Zcash." |

Total: 2:50.

## Facts used (source)

- 84,000 wallets, 38,000 first-time holders, about half the ZEC still there, almost no one bridged: README, Raydium CMO forum thread.
- Today's path 2 sites, 2+ approvals, about 8 minutes: README (ZecHub guide).
- 0.0014 ZEC in, 108,000 zat quoted, 115,000 zat landed, +7,000 zat, height 3,512,504, Ironwood, v6, Orchard receiver, 84 s, $1.70 in, $1.31 out, about 39 cents, fixed 0.00032 ZEC withdraw fee: README and replay.json.
- Replay speed 6x: `web/shore/api.js`.
- Invite links (suggested amount 0 to 0.02 ZEC, note as plain text, nothing stored, opens the live flow): README and `web/shore/shore.js`.
