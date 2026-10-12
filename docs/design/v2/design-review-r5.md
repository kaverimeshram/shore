# SHORE first screen, design review round 5

Critic: fresh demo-judge (no prior rounds seen). Date 2026-10-12.
Evidence: r10-1440.png, r10-390.png, r9-390-full.png, plus fresh captures of http://127.0.0.1:8642/shore/ at 1440x2600 and `?replay=1&tour=1&speed=8` at 1440x900.

## Gate zero

- Verb the brief names: CWF Zcash track, "startups integrating ZEC into their products"; judge signal "utilizing ZEC the asset" that "could actually find some PMF".
- Code path that performs it: `web/shore/shore.js:309` `S.wallet.provider.signAndSendTransaction(tx)` (Phantom send to the 1Click deposit address); `web/shore/replay.json` is sourced from a recorded mainnet run on 2026-10-10 (sender ia3MLuk...Vsa5, deposit DMDdCzum...9SjbM, u1 recipient).
- What won last time: not re-verified in this review.
- VERDICT: ELIGIBLE. The product moves ZEC, it is not read-only.

## Scores

| Axis | Score | Evidence |
|---|---|---|
| Visual hierarchy | 8 | ~56px heavy three-line H1 left, receipt card right, both inside the 1440x900 fold. Eye goes headline, then highlighted public values. The "7 / 3" figure is the largest numeral but counts rows, not an outcome. |
| Typography | 7 | Grotesk display plus mono for addresses and who-can-read labels is the right pairing. Refusal explanations and mono labels render ~12px at 1440 and wrap awkwardly at 390 ("delivery tx has transparent / inputs"). Subhead is 54 words of 18px grey prose over 5 lines. |
| Color and semantic use | 8 | Color carries meaning: pink hazard fill = public value, red ANYONE, teal 1CLICK/NEAR, green NOBODY, black chip = enforced refusal, black bar = redaction. One stray: the blue CTA is the only colour with no semantic role. |
| Spacing and layout | 7 | 1440: left column ends at y~580, leaving ~260px of dead space beside the receipt. Receipt header crams 7/3 into the top-right corner with labels at ~13px. Redaction bar sits at the top of its row, not on the label baseline. |
| Component quality | 8 | Receipt rows are a real component: label, value chip, audience tag, guard codes on the row they protect. Footer line names the one private item. Below fold, the dark "Not started" card shows a 40px heading for an empty state, which wastes the strongest type on nothing. |
| Responsive at 390 | 7 | H1, subhead, both CTAs and the receipt header fit the first 844px; first receipt row visible. Value chip truncates to "contents" at 390 vs "value · recipient · memo" at 1440, losing the point of that row. Mono 11px labels wrap mid-phrase. |
| Copy and argument | 8 | "Every bridge says private. SHORE itemizes it." is a memorable, ownable claim, and the receipt beside it proves it. Weakened by two demand numbers that disagree: hero "86,000 Solana wallets received ZEC" vs Why-now "Raydium's CMO: 38,000 first-time ZEC holders". Business model paragraph ends "none has paid yet" and "switched off today", which reads as confession on a pitch surface. |
| AI-slop (10 = none) | 8 | Not a hero-plus-three-cards template. Remaining tells: duplicated "Watch the 84 s run" (nav and hero), status-pill-per-step stepper ("Waiting" x4) is a stock pattern. |

Average 7.6, min 7.

## Questions

1. Designed product or styled document? Designed product. The receipt is data set as data, with audience and guard encoded per row. The only document-like part is the 5-line subhead and the prose Why-now/Business blocks below.
2. Primary job obvious in 3 seconds? Half. "Bridges leak, SHORE itemizes" lands in 3 seconds. "Move your Phantom ZEC into a shielded note" needs the subhead. And the flow's primary button is "Connect recorded wallet" while "Use it live (mainnet)" is secondary, so a visitor who clicks "Shield my ZEC" lands in a replay, not the verb.
3. Would a competent practitioner produce exactly this without thinking about the content? No. Guard codes pinned to the row they protect, audience labels per link, and a redaction bar as the single private item only come from knowing the 1Click /v0/status leak. The stepper and dark status card below the fold are the generic part.

## Three highest-leverage fixes

1. Flow card buttons (below fold, "Connect Phantom"): make "Shield my ZEC (mainnet)" the primary filled button and demote "Connect recorded wallet" to the outline. Hero "Shield my ZEC" must arrive at the live verb; the replay already has its own "Watch the 84 s run" entry.
2. Receipt header "7 links itemized / 3 guards enforced": replace or lead with the outcome read back from Zcash, e.g. "115,000 zat landed · read-back at block <h>" with the viewing-key check as a green status line. The receipt currently proves exposure but the first viewport never shows the note landed, which is half the promise in the subhead.
3. Left column dead space at 1440 and the conflicting demand numbers: cut the subhead to two lines (one approval, Phantom ZEC to shielded note, itemized receipt), and use the freed space for one reconciled demand figure with its source. Pick 86,000 or 38,000, label exactly what each counts, and remove "none has paid yet" / "switched off today" from the Business model block (reframe as "fee line ships in the quote via 1Click appFees").

Single highest-leverage fix: route "Shield my ZEC" to a live-mainnet primary button, so the CTA performs the verb instead of a replay.
