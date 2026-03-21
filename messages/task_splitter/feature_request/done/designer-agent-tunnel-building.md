# agent-tunnel-building

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement Agent Tunnel construction behavior as defined in `artifacts/designer/design/syndicate_objects.md` under 'Agent > Building'.

**Tunnel Construction Flow:**
1. Agent receives build command and walks to the target location.
2. Construction begins — the partially-built Tunnel appears at the location with the Agent inside it. The Tunnel starts at 10% HP (ConstructionHP Rule from `artifacts/designer/design/entities.md`). The Agent is untargetable for the duration of construction.
3. **If construction completes (480 frames / 30 seconds):** The Tunnel becomes operational and the Agent is inside the Tunnel Network, available for redeployment from any Tunnel.
4. **If the partially-built Tunnel is destroyed:** The Agent survives and emerges at the location. The Tunnel is lost and any Supplies spent are lost.

**ConstructionHP Rule** (from entities.md): HP during construction = MaxHP x (10% + 90% x construction_progress). A partially-built structure can be attacked and destroyed before completion.

**Construction Time:** 480 frames (30 seconds). Agent must be present for the duration.

**Cost:** Based on how many Tunnels the player currently owns (see Tunnel Cost Scaling in syndicate_objects.md). 1st Tunnel is free (starting Tunnel), 2nd costs 1 Supply, 3rd costs 2, etc.

## QA Instructions

1. Order an Agent to build a Tunnel via the A command + placement.
2. Verify the Agent walks to the target location.
3. Once the Agent arrives, verify a partially-built Tunnel appears at the location.
4. Verify the Tunnel starts at 10% of its max HP (60 HP for a Tier 1 Tunnel with 600 max HP).
5. Verify the Agent becomes untargetable during construction.
6. Wait 30 seconds (480 frames). Verify the Tunnel reaches full HP and becomes operational.
7. Verify the Agent is now inside the Tunnel Network (available for ejection from any Tunnel).
8. Start another Tunnel construction. While in progress, attack the partially-built Tunnel until destroyed.
9. Verify the Agent survives and appears at the Tunnel's location.
10. Verify Supplies spent on the destroyed Tunnel are NOT refunded.
11. Verify the construction cost scales correctly: 2nd Tunnel = 1 Supply, 3rd = 2 Supplies, etc.
