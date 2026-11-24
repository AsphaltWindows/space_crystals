# Q0038: Gathering Drones, collision and the economy targets

- **Date**: 2026-11-24T05:02:31-05:00
- **Blocking**: **yes**, for slice H's final behaviour and for the Colonists oracle (K1). H starts with the rules as they read now.
- **Docs**: `factions/colonists/units.md > Patch Occupancy (one Drone per patch)`; `systems/units.md > UnitCollision`, `> Who yields`; `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`.

## Question
1. **Do Drones on a Gather job collide with each other?** Today every ground unit collides, and units busy with a stationary task, mining included, never yield (`> Who yields`). A Drone on a Gather job always has an active command, so it never yields, whether it is mining, waiting at an occupied patch or travelling.
   - At saturation the reference check puts about 24 working Drones in the 2-GU (128 su) strip between the row and the Egress Node.
   - There, one Drone mines in front of each patch and others wait beside it, while the rest cross the strip both ways.
   - Returning Drones from a 512-su row converge on a 2x2 node.

   **Proposal:** Option B. Drones on a Gather job pass through other Drones on a Gather job, and still collide with everything else.
2. **Reference check geometry.** "Egress Node 2 grid units from the row's center": edge to edge, or centre to centre?
   - Centre to centre leaves a 32 su gap between the node and the middle patches, narrower than a Drone (36 su).
   - The occupancy note's figures, about 2 Drones at 2 GU and 2.5 at 3 GU, fit legs of 2 and 3 GU of travel at 8 su per frame.

   **Proposal:** a gap of 2 GU between the node's edge and the edge of the row's middle patches, with the node on the row's long side.
3. **How close must the oracle come to the table?** **Proposal:** targets 1–4 must hold, and each income figure must be within ±15% of the table. A larger gap comes back to you as a question with the measured table, not as a code change.

## Context
The targets (about 23 per second at 2 Drones per patch, about 32 at full saturation with about 24 working Drones) treat patch occupancy as the only limit. Under hard collision with busy units that never yield, crowding at the row and the node will lower the saturation and can jam. Every RTS with walking workers faces this; many let gathering workers pass through each other. This shapes slice H and is the main reason the oracle may miss the targets.

## Options
- **Point 1:**
  - A, hard collision as now. Expect saturation and income below the targets, possible jams, and more pathing work.
  - B, gathering Drones pass through each other, as proposed. The targets become reachable; they still collide with other units and structures.
  - C, gathering Drones yield to each other only. Partial relief, and still congested at the node.
- **Point 2:** A, edge to edge, as proposed. B, centre to centre.
- **Point 3:** A, as proposed. B, a tolerance you give.
