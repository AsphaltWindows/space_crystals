# Q0058 — AttackMove: is a target lost to the fog left alone for the rest of the leg, like one dropped at the leash?

- **Date**: 2026-11-24T21:51:40-05:00
- **Blocking**: no. M3-L3 keeps the M3-L2 behaviour (below) for AttackMove and Patrol; Attack, Follow and idle auto-targeting follow 0200.
- **Docs**: `systems/controls/behaviors.md > AttackMovingToLocation > Algorithm` step 3 (dropped target, 0204), `> AttackingObject > Algorithm` step 6 (0200).

## Question
- **The case.** An attack-moving unit takes an enemy at the edge of its sight. The way to the enemy leads out of sight, for example back round a wall or a cliff. Under 0200 the attack ends once the enemy is no longer visible (after the 0.5 s fog delay), and the unit walks on along its leg. A little farther on it sees the same enemy again.
- **The loop.** If the normal acquisition rules apply again (the enemy is not one "dropped at the leash", 0204), the unit takes it again, turns back, loses sight of it, and so on, forever.
  - This is the loop of Q0054 point 1 (soak seeds 1 and 29). There the cause was the approach leading **out of sight**, not beyond the leash.
  - Tried in M3-L3: a Guard attack-moving past an unarmed Drone across a wall took it 78 times in 1500 ticks and never arrived. In the 40-seed soak, seeds 8 and 29 never finish their AttackMoves.
  - A fixed wait doesn't help. The unit sees the Drone from 384 su of its route (77 ticks of walking), so it would take it again after any short wait.
- **Current behaviour (M3-L2, kept for now):** a target lost from sight while the attack-moving unit was engaged with it counts as dropped. It is left alone for the rest of the leg, unless it comes within weapon range, where it is fought as always.
- Is that right, or should a target lost to the fog be taken again in some other case?

## Context
- **Where it was built.** M3-L3 built 0200 for Attack, Follow and idle auto-targeting; this case is what remains open. The case and the measurements are under Question above.
- **A stale line in the docs.** `systems/controls/selection_and_hud.md`, around line 220, still says "(matching chasing a target into the fog …)". That describes the walk to the last seen position, which 0200 removed.

## Options
- A, as now: like a dropped target (0204). Left alone for the rest of the leg, unless it comes within weapon range.
- B, left alone until it moves a set distance from where it was lost, then taken normally. A target that stands still (as in the loop above) stays left alone.
- C, taken again normally whenever seen (the loop above can happen near walls and cliffs).

Note: 0223's Core principle ("predictable, not optimal") may already settle it. A is the rule a target dropped at the leash follows (0204); C is simple too, but its failure is the loop above.
