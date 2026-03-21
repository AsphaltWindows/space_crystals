# command-panel-rightclick-cancel

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-command-panel-framework.md

## Task

Add right-click cancel (back to previous state) for all multi-stage ObjectInterfaceState states that currently only support Escape cancel. The command panel framework is already fully implemented (3x3 grid, hotkeys, Z=Back, X=Cancel, C=Rally, CommonCommands vs GroupCommands visual distinction, Tab group cycling, Escape cancel). The only gap is QA step 6: right-click from a sub-menu should behave the same as Z/Escape.

**What needs to be added:**

A system (or addition to an existing system) that handles right-click when the ObjectInterfaceState is in a non-Default, non-placement multi-stage state, resetting it to the previous state — same transitions as Escape in `command_panel_hotkeys` (lines ~867-907 of command_panel.rs).

Specifically, right-click should cancel from these states (matching Escape behavior):
- `StructureMenu(DcBuildMenu)` → `StructureMenu(DcIdle)`
- `StructureMenu(DcConstructing)` → `StructureMenu(DcIdle)`
- `StructureMenu(DcReadyToPlace)` → `StructureMenu(DcIdle)`
- `StructureMenu(TunnelExpandMenu)` → `StructureMenu(TunnelIdle)`
- `StructureMenu(TunnelEjectMenu)` → `StructureMenu(TunnelIdle)`
- `StructureMenu(EfConstructing)` → `StructureMenu(EfIdle)`
- `StructureMenu(EfReadyToPlace)` → `StructureMenu(EfIdle)`
- `AwaitingTarget(SetRallyPoint)` → appropriate previous state (BarracksMenu/HeadquartersMenu/SupplyTowerMenu)
- `AwaitingTarget(ScheduleDeliveries)` → `StructureMenu(SupplyTowerMenu)`

**Already handled (do NOT duplicate):**
- Placement modes (`DcAwaitingPlacement`, `EfAwaitingPlacement`, `TunnelAwaitingPlacement`, `AgentAwaitingPlacement`) — right-click cancel is already in `faction.rs` ~line 1301
- Regular unit AwaitingTarget modes (Move, Attack, Patrol, etc.) — right-click in `right_click_move_command` issues a command which resets state

**Key files:**
- `artifacts/developer/src/ui/command_panel.rs` — see `command_panel_hotkeys` for Escape transitions to replicate
- `artifacts/developer/src/game/units/systems/core.rs` — `right_click_move_command` and `set_rally_point_click_system`
- `artifacts/developer/src/game/world/faction.rs` — existing right-click cancel for placement modes (~line 1301)

**Implementation approach:** Add right-click handling to either an existing system or a new focused system. Check `MouseButton::Right` just_pressed, match against the multi-stage states listed above, and apply the same state transitions as Escape. Must not conflict with `production_rally_point_system` (which only fires from BarracksMenu/HeadquartersMenu/SupplyTowerMenu DefaultState) or the placement cancel in faction.rs.
