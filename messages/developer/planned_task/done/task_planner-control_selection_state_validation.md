# control-selection-state-validation

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-control-state-selection.md

## Task

Implement ObjectInterfaceState reset and validation tied to selection changes.

### 1. Reset ObjectInterfaceState on Selection/ActiveGroup Change

Per the design doc: "Reset to the default state when the Selection or ActiveGroup changes."

Add a system (e.g., `interface_state_selection_reset_system`) that:
- Tracks the previous Selection state (active_group_index + group types) using a Local or dedicated resource
- Each tick, compares current Selection to previous
- If Selection groups changed OR active_group_index changed: reset `ObjectInterfaceState` to `ObjectInterfaceState::Default`
- Update the tracking state
- Register in the appropriate system set, after `selection_group_sync_system` and `active_group_cycle_system`

### 2. ObjectInterfaceState Tick Validation

Per the design doc: "Each tick, the ObjectInterfaceState is validated against the active SelectionGroup's game state — if the current state is no longer valid, it resets to the default state."

Add a system (e.g., `interface_state_validation_system`) that:
- Reads current `ObjectInterfaceState` and `Selection`
- Checks if the current state is still valid for the active group. Examples:
  - `StructureMenu(DcIdle)` is invalid if no DeploymentCenter is in the active group
  - `StructureMenu(BarracksMenu)` is invalid if no Barracks is in the active group
  - `StructureMenu(DcConstructing)` is invalid if the DC no longer has an active construction
  - `AgentMenu(*)` is invalid if no SyndicateAgent is in the active group
  - `AwaitingTarget(*)` is invalid if the active group has no entities
- On invalid state: reset to `ObjectInterfaceState::Default`
- Register after the selection reset system

Both systems go in `game/world/resources.rs` or `ui/` depending on existing conventions. The existing `selection_validation_system` in `game/world/resources.rs` is a good reference for the pattern.

## Technical Context

### Files to modify

1. **`artifacts/developer/src/game/world/resources.rs`** — Add both new systems here. This file already contains `selection_validation_system` (line 848), `selection_group_sync_system` (line 779), and `active_group_cycle_system` (line 811). The module is `pub(crate)` scoped and all selection-related systems live here.

2. **`artifacts/developer/src/game/world/mod.rs`** — Register both new systems in the `FactionPlugin::build()` method (lines 73-91), in the `Update` schedule under `DiagCategory::Faction`. They must be ordered after existing selection systems.

### Key types and imports needed

- **`ObjectInterfaceState`** (resource): `crate::ui::types::ObjectInterfaceState` — already imported in resources.rs line 5. Enum with variants: `Default`, `AwaitingTarget(CommandType)`, `StructureMenu(StructureMenuState)`, `AgentMenu(AgentMenuState)`.
- **`Selection`** (resource): `crate::types::Selection` — has `groups: Vec<SelectionGroup>`, `active_group_index: Option<usize>`. Key methods: `active_group()`, `active_type()`.
- **`SelectionGroup`**: `crate::types::SelectionGroup` — has `object_type: ObjectEnum`, `entities: Vec<Entity>`.
- **`ObjectEnum`**: `crate::types::ObjectEnum` — distinguishes `SyndicateAgent`, `DeploymentCenter`, `Barracks`, `Tunnel`, etc. Has `.is_structure()` method.
- **Structure state components**: `DeploymentCenterState`, `BarracksState`, `ExtractionFacilityState`, `SupplyTowerState`, `HeadquartersState`, `TunnelState` — all in `crate::game::types::structures` (imported via `super::types::*`).
- **`StructureMenuState`**, **`AgentMenuState`**: `crate::ui::types::{StructureMenuState, AgentMenuState}` — need to add these imports in resources.rs (currently only `ObjectInterfaceState` is imported from ui::types on line 5).

### System 1: `interface_state_selection_reset_system`

**Pattern**: Use `Local<Option<(Option<usize>, Vec<ObjectEnum>)>>` to track previous state. Each tick:
1. Extract current snapshot: `(selection.active_group_index, selection.groups.iter().map(|g| g.object_type).collect::<Vec<_>>())`
2. Compare to `Local` cached value
3. If different: set `*interface_state = ObjectInterfaceState::Default`
4. Update cached value

**Signature**:
```rust
pub fn interface_state_selection_reset_system(
    selection: Res<Selection>,
    mut interface_state: ResMut<ObjectInterfaceState>,
    mut prev_state: Local<Option<(Option<usize>, Vec<ObjectEnum>)>>,
)
```

