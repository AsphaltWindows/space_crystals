# hud-bar-content-redistribution

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-hud_bar_layout.md

## Task

Migrate content rendering from the monolithic `update_selected_units_grid_system` into separate per-section update systems for Portrait, InfoPanel, and SelectionPanel. This completes the HUD Bar layout by populating each section with appropriate content.

### What to do:

1. **Portrait section** (`update_portrait_section_system`):
   - When 1+ objects selected, display the active selection group's representative portrait (object type name, faction-colored icon/box, type info)
   - Single-select: show the one selected object's portrait
   - Multi-select: show the ActiveGroup's type portrait
   - When nothing selected: section is hidden (handled by visibility system from scaffold task)

2. **InfoPanel section** (`update_info_panel_section_system`):
   - When 1 object selected: show detailed stats (HP, armor, damage, speed, special properties — the content currently rendered in `update_selected_units_grid_system` for single-unit/single-structure/single-resource cases)
   - When 2+ selected: show summary info for the active group (count, type name)
   - Migrate the single-unit detail display, single-structure detail display, and single-resource detail display from the existing system into this new system

3. **SelectionPanel section** (`update_selection_panel_section_system`):
   - When 2+ objects selected: show the grid of unit portrait icons (the multi-select grid currently rendered by `update_selected_units_grid_system`)
   - Portraits belonging to the ActiveGroup should have a highlight
   - Migrate the multi-select icon grid rendering from the existing system

4. **Remove** the old `update_selected_units_grid_system` and the `UnitsGridSection` marker component. Remove the temporary `UnitsGridSection` attachment from the scaffold task. Clean up any orphaned icon marker components (`UnitIcon`, `StructureIcon`, `ResourceIcon`) if they are only used by the old system — or repurpose them for the new systems.

5. **Update system registration** in the UI plugin — replace the old system with the three new ones.

6. **Health bar updates**: The existing `unit_health_bars` and `struct_health_bars` queries in the old system need to be moved to whichever new system renders them (likely InfoPanel for single-select, SelectionPanel for multi-select icons).

### Key files:
- `src/ui/hud.rs` — refactor/replace `update_selected_units_grid_system`
- `src/ui/types.rs` — clean up old markers, potentially add new ones
- `src/ui/mod.rs` — update system registration

### Dependencies:
- Depends on `hud-bar-five-section-scaffold` (the 5-section layout must exist first)

## Technical Context

### Primary file: `artifacts/developer/src/ui/hud.rs` (2204 lines)

**The monolithic system to replace** is `update_selected_units_grid_system` (lines 312-973). It currently:
- Takes `Query<Entity, With<UnitsGridSection>>` as its anchor entity (line 314)
- Queries selected units, structures, resource patches (SpaceCrystalPatch), and supply delivery stations (SupplyDeliveryStation) — lines 315-319
- Tracks existing icons via `UnitIcon`, `StructureIcon`, `ResourceIcon` queries — lines 320-322
- Handles health bar updates via `UnitHealthBar` / `StructureHealthBar` queries — lines 323-324
- Uses `Selection` resource for active group highlight logic — line 325
- Has 5 render branches:
  1. **Nothing selected** (line 347): shows "No Selection" text
  2. **Single structure** (line 366): detailed structure display with icon, health bar, stats (HP, power, build radius, size, structure-specific state like DC/BK/EF/EP/ST)
  3. **Single unit** (line 615): detailed unit display with icon, health bar, stats (type, owner, combat, movement, turret)
  4. **Single resource** (line 742): crystal patch or SDS detailed display
  5. **Multi-select** (line 905): grid of portrait cards (max 12) with active group highlighting

**Current backward-compat setup**: The scaffold task attached `UnitsGridSection` to the `InfoPanelSection` entity (line 159: `InfoPanelSection, UnitsGridSection`). This must be removed.

### Section marker components (in `artifacts/developer/src/ui/types.rs`):
- `PortraitSection` (line 35) — already spawned in `setup_hud` at line 136
- `InfoPanelSection` (line 39) — spawned at line 158
- `SelectionPanelSection` (line 48) — spawned at line 182
- `UnitsGridSection` (line 44) — **to be removed** (backward-compat shim)

