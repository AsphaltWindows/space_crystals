# 0202 — Orders given while the Riot Shield toggles

- **Date**: 2026-11-25T03:10:00-05:00
- **Answers**: Q0053 point 1 (points 2 and 3: 0198)
- **Docs**: `factions/gdo/units.md > Peacekeeper > Riot Shield > During the toggle`

## What changed and why
User chose **B**: which orders a unit accepts follows its new type from the press (0198), and carrying them out waits for the toggle to finish.
- **While lowering** (already a Peacekeeper): an explicit Attack or a right-click on an enemy is accepted and held, and the attack starts once the shield is fully down. The panel shows Attack, and pressing it must not silently skip the unit.
- **While raising** (already Shielded): no Attack; a right-click follows, as with the shield up.

## Implementation impact
This changes M3-S's current behaviour, under which a lowering unit was left out of Attack and right-click made it follow. Now it is included and the attack waits for the toggle to end.

## Verification
Press R on shielded Peacekeepers, then right-click an enemy at once. They finish lowering (6 frames) and then attack, with no second click needed.
