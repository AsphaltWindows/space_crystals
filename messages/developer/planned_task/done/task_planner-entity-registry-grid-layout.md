# entity-registry-grid-layout

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Replace the hardcoded command panel grid layout match in grid_layout.rs with EntityRegistry lookups.

**Current state:** `get_grid_slot_action()` in `ui/command_panel/grid_layout.rs` is a large match on ObjectInterfaceState mapping (row, col) to CommandButtonAction. Each entity's slots are hardcoded as match arms.

**After this task:** `get_grid_slot_action()` queries the EntityRegistry resource to find command_slots for the given entity/state, then looks up the (row, col) in those slots. The match arms are removed.

**Implementation:**
1. Modify `get_grid_slot_action()` to accept `&EntityRegistry` (or access it as a Bevy resource)
2. Look up the entity's RegistryEntry via ObjectEnum
3. Search its command_slots for matching (state, row, col) -> return the action
4. Remove all per-entity match arms from grid_layout.rs
5. Update all callers to pass the registry

**This depends on all entity migrations being complete** (all entities must have their command_slots populated in the registry before the match arms can be removed).

**Tests:**
- For every ObjectInterfaceState and every (row, col) position: registry-based lookup returns identical CommandButtonAction as the old match-based implementation
- No regression in command panel behavior
- grid_layout.rs is significantly smaller after this change

**Reference:** Architecture doc Phase 6 (Wire Registry Consumers)

## Technical Context

### Primary File to Change

- **`artifacts/developer/src/ui/command_panel/grid_layout.rs`** (lines 33-187): The `get_grid_slot_action()` function. Currently a 155-line match statement mapping `ObjectInterfaceState` + `(row, col)` to `Option<CommandButtonAction>`. This entire match body will be replaced with a registry lookup.

### Key Types & Data Structures

- **`EntityRegistry`** (`src/game/entities/types.rs:116`): `Resource` with `HashMap<ObjectEnum, RegistryEntry>`. Method `get(&ObjectEnum) -> Option<&RegistryEntry>`.
- **`RegistryEntry`** (`src/game/entities/types.rs:105`): Contains `command_slots: &'static [CommandSlotDef]`.
- **`CommandSlotDef`** (`src/game/entities/types.rs:44`): `{ state: ObjectInterfaceState, row: u8, col: u8, action: CommandButtonAction }`. Has a TODO for `condition: Option<SlotCondition>` (Phase 2) but currently no condition field.
- **`ObjectInterfaceState`** (`src/ui/types.rs:151`): Enum with variants `Default`, `AwaitingTarget(CommandType)`, `StructureMenu(StructureMenuState)`, `AgentMenu(AgentMenuState)`, `CultsRecruitMenu(CultsRecruitMenuState)`.
- **`Selection::active_type()`** (`src/shared/types.rs:150`): Returns `Option<ObjectEnum>` — the way to determine which entity's registry entry to look up.
- **`SelectedUnitCapabilities`** (`src/ui/types.rs`): Has `has_attack`, `can_reverse`, `can_target_ground`, `is_chopper` flags used by conditional slots.

### Callers to Update (3 sites)

1. **`hotkeys.rs:262`** — `command_panel_hotkeys()` system calls `get_grid_slot_action()` with 8 params. Must add `Res<EntityRegistry>` to the system's params and pass it.
2. **`ui_rebuild.rs:396`** — `rebuild_command_panel_ui()` system calls it in a loop for all 12 grid cells. Must add `Res<EntityRegistry>` to system params.
3. **`helpers.rs:553+`** — Test module calls it directly in ~20 tests (lines 689-785). Tests will need a `build_registry()` call or a test helper to create a registry.

### The Conditional Slot Problem

