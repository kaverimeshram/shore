import json, time, urllib.request, urllib.error, random, sys, collections
random.seed(1)
a = [x for x in json.load(open('taddrs.json')) if x[1] == 1 and x[2] >= 1_000_000]
sample = random.sample(a, min(int(sys.argv[1]), len(a)))
codes = collections.Counter(); hits = []
for addr, n, val, h in sample:
    try:
        r = urllib.request.urlopen(urllib.request.Request(f'https://1click.chaindefuser.com/v0/status?depositAddress={addr}', headers={'User-Agent': 'airlock-research/0.1'}), timeout=20)
        code, body = r.status, json.load(r)
    except urllib.error.HTTPError as e:
        code, body = e.code, None
    codes[code] += 1
    if code == 200: hits.append((addr, val, h, body))
    time.sleep(0.25)
print(len(a), 'candidates;', len(sample), 'probed;', dict(codes))
json.dump(hits, open('hits.json', 'w'))
for addr, val, h, b in hits[:5]:
    q = b['quoteResponse']['quoteRequest']
    print(addr, val, h, b['status'], q['originAsset'], '->', q['destinationAsset'][:30], 'recipient', q['recipient'][:14], 'refundTo', q['refundTo'][:14])
