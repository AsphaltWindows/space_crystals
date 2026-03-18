# Ticket: Tunnel Upgrade Missing Cancel Option

## Current State
When a Syndicate Tunnel is upgrading, the command panel does not show an X (Cancel) option. Players cannot cancel an in-progress tunnel upgrade, meaning resources are permanently committed once an upgrade begins. The `TunnelIdle` grid at `command_panel.rs:108-113` is static with no conditional cancel slot. There is no `TunnelCancelUpgrade` variant in `CommandButtonAction`, and no cancel handler exists to clear `TunnelState.current_operation` and refund supplies.

## Desired State
The Tunnel command panel should show an X: Cancel Upgrade button at grid position (2,1) when an upgrade is in progress, following the same pattern as DC/EF/Barracks cancel commands. Pressing it should cancel the upgrade, clear `current_operation`, and provide a full refund of the Supplies cost.

Specific implementation:
1. Add `TunnelCancelUpgrade` to `CommandButtonAction` enum in `src/ui/types.rs`
2. Add conditional slot `(2, 1) if tunnel_is_upgrading => Some(TunnelCancelUpgrade)` to the `TunnelIdle` grid
3. Pass `tunnel_is_upgrading: bool` through `get_grid_slot_action()` (same pattern as `bk_has_queue`, `dc_has_construction`)
4. Implement cancel handler: clear `current_operation`, refund full supplies cost via `tunnel_upgrade_cost()`
5. Add label in `grid_button_label()`: `"[X] Cancel Upgrade"`

## Justification
The feature spec (`features/syndicate_objects.md`) explicitly defines **X: Cancel Upgrade** in the Tunnel's DefaultState commands: "Cancels in-progress Tunnel tier upgrade (CommandIssuingTransition). Full refund of Supplies cost. Only available while an upgrade is in progress." The design doc (`design/syndicate_objects.md`) also specifies this command at grid position alongside other Tunnel commands. Bug confirmed in forum topic `tunnel_upgrade_no_cancel.md` (archived, 6/6 close votes).

## QA Steps
1. [human] Start a Syndicate game and select a Tunnel
2. [human] Initiate a tier upgrade on the Tunnel
3. [human] Verify the command panel now shows an X: Cancel Upgrade button at position (2,1)
4. [human] Note the current Supplies amount, then press X to cancel the upgrade
5. [human] Verify the upgrade is cancelled (progress stops, Tunnel returns to its previous tier)
6. [human] Verify the full Supplies cost is refunded (Supplies amount returns to pre-upgrade value)
7. [human] Verify the Cancel Upgrade button disappears when no upgrade is in progress

## Expected Experience
When a Tunnel upgrade is initiated, an X: Cancel Upgrade button should appear in the command panel. Pressing it should immediately halt the upgrade, revert the Tunnel to its previous state, and refund the full Supplies cost. The button should only be visible while an upgrade is actively in progress and should disappear once the upgrade completes or is cancelled.
