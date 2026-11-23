# HUD/UI Bugs: Command Panel, Build Button Label, and Extra Portrait Tile

## Metadata
- **Created by**: operator
- **Created**: 2026-03-25T00:00:00Z
- **Status**: open

## Close Votes
VOTE:task_planner
VOTE:developer
VOTE:designer
VOTE:software_architect
VOTE:automatic_qa
VOTE:task_splitter

## Discussion

### [operator] 2026-03-25T00:00:00Z

The user has reported three HUD/UI bugs. These all relate to the in-game HUD and selection interface.

**Bug 1: Peacekeeper has no command panel when selected**
When a Peacekeeper unit is selected, no command panel appears. Other units presumably show a command panel correctly. The Peacekeeper should have a command panel with its available actions displayed.

**Bug 2: Extraction Facility build button just says "Build"**
The tile/button to start building an Extraction Facility only displays the label "Build" with no indication of what is being built. It should say something like "Build Extraction Facility" or at minimum identify the structure.

**Bug 3: Bottom row of HUD has an extra tile / duplicate portrait**
The bottom row of the HUD appears to have an extra tile — it looks like there are two portraits being rendered when there should only be one. This could be a duplicate entity spawn, a rendering bug, or a layout issue.

All three bugs are in the HUD/UI layer. Relevant code is likely in `artifacts/developer/src/game/` — particularly any command panel, HUD, or UI modules. The developer agent should investigate and fix these.

### [designer] 2026-03-25T12:34:57Z

All three bugs are clear deviations from the design docs — no design changes needed.

**Bug 1 (Peacekeeper no command panel):** Per `gdo_objects.md`, Peacekeeper has `ObjectInterfaceState: BasicCombatUnitInterfaceState`. Per `control_system.md`, the CommandPanel is 'Visible when at least 1 player-owned object is selected.' The Peacekeeper should show the BasicCombatUnitInterfaceState grid: Q=Move, W=Reverse (conditional on CanReverse, which is false for LightInfantry so hidden), E=HoldPosition, A=Attack, S=Patrol, D=AttackGround (conditional on CanTargetGround=true, so visible), X=Stop.

**Bug 2 (EF button says 'Build'):** Per `gdo_objects.md`, the ExtractionFacility idle DefaultState command is 'Q: Build Extraction Plate'. The label should read 'Build Extraction Plate', not just 'Build'.

**Bug 3 (Extra tile/duplicate portrait):** Per `control_system.md`, the HUD Bar has exactly 5 sections (Minimap, Portrait, InfoPanel, SelectionPanel, CommandPanel) with fixed positions. A duplicate portrait or extra tile is a rendering bug.

Sending a feature_request to get these fixed.

### [automatic_qa] 2026-03-25T12:35:36Z

I investigated what existing tests and code can tell us about these bugs:

**Bug 1 (Peacekeeper no command panel):** The test `peacekeeper_default_commands_count` passes and confirms the Peacekeeper has 6 grid commands defined (Move, HoldPos, Attack, Patrol, AtkGround, Stop). The data layer is correct. The bug is likely in the UI spawning/selection layer — the system that detects a Peacekeeper is selected and triggers the command panel rebuild. This is in the runtime UI code (`ui/command_panel/ui_rebuild.rs` and `ui/hud.rs`) and would need visual/runtime verification or a targeted integration test.

**Bug 2 (Extraction Facility button says 'Build'):** Found the root cause in code. In `ui/command_panel/helpers.rs:327`, the `grid_button_label` function has specific labels for PowerPlant ('PP'), Barracks ('BK'), and SupplyTower ('ST'), but `DcBuild(ObjectEnum::ExtractionFacility)` falls through to the catch-all `DcBuild(_) => format!("[{}] Build", hotkey)`. Fix: add a specific match arm like `DcBuild(ObjectEnum::ExtractionFacility) => format!("[{}] EF\n200 SC", hotkey)` before the catch-all.

**Bug 3 (Extra tile / duplicate portrait):** Cannot diagnose this from code alone — it's a visual rendering issue. Would need runtime inspection or a screenshot to determine if it's a duplicate entity spawn or layout bug. No existing tests cover HUD portrait count.

**Automation assessment:** Bug 2 is easily testable — a unit test for `grid_button_label` with `DcBuild(ObjectEnum::ExtractionFacility)` would catch this. Bugs 1 and 3 are harder to verify automatically without runtime/visual tests.

### [task_splitter] 2026-03-25T12:35:45Z

From a task decomposition and codebase investigation perspective:

**Bug 1 (Peacekeeper no command panel)**: The Peacekeeper IS extensively handled in command_panel/helpers.rs — grid actions, attack support, common commands are all defined. The is_panel_visible() function in state.rs returns true for any non-resource unit in Default state. This suggests the issue is likely in UI rendering/rebuild rather than missing logic. The developer should investigate ui_rebuild.rs and the panel visibility/rendering path.

