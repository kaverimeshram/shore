# SHORE demo script (2:58)

Shot by shot, on the current production site https://shore-zec.pages.dev/shore/ . Spoken by Kamal.
Three parts: the judge tour (`?replay=1&tour=1&speed=3`), the invite preview, and `?live=1` up to the quote only.

Before recording:
- Tabs, in order: (A) https://shore-zec.pages.dev/shore/ , (B) https://shore-zec.pages.dev/shore/?invite=zechub&amount=0.005&note=Welcome&replay=1 ,
  (C) https://shore-zec.pages.dev/shore/?live=1 .
- For tab C: Phantom with at least 0.00133334 ZEC on Solana (0.0014 suggested; the page refuses an amount above the
  wallet's balance), and a new receiving u1 copied from your own Zcash wallet (Zashi/Zodl or Ywallet). Getting a live
  quote opens a real 1Click deposit address but moves nothing. Treat that u1 as used afterwards.
- In tab C, never tick the mainnet box and never press Approve. The take stops at the quote.
- The tour presses each button by itself about every 2.2 s, and at speed 3 the 84 s crossing plays in about 28 s.
  Times inside shot 3 follow the tour; just talk over it.
- Labels below are from production on 2026-10-12. If a redesign ships before you record, check the labels first.

| # | Time | On screen | Click | Said |
|---|------|-----------|-------|------|
| 1 | 0:00 to 0:14 | Tab A, the landing. Headline "Got ZEC in Phantom from rewards? Shield it in one approval.", the mode badge "Replay of a mainnet run", the tracker line "84s, last mainnet run, 115,000 zat landed". | Nothing. Cursor rests on the headline, then on the 84s line. | "This is SHORE. If you got ZEC in Phantom from rewards, it moves it into shielded Zcash with one approval. Everything you see first is a replay of my real mainnet run. Nothing is sent." |
| 2 | 0:14 to 0:27 | Scroll down to "Why now" and "Business model", then back to the top. | Scroll only. | "Why now: 38,000 first-time ZEC holders on Solana, almost none bridged, and today's path is two sites, two or more approvals, about eight minutes." |
| 3a | 0:27 to 0:36 | The tour runs: "Connect recorded wallet" closes into "0.0014 ZEC on Solana"; "Make a fresh u1" shows the u1 with "Orchard receiver, never used before"; the quote appears. | **Try it in 60s** (the tour does the rest). | "I'll press Try it in 60s. It connects the recorded wallet, makes a fresh shielded address, and asks NEAR Intents 1Click for a quote." |
| 3b | 0:36 to 0:40 | "Approved (simulated, nothing is sent)." then the "Signed" receipt with the Solana link. | Nothing. | "Then one approval. That's the only signature." |
| 3c | 0:40 to 1:08 | The tide band fills from Solana (Phantom) through Bridge (NEAR Intents) to Shielded (Zcash). The clock counts ("recorded time, replayed faster"). "Crossing now. Nothing else to sign." | Nothing. | "Now it crosses: Solana, through NEAR Intents, into Zcash. On mainnet, on October 10th, this took 84 seconds from send to land. Here it plays three times faster, from the recorded timestamps. There's nothing else to sign, and SHORE never holds the funds." |
| 3d | 1:08 to 1:14 | "Landed on Zcash." The read-back card counts up to 115,000 zat; then the tour opens "What's still public". | Nothing. | "And it landed." |
| 4 | 1:14 to 1:38 | Scroll up to the read-back card: 115,000 zat, height 3,512,504, "Orchard receiver, Ironwood pool (v6 transaction)", the difference from the 108,000 zat quote. Then the quote card: "0.0014 → 0.00108 ZEC", the fixed withdraw fee line. | Open **Fee breakdown (6)** in the quote card. Point at "SHORE fee: none today (0%)". | "This is the part I care about. SHORE reads the note back from Zcash with a view-only key. 115,000 zat landed in the Ironwood pool. 1Click's status said 108,000, so the note holds 7,000 more, and SHORE shows what actually landed. About $1.70 in, $1.31 out, so about 39 cents, mostly 1Click's fixed withdraw fee. The SHORE fee line is there and set to zero today." |
| 5 | 1:38 to 1:58 | Scroll down to the open "What's still public" panel and the "Counterparty risk" text. | Nothing. | "And this is what stays public. The hop in is on Solana. 1Click's status links your Phantom address to that one u1, so every shield gets a fresh one. The solver network can delay, refund or freeze funds in flight; NEAR Intents once held about $589,000 for 50 days. What stays private is everything after: spends from the note." |
| 6 | 1:58 to 2:13 | Tab B. The URL bar with `?invite=zechub&amount=0.005&note=Welcome&replay=1`, then the banner: "Invite zechub", "The sender wrote: Welcome", "Suggested amount: 0.005 ZEC", "Judge preview: this plays the recorded mainnet run. No wallet, nothing sent." The recorded run starts playing below; switch tabs before it finishes. | Nothing. | "Onboarders and bounty hosts can share an invite link with a suggested amount and a note. This is the judge preview: the same banner a holder sees, then the recording. Nothing is stored and nothing is sent until the holder approves." |
| 7a | 2:13 to 2:25 | Tab C. Mode badge "Live on mainnet". | **Connect Phantom**, approve the connection in Phantom. The receipt shows your real ZEC balance on Solana. | "Now live, on mainnet. Connecting only shares my address and balance. Nothing is signed." |
| 7b | 2:25 to 2:35 | The address step says to paste a new receiving address from your own Zcash wallet. No fresh-u1 button in live mode. | Paste the u1, press **Use this address**. | "In live mode there's no demo wallet. The ZEC goes only to an address from my own Zcash wallet." |
| 7c | 2:35 to 2:50 | The amount field, then the live quote: you send, lands quoted, the fee percentage, about how many seconds, the expiry time in IST. | Type **0.0014**, press **Get quote**, open **Fee breakdown (6)**. Stop here. Do not tick the box, do not approve. | "A real 1Click quote, with every fee before I sign, including the SHORE fee at zero. I'll stop here, because the next click spends real ZEC." |
| 8 | 2:50 to 2:58 | Hold on the quote card, then the SHORE wordmark. | Nothing. | "That's SHORE: one approval from Phantom to shielded Zcash. Try it at shore-zec.pages.dev/shore." |

Total: 2:58.

## Claims checked against README.md

- One approval, rewards holders in Phantom, replay of the 2026-10-10 mainnet run, "Try it in 60s" (`?replay=1&tour=1&speed=3`), nothing sent: intro, "For judges".
- 38,000 first-time holders, almost none bridged, 2 sites, 2+ approvals, about 8 minutes: "Who it is for".
- 84 s, 115,000 zat, Ironwood pool, v6, Orchard receiver, height 3,512,504, view-only key, 108,000 quoted so 7,000 more, $1.70 in and $1.31 out, about 39 cents, fixed withdraw fee: "Mainnet proof".
- SHORE fee built in and switched off today: "Business model", "Feature status".
- Hop in public, status links the Phantom address to that one u1, fresh address per shield, freeze risk, about $589,000 held 50 days, spends from the note stay private, SHORE never holds funds: "What's still public".
- Invite preview `?invite=zechub&amount=0.005&note=Welcome&replay=1`, "Judge preview", nothing stored or sent: "For judges" and "Invite links".
- Live mode: own Zcash wallet's u1, never the demo wallet, 1Click minimum 0.00133334 ZEC, cap 0.02 ZEC, quote with every fee shown: "For judges", "What's still public", "Do a live run yourself".
