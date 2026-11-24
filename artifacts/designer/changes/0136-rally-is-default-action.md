# 0136 — A rally point always gives the unit's default right-click command

- **Date**: 2026-11-24T16:55:00-05:00
- **Answers**: Q0037 (points 9 and 10)
- **Docs**: `systems/controls/selection_and_hud.md > What a rally point makes a new unit do` (new); `factions/colonists/structures.md > Robo Portal > RallyPoint behavior`; `factions/gdo/structures.md > Barracks > RallyPoint behavior`; `factions/colonists/units.md > Drone > Right-Click Resolution`

## What changed and why
User rule: a rally always gives the spawned unit its own default right-click command against the rally target (RightClickResolution, including 0062's move/follow fallback for units that can't carry out the command). Per-building lists no longer override it.
- Q0037 pt 9: a Drone rallied on a patch **Gathers** it (the contradicting "Move to object" line removed).
- Q0037 pt 10: a Drone rallied on an enemy **unit Follows** it (right-click resolution for unarmed units per 0062), not a Move to a point. The Drone's own Right-Click Resolution line is aligned with 0062 (enemy unit → Follow; other objects → move to location).
- 0088 still applies: a rally on an enemy becomes a ground rally once the enemy leaves sight.

## Implementation impact
One rule for every producer; delete per-building rally special cases. Drone right-click on an enemy unit = Follow (was "Move to its location").

## Verification
- Robo Portal rallied on a patch: new Drones start mining it.
- Robo Portal rallied on a visible enemy unit: new Drones follow it; if it leaves sight, they go to its last-seen spot.
- Barracks rallied on an enemy: new Peacekeepers attack it.
