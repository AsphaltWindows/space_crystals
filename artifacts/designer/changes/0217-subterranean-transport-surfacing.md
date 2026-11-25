# 0217 — Subterranean Transport: where it can surface

- **Date**: 2026-11-25T05:55:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Subterranean Transport > Surface`

## What changed and why
User: it can't surface on top of a building or on ground that tracked vehicles can't traverse. The test is the tile's own passability for tracked vehicles plus no building, **not** ground reachability. So it can surface on a plateau, or behind cliffs, that ground vehicles can't reach. This corrects my earlier "reach" wording in 0215.

## Implementation impact
Concept only. The surface check is tracked-passable terrain under the footprint, with no structure.

## Verification
—
