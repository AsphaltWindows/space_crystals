# entity-definition-cleanup

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Remove all emptied legacy code that has been superseded by the EntityRegistry and per-entity definition files. This is the final cleanup task after all entities are migrated and all consumers are wired to the registry.

**Code to remove:**

1. **utils.rs** — Remove all per-entity spawn functions (spawn_peacekeeper, spawn_syndicate_agent, spawn_tunnel, etc.) and their thin wrappers. The file should only retain non-entity utility functions (if any).

2. **unit_data.rs** — Remove all per-unit type_data() and attack_data() functions (peacekeeper_type_data, agent_type_data, guard_type_data, etc.) and per-unit constants that have been moved to entity files. Remove thin wrappers.

3. **objects.rs** — Remove per-entity ObjectType/StructureType construction code that is now in entity files. Keep the ObjectEnum enum definition itself and any shared type definitions.

4. **structures.rs** — Remove the gdo_structure_stats, syndicate_structure_stats, cults_structure_stats submodules (stat constants moved to entity files). Remove construction_cost() and production_cost() match arms (now in registry). Keep shared types (StructureCost, production state components) that are still used by systems.

5. **grid_layout.rs** — Remove any remaining per-entity match arms (should already be gone from the grid-layout wiring task, but verify and clean up).

6. **Update all remaining import paths** throughout the codebase that still reference the old locations. Tests that imported peacekeeper_type_data() etc. should import from the entity definition files instead.

**Verification:**
- cargo check passes — no dead code warnings for removed items
- cargo test --lib passes — all 1,920+ unit tests still pass (with updated imports)
- cargo test --test qa passes — all 318+ integration tests pass
- No remaining references to removed functions anywhere in the codebase
- Files are significantly smaller: utils.rs, unit_data.rs, objects.rs, structures.rs should each lose hundreds of lines

**Reference:** Architecture doc Phase 7 (Cleanup)

## Technical Context

### File 1: `artifacts/developer/src/game/utils.rs` (1,397 lines)

**Remove:** All per-entity spawn functions (lines 155-1278): `spawn_deployment_center`, `spawn_power_plant`, `spawn_barracks`, `spawn_extraction_facility`, `spawn_extraction_plate`, `spawn_peacekeeper`, `spawn_syndicate_agent`, `spawn_syndicate_guard`, `spawn_tunnel`, `spawn_tunnel_under_construction`, `spawn_headquarters`, `spawn_supply_tower`, `spawn_supply_chopper`, `spawn_recruitment_center`, `spawn_cults_storage`, `spawn_cults_storage_under_construction`, `spawn_cults_armory`, `spawn_cults_recruit`, `spawn_cults_soldier`, `spawn_cults_gunner`. Also remove the helper `spawn_structure_label()` (line 30) which is only used by those spawn functions, plus test functions at the bottom.

**Keep:** `billboard_label_system()` (line 48 — registered in `game/world/mod.rs:29`) and `spawn_ghost_side_labels()` (line 144 — used in `game/world/faction/placement.rs:95`). These are non-entity utility functions. Also keep their imports.

**Callers to update (each must switch to `EntityRegistry::get(ObjectEnum::X).spawn_fn` or direct entity def `::spawn()`):**
- `shared/testing/harness.rs:18` — `use crate::game::utils::*` (used in `spawn_unit_at_grid` and `spawn_structure_at_grid`)
- `game/world/faction/setup.rs:5-6` — game start spawning
- `game/world/faction/construction.rs:4` — HQ construction completion
- `game/world/faction/production.rs:4-6` — unit production
- `game/world/faction/placement.rs:4-8` — structure placement
- `game/units/systems/core/utility.rs:8` — unit utility spawning
- `game/units/systems/behaviors/construction.rs:14` — tunnel under construction
- Various test files that call these spawn functions locally (resource_gathering.rs, tunnel_interaction.rs)

**Pattern for replacement:** Each entity def file has a `fn spawn(commands: &mut EntityCommands, params: SpawnParams)` method. Use `SpawnParams { position, owner, player, rotation, under_construction }` from `crate::game::entities::types::SpawnParams`. For registry-based dispatch: `let entry = registry.get(&obj); (entry.spawn_fn)(&mut entity_commands, params);`

