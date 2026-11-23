# Q0009: How range is measured, and whether silhouettes rotate

- **Date**: 2026-11-23T15:47:00-05:00
- **Blocking**: no. Proceeding on the defaults below. The answer is needed before combat ranges are final, and before the first unit with a non-square silhouette.
- **Docs**: `systems/combat.md` (Range, MinRange), `systems/controls/behaviors.md > MovingToObject` ("within proximity"), `systems/units.md > UnitCollision` (Silhouette)

## Question
1. Where is Range measured from and to, especially against large targets such as structures and big vehicles? The same question applies to "within proximity" when moving to an object.
2. Does a unit's Silhouette rectangle rotate with its facing? Every silhouette specified so far is square.

## Context
Combat needs exact range checks, and collision uses the Silhouette as the unit's shape. The docs don't say what a distance is measured between. They also don't say whether a non-square silhouette turns with the unit.

## Options
**Range:**
- **A (default)**: from the attacker's centre to the nearest point of the target's silhouette or footprint. A unit can then fire at a building's edge.
- **B**: centre to centre. Large targets then need the attacker to come much closer.
- **C**: edge to edge.

**Proximity:**
- **A (default)**: the two shapes are within 8 space units of each other (1/8 tile), the same as mining reach.

**Silhouette rotation:**
- **A (default)**: it never rotates and stays axis-aligned. This is simpler and cheaper, and identical for square shapes.
- **B**: it rotates with facing. Long vehicles then turn into gaps differently.
