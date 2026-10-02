# AIRLOCK (Cross-Chain)

Read first: `../../CLAUDE.md`, your block in `../../docs/WIN-CONDITIONS.md`,
`../../research/tech/MECHANISMS.md` sections 1 and 5 (X1, X2, X3) with the probe outputs saved
beside it (`status_probe.out`, quotes, `oneclick-openapi.yaml` if saved), and
`../../research/demand/DEMAND.md` section 3. Handoff (`../handoff`) is reference only.

## Product

Swap into and out of shielded ZEC through NEAR Intents 1Click (https://1click.chaindefuser.com,
docs.near-intents.org) with the smallest public footprint, and prove the footprint.

- `airlock-1click` (Rust): typed client for `/v0/tokens`, `/v0/quote` (dry and real),
  `/v0/deposit/submit`, `/v0/status`. Hand-write the used endpoints from the OpenAPI file.
- Inbound (any token -> shielded ZEC): `EXACT_OUTPUT` quote whose recipient is a fresh
  Orchard-only UA (validator accepts it; Sapling-only is rejected). Watch the note land via
  lightwalletd. No unshield step.
- Outbound (shielded ZEC -> other chain): split the amount into ZIP 318 canonical denominations
  (`zcash_protocol::zip318`), one quote per part, each with its own one-time deposit t-address,
  fresh refund UA, destination address and isolated Tor circuit
  (`zcash_client_backend::tor::Client::isolated_client`), jittered send times. The single
  shielded-to-transparent hop per part is unavoidable; minimise and state it.
- Watchdog: polls every part to a terminal state (SUCCESS / REFUNDED / FAILED), timeout and
  escalation, and caps the value exposed to any one hold (part size).
- Exposure receipt: per swap, a table of what a chain observer, the unauthenticated `/v0/status`
  endpoint, and the solver can link. `airlock audit <swap-id>` recomputes it from live reads and
  counts shared identifiers across parts (target 0).
- Web app: swap form, live part status, receipt. Signing goes through COUNTERSIGN from 10-09
  (sign natively before that).

Measured finding to publish responsibly: unauthenticated `/v0/status?depositAddress=` returns
recipient and refundTo. Write it up for NEAR before publishing; ask the orchestrator first.

## Money

A real swap needs funds on both sides (ZEC for outbound, a small stablecoin balance on one
origin chain for inbound, e.g. Solana or Base USDC). Write exact asks to `.progress/FUNDING.md`
and message the orchestrator. Dry quotes and status reads need nothing; do them first.

## Day plan

| Date | Build | Acceptance (red then green) |
|---|---|---|
| 10-03 | Client + dry quotes for 5 origin assets into one Orchard-only UA; exposure analyzer over `/v0/status` | **B1** analyzer flags shared identifiers when two parts reuse one refund UA. Red: reuse the UA, count > 0; green: fresh UAs, count 0 |
| 10-04 | Splitter (ZIP 318) + Tor-isolated per-part quotes (non-dry, unfunded) | **B2** 3 non-dry quotes, 0 shared fields across their status payloads except asset ids |
| 10-05 | KILL GATE. First funded inbound swap into an Orchard-only UA | **B3** status SUCCESS and the note read from lightwalletd at nonzero height, amount >= quoted. Red: underpay 30%, expect REFUNDED |
| 10-06 | First funded split outbound swap; watchdog | **B4** every part terminal, destination balance delta read back on the other chain |
| 10-07..08 | Web app, receipt page; LIVE on a public URL by 10-08 | one swap completed from a phone |
| 10-09 | Switch signing to COUNTERSIGN | one mined tx signed in the browser |
| 10-10..11 | README, video script, judges | judge runs return no UNVERIFIED claim |

Register each acceptance as a progress criterion from the workspace root:
`progress --dir ../.. crit add airlock "B1-analyzer::cd projects/airlock && cargo test analyzer"`.

## Kill gate

2026-10-05: inbound does not land shielded, or 1Click stays paused (forum 57978 reports a pause
after a 2026-10-01 exploit, unverified). Narrow to the exposure analyzer and audit tool.

## Visual rules

- Always read @DESIGN.md before generating any UI.
- Do not invent colours, fonts, radii, or spacing outside DESIGN.md. Add new tokens to
  DESIGN.md first, in its existing format, with a computed contrast ratio if the token carries text.
- Use semantic tokens (`{colors.hazard}`, `{rounded.control}`, etc.), never raw hex, in components.
