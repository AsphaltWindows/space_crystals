"""Re-point citations of design sections that moved from one doc to another, including citations split across comment
lines. Usage: move_cites.py <old doc> <new doc> <section>... (a section 'A>B' maps nested 'A > B'). Prints counts."""
import re, sys, subprocess
old, new, secs = sys.argv[1], sys.argv[2], sys.argv[3:]
SEP = r'(?:[ \t]|\n[ \t]*//[!/]?)+'
def pat(text): return SEP.join(re.escape(w) for w in text.split())
files = subprocess.run(['git', 'ls-files', 'crates', 'docs', 'ARCHITECTURE.md'], capture_output=True, text=True).stdout.split()
total = 0
for f in files:
    if not f.endswith(('.rs', '.md')): continue
    s = open(f).read(); n0 = 0
    for sec in sorted(secs, key=len, reverse=True):
        parts = [p.strip() for p in sec.split('>')]
        rx = pat(old) + SEP + '>' + SEP + (SEP + '>' + SEP).join(pat(p) for p in parts)
        s, n = re.subn(rx, new + ' > ' + ' > '.join(parts), s); n0 += n
    if n0:
        open(f, 'w').write(s); total += n0; print(f"{n0:3} {f}")
print("total", total)
