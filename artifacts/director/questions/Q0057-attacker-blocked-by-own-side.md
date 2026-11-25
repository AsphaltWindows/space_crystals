# Q0057 — An attacker blocked by its own side close to its target: has it "reached" it, or does it go round and wait?

- **Date**: 2026-11-24T21:28:05-05:00
- **Blocking**: no. M3-G2 is built with the current behaviour listed under each point.
- **Docs**: `systems/controls/behaviors.md > AttackingObject` (Algorithm, Failure), `> MovingToObject > Completion` (Blocked near the target), `> MovingToLocation > Occupied target location`, `> BaseAutoTargeting > Unreachable auto-targets`; `systems/units.md > UnitCollision > Ground Collision` ("Moving units must pathfind around occupied space").

## Question
1. **A melee or short-range attacker behind its own side's attackers, within 3 grid units of the target.**
   - **What happens.** Sabre Legionnaires attack a building. The first ones stand against it and strike. The ones behind are out of weapon range, within 3 grid units of the building, and blocked by units that aren't moving. `MovingToObject > Completion > Blocked near the target` says that "counts as having reached it", with no re-route attempts. But they aren't in range, and AttackingObject doesn't say what an attacker does then.
   - **Current behaviour (unchanged by M3-G2):** the approach ends there and the unit gives the target up.
     - Under an explicit Attack the order completes and the unit is idle. It then takes the target again by itself after 32 ticks (0181), walks into the same units, and gives up again, for as long as the fight lasts.
     - Under an AttackMove the target is dropped for the rest of the leg (Q0054), and the unit walks on to its destination.
   - **Measured:** eight Sabre Legionnaires sent onto a Deployment Center (16 grid units of wall to stand against) in 12 layouts. In every layout two or three of them never strike before it falls (30 of 96 units), and they stand a few steps from free wall.
   - Should such a unit go round its own side's units to a free place at the target, and wait behind them if there is none, as a unit farther out now does (point 2)?
2. **A ranged attacker behind its own side's firing line, farther than 3 grid units from the target** (the M3 playtest's bug: the last of four Peacekeepers never fired on a building).
   - **Current behaviour (new in M3-G2):** it goes round the row of units in its way, to the end of the row, and takes a place there. If there is no way round (a corridor, a passage the row fills), it stands still behind them and waits. It keeps its order, and moves up when the unit ahead of it leaves or dies.
   - The design has "Moving units must pathfind around occupied space" for the going round. It has nothing on the waiting: `AttackingObject > Failure` says the behaviour completes when the unit "repeatedly fails to make progress toward the target". Under that reading a unit queueing in a corridor would give up after about 7 s, while the unit ahead of it is still fighting. The wait has been in the game since M1 (slice E). Is it the wanted behaviour?
3. **How long a waiting unit keeps looking for a place.**
   - **Current behaviour:** three attempts at a way round per approach (the same number as re-route attempts, 0024), plus one for every time another unit takes the place it went for. After that it stands, and looks again only when the way straight ahead is free. So when a place opens elsewhere along the row (a unit there dies), a unit that has used its attempts doesn't go for it.

## Context
Slice M3-G2 fixed the playtest bug in the movement code: a unit that chases the target of its attack, blocked by standing units of its owner that attack the same target, now goes round them. That rule applies only where the occupied-target rule doesn't: the occupied-target rule comes first, and its radius is 3 grid units (0169). A Peacekeeper (Range 4) and a Guard (Range 3) are out of range only beyond that radius, so they are never "arrived". A Sabre Legionnaire (melee) is always inside it.

## Options
- **Point 1:**
  - A, as now: within 3 grid units the approach ends, the target is given up, and the 32-tick retry applies.
  - B, an attacker never "reaches" a target by the occupied rule: behind its own side's attackers it goes round them and waits, like the ranged units of point 2. Units surrounding a target then stand in rings behind those that strike, and step in as places open. (This changes what an AttackMove does at a crowded target: it waits there, where it now walks on.)
  - C, B for an explicit Attack only; an AttackMove and an idle unit give the target up as now.
- **Point 2:**
  - A, as now: go round if there is a way, else wait without limit, keeping the order.
  - B, go round if there is a way, else give up after the usual 7 s (the order completes; an idle unit retries after 32 ticks).
- **Point 3:**
  - A, as now: three attempts, then stand until the way ahead is free.
  - B, look again every N seconds while waiting.
  - C, look again whenever a unit of the row ahead dies or leaves.
