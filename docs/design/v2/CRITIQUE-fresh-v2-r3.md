# CRITIQUE · SHORE ui-v2 · round 3 (Web3 pass) · critic=fresh

critic: fresh:grokbot-knowledgework · 2026-10-12 07:41 IST · builder notes not read before scoring
review: docs/design/v2/design-review-r3.md · shots: docs/design/v2/fresh-r3/

## r2 fixes
1. Focus ring: FIXED in colour/size (3px #2350D8, 6.5:1). Residual: it fades in over 150ms on primary buttons.
2. Live card before approval: FIXED ("Not started", "quoted, not sent", empty band, last run reduced to one line).
3. Dedupe: FIXED on desktop (one elapsed readout; ETA in the metric slot; landed amount once in the read-back). Mobile shows strip + card together after connect.
4. AA: FIXED. 0 text failures, min 4.56.
5. One ETA / mobile CTA / chip wrap: FIXED (~135s everywhere, last run labelled 84s; header outlined; "4 in general" on one line).

## Scores
SCORES: data-as-hero 8, density 8, liveness 9, interaction 8, disclosure 8, color 8, motion 8, mobile 7, tells 7, trust 8
AVERAGE: 7.9   MIN: 7   VERDICT: FAIL (0.1 under 8.0). BEATS production Harbor r3 (7.7) by 0.2.
- liveness 9: the pending/confirmed tracker, network chips, IST timestamps and height make it read like a real wallet flow.
- mobile 7: strip and card duplicate numbers after connect; marketing hero above the balance; "0.0014 / ZEC" wraps.
- tells 7: the visual kit is unchanged from r2 (dark stat card + rounded stepper + chips + pitch sections).

## Fixes to clear 8.0
1. Mobile: hide the sticky strip while the dark live card is in view (IntersectionObserver). Keep amounts on one line (white-space:nowrap, or drop "ZEC" to a smaller unit line).
2. Balance first on the page: after connect, collapse the marketing hero to the kicker line on both viewports so the live card and the active step sit at the top.
3. Focus ring instant: remove outline-color from the buttons' transition-property.
4. Move Why now / Business model behind the header "Why SHORE" link (or a separate page); the product page ends at What's still public.
5. Copy polish: "Confirmed, send to land" → "Confirmed in 84s"; "Lands shielded, quoted" → "You'll receive (quoted)"; add USD to the arrival line.

ROUND: 7.9 min=7 round=3 critic=fresh:grokbot-knowledgework review=/Users/kamal/Desktop/solana/cwf/zcash/projects/airlock/docs/design/v2/design-review-r3.md
