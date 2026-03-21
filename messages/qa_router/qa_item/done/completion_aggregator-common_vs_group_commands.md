# common_vs_group_commands

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

# common-vs-group-commands

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Fix: CommonCommand vs GroupCommand classification. As defined in `artifacts/designer/design/control_system.md` under 'CommandPanel'.

**Current design rule:** CommonCommands are commands available to EVERY object in the Selection. GroupCommands are commands available to objects of type ActiveGroup.

**The bug:** Commands should only appear as CommonCommands if ALL selected entities (across all SelectionGroups) support them. If even one selected entity type does not support a command, it must NOT be shown as a CommonCommand — it should only appear as a GroupCommand when its supporting group is the ActiveGroup.

**Behavior:**
- Clicking a CommonCommand issues it to ALL selected objects
- Clicking a GroupCommand issues it only to objects in the ActiveGroup

**Example:** If you select Peacekeepers (which have Attack) and a Supply Chopper (which has no Attack), then Attack must NOT be a CommonCommand. It should only appear as a GroupCommand when Peacekeepers are the ActiveGroup.

**Note:** Some of this functionality may already be partially implemented. Downstream agents should verify the current classification logic.

## QA Instructions

1. Select a group of Peacekeepers only. Verify Attack shows as a CommonCommand.
2. Now add a Supply Chopper to the selection (shift-click).
3. Verify Attack is NO LONGER a CommonCommand.
4. Tab to make Peacekeepers the ActiveGroup — verify Attack appears as a GroupCommand.
5. Tab to make Supply Chopper the ActiveGroup — verify Attack does NOT appear.
6. Verify Stop appears as a CommonCommand (both types support it).
7. Click Stop — verify both Peacekeepers and Supply Chopper receive the Stop command.
8. Click Attack (as GroupCommand with Peacekeepers active) — verify only Peacekeepers receive it.
