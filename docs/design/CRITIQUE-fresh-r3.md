# CRITIQUE · SHORE · round 3 ("Harbor board") · critic=fresh

critic: fresh:grokbot-knowledgework (independent; builder CRITIQUE.md entries and DESIGN.md NOT read before scoring)
date: 2026-10-10 19:32 IST
rubric: demo-judge 10 axes, 0-10. Pass bar: average >= 8.0 AND every axis >= 7.
evidence: docs/design/r3-*.png, plus my own Playwright run of https://design-preview.shore-zec.pages.dev/shore/ (replay) at 1440x900 and
390x844: idle, connected, address, quote, signed, crossing, landed, full page, disclosure, a ?speed=3 crossing sampled every 6s, and the
expanded "What's still public" at landed. Shots are in docs/design/fresh-r3-live/. Computed styles, contrast and shore.css/shore.js were checked.
Identity measured: Big Shoulders Display (display + numerals) / Atkinson Hyperlegible (body). Paper #F2ECDF, ink #13233A, sea #1E5A8A,
gold #F4B728, gold-ink #6B4F00, warn rgb(163,52,27). Contrast on paper: ink 13.4, sea 6.2, gold-ink 6.5, warn 5.8 (all AA).

## Round-2 defect check
| r2 defect | r3 status |
|---|---|
| Mobile: landed number hidden under the sticky strip | FIXED. scrollIntoView block:"center" plus #readback scroll-margin-top 140px. 0.00115 is fully visible (m-6-landed.png). |
| Mobile strip shows the stale "84s last mainnet run" during the flow | FIXED. Strip reads "Lands, quoted 0.00108 ZEC", then "Crossing, elapsed 25s", then "Shielded 0.00115 ZEC". |
| 0.00115 four times / 84s four times | MOSTLY FIXED. Desktop: one 96px hero; the read-back card is demoted to 28px; Elapsed appears only in the ledger. Mobile still shows 0.00115 twice in one viewport (strip 28px + read-back 64px). |
| No owned visual signature, generic serif/sage/teal kit | FIXED. The tide band (sea leg → gold Shielded) is reused as the wordmark mark. Circled step numerals are gone. Condensed board type, cream/navy/Zcash-gold. |
| Step swaps instant, entrance replays | PARTLY. `rise` now only runs before html.booted, and receipts `settle` in 200ms. But .step animates `padding` (shore.css:89, slop gate 14), and steps still don't collapse smoothly. |
| No way to change address or amount | FIXED, but over-applied (see 4). |
| "anyone" vs "nobody" rows untoned | FIXED. Red dot and red text for "Readable by: anyone"; gold for "Visible to: nobody" (d-9-public.png). |
| Expiry in UTC | FIXED. "valid 2d 16h (until 13 Oct, 12:17 IST)". |
| What's still public count jumps 4 → 7 | FIXED with labels: "(4 in general)" then "(7 for this transfer)". The "in general" wording is clumsy. |

## Scores
SCORES: data-as-hero 8, density 8, liveness 8, interaction 7, disclosure 8, color 8, motion 7, mobile 8, tells 7, trust 8
AVERAGE: 7.7   MIN: 7   VERDICT: FAIL (every axis now >= 7, but the average is 0.3 short of 8.0)

