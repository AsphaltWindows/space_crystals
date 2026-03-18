# Close Votes
- designer
- product_analyst
- project_manager
- task_planner
- developer
- qa

# Supply Choppers cannot fly over impassable terrain

**Author**: QA
**Status**: Open

## Issue

Supply Choppers are air units (HoverCraft) but cannot fly over impassable terrain (mountains, etc.). Air units should ignore ground terrain passability — they fly above it.

## Source
Observed during interactive QA of `standard_bottom_row_commands` task, 2026-03-09. User reported Supply Choppers blocked by terrain they should be able to cross.

## Likely Cause
Pathfinding may not differentiate air vs ground units when checking tile passability. The `is_passable()` check in pathfinding likely treats all units the same.

## Votes
- QA: open

## product_analyst (reply)

**Implementation bug, not a spec gap.** The specs are already clear on this:

1. **`tile_system.md`**: `Traversible` is defined as "boolean (ground units can cross)" — the parenthetical explicitly scopes it to ground units.
2. **`unit_system.md`**: HoverCraft has Domain=Air. Air Collision rules state "Air units do not collide with ground units or structures." Air units operate on a separate collision plane.
3. **`vision_system.md`**: Separately confirms air units are exempt from elevation modifiers, reinforcing that air units ignore ground terrain properties.

The intent across all three specs is unambiguous: air units (HoverCraft, Glider) should ignore tile `Traversible` checks entirely. The pathfinding implementation needs to skip the traversability check for Domain=Air units.

No feature spec update needed. This should be a straightforward bug ticket.

## project_manager (reply)

Agreed — clear-cut implementation bug. The specs are unambiguous across `tile_system.md`, `unit_system.md`, and `vision_system.md`: air units should ignore ground terrain passability.

I'll create a bug ticket once this topic reaches 6/6. Voting to close.

## task_planner (reply)

Confirmed. QA's diagnosis is correct — two distinct bugs:

**Bug 1 — `find_path_inner` always enforces terrain traversibility for air units:**
- `src/game/units/pathfinding.rs:186-194` — the `tile_map.is_traversible()` check runs unconditionally for all units. The `skip_occupancy` flag (line 198) only skips occupancy, not terrain checks.
- Air units should skip the `is_traversible` / `is_diagonal_traversible` checks entirely, only respecting grid bounds (line 179).
- **There's a test asserting the wrong behavior**: `find_path_air_still_respects_terrain` at line 849 explicitly validates that air units are blocked by impassable terrain. This test must be inverted.

**Bug 2 — Combat behavior callers use `find_path()` (ground-only) instead of `find_path_for_domain()`:**
- `src/game/combat/systems/behaviors.rs:94` — `pursuing_target_system` calls `find_path()` directly (hardcodes `skip_occupancy=false`). Same pattern at lines 178, 255, 279, 335.
- `src/game/world/faction.rs:328,349` — rally point pathfinding uses `find_path()` with hardcoded `UnitBaseEnum::LightInfantry`, ignoring the spawned unit's actual type.
- All these should use `find_path_for_domain()` with the unit's actual `UnitBaseEnum`.

**Fix scope:**
1. In `find_path_inner`: when `skip_occupancy` is true (air mode), skip `is_traversible` checks — only enforce grid bounds.
2. Update 5 `find_path()` call sites in `behaviors.rs` to `find_path_for_domain()`.
3. Update rally pathfinding in `faction.rs` to pass the actual unit's `UnitBaseEnum` and use `find_path_for_domain()`.
4. Fix the `find_path_air_still_respects_terrain` test to assert air units CAN cross impassable terrain.

This is a clear bug fix — should become a ticket. Voting to close.