**Bug 2 (EF build button says 'Build')**: Found the root cause — helpers.rs line 327 has a catch-all `DcBuild(_) => format!("[{}] Build", hotkey)`. PowerPlant, Barracks, and SupplyTower have specific labels, but ExtractionFacility falls through to this generic fallback. Fix: add `DcBuild(ObjectEnum::ExtractionFacility) => format!("[{}] EF\n{cost} SC", hotkey)` before the catch-all.

**Bug 3 (extra portrait tile)**: Need more investigation in hud.rs portrait rendering.

**Splitting recommendation**: When this becomes a feature_request, all three are small fixes in the UI layer (command_panel + hud.rs). I'd recommend a single developer_task covering all three — they're all quick HUD fixes in closely related code, and splitting further would add overhead without benefit.

### [task_planner] 2026-03-25T12:38:13Z

I've investigated all three bugs in the codebase. Here are my findings:

**Bug 1 (Peacekeeper no command panel):** The code path *appears* correct from static analysis. Peacekeeper has the `Unit` component (peacekeeper.rs:132), falls to `ObjectInterfaceState::Default` in `update_command_panel_state()` (state.rs:311-316), and has 6 command slots registered in the EntityRegistry (peacekeeper.rs:55-62). The `is_panel_visible()` function (state.rs:124) returns true for Default state with non-empty non-resource selection. The rebuild trigger at ui_rebuild.rs:34 checks `selection.is_changed()` which should fire. This needs runtime investigation — possible causes include: (a) the entity not getting the `Selected` component properly, (b) a query filter mismatch, or (c) the `owned_by_local_player` check failing. The `hud_section_visibility_system` (hud.rs:250) hides CommandPanelSection when `!owned`, so ownership detection is another suspect.

**Bug 2 (EF build button just says Build):** CONFIRMED. In `grid_button_label()` at helpers.rs:322-327, there are specific label arms for PowerPlant (line 324: `PP 150 SC`), Barracks (line 325: `BK 200 SC`), and SupplyTower (line 326: `ST 200 SC`), but ExtractionFacility falls through to the catch-all at line 327: `DcBuild(_) => format!('[{}] Build', hotkey)`. Fix: add `CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility) => format!('[{}] EF\n200 SC', hotkey)` before the catch-all. The cost is 200 SC per extraction.rs test (line 235).

**Bug 3 (Extra portrait tile):** The info panel system (`update_info_panel_section_system`, hud.rs:441) uses global queries for `UnitIcon`, `StructureIcon`, and `ResourceIcon` (lines 449-451) to detect existing icons, but the selection panel (`update_selection_panel_section_system`, hud.rs:1106) also spawns entities with those same marker components (via `spawn_selection_portrait` at lines 1192-1196). This cross-contamination means the info panel's `existing_count` (line 464) includes selection panel icons, causing incorrect rebuild decisions and potential duplicate/orphaned UI elements. The fix should either scope the queries by parent entity, or use distinct marker components for info panel vs selection panel icons.

All three bugs are in the UI layer and can be addressed as separate tasks. Bug 2 is a trivial one-line fix. Bugs 1 and 3 may require runtime verification.

### [software_architect] 2026-03-25T12:40:27Z

## Architectural Analysis of All Three Bugs

I've investigated the codebase and found root causes for Bugs 1 and 2, plus a strong lead on Bug 3.

---

### Bug 1: Peacekeeper has no command panel — ROOT CAUSE FOUND

**File**: `src/ui/command_panel/state.rs`, `update_command_panel_state()`

The ownership guard logic at lines 154-157 correctly sets `unit_caps.owned_by_local_player = true` for player-owned units. However, for the unit branch (non-structure active group), line 289-291 then calls:

```rust
let new_caps = compute_selected_unit_capabilities(&selected_units, &selection);
if *unit_caps != new_caps {
    *unit_caps = new_caps;
}
```

`compute_selected_unit_capabilities()` (in helpers.rs:18-58) returns a `SelectedUnitCapabilities` where `owned_by_local_player` defaults to `false` — it never sets this field. The full struct replacement at line 291 overwrites the ownership flag that was just set.

Then `rebuild_command_panel_ui()` at line 43 checks:
```rust
if !is_panel_visible(&interface_state, &selection) || !unit_caps.owned_by_local_player {
    return;
}
```

Since `owned_by_local_player` is always `false` for unit selections, the command panel never renders for ANY unit in Default state (Peacekeeper, SupplyChopper, SyndicateGuard, CultsSoldier, CultsGunner). Agents and Recruits are unaffected because they use AgentMenu/CultsRecruitMenu paths.

**Fix**: After line 291 (`*unit_caps = new_caps`), add `unit_caps.owned_by_local_player = owned;` to preserve the ownership flag. Or better, pass `owned` into `compute_selected_unit_capabilities` and set it there.

---

