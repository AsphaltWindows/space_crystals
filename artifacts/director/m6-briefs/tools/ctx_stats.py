"""Summarize a subagent transcript: API calls, context growth, total input tokens, and what fills the context.
Prints aggregates only."""
import json, sys, collections, re
path = sys.argv[1]
calls = []          # (input+cache_read+cache_creation, output) per API response, deduped by message id
seen = set()
res_by_tool = collections.Counter(); n_by_tool = collections.Counter()
read_files = collections.Counter(); bash_kinds = collections.Counter(); bash_n = collections.Counter()
pending = {}        # tool_use_id -> (name, key)
first_user = 0
compactions = 0
def size(c):
    if isinstance(c, str): return len(c)
    if isinstance(c, list): return sum(size(x.get("text", "")) if isinstance(x, dict) else len(str(x)) for x in c)
    return len(str(c))
for line in open(path):
    try: e = json.loads(line)
    except Exception: continue
    if e.get("type") == "system" and "compact" in json.dumps(e)[:400].lower(): compactions += 1
    m = e.get("message") or {}
    if e.get("type") == "assistant":
        mid = m.get("id")
        u = m.get("usage") or {}
        if mid and mid not in seen and u:
            seen.add(mid)
            ctx = u.get("input_tokens", 0) + u.get("cache_read_input_tokens", 0) + u.get("cache_creation_input_tokens", 0)
            calls.append((ctx, u.get("output_tokens", 0)))
        for b in m.get("content") or []:
            if isinstance(b, dict) and b.get("type") == "tool_use":
                name = b.get("name"); inp = b.get("input") or {}
                key = ""
                if name == "Read": key = inp.get("file_path", "")
                elif name == "Bash":
                    cmd = inp.get("command", "").strip()
                    if "cargo test" in cmd: key = "cargo test"
                    elif "cargo clippy" in cmd: key = "cargo clippy"
                    elif "cargo build" in cmd or "cargo check" in cmd: key = "cargo build/check"
                    elif "cargo run" in cmd: key = "cargo run"
                    elif re.match(r"^(grep|rg)\b", cmd): key = "grep"
                    elif re.match(r"^(sed|head|tail|cat|awk)\b", cmd): key = "sed/head/cat"
                    elif cmd.startswith("git"): key = "git"
                    else: key = "other: " + cmd.split()[0][:20] if cmd else "other"
                pending[b.get("id")] = (name, key)
    elif e.get("type") == "user":
        c = m.get("content")
        if isinstance(c, str) and not first_user: first_user = len(c)
        if isinstance(c, list):
            for b in c:
                if isinstance(b, dict) and b.get("type") == "tool_result":
                    name, key = pending.get(b.get("tool_use_id"), ("?", ""))
                    s = size(b.get("content"))
                    res_by_tool[name] += s; n_by_tool[name] += 1
                    if name == "Read": read_files[key] += s
                    if name == "Bash": bash_kinds[key] += s; bash_n[key] += 1
tot_in = sum(c for c, _ in calls); tot_out = sum(o for _, o in calls)
print(f"API calls {len(calls)}; total input tokens {tot_in/1e6:.1f}M (sum of context over calls); output {tot_out/1e3:.0f}k; compaction-ish events {compactions}")
if calls:
    ctxs = [c for c, _ in calls]
    q = lambda p: sorted(ctxs)[int(p*(len(ctxs)-1))]
    print(f"context: first {ctxs[0]/1e3:.0f}k, median {q(0.5)/1e3:.0f}k, p90 {q(0.9)/1e3:.0f}k, max {max(ctxs)/1e3:.0f}k, mean {tot_in/len(calls)/1e3:.0f}k")
    for frac in (0.1, 0.25, 0.5, 0.75, 1.0):
        i = max(0, int(frac*len(ctxs))-1); print(f"  after {frac:.0%} of calls: {ctxs[i]/1e3:.0f}k")
tot_res = sum(res_by_tool.values())
print(f"tool results: {tot_res/1e6:.2f}M chars (~{tot_res/3.6/1e3:.0f}k tokens) over {sum(n_by_tool.values())} results")
for k, v in res_by_tool.most_common(8): print(f"  {k:10s} {v/1e3:7.0f}k chars  n={n_by_tool[k]}")
print("Bash by kind:")
for k, v in bash_kinds.most_common(9): print(f"  {k[:24]:24s} {v/1e3:7.0f}k chars  n={bash_n[k]}")
print("Read, top files:")
for k, v in read_files.most_common(12): print(f"  {v/1e3:6.0f}k  {k.replace('/home/iv/dev/space_crystals/','')[-90:]}")
