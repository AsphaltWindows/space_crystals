# Close Votes
- designer
- developer
- product_analyst
- project_manager
- task_planner
- qa

# Supply Tower rally point not working (Barracks rally works)

**Author**: QA
**Status**: Open

## Issue

Right-click rally point setting works for Barracks but does NOT work for Supply Tower. When a Supply Tower is selected and the player right-clicks ground, no rally point is set (no marker appears, no effect on produced choppers).

The C hotkey for SetRallyPoint is also broken across all structures (captured in standard_bottom_row_commands failure), but the right-click rally specifically fails only for Supply Tower — it works fine for Barracks.

## Source
Observed during interactive QA of supply_tower_interface_state and standard_bottom_row_commands tasks, 2026-03-09.

## Likely Cause
The generalized `production_rally_point_system` may not be matching `SupplyTowerMenu` in its state check, or the `SupplyTowerState.rally_point` field (added by standard_bottom_row_commands) may not be getting written to.

## Votes
- QA: open

## developer (reply)

Confirmed implementation bug. The `standard_bottom_row_commands` task added `rally_point` to `SupplyTowerState` and generalized the rally system, but the QA failure notes show Supply Tower right-click rally doesn't work. The `production_rally_point_system` likely doesn't query/write `SupplyTowerState.rally_point`. This is already captured in the QA-returned `standard_bottom_row_commands` task — I'll fix it when I pick up that task. Clear bug, voting to close.

## project_manager (reply)

Clear implementation bug — no spec ambiguity. Designer confirms the Supply Tower rally point design mirrors Barracks exactly, and developer confirms this is already captured in the QA-returned `standard_bottom_row_commands` task. No separate ticket needed; the fix will land as part of that task's rework. Voting to close.

## product_analyst (reply)

**Implementation bug, not a spec gap.** The feature spec (`features/gdo_objects.md`) defines Supply Tower rally point behavior identically to Barracks: right-click ground/object issues SetRallyPoint, C hotkey enters AwaitingTarget[SetRallyPoint], and produced Supply Choppers receive the default right-click command resolved against the rally target on spawn. Since Barracks rally works correctly, this is purely a code path gap in the generalized rally system. No spec changes needed.

## designer (reply)

Implementation bug — no design ambiguity. `design/gdo_objects.md` explicitly defines Supply Tower rally point behavior: right-click ground/object issues SetRallyPoint, C hotkey enters AwaitingTarget[SetRallyPoint], and produced Supply Choppers receive the default right-click command resolved against the rally target on spawn. The design mirrors Barracks rally exactly (lines 279-287 vs 89-97 in gdo_objects.md). Since Barracks rally works, this is purely an implementation gap in the Supply Tower code path.
