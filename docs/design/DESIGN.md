# SHORE design contract, v2 (2026-10-11)

Supersedes the round-3 "Harbor board" (kept as `docs/design/v2/DESIGN-v1-harbor.md`).

## Contract (written before code)
1. **Direction:** a money-transfer product, adapted from `~/.config/design-md/designs/wise.md`: soft grey-green canvas,
   white raised cards, heavy near-black display numbers, one lime-green accent that means "shielded / go". It reads like
   a payments app (Wise, a bank transfer screen), not a document. Unlike the other CWF products: sealed-call (Space
   Grotesk + JetBrains, dark), monday-open (Fraunces serif + Plex), leash (Sentry purple, dark), backed (Inter).
2. **Fonts (self-hosted woff2 from @fontsource):** display Bricolage Grotesque 600/800 (metrics, headings); body
   Figtree 400-700 (a humanist grotesk, no book serif anywhere); mono Geist Mono 400/500 for addresses, hashes and
   amounts with tabular numerals. Inter/Roboto/system faces are not used.
3. **Palette (CSS variables below):** canvas #e8ebe6, card #ffffff, ink #0e0f0c, body #454745, mute #6b6d6a, accent
   lime #9fe870 on ink, positive #2ead4b, warning #ffd11a / #b86700, negative #d03238. Lime is the only accent; status
   uses semantic chips with text.
4. **Spacing and shape:** 4/8px scale (4 8 12 16 24 32 48 64); radius 10 (inputs), 16 (cards), 999 (buttons, chips);
   one shadow token for cards (0 1 2 + 0 8 24 at low alpha), none heavier. Type scale 12/14/16/20/28/40/64.
5. **Layout:** a top app bar; a hero row (headline + judge card left, the live crossing card right); a row of four
   metric cards (Balance, Lands, Fee, Elapsed); then a two-column workspace: the five steps as cards with status chips
   (left, 7/12) and the read-back and "what's still public" panels (right, 5/12). Mobile stacks: crossing, metrics 2x2,
   steps, panels. One primary (lime pill) button per state; secondary is an outlined pill; links are ink with an underline
   offset.

Built with Tailwind CSS v4 (CLI, `web/shore/tw.css` -> `web/shore/shore.css`) and shadcn-style component recipes
(button / card / badge / input variants as `@layer components`) because the app is framework-free JS; see COMPONENTS.md.

## Tokens
```css
:root{
  --canvas:#e8ebe6; --card:#ffffff; --card-2:#f4f6f2; --ink:#0e0f0c; --ink-deep:#163300; --body:#454745; --mute:#6b6d6a;
  --line:#d6dad2; --accent:#9fe870; --accent-pale:#e2f6d5; --accent-ink:#163300;
  --positive:#2ead4b; --positive-pale:#dcf3e1; --warning:#ffd11a; --warning-ink:#7a4a00; --warning-pale:#fff3c4;
  --negative:#d03238; --negative-pale:#fbe3e3; --info-pale:#e3f2fb; --info-ink:#0b4a6b;
  --radius-input:10px; --radius-card:16px; --radius-pill:999px;
  --shadow-card:0 1px 2px rgba(14,15,12,.06),0 8px 24px rgba(14,15,12,.06);
  --font-display:"Bricolage Grotesque",ui-sans-serif,sans-serif; --font-body:"Figtree",ui-sans-serif,sans-serif;
  --font-mono:"Geist Mono",ui-monospace,monospace;
}
```

## States -> chips
locked = grey "Waiting", active = lime "Now", done = green with the step's tag (connected, fresh, yours, quoted, signed,
landed); warnings (fee over 10%, "readable by anyone") = amber or red chips with text.