### Icon/health marker components (in `artifacts/developer/src/ui/types.rs`):
- `UnitIcon { unit_entity: Entity }` (line 69) — **keep**, repurpose for InfoPanel single-unit and SelectionPanel multi-select
- `UnitHealthBar { unit_entity: Entity }` (line 75) — **keep**, used for live health bar updates
- `StructureIcon { structure_entity: Entity }` (line 109) — **keep**, repurpose similarly
- `StructureHealthBar { structure_entity: Entity }` (line 115) — **keep**, used for live health bar updates
- `ResourceIcon { resource_entity: Entity }` (line 121) — **keep**, repurpose for resource displays
- `SelectionPortrait { entity: Entity }` (line 434) — **keep**, used by `selection_portrait_click_system` for click handling on multi-select cards

### Helper functions to reuse (in `hud.rs`):
- `spawn_stat_text()` (line 1473) — spawns a styled text node with optional bottom margin
- `spawn_selection_portrait()` (line 978) — spawns a portrait card for multi-select; used in the multi-select branch. Move this to SelectionPanel system scope
- `spawn_resource_portrait()` (line 1127) — spawns a resource portrait card for multi-select. Move to SelectionPanel scope

### Scaffold layout (from `setup_hud`, lines 124-206):
- **PortraitSection**: 150px wide, Column flex, placeholder "Portrait" text
- **InfoPanelSection**: flex_grow=1.0, overflow clip, placeholder "No Units Selected" text
- **SelectionPanelSection**: 250px wide, Column flex, overflow clip, placeholder "Selection Panel" text

### Visibility system (`hud_section_visibility_system`, lines 218-253):
- Already handles showing/hiding each section based on selection count
- Portrait: visible when total >= 1
- InfoPanel: visible when total >= 1
- SelectionPanel: visible when total >= 2
- **No changes needed** to this system — it's already correct

### System registration (in `artifacts/developer/src/ui/mod.rs`, lines 37-53):

Current registration:
```rust
hud::update_selected_units_grid_system.after(command_panel::command_panel_hotkeys),
hud::selection_portrait_click_system.after(hud::update_selected_units_grid_system),
```

Replace with:
```rust
hud::update_portrait_section_system.after(command_panel::command_panel_hotkeys),
hud::update_info_panel_section_system.after(command_panel::command_panel_hotkeys),
hud::update_selection_panel_section_system.after(command_panel::command_panel_hotkeys),
hud::selection_portrait_click_system.after(hud::update_selection_panel_section_system),
```

Note: `selection_portrait_click_system` (line 1215) handles click interactions on `SelectionPortrait` components in the multi-select grid. It must run **after** the SelectionPanel system that spawns those portraits.

### Selection resource (`artifacts/developer/src/shared/types.rs`):
- `Selection { groups: Vec<SelectionGroup>, active_group_index: Option<usize> }`
- `selection.total_entity_count()` — total selected entities
- `selection.active_group()` — returns `Option<&SelectionGroup>` (the highlighted group)
- `selection.active_type()` — returns `Option<ObjectEnum>` for the active group's type
- `selection.is_changed()` — Bevy change detection, used to trigger rebuild

### New system designs:

**`update_portrait_section_system`**:
- Query: `Query<Entity, With<PortraitSection>>`
- Needs: `Res<Selection>`, selected units query (for UnitType, ObjectInstance, Owner), selected structures query, selected resources query
- On selection change: despawn children, rebuild portrait for active group's representative entity
- For single-select: show that entity's portrait (color swatch + type name)
- For multi-select: show the active group's type portrait (use `selection.active_group()` to get the type, find one representative entity)
- Use `Owner::color()` for faction coloring (pattern seen at lines 400-411, 648-655)