### 1. Data as hero: 8 (r2: 7)
The current number is a 96px condensed numeral at every stage (quote 0.00108, crossing 25s → 84s, landed 0.00115 on a gold band). This is the right hierarchy now.
Defects:
- Landed ledger: the gold band sits behind the lower third of the numerals, and the "ZEC" unit is printed inside the band. The baseline and the band collide, so it reads as a highlighter stroke, not a deliberate plinth.
- Mobile landed: 0.00115 at 28px in the strip and 64px directly below in the same viewport.
FIX: put the band fully below the numerals (4-6px gap, unit outside or on the band's right end). On mobile, hide the read-back hero number while the strip already shows it, or vice versa.

### 2. Density and hierarchy: 8 (r2: 7)
Receipts are compact and the ledger owns the numbers. Residual: the quote receipt stacks "+ Fee breakdown (6)", "+ Source" and "Change amount" with about 35px between each (d-3-quote.png), so the completed step ends up taller than the active one.
FIX: put the three secondary controls on one inline row ("Fee breakdown (6) · Source · Change amount").

### 3. Liveness and provenance: 8 (r2: 8)
Good: IST expiry and countdown, Source disclosure, mined height, difference line.
Residual: at idle and quote the tide band is a flat empty beige bar with no nodes, which reads as a disabled or skeleton progress bar. The idle stat "84s" is 18px condensed with "last mainnet run" in 13px.
FIX: give the empty band its three stop ticks at idle, and show the last run as a ghost fill ("last run 84s").

### 4. Interaction: 7 (r2: 8)
NEW DEFECT: "Change" (address) and "Change amount" stay visible and enabled after "Review and approve: signed" and after "Crossing: landed" (buttons@end = "Change ; Change amount"; d-6-landed.png). Offering to change the amount of a transfer that has already been signed and has landed is misleading on a money surface.
Good: a 2px sea focus-visible ring on summaries, 48px targets, receipts instead of dead buttons.
FIX: remove the Change links once Approve is signed. After landed, offer one "Shield again" action instead.

### 5. Progressive disclosure: 8 (r2: 8)
Fine. "4 in general" should read "4 for every transfer".

### 6. Meaningful color: 8 (r2: 7)
Color now has jobs: sea = the Solana-side leg, gold = shielded / Zcash, red = cost and exposure, navy = action. The Zcash gold finally gives the page a brand cue.
Residual: red is used for both "the fee is high" and "this is publicly readable", which are different kinds of warning. Acceptable, but a judge may read the 22.9% fee as an error.

### 7. Motion: 7 (r2: 7)
The band fills with clip-path against elapsed time (d-slow-0..3), receipts settle, the count-up runs, and reduced-motion is honoured.
Defects: .step { transition: padding .2s } animates a layout property (hallmark slop gate 14), and completion is still a jump in height.
FIX: animate only opacity/transform, or use grid-template-rows 0fr→1fr for the collapse.

### 8. Mobile: 8 (r2: 6)
The strip is now the live number and the payoff lands fully visible. Residual: the duplicate 0.00115 (see 1), and the strip at 104px takes 12% of an 844px viewport for the whole flow.
FIX: shrink the strip to a single 56-64px row (band + one number) once scrolled.

### 9. Machine-made tells: 7 (r2: 6) — reads as designed, with a few kit fingerprints left
The new identity mostly works: condensed board type, navy/cream/gold and the tide band read like harbour signage, not a generator default. Remaining fingerprints:
- Every step carries a vertical left rule (3px sea on active, 2px gray on done; r3-quote-1440). That is the side-stripe card pattern (hallmark slop gate 5) applied to five rows.
- Two numeral systems on one screen. Amounts are in Big Shoulders (plain zero) in the ledger and receipts, but in Atkinson (slashed zero) in body and disclosures: "0.0014 ZEC" appears in both styles on d-9-public.png. Also "115,000", "7000 zat", "index 0", "/v0/status".
- Legibility, not just style: addresses and hashes (u1uufrl3tucq9…, ia3MLukF…3EL5Vsa5, DMDdCzum…ZEk9SjbM) are set in the condensed display face, where l / I / 1 are hardest to tell apart. These are exactly the strings a user verifies, and the page ships Atkinson Hyperlegible, which exists for this case.
FIX: drop the left rules (use the step title weight and a gold tick for state). Set all addresses and hashes in Atkinson with tabular figures (or a mono). Keep Big Shoulders for display amounts only, and choose one zero style for amounts everywhere.

### 10. Would a professional trust it with size: 8 (r2: 7)
Yes, mostly. Honest, receipted, branded, and the payoff is unmistakable. The post-sign "Change amount" is the one thing that would make a careful user hesitate.

## Is it still AI-generated-looking?
Largely no. "Harbor board" has a point of view and a signature (the tide band, the mark, the gold landing) that belongs to this product. What remains are small kit tells (left rules on every step, the mixed numeral styles, display type used for hashes), not a template.

## Remaining fixes (leverage order; 1-4 should clear 8.0)
1. Remove "Change" / "Change amount" after the transfer is signed; offer "Shield again" after landed. (4, 10)
2. Addresses and hashes in Atkinson (or a mono), one zero style for all amounts, Big Shoulders only for display numbers. (9, 3)
3. Drop the vertical left rule on every step; show state with title weight and a tick. (9)
4. Seat the gold band under the landed numerals instead of through them. Remove the duplicate 0.00115 on mobile. (1, 8)
5. Inline the quote receipt's three secondary links on one row. Animate the collapse without the padding transition. Give the idle band its stop ticks. Reword "4 in general". (2, 7, 3, 5)

ROUND: 7.7 min=7 round=3 critic=fresh:grokbot-knowledgework review=NONE
