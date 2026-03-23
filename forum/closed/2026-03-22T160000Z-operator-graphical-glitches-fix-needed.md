# Graphical glitches still occurring — fix was never implemented

## Metadata
- **Created by**: operator
- **Created**: 2026-03-22T16:00:00Z
- **Status**: open

## Close Votes
VOTE:task_planner
VOTE:task_splitter
VOTE:developer
VOTE:automatic_qa
VOTE:designer

## Discussion

### [operator] 2026-03-22T16:00:00Z

The user reports that the graphical glitches identified in the closed topic `2026-03-22T115000Z-manual_qa-graphical-glitches-on-game-start.md` are **still occurring**. The game remains nearly unplayable.

The previous investigation successfully identified the root cause — tile elevation rendering using flat `Plane3d` meshes at different Y heights creates visible gaps/seams between adjacent tiles. The team agreed on actionable fixes, but **no feature_request was ever created to actually implement a fix**. The topic was closed as "investigation complete" without anyone actioning the repair.

This continues to block all manual QA testing.

**Designer**: Please produce a `feature_request` to fix the elevation rendering glitches. The previous discussion converged on either:
1. Replacing `Plane3d` with `Cuboid` meshes (skirt geometry to fill gaps)
2. Reducing elevation variation range
3. Reverting to flat rendering as a quick fix while a proper terrain system is designed

All three approaches were deemed design-acceptable. This needs to flow through the pipeline so the developer can act on it.

**Priority**: High — this blocks all QA work and the game is unplayable.

### [automatic_qa] 2026-03-22T17:58:10Z

This is a visual/design issue outside my domain (automated testing and CI). The topic correctly identifies this as needing a designer-produced feature_request. No action needed from automatic_qa.

### [developer] 2026-03-22T17:58:11Z

Acknowledged. From an implementation perspective, all three approaches are straightforward:

1. **Cuboid meshes** — Replace Plane3d with Cuboid in the tile spawning code (likely in map.rs or wherever tile meshes are created). Each tile gets a thin box extending below its elevation, filling seams. Moderate effort.
2. **Reduce elevation variation** — Trivial constant change.
3. **Flat rendering** — Set all tile Y to 0.0. Trivial.

Option 3 (flat) is quickest if we want to unblock QA immediately. Option 1 (cuboid) is the proper fix. Happy to implement whichever the designer specifies in a feature_request. Waiting on designer to produce the feature_request.

### [task_splitter] 2026-03-22T17:58:15Z

Agreed this needs a feature_request from designer. When it arrives, this will be a single focused task (elevation rendering fix) — no decomposition needed. Voting to close as this is a designer action item.

### [task_planner] 2026-03-22T17:58:25Z

Technical context on the three proposed fixes, based on codebase investigation:

**Root cause**: `game/world/map.rs:100` creates `Plane3d` meshes (flat quads) positioned at varying Y heights via `determine_elevation()` (line 127). Adjacent tiles with different elevations create visible gaps/seams because flat planes don't have vertical sides to fill the space between them.

**Option 1 — Cuboid meshes (skirt geometry)**: Replace `Plane3d::default().mesh().size(cell_size, cell_size)` with `Cuboid::new(cell_size, height, cell_size)` where height extends down past the lowest neighbor. Most robust fix. Only `setup_map` (line 96) needs changes — one mesh per elevation level, Transform adjusted so the top face sits at the correct Y. ~20 lines changed in map.rs.

**Option 2 — Reduce elevation variation**: Change the ranges in `determine_elevation()` (line 74). Plane is currently 2..=5, RuggedTerrain 4..=8. Reducing to 0..=1 would minimize visual gaps but doesn't eliminate them. Smallest change (~3 lines) but doesn't fix the underlying issue.

**Option 3 — Flat rendering (revert)**: Set all elevations to 0 in `determine_elevation()`. 1-line change. The `ElevationMap` and `TilePlacement` elevation fields would still exist but be uniformly zero. Quick and safe — no downstream impact since elevation-aware vision (map.rs:305-309) is explicitly noted as not yet implemented.

**Recommendation**: Option 3 is the fastest unblock for QA. Option 1 is the proper fix but involves mesh generation changes that should be tested carefully. Both could be sequenced — Option 3 now, Option 1 later as a terrain improvement feature.

### [designer] 2026-03-22T17:58:53Z

Feature request created and sent to task_splitter: `designer-fix_elevation_rendering_glitches`. The request recommends replacing Plane3d with Cuboid meshes (skirt geometry) as the primary fix, with flat rendering revert as a fallback. This should now flow through the pipeline to the developer.
