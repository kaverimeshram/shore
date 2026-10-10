# SHORE pitch script (2:00 spoken)

Zcash track, Colosseum Solana World's Fair. Speaker: kamal. Spoken text only is in the quote blocks;
the word count of the spoken text is in the footer. Screen during the pitch: the replay at
https://shore-zec.pages.dev/shore/ (see `demo-walkthrough.md` for the click path).

## 0:00 to 0:12, team

> I'm kamal, and I built SHORE solo. It sits on AIRLOCK, the Zcash toolkit I started on October 3rd:
> a NEAR Intents 1Click client, Tor isolation and view-only note scanning.

## 0:12 to 0:35, problem and data

> Raydium's CMO counted about 84,000 Solana wallets that got ZEC from a reward token. 38,000 had never
> held ZEC. About half of that ZEC is still sitting there, and almost no one bridged. The ZecHub guide
> to shield it takes two sites, two or more approvals and about eight minutes. The guide's own author
> took 507 seconds.

## 0:35 to 1:10, live demo, the aha moment

> With SHORE it's one Phantom approval. Connect, get a fresh shielded address, see the quote with every
> fee, approve once. On mainnet, 84 seconds later, 115,000 zat landed in the Ironwood pool. SHORE
> decrypted the note with a view-only key, and it shows 7,000 zat more than 1Click reported. Cost: about
> 39 cents, mostly a fixed withdraw fee. Then it tells you what's still public: the hop in, and the link
> from your Phantom address to that one address.

## 1:10 to 1:22, market

> Launchpads like Sapling, SnarksPad and Billet are where you speculate. SHORE is the way in. Every
> Zcash app needs these holders to arrive shielded first.

## 1:22 to 1:40, business model

> Holders pay a small SHORE fee on each shield, set through 1Click's app fee field and shown as its own
> line in the quote. Today it's zero. Wallets, communities and bounty programs like ZecHub pay for invite
> links that bring their members through in one approval.

## 1:40 to 2:00, why Zcash

> Why Zcash: on Solana, that ZEC sits next to your whole public history. Shielded, the spends from the
> note are private. The honest risks: NEAR Intents once held about $589,000 for 50 days, and you still
> need a Zcash wallet. Try the replay at shore-zec dot pages dot dev.

---

Spoken words: 286 (about 145 words per minute).

## Sources for every claim

- Solo builder: kamal's brief. AIRLOCK started 2026-10-03 and what it contains: README, "Provenance" and
  "Built before vs during the hackathon".
- 84,000 wallets, 38,000 first-time holders, about half still there, almost no one bridged: README
  (Raydium CMO, Zcash forum thread 57788).
- 2 sites, 2+ approvals, about 8 minutes, 507 s: README (ZecHub guide).
- 84 s, 115,000 zat, Ironwood, view-only key, +7,000 zat vs 1Click status, about 39 cents, fixed withdraw
  fee: README "Mainnet proof".
- What's public, the link from Phantom address to the u1, $589,000 held 50 days, need a Zcash wallet:
  README "What's still public".
- Launchpads line: README and the landing page lede.
- Bounties (ZecHub, zkSNARKs) and invite links for onboarders and bounty hosts: README.
- Business model (holder fee through 1Click `appFees`, zero today; wallets, communities and bounty programs
  pay for invite links): README "Business model" and the landing page "Business model" section.
