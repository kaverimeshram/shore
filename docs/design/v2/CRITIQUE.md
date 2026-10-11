# SHORE v2 UI critique (self-critique log; not the independent gate critic)

## Round 1 (shots: docs/design/v2/r1/)
What the shots show: a payments-app layout, soft grey-green canvas, white cards with shadow, an ink crossing card with
a lime tide band, four metric cards, step cards with chips, mono address pills with Copy, a ringed read-back card.
It no longer reads as a document. Problems, worst first:
1. Landing metrics are four cards of grey dashes: a dead row before connect, the loudest "empty template" signal.
2. 390: the first action (Connect) sits below about 1.5 screens of headline, judge card and empty metrics.
3. 390: the app bar wraps ("Try it / in 60s" on two lines) and the "7 for this transfer" chip wraps in the panel.
Also seen: the readback amount is strong; the quote pair reads well; the Source folds feel like debug rows.

Fixes for round 2: (1) metrics show the last mainnet run (0.0014 sent, 0.00115 landed, 22.9%, 84s) labelled
"last run" until a wallet connects; (2) mobile order: crossing, headline, metrics, steps, then the judge card;
(3) nowrap on app bar and chips, shorter app-bar label on mobile.

## Round 2 (shots: docs/design/v2/r2/)
Fixed from r1: the metric row now carries the last run's numbers in a muted "idle" tier, the app bar holds one line,
and the mobile order puts the crossing card first. Still wrong, worst first:
1. 1440 during the crossing: the page has scrolled to the step list, so the ink crossing card with the tide band (the
   one moving thing) is off screen; the "Crossing" step only says "Crossing now".
2. 390: Connect still sits below the fold; the crossing card (84s + caption + band) and a 6-line lede take the first screen.
3. 390: metric labels wrap to two lines ("BALANCE ON / SOLANA") and the sub-captions truncate.

Fixes for round 3: (1) a live mini tide band and elapsed counter inside the Crossing step, driven by the same tide();
(2) mobile: tighter crossing card (smaller hero, no caption), lede clamped to 3 lines; (3) short metric labels
(Balance, Lands, Fee, Elapsed).

## Round 3 (shots: docs/design/v2/r3/)
Fixed: the Crossing step now carries its own lime tide band with Solana / NEAR Intents / Zcash stops, so the moving part
is on screen wherever the page is scrolled; the mobile crossing card is compact; metric labels fit on one line.
Seen and accepted (time box): on 390 the first action sits just under the fold after the crossing card, the headline
and the lede (the lede keeps the positioning line, so it is not clamped); the "Source" folds are still plain rows;
desktop leaves empty space under the right-hand panels during the early steps.
Self-critique only. The cwf-ui-gate requires an independent critic (fresh:<agent> or design-review) with a FINAL line.
