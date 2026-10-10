# CRITIQUE · SHORE · round 1 · critic=fresh

critic: fresh:grokbot-knowledgework (independent, did not build; DESIGN.md / CRITIQUE.md / builder notes NOT read before scoring)
date: 2026-10-10 IST
rubric: demo-judge 10 axes (the independent critic named in cwf-pro-ui step 5), scored 0-10. Pass bar: average >= 8.0 AND every axis >= 7.
evidence: docs/design/r1-1440.png, r1-390.png, r1-quote-1440.png, r1-quote-390.png, docs/screens/shore-*.png,
          web/shore/design-research/negative/shore-r0-harbor-*.png, plus my own Playwright run of
          https://design-preview.shore-zec.pages.dev/shore/ at 1440x900 and 390x844, walking connect, fresh u1, quote, approve, crossing, read-back
          (shots in docs/design/fresh-r1-live/). Computed styles were measured with getComputedStyle.

## Gate zero
Verb: turn ZEC held on Solana into shielded Zcash with one Phantom approval.
Path: the flow does it end to end in replay (one SPL transfer, then 1Click, then a note decrypted at height 3,512,504). A live mainnet button exists.
VERDICT: ELIGIBLE. The problems are in the surface, not the category.

## Versus r0 (negative)
Real progress. The cards inside cards are gone (just hairline rows now). Muted text is rgb(59,72,67) on rgb(238,241,236), 8.4:1, where r0 was well under 4.5:1. Steps are expanded with copy instead of collapsed gray rows. The default dark template is replaced by a sage paper ground. The 5/10 problems are fixed. Round 1 now has a new set of problems.

## Scores
SCORES: data-as-hero 4, density 6, liveness 7, interaction 6, disclosure 7, color 5, motion 4, mobile 4, tells 5, trust 6
AVERAGE: 5.4   MIN: 4   VERDICT: FAIL (needs avg >= 8.0 and every axis >= 7; 6 of 10 axes are below 7)
Advisory letter grades (not a gstack /design-review report, so it cannot be used as review=): Design C+, AI Slop C.

### 1. Data as hero: 4
- Desktop, every state: the largest element on screen is the h1 "Move your Solana ZEC into the shielded pool." (Newsreader 60px). It is sticky, so it stays the biggest thing through quote, signing and crossing even though it carries no information after the first second.
- Step 1, connected: the user's balance "0.0014 ZEC on Solana" is about 13-15px, set inline as a sub-line under the step title. The number the user came for is body text.
- Step 3 quote: "You get, quoted 0.00108 ZEC" (22px serif) is row 2 of 9 rows that all share the same hairline. The worst news on the page, "22.9% fee", is an 11-13px gray .tag in the corner of the step header.
- Read-back: the payoff "0.00115 ZEC" in teal 56px sits at about y=1700 on desktop (shore-desktop-4, f-d-e.png). It is below the fold and needs a manual scroll. Nothing scrolls it into view or mirrors it in the sticky column.
- At idle, the biggest number is "84s", a historical stat from a past run.
FIX: after connect, replace the sticky h1 with a live ledger in the left column: send amount, then lands amount, then fee %, then ETA, then landed amount. Show it at 56-64px tabular serif, so the current number is always the biggest thing on screen. When the note decrypts, put "0.00115 ZEC shielded" in that slot and scroll or focus it.

### 2. Density and hierarchy: 6
- Desktop at 1440: after the rail, the left 45% of the width is empty paper for most of the 2.7-3.5k px scroll (r1-quote-1440 / shore-desktop-4). All the work happens in a 390px-wide right column, which is still a narrow column, just moved to the right.
- The quote table has 9 rows with equal weight (You send, You get, Worst case, Zcash withdraw fee, Refund fee, Total cost, 1Click minimum, Estimated time, Quote valid until). Primary, secondary and fine-print rows are not distinguished.
- Step 1 says the same thing twice: "Phantom shares your Solana address and ZEC balance. Nothing is signed yet." followed by "SHORE reads your ZEC balance on Solana. It never asks for a seed phrase..."
FIX: split the quote into a primary pair (send, then lands) at display size, a secondary line (fee %, ETA), and a disclosure "Fee breakdown (6)" for the other rows. Cut one of the two step-1 sentences.

