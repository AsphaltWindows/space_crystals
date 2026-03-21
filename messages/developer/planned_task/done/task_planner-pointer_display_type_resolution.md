# pointer_display_type_resolution

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-pointer_display_types.md

## Task

Define the PointerDisplayType enum and resolution system. This is a new system that determines which cursor display type to show each frame.

### What to implement:

1. **PointerDisplayType enum** in `ui/types.rs`:
   - Variants: Inactive, Move, Attack, AttackGround, Patrol, GatherResources, ReturnResources, Enter
   - Derive Default (Inactive)

2. **PointerDisplayType as a Resource** — add to `ui/types.rs`, init in `ui/mod.rs` plugin setup.

3. **resolve_pointer_display_type system** in `ui/command_panel.rs` (or a new `ui/pointer.rs` module):
   - Runs each frame, reads: ObjectInterfaceState, CursorTarget, CursorTargetEnum, Selection, SelectedUnitCapabilities, ActiveGroup data
   - Sets the PointerDisplayType resource based on resolution rules (DefaultState, AwaitingTarget, placement mode)

4. **Unit tests** verifying resolution rules for each combination.

### Key references:
- Design: `artifacts/designer/design/control_system.md` — PointerDisplayType section
- CursorTarget resource: `ui/types.rs` line 118
- ObjectInterfaceState: `ui/types.rs` line 150
- Right-click resolution logic: `game/units/systems/core.rs` `right_click_move_command`
- SelectedUnitCapabilities: `ui/types.rs` line 369
- AgentCarryState: `game/units/types/state/types.rs`

## Technical Context

### Files to Change

1. **`artifacts/developer/src/ui/types.rs`** (after line 409, end of file):
   - Add the `PointerDisplayType` enum as a Bevy Resource:
   ```rust
   #[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
   pub enum PointerDisplayType {
       #[default]
       Inactive,
       Move,
       Attack,
       AttackGround,
       Patrol,
       GatherResources,
       ReturnResources,
       Enter,
   }
   ```

2. **`artifacts/developer/src/ui/mod.rs`** (line 11 imports, line 18-22 resource init):
   - Add `PointerDisplayType` to the import on line 11: `use types::{..., PointerDisplayType};`
   - Add `.init_resource::<PointerDisplayType>()` alongside the other `.init_resource` calls (after line 22)
   - Register the new system in the Update systems list (line 31-44), ordered **after** `update_command_panel_state`:
     ```rust
     command_panel::resolve_pointer_display_type.after(command_panel::update_command_panel_state),
     ```

3. **`artifacts/developer/src/ui/command_panel.rs`** — Add the `resolve_pointer_display_type` system (recommended location: after `update_command_panel_state`, ~line 430+):
   - This is the main implementation file. The system mirrors the right-click resolution in `core.rs:179-583` but only determines the display type — it does NOT issue commands.

### System Signature

```rust
pub fn resolve_pointer_display_type(
    interface_state: Res<ObjectInterfaceState>,
    cursor_target: Res<CursorTarget>,
    selection: Res<Selection>,
    unit_caps: Res<SelectedUnitCapabilities>,
    mut pointer_display: ResMut<PointerDisplayType>,
    // For DefaultState Enter check: need to query tunnel info
    target_info: Query<(Option<&SpaceCrystalPatch>, Option<&SupplyDeliveryStation>, Option<&TunnelState>, &Owner), With<ObjectInstance>>,
    // For detecting production buildings in selection
    selected_structures: Query<(Option<&BarracksState>, Option<&HeadquartersState>, Option<&SupplyTowerState>, Option<&DeploymentCenterState>, Option<&ExtractionFacilityState>), (With<StructureInstance>, With<Selected>)>,
    local_player: Res<LocalPlayer>,
) {
```

### Resolution Logic (Implementation Guide)

**Placement mode** (`interface_state.is_placement_mode()`): Set `Inactive`, return early.

**DefaultState** (`ObjectInterfaceState::Default` or structure/agent menu states):
- If `selection.total_entity_count() == 0` → `Inactive`
- Check `selection.active_type()` (returns `Option<ObjectEnum>`):
  - If active type is a structure with a production state (BarracksState, HeadquartersState, SupplyTowerState, DeploymentCenterState, ExtractionFacilityState) → `Move` (rally point preview)
  - Iterate the active group's entities through `selected_structures` query to detect this
