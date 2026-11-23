# entity-migration-gdo-buildings

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Migrate all 6 GDO structures into per-entity definition files following the established pattern.

**Create 6 files:**
- src/game/entities/gdo/buildings/deployment_center.rs
- src/game/entities/gdo/buildings/power_plant.rs
- src/game/entities/gdo/buildings/barracks.rs
- src/game/entities/gdo/buildings/extraction_facility.rs
- src/game/entities/gdo/buildings/extraction_plate.rs
- src/game/entities/gdo/buildings/supply_tower.rs

For each, implement EntityDefinition + StructureDefinition traits. Migrate from:
- Structure stats from structures.rs gdo_structure_stats module (HP, armor, power, symmetry, build_radius)
- Construction/production costs from DeploymentCenterState::construction_cost() and BarracksState::production_cost()
- Command panel slots from grid_layout.rs
- Spawn logic from respective spawn_*() functions in utils.rs
- ObjectType data from objects.rs

Register all 6 in build_registry(). Keep old functions as thin wrappers during transition.

**Tests per entity file:**
- Stats match current gdo_structure_stats values
- Construction/production costs match current match arms
- Command slots match current grid_layout.rs entries
- Spawn produces same components

**Reference:** Architecture doc Phase 3 (GDO Entities)

## Technical Context

### Directory Structure

Files go under `artifacts/developer/src/game/entities/factions/gdo/buildings/`. The `mod.rs` already exists (at `factions/gdo/buildings/mod.rs`) but is nearly empty — just a doc comment. Add `pub mod deployment_center;` etc. for each new file.

### Established Pattern — Follow Peacekeeper Pilot

The Peacekeeper at `artifacts/developer/src/game/entities/factions/gdo/units/peacekeeper.rs` (359 lines) is the canonical example. Key patterns:

1. **Marker struct**: `pub struct DeploymentCenter;` (no fields)
2. **Implement `EntityDefinition` trait** (`entities/types.rs:81-88`):
   - `const OBJECT: ObjectEnum` — the enum variant
   - `fn object_type() -> ObjectType` — name, size, destructible, sight_range, groupable
   - `fn base_stats() -> BaseStats` — max_hp (as u32), point_armor, full_armor, faction
   - `fn cost() -> Option<StructureCost>` — construction cost if applicable
   - `fn command_slots() -> &'static [CommandSlotDef]` — grid layout entries
   - `fn spawn(commands: &mut EntityCommands, params: SpawnParams)` — insert ECS components
3. **Implement `StructureDefinition` trait** (`entities/types.rs:98-100`):
   - `fn structure_stats() -> StructureStats` — power_value, build_radius, symmetry

### Key Types (all in `entities/types.rs`)

- `BaseStats { max_hp: u32, point_armor: u32, full_armor: u32, faction: FactionEnum }`
- `StructureStats { power_value: i32, build_radius: u32, symmetry: SymmetryTypeEnum }`
- `CommandSlotDef { state: ObjectInterfaceState, row: u8, col: u8, action: CommandButtonAction }`
- `SpawnParams { position: Vec3, owner: Owner, player: u8, rotation: Option<StructureRotation>, under_construction: bool }`

### Registry Registration

In `artifacts/developer/src/game/entities/registry.rs`:
- Currently registers only `Peacekeeper` via `registry.register_unit::<Peacekeeper>()`
- Add `registry.register_structure::<DeploymentCenter>()` etc. for all 6 structures
- Import each struct from its module
- Update `test_build_registry_has_peacekeeper` test — count will change from 1 to 7

### Source Data Locations — Per Structure

**Deployment Center** (`ObjectEnum::DeploymentCenter`):
- ObjectType: `objects.rs:240` — size (4,4), sight_range 6, groupable false
- Stats: `structures.rs:392-396` — DC_MAX_HP=1000, DC_POINT_ARMOR=1, DC_FULL_ARMOR=16, DC_BUILD_RADIUS=12, DC_POWER=20
- Symmetry: `objects.rs:367` — AAAA
- Cost: None (it IS the primary builder; not itself built by DC)
- Command slots: `grid_layout.rs:45-67` — DcIdle, DcBuildMenu, DcConstructing, DcReadyToPlace states
- Spawn: `utils.rs:155-192` — components: ObjectInstance, StructureInstance::default(), Owner, Selectable, SelectionBounds, GridPosition, PowerValue, BuildRadiusExtension, DeploymentCenterState::default(), SightRange

