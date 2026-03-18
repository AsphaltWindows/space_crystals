# Ticket: GDO Power Grid System

## Current State
No power grid mechanic exists. GDO Power is defined as a resource type but the capacity calculation and slowdown logic are not implemented.

## Desired State
Implement the GDO Power grid as a flat capacity system:

- Each GDO building has a static Power value: positive for generators, negative for consumers.
- A player's total Power is the **sum** of Power values across all owned buildings.
- If total Power is **zero or positive**: all buildings operate normally.
- If total Power is **negative**: all power-consuming buildings operate slower, proportional to (total available power / total required power). "Total available power" = sum of all positive Power values. "Total required power" = absolute value of sum of all negative Power values.
- Power recalculates whenever a building is added, removed, or changes Power value.

## Justification
Required by `features/factions_and_resources.md` (GDO Resources: Power section). Power is a core GDO mechanic that gates building performance and forces strategic decisions about generator placement.

## QA Steps
1. Place a GDO generator building (+10 Power) and a consumer building (-5 Power). Verify total Power shows +5 and the consumer operates at full speed.
2. Add another consumer (-10 Power). Total Power is now -5. Verify consumers operate at reduced speed: available = 10, required = 15, so speed factor = 10/15 = 0.667.
3. Remove the generator. Verify total Power updates correctly and slowdown ratio recalculates.
4. Place only consumer buildings (no generators). Verify all consumers are fully slowed (available = 0).
5. Verify non-GDO factions are unaffected by Power mechanics.

## Expected Experience
When a GDO player has sufficient power, buildings behave normally. When power goes negative, building operations (production times, research speed, etc.) visibly slow down proportional to the power deficit. The Power HUD display shows current/total values updating in real time as buildings are placed or destroyed.