- If `cursor_target.kind == CursorTargetEnum::EnemyObject` and `unit_caps.has_attack` → `Attack`
- If cursor is over a resource node: query `cursor_target.entity` via `target_info` — if it has `SpaceCrystalPatch` or `SupplyDeliveryStation`, and active type is `SyndicateAgent` or `SupplyChopper` (resource gatherers) → `GatherResources`
- If cursor is over own Tunnel: query entity for `TunnelState` + `Owner`, check `Owner.player_number() == local_player`, and active type is Syndicate unit → `Enter` (simplified — full check would use `can_enter_tunnel()` but that requires per-unit UnitBaseEnum which this system doesn't query; acceptable simplification)
- For drop-off (`ReturnResources`): if agent is carrying (`unit_caps.agent_carrying`) and cursor is over own Tunnel → `ReturnResources`. Similarly if `unit_caps.chopper_has_supplies` and cursor over SupplyTowerState → `ReturnResources`
- If cursor is Ground/FriendlyObject/NeutralObject and selection has movable units → `Move`
- Otherwise → `Inactive`

**AwaitingTarget(cmd)** (`interface_state.awaiting_command_type()` returns `Some(ct)`):
- Match on `ct`:
  - `CommandType::Attack`:
    - `EnemyObject` → `Attack`
    - `Ground` → `Attack` (AttackMove preview)
    - `FriendlyObject | NeutralObject` → `Inactive`
  - `CommandType::Move`: any target → `Move`
  - `CommandType::Patrol`: `Ground` → `Patrol`, else → `Inactive`
  - `CommandType::AttackGround`: `Ground` → `AttackGround`, else → `Inactive`
  - `CommandType::Reverse`: `Ground` → `Move`, else → `Inactive`
  - `CommandType::ScheduleDeliveries`: target has `SupplyDeliveryStation` → `GatherResources`, else → `Inactive`
  - `CommandType::SetRallyPoint`: any → `Move`
  - `CommandType::Enter`: target has own TunnelState → `Enter`, else → `Inactive`
  - `CommandType::Gather`: target is resource node → `GatherResources`, else → `Inactive`
  - `CommandType::DropOff`: target is own tunnel → `ReturnResources`, else → `Inactive`
  - All others → `Inactive`

### Existing Patterns to Follow

- **CursorTarget usage**: See `update_cursor_target` at command_panel.rs:175 — sets `cursor_target.kind`, `.entity`, `.location` each frame. Your system reads these.
- **SelectedUnitCapabilities**: Already computed by `update_command_panel_state` (line 281). Fields: `has_attack`, `can_target_ground`, `can_reverse`, `agent_carrying`, `is_chopper`, `chopper_has_supplies`.
- **Selection.active_type()**: Returns `Option<ObjectEnum>` — use to determine if active group is agent/chopper/structure (see `shared/types.rs:150`).
- **ObjectEnum helpers**: `.is_structure()` (objects.rs:359), `.is_unit()` (objects.rs:364), `.is_resource()` (objects.rs:370).
- **CommandType enum**: Defined at `game/units/types/state/commands.rs:80` — all variants listed there. The `CommandType` is already imported in command_panel.rs line 10.
- **LocalPlayer**: Resource at `game/types/factions.rs` — `local_player.0` is the player number.
- **Owner component**: `Owner.player_number()` returns `Option<u8>`.

### Testing Pattern

Tests in command_panel.rs (line 2488+) use direct function-level tests for `get_grid_slot_action`. For the resolution system, since it's a Bevy system, you have two options:
1. **Extract resolution to a pure function** that takes the relevant state as params and returns `PointerDisplayType` — then the system just calls it and writes the resource. Tests call the pure function directly. **This is the recommended approach** — matches the existing pattern of testable logic separated from ECS wiring.
2. Alternatively, use `TestApp` from `shared/testing/test_app.rs` for full ECS tests.

### Key Imports Already Available in command_panel.rs

Line 1-14 already import: `CommandType`, `AgentCarryState`, `AttackState`, `AttackCapability`, `AttackType`, `ObjectInstance`, `Player`, `DeploymentCenterState`, `BarracksState`, `ExtractionFacilityState`, `SupplyTowerState`, `TunnelState`, `HeadquartersState`, `SupplyChopperState`. Also `super::types::*` which covers all UI types.

Missing imports you'll need: `LocalPlayer` from `crate::types` (already available via `use crate::types::*` on line 2), `SpaceCrystalPatch` and `SupplyDeliveryStation` from `crate::game::types`.

## Dependencies

- **`update_cursor_target` system** (command_panel.rs:175): Must run before `resolve_pointer_display_type` so CursorTarget is fresh. Already runs in the UiHud set. Ordering enforced via `.after(update_command_panel_state)` which itself is `.after(update_cursor_target)`.
- **`update_command_panel_state` system** (command_panel.rs:281): Must run before resolution so `SelectedUnitCapabilities` and `ObjectInterfaceState` are up-to-date. Enforce with `.after(update_command_panel_state)` in mod.rs registration.
- **Sibling task `pointer_display_rendering`**: Depends on this task's `PointerDisplayType` resource. That task reads the resource to update visuals. No code conflict — the rendering task only adds a new system and entity, doesn't modify the same files.
