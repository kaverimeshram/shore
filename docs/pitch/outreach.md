# SHORE outreach drafts

**DRAFTS ONLY, FOR KAMAL APPROVAL. Nothing here has been sent or posted.** Every fact is from README.md.

## 1. Reply or DM to @zksnarks_ (for kamal approval)

Context: their "zero to first shielded transaction" bounty, https://x.com/zksnarks_/status/2104718302606205130

> Built for exactly this bounty: SHORE takes ZEC sitting in Phantom on Solana into a shielded Zcash balance with one
> approval. Our mainnet run landed in 84s for about 39 cents, and SHORE reads the note back with a viewing key so you
> see what actually arrived. No custody. You can hand people an invite link with a suggested amount and your note:
> https://shore-zec.pages.dev/shore/?invite=zksnarks&note=Your%20first%20shielded%20ZEC
> 60s replay, nothing sent: https://shore-zec.pages.dev/shore/?replay=1&tour=1&speed=3
> Happy to run a batch of your bounty participants through it.

## 2. Zcash forum reply, Raydium thread t/57788 (for kamal approval)

Thread: https://forum.zcashcommunity.com/t/38-000-new-zec-holders-on-solana-in-six-weeks-now-what/57788

> Thanks for the numbers. 38,000 first-time ZEC holders on Solana, about half the ZEC still in those wallets, and
> almost no one bridged. We built SHORE for that gap.
>
> SHORE moves the SPL ZEC in a Phantom wallet into a shielded Zcash balance with one Phantom approval, through NEAR
> Intents 1Click. Today's path in the ZecHub guide is 2 sites, 2+ approvals and about 8 minutes. Our mainnet run on
> 2026-10-10 sent 0.0014 ZEC and landed 115,000 zat in the shielded pool 84 seconds later, for about 39 cents, most
> of it 1Click's fixed withdraw fee. After landing, SHORE decrypts the note with a viewing key (never a spending key)
> and shows what arrived.
>
> What stays public, plainly: the hop in is on Solana, and 1Click's status links your Phantom address to the one u1
> it paid, which is why SHORE uses a fresh address for every shield. While funds are in flight, the solver network
> can delay, refund or freeze them; SHORE never holds funds but cannot overrule 1Click. You also need a Zcash wallet
> (Zashi/Zodl, Ywallet) to receive.
>
> Try the replay (nothing sent): https://shore-zec.pages.dev/shore/?replay=1&tour=1&speed=3
> Live, capped at 0.02 ZEC per shield: https://shore-zec.pages.dev/shore/?live=1
> Code and proof txids: README in the repo. Feedback from reward holders very welcome.

## 3. Short message for Solana ZEC holder communities (for kamal approval)

> Got ZEC in Phantom from rewards? You can shield it in one approval. SHORE moves it into a shielded Zcash balance
> (you need a Zcash wallet like Zashi or Ywallet to receive), then shows you what landed. A mainnet run took 84
> seconds for about 39 cents. No custody, capped at 0.02 ZEC per shield while it is new.
> Watch a 60s replay first: https://shore-zec.pages.dev/shore/?replay=1&tour=1&speed=3
