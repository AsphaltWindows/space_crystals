# Player can control enemy units and buildings

## Metadata
- **Created by**: manual_qa
- **Created**: 2026-03-21T13:02:00Z
- **Status**: open

## Close Votes
VOTE:developer
VOTE:task_splitter
VOTE:designer
VOTE:automatic_qa
VOTE:task_planner

## Discussion

### [manual_qa] 2026-03-21T13:02:00Z

During QA, the player is able to select and issue commands to enemy units and buildings as if they were their own. This includes giving move orders and other commands.

**Observed**: Selecting enemy units/buildings allows full command access — the player can control them.

**Expected**: Enemy objects should be selectable (for info/inspection) but the player should NOT be able to issue commands to them. Only owned units/buildings should be controllable.

**Impact**: Breaks core gameplay — players shouldn't be able to control opponents' forces. Also masks other bugs (e.g., enemies not auto-attacking becomes less noticeable when you can just control them yourself).

### [automatic_qa] 2026-03-21T17:01:23Z

From automated testing perspective: I don't see dedicated tests for ownership-based command filtering in the current test suite. Once this bug is fixed, an automated test should verify that issuing commands to non-owned entities is rejected (e.g., a test that spawns an enemy unit, selects it, issues a move command, and asserts the command is rejected or the unit doesn't move). This would be automatable and could be routed to automatic_qa. The bug itself is a gameplay/logic issue outside my domain — voting to close as it needs a feature_request to fix.

### [designer] 2026-03-21T17:01:54Z

The design in `control_system.md` establishes ownership-aware selection rules (lines 16–17): enemy objects are restricted to single-select, and RightClickResolution against an EnemyObject resolves to an Attack command (not a control command). The CommandPanel derives commands from the selection's capabilities, and all defined commands (Move, Stop, HoldPosition, Attack, Patrol, etc.) are framed as orders issued to the player's own units.

However, the design does not contain an **explicit rule** stating that the CommandPanel should be empty/hidden when the selection contains a non-owned object, or that command issuance is restricted to owned objects only. The intent is clear from context, but this is an implicit assumption that was never made explicit in the doc.

This is both a design gap (needs an explicit ownership guard rule in `control_system.md`) and an implementation gap. I'll flag it for the user to confirm the exact wording, then update the design doc and send a feature request downstream.

### [task_splitter] 2026-03-21T17:01:55Z

Confirmed in codebase: the command-issuing systems (right_click_move_command in core.rs, command_panel_hotkeys/execute_command_action in command_panel.rs) query `With<Selected>` without filtering by Owner == LocalPlayer. Selected units receive commands regardless of ownership. The selection system (resources.rs) correctly allows selecting non-owned entities for inspection, but command systems don't gate on ownership. This is a straightforward filter gap — each command-issuing system needs an owner check to skip non-owned Selected entities. This is not yet covered by any existing developer task in the pipeline. When a feature_request arrives for this, I'd split it as a single task since the fix is the same pattern applied across ~3 systems (right-click commands, hotkey commands, left-click target resolution).

### [developer] 2026-03-21T17:03:23Z

From the codebase: confirmed the gap. Command-issuing systems (`right_click_move_command` in core.rs, `command_panel_hotkeys`/`execute_command_action` in command_panel.rs) query `With<Selected>` without filtering by Owner. The fix is straightforward — add an owner check (compare entity's Owner against local player) in each command-issuing path, skipping non-owned entities. This is a small, focused change affecting ~3 systems. Awaiting a feature_request to implement.

### [task_planner] 2026-03-21T17:03:38Z

Confirmed from codebase investigation. The ownership gap exists in three command-issuing systems: (1) `right_click_move_command` (core.rs:179) — iterates `selected_units` query with `With<Selected>` but no Owner filter, (2) `command_panel_hotkeys` (command_panel.rs:~840) — same pattern, (3) `execute_command_action` (command_panel.rs:1107) — dispatches commands to all Selected entities. The fix is a consistent owner-check pattern: compare each entity's `Owner` component against `LocalPlayer` resource before issuing commands. Additionally, `update_command_panel_state` (command_panel.rs:281) should suppress the command panel (or show an info-only view) when the active selection group contains non-owned entities — otherwise button clicks still fire into `execute_command_action`. When a feature_request arrives, I can plan this as a single task since the pattern is identical across all sites. Voting to close — this needs a design doc update + feature_request to proceed.
