# AIRLOCK demo script (target 2:50)

Recorded in one browser tab on https://airlock-zec.vercel.app and one terminal. Every number on
screen is read live. Narration is in the presenter's own voice, one breath group per line.

## 0:00 to 0:25, the problem on a real swap

Screen: Check page. Paste the deposit address `t1UH6mFsvCCzTYvKgTgNCbNBW9t8Vd1NCoq`, press
"Print the receipt".

> This is a swap anyone made through NEAR Intents. I did not ask for a key.
> The status endpoint hands me the refund address and the recipient. Three fields, for a stranger.
> Every ZEC exit crosses one transparent address like this. AIRLOCK makes the crossing small and
> shows you exactly what it leaves behind.

Highlighter marks DEPOSIT_ADDRESS, REFUND_TO and RECIPIENT as the receipt prints. Footer line
"Quote profile: shared by 0 of 909 sampled swaps" stays on screen for two seconds.

## 0:25 to 1:05, the split

Screen: Split page. Amount 0.02, cap 0.01, destination USDC on Base.

> Twenty million zatoshi, cut into two ZIP 318 denominations. Each part gets its own deposit
> address, its own fresh Orchard-only refund address and its own destination.
> Part quotes leave on separate Tor circuits, minutes apart. The browser holds the swap. The
> service stores nothing.

Screen: the two part cards appear with different deposit addresses. Receipt table shows
"Shared identifiers across parts: 0".

## 1:05 to 1:50, the audit goes red, then green

Screen: terminal.

```
airlock audit <swap-id>
SHARED refund_to = u1czvd...     exit 2     (a swap built with one refund address reused)
airlock audit <swap-id>
shared identifiers: 0            exit 0     (the same flow with fresh refund addresses)
```

> The audit recomputes the receipt from live reads and counts what two parts share.
> Reuse one refund address and it fails. Use fresh ones and it prints zero.

## 1:50 to 2:25, the money moves on chain

Screen: terminal, `airlock fund test tmN6ESmKKcDkzm7bwn4wWc1rJ2vRvPhHfRU 40000`, then
`handoff-tx tx a2f1c432...ea266`.

> This is the shielded to transparent hop a deposit address needs. Built from a view-only wallet,
> approved and signed by COUNTERSIGN's policy check, proved, broadcast.
> Mined on Zcash testnet after the NU7 activation, height 4,466,716, read back through lightwalletd.

## 2:25 to 2:50, inbound and the close

Screen: Inbound page, 0.02 ZEC to receive, USDC on Base. A real Base deposit address and a quote
appear. Cut to `airlock note test 200000`: the view-only watcher prints the txid
`a37a3dbc...cd39` and mined height 4,435,638.

> Going the other way, the quote's recipient is a fresh Orchard-only address. No unshield step.
> The watcher finds the note with a viewing key, never a spending key.
> AIRLOCK. Swap in and out of shielded ZEC, and read your own receipt first.
