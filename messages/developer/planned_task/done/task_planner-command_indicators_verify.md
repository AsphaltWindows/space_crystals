# command-indicators-verify

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-command-indicators.md

## Task

Verify the existing CommandIndicators implementation against the feature request spec. The system appears fully implemented:

- **CommandIndicatorType** enum (Location, Object) in `game/units/types/types.rs`
- **CommandIndicator** component with owner_unit, indicator_type, target_entity, patrol_index
- **command_indicator_color()** maps commands to Green/Red/Orange per spec
- **command_has_indicator()** filters which commands show indicators
- **command_indicator_sync_system** in `game/units/systems/core.rs`: runs every frame, diffs desired vs existing indicators, despawns stale ones, spawns new ones. Handles Location (cylinder mesh at ground), Object (torus mesh parented to target entity). Materials cached per color (green/red/orange).
- Patrol produces 2 Location indicators (start index=0, end index=1)
- Extensive tests exist

**Verification checklist:**
1. Confirm all command->color mappings match spec (Move=Green, Attack=Red, AttackMove=Orange, AttackGround=Red, Patrol=Orange, Reverse=Green, Enter=Green)
2. Confirm all command->indicator type mappings match spec (Move=Location, Attack=Object, AttackMove=Location, AttackGround=Location, Patrol=Location x2, Reverse=Location, Enter=Object)
3. Confirm sync system only shows indicators for Selected units
4. Confirm indicators are despawned when unit deselected or command changes
5. Run `cargo test` to verify all existing tests pass
6. If everything matches, no code changes needed -- just confirm.

## Technical Context

### Key Files

1. **`artifacts/developer/src/game/units/types/types.rs`** (lines 9-66, 161-475):
   - `CommandIndicatorType` enum (line 9): `Location`, `Object` variants
   - `CommandIndicator` component (line 19): `owner_unit`, `indicator_type`, `target_entity`, `patrol_index`
   - `command_indicator_color()` (line 32): Maps commands to colors
   - `command_has_indicator()` (line 52): Filters which commands show indicators
   - ~30 unit tests (lines 161-475) covering indicator presence, colors, and component creation

2. **`artifacts/developer/src/game/units/systems/core.rs`** (lines 1256-1410):
   - `command_indicator_sync_system()` (line 1256): The main sync system
   - Query filter at line 1258: `(With<Unit>, With<Selected>)` -- only selected units get indicators
   - Diff algorithm (lines 1322-1337): Despawns stale indicators, keeps matching ones
   - Spawn logic (lines 1340-1409): Creates Location (Cylinder mesh) or Object (Torus mesh parented to target)
   - ~15 integration tests (lines 1680-2073) testing the full sync lifecycle

3. **`artifacts/developer/src/game/units/mod.rs`** (line 47):
   - System registration: `command_indicator_sync_system.after(right_click_move_command)` in `DiagCategory::Movement`

### Spec Compliance Analysis (pre-verified)

**Color mappings (line 32-48) vs spec:**
| Command | Spec Color | Code Color | Match? |
|---------|-----------|------------|--------|
| Move | Green | Green (0,1,0) | YES |
| Attack (AttackTarget) | Red | Red (1,0.2,0) | YES |
| AttackMove | Orange | Orange (1,0.6,0) | YES |
| AttackGround (AttackLocation) | Red | Red (1,0.2,0) | YES |
| Patrol | Orange | Orange (1,0.6,0) | YES |
| Reverse | Green | Green (0,1,0) | YES |
| Enter | Green | Green (0,1,0) | YES |

**Indicator type mappings (sync system lines 1276-1318) vs spec:**
| Command | Spec Type | Code Type | Match? |
|---------|----------|-----------|--------|
| Move | Location | Location | YES |
| Attack | Object | Object | YES |
| AttackMove | Location | Location | YES |
| AttackGround | Location | Location | YES |
| Patrol | Location x2 | Location x2 (indices 0,1) | YES |
| Reverse | Location | Location | YES |
| Enter | Object | Object | YES |

### Known Minor Discrepancy (NOT a spec violation)

`command_has_indicator()` (line 52) returns true for `Gather`, `DropOffResources`, and `BuildTunnel`, which are NOT in the original spec table. The sync system handles `BuildTunnel` correctly (Location, Green) but `Gather(Entity)` and `DropOffResources(Entity)` fall through to the `_ => {}` catch-all (line 1318) -- meaning `command_has_indicator` says they should show indicators but the sync system silently skips them. This is harmless for spec compliance but is a minor internal inconsistency. If you spot it, note it but do NOT fix it in this verification task -- it's outside scope.

### Selected-only check
- Line 1258: Query uses `With<Selected>` filter, so only selected units produce indicators
- When a unit is deselected (Selected component removed), next tick the sync system won't include it in `desired`, and any existing indicators for that unit will be despawned in the diff phase (lines 1322-1337)

### Running Tests
```bash
cd artifacts/developer && cargo test command_indicator
```
This will run all ~45 tests related to command indicators (both unit tests in types.rs and integration tests in core.rs).

## Dependencies

None. This is a standalone verification task. The command indicator system is fully self-contained with no pending dependencies on other tasks.
