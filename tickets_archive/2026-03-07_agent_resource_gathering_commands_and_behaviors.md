# Ticket: Agent Resource Gathering Commands and Behaviors

## Current State
The command-to-behavior pipeline supports 9 commands and 10 behaviors. The Agent unit's data definition exists (stats, gathering attributes, carry capacities) but no Gather or DropOffResources commands exist in the command table, and no GatheringResource or DroppingOffResources behaviors exist in the behavior system. Agents cannot actually gather resources or deliver them to Tunnels through the command pipeline.

## Desired State
Add two new commands and two new behaviors to the command-to-behavior pipeline for Agent resource gathering.

### Commands

**Gather**
- CommandType: Gather
- TargetLocation: None
- TargetObject: Resource source (ObjectInstance) — Space Crystal Patch or Supply Delivery Station
- Availability: Agent (resource-gathering units) only

**DropOffResources**
- CommandType: DropOffResources
- TargetLocation: None
- TargetObject: Own Tunnel (ObjectInstance)
- Availability: Agent only, when carrying resources (greyed out otherwise per Agent ObjectInterfaceState)

### Behaviors

**GatheringResource**
Agent moves to the target resource source (MovingToObject sub-behavior), then performs resource extraction:
- **Space Crystals**: MiningDuration of 48 frames at Space Crystal Patch, picks up 50 SC
- **Supplies**: PickUpDuration of 48 frames at Supply Delivery Station, picks up 1 Supply

After extraction, the Agent **automatically** moves to the nearest own Tunnel's appropriate side:
- Side B for crystals
- Side C for supplies

Then performs drop-off (DropOffDuration: 48 frames for both types). One Agent at a time per drop-off side.

The behavior encompasses the full gather-deliver cycle: approach resource -> extract -> travel to Tunnel -> drop off.

**DroppingOffResources**
Agent moves to the target Tunnel's appropriate side based on carried resource type:
- Side B for crystals
- Side C for supplies

Then performs drop-off (DropOffDuration: 48 frames). The behavior auto-routes to the correct side — the player only needs to target the Tunnel, not a specific side.

### Interaction Between Commands
- Gather command triggers GatheringResource behavior (full cycle including auto-delivery)
- DropOffResources command triggers DroppingOffResources behavior (explicit delivery without gathering first)
- The GatheringResource behavior's auto-delivery phase effectively contains DroppingOffResources logic internally

## Justification
These commands and behaviors are specified in `features/unit_commands_and_behaviors.md` (command table rows for Gather and DropOffResources, behavior descriptions for GatheringResource and DroppingOffResources). The Agent's gathering stats are defined in `features/syndicate_objects.md` (Agent Gathering section). Without these behaviors implemented in the command pipeline, the Agent unit definition and UI panel exist but cannot actually perform resource gathering. This is the execution-layer counterpart to the data layer (agent unit ticket) and the UI layer (agent interface state ticket).

## QA Steps
1. [semi] Select an Agent and right-click a Space Crystal Patch. Verify the Gather command is issued and the Agent walks toward the patch.
2. [auto] Verify the Agent performs mining for exactly 48 frames upon reaching the Space Crystal Patch.
3. [auto] Verify the Agent picks up 50 Space Crystals after mining completes.
4. [semi] After mining, verify the Agent automatically moves to the nearest own Tunnel's Side B (crystal drop-off side) without player input.
5. [auto] Verify the drop-off takes exactly 48 frames at Side B.
6. [auto] Verify 50 Space Crystals are added to the player's crystal count after drop-off completes.
7. [semi] Select an Agent and right-click a Supply Delivery Station. Verify the Agent walks to the station and performs pickup for 48 frames.
8. [auto] Verify the Agent picks up 1 Supply after pickup completes.
9. [semi] After pickup, verify the Agent automatically moves to the nearest own Tunnel's Side C (supply drop-off side).
10. [auto] Verify the drop-off takes exactly 48 frames at Side C and 1 Supply is added to the player's supply count.
11. [auto] Send two Agents to drop off crystals at the same Tunnel Side B simultaneously. Verify only one Agent drops off at a time — the second waits.
12. [auto] Send one Agent to drop off crystals (Side B) and another to drop off supplies (Side C) at the same Tunnel. Verify both can drop off simultaneously (separate sides).
13. [semi] Select an Agent carrying crystals. Issue an explicit DropOffResources command targeting an own Tunnel. Verify the Agent walks to Side B and drops off.
14. [semi] Select an Agent carrying supplies. Issue an explicit DropOffResources command targeting an own Tunnel. Verify the Agent walks to Side C and drops off.
15. [auto] Verify the DropOffResources command is unavailable (greyed out) when the Agent is not carrying resources.
16. [auto] Verify that non-Agent units cannot receive the Gather or DropOffResources commands.

## Expected Experience
The resource gathering loop should feel smooth and automated: the player right-clicks a resource source, and the Agent handles the full gather-deliver cycle without further input. After mining/picking up resources, the Agent automatically finds the nearest own Tunnel and routes to the correct side. Crystal deliveries go to Side B, supply deliveries to Side C. The player can see both a crystal gatherer and a supply gatherer working at the same Tunnel simultaneously on different sides. The explicit Drop Off command provides manual control when needed (e.g., redirecting to a specific Tunnel), but the automatic delivery in the Gather cycle handles the common case.
