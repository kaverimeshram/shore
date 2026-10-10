# SHORE HTTP API (contract v1)

Base: `airlock serve [addr]` (default `127.0.0.1:8787`). All bodies JSON. Errors are
`{ "error": "<message>", "code": "<MACHINE_CODE>" }` with a 4xx/5xx status.
Amounts are strings of base units (Solana-ZEC and Zcash both use 8 decimals, 1 ZEC = 100,000,000).
Every figure the UI shows comes from one of these calls; nothing is typed into markup.

Flow: `address` -> `quote` -> (user approves one Phantom SPL transfer to `depositAddress`)
-> poll `status` -> `readback` -> `exposure`.

## POST /api/shield/address

Creates a shield session and its recipient.

Request, either:
- `{ "mode": "fresh" }` : the server derives the next never-used Orchard-only `u1` of the demo wallet
  (persistent `work/ua-index`, incremented before the address is returned, so no index is handed out twice).
- `{ "mode": "own", "address": "u1..." }` : the user's own address.
  Refused: transparent `t1/t3` (`TRANSPARENT_REFUSED`), Sapling `zs` (`NOT_UNIFIED`), a UA with no
  shielded receiver (`NO_SHIELDED_RECEIVER`), any address already used by an earlier shield on this
  server (`ADDRESS_REUSED`, HTTP 409). Revision 2 `zu1` is re-encoded to `u1`.

Response 200:
```json
{ "id": "a1b2c3d4e5f6", "recipient": "u1...", "receivers": ["orchard"],
  "source": "fresh" | "own", "uaIndex": 224, "readbackAvailable": true }
```
`uaIndex` is present only for `fresh`; `readbackAvailable` is true only for demo-wallet addresses.

## POST /api/quote

Request: `{ "id": "<shield id>", "amount": "140000", "sender": "<Solana pubkey of the Phantom wallet>", "dry": false }`

- `amount`: base units of ZEC on Solana (SPL mint `A7bdiYdS5GjqGFtxf17ppRHtDKPkkRqbKtR27dxvQXaS`).
  Minimum `133334` (`BELOW_MINIMUM`, the 1Click floor measured 2026-10-10); maximum `2000000` (launch cap, `ABOVE_CAP`).
- `sender` is the refund address (refunds go back to the Phantom wallet).
- `dry: true` returns amounts with no deposit address.

1Click call: `EXACT_INPUT`, origin `1cs_v1:sol:spl:A7bdiYdS5GjqGFtxf17ppRHtDKPkkRqbKtR27dxvQXaS`,
destination `nep141:zec.omft.near`, recipient = the session's u1. The server refuses a quote whose
echoed request differs from what it asked for (`ECHO_MISMATCH`).

Response 200:
```json
{ "id": "...", "depositAddress": "DMDd...SjbM", "mint": "A7bd...XaS", "decimals": 8,
  "amountIn": "140000", "amountInFormatted": "0.0014",
  "amountOut": "108000", "amountOutFormatted": "0.00108", "minAmountOut": "106920",
  "fees": { "withdrawFee": "32000", "refundFee": "844", "totalFeeBps": 2285 },
  "minAmount": "133334", "deadline": "2026-10-13T06:47:48.000Z", "timeEstimateSecs": 135,
  "recipient": "u1...", "correlationId": "..." }
```
The UI builds one SPL `transferChecked` of `amountIn` to `depositAddress` for the user to approve.
Optional after signing: `POST /api/shield/:id/tx { "txHash": "<solana sig>" }` notifies 1Click (`/v0/deposit/submit`) to speed detection.

## GET /api/status/:deposit

Polls 1Click `/v0/status` and checks the Zcash side.
```json
{ "depositAddress": "...", "status": "PENDING_DEPOSIT|KNOWN_DEPOSIT_TX|PROCESSING|SUCCESS|REFUNDED|FAILED|INCOMPLETE_DEPOSIT",
  "terminal": false, "updatedAt": "...",
  "solanaTx": "2LLV..." | null, "zcashTx": "fa8d..." | null,
  "landed": { "txid": "fa8d...", "height": 3512504 } | null,
  "quotedOut": "108000", "refundReason": null }
```
`landed` is set once the Zcash txid is mined (height read from lightwalletd). Poll every 5 s.

## GET /api/readback/:id

View-key read-back of the landed note (demo wallet only; scan with the UFVK, never a spending key).
```json
{ "id": "...", "state": "LANDED" | "WAITING" | "UNAVAILABLE",
  "pool": "ironwood", "valueZat": 115000, "height": 3512504, "txid": "fa8d...",
  "quotedOutZat": 108000, "deltaZat": 7000, "scannedTo": 3512512 }
```
`valueZat` is the decrypted note value, not the quote. `UNAVAILABLE` for user-supplied addresses.

## GET /api/exposure/:id

What is still public about this shield. Each row: `{ "what", "value", "whoCanSee", "source", "exposed": bool }`.
```json
{ "id": "...", "rows": [
  { "what": "Solana sender", "value": "ia3M...", "whoCanSee": "anyone (Solana explorer)", "source": "solana tx", "exposed": true },
  { "what": "Deposit address", ... },
  { "what": "Amount in and timing", ... },
  { "what": "Recipient u1 via 1Click status", "value": "u1...", "whoCanSee": "anyone holding the deposit address (unauthenticated /v0/status)", "exposed": true },
  { "what": "Amount entering the shielded pool", "value": "115000", "whoCanSee": "anyone (delivery tx has transparent inputs)", "exposed": true },
  { "what": "Note contents to the payer", "whoCanSee": "1Click/NEAR (sender holds the outgoing view key)", "exposed": true },
  { "what": "Later spends of the note", "whoCanSee": "nobody", "exposed": false } ],
  "publicOnStatus": [ { "field": "recipient", "value": "u1..." } ],
  "addressReused": false }
```

## Other
`GET /api/health` -> `{ "ok": true }`. Existing AIRLOCK routes (`/api/tokens`, `/api/inspect/:addr`, ...) are unchanged.
