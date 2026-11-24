# 0074 — Shielded Peacekeepers follow on right-click

- **Date**: 2026-11-24T00:55:00-05:00
- **Answers**: Q0020 (point 1; points 2–5 still open)
- **Docs**: `factions/gdo/units.md > Peacekeeper > Riot Shield` (right-click line rewritten)

## What changed and why
The user picked B. The general rule from 0062 wins over the older Peacekeeper line. A Peacekeeper with its shield up that is right-clicked onto an enemy **unit** follows that unit (a Move to the unit). On an enemy **structure**, it moves to the structure.

The reason: in a mixed group, the other units attack and the shield wall stays with them. A Move to the enemy's old location gave nothing that a ground click doesn't already give. For comparison, Move + click on an enemy makes every unit follow, the attackers included.

## Implementation impact
- Replaces the "Move order to that location" behaviour (the Director's assumed default under Q0020 point 1). Shielded Peacekeepers use the 0062 path: Move to the target object, which is Follow for a unit.
- The follow ends when the target dies or leaves the map (existing Follow rules). It's a Move, so the Peacekeeper still doesn't attack or auto-target while its shield is up.

## Verification
1. Select 6 Peacekeepers with shields up plus 4 Rocketmen. Right-click an enemy tank that is driving away. The Rocketmen attack it. The Peacekeepers follow the tank and stay near it.
2. Right-click an enemy structure with shielded Peacekeepers. They move to it and stop nearby without attacking.
3. Lower the shields and right-click an enemy. The Peacekeepers attack normally.
