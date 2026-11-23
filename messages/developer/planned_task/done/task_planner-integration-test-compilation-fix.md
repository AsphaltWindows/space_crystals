# integration-test-compilation-fix

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-integration-test-compilation-fix.md

## Task

Fix compilation errors in integration tests (tests/qa/) and scenario tests (tests/scenarios/) caused by the declarative entity definitions refactoring. All errors are import path updates and API adaptation — no source code changes needed.

## Technical Context

### Error Summary

There are **13 compilation errors across 6 QA test files** and **4 errors in 1 scenario test file** (19 total). The task description lists 5 files, but the actual errors span **7 files**.

### File-by-File Fix Guide

#### 1. `tests/qa/agent_tunnel_building_command_and_behavior.rs` (line 7)
- **Error**: `AGENT_TUNNEL_BUILD_FRAMES` not found in `game::units::types::unit_data`
- **Fix**: Change import from `space_crystals::game::units::types::unit_data::AGENT_TUNNEL_BUILD_FRAMES` to `space_crystals::game::entities::factions::syndicate::units::agent::AGENT_TUNNEL_BUILD_FRAMES`
- The constant is at `src/game/entities/factions/syndicate/units/agent.rs` line 63, value 480

#### 2. `tests/qa/guard_unit.rs` (lines 3-4, 39)
- **Error**: 5 missing imports from `game::units::types::unit_data`: `guard_type_data`, `guard_attack_data`, `GUARD_CONTROL_COST`, `GUARD_TUNNEL_SPACE_COST`, `GUARD_RUGGED_BONUS`
- **Error**: `spawn_syndicate_guard` not found in `game::utils` (line 39)
- **Fix for constants**: Import from `space_crystals::game::entities::factions::syndicate::units::guard::{GUARD_CONTROL_COST, GUARD_TUNNEL_SPACE_COST, GUARD_RUGGED_BONUS}`
- **Fix for `guard_type_data()`**: Replace with `SyndicateGuardDef::base_stats()` (returns `BaseStats { max_hp: 80, point_armor: 1, full_armor: 1, faction: TheSyndicate }`). Import `space_crystals::game::entities::factions::syndicate::units::guard::SyndicateGuardDef` and `space_crystals::game::entities::types::EntityDefinition`.
- **Fix for `guard_attack_data()`**: Replace with `SyndicateGuardDef::attack_data()` (returns `Option<AttackAttributesData>`). Import `space_crystals::game::entities::types::UnitDefinition`.
- **Fix for `spawn_syndicate_guard()`** (line 34-43): Replace the `run_system_once` block with `TestHarness::spawn_unit_at_grid()`. Pattern:
  ```rust
  let guard_entity;
  {
      let mut harness = TestHarness::new(&mut test_app.app);
      guard_entity = harness.spawn_unit_at_grid(ObjectEnum::SyndicateGuard, 20, 20, Owner(Some(0)));
  }
  ```
  The `TestHarness` is already imported via `helpers::*`. This pattern is used throughout other QA test files (e.g., `syndicate_agent_unit.rs` line 16-18).
- **Adapting test assertions**: Where tests check `type_data.faction`, `type_data.max_hp`, etc., use `SyndicateGuardDef::base_stats()` fields. Where tests check attack stats, use `SyndicateGuardDef::attack_data().unwrap()` fields. See `src/game/entities/factions/syndicate/units/guard.rs` lines 220-259 for the pattern used in the entity definition's own unit tests.

#### 3. `tests/qa/syndicate_agent_unit.rs` (lines 3, 25, 28)
- **Error**: `AGENT_CONTROL_COST` not found (was `use ...::unit_data::*` — the constant moved out)
- **Error**: `agent_type_data()` not found
- **Fix**: Remove `use space_crystals::game::units::types::unit_data::*;` (line 3, now empty/useless). Add:
  ```rust
  use space_crystals::game::entities::factions::syndicate::units::agent::{SyndicateAgentDef, AGENT_CONTROL_COST};
  use space_crystals::game::entities::types::{EntityDefinition, UnitDefinition};
  ```
- **Fix for `agent_type_data()` (line 28)**: Replace with `SyndicateAgentDef::base_stats()`. Adjust field access: `type_data.faction` → `base_stats.faction`, `type_data.max_hp` → `base_stats.max_hp`.
- The `UnitControlCost` import on line 23 (`space_crystals::game::units::types::unit_data::UnitControlCost`) should still work — `UnitControlCost` remains in `unit_data`.

#### 4. `tests/qa/gdo_supply_tower_and_chopper.rs` (lines 71, 78, 448)
- **Error**: `SupplyTowerState::production_cost()` does not exist (lines 71, 78)
- **Error**: `DeploymentCenterState::construction_cost()` does not exist (line 448)
- **Fix for `SupplyTowerState::production_cost(&ObjectEnum::SupplyChopper)`**: Replace with `EntityRegistry`-based cost lookup or direct entity def trait call. The simplest approach: use the entity definition directly:
  ```rust
  use space_crystals::game::entities::factions::gdo::units::supply_chopper::SupplyChopperDef;
  use space_crystals::game::entities::types::EntityDefinition;
  // ...
  let cost = SupplyChopperDef::cost(); // Returns Option<StructureCost>
  ```
  For the negative test (`Peacekeeper should not be buildable at Supply Tower`), this is now a different check — confirm that the SupplyTower's producible list doesn't include Peacekeeper. Alternatively, skip or rephrase this assertion since production routing is no longer a state method.
