# SHORE first screen, design review round 6 (fresh critic, demo-judge)

Evidence: r12-1440.png, r12-390.png, r11-390-full.png, plus a fresh 1440x2400 capture of
http://127.0.0.1:8642/shore/ (HTTP 200). Markup read from web/shore/index.html and shore.js.

## Gate zero

- Verb the brief names: CWF Zcash track, "startups integrating ZEC into their products"; judge
  signal "utilizing ZEC the asset". Zecathon: "Would someone use this on Monday", "Working beats ambitious".
- Code path that performs it: web/shore/shore.js:309 `signAndSendTransaction(tx)` (Phantom send to
  the 1Click deposit address), src/oneclick.rs:172 `/v0/deposit/submit`. Receipt cites a mainnet
  read-back at block 3,512,504.
- What won last time: not verified in this review (no prior Zcash-track winner list read).
- VERDICT: ELIGIBLE. The product moves ZEC; it is not read-only.

## Scores

| Axis | Score | Evidence |
|---|---|---|
| Visual hierarchy | 7 | Headline (about 52px, 3 lines) then receipt outcome `0.00115 ZEC landed` (about 34px) then 7 rows. Clear order. But the left column ends at y~520 at 1440 and leaves ~380px of empty grey beside the receipt. |
| Typography | 7 | Grotesk display, sans body, mono for chain data: a correct three-voice system. Uppercase mono red `ANYONE HOLDING THE DEPOSIT ADDRESS` is the loudest text in the receipt and outshouts the row label. |
| Color and semantic use | 8 | Pink highlight = public value, red caps = ANYONE, teal = 1CLICK/NEAR, green = NOBODY, black redaction bar = private, black chips = refusal codes. Color carries state. Blue primary button and brown "ZEC landed" sit outside that grammar. |
| Spacing and layout | 6 | Dead left column at 1440. Below the fold: four identical "Waiting" step cards at uniform rhythm, then a dark Crossing panel whose Lands/Fee/ETA show bare dashes. |
| Component quality | 7 | Refusal-code chips pinned on the rows they protect and the redaction bar are real, content-specific components. The receipt itself is a stock white rounded card with a dashed divider; DESIGN.md promises thermal paper, dotted leaders and a rubber-stamp seal, none of which render. |
| Responsive at 390 | 7 | Hero, both CTAs, demand line and the receipt outcome fit in the first 844px. Rows stack value over label cleanly. Recipient row wraps into 4 lines of mixed mono/sans; the dark Crossing panel appears before the Connect step it reports on. |
| Copy and argument | 8 | "Every bridge says private. SHORE itemizes it." is a claim a stranger can test, and the receipt tests it on the same screen. Demand line is sourced. The hero figure is a dust amount (0.00115 ZEC), which reads as a test transfer, not usage. |
| AI-slop (10 = none) | 7 | First screen is specific. Tells: duplicate "Watch the 84 s run" in nav and hero; header pill "Replay of a mainnet run" contradicting the "Shield my ZEC (live mainnet)" CTA; generic vertical stepper with identical Waiting pills; dashes as empty-state values. |

Average 7.1, min 6.

## Answers

1. Designed product or styled document? A designed product on the first screen: the receipt is
   data presented as data, with state encoded in color and refusals attached to rows. Below the
   fold it falls back to a generic stepper.
2. Primary job obvious in 3 seconds? Mostly. "Shield my ZEC" plus the receipt says what happens and
   what it proves. The header pill "Replay of a mainnet run" muddies whether this is live, and
   "Shield my ZEC" reloads to `?live=1#flow` where the user meets two more buttons ("Use it live
   (mainnet)" / "Connect recorded wallet") before anything happens.
3. Would a competent practitioner produce exactly this without the content? The receipt, no: the
   who-can-read column, redaction bar and refusal codes only come from knowing the exposure model.
   The card shell, two-column hero and the stepper below, yes.

## Three highest-leverage fixes

1. Put the action in the dead left column. At 1440 the Connect Phantom step (currently y~940) should
   sit directly under the CTAs, so "Shield my ZEC" connects in place instead of reloading and
   scrolling to a second choice. Drop the nav duplicate of "Watch the 84 s run" and make the header
   pill state the mode the CTA uses.
2. Make the receipt the till roll DESIGN.md describes: perforated top/bottom edge, dotted leaders
   from label to value, a stamp for "READ BACK 3,512,504" with a live tip delta ("tip N, M blocks
   since"). Today it is a white card any template ships; the object should be the brand.
3. Stop leading with a dust figure. 0.00115 ZEC as the largest number signals a test. Either lead
   the outcome line with what is impressive and scale-free ("84 s, Phantom to shielded, 1 approval,
   7 links itemized, 3 guards enforced") with the amount secondary, or record a run at a size a holder
   recognises (operator money sign-off).

Single highest-leverage fix: move the Connect step into the empty left column so "Shield my ZEC"
acts on the first screen.
