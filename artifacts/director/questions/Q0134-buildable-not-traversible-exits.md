# Q0134 — Buildings on Buildable ground that units can't walk on: how do their units come out?

- **Date**: 2026-11-26T02:14:15-05:00
- **Blocking**: no — proceeding on (a): a unit is never released onto ground it can't stand on, so such a doorway holds its finished units.
- **Docs**: `core/map.md > Tile`; `core/entities.md > Structure Type > Exit Doorway`

## Question
A custom tile preset may be Buildable but not Traversible ("Any combination of properties is valid"). When a production building stands on such ground, and its exit side lies over it, how do its units come out?

## Context
- **Units leave through the building's own tiles:** a released unit walks out of its building's footprint through the Exit Doorway; only the releasing building lets it overlap its footprint (`> Exit Doorway`, point 1).
- **With the default presets this never arises,** since all Buildable ground (Plane) is Traversible. It can arise on a map with a custom preset, and today also through debug spawns.
- **What happened before the fix:** robustness fuzz seed 155 had a Recruitment Hall debug-spawned over Cliffs. Its Recruits came out over the Cliff under side B, and when the Hall was removed, a Recruit was left standing on the Cliff.
- **M5-Z1's fix:** a leaving unit may overlap the building, but never ground under it that the unit couldn't stand on. So a Hall whose side B lies over such ground holds its finished Recruit for good, like a doorway that stays occupied (point 5).
- **The same holds for every faction's production buildings** with exit doorways, and for Eject All.
- **The other reading:** let the unit through, as before the fix. A unit could then be left standing where nothing can walk once the building is destroyed.

## Options
- **(a) As built:** the unit isn't released while its way out crosses such ground, so the building holds it indefinitely, and the player can't tell why.
- **(b) Placement refuses it:** a building whose exit side (or entrance side) would lie over ground its units can't walk on can't be placed there. This is a new placement rule, with a reason shown on the ghost.
- **(c) Such presets imply Traversible:** Buildable ground is always Traversible for ground units, and the "any combination" rule gets that exception.
- **(d) Something else.**
