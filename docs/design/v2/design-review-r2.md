# Design review r2 · SHORE ui-v2 fix round (f7a138c)

reviewer: fresh:grokbot-knowledgework (independent; builder notes not read)
date: 2026-10-11 22:46 IST · target https://ui-v2.shore-zec.pages.dev/shore/ (replay)
method: manual review in the gstack /design-review scoring system (10 weighted categories; High = -1 letter, Medium = -1/2). I did not run the
gstack skill itself: it is a Claude Code skill that edits and commits fixes, which would change the code under review.
evidence: Playwright at 1440x900 and 390x844 through idle → connect → u1 → quote → approve → crossing → landed → public, plus a keyboard focus
probe and a composited contrast audit of every visible text node in 16 states. Files are in docs/design/v2/fresh-r2/ (contrast.json).

Design Score: B-
AI Slop Score: C+ (coherent and no longer a Wise clone, but a familiar fintech kit: dark stat card beside a light stepper of rounded cards
with Now/Waiting chips, Bricolage + Figtree. The gold band is the only SHORE-specific device.)

| Category | Weight | Grade | Findings |
|---|---|---|---|
| Visual Hierarchy | 15% | B | Fixed: a sticky live card beside the flow keeps the current number (0.00108, then 32s, then 0.00115) on screen on desktop, and a sticky Lands/Fee/Elapsed strip does the same on mobile. M: duplication. During crossing, elapsed shows 4 times on one 1440 screen (hero "32s", band "32s elapsed", Elapsed metric, crossing card "32s elapsed, ETA about 84s"). At landed, 0.00115 shows 3 times (hero 56px, metric, read-back 44px) and 84s 4 times. M: in the quote state the live card's band is fully gold at "landed" and the caption still reads "Solana to shielded Zcash, 115,000 zat landed", so the card reports a finished past run while you price a new one (d-3-quote.png). |
| Typography | 15% | B+ | Floor raised: 13px is now the smallest common size (495 of 1,862 nodes; 12px only 60). Geist Mono addresses are legible. P: the "4 in general" chip wraps over 2 lines on 390 (m-5-crossing.png). |
| Spacing & Layout | 15% | B+ | Nesting flattened: explainer is one line, disclosures are rows, send→lands is inline, Why now / Business model are flat sections below the flow. P: the right column below the sticky card is empty for the whole left stack. |
| Color & Contrast | 10% | B | Own system: blue #2350D8 = action, teal = in flight, Zcash gold = shielded/landed, amber = fee, green chips = done. Wise hexes removed. H: 2 AA failures, both 4.27:1 at 13px/600: the teal "crossing" chip and "Readable by: 1Click/NEAR …". |
| Interaction States | 10% | C+ | H: invisible focus ring on primary buttons. "Connect recorded wallet" and the other blue buttons use outline 2px solid #FFFFFF with offset 2px on a #FFFFFF card, so keyboard focus is not visible (WCAG 2.4.7; d-9-focus-connect.png). Links do get a blue outline. M: at 390 idle there are two filled blue primaries (header "Try it in 60s" + "Connect recorded wallet"). On desktop the header correctly drops to outline once the flow starts. |
| Responsive | 10% | B | Mobile strip works, landed number fully visible, page 3,078px (from 3,943). M: the strip appears only after connect; the idle dark card on mobile shows the finished last run. |
| Content Quality | 10% | B | M: the stale "115,000 zat landed" caption during quote. M: ETA mismatch: the quote says "about 135s", the band and crossing card say "ETA about 84s". Pitch sections are flat now and acceptable. |
| AI Slop | 5% | C+ | See headline. |
| Motion | 5% | B+ | Band with ticks, a moving head and elapsed/ETA. Teal-to-gold on landing. |
| Performance Feel | 5% | B | Fast; no jank observed. |

## WCAG AA (re-measured)
- PASS: ink on page/card 16-19:1; muted labels and captions ≥4.5:1; white on blue #2350D8 ≈6.5:1; gold/white figures on the dark card pass large-text.
- FAIL: teal "crossing" chip, 13px/600, 4.27:1. FAIL: "Readable by: 1Click/NEAR (sender holds the outgoing view key)", 13px/600, 4.27:1.
- FAIL (non-text, 2.4.7 / 1.4.11): white focus outline on a white card for blue primary buttons.
- min text contrast across 16 states: 4.27:1 (r1: 2.03).

## BACKED v3 direction
| Requirement | r1 | r2 |
|---|---|---|
| Metric cards | mis-placed | GOOD: in the sticky live card and the mobile strip |
| Panels | nested | GOOD: flat |
| Status chips | good | GOOD |
| One clear primary action | partial | MOSTLY: desktop yes; mobile idle has 2 filled primaries |
| Status color system + secondary accent | borrowed | GOOD: blue action, teal in-flight, gold shielded, amber fee |
| Polished progress/chart | weak | GOOD for progress (ticks, head, ETA); still no chart, but none is needed for this flow |
| Type scale + AA | 3 fails | PARTIAL: 13px floor, 2 text fails at 4.27, invisible focus ring |
