# hud-bar-five-section-scaffold

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-hud_bar_layout.md

## Task

Restructure the bottom HUD panel in `src/ui/hud.rs` (`setup_hud`) from its current 3-section layout (Minimap, UnitsGridSection, CommandPanelSection) into the design-specified 5 fixed-position sections: Minimap, Portrait, InfoPanel, SelectionPanel, CommandPanel.

### What to do:

1. **Add new marker components** in `src/ui/types.rs`: `PortraitSection`, `InfoPanelSection`, `SelectionPanelSection`. Keep existing `MinimapSection` and `CommandPanelSection`.

2. **Restructure `setup_hud`** bottom panel children:
   - All 5 sections share the same vertical height (100% of the HUD bar)
   - Horizontal positions are fixed — use explicit widths (not flex-grow) so sections never shift
   - Suggested widths: Minimap ~200px, Portrait ~150px, InfoPanel ~flex-grow for remaining, SelectionPanel ~250px, CommandPanel ~200px (adjust as reasonable, but all must be present and fixed-position)
   - Each section gets its marker component
   - Spawn placeholder content (e.g., section name text) in Portrait, InfoPanel, and SelectionPanel sections
   - Leave Minimap and CommandPanel sections unchanged

3. **Add `hud_section_visibility_system`**: A new system that runs each frame and toggles section visibility using `Visibility::Hidden` vs `Visibility::Inherited`:
   - Minimap: always visible
   - Portrait: visible when 1+ objects selected, else Hidden
   - InfoPanel: same as Portrait (1+ selected)
   - SelectionPanel: visible when 2+ objects selected
   - CommandPanel: visible when 1+ player-owned objects selected
   - Use `Visibility::Hidden` (not `Display::None`) to keep slots reserved

4. **Register the new system** in the UI plugin alongside existing HUD systems.

5. **Remove** the old `UnitsGridSection` marker from setup but temporarily attach `UnitsGridSection` to the new InfoPanel section so the existing `update_selected_units_grid_system` keeps working.

### Key files:
- `src/ui/hud.rs` — `setup_hud`, add visibility system
- `src/ui/types.rs` — new marker components
- `src/ui/mod.rs` — register new system

## Technical Context

### Files to modify:

1. **`artifacts/developer/src/ui/types.rs`** — Add 3 new marker component structs:
   ```rust
   /// Marker component for the portrait section
   #[derive(Component)]
   pub struct PortraitSection;

   /// Marker component for the info panel section
   #[derive(Component)]
   pub struct InfoPanelSection;

   /// Marker component for the selection panel section
   #[derive(Component)]
   pub struct SelectionPanelSection;
   ```
   Place these near the existing `MinimapSection` (line 32), `UnitsGridSection` (line 36), and `CommandPanelSection` (line 114) markers.

