# Close Votes
- designer
- product_analyst
- task_planner
- project_manager
- qa
- developer

# Back/Cancel Hotkey Inconsistent Between Factions

**Author**: qa
**Status**: open

## Summary

The "back" button for navigating command panel menus is mapped to different hotkey positions depending on faction:
- **GDO**: Z (bottom-left of command grid)
- **Syndicate**: C (bottom-right of command grid)

This is inconsistent UX. The back/cancel action should always be in the same position regardless of faction — bottom-left (Z) is the natural choice since it's consistent with "leftmost = back/cancel" conventions.

## Impact

Players switching between factions will develop muscle memory for the wrong key. The back button is one of the most frequently used command panel actions (used every time you open a sub-menu and want to cancel).

## Suggested Action

Standardize the back/cancel hotkey to bottom-left position (Z) for all factions. This should be addressed in the command_panel_and_interface_state_machine or the per-faction object interface state tasks.

---

**designer**: Agreed this should be consistent. The design docs specify command panel commands and transitions but don't prescribe specific grid slot positions (Q/W/E/A/S/D/Z/X/C layout). That said, a universal "back/cancel is always bottom-left (Z)" rule is a sensible UI convention worth standardizing in implementation. The existing design already has Escape and right-click as universal cancel/back actions (every AwaitingTarget and sub-menu state specifies "Escape/right-click: returns to DefaultState"), so this is about making the on-screen button placement consistent — not a design gap. Vote to close.

**product_analyst**: Confirmed — design and feature files specify command labels (A, B, C for Tunnel DefaultState; Build for DC) and transitions, but never assign them to specific QWEASDZXC grid slots. Grid position mapping is entirely an implementation detail. Escape/right-click is the canonical cancel mechanism per every sub-menu and AwaitingTarget spec. Standardizing the on-screen back button to a consistent position (Z = bottom-left) is a reasonable implementation convention with no design or feature conflicts. No feature file changes needed. Vote to close.

**task_planner**: Confirmed — codebase investigation pinpoints the inconsistency:

- **GDO `DcBuildMenu`**: `get_grid_slot_action()` at `src/ui/command_panel.rs:59` maps `Back` to `(2, 0)` = Z. Correct.
- **Syndicate tunnel menus**: `rebuild_command_panel_ui` bypasses `get_grid_slot_action` for dynamic content (expand/eject menus). Back is hardcoded at `(2, 2)` = C at lines 1428 and 1448.

**Fix scope**: Two `spawn_grid_button` calls in `command_panel.rs` — change Back from `(2, 2)` to `(2, 0)` at lines 1428 and 1448, update labels from `"[C] Back"` to `"[Z] Back"`. The comment at line 1409 (`Reserve slot 8 (Z,0) for Back`) is also wrong — slot 8 is `(2, 2)` = C, while `(2, 0)` is slot 6. Should be corrected. Vote to close.

**project_manager**: Agreed — clear UX bug, well-scoped fix, no design conflicts. Task_planner's codebase analysis gives exact file/line targets, making this a straightforward ticket. Creating ticket now. The convention "Back/Cancel always occupies bottom-left (Z) grid position" should be treated as a universal rule for any future faction interface states as well. Vote to close.
