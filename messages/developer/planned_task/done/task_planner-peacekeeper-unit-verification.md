# peacekeeper-unit-verification

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

This is a **verification-only task** — no code changes expected. All Peacekeeper code is already implemented and has extensive test coverage.

### Files to Inspect (all under `artifacts/developer/src/`)

1. **`game/types/objects.rs`** (line ~218): `ObjectEnum::Peacekeeper` variant and its `object_type()` match arm returns `ObjectType { name: "Peacekeeper", size: (24, 24), destructible: true, sight_range: 5, groupable: true }`. Also has `unit_control_cost()` returning `PEACEKEEPER_CONTROL_COST` (line ~378).

2. **`game/units/types/unit_data.rs`** (lines 113-148):
   - `peacekeeper_type_data()` — `LightInfantry`, `GDO` faction, silhouette 24x24, max_hp=50, point_armor=1, full_armor=1
   - `peacekeeper_attack_data()` — `FullyConnected`, `Ranged` subtype, `Ground` target, damage=10, range=4, aim=2, fire=1, cooldown=2, reload=12
   - `PEACEKEEPER_CONTROL_COST: u32 = 1` (line 145)
   - `PEACEKEEPER_RUGGED_BONUS: f32 = 0.5` (line 148)

3. **`game/types/structures.rs`** (line ~152): `BarracksState::production_cost(ObjectEnum::Peacekeeper)` returns `StructureCost { space_crystals: 50, build_frames: 80 }`

4. **`game/utils.rs`** (line ~396): `spawn_peacekeeper()` function that assembles the full entity with all components.

### Existing Tests (all values already verified)

- `unit_data.rs`: `peacekeeper_type_data_fields`, `peacekeeper_attack_data_fields`, `peacekeeper_attack_timing_*` (aim, fire, cooldown, reload), `peacekeeper_control_cost_is_one`, `peacekeeper_rugged_bonus_is_fifty_percent`, `peacekeeper_object_type_matches_ticket_spec`, `peacekeeper_is_unit_not_structure`, `peacekeeper_light_infantry_*` (no_turret, cannot_reverse, ground_domain), `peacekeeper_movement_speed_derivation`
- `objects.rs`: `test_is_resource_false_for_peacekeeper`, `test_destructible_peacekeeper_has_hp`, `test_peacekeeper_unit_control_cost`, `test_units_are_units`
- `structures.rs`: `bk_production_cost_peacekeeper` (asserts 50 SC / 80 frames), `bk_try_queue_*`, `bk_cancel_last_*`

### Verification Approach

Run: `cargo test -p space_crystals 2>&1 | grep -i peacekeeper` to see all peacekeeper tests pass. Then spot-check the source values against the spec. Every spec value has a dedicated unit test already, so passing tests = verified.

## Dependencies

None — this is a standalone verification task with no code changes expected.
