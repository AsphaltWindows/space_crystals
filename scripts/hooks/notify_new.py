#!/usr/bin/env python3
"""Notification hook shared by the Designer and Director sessions.

Usage: notify_new.py <watch_dir> <prefix> <seen_file> <label>

Watches <watch_dir> for files named <prefix>NNNN-slug.md. <seen_file> holds the
highest number the agent has processed (written by the agent itself). The hook
injects a short notice into the main session's context:
  - SessionStart: every entry newer than <seen_file> (the backlog).
  - Other events: only entries not yet announced (tracked in a sibling
    .notified file so each entry is announced once).
Hooks firing inside sub-agents are ignored so the notice reaches the main agent.
"""
import json, os, re, sys

watch_dir, prefix, seen_file, label = sys.argv[1:5]
root = os.environ.get("CLAUDE_PROJECT_DIR", os.getcwd())
watch_dir = os.path.join(root, watch_dir)
seen_file = os.path.join(root, seen_file)
notified_file = os.path.join(os.path.dirname(seen_file), ".notified_" + os.path.basename(seen_file))

try:
    data = json.load(sys.stdin)
except Exception:
    data = {}
if data.get("agent_id"):  # inside a sub-agent
    sys.exit(0)
event = data.get("hook_event_name", "")

def read_num(path):
    try:
        return int(open(path).read().strip() or 0)
    except Exception:
        return 0

pat = re.compile(r"^" + re.escape(prefix) + r"(\d{4,})-.*\.md$")
entries = []
if os.path.isdir(watch_dir):
    for name in os.listdir(watch_dir):
        m = pat.match(name)
        if m:
            entries.append((int(m.group(1)), name))
entries.sort()

seen = read_num(seen_file)
floor = seen if event == "SessionStart" else max(seen, read_num(notified_file))
new = [name for num, name in entries if num > floor]
if not new:
    sys.exit(0)

os.makedirs(os.path.dirname(notified_file), exist_ok=True)
with open(notified_file, "w") as f:
    f.write(str(entries[-1][0]))

rel = os.path.relpath(watch_dir, root)
msg = (f"[{label}] {len(new)} new: " + ", ".join(new) + f" (in {rel}/; last processed: {seen}). "
       "Handle per your prompt.")
print(json.dumps({"hookSpecificOutput": {"hookEventName": event, "additionalContext": msg}}))