### File 2: `artifacts/developer/src/game/units/types/unit_data.rs` (693 lines)

**Remove (lines 115-217):** Thin wrapper functions and re-exports:
- `peacekeeper_type_data()`, `peacekeeper_attack_data()` (delegate to `Peacekeeper::base_stats()`/`Peacekeeper::attack_data()`)
- `agent_type_data()`, `agent_attack_data()` (delegate to `SyndicateAgentDef`)
- `guard_type_data()`, `guard_attack_data()` (delegate to `SyndicateGuardDef`)
- All `pub use` re-exports of constants (PEACEKEEPER_CONTROL_COST, AGENT_MINING_DURATION, etc.)
- Related imports of entity def types (lines 119-154)

**Keep (lines 1-114):** Shared type definitions used throughout: `UnitType` (Component), `UnitTypeData` (struct), `TurretAttributesData` (Component), `AttackAttributesData` (struct), `UnitMeshType`, `UnitControlCost` (Component), `RuggedTerrainDefenseBonus` (Component), `TunnelSpaceCost` (Component), `OriginatingCenters` (Component).

**Keep:** `frames_to_seconds()` (line 215) — shared utility, not per-entity.

**Callers to update:**
- `ui/command_panel/tunnel_actions.rs:253,285-286` — uses `agent_type_data().unit_base`, `guard_type_data().unit_base` → import from entity defs directly
- `ui/command_panel/helpers.rs:2838,2846` — same pattern
- `game/world/faction/production.rs:68,241-242` — uses `peacekeeper_type_data().unit_base`, `agent_type_data().unit_base`, `guard_type_data().unit_base` → use entity def `::unit_stats().unit_base`
- Test module (lines 219-693): tests for the wrapper functions should be removed. Tests for shared types (TurretAttributesData validation etc.) should stay. Tests referencing `peacekeeper_type_data()` etc. should import from the entity defs.

### File 3: `artifacts/developer/src/game/types/objects.rs` (1,312 lines)

**Remove:** The `ObjectEnum::object_type()` method (lines 215-360) — the giant match returning `ObjectType { ... }` for each variant. This is replaced by `EntityRegistry::get(obj).object_type`.

**Keep:** `ObjectType` struct (line 14), `ObjectInstance` struct/impl (lines 54-120), `StructureLabel` marker (line 123), `StructureInstance` struct/impl (lines 127-213), `InfoPanel` trait (line 7), all shared type definitions.

**Callers of `object_type()` to update (~30 call sites):**
- `ui/hud.rs` — 10 calls like `obj_instance.object_type.object_type()`
- `ui/command_panel/tunnel_actions.rs` — 4 calls like `expansion.object_type().name`
- `ui/command_panel/ui_rebuild.rs` — 5 calls
- `ui/command_panel/helpers.rs` — 4 calls
- `shared/types.rs:866` — test call
- `shared/testing/harness.rs:288` — groupable check
- `game/utils.rs` — sight_range lookups (these calls will be gone with the file cleanup)
- `game/units/types/unit_data.rs` — test calls (will be gone)

**Replacement pattern:** Systems that need object_type data at runtime should accept `Res<EntityRegistry>` and call `registry.get(&obj_enum).map(|e| &e.object_type)`. For non-system code (e.g., tests), use `build_registry()` from `crate::game::entities::registry`.

**Note:** The `entity-registry-spawn-dispatch` task may already wire some of these callers. Verify what remains after that task completes.

### File 4: `artifacts/developer/src/game/types/structures.rs` (2,312 lines)

**Remove:**
- `gdo_structure_stats` module (lines 390-439) — constants now in entity def files (`game/entities/factions/gdo/buildings/base.rs`, `military.rs`, `extraction.rs`)
- `syndicate_structure_stats` module (lines 442-465) — constants now in `game/entities/factions/syndicate/buildings/tunnel.rs`, `headquarters.rs`
- `cults_structure_stats` module (lines 468-491) — constants now in `game/entities/factions/cults/buildings/`
- `construction_cost()` and `production_cost()` match arms in `DeploymentCenterState`, `BarracksState`, and any other state structs — replaced by registry cost lookups
- Corresponding test functions that test the removed match arms

