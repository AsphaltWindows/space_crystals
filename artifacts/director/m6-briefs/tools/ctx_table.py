"""One line per subagent transcript: brief, API calls, total input tokens, mean/max context, code-reading share."""
import json, sys, re, os, glob
rows = []
for path in sys.argv[1:]:
    brief = None; calls = 0; tot = 0; mx = 0; seen = set(); readc = 0; allc = 0; pending = {}
    for line in open(path):
        try: e = json.loads(line)
        except Exception: continue
        m = e.get("message") or {}
        if brief is None and e.get("type") == "user":
            c = m.get("content"); t = c if isinstance(c, str) else json.dumps(c)[:5000]
            mm = re.search(r"briefs/(m[0-9][a-z0-9-]+)\.md", t); brief = mm.group(1) if mm else "?"
        if e.get("type") == "assistant":
            u = m.get("usage") or {}; mid = m.get("id")
            if mid and mid not in seen and u:
                seen.add(mid); ctx = u.get("input_tokens",0)+u.get("cache_read_input_tokens",0)+u.get("cache_creation_input_tokens",0)
                calls += 1; tot += ctx; mx = max(mx, ctx)
            for b in m.get("content") or []:
                if isinstance(b, dict) and b.get("type") == "tool_use":
                    cmd = (b.get("input") or {}).get("command", "") if b.get("name") == "Bash" else ""
                    pending[b.get("id")] = b.get("name") == "Read" or bool(re.match(r"^\s*(sed|head|tail|cat|grep|rg|awk)\b", cmd))
        elif e.get("type") == "user" and isinstance(m.get("content"), list):
            for b in m["content"]:
                if isinstance(b, dict) and b.get("type") == "tool_result":
                    c = b.get("content"); n = len(c) if isinstance(c, str) else sum(len(x.get("text","")) for x in c if isinstance(x, dict)) if isinstance(c, list) else 0
                    allc += n
                    if pending.get(b.get("tool_use_id")): readc += n
    if calls >= 150 and brief and brief.startswith("m6"):
        rows.append((brief, calls, tot, tot/calls, mx, readc/max(allc,1)))
print(f"{'brief':28s} {'calls':>5s} {'input':>7s} {'mean':>6s} {'max':>6s} {'reading':>7s}")
for r in sorted(rows, key=lambda r: r[0]):
    print(f"{r[0][:28]:28s} {r[1]:5d} {r[2]/1e6:6.0f}M {r[3]/1e3:5.0f}k {r[4]/1e3:5.0f}k {r[5]:7.0%}")
