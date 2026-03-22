# command_panel_ownership_guard

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-command_panel_ownership_guard.md

## Task

Add an ownership guard so that the CommandPanel, hotkeys, and right-click command resolution only function when the Selection contains objects owned by the local player. Currently these systems operate on any selected entity regardless of ownership.

### Changes needed:

1. **`is_panel_visible()` in `command_panel.rs`**: Add a check that all entities in the Selection are owned by the local player (`Owner::player_number() == Some(local_player.0)`). If any selected entity is enemy-owned or neutral, the panel should be hidden. This function needs access to `LocalPlayer` and an `Owner` query — either pass them as parameters or refactor `is_panel_visible` into a system helper that can query ownership.

2. **`update_command_panel_state()` in `command_panel.rs`**: Early-return (reset to default state, clear panel target) if the selection contains non-owned entities. Add `LocalPlayer` resource and `Owner` query to the system parameters.

3. **`command_panel_hotkeys()` in `command_panel.rs`**: Already calls `is_panel_visible()` — if that function properly gates on ownership, hotkeys are covered. Verify the early return at line 841 is sufficient.

4. **`right_click_move_command()` in `core.rs`**: Add an early-return guard that checks all entities in `selected_units` are owned by `local_player`. The query already fetches `&Owner` — add a check like: `if selected_units.iter().any(|(.., owner, ..)| owner.player_number() != Some(local_player.0)) { return; }`. This prevents issuing move/attack/enter/gather/etc. commands to enemy units.

5. **`execute_command_action()` in `command_panel.rs`**: Also add an ownership guard — this function handles button clicks and hotkey actions. It already has access to `selected_owners` for structures; add equivalent check for units.

### Existing patterns to follow:
- `LocalPlayer` resource is already used in `right_click_move_command` and in the click-target resolution system at line 180 of command_panel.rs
- `Owner` component is on all `ObjectInstance` entities
- `Owner::player_number()` returns `Option<u8>`, neutral entities return `None`
- The selection system already prevents mixed player/enemy selections, so the guard just needs to check if the first entity is owned (but a full check is safer)

## Technical Context

### Files to modify

1. **`artifacts/developer/src/ui/command_panel.rs`** — Primary file, 4 of 5 change sites:
   - `is_panel_visible()` (line 268): Currently a pure helper taking `(&ObjectInterfaceState, &Selection) -> bool`. To add ownership checks, either:
     - **(Recommended)** Add two more params: `local_player: &LocalPlayer` and `owner_query: &Query<&Owner>`. Then iterate `selection.groups.iter().flat_map(|g| &g.entities)` and check each entity's `Owner`. If any entity has `owner.player_number() != Some(local_player.0)`, return `false`.
     - Update all 2 call sites: `command_panel_hotkeys` (line 841) and the render/visibility system.
   - `update_command_panel_state()` (line 283): Add `local_player: Res<LocalPlayer>` and `owner_query: Query<&Owner>` to the system params. Early-return at the top after `active_type` if any entity in the selection fails the ownership check. On early-return: `*interface_state = ObjectInterfaceState::Default; panel_target.entity = None; return;`
   - `command_panel_hotkeys()` (line 822): Already calls `is_panel_visible()` at line 841. Once `is_panel_visible` gets ownership checking, add `local_player: Res<LocalPlayer>` and `owner_query: Query<&Owner>` to this system's params and pass them through. This system already has many params — the new ones go after `selection`.
   - `handle_command_button_clicks()` (line 791): Calls `execute_command_action` at line 817. Add `local_player: Res<LocalPlayer>` and `owner_query: Query<&Owner>`. Add a guard before the `execute_command_action` call that checks ownership of the active selection group.
   - `execute_command_action()` (line 1105): This is a regular function (not a system). Add `local_player: &LocalPlayer` param. It already receives `selection: &Selection` — add an early-return at the top if any entity in the active group is not owned by `local_player`. All callers (line 817 and line 1015) need to pass the new param.

2. **`artifacts/developer/src/game/units/systems/core.rs`** — 1 change site:
   - `right_click_move_command()` (line 241): Already has `local_player: Res<LocalPlayer>` (line 252) and `selected_units` query with `&Owner` at tuple position 3 (line 248). Add a guard after the early returns (after line 313, before line 315): `if selected_units.iter().any(|(_, _, _, owner, _, _, _, _, _)| owner.player_number() != Some(local_player.0)) { return; }`. The 9-element tuple is: `(Entity, &Transform, &UnitBaseEnum, &Owner, Option<&AttackState>, Option<&SupplyChopperState>, &ObjectInstance, Option<&AgentCarryState>, &mut CommandQueue)`.

### Key types and imports

- `LocalPlayer` — `crate::shared::types::LocalPlayer` (already re-exported as `crate::types::LocalPlayer`). A `Resource` wrapping `pub u8`.
- `Owner` — `crate::shared::types::Owner` (re-exported as `crate::types::Owner`). A `Component` wrapping `pub Option<u8>`. Method `player_number() -> Option<u8>`.
- `Selection` — `crate::shared::types::Selection`. Resource with `groups: Vec<SelectionGroup>`. Each `SelectionGroup` has `entities: Vec<Entity>`.
- `ObjectInterfaceState` — `crate::ui::types::ObjectInterfaceState`. Resource, the panel state machine.
- `CommandPanelTarget` — `crate::ui::types::CommandPanelTarget`. Resource with `entity: Option<Entity>`.

### Ownership check pattern

The idiomatic check for "all selected entities are owned by local player":

```rust
fn selection_owned_by_local_player(
    selection: &Selection,
    local_player: &LocalPlayer,
    owner_query: &Query<&Owner>,
) -> bool {
    selection.groups.iter().all(|group| {
        group.entities.iter().all(|entity| {
            owner_query.get(*entity).map_or(false, |owner| {
                owner.player_number() == Some(local_player.0)
            })
        })
    })
}
```

Consider adding this as a standalone helper function near `is_panel_visible` in `command_panel.rs` since it's needed in multiple places. Both `is_panel_visible` and `update_command_panel_state` can call it.

### System ordering

No new system ordering constraints needed. The existing ordering is:
- `update_command_panel_state` runs in `DiagCategory::UiHud`
- `command_panel_hotkeys` runs in `DiagCategory::UiHud`
- `right_click_move_command` runs in `DiagCategory::Commands`

The ownership guard is a filter within each system, not a new system — no ordering changes required.

### Render-side visibility

Check `update_command_panel_visibility` or similar rendering system — it likely calls `is_panel_visible()`. If it does, ensure it also gets the new params for the ownership-aware version. Search for all call sites of `is_panel_visible` to be thorough.

### Testing considerations

- Existing tests may need `LocalPlayer` resource inserted in test `App` setup
- Test that selecting an enemy unit hides the panel: spawn entity with `Owner(Some(2))`, set `LocalPlayer(1)`, verify `is_panel_visible` returns false
- Test that selecting own unit shows the panel: spawn with `Owner(Some(1))`, `LocalPlayer(1)`, verify returns true
- Test that `right_click_move_command` does nothing when enemy unit selected

## Dependencies

None — this is a standalone guard/filter change to existing systems with no dependency on other pending tasks.
