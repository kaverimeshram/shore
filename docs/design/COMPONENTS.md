# Components

- Step register: originui Stepper, vertical with inline titles and descriptions. https://21st.dev/r/originui/stepper (preview https://cdn.21st.dev/user_originui/stepper/vertical-with-inline-titles-and-descriptions/preview.png). The registry source needs a 21st.dev API key, and this app is plain HTML with no React, so the anatomy (28px numbered indicator, inline title, always-visible description, done state as a check) is rebuilt in web/shore/shore.css (.step) with DESIGN.md tokens.
- Buttons: shadcn/button sizing (`uicraft code shadcn/button`: h-9/h-10, px-4, rounded-md, focus ring), scaled to 48/54px touch targets, teal primary, outlined ghost.
- Separators: shadcn/separator (https://ui.shadcn.com/r/styles/new-york/separator.json) as 1px rules (.kv rows, .public rows, .sect tops).

## v2 (2026-10-11)
FALLBACK: shadcn/ui is React-only and SHORE is framework-free JS, so its button/card/badge/input variants are ported as Tailwind v4 `@layer components` recipes in web/shore/tw.css (.btn/.ghost/.btn-sm, .card/.card-soft, .chip-*, input), plus an address pill with Copy.