**Note**: Comparing group types (not entities) ensures reset triggers on group composition changes without being noisy when only entity health changes, etc.

### System 2: `interface_state_validation_system`

**Pattern**: Match on current `ObjectInterfaceState` and verify the active group supports it.

**Signature**:
```rust
pub fn interface_state_validation_system(
    selection: Res<Selection>,
    mut interface_state: ResMut<ObjectInterfaceState>,
    dc_query: Query<&DeploymentCenterState>,
    bk_query: Query<&BarracksState>,
    ef_query: Query<&ExtractionFacilityState>,
    st_query: Query<&SupplyTowerState>,
    hq_query: Query<&HeadquartersState>,
    tunnel_query: Query<&TunnelState>,
)
```

**Validation logic**:
- `StructureMenu(DcIdle | DcBuildMenu | DcReadyToPlace | DcAwaitingPlacement)` → active group must be `ObjectEnum::DeploymentCenter` AND entities must have `DeploymentCenterState`
- `StructureMenu(DcConstructing)` → same, plus the DC must have an active construction (check `dc_state.construction` is `Some`)
- `StructureMenu(BarracksMenu)` → active group is `ObjectEnum::Barracks` with `BarracksState`
- `StructureMenu(EfIdle | EfConstructing | EfReadyToPlace | EfAwaitingPlacement)` → active group is `ObjectEnum::ExtractionFacility` with `ExtractionFacilityState`
- `StructureMenu(SupplyTowerMenu)` → active group is `ObjectEnum::SupplyTower` with `SupplyTowerState`
- `StructureMenu(HeadquartersMenu)` → active group is `ObjectEnum::Headquarters` with `HeadquartersState`
- `StructureMenu(TunnelIdle | TunnelExpandMenu | TunnelEjectMenu | TunnelAwaitingPlacement)` → active group is `ObjectEnum::Tunnel` with `TunnelState`
- `StructureMenu(Inert)` → active group must be a structure type
- `AgentMenu(_)` → active group is `ObjectEnum::SyndicateAgent`
- `AwaitingTarget(_)` → active group must have at least one entity
- `Default` → always valid (no check needed)

### System ordering (in mod.rs FactionPlugin)

```rust
resources::interface_state_selection_reset_system
    .after(resources::selection_group_sync_system)
    .after(resources::active_group_cycle_system),
resources::interface_state_validation_system
    .after(resources::interface_state_selection_reset_system),
```

These must run BEFORE `update_command_panel_state` (which is in `DiagCategory::UiHud` in ui/mod.rs). Since `DiagCategory::Faction` and `DiagCategory::UiHud` are separate system sets in `Update`, Bevy does NOT guarantee ordering between them unless explicit `.before()`/`.after()` constraints are added. However, `update_command_panel_state` already handles forcing correct structure menus based on active type, so the two layers are complementary — the reset system provides a clean slate, and `update_command_panel_state` then sets the correct structure menu if needed.

### Interaction with `update_command_panel_state`

`update_command_panel_state` (command_panel.rs:281) already forces interface state transitions when the active group type changes (e.g., switching to a structure auto-sets DcIdle/BarracksMenu/etc.). The new reset system resets to `Default` on selection changes, and then `update_command_panel_state` will naturally transition to the correct structure menu state in the same frame or next frame. This is the intended layered design — reset provides correctness, `update_command_panel_state` provides convenience.

### Test patterns

Follow existing test patterns in `resources.rs` line 890+:
- Use `setup_test_world()` helper (line 899) which creates a `World` with `Selection::default()`
- Also insert `ObjectInterfaceState` as a resource in the test world
- Use `world.run_system_once(system_name)` for system tests
- Test cases:
  - Selection change resets AwaitingTarget to Default
  - Active group index change resets to Default
  - No change = no reset
  - Invalid StructureMenu state resets to Default
  - Valid state is preserved
  - AgentMenu with non-agent active group resets

## Dependencies

- **`selection_group_sync_system`** (existing, `game/world/resources.rs:779`): Must run before the reset system, because the reset system reads `Selection` which is populated by sync.
- **`active_group_cycle_system`** (existing, `game/world/resources.rs:811`): Must run before the reset system, because Tab cycling changes `active_group_index`.
- **`selection_validation_system`** (existing, `game/world/resources.rs:848`): Must run before sync (already ordered), so dead entities are removed before sync rebuilds groups.
- **`update_command_panel_state`** (existing, `ui/command_panel.rs:281`): Complementary — handles forcing correct structure menus. Runs in a different DiagCategory set but is not dependent on the new systems' output in the same frame.
