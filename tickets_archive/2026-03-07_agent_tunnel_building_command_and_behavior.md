# Ticket: Agent Tunnel Building Command and Behavior

## Current State
The command-to-behavior pipeline has no BuildTunnel command or BuildingTunnel behavior. The Agent's ObjectInterfaceState has an AwaitingPlacement flow for Tunnel building (UI layer), and the Agent unit definition includes building stats (data layer), but the command pipeline cannot execute the build sequence. The ConstructionHP Rule exists in the entity system, but no behavior drives the Agent through the construction flow.

## Desired State
Add the BuildTunnel command and BuildingTunnel behavior to the command-to-behavior pipeline.

### Command

**BuildTunnel**
- CommandType: BuildTunnel
- TargetLocation: location (the validated placement location from AwaitingPlacement)
- TargetObject: None
- Availability: Agent only, issued via AwaitingPlacement (left-click valid location in Agent's ObjectInterfaceState)

### Behavior

**BuildingTunnel**
Execution sequence:
1. Agent moves to the target build location (MovingToLocation sub-behavior)
2. Construction begins — a partially-built Tunnel appears at the location, starting at **10% HP** (ConstructionHP Rule: `HP = MaxHP x 10%` = 60 HP for T1 Tunnel)
3. The Agent **embeds inside** the partially-built Tunnel and becomes **untargetable** for the duration of construction
4. HP increases linearly over 480 frames: `HP = MaxHP x (10% + 90% x construction_progress)` where `construction_progress` goes from 0.0 to 1.0
5. **If construction completes (480 frames)**: The Tunnel becomes fully operational. The Agent is placed inside the Tunnel Network (not on the surface), available for redeployment from any Tunnel.
6. **If the partially-built Tunnel is destroyed during construction**: The Agent survives and emerges at the Tunnel's location. The Tunnel is lost and any Supplies spent are not refunded.

### Constraints
- Only one Agent may construct a given Tunnel — multiple Agents cannot speed up construction
- The Agent must remain present for the full 480-frame build duration
- Construction cost follows the Tunnel cost scaling formula: cost = current number of Tunnels owned, in Supplies (see `features/syndicate_objects.md` Cost Scaling section)

## Justification
Specified in `features/unit_commands_and_behaviors.md` (BuildTunnel command row, BuildingTunnel behavior description) and `features/syndicate_objects.md` (Agent Building section, Tunnel Construction Flow). This is the execution-layer implementation that connects the Agent's Build Tunnel UI flow (AwaitingPlacement in the interface state) to actual game-world construction. Without this, the player can enter the placement UI but the Agent cannot execute the build.

## QA Steps
1. [semi] Select an Agent, press A (Build Tunnel), place a ghost on a valid location, and left-click. Verify the BuildTunnel command is issued and the Agent begins walking to the target location.
2. [semi] Verify a partially-built Tunnel appears at the build location when the Agent arrives, starting at 10% of MaxHP (60 HP for a T1 Tunnel with 600 MaxHP).
3. [auto] Verify the Agent becomes untargetable once embedded in the partially-built Tunnel.
4. [auto] Verify HP increases linearly during construction. At 50% construction progress (240 frames), HP should be `600 x (0.10 + 0.90 x 0.50)` = 330 HP.
5. [auto] Verify construction completes after exactly 480 frames and the Tunnel becomes fully operational (full HP, functional sides).
6. [auto] After construction completes, verify the Agent is inside the Tunnel Network (not visible on the surface) and can be ejected from any Tunnel.
7. [auto] During construction, destroy the partially-built Tunnel. Verify the Agent survives and appears at the Tunnel's former location.
8. [auto] Verify the surviving Agent is targetable again after emerging from a destroyed construction site.
9. [auto] Verify the Tunnel construction cost follows the scaling formula: 2nd Tunnel costs 1 Supply, 3rd costs 2 Supplies, etc.
10. [auto] Attempt to assign a second Agent to construct the same partially-built Tunnel. Verify this is rejected — only one Agent per construction.
11. [auto] Verify that non-Agent units cannot receive the BuildTunnel command.

## Expected Experience
The tunnel building sequence should feel weighty and committal: the Agent walks to the site, a foundation appears, and the Agent visually merges into the structure. The player sees the Tunnel's HP bar climbing steadily over 30 seconds. The Agent is safe inside but the partially-built Tunnel is vulnerable — if enemies destroy it, the Agent pops out alive but the investment is lost. When construction finishes, the Tunnel snaps to full operation and the Agent disappears into the network, ready to be redeployed from any Tunnel. The cost scaling means each additional Tunnel is more expensive, creating meaningful expansion decisions.