2. **`artifacts/developer/src/ui/hud.rs`** — Two changes:

   **a) Restructure `setup_hud`** (lines 65-161, the bottom panel `.with_children` block):
   - Current layout: Minimap (200px) | UnitsGridSection (flex-grow) | CommandPanel (200px)
   - New layout: Minimap (200px) | Portrait (150px) | InfoPanel (flex-grow) | SelectionPanel (250px) | CommandPanel (200px)
   - The Minimap section (lines 86-122) stays unchanged — keep `MinimapSection` marker, same Node style.
   - Replace the center UnitsGridSection (lines 124-144) with three new sections:
     - Portrait section: 150px wide, spawn placeholder text \Portrait\, attach `PortraitSection` marker
     - InfoPanel section: flex-grow, spawn placeholder text, attach BOTH `InfoPanelSection` AND `UnitsGridSection` markers (so existing `update_selected_units_grid_system` continues to work — it queries `With<UnitsGridSection>` at line 225)
     - SelectionPanel section: 250px wide, spawn placeholder text, attach `SelectionPanelSection` marker
   - The CommandPanel section (lines 147-160) stays unchanged.
   - All sections: `height: Val::Percent(100.0)`, use explicit `width: Val::Px(N)` except InfoPanel uses `flex_grow: 1.0`.

   **b) Add `hud_section_visibility_system`**:
   ```rust
   pub fn hud_section_visibility_system(
       selection: Res<Selection>,
       unit_caps: Res<SelectedUnitCapabilities>,
       mut portrait_q: Query<&mut Visibility, (With<PortraitSection>, Without<InfoPanelSection>, Without<SelectionPanelSection>, Without<CommandPanelSection>)>,
       mut info_q: Query<&mut Visibility, (With<InfoPanelSection>, Without<PortraitSection>, Without<SelectionPanelSection>, Without<CommandPanelSection>)>,
       mut selection_q: Query<&mut Visibility, (With<SelectionPanelSection>, Without<PortraitSection>, Without<InfoPanelSection>, Without<CommandPanelSection>)>,
       mut command_q: Query<&mut Visibility, (With<CommandPanelSection>, Without<PortraitSection>, Without<InfoPanelSection>, Without<SelectionPanelSection>)>,
   ) {
       let total = selection.total_entity_count();
       let owned = unit_caps.owned_by_local_player;

       // Portrait: visible when 1+ selected
       if let Ok(mut vis) = portrait_q.single_mut() {
           *vis = if total >= 1 { Visibility::Inherited } else { Visibility::Hidden };
       }
       // InfoPanel: visible when 1+ selected
       if let Ok(mut vis) = info_q.single_mut() {
           *vis = if total >= 1 { Visibility::Inherited } else { Visibility::Hidden };
       }
       // SelectionPanel: visible when 2+ selected
       if let Ok(mut vis) = selection_q.single_mut() {
           *vis = if total >= 2 { Visibility::Inherited } else { Visibility::Hidden };
       }
       // CommandPanel: visible when 1+ player-owned selected
       if let Ok(mut vis) = command_q.single_mut() {
           *vis = if total >= 1 && owned { Visibility::Inherited } else { Visibility::Hidden };
       }
   }
   ```
   Key detail: `Selection::total_entity_count()` (shared/types.rs:155) sums all group entity counts. `SelectedUnitCapabilities::owned_by_local_player` is already computed by `update_command_panel_state` (command_panel/state.rs:154-156).

3. **`artifacts/developer/src/ui/mod.rs`** — Register the new system:
   - Add `hud::hud_section_visibility_system` to the `Update` system set within `DiagCategory::UiHud` (line 37-52)
   - It should run `.after(command_panel::update_command_panel_state)` since it reads `SelectedUnitCapabilities` which is written there
   - Add `PortraitSection, InfoPanelSection, SelectionPanelSection` to the imports from `types` module if needed

### Existing patterns to follow:
- Section marker pattern: each section has a unit struct with `#[derive(Component)]` — see `MinimapSection` (types.rs:32), `CommandPanelSection` (types.rs:114)
- HUD bottom panel uses absolute positioning: `position_type: PositionType::Absolute, bottom: Val::Px(0.0)` with explicit height `HUD_BOTTOM_PANEL_HEIGHT = 220.0`
- Background colors: Minimap uses `Color::srgba(0.2, 0.2, 0.2, 0.8)`, center uses `Color::srgba(0.15, 0.15, 0.15, 0.8)`, CommandPanel uses `Color::srgba(0.12, 0.15, 0.12, 0.9)`
- `column_gap: Val::Px(10.0)` between sections (set on parent HudPanel node at line 76)

### Key types/resources:
- `Selection` resource (`shared/types.rs:138`) — `.total_entity_count()`, `.groups`, `.active_group()`
- `SelectedUnitCapabilities` resource (`ui/types.rs:401`) — `.owned_by_local_player`
- `Visibility::Hidden` vs `Visibility::Inherited` — Hidden keeps layout space reserved, Inherited shows content. This matches the design requirement that positions are locked.

### Important Bevy ECS notes:
- Queries for the visibility system need `Without<>` filters to disambiguate when multiple marker components exist on different entities
- `Visibility` is automatically available on spawned Node entities in Bevy 0.17
- System ordering: `hud_section_visibility_system` must run after `update_command_panel_state` (which writes `SelectedUnitCapabilities`)

## Dependencies

- **`update_command_panel_state`** (existing system in `command_panel/state.rs`): The visibility system reads `SelectedUnitCapabilities::owned_by_local_player` which is computed by this system. Must run after it.
- **`update_selected_units_grid_system`** (existing system in `hud.rs:223`): This system queries `With<UnitsGridSection>`. The InfoPanel section must carry both `InfoPanelSection` AND `UnitsGridSection` markers to keep this working until the follow-up content redistribution task replaces it.
- No dependency on other pending tasks — this is standalone scaffolding.
