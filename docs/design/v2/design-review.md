# Design review · SHORE ui-v2 ("wise" design system)

reviewer: fresh:grokbot-knowledgework (independent; builder critique and DESIGN.md not read)
date: 2026-10-11 22:32 IST
target: https://ui-v2.shore-zec.pages.dev/shore/ (replay, plus ?replay=1&tour=1&speed=3), branch ui-v2 @ f515c53
method: I did NOT run gstack /design-review. It is installed only as a Claude Code skill (~/.claude/skills/gstack/design-review), with no
standalone CLI. Running it means launching a second agent (`claude` is aliased to --dangerously-skip-permissions), and by design that skill
edits and commits fixes, which would change the code under review. So this report is a manual review using the gstack scoring system
(10 weighted categories, A-F; each High finding drops a letter, each Medium drops half).
evidence: my own Playwright run at 1440x900 and 390x844 (idle → connect → fresh u1 → quote → approve → crossing → landed → public), the tour
mode, and a computed-contrast audit of every visible text node in 16 states. Shots and contrast.json are in docs/design/v2/fresh/.

Design Score: C+
AI Slop Score: C (a competent Wise-template clone; the nested cards the owner rejected in r0 are back)

| Category | Weight | Grade | Findings |
|---|---|---|---|
| Visual Hierarchy | 15% | C | H: during crossing, no elapsed or progress number is on screen. The dark hero card and the metric row are scrolled off above (d-5-crossing.png, scrollY≈1000). H: at landed, the "0.00115 ZEC" read-back hero is clipped under the sticky header at 1440 (d-6-landed.png; same in the builder's after/readback-1440.png). M: at idle, the primary "Connect recorded wallet" sits below the fold at 1440 (card top ≈ y620) under an explainer panel and 4 metric cards. |
| Typography | 15% | B | Bricolage Grotesque 800 display / Figtree / Geist Mono for addresses is a good pairing, and addresses are legible. M: 12px carries 404 of 1,862 rendered text nodes across states (second only to 14px at 816), including all metric labels, captions, chips and "Readable by" rows. M: idle metric caption truncates ("ZEC sent, last mainnet r…") at 390. |
| Spacing & Layout | 15% | C | H: cards inside cards. The "What you are seeing" panel holds 3 bordered tiles, "Why now" holds 4 tiles, the quote card holds a bordered send→lands panel plus 2 bordered disclosure boxes. M: the right column is a stub ("Read-back… shows the amount here", "What's still public") beside a 5-card left stack, leaving uneven holes until landed. |
| Color & Contrast | 10% | C | H: 3 AA failures (below). M: the palette is Wise's own brand (#9FE870 bright green, #163300 forest, #0E0F0C ink, all literal in tw.css). The Zcash gold identity from r3 is gone. M: lime does 7 jobs (brand dot, header CTA, every step primary, "Now" chip, progress fill, landed border, landed tint). |
| Interaction States | 10% | B | Good: 2px solid focus outline (rgb 18,34,6), Copy on every address pill, a status chip per step (connected / fresh / quoted / signed / Now / Waiting / landed), "Change" links correctly gone after signing. M: the header lime "Try it in 60s" pill stays visible next to each step's lime primary, so two primaries are on screen at almost every scroll position. |
| Responsive | 10% | C+ | M: 390 page is 3,943px at landed. The explainer and the pitch panels are stacked below the flow, the Connect step is ≈1.2 screens down at idle, and "84s" is duplicated in the first viewport. M: "0.0014 / ZEC" wraps in the send→lands panel. Landed scroll is correct on mobile (number fully visible). |
| Content Quality | 10% | C | M: pitch content ("Why now", "Business model", "Launchpads (Sapling, SnarksPad, Billet) are where you speculate") inside the product surface. M: hero card at landed captions 0.00115 ZEC with "send to land, recorded on mainnet", which is the elapsed caption. P: "4 in general". |
| AI Slop | 5% | C | Wise clone, numbered 3-equal-tile explainer (slop gate 3), nested cards (gate 4), uppercase tracked micro-labels on every metric, the stock dark "hero stat" card. |
| Motion | 5% | B | Progress bar fill, chip state changes, tour mode. The idle hero band is lime at 35% opacity on black, which reads olive/muddy. |
| Performance Feel | 5% | B | Fast load. Tailwind v4 output plus 3 webfonts; no layout jank observed. |

## WCAG AA contrast (measured, composited against the real ancestor background)
- ink #0E0F0C on page/card: 17-19:1 PASS. Muted metric labels rgb(107,109,106) on white: 5.2:1 PASS. Waiting step titles rgb(107,109,106) on the waiting-card tint: >=4.5:1 PASS (no failures in the audit).
- FAIL "Readable by: anyone …" rows: 12px/600, 4.10:1 (need 4.5).
- FAIL footer "No custody, no account. Built on AIRLOCK.": 14px/400, 4.34:1 (need 4.5).
- FAIL metric placeholder "–" (Lands/Fee/Elapsed before data): 40px/800, 2.03:1 (need 3.0 for large text).
- min contrast across all 16 audited states: 2.03.

## Against the outside reviewer's approved direction (BACKED v3 reference)
| Requirement | Verdict |
|---|---|
| Metric cards | PRESENT (Balance / Lands / Fee / Elapsed), but placed above the flow so they are off-screen while the user acts. At idle they show last-run numbers in the same style as live values. |
| Panels | PRESENT, but nested 2-3 deep (the r0 defect). |
| Status chips | GOOD. A consistent chip per step and on the hero card. |
| One clear primary action | PARTIAL. One lime primary per step, but the always-visible lime header pill competes with it. |
| Deliberate status color system + secondary accent | PARTIAL. Lime/forest = active/positive, amber = fee, red = exposed, blue = replay chip. But lime is overloaded, amber is used once, and the system is borrowed from Wise. |
| Polished progress/chart visuals | WEAK. The crossing progress is a flat 8px bar with 3 text labels, no stop ticks and no time. The hero band is a 35%-opacity bar. BACKED has a real equity chart, loss line and gauge; SHORE has no chart. |
| Type scale + WCAG AA | PARTIAL. The scale is coherent (12/14/18/24/40/56) but leans on 12px for all labels/captions, with 3 AA failures. |
