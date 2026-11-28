"""Stale design citations: from the ignored design_citations report, the cited sections whose name appears nowhere
in the cited design file (not as a heading, nor in bold, nor in text). Usage: stale_cites.py <report> [--sites]"""
import re, sys, collections, os
D = '/home/iv/dev/space_crystals/artifacts/designer/design/'
rep = open(sys.argv[1]).read().splitlines()
stale = collections.defaultdict(list)
cache = {}
for line in rep:
    m = re.match(r'^(\S+?):(\d+): (\S+\.md) > (.+?)\s*$', line)
    if not m or m.group(3).startswith('docs/'):
        continue
    site, path, sec = f"{m.group(1)}:{m.group(2)}", m.group(3), m.group(4)
    if path not in cache:
        cache[path] = open(D + path).read().lower() if os.path.exists(D + path) else None
    text = cache[path]
    first = sec.split(' > ')[-1].strip().strip('*`').lower()
    if text is None or first not in text:
        stale[f"{path} > {sec}"].append(site)
total = sum(len(v) for v in stale.values())
print(f"stale citations: {total} at {len(stale)} distinct sections")
for k, v in sorted(stale.items(), key=lambda kv: -len(kv[1])):
    files = sorted({s.split(':')[0] for s in v})
    print(f"{len(v):3} {k}  [{len(files)} files]")
    if '--sites' in sys.argv:
        for s in v: print("      ", s)
