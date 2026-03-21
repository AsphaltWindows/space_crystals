# enter_command_tunnel

## Metadata
- **From**: completion_aggregator
- **To**: qa_router

## Content

## Content

# enter-command-tunnel

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement the Enter command and EnteringTunnel behavior as defined in `artifacts/designer/design/control_system.md` (Enter command) and `artifacts/designer/design/syndicate_objects.md` (Tunnel Side A).

**Enter Command:**
- Order the unit to enter a Tunnel Network via a specific Tunnel
- The unit walks to Side A of the target Tunnel and enters the network
- Only available to Syndicate units when the target Tunnel's tier is sufficient for the unit's base category

**Sets BaseCommandState:**
- CommandType = Enter
- TargetLocation = None
- TargetObject = Tunnel (ObjectInstance)

**Transit Tier Requirements** (from syndicate_objects.md):
- Tier 1+: Infantry (Heavy Infantry)
- Tier 2+: Vehicles (Wheeled, Tracked, Drill, Hover Vehicle, Mech)
- Tier 3+: Air units (Hover Craft, Glider)

**EnteringTunnel Behavior:**
1. Unit receives Enter command targeting a Tunnel
2. Unit pathfinds and walks to Side A of the target Tunnel
3. On arrival at Side A: unit despawns from the map and enters the Tunnel Network
4. Unit is now available for ejection from any sufficiently-tiered Tunnel

**Right-click integration:**
- BasicCombatUnitInterfaceState: right-click own Tunnel (Syndicate units only, tier sufficient) → Enter command
- Agent: right-click own Tunnel (not carrying resources) → Enter command

## QA Instructions

1. Select a Syndicate infantry unit (Guard or Agent not carrying resources).
2. Right-click an own Tier 1 Tunnel.
3. Verify the unit walks to Side A of the Tunnel.
4. Verify the unit despawns from the map upon reaching Side A.
5. Select the Tunnel, open EjectMenu (C) — verify the unit appears in the network.
6. If vehicles exist: verify a vehicle cannot enter a Tier 1 Tunnel (command rejected or no action).
7. Upgrade Tunnel to Tier 2, then attempt vehicle entry — verify it works.
8. Verify an Agent carrying resources right-clicks a Tunnel → Drop Off (not Enter).
