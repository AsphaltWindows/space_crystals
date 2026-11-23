# add_colonists_bioport

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Added a new Colonists structure, the Bioport, to `artifacts/designer/design/colonists_objects.md`. This is the Colonists' basic military production building — the counterpart to GDO's Barracks and Cults' Armory — filling part of the "Colonists currently have zero combat units" gap noted in designer insights. The actual combat unit(s) it produces are NOT yet designed (still TBD, tracked as a loose end) — this feature_request covers only the building itself: its stats, production framework, and Warp In cost.

Changes to `colonists_objects.md`:
- New `## Bioport` section, placed after `## Roboport`, following the same structural pattern (Entity Type, Faction, Size, HP/Shields/Armor/SightRange, Destructible, Groupable, SymmetryType).
- Stats: 3x3 grid units, 250 MaxHP, 15 MaxShields, 1/9 armor, SightRange 3, Groupable true, SymmetryType AABB.
- New **Exit Behavior** subsection: unlike the Roboport (AAAA, fully symmetric, no fixed exit side), the Bioport's AABB symmetry gives it two exit-eligible "A" sides. A produced unit exits from whichever eligible A side is closest to the RallyPoint; if no RallyPoint is set, or the closest side is blocked/obstructed, the unit exits from whichever eligible A side is unblocked, chosen randomly among unblocked candidates.
- `BioportInstanceState`: RallyPoint, BuildQueue (max 20), CurrentBuild, CurrentBuildProgress — identical shape to `RoboportInstanceState`.
- RallyPoint behavior: same pattern as GDO's Barracks / Colonists' Roboport (issues the default right-click command against the RallyPoint target on spawn; resets to None if the referenced ObjectInstance no longer exists).
- `Produces:` section explicitly marked TBD — no combat units are defined yet for the Colonists faction. Do not implement production output/build-queue contents until a follow-up feature_request defines the actual unit(s).
- Added a Bioport row to the Constructs List table (in the `## Warp In` section): 150 Space Crystals, 480 frames (30 seconds) WarpInTime — this is the cost/time to Warp In a Bioport from an established Beacon, distinct from whatever the Bioport itself will later charge to produce units.
- Full ObjectInterfaceState (panel layout, which button queues a unit) is TBD, consistent with the same still-open item on the Roboport.

## QA Instructions

1. Get an established Colonists Beacon and trigger Warp In. Confirm "Bioport" now appears as a selectable option in the Constructs list, costing 150 Space Crystals with a 480-frame (30 second) transformation, during which HP/Shields interpolate per the existing Warp In interpolation formula (from the Beacon's own MaxHP/MaxShields toward the Bioport's 250 HP / 15 Shields).
2. Confirm the completed Bioport has: 250 MaxHP, 15 MaxShields, 1 PointArmor / 9 FullArmor, SightRange 3, is Groupable, and its placement ghost only offers the orientations consistent with AABB symmetry (fewer distinct orientations than a fully asymmetric building, more than a fully symmetric AAAA one).
3. Since no combat unit is defined yet, the Bioport's build queue/production UI is not expected to be functional yet — this feature only covers the structure existing, being placeable via Warp In, and having correct stats. Do not expect a "produce unit" button to work until a follow-up feature adds an actual Colonists combat unit.
4. If a rally point mechanic is testable in isolation (e.g., via a debug/test unit substituted into the build queue), confirm produced units exit from whichever of the Bioport's two "A" sides is closest to the set RallyPoint, and that a blocked/obstructed near side causes the unit to exit from the other (unblocked) side instead.
