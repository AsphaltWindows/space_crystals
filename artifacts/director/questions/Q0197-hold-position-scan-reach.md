# Q0197 — Hold Position's target scan: SightRange from the centre, or weapon range?

- **Date**: 2026-11-28T04:25:48-05:00
- **Blocking**: no. Proceeding on (a), as built: a unit on Hold Position scans for targets within its weapon range.
- **Docs**:
  - `systems/controls/command_behaviors.md > HoldingPosition` (Algorithm 3 and 4);
  - `systems/controls/behaviors.md > BaseAutoTargeting > Active during HoldPosition` and `> Active during Idle`;
  - change 0467, part 1.

## Question
How far does a unit on Hold Position scan for targets: its SightRange from its centre, as an idle unit does, or its weapon range?

## Context
- **HoldingPosition's algorithm** says: "scan for ValidTarget enemies within weapon range" (step 3). A unit that can't turn in place "can only engage ValidTarget enemies that happen to be in the unit's current facing and weapon range" (step 4). The game does this, measuring edge to edge.
- **BaseAutoTargeting > Active during HoldPosition** says: "Same target scanning as idle, but Locomotion is locked to Stationary. The unit never chases."
- **0467** says: "Every target scan measures SightRange from the unit's centre, as vision does: idle, HoldPosition, AttackMove and Patrol."
- A unit on hold never moves, so it can only fire at what is within its weapon range. The two readings differ only for units whose weapon reaches further than their scan:
  - **The Gunner, the Rocketman and the Twister**, by their half width (12, 12 and 26 su). This is the band of Q0196 point 2.
  - **The Artillery Rig, the Hand of Heaven and the Laser Battery**, whose Range exceeds their SightRange. As built, one of them on hold fires at an enemy within its weapon range that another unit spots, beyond its own sight.

## Options
- **(a) As built: weapon range** (HoldingPosition step 3).
  - A unit on hold fires at any visible enemy within its weapon range, whether it spots the enemy itself or another unit does.
  - "Same target scanning as idle" would then mean the choice of target (ValidTarget, priorities), not the reach.
  - No change.
- **(b) Both: within SightRange from the unit's centre, and within weapon range.**
  - A unit on hold engages only what its own idle scan would reach.
  - Artillery on hold doesn't fire at enemies beyond its own sight, and the three units above don't fire in their band.
  - A small change; some goldens may move.
