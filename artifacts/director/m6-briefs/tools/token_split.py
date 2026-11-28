"""Input tokens per day: the Director's session vs its subagents (summed over API calls)."""
import json, glob, collections, sys
base = "/home/iv/.claude/projects/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6"
def scan(path, acc, key):
    seen = set()
    for line in open(path):
        if '"usage"' not in line: continue
        try: e = json.loads(line)
        except Exception: continue
        if e.get("type") != "assistant": continue
        m = e.get("message") or {}; u = m.get("usage") or {}; mid = m.get("id")
        if not u or mid in seen: continue
        seen.add(mid)
        day = (e.get("timestamp") or "")[:10]
        ctx = u.get("input_tokens",0)+u.get("cache_read_input_tokens",0)+u.get("cache_creation_input_tokens",0)
        a = acc[(day, key)]; a[0] += 1; a[1] += ctx; a[2] += u.get("output_tokens",0); a[3] = max(a[3], ctx)
acc = collections.defaultdict(lambda: [0,0,0,0])
scan(base + ".jsonl", acc, "director")
for p in glob.glob(base + "/subagents/*.jsonl"): scan(p, acc, "agents")
days = sorted({d for d, _ in acc})
tot = collections.Counter()
print(f"{'day (UTC)':10s} {'director calls':>14s} {'director input':>14s} {'dir max ctx':>11s} {'agents calls':>12s} {'agents input':>12s}")
for d in days:
    a = acc[(d,"director")]; b = acc[(d,"agents")]
    tot["dc"] += a[0]; tot["di"] += a[1]; tot["ac"] += b[0]; tot["ai"] += b[1]
    print(f"{d:10s} {a[0]:14d} {a[1]/1e6:13.0f}M {a[3]/1e3:10.0f}k {b[0]:12d} {b[1]/1e6:11.0f}M")
print(f"{'total':10s} {tot['dc']:14d} {tot['di']/1e6:13.0f}M {'':>11s} {tot['ac']:12d} {tot['ai']/1e6:11.0f}M   director share {tot['di']/(tot['di']+tot['ai']):.0%}")
