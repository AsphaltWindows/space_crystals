# 0417 — Tracked vehicles: a tread-based turning law (TrackWidth replaces SpeedToTurnRadiusRatio)

- **Date**: 2026-11-26T17:30:00-05:00
- **Answers**: Q0153 parts 1 and 3 (parts 2 and 4 follow separately)
- **Docs**: `systems/units.md > SpeedTurnRadiusMovement` (stats and "The turning law"); `systems/combat.md > LocomotionOrientationConstraints[SpeedTurnRadiusMovement]`; Placeholder stats of `factions/gdo/units/{artillery_rig,prospector}.md`, `factions/syndicate/units/{laser_battery,subterranean_transport}.md`, `factions/cults/units/hand_of_heaven.md`

## What changed and why
Q0153 showed that "radius = speed × ratio" gives the same turning rate at every speed. The user pointed out that the law itself was wrong: at full speed both treads are at maximum, so a tracked vehicle can't turn at all (infinite radius). The new law follows the two treads (forward speed = their average, turning = their difference, each tread's top speed = MaxSpeed):
- **SpinRate = 2 × MaxSpeed ÷ TrackWidth** rad/frame (stopped).
- **maxTurnRate(v) = SpinRate × (1 − |v| ÷ MaxSpeed)**: falls linearly to **0 at MaxSpeed**.
- **Tightest radius(v) = (TrackWidth ÷ 2) × |v| ÷ (MaxSpeed − |v|)**.
- Reversing: same law with |v|; new stat **ReverseMaxSpeed** (placeholder: half MaxSpeed). Reverse still only on the Reverse order.
- Turning costs no speed by itself; a vehicle slows with its Deceleration to turn more sharply.
- **SpeedToTurnRadiusRatio is removed**; new stat **TrackWidth** (user approved the name).

Placeholders (designer): TrackWidth ≈ ¾ of the silhouette's width.

| Unit | MaxSpeed | TrackWidth | ReverseMaxSpeed | Spin |
|---|---|---|---|---|
| Artillery Rig | 4 | 36 | 2 | ~12.7°/frame |
| Prospector | 4 | 36 | 2 | ~12.7°/frame |
| Laser Battery | 4.5 | 33 | 2.25 | ~15.6°/frame |
| Hand of Heaven | 5 | 30 | 2.5 | ~19°/frame |
| Subterranean Transport (surfaced) | 4.5 | 36 | 2.25 | ~14.3°/frame |

**Q0153 part 3**: the surfaced Transport takes the Laser Battery's Acceleration 0.4 and Deceleration 1, plus the TrackWidth and ReverseMaxSpeed above (designer placeholders).

## Implementation impact
Replaces the turn law and the ratio stat for every TrackedVehicle (and the DrillUnit above ground). Pathfinding's maneuver tables (Pathfinding note) must use the new law: to turn at all a vehicle must be below MaxSpeed, and sharp turns need real slowing.

## Verification
- A Hand of Heaven standing still turns 90° in about 5 frames (0.3 s).
- At full speed, ordered to a point 90° to its side, it first slows, then turns; it never turns while at MaxSpeed.
- At half speed its tightest circle has a radius of about 15 su; at 4 su/frame about 60 su.
- Reverse order: it backs up at no more than 2.5 su/frame and turns by the same law.
