# SHORE design review, round 8 (fresh critic, demo-judge)

Captures read: docs/design/r16-1440.png, r16-390.png, r16-390-full.png, plus a fresh
`?live=1` capture at 1440x900 (/tmp/r8-live.png). Live and replay first screens are identical
apart from the header badge ("Live on mainnet" vs "Replay of a mainnet run").

## Gate zero

Verb the brief names: CWF Zcash track, "startups integrating ZEC into their products";
judge signal "utilizing ZEC the asset". Zecathon Cross-Chain: move value across chains privately.
Code path that performs it: web/shore/shore.js:309 `signAndSendTransaction` (Phantom), with
web/shore/api.js:53 `/api/quote`, :76 `/api/status/`, :88 `/api/readback/`. The receipt links a
mainnet Solana tx and Zcash tx and states a read-back at block 3,512,504.
VERDICT: ELIGIBLE. The product moves ZEC; it is not read-only.

## Scores

| Axis | Score | Evidence |
|---|---|---|
| Visual hierarchy | 7 | Largest element (H1, ~52px) is a claim, not data. The receipt's verdict "6 public . 1 private" sits at y~817 of 900 at ~18px, the last thing read. "84 s" stamp is the only data hero and it is in the left column below the CTAs. |
| Typography | 7 | Grotesk display + mono for addresses and attributions is right for a till roll. Mono metadata ("Solana explorer", "unauthenticated /v0/status") is ~12px grey and near the floor. Sans row labels next to mono chips reads as two systems not one roll. |
| Color and semantics | 7 | Blue only on the CTA: correct. Gold row for the private line: correct and the best moment on the page. Red "ANYONE" plus red-tint chips on 5 of 7 rows reads as error state rather than highlighter. Teal, defined as "in flight", labels the 1CLICK/NEAR party and its "value . recipient . memo" chip: semantic drift. |
| Spacing and layout | 8 | Two-column desktop balances; receipt rows have even rhythm, dotted leaders align. Left column ends ~y700 leaving dead space under the demand quote while receipt runs to the fold. |
| Component quality | 8 | Zigzag receipt edges, dotted leaders, guard code chips (TRANSPARENT_REFUSED, ADDRESS_REUSED, KEY_MISMATCH) with plain-English reasons, double-rule rotated stamp. These are rendered nouns, not decoration. |
| Responsive at 390 | 6 | First screen shows H1, CTAs and 3 red ANYONE rows. The landing proof (84 s, block 3,512,504) is pushed to ~y1560 and the crossing panel to ~y1880. Header meta wraps "Solana tx . / Zcash tx" mid-list. Dotted leaders vanish, chips stack under labels. On a phone the page is all exposure and no proof. |
| Copy and argument | 7 | "Every bridge says private. SHORE itemizes it." is a real argument. The 86,000-wallets line with a dated source is strong. Row copy leans on protocol vocabulary (u1, t-address, /v0/status, delivery tx has transparent inputs) without a one-line gloss for a Phantom user. |
| AI-slop (10 = none) | 8 | Split hero (text left, card right) and a pill badge in the header are the stock pattern. Everything inside the card is specific to the content. No gradients, no three-feature grid, no emoji. |

Average 7.3, min 6.

## Questions

1. Designed product or styled document? Designed product on desktop: the receipt is an object
   with its own grammar. At 390 it degrades toward a styled document, a long scroll of rows.
2. Primary job obvious in 3 s? Half. "Shield my ZEC" and "One Phantom approval moves your
   Solana ZEC into shielded Zcash" land in 3 s. The H1 sells the audit, not the move, so a
   cold reader first thinks "privacy analyzer", then reads the subhead.
3. Would a competent practitioner produce exactly this without thinking about the content? No.
   The shell is generic (split hero), but the receipt with per-row "who can read it", the
   refusal codes, and the single gold private row cannot be produced without the content.
4. Three highest-leverage fixes:
   - **390 proof order.** On mobile, put the stamp (84 s, 0.00115 ZEC, block 3,512,504)
     directly under the CTAs and above the receipt, or collapse receipt rows to one line each
     (label + ANYONE/NOBODY) with detail on tap. Today the first phone screen has zero proof
     the note landed.
   - **Verdict first.** Move "6 public . 1 private" into the receipt header beside or instead of
     "7 links itemized . 3 guards enforced", at the 32px numeral size already used for "7" and
     "3". The answer to the page's question is currently its last line.
   - **Fix teal and live status.** Teal is "in flight"; give the solver/counterparty a neutral
     ink or its own token, keep teal for the crossing panel. In `?live=1`, the badge says
     "Live on mainnet" over a 10 Oct receipt with no tip height or "read Ns ago"; add a
     tip/freshness indicator next to the badge so "live" is evidenced, not asserted.

Single highest-leverage fix: on 390, put the landed-shielded stamp above the receipt so the
first phone screen carries proof, not only exposure.
