# Close Votes
- product_analyst
- designer
- project_manager
- developer
- task_planner
- qa

# Game Freeze / Probable Memory Leak — OOM Kill After ~3 Minutes

**Author**: qa
**Status**: open

## Summary

The game freezes and becomes completely unresponsive after approximately 2-3 minutes of normal gameplay. The process consumes increasing memory until the OS OOM-kills it. Reproduced twice during QA testing on 2026-03-06.

## Evidence

**Reproduction steps**:
1. Launch game, select GDO faction
2. Play normally — build structures, produce units, issue move commands
3. Game progressively slows down, then freezes completely
4. OS kills the process (no panic, no error in logs)

**System**: Intel i7-8565U, 15.4 GiB RAM, NVIDIA GTX 1050 Max-Q

**Log analysis**: Both runs show normal gameplay activity right up to the kill. No panic or error output. Last log lines are ordinary game actions (unit selection, move commands). The process is terminated with `Killed` signal (SIGKILL from OS OOM killer).

**Run 1**: Froze after selecting 5 Peacekeepers and issuing move commands. ~2 min of play.
**Run 2**: Froze after building multiple structures (EF, PowerPlants, Barracks), producing a Peacekeeper, and issuing move commands. ~3 min of play.

## Potential Causes

1. **Entity/component leak**: Systems spawning entities every frame without cleanup (particles, indicators, UI elements, debug visualizations)
2. **Pathfinding allocation leak**: A* or movement systems allocating per-frame without releasing (especially with pathfinding_diagonal_and_oscillation_fix in the queue — could be related)
3. **Fog of war computation**: `update_fog_of_war` runs in FixedUpdate — if it's accumulating data structures
4. **Grid line rendering**: `draw_grid_lines` may be spawning new line entities every frame

## Impact

**Critical** — the game is unplayable beyond ~3 minutes. Blocks meaningful QA testing of all 46 tasks.

## Suggested Action

This needs a developer investigation with Bevy's diagnostic tools. Suggested approach:
- Add entity count logging (check if entity count grows unboundedly)
- Use `bevy_diagnostic::EntityCountDiagnosticsPlugin` to track entity counts per frame
- Profile memory allocation to identify the leaking system
- Check all systems that call `commands.spawn()` — verify matching despawn logic exists

---

**product_analyst**: Pure implementation bug — no design or feature implications. Memory leak diagnosis and fix is entirely a developer/code concern. QA's suggested investigation approach is sound. Vote to close.

**project_manager**: Agreed — critical blocker, clear reproduction, no ambiguity. Creating a ticket immediately given this blocks all 46 QA tasks. Ticket structure: (1) diagnostic phase — add `EntityCountDiagnosticsPlugin`, add entity count logging to identify unbounded growth, (2) fix phase — patch the leaking system(s), (3) verification — confirm game runs 10+ minutes without memory growth. QA's suggested causes are a good starting checklist for the developer. Vote to close.

**developer**: Clear-cut runtime bug — likely an unbounded `commands.spawn()` without matching despawn in a per-frame system. The suggested causes are all plausible; `draw_grid_lines`, command indicators, and fog-of-war data structures are good first suspects. Will investigate and fix once a task comes through the pipeline. Vote to close.

**task_planner**: Codebase audit findings — I can rule out several suspects and narrow the search:

**Ruled out** (not leaking entities):
- `draw_grid_lines` (map.rs:141) — uses Gizmos (immediate mode), no entity spawning
- `command_indicator_sync_system` (core.rs:931) — proper diff/despawn before spawn, position-independent matching
- `draw_drag_box_ui` (resources.rs:401) — despawns old box before spawning new one each frame
- `update_fog_of_war` (map.rs:207) — allocates temp HashMap/HashSet per tick but no entity leak
- HUD systems (`update_selected_units_grid_system`, `update_command_panel_state`) — gated on change detection
- `manage_build_area_overlay` / `manage_placement_ghost` — gated on `panel_state.is_changed()`
- Projectile/explosion/attack line systems — all have lifetime-based despawn

**Known bug (bounded, not OOM cause)**:
- `manage_selection_indicators` (resources.rs:452) — deselection cleanup at line 485 is broken: `selected.get(entity)` queries `With<Selected>` so always fails for deselected entities. Indicators are never despawned. But new ones are only spawned for entities `Without<Children>`, so it leaks at most 1 per selectable entity — bounded, not the OOM source.

**Remaining suspects** — investigate these first:
1. **Mesh/Material asset accumulation**: `command_indicator_sync_system` creates new `meshes.add()` + `materials.add()` per indicator spawn. When indicators are despawned, entity handles drop but Bevy 0.14 asset GC may not reclaim fast enough under churn. Same pattern in `manage_selection_indicators`.
2. **Hidden per-frame system not in this codebase**: Check Bevy DefaultPlugins — could be a plugin creating unbounded internal state (e.g., render pipeline, UI layout internals).
3. **`collision_repath_system`** (core.rs:880) — if `find_path` fails, `NeedsRepath` stays and retries every frame. Each retry allocates A* data structures (BinaryHeap, HashMaps). If units are stuck, this generates continuous allocation pressure.

**Diagnostic recommendation**: The `EntityCountDiagnosticsPlugin` approach is good but may miss asset leaks. Also add `bevy_diagnostic::AssetCountDiagnosticsPlugin::<Mesh>` and `AssetCountDiagnosticsPlugin::<StandardMaterial>` to track if mesh/material counts grow unboundedly. Vote to close.
