# Ticket: Retroactive [auto] Re-tagging of UI-Focused QA Tasks

## Current State
Approximately 8 UI-focused QA tasks in `/qa_tasks` have QA steps tagged as `[human]` or `[semi]` for deterministic UI state checks (button visibility, slot assignments, interface state transitions, info panel content). These steps currently require human QA sessions even though they verify ECS-queryable state.

The affected tasks include:
- `barracks_interface_state` — button layout and hotkey assignments
- `supply_tower_interface_state` — same pattern, different structure
- `dc_default_state_cancel_commands` — cancel button visibility
- `dc_ef_no_auto_enter_construction_submenu` — menu state on selection
- `standard_bottom_row_commands` — slot assignments and presence
- `basic_combat_unit_interface_state` — conditional command visibility
- `selection_panel` — ActiveGroup highlight, click interactions
- `info_panel_stale_on_control_group_switch` — HUD data staleness

## Desired State
Each of the affected QA task files has its QA steps reviewed and re-tagged:
- Steps that verify button presence/absence in specific slots: `[auto]` (using `assert_command_visible`, `assert_command_not_visible`)
- Steps that verify interface state transitions: `[auto]` (using `assert_interface_state`)
- Steps that verify info panel content: `[auto]` (using `assert_info_panel_shows`)
- Steps that verify ActiveGroup highlighting: `[auto]` (using `assert_active_group_type`, `get_selection_panel_portraits`)
- Steps that verify visual rendering, animation, or UX feel: remain `[human]`

## Justification
Per `features/automated_qa_system.md` Retroactive Tagging section and the UI State Queries update. The new UI State Query API makes these deterministic state checks fully automatable. Re-tagging enables the Automated QA Runner (Layer 3) to process these tasks without human interaction for the automatable steps. This directly addresses the QA bottleneck identified in forum topic `qa_automation_epic_ui_testing_push.md`.

**Dependency**: This ticket depends on `2026-03-09_automated_qa_ui_state_queries.md` being implemented first. The re-tagging is only valid once the UI State Query methods exist in the TestHarness.

## QA Steps
1. [semi] Open each of the 8 listed QA task files in `/qa_tasks` (or `/qa_human_review`). For each file, verify that:
   - Every step checking button presence/slot assignment is tagged `[auto]`
   - Every step checking interface state (DefaultState, ConstructionSubmenu, etc.) is tagged `[auto]`
   - Every step checking info panel content is tagged `[auto]`
   - Every step checking ActiveGroup state is tagged `[auto]`
   - Every step requiring visual verification remains `[human]`
   - No step has lost its original verification intent (the check being performed is the same, only the tag changed)
2. [auto] Run `grep -c '\[auto\]' /qa_tasks/<each_file>` — verify the auto-tagged step count has increased compared to the original (at minimum, each file should have at least 1 `[auto]` step).
3. [semi] Spot-check 2-3 re-tagged `[auto]` steps: confirm the check they describe is achievable using the UI State Query API methods (e.g., a step saying "verify Build button in slot (0,0)" maps to `assert_command_visible((0,0), CommandButtonAction::Build)`).

## Expected Experience
After this ticket is complete, running the Automated QA Runner against these 8 tasks should execute the `[auto]`-tagged steps programmatically. Only visual verification steps (sprite rendering, highlight appearance) should remain for human QA sessions. The total number of steps requiring human review across these 8 tasks should decrease significantly (estimated 60-80% of steps become `[auto]`).
