# Ticket: Selection System and Control Groups

## Current State
No client-side selection or control group system exists. There is no way to track which objects the player has selected, group them by type, or save/recall selections.

## Desired State
Implement the client-side Selection and ControlGroups systems as part of ControlState:

**ControlState** (per player, client-side):
- Holds the current Selection and the ObjectInterfaceState for the ActiveGroup
- Validated against game state each tick: dead objects removed from Selection, invalid ObjectInterfaceState resets to default

**Selection**:
- Array of SelectionGroups, each with Type (ObjectEnum) and Instances (array of ObjectInstance refs)
- ActiveGroup: which group's commands are displayed (ObjectEnum | None when empty)
- Constraints:
  - Enemy/unowned objects: selection must contain exactly one object in one group
  - Own objects: no limit on count or mixing of types
  - Ungroupable objects (Groupable=false): always occupy their own SelectionGroup, even among same-type instances
  - ActiveGroup must be a type that exists in current Groups

**SelectionGroup**:
- Type: ObjectEnum
- Instances: array of ObjectInstance references
- Groupable objects of the same type combine into one group; Ungroupable objects each get their own group

**ControlGroups** (10 saved selections, indexed 0-9):
- Each group holds an array of ObjectInstance references
- Entities can belong to multiple groups simultaneously
- Persist for the duration of the game
- Operations:
  - Assign: replace group contents with current Selection
  - Add: merge current Selection into group (no duplicates)
  - Recall: replace current Selection with group contents
  - Recall and Center: Recall + center camera on group centroid
- Dead entities silently removed on recall

## Justification
Defined in `features/control_system.md` (Selection, SelectionGroup, ControlGroups sections). These are the foundational data structures that every other part of the control system depends on. ControlGroups are a standard RTS feature enabling rapid army management.

## QA Steps
1. Select a single owned unit. Verify Selection contains one SelectionGroup with that unit's type and one instance. Verify ActiveGroup is set to that type.
2. Select multiple owned units of the same type. Verify they combine into one SelectionGroup.
3. Select owned units of different types. Verify each type gets its own SelectionGroup. Verify ActiveGroup defaults to one of the types.
4. Select an enemy/unowned object. Verify the selection contains exactly one object and one group.
5. Attempt to add a second enemy object to the selection. Verify it replaces the first (constraint enforced).
6. Select a mix of owned and unowned objects. Verify the constraint prevents this (only unowned selection rules OR owned selection rules apply).
7. Select an Ungroupable object alongside another instance of the same type. Verify each occupies its own SelectionGroup.
8. Assign a selection to ControlGroup 0. Clear the selection. Recall ControlGroup 0. Verify the selection is restored.
9. Assign a selection to ControlGroup 1. Select additional units. Add to ControlGroup 1. Recall ControlGroup 1. Verify merged contents with no duplicates.
10. Assign a selection containing a unit to a ControlGroup. Destroy that unit. Recall the ControlGroup. Verify the dead unit is silently absent.
11. Recall and Center a ControlGroup. Verify the camera centers on the centroid of the group's members.
12. Verify an entity can belong to ControlGroups 0 and 1 simultaneously with correct recall from each.
13. Verify that each tick, if a selected object dies, it is removed from the Selection and ActiveGroup resets if its group becomes empty.

## Expected Experience
Selecting units should feel responsive. Selection constraints (enemy = single, ungroupable = isolated) should be enforced silently without error messages. ControlGroup operations (assign, add, recall, recall-and-center) should work instantly. Dead units should disappear from selections and control groups without any visible glitch or notification.
