# Inbound landing gap (2026-10-10)

## What is proven

The inbound quote path is live and accepts a fresh Orchard-only unified address.

Command (from `zcash/projects/airlock`, uses existing `work/seed.hex`, no new secrets):

```bash
cargo run -- inbound 0.02 \
  nep141:base-0x833589fcd6edb6e08f4c7c32d4f71b54bda02913.omft.near \
  0x2527D02599Ba641c19FEa793cD0F167589a0f10D
```

Measured output (2026-10-10, Tor bootstrap ~129 s):

```
inbound 0f9def50dc17
  send 25.178605 (min 24675032) of nep141:base-0x833589fcd6edb6e08f4c7c32d4f71b54bda02913.omft.near
  to 0x67870BefaCcA10a39B20B7047aC6420108ddc200
  lands in u1sk5ua22x6vh49qhvp6vc4s8z6tnd5h3cvg6lcvnvf602e0mctcdwsvhq0dnn32cneqyes9d3kj6xdt2ecvfkmw35ja8dql2q9ylun83s
  deadline 2026-10-13T05:31:33.000Z
```

Saved record: `work/inbound/0f9def50dc17.json`.

Status read before any payment:

```bash
cargo run -- inspect 0x67870BefaCcA10a39B20B7047aC6420108ddc200
```

Returns `PendingDeposit` with `recipient` equal to the Orchard-only UA above and `refund_to` equal to the Base refund address in the quote command.

## What is not proven

A NEAR Intents 1Click solver payout into that UA after an origin-chain deposit. A quote or `PendingDeposit` is not landing.

The testnet note `a37a3dbc29b5093aee3f016278321e8ab999d0f1be223d6fbf8abd511c24cd39` at height 4,435,638 was a COUNTERSIGN self-send on testnet (`airlock fund test ...`), read back by `airlock note test`. It is not evidence that 1Click delivered shielded ZEC to a quoted recipient.

## Blocker

**Missing input:** Base mainnet USDC sent from an operator-controlled wallet to the deposit address `0x67870BefaCcA10a39B20B7047aC6420108ddc200`.

| Field | Value for quote `0f9def50dc17` |
|---|---|
| Pay asset | USDC on Base (`nep141:base-0x833589fcd6edb6e08f4c7c32d4f71b54bda02913.omft.near`) |
| Pay amount | **25.178605 USDC** (minimum **24675032** base units, 6 decimals) |
| Deposit address | `0x67870BefaCcA10a39B20B7047aC6420108ddc200` |
| Refund if underpaid | `0x2527D02599Ba641c19FEa793cD0F167589a0f10D` (must be the sender's Base wallet) |
| Expected shielded recipient | `u1sk5ua22x6vh49qhvp6vc4s8z6tnd5h3cvg6lcvnvf602e0mctcdwsvhq0dnn32cneqyes9d3kj6xdt2ecvfkmw35ja8dql2q9ylun83s` |
| Expected note value | **2,000,000 zat** (0.02 ZEC), EXACT_OUTPUT |
| Pay before | `2026-10-13T05:31:33.000Z` (this quote's deadline; a new quote gets a new address and deadline) |

**Who must fund:** the operator's own Base wallet. AIRLOCK holds no origin-chain private key (`LAUNCH-REQUEST.md`, `.progress/LAUNCH-REQUEST.md`). Repository env has no funded Base or USDC wallet: `AIRLOCK_MAINNET_FUND` is unset and only gates Zcash mainnet sends from `work/seed.hex`, not Base USDC.

**Why testnet cannot substitute:** 1Click rejects testnet Orchard UAs as inbound recipients (measured 2026-10-03, `.progress/SECURITY-REVIEW.md`). Inbound proof requires a mainnet origin payment.

This agent did not create keys, spend USDC, or broadcast a fake transaction.

## Post-conditions that prove landing (not a quote)

Run from `zcash/projects/airlock` after the Base transfer confirms.

1. **1Click terminal status** (solver accepted the deposit and attempted delivery):

   ```bash
   cargo run -- inspect 0x67870BefaCcA10a39B20B7047aC6420108ddc200
   ```

   Pass: status `SUCCESS` (or `REFUNDED` / `FAILED` with an honest failure report). Fail: stuck at `PendingDeposit` after the deadline.

2. **Shielded note on Zcash mainnet** (view-only UFVK scan, same wallet seed as the quote):

   ```bash
   cargo run -- note main 2000000 3600
   ```

   Pass: a line `LANDED` with `value_zat >= 2000000` and `mined_height: Some(h)` where **h > 0**, for a note decryptable to the quoted recipient UA. Fail: `no note of at least 2000000 zat within 3600s`.

Optional cross-check: `cargo run -- tx main <delivery-txid>` if 1Click exposes a Zcash delivery txid in `/v0/status`.

## Library tests

```bash
cargo test --lib
```

33 passed, 0 failed (2026-10-10).
