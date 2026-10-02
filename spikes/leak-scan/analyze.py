import json, collections as C
hits = json.load(open('hits.json'))
rows = []
for addr, val, h, b in hits:
    q = b['quoteResponse']['quoteRequest']
    rows.append(dict(addr=addr, val=val, h=h, st=b['status'], origin=q['originAsset'], dest=q['destinationAsset'],
                     rec=q['recipient'], ref=q['refundTo'], upd=b['updatedAt']))
zec = [r for r in rows if r['origin'] == 'nep141:zec.omft.near']
print('1Click deposit t-addresses found:', len(rows), '| ZEC-origin (outbound):', len(zec))
print('status:', dict(C.Counter(r['st'] for r in zec)))
print('latest updatedAt:', max(r['upd'] for r in rows), ' blocks', min(r['h'] for r in rows), '-', max(r['h'] for r in rows))
def kind(a):
    return 't-addr' if a.startswith('t1') else 'unified' if a.startswith('u1') else 'sapling' if a.startswith('zs1') else 'other'
print('refundTo kind:', dict(C.Counter(kind(r['ref']) for r in zec)))
def chain(a):
    return a.split(':')[1].split('-')[0].split('.')[0] if a.startswith('nep141:') else a[:12]
print('destination chain:', dict(C.Counter(chain(r['dest']) for r in zec).most_common(8)))
rc = C.Counter(r['rec'] for r in zec); fc = C.Counter(r['ref'] for r in zec)
print('distinct recipients:', len(rc), ' recipients used by >=2 deposit addresses:', sum(1 for v in rc.values() if v > 1), ' (swaps linked by them:', sum(v for v in rc.values() if v > 1), ')')
print('distinct refundTo:', len(fc), ' refundTo used by >=2 deposit addresses:', sum(1 for v in fc.values() if v > 1), ' (swaps linked:', sum(v for v in fc.values() if v > 1), ')')
shield = [r for r in zec if kind(r['ref']) == 'unified']
print('outbound swaps whose shielded UA refund address is publicly readable from the t-address:', len(shield))
print('max swaps behind one recipient:', rc.most_common(1)[0][1], ' behind one refundTo:', fc.most_common(1)[0][1])
canon = lambda v: (lambda n: n in (1,2,5) and 1_000_000 <= v <= 10_000*10**8)(int(str(v).rstrip('0') or 0)) if v else False
print('amounts on a ZIP 318 canonical denomination:', sum(1 for r in zec if canon(r['val'])), 'of', len(zec))
inb = [r for r in rows if r['origin'] != 'nep141:zec.omft.near']
print('non-ZEC-origin hits (inbound deposit addr coincidentally on ZEC?):', len(inb))
