# Dry quotes, 5 origin assets into one Orchard-only UA (2026-10-03)

Command: `airlock dry` (src/main.rs). EXACT_OUTPUT 0.01 ZEC (1,000,000 zat), `dry:true`,
`recipient` = `Wallet::orchard_ua(0)` from a local seed (receivers read back: `["orchard"]`).
Endpoint `POST https://1click.chaindefuser.com/v0/quote`, no API key. Read 2026-10-02 18:4x UTC.

| Origin | amountIn | minAmountIn (base units) | timeEstimate |
|---|---|---|---|
| USDC sol | 13.466806 | 13332137 | 135 s |
| USDC base | 13.452743 | 13318215 | 150 s |
| USDC arb | 13.424271 | 13290028 | 140 s |
| USDC eth | 13.454441 | 13319896 | 160 s |
| BTC | 0.00015995 | 15835 | 920 s |

All five: HTTP 201, one Orchard-only UA accepted as `recipient`.

Measured failures (same command, same UA):

- USDT tron at 0.01 ZEC: 400 `Temporary swap limits: minimum swap amount is $100`.
- USDT tron at 0.1 ZEC (about $131): 400 `Internal server error`. The tron route does not quote
  into ZEC today. Not retried with another refund address, so the cause (route or refund format)
  is not separated.
- A $100 floor appears on one route only; the four other routes quote at $13.
  Not established: whether the tron floor is the post-2026-10-01 pause (forum 57978).

Blind spot: dry quotes validate format and price, they do not prove delivery. B3 does.
