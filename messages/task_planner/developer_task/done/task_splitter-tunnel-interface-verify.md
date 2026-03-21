# tunnel-interface-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-tunnel-interface.md

## Task

**Verification task — the Tunnel ObjectInterfaceState is already fully implemented.**

Verify that the existing implementation matches the design spec in `artifacts/designer/design/syndicate_objects.md` under 'Tunnel ObjectInterfaceState'. All four interface states and their transitions are already in place:

1. **DefaultState (TunnelIdle)** in `command_panel.rs`: Upgrade (A/Q slot 0,0), Expand (B/W slot 0,1), Eject (C/E slot 0,2), Cancel Upgrade (X slot 2,1 — conditional on tunnel_is_upgrading). See `StructureMenuState::TunnelIdle` match arm in `get_grid_slot_action`.

2. **ExpandMenu (TunnelExpandMenu)**: Dynamic grid built by `build_tunnel_expand_grid()` showing expansion types filtered by tunnel tier. Back (Z) at slot 2,0. Clicking an expansion enters AwaitingPlacement via `execute_tunnel_select_expansion()`.

3. **EjectMenu (TunnelEjectMenu)**: Dynamic grid built by `build_tunnel_eject_grid()` showing unit types in the Tunnel Network with counts. Units whose UnitBase exceeds tunnel tier are greyed out via `TunnelTier::can_transit()`. Back (Z) at slot 2,0. Clicking ejects via `execute_tunnel_eject_unit()` which inserts `EjectRequest`.

4. **AwaitingPlacement (TunnelAwaitingPlacement)**: Ghost preview system in `faction.rs` handles cursor tracking, grid snapping, green/red tinting, R/Shift+R rotation, F/Shift+F flipping, left-click placement, Z returns to ExpandMenu.

5. **Upgrade/Cancel systems**: `execute_tunnel_upgrade()` deducts supplies per cost formula. `execute_tunnel_cancel_upgrade()` refunds full cost. Both in `command_panel.rs`.

6. **Ejection system**: `ejection_tick_system()` in `faction.rs` processes `EjectRequest` markers, manages `EjectionQueue` with 8-frame cooldown, spawns units at Side A position.

Run `cargo test` to confirm all existing tests pass. Review the test suite (search for `tunnel_idle`, `tunnel_expand`, `tunnel_eject`, `tunnel_awaiting_placement`, `ejection_queue`, `tunnel_cancel_upgrade`) and confirm coverage of the QA-specified scenarios. If any gaps exist between the spec and implementation, fix them.
