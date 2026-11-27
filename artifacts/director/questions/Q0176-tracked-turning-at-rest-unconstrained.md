# Q0176 — combat.md's table says a tracked unit at rest turns "unconstrained"; the tread law says SpinRate

- **Date**: 2026-11-27T00:42:11-05:00
- **Blocking**: no. M5-V3 follows the turning law at every speed, rest included, and stays so until the answer.
- **Docs**:
  - `systems/combat.md > LocomotionOrientationConstraints[SpeedTurnRadiusMovement]`;
  - `systems/units.md > SpeedTurnRadiusMovement` (The turning law);
  - `factions/cults/units/hand_of_heaven.md > Placeholder stats`;
  - design change 0417.

## Question
Two docs disagree on how fast a tracked unit can turn while stationary or stopping. Which one holds?
- **`combat.md`'s table** reads "Stationary + Turning: **unconstrained**" and "Stopping + Turning: **unconstrained**". Taken literally, that is an instant turn.
- **`units.md`'s turning law** reads "**Spin rate** (stopped, treads in opposite directions): **SpinRate = 2 × MaxSpeed ÷ TrackWidth** radians per frame", and "maxTurnRate = SpinRate × (1 − |v| ÷ MaxSpeed)" at speed v.
- **The Hand of Heaven's doc** agrees with the law: it "spins about 19° a frame when stopped".

## Context
- The table's Moving and Reversing rows already quote the law. Only the Stationary and Stopping rows say "unconstrained", which looks like text from before 0417.
- **Built so (M5-V3):** the law applies at rest, so a Hand of Heaven spins at SpinRate, 1/3 radian (about 19°) a frame. While stopping, it uses the rate for its current speed.
- This matches 0417's Verification and the Hand of Heaven's doc.

## Options
1. **The law at every speed** (as built). The Stationary and Stopping rows of `combat.md` would read "maxTurnRate = SpinRate" and "the law at the current speed".
2. **"Unconstrained" means as written**, an instant turn at rest: the Hand of Heaven's "about 19° a frame" would change, and so would 0417's Verification.