**Keep:** All component/state types that are used as ECS components: `ConstructionHP`, `PowerValue`, `BuildRadiusExtension`, `StructureCost`, `RallyTarget`, `TunnelState`, `TunnelTier`, `TunnelArea`, `TunnelExpansionMarker`, `HeadquartersState`, `DeploymentCenterState`, `BarracksState`, `ExtractionFacilityState`, `ExtractionPlateState`, `SupplyTowerState`, `SupplyChopperState`, `CultsConstructionState`, `ArmoryState`. Also keep `tunnel_construction_cost()` (line 831) if it has game logic beyond static data.

**Callers of stat constants to update (~50 references across codebase):**
- `game/world/faction/construction.rs:135,173-174` — syndicate build frames
- `game/world/faction/production.rs:463-464,824,858,864,876,882` — cults training frames/costs
- `game/world/faction/placement.rs:442` — HQ cost
- `game/world/faction/resources.rs:375` — power constants
- `game/world/faction/ejection.rs:278,436` — RC max HP
- `ui/command_panel/tunnel_actions.rs:213` — HQ cost
- `ui/command_panel/ui_rebuild.rs:228,249,256-257` — armory capacity, build frames
- `ui/command_panel/helpers.rs:358` — HQ cost
- `ui/command_panel/actions.rs:8` — training costs
- `game/units/systems/behaviors/tunnel_interaction.rs:11` — armory capacity
- `game/units/systems/core/command_dispatch.rs:5` — armory capacity
- Entity def files that import `*_structure_stats::*` (these may already import the local constant — verify)

**Replacement pattern:** Import constants directly from entity def files, e.g.: `use crate::game::entities::factions::syndicate::buildings::tunnel::{TUNNEL_CONSTRUCTION_FRAMES, ...}` or `use crate::game::entities::factions::cults::buildings::armory::{SOLDIER_TRAINING_COST, ...}`

### File 5: `artifacts/developer/src/ui/command_panel/grid_layout.rs` (187 lines)

**Verify and clean:** The `entity-registry-grid-layout` task should have already replaced the hardcoded match arms in `get_grid_slot_action()` with registry-driven lookups. If any per-entity match arms remain, remove them. If the function now delegates to the registry, ensure it works correctly.

**Current state (pre-wiring):** `get_grid_slot_action()` has ~185 lines of hardcoded match arms per interface state. After the grid-layout wiring task, this should be substantially smaller or completely registry-driven.

### Cross-cutting concerns

**Import path updates:** After removing the thin wrappers, update all `use crate::game::units::types::unit_data::{peacekeeper_type_data, ...}` imports to point to the entity def files. Similarly, update `use crate::game::types::gdo_structure_stats::*` to `use crate::game::entities::factions::gdo::buildings::base::*` (or specific constants).

**Test migration:** Many tests in unit_data.rs (lines 219-693), objects.rs, and structures.rs test the removed functions. Tests that verify stat VALUES should be migrated to entity def test modules. Tests that verify shared TYPE BEHAVIOR (e.g., TurretAttributesData validation, StructureInstance orientation) stay.

**Dead code warnings:** After removing items, run `cargo check` to find any cascading dead code. The `#![allow(dead_code)]` at the top of several files (utils.rs, structures.rs, objects.rs, unit_data.rs) may mask warnings — consider removing those attributes temporarily to surface issues.

### Architecture reference

See `artifacts/software_architect/architecture/declarative-entity-definitions.md`, Phase 7 (Cleanup, line 211).

## Dependencies

- **entity-registry-grid-layout** (planned_task, pending for developer): Must complete first — it wires grid_layout.rs to use registry-based command slot lookups. Without it, removing hardcoded match arms from grid_layout.rs would break the command panel.
- **entity-registry-spawn-dispatch** (planned_task, pending for developer): Must complete first — it wires all spawn call sites (setup, production, placement, construction) to use registry spawn functions. Without it, removing spawn functions from utils.rs would break spawning.
- **entity-registry-costs** (planned_task, active with developer): Must complete first — it wires cost lookups (construction_cost, production_cost) to use registry. Without it, removing cost match arms from structures.rs would break cost checks.
- All 9 entity migration tasks (foundation + pilot + 7 faction/neutral migrations): All complete. Entity def files with spawn functions and stats are in place.
