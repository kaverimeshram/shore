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

## Fix round after fresh critic v2 (6.6, min 5). Shots: docs/design/v2/fix1/, docs/design/r2-*.png
1. Live state at the action: the dark crossing card now sits in a sticky right column beside the steps (desktop) and
   carries Lands / Fee / Elapsed; mobile gets a sticky Lands / Fee / Elapsed strip under the header once a wallet connects.
   The Crossing step shows "Ns elapsed, ETA about Ns"; the read-back card has scroll-margin-top (140px desktop, 130px mobile).
2. Flattened: the explainer is one line of three numbered items (hidden on mobile; the tour covers it); Why now and
   Business model moved to a footer section (header link "Why SHORE"); the quote's send/lands pair and the fee/source
   disclosures are rows with separators; exposure rows are rows, not tiles; metric cards are gone from the page body.
3. Own palette: harbour slate + Zcash gold (#f4b728) for shielded only, one action blue (#2350d8) for buttons,
   teal (#0e7c86) as the in-flight accent, success/warning/danger/info set. "Try it in 60s" turns outline once the flow starts.
4. Contrast: mute text #565f6b (about 6:1 on the canvas), danger chips #b4232a on #fbe4e5, footer in body ink, the
   placeholder dash sits on the dark card at 70% white; labels and captions are 13px+.
5. Progress: band with three stop ticks, a moving head, elapsed/ETA labels, full-strength gold at idle and landed,
   landed caption "Landed in Ns, read back from the note at height N", 84s shown once at idle (Elapsed reads "starts when you approve").