The current match has 11 conditional guards (e.g., `if bk_has_queue`, `if has_active_construction`, `if caps.has_attack`). The `CommandSlotDef` struct does NOT yet have a condition field (there's a TODO at line 51 of types.rs). There are two approaches:

**Option A (Recommended): Add SlotCondition enum now.** Add a `condition: Option<SlotCondition>` field to `CommandSlotDef`. Define `SlotCondition` enum with variants like `HasQueue`, `HasActiveConstruction`, `TunnelIsUpgrading`, `HasReadyPlate`, `HasAttack`, `CanReverse`, `CanTargetGround`. The lookup function evaluates the condition against the runtime context. This makes the entity definitions fully self-contained.

**Option B: Keep conditionals in the lookup function.** The registry lookup finds the slot, then the lookup function applies a separate conditional filter based on the action type. Simpler to implement but the conditional logic stays in grid_layout.rs rather than being data-driven.

Whichever option is chosen, the function signature changes. Currently:
```rust
pub(super) fn get_grid_slot_action(
    state: &ObjectInterfaceState, row: u8, col: u8,
    bk_has_queue: bool, has_active_construction: bool,
    caps: &SelectedUnitCapabilities, tunnel_is_upgrading: bool, has_ready_plate: bool,
) -> Option<CommandButtonAction>
```

New signature adds `registry: &EntityRegistry` and `active_type: Option<ObjectEnum>`.

### Lookup Strategy

The function needs to map `ObjectInterfaceState` → which `ObjectEnum` to look up in the registry. The mapping:
- `StructureMenu(DcIdle|DcBuildMenu|...)` → `ObjectEnum::DeploymentCenter`
- `StructureMenu(BarracksMenu)` → `ObjectEnum::Barracks`
- `StructureMenu(EfIdle|EfAwaitingPlacement)` → `ObjectEnum::ExtractionFacility`
- `StructureMenu(HeadquartersMenu)` → `ObjectEnum::Headquarters`
- `StructureMenu(TunnelIdle|...)` → `ObjectEnum::Tunnel`
- `StructureMenu(SupplyTowerMenu)` → `ObjectEnum::SupplyTower`
- `StructureMenu(RecruitmentCenterMenu)` → `ObjectEnum::RecruitmentCenter`
- `StructureMenu(ArmoryMenu)` → `ObjectEnum::CultsArmory`
- `StructureMenu(Inert)` → Use `active_type` from Selection (PowerPlant or ExtractionPlate)
- `AgentMenu(_)` → `ObjectEnum::SyndicateAgent`
- `CultsRecruitMenu(_)` → `ObjectEnum::CultsRecruit`
- `Default` → Use `active_type` from Selection (Peacekeeper, Guard, SupplyChopper, etc.)
- `AwaitingTarget(_)` → Special case: always `Back` at (2,0), no registry needed OR encode in each entity's slots

For `Default` state, note the special is_chopper guard at line 137 — SupplyChopper uses `Default` state but has different slots than other units. With registry lookup, this is automatic: look up the active_type's command_slots and filter by state=Default.

For `AwaitingTarget`, all entities currently share the same Back button at (2,0). This could be a fallback in the function rather than duplicated in every entity's command_slots.

### Entity Definition Files with command_slots (already implemented)

All registered entities already have command_slots populated:
- `entities/factions/gdo/units/peacekeeper.rs` — 6 slots (Default state: Move, HoldPosition, Attack, Patrol, AttackGround, Stop)
- `entities/factions/gdo/units/supply_chopper.rs` — slots for Default state with chopper-specific commands
- `entities/factions/gdo/buildings/base.rs` — DC_COMMAND_SLOTS (15 slots across DcIdle, DcBuildMenu, DcConstructing, DcReadyToPlace)
- `entities/factions/gdo/buildings/military.rs` — Barracks, SupplyTower slots
- `entities/factions/gdo/buildings/extraction.rs` — EF, EP slots
- `entities/factions/syndicate/units/agent.rs` — Agent slots (AgentMenu states)
- `entities/factions/syndicate/units/guard.rs` — Guard slots (Default state)
- `entities/factions/cults/units/recruit.rs` — Recruit slots (CultsRecruitMenu states)
- `entities/factions/cults/units/soldier.rs`, `gunner.rs` — Soldier/Gunner slots

### NOT Yet Registered (dependencies)

These entities have grid slots in the current match but are NOT yet in the registry (`registry.rs`):
- **Syndicate buildings**: Tunnel, Headquarters — `entities/factions/syndicate/buildings/` has no definition files yet (task `entity-migration-syndicate-buildings` is pending at developer)
- **Cults buildings**: RecruitmentCenter, CultsStorage, CultsArmory — `entities/factions/cults/buildings/` has no definition files yet (task `entity-migration-cults-buildings` is pending at developer)
- **Neutral entities**: SpaceCrystalPatch, SupplyDeliveryStation — `entities/neutral/` has no definition files (task `entity-migration-neutral` is active at developer)

**All three migration tasks must complete before this task can remove match arms.** Once they register with command_slots, the lookup will work automatically.

### Existing Tests to Update

- `helpers.rs` test module (starting ~line 553): ~20 tests call `get_grid_slot_action()` directly with various states and (row, col). These must be updated to pass a registry (use `build_registry()` from `game/entities/registry.rs`).
- `helpers.rs:621` — Conditional tests (`all_caps()` helper) verify capability-gated slots. These need to test the new condition system.
- Add new tests verifying registry lookup matches old match for every entity's command slots (comprehensive equivalence test).

### Pattern to Follow

The existing `build_registry()` function (`src/game/entities/registry.rs:17`) constructs the full registry. Use this in tests:
```rust
use crate::game::entities::registry::build_registry;
let registry = build_registry();
// Then pass &registry to get_grid_slot_action()
```

### Bevy System Integration

Both caller systems (`command_panel_hotkeys` in hotkeys.rs, `rebuild_command_panel_ui` in ui_rebuild.rs) need `Res<EntityRegistry>` added to their system parameter list. Also need `Res<Selection>` to get `active_type()`. The `Selection` resource is already available in both systems (check existing params). The `EntityRegistry` is inserted at startup by `EntitiesPlugin` (`registry.rs:40`), which is added in `main.rs`.

## Dependencies

- **`entity-migration-syndicate-buildings`** (pending at developer) — Must complete first. Tunnel and Headquarters need command_slots registered before match arms can be removed.
- **`entity-migration-cults-buildings`** (pending at developer) — Must complete first. RecruitmentCenter, CultsStorage, CultsArmory need command_slots registered.
- **`entity-migration-neutral`** (active at developer) — Must complete first. Neutral entities (SCP, SDS) have no grid slots, but they should be registered for completeness.
- **`entity-definition-foundation`** (done) — Provides the EntityRegistry, CommandSlotDef, and trait infrastructure this task builds on.
