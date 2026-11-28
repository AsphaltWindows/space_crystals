"""Rewrap the comment paragraphs of lines changed in the working tree (git diff) that exceed WIDTH. Usage: rewrap.py"""
import re, subprocess
WIDTH = 120
diff = subprocess.run(['git', 'diff', '-U0'], capture_output=True, text=True).stdout
changed = {}
cur = None
for l in diff.splitlines():
    if l.startswith('+++ b/'): cur = l[6:]
    elif l.startswith('+') and not l.startswith('+++') and cur and len(l) - 1 > WIDTH:
        changed.setdefault(cur, set()).add(l[1:])
PRE = re.compile(r'^(\s*(?://[!/]?)\s?)(.*)$')
def split_prefix(line):
    m = PRE.match(line); return (m.group(1), m.group(2)) if m and m.group(1).strip() else (None, line)
def wrap(words, first, cont):
    out, cur = [], first
    for w in words:
        if cur.strip() and len(cur) + (0 if cur.endswith(' ') else 1) + len(w) > WIDTH and cur.rstrip() not in (first.rstrip(), cont.rstrip()):
            out.append(cur.rstrip()); cur = cont + w
        else:
            cur = cur + ('' if cur.endswith(' ') or cur == '' else ' ') + w if cur.strip() else cur + w
    out.append(cur.rstrip()); return out
n = 0
for f, targets in changed.items():
    lines = open(f).read().split('\n')
    i = 0
    while i < len(lines):
        if lines[i] in targets:
            pre, text = split_prefix(lines[i])
            if pre is None: i += 1; continue
            marker = pre.rstrip()
            item = re.match(r'^(\s*)([-*] |\d+\. )', text)
            j, texts = i + 1, [text]
            while j < len(lines):
                p2, t2 = split_prefix(lines[j])
                if p2 is None or p2.rstrip() != marker or not t2.strip() or re.match(r'^\s*([-*] |\d+\. |#|```)', t2): break
                if not t2.startswith(' ') and item: break
                texts.append(t2.strip()); j += 1
            lead = re.match(r'^\s*', text).group(0)
            cont = pre + lead + (' ' * len(item.group(2)) if item else '')
            new = wrap(' '.join(t.strip() for t in texts).split(' '), pre + lead, cont)
            lines[i:j] = new; i += len(new); n += 1
        else: i += 1
    open(f, 'w').write('\n'.join(lines))
print(f"rewrapped {n} paragraphs in {len(changed)} files")
