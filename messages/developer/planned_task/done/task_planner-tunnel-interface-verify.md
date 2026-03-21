# tunnel-interface-verify

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Files to Verify

1. **`artifacts/developer/src/ui/command_panel.rs`** — Primary implementation file containing:
   - `get_grid_slot_action()` (line ~112): `StructureMenuState::TunnelIdle` match arm — maps (0,0)=Upgrade, (0,1)=Expand, (0,2)=Eject, (2,1)=CancelUpgrade (conditional on `tunnel_is_upgrading`)
   - `StructureMenuState::TunnelExpandMenu` match arm (line ~119): (0,0)=Headquarters, (2,0)=Back
   - `StructureMenuState::TunnelEjectMenu` match arm (line ~127): (2,0)=Back only (eject actions are dynamic via `build_tunnel_eject_grid`)
   - `StructureMenuState::TunnelAwaitingPlacement` (line ~132): Returns None for all grid slots (placement handled by mouse)
   - `execute_tunnel_upgrade()` (line ~1599): Validates tier, computes cost via `tunnel_t2_upgrade_cost()`/`tunnel_t3_upgrade_cost()`, deducts supplies, sets `TunnelOperation::Upgrading`
   - `execute_tunnel_cancel_upgrade()` (line ~1663): Mirrors upgrade cost logic for refund, clears `current_operation`
   - `execute_tunnel_select_expansion()` (line ~1717): Sets `PlacementState`, transitions to `TunnelAwaitingPlacement`
   - `execute_tunnel_eject_unit()` (line ~1759): Inserts `EjectRequest` component on tunnel entity
   - `build_tunnel_expand_grid()` (line ~1771): Dynamic grid showing expansions filtered by tier
   - `build_tunnel_eject_grid()` (line ~1832): Dynamic grid showing network unit types with counts, grey out by `TunnelTier::can_transit()`

2. **`artifacts/developer/src/game/world/faction.rs`** — Placement and ejection systems:
   - `ejection_tick_system()` (line ~1816): Processes `EjectRequest` markers, manages `EjectionQueue` with 8-frame cooldown, spawns units at Side A via `tunnel_side_world_position()`
   - Ghost preview / placement system for `TunnelAwaitingPlacement` (line ~1071): Cursor tracking, grid snapping, green/red tinting, rotation (R/Shift+R), flipping (F/Shift+F), left-click placement
   - Escape/Z handler returns to `TunnelExpandMenu` from `TunnelAwaitingPlacement` (line ~1310, ~1432)

3. **`artifacts/developer/src/ui/types.rs`** — Type definitions:
   - `StructureMenuState` enum: `TunnelIdle`, `TunnelExpandMenu`, `TunnelEjectMenu`, `TunnelAwaitingPlacement` (line ~221)
   - `EjectRequest` component (line ~397) — marker processed by `ejection_tick_system`
   - `EjectionQueue` component — `queue: VecDeque<Entity>`, `cooldown: u32`
   - `ObjectInterfaceState::is_placement_mode()` includes `TunnelAwaitingPlacement` (line ~169)

4. **`artifacts/developer/src/game/world/mod.rs`** (line ~101) — `ejection_tick_system` registered in plugin

### Design Spec Reference

- `artifacts/designer/design/syndicate_objects.md` lines 49-73: Tunnel ObjectInterfaceState spec
- DefaultState commands: A=Upgrade, B=Expand, C=Eject, X=Cancel Upgrade (conditional)
- EjectMenu: unit type grid with counts, greyed out if tier insufficient, Z=Back
- ExpandMenu: expansion types filtered by tier, Z=Back
- AwaitingPlacement: ghost preview, R/Shift+R rotate, F/Shift+F flip, left-click place, Z=Back to ExpandMenu
- Upgrade costs: T2 = 2 + 2*(T2+ count), T3 = 3 + 3*(T3 count) — full refund on cancel

### Existing Test Coverage (all in `command_panel.rs` test module, line ~2752+)

Tests already present — verify these all pass with `cargo test`:
- `tunnel_idle_shows_three_commands` — Upgrade/Expand/Eject at correct slots
- `tunnel_idle_remaining_slots_are_none` — Empty slots return None
- `tunnel_idle_cancel_upgrade_shown_when_upgrading` / `..hidden_when_not_upgrading` — Conditional X slot
- `tunnel_cancel_upgrade_label` / `tunnel_cancel_upgrade_is_not_common_command`
- `tunnel_expand_menu_headquarters_at_0_0` / `..back_at_2_0` / `..empty_slots_return_none`
- `tunnel_eject_menu_back_at_2_0` / `..other_slots_return_none`
- `tunnel_awaiting_placement_no_grid_buttons` / `..is_placement_mode`
- `tunnel_idle_is_not_placement_mode` / `tunnel_expand_menu_is_not_placement_mode`
- `tunnel_upgrade_label` / `tunnel_expand_label` / `tunnel_eject_label` / `tunnel_select_expansion_label`
- `ejection_queue_default_is_empty` / `..can_push_and_pop` / `..cooldown_tracking`
- `panel_visible_when_tunnel_idle` / `..expand_menu` / `..eject_menu` / `..awaiting_placement`
- `tunnel_actions_are_not_common_commands`

### Potential Gaps to Check

1. The **EjectMenu grid slot test** (`tunnel_eject_menu_other_slots_return_none`) only tests the static `get_grid_slot_action()` which returns None for non-Back slots. The dynamic `build_tunnel_eject_grid()` builds actual UI buttons — no unit test verifies that greyed-out logic (`can_transit`) works correctly for eject buttons. Consider whether a unit test for the grey-out filtering is needed.
2. No integration test for `execute_tunnel_upgrade` cost deduction or `execute_tunnel_cancel_upgrade` refund — these are complex functions with tier-counting logic. Consider adding targeted unit tests.
3. No test for `ejection_tick_system` as a Bevy system — only `EjectionQueue` data structure tests exist. The actual system logic (find matching unit, teleport to Side A, set cooldown) is untested.

## Dependencies

None — this is a standalone verification task. All referenced systems are already implemented.