- **Fix for `DeploymentCenterState::construction_cost(&ObjectEnum::SupplyTower)`** (line 448): Replace with:
  ```rust
  use space_crystals::game::entities::factions::gdo::buildings::military::SupplyTowerDef;
  use space_crystals::game::entities::types::EntityDefinition;
  let cost = SupplyTowerDef::cost(); // Returns Option<StructureCost { space_crystals: 200, build_frames: 240 }>
  ```
  Note: `DeploymentCenter::cost()` returns `None` (line 73 of base.rs) because DC is the primary builder itself.

#### 5. `tests/qa/unit_cap_systems.rs` (lines 181, 240)
- **Error**: `BarracksState::production_cost()` does not exist
- **Fix**: Replace with direct entity def cost lookup:
  ```rust
  use space_crystals::game::entities::factions::gdo::units::peacekeeper::Peacekeeper;
  use space_crystals::game::entities::types::EntityDefinition;
  let cost = Peacekeeper::cost().unwrap(); // StructureCost { space_crystals, build_frames }
  ```
  Used on lines 181 and 240 to get `cost.build_frames` for setting `current_build_progress`.

#### 6. `tests/qa/tunnel_expansions_and_starting_condition.rs` (lines 178, 341)
- **Error**: `HeadquartersState::production_cost()` does not exist
- **Fix**: Replace with direct entity def cost lookup:
  ```rust
  use space_crystals::game::entities::factions::syndicate::units::agent::SyndicateAgentDef;
  use space_crystals::game::entities::types::EntityDefinition;
  let cost = SyndicateAgentDef::cost().expect("Agent should have a cost");
  ```
  On line 178: `cost.space_crystals` (100), `cost.build_frames` (160).
  On line 341: just `SyndicateAgentDef::cost().is_some()`.

#### 7. `tests/scenarios/main.rs` (lines 6, 111, 117, 157, 161, 210)
- **Error**: `spawn_peacekeeper`, `spawn_tunnel`, `spawn_headquarters` not found in `game::utils`
- **Error**: Type inference failure on `entity_mut(entity).insert(Selected)` (lines 117, 161, 210)
- **Fix for spawn functions**: Replace all `spawn_peacekeeper` calls with `TestHarness::spawn_unit_at_grid()`. The old pattern:
  ```rust
  let entity = app.world_mut().run_system_once(|mut commands, mut meshes, mut materials| {
      spawn_peacekeeper(&mut commands, &mut meshes, &mut materials, 30, 30, Owner(Some(0)))
  }).unwrap();
  ```
  Becomes:
  ```rust
  let entity;
  {
      let mut harness = TestHarness::new(&mut test_app.app);
      entity = harness.spawn_unit_at_grid(ObjectEnum::Peacekeeper, 30, 30, Owner(Some(0)));
  }
  ```
  Remove the `use space_crystals::game::utils::{spawn_peacekeeper, spawn_tunnel, spawn_headquarters};` import. `TestHarness` is already imported (line 11). If `spawn_tunnel` or `spawn_headquarters` are used, use `TestHarness::spawn_structure_at_grid()` and `TestHarness::spawn_headquarters_at_grid()` respectively.
- **Fix for type inference** on `entity_mut().insert(Selected)`: Add explicit `EntityWorldMut` return type annotation:
  ```rust
  let _: bevy::ecs::world::EntityWorldMut = test_app.app.world_mut().entity_mut(unit_entity).insert(Selected);
  ```
  Or cast to `()`: `let _ = test_app.app.world_mut().entity_mut(unit_entity).insert(Selected);`
  The issue is that Bevy 0.17's `insert()` returns a generic type that can't be inferred. The simplest fix is just `let _ = ...;` on each line.

### Key Entity Definition API (for reference)

| Old API | New API | Location |
|---------|---------|----------|
| `guard_type_data()` | `SyndicateGuardDef::base_stats()` | `entities::factions::syndicate::units::guard` |
| `guard_attack_data()` | `SyndicateGuardDef::attack_data()` | Same (via `UnitDefinition` trait) |
| `agent_type_data()` | `SyndicateAgentDef::base_stats()` | `entities::factions::syndicate::units::agent` |
| `spawn_syndicate_guard()` | `TestHarness::spawn_unit_at_grid()` | `shared::testing::harness` |
| `BarracksState::production_cost(obj)` | `<EntityDef>::cost()` | Per entity definition module |
| `HeadquartersState::production_cost(obj)` | `<EntityDef>::cost()` | Per entity definition module |
| `SupplyTowerState::production_cost(obj)` | `<EntityDef>::cost()` | Per entity definition module |
| `DeploymentCenterState::construction_cost(obj)` | `<EntityDef>::cost()` | Per entity definition module |

### Trait Import Paths
- `EntityDefinition` trait: `space_crystals::game::entities::types::EntityDefinition` (needed for `::base_stats()`, `::cost()`, `::object_type()`)
- `UnitDefinition` trait: `space_crystals::game::entities::types::UnitDefinition` (needed for `::attack_data()`, `::unit_stats()`)
- These are re-exported from `space_crystals::game::entities::*`

### Warnings to Fix (non-blocking but clean up)
- `helpers.rs` lines 37, 52: `send_event` is deprecated → use `write_message` (Bevy 0.17 API change). This is a warning, not an error — fix if convenient.
- `syndicate_agent_unit.rs` line 3: unused import warning after removing `unit_data::*`

### Verification
```bash
cargo test --test qa        # Must compile and all tests pass
cargo test --test scenarios  # Must compile
cargo test --lib             # Must still pass (2,094+ tests, 0 fail)
```

## Dependencies

None — this is a standalone compilation fix task. All source code changes were already completed in the entity definitions refactoring. Only test import paths and API calls need updating.
