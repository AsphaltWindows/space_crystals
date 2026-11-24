# Q0044: Gathering details found while building it: fog, plates, empty Drop Off, blocked Drones, redirect timing, the Egress Node's sight

- **Date**: 2026-11-24T10:02:43-05:00
- **Blocking**: no. M3-H is built with the current behaviour listed under each point.
- **Docs**: `factions/colonists/units.md > Drone > Gathering`, `> Patch Occupancy (one Drone per patch)`, `> Assigned Drop-off`, `> Unit Commands`; `factions/colonists/economy.md > Egress Node`; Q0017 point 2 (targeting remembered objects).

## Question
1. **Gather on a patch remembered in the fog.**
   - **Current behaviour:** refused as "not visible", following Q0017's default for remembered targets. A player can't send Drones to a patch field they scouted earlier until something sees it again.
2. **A GDO plate placed on a patch a Colonists Drone is mining.**
   - **Current behaviour:** the plate is allowed, since the Drone stands beside the patch, not on it. The Drone treats the patch as gone and redirects.
3. **A Drop Off order with nothing carried.**
   - **Current behaviour:** the Drone goes straight back to its patch.
4. **A Drone that can't reach a free patch** because a non-gathering unit or a structure blocks the way.
   - **Current behaviour:** it retries forever, with no rule to give up.
5. **A patch removed (depleted) while Drones are heading to it.**
   - **Current behaviour:** they redirect at once, not on arrival.
6. **The Egress Node's sight (3 grid units) doesn't reach a patch row 2 grid units from its edge,** the reference layout of 0107. When no Drone is near, those patches are remembered rather than visible. With point 1's current behaviour, a right-click Gather on them is refused.

## Context
All of these came up while building Drone gathering (M3-H). The gathering docs, 0106–0108 and Q0037 don't cover them.

## Options
- **Point 1:**
  - A, as now: refused.
  - B, allowed: the Drone walks to the remembered patch and gathers if it's still there on arrival, otherwise it redirects.
- **Point 2:**
  - A, as now: the plate wins and the Drone redirects.
  - B, a patch being mined counts as occupied, so the plate can't be placed there.
- **Point 3:**
  - A, as now: back to the patch.
  - B, the Drone goes to the node and idles.
  - C, refused.
- **Point 4:**
  - A, as now: retry forever.
  - B, give up after a number of attempts and idle.
  - C, try the next free patch.
- **Point 5:**
  - A, as now: at once.
  - B, on arrival.
- **Point 6:**
  - A, as now: Drones' sight covers it in practice.
  - B, raise the Egress Node's sight.
  - C, settled by point 1 B.