### 3. Liveness and provenance: 7
- Good: the "REPLAY · RECORDED MAINNET RUN" mode tag, tx links for Solana, NEAR and Zcash, mined height 3,512,504, the decrypted-vs-quoted difference, and the "Readable by" table.
- Defects: debug filenames in user copy ("from shore-funded-quote-v2.json", "Every value comes from shore-live-run.json", "correlation 9136ee09..."). "Quote valid until 10/13/2026, 12:17:48 PM" uses a US locale date and shows no live countdown. The bar under it is static.
- Idle state (r1-1440, live-d-0): the rail already shows Solana ZEC and Bridge filled and Shielded active-teal before the user has done anything. It reads as "already done", not as "last run".
- Naming: the address chip says "orchard", but the result says "Shielded in Ironwood" / "Pool ironwood", with no explanation of how the two relate.
FIX: move file and correlation provenance behind a "Source" disclosure. Use an ISO or IST timestamp with a ticking "valid 2d 23h" countdown. Render the idle rail as outlined "last run 84s" or as empty, never as completed. Reconcile Orchard/Ironwood in one label.

### 4. Interaction: 6
- After signing, "Approve in Phantom (replay)" stays as a full-width bordered button with gray text (f-d-c, f-m-d). It looks like an enabled secondary action. It is a dead control.
- "Get quote" stays as a full-size primary button after the quote has rendered (f-d-b), so it competes with Approve.
- Once quoted, the amount input renders "0.0014" in ink-2 on a transparent ground (readonly). It reads as placeholder text, not as the committed value.
- Mobile 390: the paste-your-own placeholder is clipped ("u1... (your own wallet, never used before" with the paren cut off; live-m-1).
- Good: a focus-visible ring (2px solid tide) is present, buttons have 48px targets, and hover states are specified per property.
FIX: collapse finished steps to a one-line summary with an "Edit" text link and remove their buttons. Swap "Approve" for a "Signed · 2LLVDB...86au" receipt line. Shorten the placeholder to "Paste a fresh u1".

### 5. Progressive disclosure: 7
- The page leads with the outcome (headline), then the action. Steps unlock in order.
- But "What's still public" (5 paragraphs) and "Counterparty risk" are fully expanded at idle and after read-back, and the "Readable by" table repeats the same content in a second format. On mobile this essay pushes the rail to the bottom (see 8).
FIX: keep one version (the "Readable by" table is better) and put it behind a disclosure row "What's still public (7)" until read-back.

### 6. Meaningful color: 5
- One accent (teal rgb(11,110,99)) is used for six unrelated jobs: primary buttons, the active step badge, the "Shielded" rail label, the "orchard" tag, the "Later spends of the note" link-looking label, and the landed number. Color does not encode state.
- The 22.9% fee, the worst case after slippage and the counterparty risk are all neutral gray. No warning tone exists.
- No Zcash or Phantom identity cue anywhere. The palette could belong to any product.
FIX: reserve teal for "shielded / private" (the rail end-state, the landed number, the "nobody" row). Make primary buttons ink. Add one warm caution tone (an amber or ochre, AA on paper) for fee >10% and "Readable by: anyone".

### 7. Motion: 4
- The only motion is a 6px "rise" entrance on load (shore.css:140-142). Step transitions swap instantly. Rail dots just swap fill, with no progress line between Solana, Bridge and Shielded. The landed amount appears without counting up. The crossing is the product's dramatic moment, and it happens off-screen on mobile and as a dot color change on desktop.
- Good: prefers-reduced-motion is respected.
FIX: draw a connector line on the rail that fills Solana to Bridge to Shielded against the elapsed clock. Give the 0.00115 a short tabular count-up (300-500ms). Cross-fade a finished step into its summary row.

