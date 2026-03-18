# Ticket: Tunnel ObjectInterfaceState

## Current State
The Tunnel structure is defined as Ungroupable (each Tunnel is its own SelectionGroup), but there is no ObjectInterfaceState implementing the Tunnel's command interface. The control system framework (`features/control_system.md`) defines ObjectInterfaceState as the per-type command panel, but no concrete implementation exists for Tunnels.

## Desired State
Implement a full ObjectInterfaceState for the Tunnel structure with 4 interface states:

### DefaultState
3 commands in the CommandPanel:
- **A: Upgrade Tunnel** — CommandIssuingTransition. Upgrades the Tunnel to the next tier. Costs Supplies per the upgrade cost formula. Unavailable if already Tier 3 or if the Tunnel is currently performing an operation (construction or upgrade).
- **B: Expand Tunnel** — StateOnlyTransition to ExpandMenu. Multi-stage: select an underground expansion type, then place it within the Tunnel Area.
- **C: Eject** — StateOnlyTransition to EjectMenu. Multi-stage: select units from the Tunnel Network to eject from this Tunnel.

### EjectMenu
Displays a grid of unit type tiles representing all units currently in the Tunnel Network (not just this Tunnel). Each tile shows the unit type icon and a count of that type in the network.
- Unit types whose base category exceeds this Tunnel's tier are visible but greyed out (disabled).
- Click an enabled unit type tile: ejects one unit of that type from this Tunnel's Side A (CommandIssuingTransition). Ejected units are queued — a new unit begins ejecting every **8 frames minimum** (0.5 seconds). Actual throughput is limited by unit speed and collision at Side A. Standard movement and collision mechanics apply as units emerge.
- Escape/right-click: returns to DefaultState (StateOnlyTransition).

### ExpandMenu
Displays available underground expansion types for this Tunnel's current tier. Only expansions at or below the Tunnel's tier are available.
- Click only works if the Tunnel is not already performing an operation (no concurrent construction/upgrade).
- Click an expansion type: enters AwaitingPlacement for that expansion.
- Escape/right-click: returns to DefaultState (StateOnlyTransition).

### AwaitingPlacement (Expansion)
- Ghost preview of the expansion follows cursor within the Tunnel Area, snapped to grid. Tinted green when valid placement, red when invalid.
- Expansion must fit entirely within the Tunnel Area.
- R rotates 90 degrees clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically.
- Left-click valid location: places expansion, begins construction (CommandIssuingTransition, returns to DefaultState).
- Escape/right-click: returns to ExpandMenu (StateOnlyTransition).

## Justification
The Tunnel ObjectInterfaceState is the primary way the player interacts with Syndicate infrastructure — upgrading Tunnels, building underground expansions, and deploying units from the network. Without this interface, the Tunnel is a passive structure with no player interaction. Specified in `features/syndicate_objects.md` (Tunnel ObjectInterfaceState section, lines 56-81). Depends on `tunnel_structure_and_network` (Tunnel definition, tier system), `tunnel_area_and_construction_rules` (Tunnel Area, placement validation), `tunnel_expansions_and_starting_condition` (expansion types), and `command_panel_and_interface_state_machine` (ObjectInterfaceState framework).

## QA Steps
1. Select a Tunnel — verify the CommandPanel shows 3 commands: Upgrade Tunnel (A), Expand Tunnel (B), Eject (C)
2. With a Tier 1 Tunnel, click Upgrade Tunnel — verify the Tunnel begins upgrading to Tier 2 and the correct Supply cost is deducted
3. While the Tunnel is upgrading, verify Upgrade Tunnel and Expand Tunnel commands are unavailable (one operation at a time)
4. With a Tier 3 Tunnel, verify Upgrade Tunnel is unavailable (already max tier)
5. Click Expand Tunnel — verify the ExpandMenu appears showing available expansion types for the current tier
6. In ExpandMenu, verify only expansions at or below the Tunnel's tier are shown as available
7. Click an expansion type — verify AwaitingPlacement activates with a ghost preview following the cursor
8. Move cursor within the Tunnel Area — verify the ghost preview snaps to grid, shows green on valid cells and red on invalid cells
9. Move cursor outside the Tunnel Area — verify the ghost shows red (expansion must fit entirely within the area)
10. Press R — verify ghost rotates 90 degrees clockwise. Press Shift+R — verify counter-clockwise rotation. Press F — verify horizontal flip. Press Shift+F — verify vertical flip.
11. Left-click a valid location — verify the expansion is placed and construction begins. Interface returns to DefaultState.
12. Press Escape in AwaitingPlacement — verify return to ExpandMenu. Press Escape in ExpandMenu — verify return to DefaultState.
13. Click Eject — verify the EjectMenu appears showing a grid of unit type tiles with counts from the entire Tunnel Network
14. Verify unit types whose base category exceeds this Tunnel's tier are visible but greyed out (cannot be clicked)
15. Click an enabled unit type tile — verify one unit of that type ejects from Side A
16. Click multiple unit type tiles rapidly — verify ejection queue processes at 8 frames minimum between ejections
17. Press Escape in EjectMenu — verify return to DefaultState
18. Right-click at any submenu level — verify it returns to the parent state (EjectMenu/ExpandMenu to DefaultState, AwaitingPlacement to ExpandMenu)

## Expected Experience
- Selecting a Tunnel shows a clear 3-button command panel at the bottom of the screen
- Upgrade Tunnel provides immediate feedback (cost deducted, operation begins, command becomes unavailable during upgrade)
- The Expand flow is a natural 3-step progression: select type from menu, position ghost preview on grid, click to place. Ghost preview clearly communicates valid vs invalid positions with green/red tinting.
- The Eject flow shows all network units at a glance with counts. Greyed-out tiles make tier restrictions obvious. Units emerge from Side A one at a time with a visible delay between each.
- Escape and right-click always provide a way back to the previous state, creating a comfortable navigation feel.
