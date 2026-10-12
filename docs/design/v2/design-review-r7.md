# SHORE first screen, design review round 7 (fresh critic, demo-judge)

Date: 2026-10-12. Critic had not seen earlier rounds.

Looked at: `docs/design/r14-1440.png`, `docs/design/r14-390.png`, `docs/design/r13-390-full.png`, plus fresh captures of
`http://127.0.0.1:8642/shore/` (default, `?live=1`, `?replay=1&tour=1&speed=8`) at 1440x900 and 390x844, and computed
styles read from the live DOM through `bhn deepsurge eval` at a 1459 px viewport.

## Gate zero

- Verb the brief names: CWF Zcash track, "startups integrating ZEC into their products"; Zecathon Cross-Chain, "Would
  someone use this on Monday", "Working beats ambitious".
- Code path that performs it: `web/shore/shore.js:300-311` `sendTransfer()` builds an SPL `transferChecked` to the 1Click
  deposit address and calls Phantom `signAndSendTransaction`; the receipt links a real Solana tx (`2LLVDB...QQ86au`) and
  Zcash tx (`fa8d4d6b...fa408`) and a read-back at block 3,512,504.
- What won last time: not checked in this pass.
- VERDICT: ELIGIBLE. The product moves ZEC; it is not read-only.

## Scores

| Axis | Score | Evidence |
|---|---|---|
| Visual hierarchy | 7 | H1 Bricolage 800 at 50 px leads; receipt right column fits the 1440 fold (bottom edge at y=868). But the stamp "84 s" (44 px) competes with the CTA row directly above it, and the receipt's punchline row, "Later spends of the note / NOBODY", is the quietest thing in it: a 112x16 black bar plus 12 px mono label. |
| Typography | 7 | Three faces used with intent (Bricolage display, Figtree body, Geist Mono data). Each receipt row stacks four registers (15 px sans label, 13 px mono value, 12 px mono caps reader, 13 px sans refusal line), which reads busy. Stamp provenance line is 11.5 px mono in brown on a rotated box: softest, least legible text on the screen and it carries the strongest proof. |
| Color and semantic use | 7 | Reader tiers are encoded well: red `rgb(180,35,42)` ANYONE, teal `rgb(10,95,103)` 1CLICK/NEAR, green `rgb(29,122,70)` NOBODY. But four more hues carry no meaning: blue primary `rgb(35,80,216)`, brown stamp `rgb(122,82,0)`, Zcash-yellow logo dot, slate-blue header pill. The public-value highlight `rgb(251,228,229)` is so pale the exposed values do not pop. `?live=1` renders "Live on mainnet" as a red pill, the same red that means "anyone can read this". |
| Spacing and layout | 7 | Clean 2-column grid, generous gutters. Left column ends at y~705 at 1440, leaving a dead lower-left quadrant beside the receipt. Receipt rows have identical rhythm, so nothing marks the private row as different in kind. |
| Component quality | 7 | Receipt is the best object on the page: perforated edges, dotted leaders, redaction bar, refusal codes pinned to the rows they guard. Stamp is a double CSS border rotated a degree; no ink texture, no seal shape, and it is a link to the Zcash tx with no link affordance. Refusal codes are solid black chips, the heaviest marks in the receipt, louder than the values they protect. |
| Responsive at 390 | 6 | No overflow and values drop under labels cleanly. But the receipt starts at y~680 of 844: the first screen is headline, lede, buttons, stamp, demand quote, and the itemization (the product's whole argument) is not visible. Meta line wraps to orphan "Zcash tx". Rotated stamp's outer border runs to ~6 px from the right edge. |
| Copy and argument | 8 | "Every bridge says private. SHORE itemizes it." is specific and earns the receipt. "7 links itemized · 3 guards enforced" is a good data headline. Demand line is sourced. Weak spots: "send to land" is clumsy; `TRANSPARENT_REFUSED`, `ADDRESS_REUSED`, `KEY_MISMATCH` are API strings shown to a wallet user without translation leading. |
| AI-slop (10 = none) | 7 | Content is non-generic. Tells: emoji in the document title ("🟢 SHORE: ..."); status pill beside the logo; standard SaaS hero split (headline + lede + primary/secondary button pair left, artifact card right); "Watch the 84 s run" appears twice on the first screen (header and hero). The replay flow below uses a different visual language (white step cards with green status chips, a dark navy tracker panel with gold numerals) than the till-roll first screen, so the page reads as two products stitched together. |

Average 7.0, min 6.

Design contract drift: root `DESIGN.md` specifies warm paper `#EFEBE1`, Instrument Serif, hazard orange, ink button;
`docs/design/DESIGN.md` (v2) specifies grey-green canvas and a lime accent. The render matches neither: cool grey canvas,
blue primary, pink highlight. Whichever is canonical, the other must go.

## Answers

1. **Designed product or styled document?** Designed product on the right, a styled landing page on the left. The receipt is an object made for this content; the left column is a template hero.
2. **Primary job obvious in 3 s?** At 1440 yes: headline plus "Shield my ZEC" says it. At 390 the job is clear but the proof (the receipt) is below the fold.
3. **Would a competent practitioner produce exactly this without thinking about the content?** The left column, yes. The receipt, no: reader tiers, refusal codes on guarded rows, and a redaction bar on the one private item only come from thinking about this crossing.
4. **Three highest-leverage fixes:**
   1. **Make the private row the payoff.** Change the summary to "6 public · 1 private" at 28 px, give the NOBODY row the only filled seal treatment on the page (green fill, larger label, the redaction bar sized like a value), and demote refusal-code chips to outlined. Today the argument's conclusion is the smallest row.
   2. **One palette with meaning.** Pick one contract and delete the other. Primary button goes ink (or the "shielded" colour), stamp loses the brown, "Live on mainnet" stops using exposure red, highlight saturates enough that exposed values read at a glance. Then carry the till-roll language into the replay flow so the first screen and the run below are one product.
   3. **390: show the receipt on the first screen.** Move stamp and demand line under the receipt (or collapse the demand line to one line), so at least the receipt header and first two rows sit above y=844. Also drop the duplicate header CTA and the emoji in the title.
