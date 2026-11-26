"""Exact, asserted edits to the Director's Markdown artifacts (plan.md, insights.md).

Usage, from a throwaway script in the scratchpad:

    import sys; sys.path.insert(0, '/home/iv/dev/space_crystals/artifacts/director/tools')
    from plandoc import Doc
    P = Doc(root + '/plan.md')          # root: 'artifacts/director', or a dry-run copy
    P.rep(old, new)                     # old must occur exactly once
    P.setcell('| 0337 ', 1, 'done ...') # replace a table cell
    P.addto('| 0337 ', 1, '. More.')    # append to a table cell
    P.after_row('| Q0141 ', '| Q0142 | ... |')  # insert a row below another
    P.save()

Table cells are counted from the left, after the row's first cell (its id):
- a change row `| 0337 entry | status | — |`: the status is cell 1;
- a question row `| Q0142 | question | blocking | status |`: the status is cell 3.

Dry-run every edit on a copy first (`cp plan.md <scratch>/dry/plan.md`, run with that root, diff), then apply.
Chained `rep`s see the text the earlier ones produced: never replace the same passage twice.
"""
import re


class Doc:
    def __init__(self, path):
        self.p = path
        self.s = open(path).read()

    def rep(self, old, new):
        assert self.s.count(old) == 1, (self.p, self.s.count(old), old[:90])
        self.s = self.s.replace(old, new)

    def _row(self, prefix):
        m = list(re.finditer(r'^' + re.escape(prefix) + r'.*$', self.s, re.M))
        assert len(m) == 1, (self.p, prefix, len(m))
        return m[0]

    def setcell(self, prefix, idx, val):
        m = self._row(prefix)
        parts = m.group(0).split('|')
        parts[idx + 1] = ' ' + val + ' '
        self.s = self.s[:m.start()] + '|'.join(parts) + self.s[m.end():]

    def addto(self, prefix, idx, text):
        m = self._row(prefix)
        parts = m.group(0).split('|')
        parts[idx + 1] = parts[idx + 1].rstrip() + text + ' '
        self.s = self.s[:m.start()] + '|'.join(parts) + self.s[m.end():]

    def after_row(self, prefix, row):
        m = self._row(prefix)
        self.s = self.s[:m.end()] + '\n' + row + self.s[m.end():]

    def save(self):
        open(self.p, 'w').write(self.s)
