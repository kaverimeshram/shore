# SHORE first screen, design review round 4 (fresh critic, demo-judge)

Inputs: r7-1440.png, r6-390.png, r4-1440.png, r4-390.png, plus fresh captures of
http://127.0.0.1:8642/shore/ at 1440x2400 and 390x2400 (2026-10-12).

## Gate zero
Verb (CWF Zcash track): products "utilizing ZEC the asset". SHORE executes a mainnet
crossing (Solana ZEC to a shielded Orchard note via 1Click, recorded 10 Oct, 84 s,
Solana tx and Zcash tx linked in the receipt header). It moves value. ELIGIBLE.

## Scores
| Axis | Score | Evidence |
|---|---|---|
| Visual hierarchy | 6 | Headline is the largest thing; the receipt's one metric ("6 of 7") is 15px body text in the receipt footer. |
| Typography | 6 | Heavy grotesk display + mono values works. Label column collides with values at 1440 ("Amount entering the shielded pool" abuts "0.00115 ZEC"). Receipt meta line mono, title sans caps, three type voices in one card header. |
| Color and semantic use | 7 | Red = public, teal = counterparty, green = private actually encodes state. Blue CTA is unrelated to the receipt palette; 6 of 7 rows red makes the page read as an alarm. |
| Spacing and layout | 4 | At 1440 the receipt and hero sit in a ~760px left column; the right ~500px of the first viewport is empty. The live product (Connect Phantom + Crossing panel) starts at y~1175, below a 1100 fold. r4 used the width; r7 regressed to a document column. |
| Component quality | 6 | Pills are generic tinted badges with dots; three wrap to two lines at 1440 so row heights jump (55/72/72px). Refusal strip is three equal bordered cards. Receipt card itself is clean. |
| Responsive at 390 | 7 | Stacks cleanly, pills under values read well. Header meta wraps with an orphan " · Zcash tx"; CTAs stack at unequal widths; first viewport shows hero + 1.5 receipt rows. |
| Copy and argument | 7 | "Every bridge says private. SHORE itemizes it." is a real, memorable claim, far stronger than r4's "Got ZEC in Phantom from rewards?". But the receipt has no baseline: a judge reads "6 of 7 readable" as SHORE leaking, not as SHORE disclosing what every bridge hides. What SHORE prevents (refusals) sits below the receipt, outside the fold at 1440. |
| AI-slop | 6 | Gone: numbered 3-step cards, stat-grid. Remaining: headline + 3-line subhead + primary/secondary button pair, dot-pills on every row, three-equal-card strip, grey #eef canvas. DESIGN.md's till-roll / redaction-log identity appears only as one dashed rule. |

Average 6.1, min 4 (layout).

## Questions
1. Styled document with one designed component. The receipt is product; the page around it is a left-aligned column.
2. Mostly. "Shield my ZEC" is obvious, but four names for one action compete: "Try it in 60s", "Shield my ZEC", "Connect recorded wallet", "Use it live (mainnet)".
3. The receipt content, no: real addresses, the unauthenticated /v0/status finding, OVK note. The frame around it, yes.

## Three highest-leverage fixes
1. Layout, 1440 first viewport. Wrong: right 500px empty, product flow below fold. Fix: two columns: receipt left, the dark Crossing panel (scrubbable replay) right at the same y, so the receipt rows light up as the replay scrubber passes each hop.
2. Receipt summary "6 of 7 items are readable by someone." Wrong: the only number is 15px and reads as self-indictment. Fix: add a "typical bridge" vs "SHORE" column (bridge: unknown/unstated per row; SHORE: stated + refusal code that blocks the worse case), and set the count as a 48px numeral in the receipt header: "7 of 7 stated. 3 linkages refused."
3. Exposure pills. Wrong: generic dot-badges, uneven heights, red wall. Fix: apply DESIGN.md's redaction log: public values get the hazard highlighter on the value itself, private ones a black redaction bar, reader as a one-line mono label; fold the refusal codes into the rows they protect and delete the three-card strip.

Highest leverage: fix 1, put the live crossing beside the receipt so the first screen is the product, not a memo about it.

FINAL: 6.1 min=4 round=4 critic=fresh:demo-judge review=/Users/kamal/Desktop/solana/cwf/zcash/projects/airlock/docs/design/v2/design-review-r4.md
