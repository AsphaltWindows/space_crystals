# Q0074 — AttackMove: should the unit's own movement re-check an enemy it rejected?

- **Date**: 2026-11-24T23:08:54-05:00
- **Blocking**: no — proceeding on assumption: A (as built in M3-L4)
- **Docs**: `systems/controls/behaviors.md > AttackMovingToLocation > Algorithm` (step 2, "Re-checking a rejected enemy", 0230; step 3, 0235)

## Question
0230 says a rejected enemy is checked again "only when something relevant changes, i.e. the enemy moves a meaningful distance, or the obstacle knowledge the path used changes", and "Otherwise a rejected enemy costs nothing more." The unit's own movement is not on that list, and M3-L4 built it that way. Is the consequence below intended?

## Context
- **As built.** A rejection is checked again when:
  - the enemy moves 1 GU (placeholder) from where it was judged;
  - the obstacle knowledge changes;
  - the unit's PathReference is replaced by a different route.

  Within weapon range an enemy is always taken. The unit's own movement doesn't trigger a check.
- **The consequence.** A Guard attack-moves along a wall, and sees an enemy building on the far side.
  - The path round the wall would leave the leash or lose sight of the building, so the Guard rejects it and walks on.
  - It rounds the end of the wall and is now in the open, within sight of the building but out of weapon range.
  - It doesn't attack unless its route brings the building within weapon range.
  - A second Guard that arrives at the same spot with no history attacks the building, because its first judgement passes.
- **So the response depends on what happened before,** which 0230 set out to avoid ("the same enemy, or an identical one swapped in unseen, must get the same response").
- **Who it affects.** Moving enemies are checked again anyway as they move. This mostly affects standing targets: structures, and units holding position.
- **Other rejections.** They also come from a chase given up by the Failure rule (0235) and, as a technical choice in M3-L4, from a target lost to the fog during a chase. Those keep 0230's triggers under every option below.

## Options
- **A. As built.** Own movement is not a trigger. This is the cheapest: a rejected standing enemy costs nothing more. The behaviour above stays.
- **B. Also check again once the unit has moved 1 GU (placeholder) from where it judged,** for reach-rule rejections only.
  - The same situation then gets the same response.
  - Cost: about one short, bounded path search per rejected enemy for each GU the unit travels, within the existing limit of 2 judgement paths per unit per tick. With large armies passing many unreachable targets, that is several times the judgement paths of A. It would be measured.
