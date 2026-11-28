"""Compare a checks run's computer-player rows with the base (outcomes only, no think times). Args: ai_games file [base]."""
import re, sys, difflib
T = re.compile(r"^[0-9.]+(µs|ms|s|ns)$")
def strip(line):
    toks = line.split()
    if len(toks) >= 3 and T.match(toks[-1]) and toks[-2] == "/" and T.match(toks[-3]):
        toks = toks[:-3]
    return " ".join(toks)
path = sys.argv[1]
base_path = sys.argv[2] if len(sys.argv) > 2 else "/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/dry/base_rows.txt"
rows = sorted(strip(l) for l in open(path) if l.startswith(("--map", "--scenario")))
base = sorted(" ".join(l.split()) for l in open(base_path))
print(len(rows), "rows;", len(base), "base;", "identical" if rows == base else "DIFFER")
if rows != base:
    for d in list(difflib.unified_diff(base, rows, lineterm=""))[:20]: print(d)
