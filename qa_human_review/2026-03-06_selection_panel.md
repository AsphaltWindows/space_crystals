# Task: SelectionPanel

## Current State
No selection panel exists. When multiple units are selected, `update_selected_units_grid_system` in `src/ui/hud.rs:258` renders a simple multi-select card grid (up to 12 cards) with no click interactions and no ActiveGroup highlight. The center section of the HUD (`UnitsGridSection`) handles 0-selection, 1-unit, 1-structure, and multi-select cases in a single monolithic function.

## Desired State
Implement the SelectionPanel as a grid of unit portraits with click interactions and ActiveGroup highlighting. Visible when `Selection` contains 2+ entities, hidden otherwise.

## Technical Context

### Key Data Structures

- **`Selection` resource** at `src/shared/types.rs:124`: `groups: Vec<SelectionGroup>`, `active_group_index: Option<usize>`. Already has `active_group()`, `total_entity_count()`, `contains_entity()`, `remove_entity()`, `build_from_entities()`, `cycle_active_group()`.
- **`SelectionGroup`** at `src/shared/types.rs:115`: `object_type: ObjectEnum`, `entities: Vec<Entity>`.
- **`Selected` marker component** at `src/shared/types.rs` — added/removed by `commands.entity(e).insert(Selected)` / `.remove::<Selected>()`.
- **`selection_group_sync_system`** at `src/game/world/resources.rs:726` — rebuilds `Selection` from `Selected` markers each frame. Changes to `Selected` markers will automatically flow into `Selection` next frame.

### Where to Implement

**Primary file**: `src/ui/hud.rs` — modify `update_selected_units_grid_system` (line 258).

The existing multi-select branch (line 706-735) renders portrait cards using `spawn_multi_select_card()` (line 754). This branch needs to be replaced with the SelectionPanel implementation.

**New components** to add in `src/ui/types.rs`:
- `SelectionPortrait { pub entity: Entity }` — marker for each portrait node (replaces or extends existing `UnitIcon`/`StructureIcon` for multi-select)
- No other new types needed — `Selection` resource and `Selected` marker already exist

### Existing Patterns to Follow

1. **Portrait card rendering**: `spawn_multi_select_card()` at `src/ui/hud.rs:754` — current pattern for multi-select cards with owner color swatch, name, HP text, and health bar. Reuse this pattern for portrait rendering.

2. **Selection change detection**: Lines 278-279 compare `existing_count != total_selected` to detect selection changes and trigger UI rebuilds. Keep this pattern.

3. **Camera centering (for alt-click)**: `control_group_system` at `src/game/world/resources.rs:675-692` shows the pattern — query `MainCamera` transform, set `translation.x`/`.z` from target entity's transform.

4. **Click handling on UI nodes**: Add `Interaction` component to portrait nodes. Check `Interaction::Pressed` in a new system. Follow the pattern from `handle_command_button_clicks` in `src/ui/command_panel.rs`.

### Implementation Steps

1. **Add `SelectionPortrait` component** in `src/ui/types.rs`:
   ```rust
   #[derive(Component)]
   pub struct SelectionPortrait {
       pub entity: Entity,
   }
   ```

2. **Modify multi-select branch** in `update_selected_units_grid_system` (hud.rs line 706+):
   - Keep grid layout (`Display::Grid`, `RepeatedGridTrack`)
   - For each entity across all `Selection.groups`, spawn a portrait node with:
     - `SelectionPortrait { entity }` component
     - `Interaction::default()` component (enables click detection)
     - Owner color swatch, unit/structure name, health bar (existing pattern)
     - **ActiveGroup highlight**: If the entity is in `selection.active_group()`, add a semi-transparent overlay (e.g., white at 15% opacity) via an additional child node or adjusted `BackgroundColor`
   - System needs `Res<Selection>` parameter added to its signature

