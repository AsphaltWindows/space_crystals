# Q0024: Production queue and rally point details

- **Date**: 2026-11-23T22:18:49-05:00
- **Blocking**: no. M2 builds the proposals.
- **Docs**:
  - `factions/gdo/structures.md > Barracks > BarracksInstanceState`, `> RallyPoint behavior`, `> ObjectInterfaceState[Barracks]`;
  - `factions/gdo/structures.md > DeploymentCenter > Cancellation`;
  - `factions/gdo/overview.md > GDOResources > Unit Control`;
  - `systems/controls/selection_and_hud.md > CommandPanel`, `> CommandIndicators`;
  - `core/tech_and_research.md > Research > Shared Rules`.

## Question
1. **Does the limit of 20 include the unit in production?**
   - BuildQueue (max 20) and CurrentBuild are separate fields.
   - **Proposal:** 20 in total, the unit in production included.
2. **Unit Control for queued units.**
   - Q needs "sufficient ... Unit Control", and "the player cannot build units if doing so would exceed the cap".
   - **Proposal:** a queued unit takes its Unit Control when it is queued, as it pays its crystals then. The HUD's "used" count includes queued units, and cancelling gives the Unit Control back.
   - **Alternative:** only existing units count, and a unit waits at the front of the queue while the cap is full.
3. **Several Barracks selected, and not enough crystals for all of them.**
   - By 0071 (no smart-casting), each press of Q goes to every Barracks in the ActiveGroup, so each one queues a Peacekeeper.
   - **Proposal:** the Barracks pay in a fixed order while crystals last. Those that can't pay don't queue.
   - **Alternative:** no Barracks queues unless all of them can pay.
4. **A producer or constructor destroyed while working.**
   - The GDO docs are silent. Research is refunded in full when its building is destroyed.
   - **Proposal:** treat it like cancelling:
     - the Barracks queue is refunded in full;
     - a building under construction in the DC is refunded in full;
     - a ready building is refunded at 75%, rounded down.
   - **Alternative:** everything in progress is lost.
5. **Clearing a rally point.**
   - The design gives no way back to "None" except the target disappearing.
   - **Proposal:** setting the rally point on the Barracks itself clears it.
6. **Showing the rally point.**
   - CommandIndicators has no rally entry.
   - **Proposal:** while the building is selected, show a green location marker for a ground rally, a green object marker on a friendly or neutral object, and a red object marker on an enemy.
7. **A rally on an enemy that can't be seen when the unit appears.**
   - A unit can't be ordered to attack a target its owner doesn't see (Q0017's assumption).
   - **Proposal:** the unit attacks if the enemy is visible. Otherwise it moves to the spot where the rally point was set.

## Context
These points come from planning Barracks production for M2. The proposals follow the crystal rules: costs are paid when queued, and cancelling refunds them in full.

## Options
- **Point 1:** A, 20 in total; B, 20 waiting plus the unit in production.
- **Point 2:** A, reserved when queued; B, checked when production starts.
- **Point 3:** A, a fixed order while crystals last; B, none queue unless all can pay.
- **Point 4:** A, refunded as for a cancel; B, lost.
- **Point 5:** A, rally on itself clears it; B, a dedicated command.
- **Points 6 and 7:** A, as proposed; B, the Designer's alternative.
