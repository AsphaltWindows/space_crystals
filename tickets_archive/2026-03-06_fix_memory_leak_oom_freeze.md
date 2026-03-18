# Ticket: Fix Memory Leak Causing OOM Kill After ~3 Minutes

## Current State
The game becomes unresponsive and is killed by the OS OOM killer after approximately 2-3 minutes of normal gameplay. Memory consumption grows unboundedly during play. Reproduced twice during QA testing with both minimal gameplay (selecting units, issuing move commands) and heavier gameplay (building structures, producing units). No panic or error output — the process is simply SIGKILL'd by the kernel.

## Desired State
The game runs indefinitely without unbounded memory growth. Entity counts and memory allocations remain stable during normal gameplay. The game should be playable for extended sessions (30+ minutes) without degradation.

## Justification
Critical blocker identified via forum topic `forum/game_freeze_memory_leak.md`. Blocks meaningful QA testing of all 46 pending tasks — the game is unplayable beyond ~3 minutes.

**Investigation checklist** (from QA's analysis):
1. **Entity/component leak**: Systems spawning entities every frame without cleanup (particles, indicators, UI elements, debug visualizations). Check all `commands.spawn()` calls for matching despawn logic.
2. **Grid line rendering**: `draw_grid_lines` may be spawning new line entities every frame instead of reusing existing ones.
3. **Fog of war computation**: `update_fog_of_war` runs in FixedUpdate — check if it accumulates data structures across frames.
4. **Pathfinding allocation leak**: A* or movement systems allocating per-frame without releasing.

## QA Steps
1. Add `bevy_diagnostic::EntityCountDiagnosticsPlugin` and `LogDiagnosticsPlugin` to the app (if not already present) to log entity counts per frame.
2. Launch the game, select GDO faction.
3. Build 2-3 structures (EnergyFoundation, PowerPlant, Barracks).
4. Produce 5 Peacekeeper units.
5. Issue move commands to all 5 units.
6. Observe diagnostic output for 5+ minutes — entity count should remain stable (not growing every frame).
7. After 10 minutes of play, confirm the game is still responsive and memory usage (via system monitor) has not grown significantly beyond initial load.
8. Remove diagnostic plugins after verification (or gate behind a debug feature flag).

## Expected Experience
- The game launches and runs normally.
- During the 10-minute play session, the game remains responsive with stable frame rates.
- Diagnostic logs (if enabled) show entity counts stabilizing after initial setup — no unbounded growth.
- Memory usage (observed via system monitor) plateaus after the first minute and does not continuously climb.
- The game does not freeze, slow down, or get killed by the OS.
