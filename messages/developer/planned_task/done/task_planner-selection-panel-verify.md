# selection-panel-verify

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-selection-panel.md

## Task

**Verification task — the SelectionPanel is already fully implemented.**

Verify that the existing SelectionPanel implementation in `ui/hud.rs` matches the design spec. The following should already be working:

1. **Display system** (`update_selected_units_grid_system`): Renders a grid of unit portraits in the `UnitsGridSection` when 2+ entities are selected. Single-entity selection shows the InfoPanel instead. Portraits for entities in the ActiveGroup should have a highlight (sheer tint).

2. **Click interaction system** (`selection_portrait_click_system`): Handles all 5 interaction modes via `SelectionPortrait` component:
   - Left-click: Replace selection with only that unit
   - Shift-click: Remove that unit from selection
   - Ctrl-click: Replace selection with all units of same type in current selection
   - Ctrl-Shift-click: Remove all units of same type from selection
   - Alt-click: Center camera on that unit (no selection change)

3. **Edge cases**: Selection reducing to 1 unit hides panel and shows InfoPanel. Selection reducing to 0 becomes empty.

**What to verify:**
- Run `cargo test` — all existing tests pass
- Read `update_selected_units_grid_system` and confirm it shows portraits when 2+ selected, hides when 0-1
- Read `selection_portrait_click_system` and confirm all 5 click modes are implemented correctly
- Confirm `SelectionPortrait` component stores the referenced entity
- Confirm active group highlighting logic exists (portraits in ActiveGroup get visual distinction)
- If any gap is found, implement the fix

## Technical Context

### Key Files
- **`artifacts/developer/src/ui/hud.rs`** — Primary file to verify. Contains both systems and their tests.
- **`artifacts/developer/src/ui/types.rs`** — Defines `SelectionPortrait` component (line 382), `UnitsGridSection` marker (line 36).
- **`artifacts/developer/src/ui/mod.rs`** — System registration and ordering (lines 35-36).

### Implementation Summary (Already Present)

**`update_selected_units_grid_system`** (hud.rs:223):
- Queries selected units, structures, SCPs, and SDSs.
- Counts total selected entities (`total_selected = unit_count + struct_count + resource_count`).
- `total_selected == 0`: Shows 'No Selection' text.
- `total_selected == 1`: Shows InfoPanel (single structure, unit, SCP, or SDS detail view).
- `total_selected >= 2`: Rebuilds a 4-column grid (`grid_template_columns: RepeatedGridTrack::flex(4, 1.0)`) of portrait cards, max 12 cards. Structures shown first, then units, then resources.
- Active group highlighting: Computes `active_entities` from `selection.active_group()`, passes `in_active_group` bool to portrait spawn helpers.

**`spawn_selection_portrait`** helper (hud.rs:889):
- Spawns a card with `SelectionPortrait { entity }` + `Interaction::default()` for click handling.
- Shows owner color swatch, name (truncated at 14 chars), HP text, type label, and health bar.
- If `in_active_group`: spawns an absolute-positioned overlay with `BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.15))` (line 1019-1031).

**`spawn_resource_portrait`** helper (hud.rs:1038):
- Simpler card for SCP/SDS (no owner, no health bar). Also has active group overlay at line 1106.

**`selection_portrait_click_system`** (hud.rs:1126):
- All 5 click modes are implemented:
  - Alt+click (line 1145): Centers camera on portrait entity. Uses `ButtonInput<MouseButton>` rather than `Interaction::Pressed` due to Linux WM Alt+Click interception.
  - Ctrl+Shift-click (line 1179): Removes all entities of same `ObjectEnum` type from selection.
  - Shift-click (line 1192): Removes single entity via `selection.remove_entity()`.
  - Ctrl-click (line 1196): Retains only groups matching target type, clears all others.
  - Plain left-click (line 1220): Replaces entire selection with single clicked entity.
- Both `Selected` component and `Selection` resource are updated for immediate response.

**`SelectionPortrait`** (types.rs:382):
- Simple component: `pub struct SelectionPortrait { pub entity: Entity }`.

### System Ordering (mod.rs:35-36)
- `update_selected_units_grid_system` runs after `command_panel_hotkeys`
- `selection_portrait_click_system` runs after `update_selected_units_grid_system`
- Both in `DiagCategory::UiHud` set.

### Existing Tests (hud.rs:1600+)
- `selection_portrait_stores_entity` — Component field access (line 1602)
- `selection_portrait_click_removes_entity_on_shift` — Shift-click removal (line 1611)
- `selection_portrait_ctrl_click_retains_same_type` — Ctrl-click type filter (line 1634)
- `selection_portrait_ctrl_shift_removes_all_of_type` — Ctrl+Shift removal (line 1663)
- `selection_portrait_left_click_replaces_selection` — Plain click replacement (line 1693)
- `spawn_selection_portrait_creates_interaction_component` — Interaction component present (line 1766)

### Verification Steps
1. Run `cargo test` in `artifacts/developer/` — all tests should pass.
2. Read through the two systems in hud.rs to confirm spec compliance. Pay particular attention to:
   - The `total_selected >= 2` branch (line 816) uses a grid layout, not InfoPanel — correct.
   - The `total_selected == 1` branches (lines 277-815) show InfoPanel per entity type — correct.
   - Active group overlay color is semi-transparent white (`srgba(1.0, 1.0, 1.0, 0.15)`) — reasonable 'sheer tint'.
   - Alt-click camera centering accounts for oblique camera angle via z_offset calculation (line 1155).
3. If any gap is found (unlikely), implement the fix in hud.rs following the existing spawn helper pattern.

## Dependencies

None — this is a verification-only task. The SelectionPanel implementation is already complete and self-contained.
