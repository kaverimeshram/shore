# CRITIQUE · SHORE · round 2 · critic=fresh

critic: fresh:grokbot-knowledgework (independent; builder CRITIQUE.md entries and DESIGN.md NOT read before scoring)
date: 2026-10-10 19:11 IST
rubric: demo-judge 10 axes, 0-10. Pass bar: average >= 8.0 AND every axis >= 7.
evidence: docs/design/r2-1440, r2-390, r2-quote-*, r2-crossing-*, r2-readback-*, r2-full-*. Plus my own Playwright run of
https://design-preview.shore-zec.pages.dev/shore/ (replay) at 1440x900 and 390x844: idle, connected, address, quote, signed, crossing,
landed, full page, keyboard-opened disclosure, plus a ?speed=3 crossing sampled every 6s. Shots are in docs/design/fresh-r2-live/.
Computed styles and source (shore.css/shore.js) were checked for motion, sticky and scroll behaviour.

## Round-1 defect check
| r1 defect | r2 status |
|---|---|
| Static sticky h1 is the biggest thing at every step | FIXED (desktop). After connect, a live ledger replaces it: "Lands in the shielded pool, quoted 0.00108", then "Crossing, elapsed 25s", then "Shielded, read back 0.00115", 64px serif. |
| Landed amount below the fold | FIXED on desktop (in the sticky ledger). PARTLY on mobile: auto-scroll lands it UNDER the sticky strip (see 8). |
| Left 45% empty | MOSTLY FIXED. The ledger and rail fill it during the flow. |
| 9 equal quote rows | FIXED. Send → lands pair, "22.9% fee · about 135s · valid 2d 17h", then "Fee breakdown (6)" disclosure. |
| Step 1 repeated copy | FIXED. |
| Debug filenames / correlation id in copy | FIXED. Moved behind "+ Source". |
| US-locale expiry, no countdown | FIXED. "valid 2d 17h" plus "2026-10-13 06:47 UTC" in the breakdown (UTC, not the user's IST; minor). |
| Idle rail shown as completed | FIXED. Hollow nodes, "84s last mainnet run". |
| Orchard vs Ironwood unexplained | FIXED. "Orchard receiver, Ironwood pool (v6 transaction)". |
| Dead "Approve (replay)" and stale "Get quote" | FIXED. Visible buttons at quote: only Approve. At end: none. Steps become receipts ("Signed · 2LLVDB…QQ86au"). |
| Amount input reads as placeholder | FIXED. The quoted step shows the amounts as set values. |
| Mobile placeholder clipped | FIXED. "Paste a fresh u1". |
| What's still public always expanded | FIXED. Collapsed "What's still public (4) +". |
| Teal does six jobs / no warning tone | MOSTLY FIXED. Ink primary buttons, amber rgb(138,75,0) for 22.9%, teal for the shielded end-state and landed. No brand cue yet. |
| Motion = load rise only | MOSTLY FIXED. The rail connector fills against elapsed time (tide-fill scaleX), and the landed number counts up (countUp/rAF). Step swaps are still instant. |
| Mobile tracker at page bottom / "Tracking above" lie | FIXED. A sticky 3-node strip under the header. Copy is now "Crossing now. Nothing else to sign." |
| Generator-default look | NOT FIXED. Same type, palette and stepper (see 9). |

## Scores
SCORES: data-as-hero 7, density 7, liveness 8, interaction 8, disclosure 8, color 7, motion 7, mobile 6, tells 6, trust 7
AVERAGE: 7.1   MIN: 6   VERDICT: FAIL (2 axes below 7: mobile, tells; average 0.9 short)

### 1. Data as hero: 7 (r1: 4)
- Desktop is now right: the current number is always the largest element.
- Landed state repeats the same value 4 times on one 1440x900 screen: ledger 64px "0.00115", right-column 60px "0.00115", ledger row "Landed 0.00115 ZEC", and "0.00115 ZEC shielded" under the rail (d-6-landed.png). "84s" also appears 4 times (Elapsed row, under-rail stat, Crossing tag, "Landed in 84s"). Crossing state shows "25s" 3 times.
- Mobile has no ledger. During quote, the strip still shows the historical "84s last mainnet run, 115,000 zat landed" (m-3-quote.png), so the strip's number belongs to a past run while the user is pricing this one.
FIX: on desktop, drop the right-column "Shielded 0.00115" hero (keep its receipts table) and the under-rail stat line once the ledger is live. On mobile, swap the strip's stat to the current flow value (quoted lands, then elapsed, then landed) as soon as the user connects.

### 2. Density and hierarchy: 7 (r1: 6)
The split works now. The remaining cost is the duplication above. The right column at landed still stacks five receipt rows plus a duplicate hero. The ledger repeats "Send / Lands / Fee", which the Quote step also shows.
FIX: once the ledger exists, completed steps on the right should be one line each (title + receipt). Let the ledger own the numbers.

### 3. Liveness and provenance: 8 (r1: 7)
Good: tx links, mined height, decrypted-vs-quoted difference, Source disclosure, countdown.
Nit: the expiry is in UTC while every other time is relative. Show "valid 2d 17h (13 Oct, 12:17 IST)" or local time.

### 4. Interaction: 8 (r1: 6)
Receipts replace dead controls. Keyboard: the summary takes Enter with a visible 2px tide outline (d-8-disclosure.png). 48px targets.
Missing: there is no "Edit amount" or "Change address" once a step collapses. Recovery means reloading.
FIX: a small "Change" text link on the address and quote receipts.

### 5. Progressive disclosure: 8 (r1: 7)
Fee breakdown, Source and What's still public are collapsed with counts.
Nit: the count changes silently from "(4)" at crossing to "(7)" at landed. Either keep it stable or mark the 3 new items.

### 6. Meaningful color: 7 (r1: 5)
Color now encodes state: ink = action, amber = cost, teal = shielded.
Still missing: "Readable by: anyone" rows are not toned, so the privacy leak table is monochrome. Nothing ties the page to Zcash or Phantom (no mark, no hue).
FIX: amber dot on "anyone" rows, teal on "nobody". Add one small owned mark (a shoreline/tide glyph in the wordmark or on the rail end-node).

### 7. Motion: 7 (r1: 4)
The rail fill progresses (d-slow-0..3: tip moves Solana → Bridge → Shielded) and the landed value counts up. prefers-reduced-motion is honoured.
Gaps: step completion is an instant DOM swap with no collapse or cross-fade. The builder's own r2-full-1440.png caught the ledger at roughly 15% opacity, which suggests the `rise` entrance replays on state change (shore.css:182). Verify that it does not flash on each step.
FIX: 180-220ms height/opacity collapse via a grid-rows 0fr→1fr trick or View Transitions. Run `rise` once per load only.

### 8. Mobile: 6 (r1: 4)
- Sticky strip: good idea, works during crossing (m-5-crossing.png).
- DEFECT (reproduced): on landing, shore.js:323 calls `card.scrollIntoView({block:"start"})`, and no element has scroll-margin-top. The 98px sticky `.crossing` strip covers the top of the "0.00115 ZEC" hero, so the payoff number is cut in half at the exact moment it appears (r2-readback-390.png and my m-6-landed.png).
- The strip shows a stale last-run stat during connect, address and quote (see 1).
- There is no mobile ledger, so send/lands/fee are only visible inside the Quote step and scroll away.
FIX: `scroll-margin-top: calc(var(--strip-h) + 16px)` on the result card (or scroll to `block:"center"`). Make the strip's stat the live flow number. Add "lands 0.00108" to the strip after quote.

### 9. Machine-made tells: 6 (r1: 5) — still reads as AI-generated
The behaviour is now designed. The look is not. It is still Newsreader serif display + Manrope body on sage paper (rgb 238,241,236) + one deep teal + hairline rules + circled 1-5 numbered stepper + serif section titles ("Read-back", "What's still public"). That is the default "tasteful editorial" kit a generator reaches for. Cover the wordmark and this could be any 2026 fintech hackathon entry. The one concept-specific device, the "tide-fill" rail, is a plain 1px line with three circles.
FIX: make the rail the signature. Give it a heavier, owned shoreline/tide band (e.g. 6-8px band that floods teal from left, with the landed amount resting on the Shielded node), reuse that motif as the wordmark mark, and drop the circled step numerals for receipts/ticks. This is the change most likely to move a judge from "nice template" to "designed".

### 10. Would a professional trust it with size: 7 (r1: 6)
Honest, receipted and controllable now. The half-hidden payoff on mobile and the repeated numbers are what still read as unfinished.

## Is it still AI-generated-looking?
Yes, but only in the visual system, no longer in the behaviour. Structure, data hierarchy and state handling now look deliberate. The type/palette/stepper kit is still the generator default, and nothing visual is unique to SHORE.

## Remaining top fixes (leverage order)
1. Mobile landed occlusion: add scroll-margin-top equal to the sticky strip height (or scroll to center) so 0.00115 ZEC is fully visible when it appears. (8, trust)
2. Mobile strip shows the live flow number (quoted lands, then elapsed, then landed), not "84s last mainnet run", once connected. (8, 1)
3. Kill duplication: one display-size landed number on screen (keep the ledger, demote the right-column hero to its receipts). One elapsed readout, not four. (1, 2)
4. Owned visual signature: turn the 1px rail into a tide band that floods to the Shielded node, reuse it as the wordmark mark, and replace circled step numerals. (9; probably +1-2 on tells and color)
5. Motion and recovery polish: collapse/cross-fade on step completion, `rise` only once per load, "Change" links on collapsed address/quote, tone "anyone" vs "nobody" rows. (7, 4, 6)
Fixing 1-3 should lift mobile to 7-8. Fix 4 is required to get tells to 7. Without it the average tops out around 7.6.

ROUND: 7.1 min=6 round=2 critic=fresh:grokbot-knowledgework review=NONE
