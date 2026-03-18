# Ticket: Agent ObjectInterfaceState

## Current State
The Agent unit has no ObjectInterfaceState defined. The command panel shows no Agent-specific commands when an Agent is selected. The Agent does not use BasicCombatUnitInterfaceState because its right-click resolution is more complex (resource-context-sensitive Tunnel interaction).

## Desired State
Implement the Agent's ObjectInterfaceState as a unique interface (not BasicCombatUnitInterfaceState). Since Agent is Ungroupable, the SelectionGroup always contains exactly one Agent instance, so the panel displays that Agent's interface state.

### DefaultState Commands
- **A: Build Tunnel** -- enters AwaitingPlacement for a Tunnel (StateOnlyTransition). Ghost preview follows cursor, snapped to grid. Tinted green when valid, red when invalid. R rotates 90 degrees clockwise, Shift+R counter-clockwise. F flips horizontally, Shift+F flips vertically. Left-click valid location confirms placement and dispatches the Agent to that location (CommandIssuingTransition, returns to DefaultState). Escape/right-click cancels back to DefaultState.
- **B: Drop Off Resources** -- targeted command (CommandIssuingTransition). Requires clicking an own Tunnel. Agent walks to the appropriate side automatically (Side B for crystals, Side C for supplies). Always visible, **greyed out when Agent is not carrying resources**.

### Unit Commands
Move, Stop, Attack, Enter, Gather, Drop Off Resources, Build Tunnel

### Right-Click Resolution
| Right-Click Target | Command Issued |
|---|---|
| Crystal field | Gather crystals |
| Supply source | Gather supplies |
| Own Tunnel (carrying resources) | Drop off resources (auto-routes to correct side) |
| Own Tunnel (not carrying resources) | Enter |
| Enemy unit/building | Attack (melee) |
| Ground | Move |

### Multi-Select Note
Despite being Ungroupable, right-click commands are issued to **all selected Agents** simultaneously (not just the ActiveGroup Agent).

## Justification
`features/syndicate_objects.md` (Agent ObjectInterfaceState section). The Agent's interface is distinct from BasicCombatUnitInterfaceState due to resource-context-sensitive right-click resolution (carry state determines Enter vs. Drop Off on own Tunnel) and resource-gathering targets (Crystal field -> Gather, Supply source -> Gather). Added in `feature_updates/2026-03-06_syndicate_objects_agent_control_panel.md`. Depends on the command panel and interface state machine (`command_panel_and_interface_state_machine` ticket) for the ObjectInterfaceState framework, and on Agent resource commands (Gather, DropOffResources, BuildTunnel) being implemented.

## QA Steps
1. Select an Agent. Verify the command panel shows two DefaultState commands: A (Build Tunnel) and B (Drop Off Resources).
2. Verify button B (Drop Off Resources) is greyed out when the Agent is not carrying resources.
3. Have the Agent pick up crystals from a Space Crystal Patch. Verify button B is now active (not greyed out).
4. Press A (or hotkey). Verify the interface enters AwaitingPlacement: a ghost Tunnel preview follows the cursor, snapped to grid, tinted green on valid placement and red on invalid.
5. While in AwaitingPlacement, press R. Verify the ghost rotates 90 degrees clockwise. Press Shift+R. Verify counter-clockwise rotation. Press F. Verify horizontal flip. Press Shift+F. Verify vertical flip.
6. Left-click a valid location. Verify the Agent is dispatched to build at that location (CommandIssuingTransition) and the interface returns to DefaultState.
7. Enter AwaitingPlacement again, then press Escape. Verify the interface returns to DefaultState without issuing a command.
8. Right-click a Space Crystal Patch. Verify the Gather crystals command is issued (Agent walks to the patch and begins mining).
9. Right-click a Supply Delivery Station. Verify the Gather supplies command is issued.
10. Have the Agent carry crystals. Right-click an own Tunnel. Verify the Drop Off Resources command is issued (Agent walks to Tunnel Side B).
11. Have the Agent carry supplies. Right-click an own Tunnel. Verify the Drop Off Resources command is issued (Agent walks to Tunnel Side C).
12. Have the Agent carry nothing. Right-click an own Tunnel. Verify the Enter command is issued (Agent walks to Side A and enters the Tunnel Network).
13. Right-click an enemy unit. Verify the Attack command is issued (Agent moves to melee range and attacks).
14. Right-click empty ground. Verify the Move command is issued.
15. Select multiple Agents. Right-click on ground. Verify all selected Agents receive the Move command, not just the ActiveGroup Agent.
16. Select multiple Agents. Right-click a Crystal field. Verify all selected Agents receive the Gather command.

## Expected Experience
The Agent's command panel should feel distinct from standard combat units. The two buttons (Build Tunnel and Drop Off Resources) should clearly indicate the Agent's worker role. The Drop Off button should visually grey out when the Agent has no resources, providing clear feedback on carry state. The AwaitingPlacement flow for Tunnel building should match the existing building placement UX (ghost preview, rotation, flipping, green/red tinting). Right-clicking should feel context-aware: the Agent automatically does the "right thing" based on what was clicked and whether it's carrying resources. Clicking an own Tunnel should intelligently choose Enter vs. Drop Off based on carry state, without the player needing to explicitly pick the command.