### 8. Mobile: 4
- The rail and the "84s" timer are moved to the very bottom of the page at <960px, after "Counterparty risk" and just above the footer (r1-quote-390, f-m-tail.png). During crossing, the step 5 copy says "Tracking the crossing above", but on mobile the tracker is about 2,000px below. That is factually wrong on the device most judges will try.
- The page is 4,673px tall on mobile after completion. The quote table wraps values onto two lines ("0.00133334 / ZEC", r1-quote-390). The header mode tag fills the right half of the bar beside the wordmark.
FIX: on mobile, render the rail as a compact sticky 3-segment strip under the header (Solana, Bridge, Shielded, plus the timer). Put the landed amount in that strip on completion. Change the copy to "Tracking below/above" based on layout, or just "Tracking:".

### 9. Machine-made tells: 5 (still reads as AI-generated, see verdict)
- Palette and type: Newsreader display + Manrope body on sage off-white with a single deep teal and hairline rules. This is the 2025-26 "tasteful editorial fintech" default that generators produce when told to avoid the dark template. It swapped one default for another.
- Structure: a sticky left hero plus a right numbered stepper (1-5 circled counters, .step::before) with a hairline under every row and the same vertical rhythm for every section. This is the stock split-hero form template.
- Copy tics: "Honest note:", "not taken on trust", "It is read from the chain, never typed in", "That is the only signature", "Nothing else to sign". Each one is fine alone; together they read like generated reassurance. The trust claim is stated about 6 times.
- Uppercase letter-spaced micro-labels ("AMOUNT (ZEC ON SOLANA)", "OR PASTE YOUR OWN", "REPLAY · RECORDED MAINNET RUN").
FIX: give the page one owned signature tied to the concept, for example a shoreline/tide-line rail that literally fills as funds cross. Use real tabular numerals for all amounts. Cut the reassurance lines to one per screen.

### 10. Would a professional trust it with size: 6
It is honest and well-receipted. But dead buttons, debug filenames, a rail that says "done" before you start, and a payoff hidden below the fold make it look like a careful prototype, not a product.

## Is it still AI-generated-looking?
Yes, less than r0, but yes. It no longer fails on contrast or nesting. It fails on sameness. Every element (heading, steps, quote rows, essays) gets the same hairline-and-paragraph treatment. The type/palette pair is a known generator default. The structure is a template split layout. Nothing on screen could only belong to a Solana-to-shielded-Zcash crossing. A judge who has seen 30 hackathon entries will place it in the "Claude/V0 light editorial" bucket within 3 seconds.

## Top 5 fixes (in leverage order)
1. Live ledger instead of the static sticky h1 (desktop) and a sticky 3-segment strip (mobile): balance, then quote, then elapsed, then landed amount always on screen at display size. Fixes axes 1, 2, 8.
2. Fix mobile rail placement: no more tracker at the page bottom under "Tracking the crossing above". Fixes 8 and trust.
3. Collapse completed steps to one-line receipts and remove the dead "Approve (replay)" and stale "Get quote" buttons. Make the committed amount read as a value. Fixes 4, 2.
4. Give color a job: teal = shielded only, ink primary buttons, one caution tone for the 22.9% fee and "Readable by: anyone" rows. Draw an animated rail connector during crossing and count up the landed amount. Fixes 6, 7.
5. Strip the generator tells: one owned visual signature (a tide-line rail), quote broken into primary/secondary/disclosure, cut repeated reassurance copy and debug filenames, reconcile Orchard/Ironwood, idle rail not shown as completed. Fixes 9, 3, 5.

## Gate-format line (round score, NOT a pass; append to CRITIQUE.md only as a round entry)
ROUND: 5.4 min=4 round=1 critic=fresh:grokbot-knowledgework review=NONE
# If the builder's process requires the FINAL syntax for parsing:
# FINAL: 5.4 min=4 round=1 critic=fresh:grokbot-knowledgework review=NONE   (gate would FAIL: score<8, min<7, no design-review report)
