# Close Votes
- [qa]
- [designer]
- [developer]
- [product_analyst]
- [project_manager]
- [task_planner]

# Topic: Phantom Command Panel Appears Under Deployment Center With No Unit Selected

**Opened by**: qa
**Status**: open

## qa (original)
During QA testing, the user discovered that left-clicking a specific spot underneath the deployment center brings up a unit command menu, even though no unit is selected at that location.

This suggests either:
1. An invisible or hidden entity at that position has a `Selectable` component and triggers the command panel
2. The deployment center's selection bounds extend below it and trigger a unit-type command panel instead of a structure-type panel
3. A spawned entity (possibly a unit spawn point or anchor) exists at that location with incorrect component setup

**Suggested action**: Investigate what entity exists at the position below the deployment center and why clicking it opens a unit command menu. Create a ticket to fix the selection/command panel behavior.

## designer (reply)
Pure implementation bug — no design ambiguity here. Per `control_system.md`, selecting a structure should show that structure's ObjectInterfaceState, and clicking empty space should deselect. A phantom command panel for a non-existent unit means something is selectable that shouldn't be, or selection bounds are wrong. Route to a bug ticket.

Vote: close

## developer (reply)
Agreed — this is a bug. Most likely cause: the rally point or a spawn anchor entity near the deployment center has a `Selectable` component it shouldn't have, or the deployment center's clickable bounds overlap a neighboring tile. The `Selectable` component is used in `src/game/utils.rs` and `src/game/units/systems/core.rs` for click detection. A bug ticket should investigate which entity is being selected and either remove its `Selectable` component or fix the selection bounds.

Vote: close

## product_analyst (reply)
Agreed — pure implementation bug, no design ambiguity. Per `features/control_system.md`, left-clicking empty space should deselect, and selecting a structure should show that structure's ObjectInterfaceState (not a unit command panel). A phantom unit command panel means an entity with `Selectable` + unit-type interface state exists where it shouldn't. No feature implications. Route to a bug ticket.

Vote: close

## project_manager (reply)
Clear consensus — this is an implementation bug with no design ambiguity. Creating a bug-fix ticket to investigate and remove the phantom selectable entity near the deployment center. Voting to close.

## qa (update)
Additional user feedback during QA session — wider scope than originally reported:
1. Phantom command panels appear on **several random tiles** across the map, not just near the deployment center. Left-clicking empty tiles with no visible entity opens a unit command menu.
2. **Buildings are very difficult to select** despite their large visual size — user has to click a very specific spot. Likely same root cause as the selection click offset bug (`qa_tasks/2026-03-06_fix_selection_click_offset.md`) — 3D raycast hit detection doesn't align with visual positions, affecting both units and structures.

The bug ticket should account for the wider scope (random tiles, not just deployment center area).
