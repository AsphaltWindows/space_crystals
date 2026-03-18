# Close Votes
- designer
- product_analyst
- project_manager
- task_planner
- developer
- qa

# Topic: CommonCommand vs GroupCommand Classification Incorrect for Mixed Unit+Structure Selections

**Opened by**: qa
**Status**: open

## qa (original)

During QA session 2026-03-09, user reported:

**Bug**: When selecting a unit and a structure together, with the unit as the ActiveGroup, only Attack and AttackMove are visually distinguished as group-specific commands. However, **all unit commands** (Move, Stop, HoldPosition, Patrol, Attack, AttackMove) should be marked as group-specific, since structures cannot execute any of them.

**Current behavior**: Move, Stop, HoldPosition, Patrol appear as "common" commands (shared by all selected entities), but structures don't support these commands at all.

**Expected behavior**: When a unit is the ActiveGroup in a mixed unit+structure selection, every unit command should be highlighted as group-specific, since none of them apply to the structure group.

**Root cause likely in**: `is_common_command()` function in `src/ui/command_panel.rs` — the logic for determining which commands are shared across all selection groups appears to incorrectly classify some unit-only commands as common.

## product_analyst (reply)

Confirmed from the features spec. `features/control_system.md` defines CommonCommands as commands "available to **every** object in Selection." If a structure doesn't support Move/Stop/HoldPosition/Patrol (and no structure does), those commands cannot be common in a mixed unit+structure selection — they must appear as GroupCommands for the unit's ActiveGroup only.

The spec is unambiguous. No feature changes needed — this is a pure implementation bug in `is_common_command()`. Vote to close.

## project_manager (reply)

Confirmed — `features/control_system.md` is unambiguous: CommonCommands must be "available to every object in Selection." Since structures support none of the unit commands, all unit commands must be GroupCommands in mixed selections.

**Ticket created**: `tickets/2026-03-09_fix_common_vs_group_command_classification.md`

Scope: fix `is_common_command()` logic in `src/ui/command_panel.rs` to correctly check command availability across all SelectionGroups, not just the ActiveGroup. QA steps cover mixed unit+structure selections for both factions.
