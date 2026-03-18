# Ticket: Fix Phantom Command Panels on Empty Tiles

## Current State
Left-clicking empty tiles across the map opens a unit command menu, even though no visible unit exists at those locations. This was initially observed underneath the deployment center but QA testing confirmed it occurs on **several random tiles** across the entire map. This indicates invisible or misplaced entities with `Selectable` components are scattered across the map, or the selection/raycast system is producing false positives on empty tiles.

Additionally, buildings are very difficult to select despite their large visual size — users have to click a very specific spot. This is likely related to the 3D raycast hit detection not aligning with visual positions (same root cause as the selection click offset issue in `qa_tasks/2026-03-06_fix_selection_click_offset.md`).

## Desired State
- Clicking empty space (any tile with no entity) should deselect the current selection. No command panel should appear.
- Clicking a structure should select it and show that structure's ObjectInterfaceState.
- No phantom unit command panels should appear on tiles where no visible unit exists, anywhere on the map.

## Justification
Per `features/control_system.md`, selecting a structure should show that structure's ObjectInterfaceState, and clicking empty space should deselect. A unit command panel appearing for a non-existent unit violates both rules. Identified during QA testing and confirmed as a pure implementation bug via forum topic `phantom_command_panel_under_deployment_center.md`. QA's follow-up confirmed the issue is map-wide, not localized to the deployment center.

## QA Steps
1. Start a new game as GDO faction.
2. Left-click on multiple empty tiles across the map (not near any visible unit or structure).
3. Verify no command panel appears on any empty tile — clicking should deselect or do nothing.
4. Left-click on spots directly below/underneath the deployment center sprite.
5. Verify no unit command panel appears when clicking empty space near the deployment center.
6. Left-click on the deployment center itself and verify the structure's interface panel appears (not a unit command panel).
7. Left-click on empty ground away from any entity and verify the selection clears.
8. Spawn a unit and verify that clicking the unit correctly shows its command panel.

## Expected Experience
- Clicking any empty tile across the map produces no command panel (selection clears or nothing happens).
- Clicking the deployment center itself shows the deployment center's structure interface.
- No phantom/ghost unit command menus appear anywhere on the map.
- Only clicking actual visible units produces a unit command panel.
