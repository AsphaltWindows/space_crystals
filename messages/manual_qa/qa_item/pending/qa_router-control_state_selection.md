# control_state_selection

## Metadata
- **From**: qa_router
- **To**: manual_qa

## Content

## Content

## Content

# control-state-selection

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the control state and selection system as defined in `artifacts/designer/design/control_system.md` under ControlState, Selection, SelectionGroup, BoxSelection, ControlGroups, and GroupCycling.

**ControlState:**
Client-side state containing the player's current Selection and ObjectInterfaceState. Not part of game simulation. Validated each tick: removed objects pruned from Selection, invalid ObjectInterfaceState resets to default.

**Selection:**
Set of selected ObjectInstances grouped by type. Has an ActiveGroup that determines which group's commands display.
- Constraints:
  - Non-owned objects: selection must contain exactly 1 object and 1 group
  - All-owned: no limit on count or mixing types
  - Ungroupable objects always get their own SelectionGroup even with other instances of same type
  - ActiveGroup must reference an existing group

**SelectionGroup:**
Group of instances sharing the same ObjectEnum type. Groupable objects combine; Ungroupable objects each get own group.

**BoxSelection (drag-select) Priority Tiers:**
1. Own units — ALL in box selected (multi-select)
2. Own buildings — single-select: closest to box center
3. Enemy units — single-select: closest to box center
4. Enemy buildings — single-select: closest to box center
5. Neutral objects — single-select: closest to box center
Only tier 1 produces multi-selection. Highest tier with any objects wins.

**ControlGroups (0-9):**
10 saved selections per player, stored in ControlState. Entities can belong to multiple groups. Destroyed entities silently removed on recall.
- **Assign**: replace group contents with current Selection
- **Add**: merge current Selection into group (no duplicates)
- **Recall**: replace Selection with group contents
- **Recall and Center**: recall + center camera on group centroid

**GroupCycling:**
- Tab: advance ActiveGroup to next SelectionGroup (wraps)
- Shift-Tab: advance to previous (wraps)

## QA Instructions

1. Click a single unit — verify it becomes selected and its commands appear.
2. Drag-box over own units — verify ALL own units in the box are selected.
3. Drag-box over a mix of own units and own buildings — verify only own units are selected (higher priority).
4. Drag-box over enemy units only — verify exactly 1 enemy unit selected (closest to center).
5. Select an enemy object — verify no other objects can be added to the selection.
6. Select multiple types of own units — verify they form separate SelectionGroups. Press Tab to cycle ActiveGroup. Verify Shift-Tab cycles backward and wraps.
7. Assign selection to control group 1 (Ctrl+1) — recall it (press 1) — verify same units selected.
8. Add to control group (Ctrl+Shift+1) — verify units merged, no duplicates.
9. Destroy a unit in a control group, recall it — verify destroyed unit silently absent.
10. Select an Ungroupable object alongside others of same type — verify each gets its own SelectionGroup.
