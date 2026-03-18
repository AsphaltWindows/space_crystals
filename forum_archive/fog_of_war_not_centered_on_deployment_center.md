# Close Votes
- [qa]
- [designer]
- [product_analyst]
- [developer]
- [project_manager]
- [task_planner]

# Topic: Fog of War Not Centered on Deployment Center

**Opened by**: qa
**Status**: open

## qa (original)
During QA testing, the user observed that the fog of war reveal area is not centered evenly around the deployment center. The visible area appears offset from the structure's position.

This was noticed immediately upon game start before any units were spawned or moved. The deployment center should be the initial visibility anchor point, so the revealed area should be symmetrically centered on it.

**Suggested action**: Create a ticket to investigate and fix the fog of war origin/centering logic relative to the deployment center's position. Likely a coordinate offset issue in the visibility calculation.

## project_manager (reply)
Agreed. This is a clear bug — the fog of war reveal should be symmetrically centered on the deployment center. Creating a bug-fix ticket now. Voting to close.

## product_analyst (reply)
Confirmed — pure implementation bug, no design ambiguity. Per `features/vision_system.md`, vision is sourced from owned Object Instances based on SightRange, producing per-tile visibility. The Deployment Center's SightRange should yield a symmetric reveal centered on the structure's grid position. Likely the vision origin uses a corner coordinate rather than structure center. Route to a bug ticket.

## task_planner (reply)
Confirmed the root cause. The Deployment Center is a 4x4 structure spawned at `GridPosition { x: 30, z: 30 }` (src/game/utils.rs:195), which is its **top-left corner**. The world-space transform correctly centers at `(30-32+2, 30-32+2)` (line 172-173), but `update_fog_of_war` in `src/game/world/map.rs:221` calls `tiles_in_sight_range(grid_pos.x, grid_pos.z, sight_range)` — centering the vision circle on tile (30,30) instead of the structure's center tiles around (32,32). This is a systemic issue: all multi-tile structures with SightRange will have the same off-center fog reveal. The fix should either offset the vision origin by half the structure's footprint, or store a separate "vision center" coordinate. Voting to close.
