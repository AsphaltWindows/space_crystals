# 0345 — Which Storage a Recruit uses: nearest to its patch, chosen at every return

- **Date**: 2026-11-26T22:00:00-05:00
- **Answers**: Q0102 (including the addendum)
- **Docs**: `factions/cults/units/recruit.md > Gathering` (Which Storage)

## What changed and why
User chose **(a)**:
- At each return, the Recruit uses the nearest own completed Storage, edge to edge from its patch.
- With none, it idles holding its load.
- The addendum is kept: a Storage lost mid-return is replaced by the one nearest the Recruit (0170), and the next return chooses from the patch again.

## Implementation impact
As built.

## Verification
- Complete a new Storage beside a busy patch: the next returns go to it.
