# 0266 — Burrowed transport passes through underground structures; surfacing rules

- **Date**: 2026-11-25T14:55:00-05:00
- **Answers**: Q0071 (gaps: underground structures, surfacing among units)
- **Docs**: `factions/syndicate/units.md > Subterranean Transport > Underground movement` (new), `> Surface`

## What changed and why
User confirmed:
1. A burrowed transport passes through every underground structure (any player's expansions, a retracted pop-up turret, GDO mines). Nothing hidden can block it, so moving reveals nothing.
2. It can surface above an expansion. Only surface buildings and non-traversable ground stop it (0217).
3. The general surfacing rules apply: no overlap with above-ground units, and the spot is reserved once surfacing starts.

## Implementation impact
- Underground collision ignores structures for DrillUnits.
- The Surface check doesn't consult underground structures.

## Verification
—
