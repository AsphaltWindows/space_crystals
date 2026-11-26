# Q0150 — The Colonists doc still says a building under Warp In has no rally point, against 0396

- **Date**: 2026-11-26T15:09:08-05:00
- **Blocking**: no. Slice M5-S6 built 0396, the later entry, for portals warping in.
- **Docs**: `factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel`; design change 0396 (rally points on unfinished buildings).

## Question
Does 0396 replace the Colonists doc's "no rally point" for a building under Warp In? If it does, the doc needs updating.

## Context
- `factions/colonists/economy.md > Beacon Deployment > While transforming: only Cancel` (user, 2026-11-24): "A Beacon under deployment and a building under Warp In show **only the Cancel command** in their CommandPanel (no rally point, no other commands)."
- 0396 (2026-11-27): "A building that will produce or release units can have its RallyPoint set while it is still under construction or transformation, for every faction. Examples: ... Colonists portals warping in." Its unfinished panel adds C: Set Rally Point next to Cancel.
- **As built (M5-S6):**
  - A Robo or Bio Portal under Warp In shows X Cancel and C Set Rally Point, and a right-click sets its rally point.
  - A cancelled Warp In drops the point.
  - A Beacon under deployment still shows only Cancel, since a Beacon trains nothing.

## Options
- **(a) 0396 wins.** The Colonists doc's "no rally point" is updated to allow a rally point on a building under Warp In that will produce units. Nothing changes in the build.
- **(b) The Colonists keep "only Cancel"** while warping in, as an exception to 0396. The Director removes C from warping portals.
