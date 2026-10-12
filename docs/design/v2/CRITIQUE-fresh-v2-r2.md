# CRITIQUE · SHORE ui-v2 · round 2 (f7a138c) · critic=fresh

critic: fresh:grokbot-knowledgework · 2026-10-11 22:46 IST · builder notes not read before scoring
review report: docs/design/v2/design-review-r2.md · shots: docs/design/v2/fresh-r2/

## My 5 r1 fixes
1. Live state next to action: FIXED. Sticky dark live card on the right on desktop and a Lands/Fee/Elapsed strip on mobile, elapsed in the Crossing card, landed number never clipped.
2. Flatten nesting: FIXED. One-line explainer, row disclosures, inline send→lands, flat pitch sections.
3. Own color system: FIXED. Zcash gold, one action blue, teal in flight, Wise hexes gone, header CTA outlined after start (desktop). Mobile idle still has 2 filled primaries.
4. AA + 13-14px: PARTIAL. Min 4.27 (2 fails at 13px) and 12px nearly gone. New: invisible white focus ring on blue buttons.
5. Progress visual: FIXED. Ticks, head, elapsed/ETA, teal → gold. But the band shows "landed" during quote (last run), and ETA 84s contradicts the quote's 135s.

## Scores
SCORES: data-as-hero 8, density 8, liveness 7, interaction 7, disclosure 8, color 8, motion 8, mobile 7, tells 7, trust 8
AVERAGE: 7.6   MIN: 7   VERDICT: FAIL (average < 8.0). Does NOT beat Harbor r3 (7.7); it is 0.1 below.
- data-as-hero 8: the current number is always on screen. But elapsed shows 4 times at crossing, and 0.00115 3 times / 84s 4 times at landed.
- liveness 7: during quote, the live card shows the past run's full band and "115,000 zat landed". ETA 84s vs quote 135s.
- interaction 7: invisible focus ring on primaries. Otherwise strong (Copy pills, chips, Change links gone after signing).
- mobile 7: strip and landing good. Idle shows 2 filled primaries and a finished last-run card. The "4 in general" chip wraps.
- tells 7: no longer a Wise clone, no nested cards. But the dark stat card + light rounded-card stepper + Now/Waiting chips is a familiar kit; the gold band is the only owned device. Harbor r3 had more identity.

## Top fixes
1. Focus ring: use outline 2px solid the action blue (or ink) with offset 2px on light surfaces. Never white on white.
2. Live card before approval: show the band empty or striped "not started", with the caption "quoted, not sent". Keep last-run data only as a small "last run 84s" line.
3. Dedupe: during crossing, keep one elapsed (the hero) and turn the others into the band label only. At landed, keep the hero and the read-back amount, and drop the Lands metric value (show "read back ✓").
4. AA: darken the teal chip text and the "1Click/NEAR" readable-by text to ≥4.5:1 (e.g. teal ≤ #0B5F5A on its tint).
5. One ETA source: use the quote's "about 135s" (or the recorded 84s, labelled as such) everywhere. Mobile idle: header CTA outlined. Keep "4 in general" on one line.

ROUND: 7.6 min=7 round=2 critic=fresh:grokbot-knowledgework review=/Users/kamal/Desktop/solana/cwf/zcash/projects/airlock/docs/design/v2/design-review-r2.md
