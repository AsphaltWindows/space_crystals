# peacekeeper-unit-verification

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-peacekeeper-unit.md

## Task

**Verification task — the Peacekeeper unit is already fully implemented.**

Verify that the existing Peacekeeper implementation matches the design spec. All code is in place:

- `ObjectEnum::Peacekeeper` in `game/types/objects.rs` — 24x24 silhouette, destructible, sight_range=5, groupable=true
- `peacekeeper_type_data()` in `game/units/types/unit_data.rs` — LightInfantry, GDO faction, MaxHP=50, PointArmor=1, FullArmor=1
- `peacekeeper_attack_data()` — FullyConnected, Ground, damage=10, range=4gu, aim=2/fire=1/cooldown=2/reload=12
- `PEACEKEEPER_CONTROL_COST=1`, `PEACEKEEPER_RUGGED_BONUS=0.5`
- `spawn_peacekeeper()` in `game/utils.rs`
- Barracks production: 50 SC, 80 frames (in `game/types/structures.rs`)
- Extensive tests already exist

Verify:
1. Run `cargo test` and confirm all peacekeeper-related tests pass
2. Confirm `ObjectEnum::Peacekeeper.object_type()` returns correct values (24x24, destructible, sight_range=5, groupable=true)
3. Confirm `peacekeeper_type_data()` matches spec (LightInfantry, GDO, HP=50, armor 1/1)
4. Confirm `peacekeeper_attack_data()` matches spec (FullyConnected, Ground, damage=10, range=4, aim=2/fire=1/cooldown=2/reload=12)
5. Confirm Barracks production cost is 50 SC / 80 frames
6. Confirm UnitControlCost is 1

No code changes expected unless a discrepancy is found.
