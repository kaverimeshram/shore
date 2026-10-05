# AIRLOCK demo script (target 2:50)

One browser tab on https://airlock-zec.vercel.app and one terminal in the repo. Every figure on
screen is read live from the service, lightwalletd or the dataset file the page loads. Narration is
the presenter's own voice, one breath group per line.

Setup before recording: two fresh Orchard-only unified addresses from `airlock ua` (run twice),
two Base addresses to use as destinations, and the Split spacing set to "Back to back".

## 0:00 to 0:30, a settled swap, read by a stranger

Click path: Check page, click "Try a settled swap". The receipt prints for `t1e5f3Gbrjd1nq6SLXNAmsTq5uQPsNpZ8nG`
(status SUCCESS, 0.234 ZEC out to Tron). The highlighter marks DEPOSIT_ADDRESS, REFUND_TO and RECIPIENT.

> This swap finished two days ago. I did not ask anyone for a key.
> The status endpoint hands me the shielded refund address and the recipient. Anyone can.
> The line under the form is what I measured across fifteen hundred mainnet blocks: 909 live deposit
> addresses, 558 showing a shielded refund address, 120 recipients tying 306 swaps together.

## 0:30 to 1:15, the split

Click path: Split page. Amount `0.02`, cap `0.01`, destination USDC on Base, spacing "Back to back".
Part 1: destination address one, refund address one. Part 2: destination address two, refund address
two. Press "Create swap".

> Two million zatoshi, cut into two ZIP 318 denominations of 0.01 ZEC. Each part gets its own deposit
> address, its own fresh Orchard-only refund address and its own destination.
> Part quotes go out on separate Tor circuits. For the take I pick back to back, a real run spreads
> them over hours. The browser holds the swap and the service stores nothing.

Screen: two part cards with different t-addresses, each showing what 1Click expects to deliver and
the least it will deliver. Receipt row "Shared identifiers across parts: 0".

## 1:15 to 1:55, the audit goes red, then green

Terminal:

```
airlock audit <swap-id-with-a-reused-refund>   # prints SHARED refund_to, exit 2
airlock audit <swap-id>                        # prints shared identifiers: 0, exit 0
```

> The audit recomputes the receipt from live reads and counts what two parts share.
> Reuse one refund address and it fails. Use fresh ones and it prints zero.

## 1:55 to 2:25, shielded ZEC reaches a transparent deposit address, on chain

Terminal: `airlock fund test tmN6ESmKKcDkzm7bwn4wWc1rJ2vRvPhHfRU 40000`, then
`airlock tx test a2f1c4325d9f066f7f6464fe146c84662aa2c01e36f60bbe50d845bf40dea266`.

> This is the shielded to transparent hop a deposit address needs. Built from a view-only wallet,
> approved by the policy check, signed, proved, broadcast.
> Mined on Zcash testnet after the NU7 activation at height 4,466,716, read back through lightwalletd.

## 2:25 to 2:50, inbound and the close

Click path: Inbound page. `0.02` ZEC to receive, USDC on Base, an Orchard-only address as recipient.
A Base deposit address and the quote appear. Cut to `airlock note test 200000`: the watcher prints
the txid `a37a3dbc29b5093aee3f016278321e8ab999d0f1be223d6fbf8abd511c24cd39` at height 4,435,638.

> Going the other way, the quote's recipient is a fresh Orchard-only address. No unshield step.
> The watcher finds a note with a viewing key and never holds a spending key.
> AIRLOCK. Swap in and out of shielded ZEC, and read your own receipt first.
