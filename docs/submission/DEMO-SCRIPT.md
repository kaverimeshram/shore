# AIRLOCK demo script (2:35 without the mainnet slot, 2:55 with it)

One browser tab on https://airlock-zec.vercel.app and one terminal in the repo. Every figure on
screen is read live from the service, lightwalletd or the dataset file the page loads. Narration is
the presenter's own voice, one breath group per line.

Setup before recording: two fresh Orchard-only unified addresses from `airlock ua` (run twice),
two Base addresses to use as destinations, a third Base address as the origin refund for the Inbound scene, and the Split spacing set to "Back to back".

## 0:00 to 0:25, a settled swap, read by a stranger

Click path: Check page, click "Try a settled swap". The receipt prints for `t1e5f3Gbrjd1nq6SLXNAmsTq5uQPsNpZ8nG`
(status SUCCESS, 0.234 ZEC out to Tron). The highlighter marks the exposed fields; the three to point at are DEPOSIT_ADDRESS, REFUND_TO and RECIPIENT.

> This swap settled days ago. I did not ask anyone for a key.
> The status endpoint hands me the shielded refund address and the recipient. Anyone can.
> The line under the form is what I measured across fifteen hundred mainnet blocks: 909 live deposit
> addresses, 558 showing a shielded refund address, 120 recipients tying 306 swaps together.

## 0:25 to 1:05, the split

Click path: Split page. Amount `0.02`, cap `0.01`, destination USDC on Base, spacing "Back to back".
Part 1: destination address one, refund address one. Part 2: destination address two, refund address
two. Press "Create swap".

> Two million zatoshi, cut into two ZIP 318 denominations of 0.01 ZEC. Each part gets its own deposit
> address, its own fresh Orchard-only refund address and its own destination.
> Part quotes go out on separate Tor circuits. For the take I pick back to back, a real run spreads
> them over hours. The browser holds the swap and the service stores nothing.

Screen: two part cards with different t-addresses, each showing what 1Click expects to deliver and
the least it will deliver. The receipt seal reads "0 shared identifiers" under "Shared across parts".

## 1:05 to 1:30, one reused address goes red, fresh ones go green

Click path: back on Split, type the same refund address into both parts and press "Create swap". The
page stops with "Two parts share one refund address, which links them." Then fresh refund addresses,
create, and the receipt seal reads "0 shared identifiers". Terminal:

```
airlock audit e89ed92b43d6   # three parts, live reads over isolated Tor circuits, shared identifiers: 0, exit 0
```

> One reused refund address links the parts, so the form refuses it.
> With fresh ones the audit recomputes the receipt from live reads and prints zero, here across three parts.

## 1:30 to 2:05, shielded ZEC reaches a transparent deposit address, on chain

Terminal: `airlock fund test <fresh testnet t-address> 40000` (about 80 s to prove and mine, cut to the
result), then `airlock tx test <the txid fund printed>`. The reference run is
`a2f1c4325d9f066f7f6464fe146c84662aa2c01e36f60bbe50d845bf40dea266` at 4,466,716.

> This is the shielded to transparent hop a deposit address needs. Built from a view-only wallet,
> approved by the policy check, signed, proved, broadcast.
> Mined on Zcash testnet after the NU7 activation at 4,465,026, read back through lightwalletd.

## MAINNET SLOT, 20 seconds, filled only from real transactions

Recorded after the capped mainnet round trip. The slot shows, in this order, each from a
command read at the time of recording:

1. `airlock note main` printing the inbound note: txid, mined height, value in zatoshi.
2. `airlock watch <swap-id>` printing both outbound parts at SUCCESS.
3. The destination balance change on Base for each part.

## 2:25 to 2:55, inbound and the close (2:05 to 2:35 when the slot is not used)

Click path: Inbound page. `0.02` ZEC to receive, USDC on Base, an Orchard-only address as recipient.
A Base deposit address and the quote appear. Cut to the watcher reading an earlier test note,
`airlock note test 200000`: it prints the txid `a37a3dbc29b5093aee3f016278321e8ab999d0f1be223d6fbf8abd511c24cd39` at height 4,435,638.

> Going the other way, the quote's recipient is a fresh Orchard-only address. No unshield step.
> Here the watcher reads a test note back with a viewing key. It never holds a spending key.
> AIRLOCK. Swap in and out of shielded ZEC, and read your own receipt first.