### Bug 2: Extraction Facility build button just says "Build" — ROOT CAUSE FOUND

**File**: `src/ui/command_panel/helpers.rs`, `grid_button_label()`

Lines 324-327:
```rust
CommandButtonAction::DcBuild(ObjectEnum::PowerPlant) => format!("[{}] PP\n150 SC", hotkey),
CommandButtonAction::DcBuild(ObjectEnum::Barracks) => format!("[{}] BK\n200 SC", hotkey),
CommandButtonAction::DcBuild(ObjectEnum::SupplyTower) => format!("[{}] ST\n200 SC", hotkey),
CommandButtonAction::DcBuild(_) => format!("[{}] Build", hotkey),
```

The catch-all `DcBuild(_)` at line 327 matches `DcBuild(ObjectEnum::ExtractionFacility)` since there's no explicit arm for it. The label shows generic "Build" instead of identifying the structure.

**Fix**: Add an explicit arm before the catch-all:
```rust
CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility) => format!("[{}] EF\n200 SC", hotkey),
```

Also check `grid_button_enabled_ext()` — lines 400-402 have explicit arms for PP/BK/ST affordability but the catch-all path may not correctly check EF's 200 SC cost.

---

### Bug 3: Extra portrait tile — INVESTIGATION LEADS

This is harder to pin down without running the game, but here are the strongest leads:

1. **SelectionPanel visibility threshold**: `hud_section_visibility_system` shows the SelectionPanel when `total >= 2`. If `Selection::total_entity_count()` is overcounting (e.g., an entity appears in multiple groups, or a dead entity lingers in selection), the SelectionPanel would incorrectly appear for single-select, showing what looks like an extra portrait card.

2. **Info Panel icon cleanup**: In `update_info_panel_section_system`, icons are despawned by iterating the `existing_unit_icons`/`existing_struct_icons` queries (lines 469-477), then section children are rebuilt. If an icon entity from a previous selection isn't properly cleaned up (e.g., the icon was a child of the info section but the component marker was removed), it could persist and show as a duplicate.

3. **Portrait initial child**: The PortraitSection is spawned with a "Portrait" text child (setup_hud line 140-144). `update_portrait_section_system` calls `despawn_children()` before rebuilding. If there's a frame where the text isn't despawned before the new portrait renders (command batching), there could be a transient duplicate.

**Recommended investigation**: Add `info!` logging to `hud_section_visibility_system` showing the total count and each section's visibility state. Check whether `Selection::total_entity_count()` returns the expected value for single-entity selection. Also check whether a dead entity or duplicate entity is lingering in the Selection resource.

### [developer] 2026-03-25T12:41:18Z

I've investigated all three bugs and found root causes for each.

**Bug 1: Peacekeeper (all units) command panel missing**

Root cause: In `src/ui/command_panel/state.rs`, `update_command_panel_state()` sets `unit_caps.owned_by_local_player = true` at line 156, but then `compute_selected_unit_capabilities()` at line 289 creates a NEW `SelectedUnitCapabilities::default()` struct (which has `owned_by_local_player: false`). Line 291 overwrites unit_caps with this new struct, clobbering the ownership flag back to false. This causes the command panel visibility check in `hud_section_visibility_system` (line 251: `total >= 1 && owned`) to always fail for units.

This affects ALL units, not just Peacekeepers. The fix: set `owned_by_local_player` on the new_caps after computing them, before the comparison/overwrite.

**Bug 2: Extraction Facility build button just says Build**

Root cause: In `src/ui/command_panel/helpers.rs` line 327, `grid_button_label()` has specific cases for PowerPlant, Barracks, and SupplyTower but falls through to a catch-all `DcBuild(_) => format!([{}] Build, hotkey)` for ExtractionFacility. Also need to add an affordability check at line 148 (EF costs 200 SC per registry tests).

Fix: Add `CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility) => format!([{}] EF\n200 SC, hotkey)` before the catch-all.

**Bug 3: Extra tile / duplicate portrait in bottom HUD row**

Most likely cause: `hud_section_visibility_system` uses `Visibility::Hidden` / `Visibility::Inherited` to toggle HUD sections. In Bevy 0.17, `Visibility::Hidden` hides rendering but the node still takes up layout space. The hidden SelectionPanel (250px wide) creates a visible gap in the bottom bar that appears as an extra empty tile between InfoPanel and CommandPanel. Fix: use `Display::None` instead of (or in addition to) `Visibility::Hidden` to collapse hidden sections from layout.

Secondary concern: The InfoPanel's cleanup code at lines 469-477 queries ALL UnitIcon/StructureIcon entities globally, including those spawned by the SelectionPanel. The comment says 'only despawn icon entities belonging to the info panel' but the code doesn't filter by parent. This could cause icon cross-contamination between panels.

All three fixes are straightforward. Ready to implement when planned_tasks arrive, or I can fix them directly if directed.
