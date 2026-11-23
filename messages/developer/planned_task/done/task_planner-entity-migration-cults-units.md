# entity-migration-cults-units

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Migrate all 3 Cults units into per-entity definition files.

**Create 3 files:**
- src/game/entities/cults/units/recruit.rs
- src/game/entities/cults/units/soldier.rs
- src/game/entities/cults/units/gunner.rs

For each, implement EntityDefinition + UnitDefinition traits. Migrate from:
- Type data and attack data functions in unit_data.rs (recruit_type_data, soldier_type_data, gunner_type_data, etc.)
- Unit-specific constants (control costs, any special attributes)
- Command panel slots from grid_layout.rs
- Spawn logic from respective spawn_*() functions in utils.rs
- Production costs from CultsArmory or RecruitmentCenter production_cost() if applicable

Register all 3 in build_registry(). Keep old functions as thin wrappers during transition.

**Tests per entity file:**
- Stats match current type_data values
- Attack data matches current attack_data values
- Entity-specific constants preserved
- Command slots match current grid_layout.rs entries
- Cost matches current production cost values
- Spawn produces same components

**Reference:** Architecture doc Phase 5 (Cults Entities)

## Technical Context

### Pattern to Follow

Follow the Syndicate Agent definition as the primary template:
- `artifacts/developer/src/game/entities/factions/syndicate/units/agent.rs` — full example of UnitDefinition with attack data, spawn, command slots, constants, and tests
- `artifacts/developer/src/game/entities/factions/syndicate/units/guard.rs` — similar pattern for a ranged unit

### Files to Create

All 3 files go under `artifacts/developer/src/game/entities/factions/cults/units/`:

1. **recruit.rs** — CultsRecruit definition
2. **soldier.rs** — CultsSoldier definition
3. **gunner.rs** — CultsGunner definition

### Files to Modify

1. **`artifacts/developer/src/game/entities/factions/cults/units/mod.rs`** — Currently just a doc comment. Add `pub mod recruit;`, `pub mod soldier;`, `pub mod gunner;`

2. **`artifacts/developer/src/game/entities/registry.rs`** — Add 3 `registry.register_unit::<T>()` calls in `build_registry()` and update the test assertions (`registry.len()` goes from 4 to 7). Import the 3 definition structs.

3. **`artifacts/developer/src/game/utils.rs`** — Keep `spawn_cults_recruit()` (line 1156), `spawn_cults_soldier()` (line 1197), `spawn_cults_gunner()` (line 1238) as thin wrappers during transition. They are called from `game/world/faction/production.rs` lines 412, 475, 479.

### Source Data Locations for Migration

**CultsRecruit:**
- ObjectType: `game/types/objects.rs` line 303 — name="Recruit", size=(1,1), destructible=true, sight_range=3, groupable=true
- Max HP: `game/utils.rs` line 1152 — `RECRUIT_MAX_HP = 50.0`
- Spawn: `game/utils.rs` lines 1156-1190 — spawns with UnitBaseEnum::LightInfantry (placeholder), UnitCommand::Idle, LocomotionChannel::Stationary, OrientationChannel::Maintaining, Velocity(Vec3::ZERO). NOTE: spawn is minimal — no AttackCapability, no CommandQueue, no BaseBehaviorState, no BaseCommandState, no MovementSpeed/RotationSpeed, no TurnRateMovementParams, no Armor, no Silhouette, no SightRange, no VisibilityStateEnum.
- Command slots: `ui/command_panel/grid_layout.rs` lines 169-180 — CultsRecruitMenu with 3 states:
  - RecruitDefault: (0,0)=RecruitConstruct, (1,1)=RecruitAssistConstruction
  - RecruitConstructMenu: (0,0)=RecruitSelectBuilding(CultsStorage), (2,0)=Back
  - RecruitAwaitingPlacement: None (no grid buttons)
- No attack data — Recruit is unarmed (no type_data or attack_data functions exist)
- Cost: RC auto-produces at no resource cost (192 frames base), so `cost()` should return `None`
- Control cost: 1 (per design doc each Recruit is 1 UC, tracked via OriginatingCenters)
- Interface state: `ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState)` — imported from `crate::ui::types`

**CultsSoldier:**
- ObjectType: `game/types/objects.rs` line 310 — name="Soldier", size=(1,1), destructible=true, sight_range=5, groupable=true
- Max HP: `game/utils.rs` line 1193 — `SOLDIER_MAX_HP = 100.0`
- Spawn: `game/utils.rs` lines 1197-1231 — same minimal pattern as Recruit (UnitBaseEnum::LightInfantry placeholder)
- Command slots: Soldier uses `ObjectInterfaceState::Default` (standard unit) — slots come from the Default grid in grid_layout.rs lines 147-156. Provide full unit command slots like Guard does (Move, HoldPosition, Attack, Patrol, Stop). Whether it gets Reverse or AttackGround depends on its UnitBaseEnum — currently LightInfantry placeholder.
- Cost: Training cost from Armory: `SOLDIER_TRAINING_COST = 75` SC, `SOLDIER_TRAINING_FRAMES = 160` frames (`game/types/structures.rs` lines 480-481). Return `Some(StructureCost { space_crystals: 75, build_frames: 160 })`.
- No attack data functions exist yet (placeholder unit). If stats are TBD, return `None` from `attack_data()` and leave as unarmed placeholder.

