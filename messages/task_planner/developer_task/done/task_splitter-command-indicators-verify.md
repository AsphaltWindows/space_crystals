# command-indicators-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

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
1. Confirm all command→color mappings match spec (Move=Green, Attack=Red, AttackMove=Orange, AttackGround=Red, Patrol=Orange, Reverse=Green, Enter=Green)
2. Confirm all command→indicator type mappings match spec (Move=Location, Attack=Object, AttackMove=Location, AttackGround=Location, Patrol=Location x2, Reverse=Location, Enter=Object)
3. Confirm sync system only shows indicators for Selected units
4. Confirm indicators are despawned when unit deselected or command changes
5. Run `cargo test` to verify all existing tests pass
6. If everything matches, no code changes needed — just confirm.
