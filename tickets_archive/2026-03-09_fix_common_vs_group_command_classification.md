# Ticket: Fix CommonCommand vs GroupCommand Classification for Mixed Unit+Structure Selections

## Current State
When a unit and a structure are selected together (with the unit as the ActiveGroup), the commands Move, Stop, HoldPosition, and Patrol appear as CommonCommands (visually distinguished as shared by all selected entities). However, structures do not support any of these commands — only Attack and AttackMove are correctly classified as GroupCommands.

## Desired State
When determining CommonCommands vs GroupCommands, a command should only be classified as Common if **every object in the Selection** can execute it. Since structures cannot execute Move, Stop, HoldPosition, or Patrol, these must appear as GroupCommands (specific to the ActiveGroup) in any mixed unit+structure selection.

## Justification
`features/control_system.md` defines:
- **CommonCommands**: "available to every object in Selection"
- **GroupCommands**: "available to objects of type ActiveGroup"

The current `is_common_command()` logic in `src/ui/command_panel.rs` incorrectly classifies unit-only commands as common when structures are also selected. This violates the spec and misleads the player about which entities will execute which commands.

Originated from forum topic: `common_vs_group_command_classification_wrong.md`

## QA Steps
1. [human] Select a GDO Agent and a GDO Headquarters simultaneously (box-select or shift-click).
2. [human] Confirm the Agent is the ActiveGroup (its portrait should be highlighted in the SelectionPanel).
3. [human] Observe the CommandPanel — verify that **all** unit commands (Move, Stop, HoldPosition, Patrol, Attack, AttackMove) appear as **group-specific** commands (not common).
4. [human] Select only a single Agent (no structure). Verify all unit commands appear normally.
5. [human] Repeat steps 1-3 with a Syndicate Agent and Syndicate Tunnel to confirm cross-faction correctness.

## Expected Experience
- In step 3, every command in the panel should be visually styled as a group-specific command (not common), since no command is shared between the unit and the structure.
- In step 4, commands appear normally as they do for single-unit selection.
- In step 5, same visual distinction as step 3 — all commands are group-specific.