**Power Plant** (`ObjectEnum::PowerPlant`):
- ObjectType: `objects.rs:247` — size (2,2), sight_range 3, groupable true
- Stats: `structures.rs:398-403` — PP_MAX_HP=350, PP_POINT_ARMOR=1, PP_FULL_ARMOR=4, PP_BUILD_RADIUS=1, PP_POWER=20
- Symmetry: `objects.rs:370` — AAAA
- Cost: `structures.rs:88-91` — 150 crystals, 160 frames (from DC construction_cost)
- Command slots: `grid_layout.rs:132-135` — StructureMenuState::Inert (no commands)
- Spawn: `utils.rs:195-244` — includes rotation/flip params, StructureInstance { rotation, flip_horizontal, flip_vertical }

**Barracks** (`ObjectEnum::Barracks`):
- ObjectType: `objects.rs:254` — size (3,2), sight_range 4, groupable true
- Stats: `structures.rs:406-410` — BK_MAX_HP=300, BK_POINT_ARMOR=1, BK_FULL_ARMOR=6, BK_BUILD_RADIUS=2, BK_POWER=-30
- Symmetry: `objects.rs:374` — ABAC
- Cost: `structures.rs:92-95` — 200 crystals, 160 frames (from DC construction_cost)
- Command slots: `grid_layout.rs:69-74` — BarracksMenu: Train Peacekeeper (0,0), Cancel (2,1 conditional), SetRallyPoint (2,2)
- Spawn: `utils.rs:248-299` — includes rotation/flip, BarracksState::default()

**Extraction Facility** (`ObjectEnum::ExtractionFacility`):
- ObjectType: `objects.rs:261` — size (3,3), sight_range 3, groupable false
- Stats: `structures.rs:412-417` — EF_MAX_HP=500, EF_POINT_ARMOR=1, EF_FULL_ARMOR=9, EF_BUILD_RADIUS=2, EF_POWER=-15
- Symmetry: `objects.rs:378` — AAAA
- Cost: `structures.rs:100-103` — 200 crystals, 320 frames (from DC construction_cost)
- Command slots: `grid_layout.rs:75-80` — EfIdle: EnterPlacement/EfBuildPlate (0,0 conditional), EfCancel (2,1 conditional)
- Spawn: `utils.rs:302-354` — includes rotation/flip, ExtractionFacilityState::default()

**Extraction Plate** (`ObjectEnum::ExtractionPlate`):
- ObjectType: `objects.rs:268` — size (1,1), sight_range 0, groupable true
- Stats: `structures.rs:419-424` — EP_MAX_HP=85, EP_POINT_ARMOR=2, EP_FULL_ARMOR=2, EP_BUILD_RADIUS=0, EP_POWER=-3
- Symmetry: `objects.rs:382` — AAAA
- Cost: `structures.rs:203-207` — 75 crystals, 96 frames (from ExtractionFacilityState::construction_cost(), NOT DC)
- Command slots: `grid_layout.rs:132-135` — StructureMenuState::Inert (no commands)
- Spawn: `utils.rs:357-396` — no rotation/flip, has `attached_patch: Entity` extra param, ExtractionPlateState { attached_patch, mining_timer: 0.0 }
- **Note**: spawn takes `attached_patch` entity — SpawnParams doesn't have this field. The spawn impl should handle this limitation (e.g., default to Entity::PLACEHOLDER or skip the ExtractionPlateState insertion when no patch is provided, OR add an extension field)

