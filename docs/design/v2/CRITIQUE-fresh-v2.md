# CRITIQUE · SHORE ui-v2 (wise) · round 1 · critic=fresh

critic: fresh:grokbot-knowledgework (independent; builder critique and DESIGN.md NOT read before scoring)
date: 2026-10-11 22:33 IST · preview https://ui-v2.shore-zec.pages.dev/shore/
rubric: demo-judge 10 axes, 0-10; pass = average >= 8.0 and every axis >= 7. Report: docs/design/v2/design-review.md
evidence: docs/design/v2/after/*, before/*, docs/design/r4-*, BACKED v3 reference (hyperliquid/projects/backed/design/shots/after-1440.png),
plus my Playwright run at 1440 and 390 with a contrast audit (docs/design/v2/fresh/).

## Scores
SCORES: data-as-hero 6, density 7, liveness 8, interaction 7, disclosure 7, color 6, motion 7, mobile 6, tells 5, trust 7
AVERAGE: 6.6   MIN: 5   VERDICT: FAIL. This is a regression from Harbor r3 (7.7): more dashboard furniture, less product.

1. Data as hero 6: the metric cards and dark hero card update live, but they sit ABOVE the flow, so they are off-screen while the user acts. During crossing, no elapsed/progress number is visible anywhere (d-5-crossing.png, m-5-crossing.png). At landed on desktop, the 0.00115 read-back hero is clipped under the sticky header (d-6-landed.png). Idle shows last-run 84s twice and last-run metrics styled like live values.
2. Density 7: substance is up, but the right column is two placeholder stubs until landed, and pitch panels pad the page.
3. Liveness 8: chips per state, mined height, Copy pills, IST expiry, replay chip.
4. Interaction 7: one lime primary per step, Copy everywhere, 2px focus outline, post-sign Change links removed. The header lime "Try it in 60s" pill is a second primary on screen at nearly every scroll position.
5. Disclosure 7: fee breakdown and source collapsed. But an explainer panel and 4 metric cards push Connect below the fold at 1440 (≈y620) and ≈1.2 screens down at 390. "Why now / Business model" pitch content belongs in the deck, not the product.
6. Color 6: borrowed wholesale from Wise (#9FE870 / #163300 / #0E0F0C literal in tw.css). The Zcash identity is dropped. Lime does 7 jobs. Amber appears once. 3 AA failures (Readable-by rows 4.10:1 at 12px, footer 4.34:1, metric "–" 2.03:1).
7. Motion 7: the bar fills, chips change, tour mode works. The idle hero band at 35% opacity reads olive.
8. Mobile 6: 3,943px page. Explainer and pitch below the flow. No elapsed during crossing. "ZEC sent, last mainnet r…" truncation. "0.0014 / ZEC" wraps. Landed scroll is fine.
9. Tells 5: still reads as AI-generated, and specifically as "Wise template". The owner's r0 complaint is back: cards inside cards (explainer panel with 3 numbered tiles, which is also slop gate 3; Why-now card with 4 tiles; quote card with a nested send→lands panel and 2 bordered disclosures). Uppercase tracked micro-labels on every metric, a stock dark stat card, rounded-pill everything.
10. Trust 7: honest and receipted, but the wrong caption on the landed hero ("send to land, recorded on mainnet" under 0.00115 ZEC) and the clipped payoff undercut it.

## BACKED v3 direction check
Metric cards present but mis-placed. Panels present but nested. Status chips good. One primary: partial (header pill competes). Status color + secondary accent: partial and borrowed. Progress/chart polish: weak (flat bar, no ticks or time, no chart). Type scale coherent, 3 AA failures. Detail in design-review.md.

## Top fixes (leverage order)
1. Put the live state where the action is. Make the hero card (or a compact metric strip: lands, fee, elapsed) sticky beside or above the active step on desktop, and the sticky strip on mobile. Show elapsed seconds inside the Crossing card. Add scroll-margin-top equal to header height so the landed number is never clipped.
2. Flatten the nesting. The explainer becomes one line of text (or goes into the tour), Why-now/Business-model move to the deck or a footer link, the send→lands panel and disclosures become rows inside the card, not boxes inside it.
3. Own the color system. Swap Wise lime/forest for a SHORE palette (bring back Zcash gold as "shielded"), keep one primary hue for actions only, and add a real secondary accent for in-flight state. Make the header CTA secondary (outline) once the flow has started.
4. Fix AA. "Readable by" rows ≥4.5:1 (darken red or go 14px), footer ≥4.5:1, metric placeholder "–" ≥3:1 (or use "—" in ink-2 with a "pending" label). Move labels and captions from 12px to 13-14px.
5. Make progress a visual. Give the crossing bar stop ticks, a moving head and elapsed/ETA labels. Give the idle hero band full-strength color, not 35% opacity. Fix the landed hero caption. Dedupe 84s at idle.

ROUND: 6.6 min=5 round=1 critic=fresh:grokbot-knowledgework review=/Users/kamal/Desktop/solana/cwf/zcash/projects/airlock/docs/design/v2/design-review.md
