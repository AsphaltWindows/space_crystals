# 0011 — Default input bindings; key rebinding

- **Date**: 2026-11-23T13:38:36-05:00
- **Answers**: Q0002 (completes it, together with 0010)
- **Docs**: `systems/controls/selection_and_hud.md > MapClickSelection` (new), `> ControlGroups > Default keys` (new), `> Key Rebinding` (new); `systems/controls/commands.md > Command` (Shift-queue rule); `core/camera.md > Panning > Arrow keys and screen edge`

## What changed and why
The user confirmed the Director's proposed defaults (option A):
- **Map clicks:** Shift-click toggles an own unit in the selection. Ctrl-click or double-click selects all own units of that type on screen. Left-click replaces the selection. Selection Constraints still apply.
- **Control groups:** Ctrl+# assigns, Shift+# adds, # recalls, and a double-tap of # recalls and centers.
- **Camera:** arrow keys and screen-edge scrolling pan at a fixed speed (value TBD), in addition to the middle-mouse grab (0010).
- **Queueing:** holding Shift while issuing a command appends it. Without Shift, the command replaces the queue.
- **Rebinding:** all of these, plus the command-panel hotkeys, are defaults. Players will be able to rebind them. **When** to build rebinding is the Director's call.

## Implementation impact
Matches the Director's assumption; nothing to undo. Keep the bindings in one rebindable table so rebinding can be added later without touching gameplay code.

## Verification
- Shift-click a selected unit: it's removed. Shift-click an unselected own unit: it's added.
- Double-click a Peacekeeper: all Peacekeepers on screen are selected, and none off-screen.
- Ctrl+1 with units selected, click elsewhere, press 1: the group returns. Press 1 twice quickly: the camera centers on it. Shift+1 with another unit: that unit is added to group 1.
- Give a move, then Shift+right-click two more points: the unit visits all three in order. A plain right-click afterwards replaces them.
- Arrow keys and pushing the cursor to an edge pan the camera.
