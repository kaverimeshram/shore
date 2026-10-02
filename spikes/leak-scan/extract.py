import json, base64, hashlib, collections, sys
B58='123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'
def b58check(ver, h):
    p = ver + h
    d = p + hashlib.sha256(hashlib.sha256(p).digest()).digest()[:4]
    n = int.from_bytes(d, 'big'); s = ''
    while n: n, r = divmod(n, 58); s = B58[r] + s
    return '1'*(len(d)-len(d.lstrip(b'\0'))) + s
txt = open('blocks.jsonl').read(); dec = json.JSONDecoder(); i = 0
addrs = collections.defaultdict(lambda: [0, 0, None])
while True:
    while i < len(txt) and txt[i].isspace(): i += 1
    if i >= len(txt): break
    b, i = dec.raw_decode(txt, i)
    for tx in b.get('vtx', []):
        for o in tx.get('vout', []):
            s = base64.b64decode(o.get('scriptPubKey', ''))
            if len(s) == 25 and s[:3] == b'\x76\xa9\x14' and s[23:] == b'\x88\xac':
                a = b58check(b'\x1c\xb8', s[3:23]); e = addrs[a]
                e[0] += 1; e[1] += int(o.get('value', 0)); e[2] = b['height']
out = [(a, *v) for a, v in addrs.items()]
json.dump(out, open('taddrs.json', 'w'))
print(len(out), 'p2pkh addresses;', sum(1 for x in out if x[1] == 1), 'with exactly one output')
