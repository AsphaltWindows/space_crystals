# Ticket: Standardize Back/Cancel Button to Bottom-Left (Z) Grid Position

## Current State
The command panel's Back/Cancel button is mapped to different grid positions depending on faction:
- GDO Deployment Center BuildMenu: Back is at grid position (2, 0) = Z (bottom-left). Correct.
- Syndicate Tunnel EjectMenu and ExpandMenu: Back is at grid position (2, 2) = C (bottom-right). Incorrect.

The label reads `[C] Back` for Syndicate menus and `[Z] Back` for GDO menus.

## Desired State
All command panel sub-menus across all factions use grid position (2, 0) = Z (bottom-left) for the Back/Cancel button. The label should read `[Z] Back` universally.

Specific changes in `src/ui/command_panel.rs`:
1. Line ~1428: Change `spawn_grid_button` for Back in Syndicate Tunnel EjectMenu from `(2, 2)` to `(2, 0)`, update label from `"[C] Back"` to `"[Z] Back"`.
2. Line ~1448: Change `spawn_grid_button` for Back in Syndicate Tunnel ExpandMenu from `(2, 2)` to `(2, 0)`, update label from `"[C] Back"` to `"[Z] Back"`.
3. Line ~1409: Fix the misleading comment (`Reserve slot 8 (Z,0) for Back`) — slot indices should match actual grid positions used.

Establish the convention: Back/Cancel always occupies bottom-left (Z) in any command panel sub-menu, for all current and future factions.

## Justification
Reported during QA testing (forum topic `forum/back_button_hotkey_consistency.md`). Players switching between factions develop conflicting muscle memory for the most frequently used command panel navigation action. The feature specs (`features/control_system.md`, `features/syndicate_objects.md`) specify command labels and transitions but do not prescribe grid slot positions — this is an implementation-level standardization. Escape/right-click remain the canonical cancel mechanism per spec; this ticket fixes the on-screen button consistency.

## QA Steps
1. Launch game, select Syndicate faction.
2. Build a Tunnel with an Agent.
3. Select the Tunnel.
4. Press C (Eject) to enter EjectMenu.
5. Verify the Back button appears in the bottom-left grid position (Z key position).
6. Press Z to go back. Verify it returns to DefaultState.
7. Press B (Expand Tunnel) to enter ExpandMenu.
8. Verify the Back button appears in the bottom-left grid position (Z key position).
9. Press Z to go back. Verify it returns to DefaultState.
10. Launch game, select GDO faction.
11. Select the Deployment Center.
12. Enter the BuildMenu.
13. Verify the Back button appears in the bottom-left grid position (Z key position).
14. Press Z to go back. Verify it returns to DefaultState.
15. Confirm that in all sub-menus tested (steps 5, 8, 13), the Back button label reads `[Z] Back`.

## Expected Experience
Regardless of faction or menu, the Back/Cancel button is always in the same bottom-left position of the command grid. Pressing Z consistently navigates back to the previous state. The player builds reliable muscle memory: Z = back, always. The button label `[Z] Back` is visually consistent across all sub-menus.
