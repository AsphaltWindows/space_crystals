# extraction_plate_power_slowdown

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-extraction_plate_power_penalty.md

## Task

Apply power_ratio slowdown to the Extraction Plate mining system.

Currently `extraction_plate_mining_system` (faction.rs ~line 563) increments `plate_state.mining_timer` by 1 each tick unconditionally. When GDO power is negative, mining should slow down proportionally, matching how other production systems (barracks, DC construction, supply tower, HQ) already use `get_power_ratio_for_owner()`.

## Technical Context

### Files to modify

1. **`src/game/types/structures.rs` (line 230-240)** — `ExtractionPlateState`:
   - Change `mining_timer: u32` to `mining_timer: f32` (line 234)
   - Change `EXTRACTION_PLATE_MINING_INTERVAL: u32 = 48` to `f32 = 48.0` (line 240)
   - `EXTRACTION_PLATE_MINING_RATE` and `EXTRACTION_PLATE_RESIDUAL_RATE` remain `u32` (they are amounts, not timers)

2. **`src/game/world/faction.rs` (line 563-595)** — `extraction_plate_mining_system`:
   - The system already has a `players: Query<(&Player, &mut GdoPlayerResources)>` parameter (line 566) — reuse it
   - Call `get_power_ratio_for_owner(owner, &players)` (defined at line 1073 in the same file) for each plate
   - Change line 569 from `plate_state.mining_timer += 1` to `plate_state.mining_timer += power_ratio`
   - Change line 571 comparison to `>= EXTRACTION_PLATE_MINING_INTERVAL` (f32 comparison works directly)
   - Change line 572 reset to `plate_state.mining_timer = 0.0`
   - Both normal mining (line 577) and residual mining (line 582) happen per-cycle, so the slowdown is on the timer progression, not the amounts

### Reference pattern — `ef_construction_tick_system` (faction.rs line 860-881):
```rust
let power_ratio = get_power_ratio_for_owner(owner, &players);
if let Some(ref mut progress) = ef_state.construction_progress {
    *progress += power_ratio;
    if *progress >= required_frames { ... }
}
```
The extraction plate system follows the same pattern but simpler — no Option wrapper, just `plate_state.mining_timer += power_ratio`.

### Callers of `mining_timer` that need updates for f32:
- `faction.rs` line 2275-2278, 2300-2303: Test spawns set `mining_timer: 0` — change to `0.0`
- `utils.rs` line 386: `ExtractionPlateState` spawn sets `mining_timer: 0` — change to `0.0`
- `combat/systems/core.rs` line 1042: Test code sets `mining_timer: 0` — change to `0.0`

### System registration
- Already registered in `FixedUpdate` at `world/mod.rs:104` in `DiagCategory::Construction` — no changes needed

### Test to add (in faction.rs test module, after line ~2312):
Spawn a `Player` + `GdoPlayerResources` with power deficit (e.g., `power_generated: 50, power_consumed: 100` → ratio 0.5), spawn an `ExtractionPlateState` with `Owner::player(player_id)` + a `SpaceCrystalPatch`. Run the system 48 times (the normal interval). Verify mining has NOT completed yet. Run it 48 more times (total 96 ticks at 0.5 ratio ≈ 48 effective). Verify mining completed and crystals were added. Use `world.run_system_once(extraction_plate_mining_system)` pattern (same as depleted_patch_despawn tests at line 2258).

### Key types:
- `GdoPlayerResources` (factions.rs:85) — `power_ratio()` method returns `f32` (1.0 if sufficient, generated/consumed if deficit)
- `Player` component — has `player_number: u8`
- `Owner` component — `Owner::player(n)`, `player_number() -> Option<u8>`
- `get_power_ratio_for_owner(owner, &players) -> f32` (faction.rs:1073) — helper that matches Owner to Player and returns power_ratio

## Dependencies

None — this is a standalone change to an existing system. The `get_power_ratio_for_owner` helper and `GdoPlayerResources::power_ratio()` already exist and are used by other systems.