**Supply Tower** (`ObjectEnum::SupplyTower`):
- ObjectType: `objects.rs:275` — size (3,3), sight_range 4, groupable false
- Stats: `structures.rs:427-433` — ST_MAX_HP=400, ST_POINT_ARMOR=1, ST_FULL_ARMOR=9, ST_BUILD_RADIUS=1, ST_POWER=-15
- Symmetry: `objects.rs:386` — AAAA
- Cost: `structures.rs:96-99` — 200 crystals, 240 frames (from DC construction_cost)
- Command slots: `grid_layout.rs:81-87` — SupplyTowerMenu: Train SupplyChopper (0,0), ScheduleDeliveries (1,0), Cancel (2,1 conditional), SetRallyPoint (2,2)
- Spawn: `utils.rs:823-875` — includes rotation/flip, SupplyTowerState::default()

### Spawn Function Adaptation

The existing spawn_*() functions use `&mut Commands` (world-level) plus `meshes`/`materials` for visual components. The `EntityDefinition::spawn()` trait takes `&mut EntityCommands` (entity-level) and `SpawnParams`. In the spawn impl:
- Insert gameplay components (ObjectInstance, StructureInstance, Owner, GridPosition, PowerValue, BuildRadiusExtension, state component, Selectable, SelectionBounds, SightRange)
- Do NOT insert visual components (Mesh3d, MeshMaterial3d) — these require asset handles not available through SpawnParams. Follow Peacekeeper's pattern (peacekeeper.rs:128-172).
- Use `params.rotation.unwrap_or_default()` for structures that support rotation
- Calculate world position from `params.position` (which should already be world coords)

### Component Imports Needed

From `crate::game::types::structures`:
- `PowerValue`, `BuildRadiusExtension`, `StructureCost`
- `DeploymentCenterState`, `BarracksState`, `ExtractionFacilityState`, `ExtractionPlateState`, `SupplyTowerState`

From `crate::shared::types`:
- `ObjectInstance`, `Selectable`, `SelectionBounds`, `GridPosition`, `SightRange`, `StructureInstance`

From `crate::types`:
- `ObjectEnum`, `FactionEnum`, `SymmetryTypeEnum`, `StructureRotation`, `Owner`

From `crate::ui::types`:
- `ObjectInterfaceState`, `StructureMenuState`, `CommandButtonAction`

### Command Slots Notes

- Many GDO structure command slots depend on dynamic state (e.g., `bk_has_queue`, `has_active_construction`, `has_ready_plate`). The current `CommandSlotDef` type has a TODO comment about adding `condition: Option<SlotCondition>` (types.rs:51). For now, include ALL slots for each state and note the conditional ones with comments.
- Structures with multiple interface states (DC has 4 states, EF has 1, Barracks has 1) need command slots for EACH state.
- Inert structures (PowerPlant, ExtractionPlate) have no command slots — return `&[]`.

### Test Pattern

Follow Peacekeeper test pattern (peacekeeper.rs:214-358):
- `{name}_base_stats()` — verify all stat values
- `{name}_object_type()` — verify name, size, destructible, sight_range, groupable
- `{name}_structure_stats()` — verify power_value, build_radius, symmetry
- `{name}_cost()` — verify construction cost values (or None for DC)
- `{name}_command_slots_count()` — verify slot count
- `{name}_command_slots_{action}()` — verify specific slot positions
- `{name}_registry_lookup()` — verify registration works
- `{name}_object_enum()` — verify OBJECT constant

## Dependencies

- **entity-registry-grid-layout** (pending task): Defines how command slots integrate with the grid layout system. However, the slot definitions themselves can be written now using the existing `CommandSlotDef` type — the integration is a separate concern.
- **entity-registry-spawn-dispatch** (pending task): Defines how registry spawn functions are called at runtime. The spawn implementations can be written now; dispatch wiring is separate.
- **Peacekeeper pilot definition** (already done): The `peacekeeper.rs` file exists and establishes the pattern. This task follows that pattern for structures using `StructureDefinition` instead of `UnitDefinition`.
- **Entity definition foundation** (already done): `entities/mod.rs`, `entities/types.rs`, `entities/registry.rs` are all implemented with the trait hierarchy and registry.
