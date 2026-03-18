# Ticket: Fix Click Offset Regression — Clicks Register ~2 Grid Spaces Above Cursor

## Current State
All click interactions (selection, commands, etc.) register approximately 2 grid spaces above where the mouse cursor is actually pointing. This is a regression of a previously-fixed bug (see `completed_tasks/2026-03-06_fix_selection_click_offset.md`).

The original fix introduced a `cursor_pos_in_viewport()` utility function to adjust cursor coordinates from window-space to viewport-space, accounting for the top HUD bar's viewport offset. The regression means either:
1. New code paths are using raw `window.cursor_position()` instead of the adjusted utility
2. The viewport offset calculation has changed (e.g., camera/viewport setup modified)
3. The utility function itself was broken by a refactor

## Desired State
Clicking on any game entity (units, buildings, resources, ground) should register exactly where the mouse cursor is pointing. All systems using cursor position with the 3D camera should use viewport-adjusted coordinates.

## Justification
The control system (`features/control_system.md`) defines selection as the core interaction. If clicks don't land where the cursor points, the game is unplayable. This is a critical regression of a previously-verified fix.

## QA Steps
1. [human] Click directly on a unit's visual center — selection should register immediately.
2. [human] Click ~2 grid spaces above a unit — should NOT select that unit (this was the old broken behavior).
3. [human] Right-click on the ground to issue a move command — the destination marker should appear exactly where clicked.
4. [human] Test at different zoom levels — the click accuracy should be consistent regardless of zoom.
5. [human] Test box/drag selection — the selection rectangle should start and end exactly at the cursor position.
6. [human] Test building placement — the placement preview and final position should match cursor location.
7. [human] Verify the fix works at different window sizes/resolutions.

## Expected Experience
Clicking anywhere on the game map should feel precise and responsive. Units should be selected when the cursor is visually over them. Move commands should send units to the exact clicked location. There should be zero perceptible offset between where the cursor appears and where clicks are interpreted.