3. **Add portrait click handler system** — new public function in `src/ui/hud.rs`:
   ```rust
   pub fn selection_portrait_click_system(
       mut commands: Commands,
       keyboard: Res<ButtonInput<KeyCode>>,
       portraits: Query<(&Interaction, &SelectionPortrait), Changed<Interaction>>,
       mut selection: ResMut<Selection>,
       selected_query: Query<Entity, With<Selected>>,
       object_instances: Query<&ObjectInstance>,
       transforms: Query<&Transform>,
       mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<ObjectInstance>)>,
   )
   ```
   Handle click modifiers:
   - **Left-click (no modifier)**: Clear all `Selected`, insert `Selected` on portrait's entity only
   - **Shift-click**: Remove `Selected` from portrait's entity, call `selection.remove_entity()`
   - **Ctrl-click**: Find all entities in selection with same `ObjectEnum` type, clear all `Selected`, insert `Selected` on those entities only
   - **Ctrl-Shift-click**: Find all entities in selection with same `ObjectEnum` type, remove `Selected` from all of them
   - **Alt-click**: Read entity's `Transform`, set `MainCamera` transform x/z to match (no selection change)

4. **Register the new system** in `src/ui/mod.rs` line 22-33 — add `hud::selection_portrait_click_system` in the Update systems tuple, ordered after `hud::update_selected_units_grid_system`.

### Key Interactions

- **Modifier key detection**: Use `keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)` (and similarly for Ctrl, Alt). Follow pattern from `selection_system` at `src/game/world/resources.rs:118`.
- **Selection sync**: After modifying `Selected` markers, `selection_group_sync_system` will rebuild the `Selection` resource next frame. For immediate UI response, also directly modify `Selection` in the click handler.
- **ActiveGroup highlight update**: The highlight should update when `Selection.active_group_index` changes (e.g., after Tab press). Since `update_selected_units_grid_system` runs every frame and checks for selection changes, the highlight will naturally update when the active group changes — but the change detection (line 278-279) only compares entity count. **Needs enhancement**: also track `active_group_index` changes to trigger rebuild on Tab cycling.
- **Edge case — selection to 1**: When shift-click or ctrl-shift-click reduces selection to 1 entity, the single-unit/structure branch (lines 312-705) will take over next frame since `total_selected == 1`.

### Camera Query Ambiguity

The existing `update_selected_units_grid_system` does not query `Transform` for camera or entities. The portrait click handler needs separate `Transform` queries for entities and camera. Use `Without<>` filters to disambiguate:
- Camera transform: `Query<&mut Transform, (With<MainCamera>, Without<SelectionPortrait>)>`
- Entity transform: `Query<&Transform, Without<MainCamera>>`

## Dependencies
- `selection_system_and_control_groups` — provides `Selection` resource, `Selected` marker, `selection_group_sync_system`, `active_group_cycle_system`
- `command_panel_and_interface_state_machine` — provides `ObjectInterfaceState`, command panel Tab cycling in `command_panel_hotkeys`

## QA Steps
1. [human] Select 2+ owned units. Verify the SelectionPanel appears as a grid of portraits.
2. [human] Verify portraits of units in the ActiveGroup have a sheer highlight. Verify portraits of units in other groups do not.
3. [human] Press Tab to change ActiveGroup. Verify the sheer highlight moves to the new ActiveGroup's portraits.
4. [auto] Select exactly 1 unit. Verify the SelectionPanel is hidden.
5. [auto] Select 0 units (click empty ground). Verify the SelectionPanel is hidden.
6. [human] With 3+ units selected, left-click a portrait. Verify the selection is replaced with only that unit and the SelectionPanel hides (now 1 unit).
7. [human] With 3+ units selected, shift-click a portrait. Verify that unit is removed from the selection. Verify the SelectionPanel updates (one fewer portrait).
8. [human] With 3+ units selected (including 2+ of the same type), ctrl-click a portrait. Verify the selection is replaced with all units of that type from the previous selection.
9. [human] With a mixed-type selection (e.g., 3 Peacekeepers + 2 other units), ctrl-shift-click a Peacekeeper portrait. Verify all Peacekeepers are removed from the selection. Verify remaining units stay selected.
10. [human] With 2+ units selected, alt-click a portrait. Verify the camera centers on that unit. Verify the selection does not change.
11. [human] With exactly 2 units selected, shift-click one portrait. Verify the selection reduces to 1 unit and the SelectionPanel hides.
12. [human] With 2 units of the same type selected, ctrl-shift-click a portrait. Verify all units of that type are removed, emptying the selection entirely.

