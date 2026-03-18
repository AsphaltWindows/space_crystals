# Ticket: Per-System Performance Diagnostics Plugin

## Current State
The game has no per-system performance visibility. Bevy's built-in `FrameTimeDiagnosticsPlugin` provides only overall frame time. As systems are added (combat, pathfinding, fog of war, power grids, etc.), there is no way to identify which systems are becoming bottlenecks without external profiling tools.

## Desired State
A custom Bevy diagnostics plugin that:

1. **Per-system timing**: Measures wall-clock time spent in each major system category per tick. Categories should include (at minimum): movement, combat, pathfinding, rendering, UI/HUD, selection, fog of war, power grid, and any other major system groups.
2. **Dual-clock tracking**: Simulation-tick systems (running at 16fps) and render-frame systems are tracked separately on their respective clocks.
3. **Rolling-window aggregation**: Collects timings over a rolling window defined in wall-clock seconds (e.g., last 2 seconds), computing min/avg/max per category. Using wall-clock seconds avoids ambiguity between render frames (~60fps) and simulation frames (16fps).
4. **Debug overlay**: An in-game overlay toggled with a dedicated key (F3) that displays the aggregated per-system metrics in real time. Shows category name, min, avg, max, and percentage of total frame time.
5. **Console logging**: Optionally logs the same aggregated metrics to the console at a configurable interval (e.g., every 5 seconds), disabled by default.
6. **Zero-cost when disabled**: The instrumentation should have negligible overhead when the overlay is not active, or should be compiled out in release builds via a cargo feature flag.

## Justification
This is diagnostic infrastructure requested by the user via forum topic `forum/performance_metrics_per_tick.md`. The codebase is growing rapidly with combat systems, pathfinding, fog of war, power grids, and more being added. Without per-system performance visibility, bottlenecks will go undetected until they cause user-facing issues. Instrumenting now means every subsequent system gets visibility for free. There is no corresponding `/features` file -- this is pure developer/QA tooling infrastructure.

## QA Steps

1. Build and run the game with the diagnostics plugin enabled.
2. Press F3 to toggle the debug overlay on.
3. Verify the overlay appears, showing a list of system categories with min/avg/max timing columns.
4. Verify the timing values update in real time (not frozen).
5. Verify simulation-tick systems (e.g., movement, combat) and render-frame systems (e.g., rendering, UI/HUD) are listed in separate sections or clearly labeled.
6. Trigger a scenario that loads a specific system (e.g., spawn many units to stress movement/pathfinding).
7. Verify the corresponding system category shows increased timing values relative to others.
8. Press F3 again to toggle the overlay off. Verify it disappears.
9. Enable console logging (via configuration or code toggle). Verify aggregated metrics print to the console at the configured interval.
10. Verify the game runs without noticeable performance degradation when the overlay is off.

## Expected Experience

- **Step 2-3**: A semi-transparent overlay appears in a corner of the screen listing system categories (e.g., "Movement", "Combat", "FogOfWar", "Rendering") each with three numeric columns (min/avg/max in milliseconds or microseconds) and a percentage column.
- **Step 4**: The numbers fluctuate slightly each update cycle, reflecting real workload variance.
- **Step 5**: Simulation systems and render systems are visually distinguishable (separate headings, different sections, or labeled with [Sim] / [Render] tags).
- **Step 6-7**: After spawning many units, the movement/pathfinding row shows noticeably higher values (e.g., jumping from <0.1ms to several ms), confirming the instrumentation correctly attributes load.
- **Step 8**: The overlay disappears cleanly with no visual artifacts.
- **Step 9**: Console output shows a formatted table of the same metrics, printed periodically.
- **Step 10**: With overlay off, frame rate remains consistent with pre-instrumentation performance.
