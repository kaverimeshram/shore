# AIRLOCK presentation (target 2:30)

Spoken, one idea per breath. Slides are the product itself: three screens and one terminal.

**Hook (0:00).** "A ZEC holder wants out. One quote, one deposit address, and anyone on the internet
can read where the money is going and where it comes back to. No key. I checked."

**What we measured (0:20).** Over 1,501 mainnet blocks we found 909 live 1Click deposit
addresses, all ZEC-origin, 896 settled. 558 expose a shielded refund address on the unauthenticated
status endpoint. 120 recipients link 306 swaps. Source: `spikes/leak-scan/FINDINGS.md`, read-only,
no funds touched.

**The product (0:55).** AIRLOCK swaps into and out of shielded ZEC through NEAR Intents and prints
a receipt of what a chain observer, the status endpoint and the solver can read. Outbound is cut
into ZIP 318 denominations. Each part has its own deposit address, fresh Orchard-only refund address
and destination, and leaves on its own Tor circuit, minutes apart. Inbound lands in a fresh
Orchard-only address with no unshield step.

**The proof (1:30).** The audit counts identifiers two parts share. A live three-part swap prints
0. Reuse one refund address and it prints `SHARED refund_to` and exits 2. The shielded to
transparent hop is built from a view-only wallet, approved by COUNTERSIGN's policy check, and mined
on Zcash testnet after the NU7 activation at height 4,466,716.

**Who uses it Monday (1:55).** ZEC holders exiting to stables: the forum case with 1,120 ZEC frozen
for 50 days, operators moving many wallets through the same rail. Wallets that route ZEC through
NEAR Intents can link their exit flow to the receipt.

**Close (2:20).** "Swap in and out of shielded ZEC, and read your own receipt first."
