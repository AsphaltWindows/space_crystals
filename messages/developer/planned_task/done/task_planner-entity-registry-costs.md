# entity-registry-costs

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Replace hardcoded production/construction cost lookups with EntityRegistry-based cost lookups.

**Current state:** Costs are hardcoded in match arms:
- `DeploymentCenterState::construction_cost(object)` matches ObjectEnum variants to StructureCost values
- `BarracksState::production_cost(object)` matches ObjectEnum variants to StructureCost values
- `HeadquartersState::production_cost(object)` matches ObjectEnum variants to StructureCost values
- Similar patterns in other production building states (CultsArmory, RecruitmentCenter, SupplyTower)

**After this task:** Cost lookups go through the registry:
1. Add a `cost(&self, object: &ObjectEnum) -> Option<StructureCost>` convenience method to EntityRegistry
2. Replace `construction_cost()` and `production_cost()` match arms with registry.cost() calls
3. Update the production queue systems (`try_queue`, construction start) to use registry cost lookups
4. Update cancellation refund logic to use registry costs

**Callers to update:**
- DeploymentCenterState construction_cost() and cancellation_refund()
- BarracksState production_cost()
- HeadquartersState production_cost()
- Any other production building cost lookups
- UI display of costs (if cost is shown in command panel tooltips or HUD)

**Tests:**
- Registry cost lookup returns identical values to current match-based lookups for all producible entities
- Registry returns None for non-producible entities (resources, starting structures)
- Production queue try_queue still correctly deducts costs
- Cancellation refund returns correct values
- All existing production/construction tests pass

**Reference:** Architecture doc Phase 6 (Wire Registry Consumers)

## Technical Context

### EntityRegistry already stores costs

The `EntityRegistry` resource (`src/game/entities/types.rs:116`) already has `RegistryEntry.cost: Option<StructureCost>` (line 108). Every entity definition file already implements `fn cost() -> Option<StructureCost>` via the `EntityDefinition` trait (line 85). The registry is populated at startup via `EntitiesPlugin` (`src/game/entities/registry.rs:57`).

**Step 1: Add convenience method to EntityRegistry** (`src/game/entities/types.rs`, after `get()` at line 194):
```rust
pub fn cost(&self, object: &ObjectEnum) -> Option<StructureCost> {
    self.entries.get(object).and_then(|e| e.cost.clone())
}
```
Note: `StructureCost` (`src/game/types/structures.rs:51`) has fields `space_crystals: u32` and `build_frames: u32`. It likely needs `#[derive(Clone)]` if not already present — verify and add.

### Hardcoded cost methods to replace

All in `src/game/types/structures.rs`:

1. **`DeploymentCenterState::construction_cost(object)`** (line 86) — match on PP/BK/ST/EF to StructureCost. Used by:
   - `actions.rs:120` — DcBuild (deduct cost)
   - `actions.rs:139,149` — DcCancel via `cancellation_refund()` (line 110)
   - `helpers.rs:98` — `get_dc_construction_info()` (progress bar)
   - `hud.rs:443` — HUD info panel (progress display)
   - `ui_rebuild.rs` — implicitly via `get_dc_construction_info()`
   - Tests: `structures.rs:1313-1337` (4 tests)

2. **`BarracksState::production_cost(object)`** (line 150) — match on Peacekeeper. Used by:
   - `actions.rs:183` — BkTrain (deduct cost)
   - `actions.rs:217` — BkCancel (refund)
   - `hud.rs:460` — HUD info panel
   - `ui_rebuild.rs:156` — progress bar
   - Tests: `structures.rs:1366-1375` (2 tests)

3. **`HeadquartersState::production_cost(object)`** (line 272) — match on SyndicateAgent/SyndicateGuard. Used by:
   - `actions.rs:236` — HqTrain (deduct cost)
   - `actions.rs:264` — HqCancel (refund)
   - `helpers.rs:142` — `grid_button_enabled_ext()` (button greying)
   - `ui_rebuild.rs:198` — progress bar
   - Tests: `structures.rs:1831-1840`, `helpers.rs:1976-1986`

4. **`ExtractionFacilityState::construction_cost()`** (line 203) — hardcoded single StructureCost. Used by:
   - `actions.rs:295` — EfBuildPlate (deduct cost)
   - `actions.rs:312` — via `cancellation_refund()` (line 212)
   - `hud.rs:477` — progress display
   - `ui_rebuild.rs:124` — progress bar

5. **`SupplyTowerState::production_cost(object)`** (line 340) — match on SupplyChopper. Used by:
   - `actions.rs:329` — StTrain (deduct cost)
   - `actions.rs:357` — StCancel (refund)
   - `hud.rs:506` — HUD info panel
   - `ui_rebuild.rs:180` — progress bar

6. **Armory training costs** (`actions.rs:379,403`) — uses `SOLDIER_TRAINING_COST`/`GUNNER_TRAINING_COST` constants from `cults_structure_stats` module, not a `production_cost()` method. Replace with `registry.cost(&ObjectEnum::CultsSoldier)` and `registry.cost(&ObjectEnum::CultsGunner)`.

### Approach for replacement

**Option A (recommended): Replace static methods with registry lookups inline.**
- Remove the `construction_cost()`/`production_cost()` methods from the State structs
- At each call site, use `registry.cost(&object_type)` instead
- For `cancellation_refund()`, take a `StructureCost` or the registry as param instead of looking it up internally

**Option B: Delegate to registry inside the static methods.**
- Change signatures to `construction_cost(registry: &EntityRegistry, object)`
- This is messier — requires threading the registry through.

