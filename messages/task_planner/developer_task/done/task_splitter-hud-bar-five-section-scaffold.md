# hud-bar-five-section-scaffold

## Metadata
- **From**: task_splitter
- **To**: task_planner

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

3. **Add `hud_section_visibility_system`**: A new system that runs each frame and toggles section visibility by setting `Display::None` vs `Display::Flex` on each section's `Node` based on selection state:
   - Minimap: always `Display::Flex`
   - Portrait: `Display::Flex` when 1+ objects selected, else `Display::None` (but keep the space reserved — use `Visibility::Hidden` instead of `Display::None` to preserve layout)
   - InfoPanel: same as Portrait (1+ selected)
   - SelectionPanel: visible when 2+ objects selected
   - CommandPanel: visible when 1+ player-owned objects selected
   - **Critical**: Use `Visibility::Hidden` (not `Display::None`) to keep slots reserved and prevent position shifting. The section nodes should always occupy their space.

4. **Register the new system** in the UI plugin alongside existing HUD systems.

5. **Remove** the old `UnitsGridSection` marker from setup (but do NOT remove the `update_selected_units_grid_system` yet — that will be migrated in a follow-up task). Instead, temporarily attach `UnitsGridSection` to the new InfoPanel section so the existing system keeps working until content is redistributed.

### Key files:
- `src/ui/hud.rs` — `setup_hud`, add visibility system
- `src/ui/types.rs` — new marker components
- `src/ui/mod.rs` — register new system
