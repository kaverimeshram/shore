# SHORE pitch script (about 2:00 spoken)

For the Colosseum brief: "Tell us what you're building, and tell us why you're the people to build it."
Spoken by Kamal, first person. Only the quoted lines are spoken; the headings are cues.

## 0:00 to 0:10, what it is

> Hi, I'm Kamal. I'm building SHORE. If you got ZEC in Phantom from rewards, SHORE moves it into
> shielded Zcash with one approval.

## 0:10 to 0:35, who it's for and the problem

> Raydium's CMO says about 84,000 Solana wallets got ZEC from a reward token, and 38,000 of them had
> never held ZEC before. About half of that ZEC is still sitting there, and almost no one bridged it.
> The guide for shielding it, from ZecHub, takes two sites, two or more approvals and about eight minutes.

## 0:35 to 1:00, what I've proved on mainnet

> So I made it one step. You connect Phantom, paste a fresh address from your Zcash wallet, see every
> fee, and sign once. On October 10th I ran it on mainnet. 84 seconds after I signed, 115,000 zat landed
> in the shielded pool. SHORE read the note back with a view-only key, and it held 7,000 zat more than
> 1Click's status said. It cost about 39 cents.

## 1:00 to 1:30, the honest trade-offs

> Here's what it doesn't hide. The hop in is public on Solana, and 1Click's status links your Phantom
> address to that one shielded address. That's why every shield gets a fresh one. The route runs through
> NEAR Intents, which once held about $589,000 for 50 days. SHORE never holds funds, but it can't overrule
> them. The app says so before you sign.

## 1:30 to 2:00, why we understand this problem

<!-- source: spikes/leak-scan/FINDINGS.md, commit 3c1eed2 (2026-10-03); README AIRLOCK section -->
> Why we understand this problem: before SHORE, we scanned 1,501 Zcash mainnet blocks and found 909 live
> 1Click deposit addresses. 558 of them leaked a shielded refund address. That's why SHORE asks for a fresh
> address every time and shows what stays public before you sign.
<!-- source: README "Mainnet proof" (2026-10-10 run: 84 s, 115,000 zat, read back with a view-only key) -->
> And we don't trust a status page: on our own mainnet run, we read the note back ourselves, and it didn't
> match what 1Click reported. The fee is built but switched off, and no one has paid yet. Try the replay at
> shore-zec.pages.dev/shore.

---

## Where each claim comes from

- One approval, Phantom, rewards holders, fresh u1 from your own Zcash wallet, every fee shown: README intro,
  "For judges", "What's still public".
- 84,000 wallets, a ZEC-paired reward token, 38,000 first-time holders, about half still there, almost no one
  bridged (Raydium's CMO, Zcash forum): README "Who it is for".
- ZecHub guide: 2 sites, 2+ approvals, about 8 minutes: README "Who it is for".
- 2026-10-10 mainnet run, 84 s, 115,000 zat in the Ironwood pool, view-only key, 1Click said 108,000 so 7,000
  more, about 39 cents: README "Mainnet proof".
- Hop in public, status links the Phantom address to that one u1, fresh address per shield, about $589,000 held
  by NEAR Intents for 50 days, SHORE never holds funds but cannot overrule 1Click: README "What's still public".
  "Before you sign": the "What's still public" panel and the counterparty-risk text on the production site.
- AIRLOCK from 2026-10-03 (first commit 322da53, "1Click client and exposure analyzer"), Rust 1Click client,
  Tor isolation, UFVK scanning: git history and README "Built before vs during the hackathon".
- 909 live 1Click deposit addresses, 558 exposing a shielded refund address: README (AIRLOCK section) and
  `spikes/leak-scan/FINDINGS.md`, commit 3c1eed2 (2026-10-03).
- Production feature audit (2026-10-11 10:30 IST), fee built but switched off, none has paid yet: README
  "Feature status" and "Business model".
- Why-us uses only measured, product-specific facts; no awards or prizes are claimed.
- No personal background is claimed beyond what the work and git history show. No team size is mentioned.