Go with **Option A** since the callers already have access to Bevy system params.

### Threading EntityRegistry to call sites

The main challenge is getting `Res<EntityRegistry>` into the functions:

1. **`execute_command_action()`** (`actions.rs:19`) — add `registry: &Res<EntityRegistry>` parameter. Thread to sub-dispatchers: `execute_dc_action()` and `execute_production_action()`.

2. **Callers of `execute_command_action()`** — both in `hotkeys.rs`:
   - `handle_command_button_clicks()` (line 18) — add `registry: Res<EntityRegistry>` system param
   - `command_panel_hotkeys()` (line 49) — add `registry: Res<EntityRegistry>` system param
   - Pass `&registry` at call sites (lines 44 and 263)

3. **`rebuild_command_panel_ui()`** (`ui_rebuild.rs:16`) — add `registry: Res<EntityRegistry>` system param for progress bar calculations

4. **`update_selected_units_grid_system()`** (`hud.rs`) — add `registry: Res<EntityRegistry>` system param for HUD info panel cost display

5. **`grid_button_enabled_ext()`** (`helpers.rs:108`) — add `registry: &EntityRegistry` parameter; caller is `rebuild_command_panel_ui()` which will now have the resource.

6. **`get_dc_construction_info()`** (`helpers.rs:95`) — add `registry: &EntityRegistry` parameter; used by `ui_rebuild.rs`.

### Import needed

Add to files that use the registry:
```rust
use crate::game::entities::EntityRegistry;
```
(`EntityRegistry` is re-exported from `entities/mod.rs` via `pub use types::*;`)

### ExtractionFacilityState special case

`ExtractionFacilityState::construction_cost()` (line 203) returns a fixed `StructureCost` (no object param — it only builds ExtractionPlates). Replace with `registry.cost(&ObjectEnum::ExtractionPlate)`. The entity definition file at `src/game/entities/factions/gdo/buildings/extraction.rs:134` already defines this cost as `Some(StructureCost { space_crystals: 75, build_frames: 96 })`. The `cancellation_refund()` method (line 212) internally calls `Self::construction_cost()` — update to take registry or StructureCost param.

### Verifying cost parity

All costs in entity definition files already match the hardcoded values:
- PowerPlant: 150 SC, 160 frames (base.rs:144-146 == structures.rs:88-91)
- Barracks: 200 SC, 160 frames (military.rs:58-60 == structures.rs:92-95)
- SupplyTower: 200 SC, 240 frames (military.rs:144-146 == structures.rs:96-99)
- ExtractionFacility: 200 SC, 320 frames (extraction.rs:58-60 == structures.rs:100-103)
- ExtractionPlate: 75 SC, 96 frames (extraction.rs:136-138 == structures.rs:204-207)
- Peacekeeper: 50 SC, 80 frames (peacekeeper.rs:92-94 == structures.rs:152-155)
- SyndicateAgent: 100 SC, 160 frames (agent.rs:119-121 == structures.rs:274-277)
- SyndicateGuard: 125 SC, 120 frames (guard.rs:95-97 == structures.rs:278-281)
- SupplyChopper: 100 SC, 160 frames (supply_chopper.rs:121-123 == structures.rs:342-345)

### Tests to update

**structures.rs tests** (~lines 1310-1840):
- `dc_construction_cost_power_plant`, `dc_construction_cost_barracks`, etc. — rewrite to test `registry.cost()` using `build_registry()`
- `bk_production_cost_peacekeeper`, `bk_production_cost_invalid_returns_none` — same
- `dc_full_refund_during_construction`, `dc_partial_refund_when_ready_to_place` — update to pass registry/cost
- `headquarters_production_cost_agent`, etc.

**helpers.rs tests** (~lines 1828-1986):
- `supply_tower_construction_cost_is_200` — uses `DeploymentCenterState::construction_cost()`
- `headquarters_state_production_cost` — uses `HeadquartersState::production_cost()`

**Pattern for new tests** — use `build_registry()` from `src/game/entities/registry.rs:24`:
```rust
use crate::game::entities::registry::build_registry;
let registry = build_registry();
let cost = registry.cost(&ObjectEnum::PowerPlant).unwrap();
assert_eq!(cost.space_crystals, 150);
```

### Files to modify (summary)

| File | Changes |
|------|---------|
| `src/game/entities/types.rs` | Add `EntityRegistry::cost()` convenience method |
| `src/game/types/structures.rs` | Remove 5 cost methods, update `cancellation_refund()` signatures, update tests |
| `src/ui/command_panel/actions.rs` | Add registry param to `execute_command_action()`, `execute_dc_action()`, `execute_production_action()`; replace all cost lookups |
| `src/ui/command_panel/hotkeys.rs` | Add `Res<EntityRegistry>` to both system signatures; pass to `execute_command_action()` |
| `src/ui/command_panel/helpers.rs` | Add registry param to `get_dc_construction_info()` and `grid_button_enabled_ext()`; update tests |
| `src/ui/command_panel/ui_rebuild.rs` | Add `Res<EntityRegistry>` system param; replace all cost lookups |
| `src/ui/hud.rs` | Add `Res<EntityRegistry>` to `update_selected_units_grid_system`; replace cost lookups |

## Dependencies

- **All entity definition migration tasks must be complete** — each entity`s `fn cost()` must return the correct value in its definition file. This is a prerequisite for the costs being present in the registry. Check: `build_registry()` in `registry.rs` registers all 20 entities.
- **No dependency on grid-layout wiring task** — this task only touches cost lookups, not command slot lookups.
- **No dependency on spawn-function wiring task** — orthogonal registry consumer.
