# extraction_plate_power_slowdown

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-extraction_plate_power_penalty.md

## Task

Apply power_ratio slowdown to the Extraction Plate mining system.

Currently `extraction_plate_mining_system` (faction.rs ~line 563) increments `plate_state.mining_timer` by 1 each tick unconditionally. When GDO power is negative, mining should slow down proportionally, matching how other production systems (barracks, DC construction, supply tower, HQ) already use `get_power_ratio_for_owner()`.

**What to change:**

1. Add a query for players with `GdoPlayerResources` to get power_ratio (the system already queries `GdoPlayerResources` for adding crystals — reuse that or use `get_power_ratio_for_owner()`).

2. Instead of `plate_state.mining_timer += 1`, use `plate_state.mining_timer += power_ratio` where `power_ratio` is obtained from the owning player's `GdoPlayerResources::power_ratio()`.

3. This requires `mining_timer` to become `f32` instead of integer (if it isn't already), since power_ratio is a fractional value. Check the current type — if it's u32/i32, change it to f32 and update the interval comparison accordingly (`>= EXTRACTION_PLATE_MINING_INTERVAL as f32`). Reset to 0.0 on completion.

4. Both normal mining rate and residual mining rate are affected (the slowdown is on the timer/interval, not on the amount mined per cycle).

**Reference pattern** — see how `barracks_production_tick_system` or `supply_tower_production_tick_system` apply power_ratio to their progress counters.

**Files to modify:**
- `src/game/world/faction.rs` — `extraction_plate_mining_system`
- `src/game/types/structures.rs` or wherever `ExtractionPlateState` is defined — change `mining_timer` field type if needed

**Add a test** verifying that with power_ratio < 1.0, the mining cycle takes proportionally longer.