**CultsGunner:**
- ObjectType: `game/types/objects.rs` line 317 — name="Gunner", size=(1,1), destructible=true, sight_range=6, groupable=true
- Max HP: `game/utils.rs` line 1234 — `GUNNER_MAX_HP = 80.0`
- Spawn: `game/utils.rs` lines 1238-1272 — same minimal pattern
- Command slots: Gunner also uses `ObjectInterfaceState::Default` — same Default grid slots as Soldier.
- Cost: `GUNNER_TRAINING_COST = 100` SC, `GUNNER_TRAINING_FRAMES = 200` frames (`game/types/structures.rs` lines 482-483). Return `Some(StructureCost { space_crystals: 100, build_frames: 200 })`.
- No attack data functions exist yet (placeholder unit). Return `None` from `attack_data()`.

### Traits to Implement

From `artifacts/developer/src/game/entities/types.rs`:
- `EntityDefinition`: `OBJECT` (ObjectEnum), `object_type()`, `base_stats()`, `cost()`, `command_slots()`, `spawn()`
- `UnitDefinition`: `unit_stats()`, `attack_data()`, `movement_params()`

### Key Types and Imports

- `ObjectEnum::CultsRecruit`, `ObjectEnum::CultsSoldier`, `ObjectEnum::CultsGunner` — all exist in `shared/types.rs`
- `FactionEnum::TheCults` — for BaseStats
- `UnitBaseEnum::LightInfantry` — current placeholder for all 3 (may change later)
- `CultsRecruitMenuState` — from `crate::ui::types` (for Recruit command slots)
- `CommandButtonAction::RecruitConstruct`, `RecruitAssistConstruction`, `RecruitSelectBuilding`, `Back` — for Recruit slots
- `StructureCost` — from `crate::game::types::structures` (for Soldier/Gunner costs)
- `MovementParams` — from `crate::game::units::types::movement`. Current spawns don't include TurnRateMovementParams, so use a reasonable placeholder (e.g. same as Agent's TurnRate with LightInfantry speed).
- `SpawnParams` — from entity definition types. `spawn()` takes `&mut EntityCommands` (not `&mut Commands`), so use `commands.insert()` pattern, no mesh/material.

### Spawn Function Notes

The existing `spawn_cults_*` functions in utils.rs use `Commands::spawn()` with Mesh3d/MeshMaterial3d (visual components). The EntityDefinition `spawn()` takes `&mut EntityCommands` and should only insert gameplay ECS components (no mesh/material handles available in SpawnParams).

The existing spawns are notably incomplete — they lack:
- CommandQueue, BaseCommandState, BaseBehaviorState
- AttackCapability, AttackState (for armed units)
- Armor, Silhouette, SightRange, VisibilityStateEnum
- MovementSpeed, RotationSpeed, TurnRateMovementParams

The entity definition `spawn()` should include ALL gameplay components (following Agent/Guard pattern), even though the current spawns are minimal. This is the right time to make spawn comprehensive.

### Constants to Define Per Entity

Following Agent pattern, define per-entity constants for:
- Control cost (1 per Recruit per design doc)
- Max HP (50/100/80 respectively)
- Move speed, rotation speed, turn rate (use LightInfantry-appropriate values)

### Test Pattern

Follow the test pattern in `agent.rs` lines 246-418:
- `{unit}_base_stats` — verify max_hp, armor, faction
- `{unit}_object_type` — verify name, size, destructible, sight_range, groupable
- `{unit}_unit_stats` — verify unit_base, silhouette, control_cost
- `{unit}_attack_data_values` — verify attack data or assert None
- `{unit}_movement_params` — verify movement model
- `{unit}_cost` — verify cost or assert None
- `{unit}_command_slots_count` — verify slot count
- Individual command slot tests for important slots
- `{unit}_registry_lookup` — register + lookup + verify
- `{unit}_object_enum` — verify OBJECT const
- `{unit}_constants` — verify constant values

## Dependencies

- **entity-registry-grid-layout** (sibling task) — that task wires registry into grid_layout.rs to replace the match arms; this task just defines the command_slots data. No blocking dependency — both can proceed in parallel since this task adds data and the other task adds the consumer.
- **Foundation system** (already implemented) — EntityDefinition/UnitDefinition traits and EntityRegistry in `entities/types.rs` and `entities/registry.rs` are already in place.
- **ObjectEnum variants** (already exist) — CultsRecruit, CultsSoldier, CultsGunner are already defined in shared/types.rs.
