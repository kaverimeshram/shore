# AIRLOCK funding ask (testnet only)

Purpose: prove the inbound read-back half on chain. A note sent to this Orchard-only UA must
be found by `airlock note test <min-zat>` (view-only scan through testnet.zec.rocks) with its
txid and mined height. This is the same code path B3 uses for the mainnet inbound swap.

Amount: 0.002 TAZ (200,000 zat), one transaction, from the parked handoff testnet wallets.

Receive address (Orchard-only, index 0 of AIRLOCK's testnet account):
- Revision 2: zutest1md9q8fvd556479xmlpmjunaw3x3thkjkc68kxfdsry0lpfhr3day43fdp8wuze8cnwhu7uymd9kyr2nl0ga0yp5wt3recdryx5xazm5r
- Revision 1 (for a sender on the older pins): utest19f9uqz427fjj9qg99lrzs5s3g4nlppcsnaatsck2nxgyt6rrvmyn98afe37vqfnst30m70skqhy99akq7ptfqpqy6s08srr6eguqnjur

The watcher db is created and its birthday is 2026-10-03 (testnet height about 4,435,630), so
any transaction after that height is seen. Run `airlock note test 200000 1200` before or after
the send.

No mainnet funding is requested.
