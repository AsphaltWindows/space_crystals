# underground-walkability-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-underground-surface-walkability.md

## Task

Verify that underground tunnel expansions do NOT block surface unit movement. This functionality appears to already be fully implemented:

- `rebuild_occupancy_map` in `src/game/units/systems/core.rs` (around line 1092) already skips structures with `DomainEnum::Underground` when populating `blocked_tiles` and `structure_tiles`.
- `spawn_headquarters` in `src/game/utils.rs` assigns `DomainEnum::Underground` to the HQ expansion entity.

**What to do:**

1. Read the existing implementation and confirm it correctly handles this case.
2. If there is NOT already a test that verifies surface units can pathfind through tiles occupied by underground expansions, add one. The test should:
   - Spawn a Tunnel (surface structure) and an underground expansion (e.g., Headquarters) within its area.
   - Confirm the Tunnel's tiles are in `blocked_tiles` (surface structure blocks movement).
   - Confirm the HQ's tiles are NOT in `blocked_tiles` (underground expansion does not block).
3. If the implementation is already correct and tested, report completion with no code changes needed.
