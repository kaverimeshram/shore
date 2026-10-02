"""Aggregate the quote profiles of the scanned swaps into data/quote-profiles.json (no addresses)."""
import json, collections as C, datetime as D
p = lambda s: D.datetime.fromisoformat(s.replace('Z', '+00:00'))
hits = json.load(open('spikes/leak-scan/hits.json'))
c = C.Counter()
for a, v, h, b in hits:
    q = b['quoteResponse']['quoteRequest']; ts = b['quoteResponse']['timestamp']
    fees = ','.join(f"{f['recipient'][:6]}:{f['fee']}" for f in q.get('appFees') or [])
    r = q['refundTo']
    kind = 'u%d' % len(r) if r.startswith('u1') else 't' if r.startswith('t1') else 'other'
    c[(q.get('referral') or '', fees, q['slippageTolerance'], round((p(q['deadline']) - p(ts)).total_seconds() / 60), kind)] += 1
out = {"sample": {"swaps": sum(c.values()), "window": "mainnet blocks 3502581 to 3504081, read 2026-10-03", "origin": "ZEC"},
       "profiles": [{"referral": k[0], "fees": k[1], "slippageBps": k[2], "deadlineMin": k[3], "refund": k[4], "count": n} for k, n in c.most_common()]}
json.dump(out, open('data/quote-profiles.json', 'w'), indent=1)
print(out['sample'], len(out['profiles']), 'profiles')