**`update_info_panel_section_system`**:
- Query: `Query<Entity, With<InfoPanelSection>>` (NO longer `With<UnitsGridSection>`)
- Absorbs the single-select branches (single structure: lines 366-614, single unit: lines 615-741, single resource: lines 742-904)
- For multi-select: show summary (active group count + type name)
- For nothing selected: show "No Selection" placeholder
- Health bar queries (`UnitHealthBar`, `StructureHealthBar`) live here for single-select health bar updates

**`update_selection_panel_section_system`**:
- Query: `Query<Entity, With<SelectionPanelSection>>`
- Absorbs the multi-select branch (lines 905-957) — the grid of portrait cards
- Uses `spawn_selection_portrait()` and `spawn_resource_portrait()` helpers
- Health bar queries for multi-select card health bars live here
- Active group highlighting via `selection.active_group().entities`

### Key queries to split across systems:

The original system has these heavy queries (lines 315-324):
```rust
selected_units: Query<(Entity, &UnitType, &ObjectInstance, &Owner, &UnitBaseEnum, &MovementSpeed, Option<&AttackCapability>, Option<&Turret>), (With<Unit>, With<Selected>)>,
selected_structures: Query<(Entity, &ObjectInstance, &Owner, Option<&PowerValue>, Option<&BuildRadiusExtension>, Option<&DeploymentCenterState>, Option<&BarracksState>, Option<&ExtractionFacilityState>, Option<&ExtractionPlateState>, Option<&SupplyTowerState>), (With<StructureInstance>, With<Selected>, Without<Unit>)>,
sc_patches: Query<&SpaceCrystalPatch>,
selected_scp: Query<(Entity, &SpaceCrystalPatch, &ObjectInstance), (With<Selected>, Without<Unit>, Without<StructureInstance>)>,
selected_sds: Query<(Entity, &SupplyDeliveryStation, &ObjectInstance), (With<Selected>, Without<Unit>, Without<StructureInstance>, Without<SpaceCrystalPatch>)>,
existing_unit_icons: Query<(Entity, &UnitIcon)>,
existing_struct_icons: Query<(Entity, &StructureIcon)>,
existing_resource_icons: Query<(Entity, &ResourceIcon)>,
unit_health_bars: Query<(&mut Node, &UnitHealthBar), ...>,
struct_health_bars: Query<(&mut Node, &StructureHealthBar), ...>,
```

Each new system needs only the queries relevant to its job:
- **Portrait**: needs selected_units (lightweight: Entity, UnitType, ObjectInstance, Owner), selected_structures (Entity, ObjectInstance, Owner), selected resources, Selection
- **InfoPanel**: needs all the detailed queries for single-select stats + health bars
- **SelectionPanel**: needs entity/name/owner/ObjectInstance for multi-select cards + health bars + Selection for active group

### Tests (lines 1491-2204):
- Existing tests at lines 1497+ cover `resource_bar_fields_for_faction` and `hud_section_visibility_system`
- The visibility system tests (lines 2059+) spawn entities with `PortraitSection`, `InfoPanelSection` (with `UnitsGridSection`), `SelectionPanelSection`
- **Update these tests**: remove `UnitsGridSection` from `InfoPanelSection` spawns (lines 2070)
- Add tests for the new systems if feasible (system tests using `world.run_system_once()`)

### Imports needed:
The current system imports at the top of `hud.rs` (lines 1-11) already include all necessary types. The new systems will need the same imports — no new dependencies needed.

## Dependencies

- **`hud-bar-five-section-scaffold`** (completed): The 5-section layout with `PortraitSection`, `InfoPanelSection`, `SelectionPanelSection` markers must exist. This task was completed and the scaffold is already in `setup_hud`. The backward-compat `UnitsGridSection` on `InfoPanelSection` (line 159) is what this task removes.
- **`hud_section_visibility_system`** (existing, no changes): Already correctly manages section visibility based on selection count. The new content systems rely on sections being visible/hidden appropriately.
- **`selection_portrait_click_system`** (existing, line 1215): Must continue to work after refactoring. It queries `SelectionPortrait` components, which will now only be spawned by `update_selection_panel_section_system`. Update system ordering in mod.rs so it runs after the SelectionPanel system.
