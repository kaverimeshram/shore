# Design review r3 · SHORE ui-v2 Web3 pass (44c35b9 / a5d7ddd)

reviewer: fresh:grokbot-knowledgework (independent; builder notes not read)
date: 2026-10-12 07:41 IST · target https://ui-v2.shore-zec.pages.dev/shore/ (replay judged; live USD "not available" is ignored, as instructed)
method: manual review in the gstack /design-review scoring system (High = -1 letter, Medium = -1/2). The gstack skill itself was not run,
because it edits and commits code.
evidence: Playwright at 1440x900 and 390x844 (idle → connect → u1 → quote → approve → pending → landed → public), a composited contrast
audit of every visible text node in 16 states, a keyboard tab-order focus audit (idle and quote, both viewports), and a focus-ring timing probe.
Files are in docs/design/v2/fresh-r3/ (contrast.json, focus.out).

Design Score: B
AI Slop Score: C+ (unchanged visual kit: hero headline + kicker + numbered 3-step line, dark stat card beside a light stack of rounded cards
with Now/Waiting chips, pitch sections under the product. The Web3 detail makes the content specific, but the look is still a familiar template.)

| Category | Weight | Grade | Findings |
|---|---|---|---|
| Visual Hierarchy | 15% | B+ | Good: after connect, the live card leads with "Balance on Solana 0.0014 ZEC" (56px); then "0.00108, quoted, not sent", "32s pending", "84s confirmed". M: at 1440 the 4-line marketing hero pushes the live card and the Connect step to y≈370, so the live card is half below the fold at idle and connect. "Balance first" is first in the card, not on the page. |
| Typography | 15% | B+ | 13px floor holds (12px: 60 of 1,904 nodes). M: on 390 the quote's "0.0014 / ZEC" and "0.00108 / ZEC" still wrap onto 2 lines (m-3-quote.png). P: "(index / 0)" wraps. |
| Spacing & Layout | 15% | B+ | Flat, consistent. P: the right column is empty below the sticky card for most of the scroll. |
| Color & Contrast | 10% | A- | 0 AA text failures. Min 4.56:1 (r2: 4.27). Blue action, teal pending, gold shielded, amber fee, green done, plus network chips (Solana mainnet blue-gray, Zcash mainnet gold). |
| Interaction States | 10% | B+ | Fixed: the focus ring is now 3px solid #2350D8 with offset 3px (6.5:1 on white, 5.7:1 on page), and 3px ink on outline buttons. M: on primary buttons the ring FADES IN over 150ms from near-white (outline-color is in the button's transition-property; measured rgb(248,249,254) → rgb(144,167,235) → rgb(35,80,216) at 0/50/100ms). Focus must appear instantly (hallmark slop gate 15). Remove outline-color from the transition list. |
| Responsive | 10% | B | Fixed: one ETA, outlined header CTA after start, "4 in general" on one line. M: on 390 the sticky Lands/Fee/Elapsed strip and the dark live card show the same numbers at once after connect (m-1-connected.png: strip "– – –" above a card reading 0.0014). M: after connect the page scrolls back so the 7-line marketing hero sits above the balance. Page 3,001px. |
| Content Quality | 10% | B+ | Good: plain-language preview "You send 0.0014 ZEC (~$1.70) from Phantom. About 0.00108 ZEC arrives shielded in about 135s. Fee $0.39 (22.9%), all to 1Click and the Zcash network." with network chips and a "USD from the 1Click quote, recorded quote, 10 Oct 2026" source line. P: "Confirmed, send to land" and "Lands shielded, quoted" read like field names. P: the arrival line has no USD. M: Why now / Business model pitch still lives in the product. |
| AI Slop | 5% | C+ | See headline. |
| Motion | 5% | B+ | Band with ticks and head, teal → gold, timestamps under the band. |
| Performance Feel | 5% | B | Fast, no jank. |

## WCAG AA (re-measured)
- Text: 0 failures across 16 states. Min contrast 4.56:1. (r2 fails fixed: the "pending" chip and the Readable-by rows now pass.)
- Focus visibility: every tab stop has a 3px outline. Contrast vs adjacent background: 5.73-17.82:1 for links, outline buttons and summaries. Blue primaries reach 6.54:1 after a 150ms fade, starting at 1:1 (white on white).

## Web3 direction check
| Ask | Verdict |
|---|---|
| Balance first, one action | PARTIAL. Balance is the live card's hero after connect, with one blue action per step. But the marketing hero precedes it on both viewports. |
| Plain-language preview with USD | GOOD. Send/arrive/fee in a sentence, USD on send and fee, with source and date. Arrival lacks USD. |
| Pending / confirmed tracker | GOOD. not started → quoted, not sent → pending (elapsed, ETA) → confirmed/shielded, mirrored in step chips. |
| Network badge | GOOD. "Solana mainnet", "Zcash mainnet" chips in the preview, network under each stage, "Replay of a mainnet run" in the header. |
| Timestamps and tx links | GOOD. "sent 10 Oct, 09:22 IST", "Confirmed on Zcash mainnet at height 3,512,504, 10 Oct, 09:24 IST", "recorded 10 Oct, 09:22 IST" on the signature, Solana/NEAR/Zcash tx links with Copy. |
