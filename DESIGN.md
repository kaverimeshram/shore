---
version: alpha
name: AIRLOCK
description: A ledger and receipt interface. The page is a sheet of warm paper; the product is a document that shows exactly what a stranger can read about a swap. Exposed values are struck with a hazard-orange highlighter, unlinked ones carry a green seal. Data is monospaced, headlines are a condensed serif, nothing glows.

colors:
  paper: "#EFEBE1"
  paper-deep: "#E4DFD1"
  ink: "#14120F"
  ink-soft: "#4A453C"
  rule: "#14120F24"
  hazard: "#E4501B"
  seal: "#1F6B4F"
  on-hazard: "#14120F"
  on-seal: "#EFEBE1"

typography:
  display:
    fontFamily: "Instrument Serif"
    fontWeight: 400
    fontSize: "clamp(44px, 7.2vw, 104px)"
    lineHeight: 0.95
    letterSpacing: "-0.02em"
  heading:
    fontFamily: "Instrument Serif"
    fontWeight: 400
    fontSize: "clamp(28px, 3.4vw, 44px)"
    lineHeight: 1.05
  body:
    fontFamily: "IBM Plex Sans"
    fontWeight: 400
    fontSize: "17px"
    lineHeight: 1.55
  data:
    fontFamily: "IBM Plex Mono"
    fontWeight: 400
    fontSize: "14px"
    lineHeight: 1.5
  label:
    fontFamily: "IBM Plex Mono"
    fontWeight: 500
    fontSize: "11.5px"
    letterSpacing: "0.06em"
    textTransform: uppercase

rounded:
  none: "0px"
  control: "2px"

spacing:
  unit: "4px"
  gutter: "24px"
  section: "clamp(72px, 10vw, 140px)"

components:
  sheet:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink}"
  receipt:
    backgroundColor: "{colors.paper-deep}"
    textColor: "{colors.ink}"
    rounded: "{rounded.none}"
    note: "narrow column, 440px max, perforated top and bottom edge made of a radial-gradient mask, dotted leaders between key and value"
  exposed:
    backgroundColor: "{colors.hazard}"
    textColor: "{colors.on-hazard}"
    note: "highlighter fill behind a value anyone can read. Never used as text colour on paper (3.23:1)."
  unlinked:
    backgroundColor: "{colors.seal}"
    textColor: "{colors.on-seal}"
    rounded: "{rounded.control}"
    note: "round seal mark with the shared-identifier count. Used only when the live count is 0."
  button-primary:
    backgroundColor: "{colors.ink}"
    textColor: "{colors.paper}"
    rounded: "{rounded.control}"
    padding: "14px 22px"
  input:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    rounded: "{rounded.none}"
    note: "bottom rule only, 2px ink, mono data type, no box"

motion:
  highlighter: "background-size 0 to 100% on exposed values, 420ms, cubic-bezier(.2,.7,.1,1), staggered 70ms per field"
  chamber: "part cells fill left to right one at a time, 260ms each, to show one hold at a time"
  reduced: "all motion off under prefers-reduced-motion; values render already highlighted"

rules:
  - One accent hue per state. Hazard means "readable by anyone". Seal means "measured zero". Nothing else is coloured.
  - Off-black ink, warm paper. Never #000, never #fff.
  - No icons except a text arrow glyph. No illustration. The receipt is the illustration.
  - Fonts are self-hosted woff2 under web/fonts. No third-party requests at load.
  - Numbers that come from the live service are never typed into markup. Every figure on screen carries the call that produced it.
---

# AIRLOCK design notes

The user pastes one deposit address and sees, in a receipt, what any stranger could read. That
single screen is the product. The rest of the site supports it: a planner that splits an exit
into denominations, and a live receipt for a swap in progress.

Source of the look: a pharmacy till roll crossed with a redaction log. Thermal paper, dotted
leaders, rubber-stamp seal. The hazard highlighter is the same gesture as someone marking up a
leaked document.
