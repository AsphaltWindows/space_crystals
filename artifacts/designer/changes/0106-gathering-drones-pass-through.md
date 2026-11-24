# 0106 — Gathering Drones pass through each other

- **Date**: 2026-11-24T12:00:00-05:00
- **Answers**: Q0038 (point 1 only; points 2 and 3 still open)
- **Docs**: `factions/colonists/units.md > Gathering Drones pass through each other` (new); `systems/units.md > UnitCollision` (new bullets: gathering Drones exception, Units that already overlap)

## What changed and why
Option B, extended. A Drone on a Gather job does not collide with any other Drone on a Gather job, **of any player** (enemy Drones included, consistent with shared patch occupancy). "On a Gather job" is every step of every job in the Drone Gathering table: travelling to the pick-up point, mining/picking up, waiting at an occupied patch, carrying and delivering. Reason: hard collision would jam the patch row and the Egress Node at saturation, while the income targets assume occupancy is the only limit.

**Drones only**: Cults Recruits and other workers keep normal collision while mining.

New general rule: two ground units that overlap without being allowed to (e.g. a Drone that leaves its Gather job while inside another Drone) ignore each other until they have separated; then normal collision resumes. No push-apart.

## Implementation impact
- Gathering Drones still collide with: non-gathering Drones (idle, moving, deploying), all other units, structures, patches, terrain.
- Patch occupancy (one mining Drone per patch) is unchanged.
- Yielding rules unchanged (gathering Drones still never yield to others; it simply no longer matters between gathering Drones).
- The overlap rule is general, not Drone-specific; it applies to any pair of ground units found overlapping.

## Verification
- Saturate a patch row with ~24 Drones: they cross each other freely between the row and the Egress Node, no jams; income approaches the Economy Targets table.
- Two players' Drones mining the same row pass through each other.
- An idle Drone parked in the strip: gathering Drones path around it / bump into it.
- Select a Drone overlapping another mining Drone and order Move: it drives out without being blocked or shoving the other; once apart, they collide normally.
- Recruits mining a patch still bump into each other.
