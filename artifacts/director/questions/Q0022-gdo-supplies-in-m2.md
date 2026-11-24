# Q0022: GDO supplies in M2: Supply Tower, Supply Chopper and Radar Station are incomplete

- **Date**: 2026-11-23T22:18:49-05:00
- **Blocking**: yes, but only for GDO supplies. M2 is planned without them: the HUD shows Supplies 0, and Supply Delivery Stations work as in M1 (0057).
- **Docs**:
  - `factions/gdo/economy.md > SupplyTower`, `> Starting Conditions`;
  - `factions/gdo/structures.md > Radar Station`, `> DeploymentCenter > Constructs`;
  - `factions/gdo/units.md > SupplyChopper`;
  - `factions/gdo/overview.md > GDOResources > Supplies`.

## Question
1. **The Supply Tower's prerequisite and cost.**
   - The Tower requires a Radar Station, which has "Full stats TBD". The Tower's own cost and build time are also TBD.
   - **Proposal:** build the Tower once the Radar Station is designed. Alternatively, give the Tower a cost and time now, and postpone its prerequisite.
2. **The Supply Chopper's flight.**
   - The Chopper is an air HoverCraft driven by accelerations, with no MaxSpeed. Air movement (0043–0047) and the acceleration model (0035) are planned for the vehicles and air milestone.
   - Its transport is TBD (`> Cargo`).
   - **Proposal:** build the Chopper with air movement, in that milestone.
3. **What supplies buy.**
   - Starting Supplies are TBD, and nothing GDO builds costs supplies yet. Collected supplies would have no use.
   - **Proposal:** Starting Supplies stay 0 until GDO supply costs exist.

## Context
`plan.md` lists "GDO supplies: Supply Tower, Chopper" under M2. Planning M2 found that none of the supply loop can be finished without the items above. Everything else in M2 (buildings, crystals, power, production, layouts) is unaffected.

## Options
- **A** (recommended): defer the supply loop to the air milestone. M2 shows Supplies 0.
- **B**: fill in the TBDs now (Radar Station stats, Tower cost and time) and build a simplified straight-line Chopper in M2. This costs about two more slices and rework when air movement arrives.
- **C**: build the Tower in M2 with no Chopper. It would collect nothing.
