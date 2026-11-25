# Q0073 — AttackMove: does a chase given up by the Failure rule count as "can't reach" until something changes?

- **Date**: 2026-11-24T22:24:23-05:00
- **Blocking**: no — proceeding on assumption: option A. M3-L4 builds it with 0230's reach rule.
- **Docs**: `systems/controls/behaviors.md > AttackMovingToLocation > Algorithm` (steps 2 and 3, 0230), `> AttackingObject > Blocked by its own side` and `> Failure` (0223)

## Question
An attack-moving unit takes an enemy, but can't get to a firing position because its own side's units fill the only way. 0223 then applies the Failure rule: after about 7 s the unit gives the chase up. Under 0230, when may it take that enemy again?

## Context
- **The reach rule (0230)** judges a chase by its **path**: does the path to a firing position stay within the leash and keep the target in sight? Paths are planned on PathKnowledge (terrain and structures), not on units. A way blocked only by the unit's own side therefore passes the check.
- **The cycle this produces**, found in slice M3-G3:
  - Four Peacekeepers attack-move onto a building through a 1-tile corridor. The front one stands and fires.
  - The ones behind take the target and are blocked, and give up after about 7 s.
  - They walk on, take the same target again, and are blocked again.
  - Each lap is about 7 s, and their AttackMove never ends while the front unit stands.
- **0223 point 1** said "An AttackMove drops the target for the leg (0204)". 0230 has since removed dropped-target memory.
- **0230's step 3** says a dropped target "is judged again like any other enemy by the reach rule in step 2, so it is not re-taken while the chase still can't be finished."

## Options
- **A. A failed chase counts as "can't reach".**
  - The enemy is treated like one the reach rule rejected. It is judged again only when 0230's triggers fire: the enemy moves a meaningful distance (placeholder 1 GU), the obstacle knowledge changes, or the leash line is replaced.
  - Within weapon range it is always taken, as before.
  - In the corridor, the units behind give up once, walk on along their route, and fight whatever comes within weapon range.
  - Code: the failed chase goes into 0230's rejection cache, so nothing new is needed.
- **B. Judged again at once by the reach rule.** The corridor cycle above stays, and the AttackMove never ends while the way stays blocked.
- **C. A, but also judged again when a unit of the player's own side leaves the way.** Units would step in as places free up. This is close to the crowd logic that 0223 rules out, and costs a check per blocking unit.
