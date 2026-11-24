# Q0025: Power penalty details

- **Date**: 2026-11-23T22:18:49-05:00
- **Blocking**: no. M2 builds the proposals.
- **Docs**:
  - `factions/gdo/overview.md > GDOResources > Power`, `> DisplayHud`;
  - `factions/gdo/economy.md > Opening Economy Baseline`.

## Question
1. **Is the Deployment Center slowed?**
   - The penalty applies to "power-consuming buildings and Extraction Plates". The DC produces power (+30), so read literally, its construction is never slowed.
   - The baseline's all-in Barracks mentions only "plates and Peacekeeper production" running at about 59%.
   - **Proposal:** the DC is never slowed. The EF (plate construction), the Barracks (production) and the plates (mining) are.
2. **The ratio.**
   - **Proposal:** generated power divided by consumed power, capped at 100%. The baseline's -21 case gives about 59% (30/51), which matches this rule.
3. **"Power: current / total".**
   - **Proposal:** current = generated − consumed, which can be negative; total = generated. The display reads `9/30` at the start and `-21/30` in the all-in case.
4. **When a building's power counts.**
   - **Proposal:** from the tick the building is placed until the tick it is destroyed.

## Context
Power is built in M2. Point 1 decides how the opening plays out at negative power.

## Options
- **Point 1:** A, the DC is never slowed; B, every GDO constructor and producer is slowed, the DC included.
- **Point 3:** A, total = generated; B, total = consumed.
