# Developer Task: Retroactive [auto] Re-tagging of UI-Focused QA Tasks

## Summary
Review 8 UI-focused QA task files and change `[human]`/`[semi]` tags to `[auto]` on steps that verify deterministic ECS-queryable UI state (button visibility, slot assignments, interface state transitions, info panel content). Steps requiring visual verification remain `[human]`.

## Dependencies
- **`2026-03-09_automated_qa_ui_state_queries.md`** — MUST complete first. The `[auto]` tags are only valid once the TestHarness UI State Query methods (`assert_command_visible`, `assert_interface_state`, `assert_info_panel_shows`, `assert_active_group_type`, `get_selection_panel_portraits`, `get_visible_commands`) exist.

## Technical Context

### This is a markdown file editing task — no Rust code changes

All work is editing QA step tags in `.md` files. No source code modifications.

### File Locations

The 8 affected files may be in `/qa_tasks/` or still in `/developer_tasks/` depending on pipeline progress. Check both directories. As of current state:

**In `/qa_tasks/`:**
1. `2026-03-09_barracks_interface_state.md`
2. `2026-03-09_supply_tower_interface_state.md`
3. `2026-03-09_dc_default_state_cancel_commands.md`
4. `2026-03-09_dc_ef_no_auto_enter_construction_submenu.md`
5. `2026-03-09_standard_bottom_row_commands.md`
6. `2026-03-09_info_panel_stale_on_control_group_switch.md`

**In `/developer_tasks/` (not yet implemented — re-tag when they reach `/qa_tasks/`):**
7. `2026-03-06_basic_combat_unit_interface_state.md`
8. `2026-03-06_selection_panel.md`

### Re-tagging Rules

**Change to `[auto]`** — steps that check:
- Button presence/absence at specific grid slots → `assert_command_visible(slot, action)` / `assert_command_not_visible(action)`
- Interface state transitions (DefaultState, ConstructionSubmenu, AwaitingTarget, etc.) → `assert_interface_state(expected)`
- Info panel showing correct entity data → `assert_info_panel_shows(entity)`
- ActiveGroup highlighting on correct unit type → `assert_active_group_type(expected_type)`
- Command panel slot layout verification → `get_visible_commands()` comparison

**Keep as `[human]`** — steps that require:
- Visual rendering verification (sprite appearance, animation, cursor changes)
- UX feel assessment ("feels responsive", "immediate visual feedback")
- Rally point marker visibility (visual indicator on ground)
- Right-click interaction observation (cursor changes, visual feedback)
- Rapid input testing (timing-sensitive behavior like "rapidly alternate")

### Per-File Re-tagging Guidance

#### 1. `barracks_interface_state.md` (9 steps)
- Step 1 `[human]` → `[auto]`: "verify command panel shows Q/X/C in correct grid positions" — `assert_command_visible` for each slot
- Step 2 `[human]`: KEEP `[human]` — "verify a Peacekeeper is added to the build queue and 50 SC is deducted" — involves pressing Q (game simulation + resource check, could be auto but also verifies visual feedback)
- Step 3 `[human]`: KEEP `[human]` — queue full rejection (could be auto but involves observing rejection behavior)
- Step 4 `[human]` → `[auto]`: "verify the last entry is removed and its full cost is refunded" — queue state + resource check is deterministic
- Step 5 `[human]` → `[auto]`: "verify nothing happens" — queue state unchanged is deterministic
- Step 6 `[human]` → `[auto]`: "verify cursor/state changes to AwaitingTarget[SetRallyPoint]" — `assert_interface_state`
- Step 7 `[human]`: KEEP `[human]` — "verify the rally point is set and state returns to DefaultState" — involves click interaction + visual rally marker
- Step 8 `[human]`: KEEP `[human]` — right-click rally involves visual verification
- Step 9 `[human]`: KEEP `[human]` — "exits from the B side and moves to the rally point" — spatial/visual verification

#### 2. `supply_tower_interface_state.md` (12 steps)
- Step 1 `[human]` → `[auto]`: slot verification — `assert_command_visible` for Q/X/C/S slots
- Step 2 `[human]`: KEEP `[human]` — production queue + visual feedback
- Step 3 `[human]` → `[auto]`: queue full rejection — deterministic state check
- Step 4 `[human]` → `[auto]`: cancel + refund — deterministic resource check
- Step 5 `[human]` → `[auto]`: empty queue cancel — deterministic no-op
- Step 6 `[human]` → `[auto]`: AwaitingTarget state — `assert_interface_state`
- Step 7 `[human]`: KEEP `[human]` — click + visual rally marker
- Step 8 `[human]`: KEEP `[human]` — right-click visual verification
- Step 9 `[human]` → `[auto]`: AwaitingTarget[ScheduleDeliveries] state check
- Step 10 `[human]`: KEEP `[human]` — click on SDS + visual verification
- Step 11 `[human]` → `[auto]`: command unavailable — button enabled state check
- Step 12 `[human]`: KEEP `[human]` — movement/rally visual verification

