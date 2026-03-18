# Feature Update: Syndicate Objects — Tunnel Z Back Hotkeys

**Date**: 2026-03-09
**Feature file**: `features/syndicate_objects.md`
**Design sources**: `design/syndicate_objects.md`, `design/control_system.md`

## Modifications

### Updated: Tunnel EjectMenu
Added **Z** as explicit back hotkey returning to DefaultState (StateOnlyTransition). Escape/right-click also still works. Aligns with standard Z (back) slot assignment.

### Updated: Tunnel ExpandMenu
Added **Z** as explicit back hotkey returning to DefaultState (StateOnlyTransition). Escape/right-click also still works.

### Updated: Tunnel AwaitingPlacement (Expansion)
Added **Z** as explicit back hotkey returning to ExpandMenu (StateOnlyTransition). Escape/right-click also still works.