## Automated QA Results (Re-run 2026-03-09)
- Steps 1-3 [human]: DEFERRED to human review
- Step 4 [auto]: PASS — single unit selection correctly has 1 entity, panel hidden
- Step 5 [auto]: PASS — empty selection correctly has 0 entities, panel hidden
- Steps 6-12 [human]: DEFERRED to human review

## QA Failure — 2026-03-09
- Step 1 [human]: PASS — portrait grid appears on multi-select
- Step 2 [human]: PASS — ActiveGroup has sheer highlight
- Step 3 [human]: PASS — Tab cycles highlight between groups
- Step 6 [human]: PASS — left-click portrait selects only that unit
- Step 7 [human]: PASS — shift-click removes unit from selection
- Step 8 [human]: PASS — ctrl-click selects all of that type
- Step 9 [human]: PASS — ctrl-shift-click removes all of that type
- **Step 10 [human]: FAIL — Alt-click on a portrait does NOT center the camera on that unit. No camera movement occurs.**
- Step 11 [human]: PASS — shift-click with 2 units reduces to 1, panel hides
- Step 12 [human]: PASS — ctrl-shift-click empties selection entirely
- Note: No InfoPanel displays during multi-select (only single-unit selection shows InfoPanel). This may be a separate issue.

## QA Failure — 2026-03-09 (repeat)
**Step 10 [human]: FAIL — Alt-click on a portrait still does NOT center the camera. No camera movement occurs. No log output related to alt-click detected in run_qa.log — the click handler does not appear to be firing or logging for alt-clicks. Third consecutive failure on this step.**

## QA Failure — 2026-03-09 (4th attempt)
**Step 10 [human]: PARTIAL — Camera centering now works, but triggers on alt+HOVER instead of alt+CLICK. Holding Alt and hovering over portraits in the selection grid automatically moves the camera to each unit as the cursor passes over them. The behavior should only trigger on a deliberate alt+click, not on hover. This makes the feature unusable in practice — any Alt interaction near the selection panel causes unwanted camera jumps.**

## QA Failure — 2026-03-09 (automated re-run, 5th attempt)
**ALL [auto] steps: FAIL — Test compilation failed.** Source code has compile errors in `src/game/world/faction.rs` and `src/ui/command_panel.rs`: `TunnelOperation::BuildingExpansion` pattern/initializer missing fields `grid_x`, `grid_z`. These are not related to this task but prevent any test from compiling. Auto steps cannot be verified until the compile errors are fixed.

## Automated QA Results (Re-run 2026-03-09, 6th attempt)
- Step 4 [auto]: PASS — single unit selection correctly has 1 entity, panel hidden
- Step 5 [auto]: PASS — empty selection correctly has 0 entities, panel hidden
- Steps 1-3, 6-12 [human]: DEFERRED to human review
- Note: Step 10 [human] has failed 4 consecutive times (alt-click triggers on hover not click). Remains the only blocking issue.

## Expected Experience
The SelectionPanel should appear instantly when 2+ units are selected and disappear instantly when the selection drops to 0 or 1. The ActiveGroup sheer highlight should update immediately on group cycling. Portrait click interactions should feel responsive with no delay. Left-click and ctrl-click should replace the selection cleanly. Shift-click and ctrl-shift-click should smoothly remove portraits from the grid without visual glitches. Alt-click should smoothly pan the camera to the clicked unit.
