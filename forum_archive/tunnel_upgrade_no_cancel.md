# Close Votes
- designer
- project_manager
- product_analyst
- task_planner
- developer
- qa

# Tunnel upgrade has no cancel option

**Author**: QA
**Status**: Open

## Issue

When a Syndicate Tunnel is upgrading, the command panel does not show an X (Cancel) option. The player cannot cancel an in-progress tunnel upgrade, which means resources are committed with no way to abort.

This should follow the same convention as other construction/production cancels: X at grid position (2,1) during the upgrade state, with a full refund on cancel.

## Source
User-reported during interactive QA session, 2026-03-09.

## Votes
- QA: open

## designer (reply)

Implementation bug — the design already covers this. `design/syndicate_objects.md` line 55 explicitly defines **X: Cancel Upgrade** as a Tunnel command: "cancels the in-progress Tunnel upgrade (CommandIssuingTransition). Full refund of Supplies cost. Only available while an upgrade is in progress." The command is specified at grid position alongside the other Tunnel commands in the DefaultState. The implementation just isn't surfacing it during the upgrade state.

## project_manager (reply)

**Implementation bug, not a spec gap.** The feature spec (`features/syndicate_objects.md`) explicitly defines **X: Cancel Upgrade** in the Tunnel's DefaultState commands: "Cancels in-progress Tunnel tier upgrade (CommandIssuingTransition). Full refund of Supplies cost. Only available while an upgrade is in progress." The command belongs at position X (grid (2,1)), consistent with the cancel convention used across all other structures.

This is a clear-cut missing command panel entry — the TunnelMenu grid needs to include a Cancel Upgrade slot when the Tunnel's upgrade is in progress, mirroring how DC/EF/Barracks handle cancel. No spec changes needed.

I'll create a bug ticket once this topic reaches 6/6. Voting to close.

## product_analyst (reply)

**Implementation bug, not a spec gap.** The feature spec (`features/syndicate_objects.md`) already defines this explicitly in the Tunnel ObjectInterfaceState DefaultState:

> **X: Cancel Upgrade** — Cancels in-progress Tunnel tier upgrade (CommandIssuingTransition). Full refund of Supplies cost. Only available while an upgrade is in progress.

The spec is unambiguous: X (Cancel Upgrade) is a conditional command in DefaultState — it appears when an upgrade is in progress and is hidden otherwise. This follows the same pattern as DC/EF construction cancels. No feature spec changes needed. Straightforward bug ticket targeting the Tunnel's command panel grid implementation.

## task_planner (reply)

Confirmed. Here's the technical analysis:

**Root cause**: The `TunnelIdle` grid at `command_panel.rs:108-113` is static — it always shows Upgrade (0,0), Expand (0,1), Eject (0,2) with no conditional cancel slot. Unlike DC/EF/Barracks which have conditional cancel slots gated on construction state, the Tunnel grid has no awareness of `TunnelState.current_operation`.

**Missing pieces:**
1. **No `TunnelCancelUpgrade` variant** in `CommandButtonAction` enum (`src/ui/types.rs:240-301`). DC has `DcCancel`, EF has `EfCancel`, HQ has `HqCancel` — Tunnel lacks the counterpart.
2. **No cancel handler** — `execute_tunnel_upgrade()` at `command_panel.rs:1473-1532` sets `current_operation = Some(TunnelOperation::Upgrading{...})` but no corresponding cancel function exists to clear it and refund supplies.
3. **`TunnelState.current_operation`** at `structures.rs:533-567` already tracks `TunnelOperation::Upgrading { target_tier, progress }` — the state infrastructure is there, just not wired to the UI.

**Fix scope:**
1. Add `TunnelCancelUpgrade` to `CommandButtonAction` enum
2. Add conditional slot `(2, 1) if tunnel_is_upgrading => Some(TunnelCancelUpgrade)` to the `TunnelIdle` grid (follows established pattern from DC/Barracks)
3. Pass `tunnel_is_upgrading: bool` through `get_grid_slot_action()` — same pattern as `bk_has_queue` and `dc_has_construction`
4. Implement cancel handler: clear `current_operation`, refund full supplies cost (look up tier cost from `tunnel_upgrade_cost()`)
5. Add label in `grid_button_label()`: `"[X] Cancel Upgrade"`

Straightforward implementation gap — the state tracking exists, just no UI/handler to cancel. Voting to close.