#### 3. `dc_default_state_cancel_commands.md` (7 steps)
- Step 1 `[human]` → `[auto]`: "only Build (Q) appears, no Cancel (X)" — `assert_command_visible((0,0), DcOpenBuildMenu)` + `assert_command_not_visible(DcCancel)`
- Step 2 `[human]` → `[auto]`: "Cancel Construction (X) now appears alongside Build (Q)" — `assert_command_visible` for both slots
- Step 3 `[human]` → `[auto]`: "full refund received, DC returns to idle" — resource check + interface state
- Step 4 `[human]` → `[auto]`: "Cancel Ready Building (X) appears" — slot check
- Step 5 `[human]` → `[auto]`: "75% refund received, DC returns to idle" — resource + state check
- Step 6 `[human]` → `[auto]`: "Cancel (X) still available inside BuildMenu" — slot check in BuildMenu state
- Step 7 `[human]` → `[auto]`: "X slot position is consistent (2,1)" — slot position check

#### 4. `dc_ef_no_auto_enter_construction_submenu.md` (6 steps)
- Step 1 `[human]` → `[auto]`: "interface shows DefaultState (DcIdle)" — `assert_interface_state(StructureMenu(DcIdle))`
- Step 2 `[human]`: KEEP `[human]` — "press Q to enter BuildMenu" + verify constructing sub-menu (could be auto but involves navigation)
- Step 3 `[human]` → `[auto]`: "return to DefaultState (DcIdle) and stay there" — `assert_interface_state` after Z/Escape
- Step 4 `[human]` → `[auto]`: "DefaultState is shown" after construction complete — interface state check
- Step 5 `[human]` → `[auto]`: "EfIdle is shown, not EfConstructing" — interface state check
- Step 6 `[human]` → `[auto]`: "return to EfIdle and stay there" — interface state check

#### 5. `standard_bottom_row_commands.md` (9 steps)
- Step 1 `[human]` → `[auto]`: "interface enters AwaitingTarget[SetRallyPoint]" — `assert_interface_state`
- Step 2 `[human]`: KEEP `[human]` — click interaction + visual rally
- Step 3 `[human]`: KEEP `[human]` — right-click interaction + visual
- Step 4 `[human]`: KEEP `[human]` — right-click object + visual
- Step 5 `[human]` → `[auto]`: "press Z, interface returns to previous state" — `assert_interface_state`
- Step 6 `[human]` → `[auto]`: "Escape behaves identically to Z" — `assert_interface_state`
- Step 7 `[human]` → `[auto]`: "X cancels, cost refunded" — resource + queue state check
- Step 8 `[human]` → `[auto]`: "X slot not shown when no queue" — `assert_command_not_visible`
- Step 9 `[human]` → `[auto]`: "Z slot not shown in DefaultState" — `assert_command_not_visible`

#### 6. `info_panel_stale_on_control_group_switch.md` (6 steps)
- Step 1-3 `[human]`: KEEP `[human]` — game setup steps (spawn, assign control groups)
- Step 4 `[human]` → `[auto]`: "verify info panel shows EF information" — `assert_info_panel_shows`
- Step 5 `[human]` → `[auto]`: "verify info panel and portrait immediately update to show DC information" — `assert_info_panel_shows`
- Step 6 `[human]`: KEEP `[human]` — "rapidly alternate" is timing-sensitive UX verification

#### 7. `basic_combat_unit_interface_state.md` (in developer_tasks — re-tag when in qa_tasks)
- Steps checking "command panel shows Attack/Move/Patrol in grid" → `[auto]`
- Steps checking "AttackGround only if CanTargetGround" → `[auto]`
- Steps checking "Reverse only if CanReverse" → `[auto]`
- Steps involving right-click resolution, cursor changes, visual feedback → `[human]`

#### 8. `selection_panel.md` (in developer_tasks — re-tag when in qa_tasks)
- Steps checking "ActiveGroup highlight on correct portraits" → `[auto]`
- Steps checking click interactions with portrait selection behavior → may be `[auto]` if using Selection resource queries
- Steps verifying visual appearance of portraits → `[human]`

### Handling Files with QA Failure Annotations

Several files have `## QA Failure` or `## QA Results` sections at the bottom. **Do NOT modify these sections** — they are historical records. Only re-tag steps in the `## QA Steps` section.

## Implementation Steps

1. For each file in `/qa_tasks/` from the list above (skip files still in `/developer_tasks/`):
   - Read the file
   - In the `## QA Steps` section, change `[human]` or `[semi]` to `[auto]` per the guidance above
   - Preserve all other content exactly
2. For files 7-8 still in `/developer_tasks/`: leave a note in your session log that they need re-tagging when they move to `/qa_tasks/`

## QA Steps
1. [semi] Open each of the 6 re-tagged QA task files in `/qa_tasks/`. For each file, verify that:
   - Every step checking button presence/slot assignment is tagged `[auto]`
   - Every step checking interface state is tagged `[auto]`
   - Every step checking info panel content is tagged `[auto]`
   - Every step checking ActiveGroup state is tagged `[auto]`
   - Every step requiring visual verification remains `[human]`
   - No step has lost its original verification intent
2. [auto] Count `[auto]` tags across all modified files — verify each file has at least 1 `[auto]` step.
3. [semi] Spot-check 2-3 re-tagged `[auto]` steps: confirm the check described maps to a UI State Query API method.

## Expected Experience
After re-tagging, the Automated QA Runner can execute `[auto]`-tagged steps programmatically. Only visual verification steps remain for human QA sessions.
