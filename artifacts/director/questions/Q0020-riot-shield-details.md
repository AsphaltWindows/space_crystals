# Q0020: Riot Shield details, and a conflict with 0062

- **Date**: 2026-11-23T23:05:00-05:00
- **Blocking**: no. The Riot Shield is built with the behaviour listed under each point.
- **Docs**: `factions/gdo/units.md > Peacekeeper > Riot Shield`, change 0062 (right-click: units that can't carry out the command move or follow)

## Question
1. **Right-click on an enemy unit while the shield is up.** The two docs conflict here.
   - The Peacekeeper doc says it "issues a Move order to that location instead of an Attack".
   - 0062 says a unit that can't carry out the command **follows** a unit target.
   - **Assumption:** the Peacekeeper doc wins, because it is the more specific rule. The unit moves to the clicked spot.
2. **Raising and lowering.** While the shield is going up or coming down, the unit is already slowed and can't attack. The 50% damage reduction applies only once the shield is fully up. Turning the shield around halfway reverses from where it is.
3. **Raising during a shot.** A shot that is already firing still lands, and the unit is then held. The alternative is to refuse "raise" until the shot ends, as the Legionnaire's Convert does with its command.
4. **Mixed group.** In a group where some shields are up and some are down, the key raises them all. Pressing it again lowers them all.
5. **Toggle time and hotkey.** Placeholder values: 6 ticks (0.375 s, under the docs' "under half a second") and R. Both are TBD in the docs.

## Context
The Riot Shield is built end to end (M1-R). Point 1 decides what the player sees when right-clicking an enemy with a shielded group. Points 2–5 are details where the docs are silent.

## Options
For each point: the current behaviour (A), or a different rule you give. For point 1, the alternative (B) is that 0062's Follow wins and the unit follows the enemy unit.
