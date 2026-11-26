# Q0119 — Vehicles among other units: yielding and reversing

- **Date**: 2026-11-25T19:18:39-05:00
- **Blocking**: no — proceeding on assumption: (a) on all three.
- **Docs**: `systems/units.md > Ground Collision`, `> Collision Response (Un-sticking)`, `> FixedTurnRadiusMovement`; `systems/controls/behaviors.md > MovingToLocation`, `> ReversingToLocation`; entry 0323

## Question
The Jackal is the game's first vehicle. 0323 settles how a moving one gets round a blocker: "**WheeledVehicles** turn and graze around the obstacle instead" of stepping sideways. Three points remain:
1. **Yielding.** An idle vehicle can't take "one short sidestep". Does it yield at all, for example by driving forward a little? Or does it stand like a unit on Hold Position? And does own idle infantry yield to a moving vehicle, as to any unit?
2. **Reversing by itself.** Does a vehicle ever back up on its own, to turn round in a tight spot or back off a wall? Or only on the Reverse order (W)?
3. **A point behind it.** Ordered to a point inside its turning circle, such as just behind it, it drives a forward loop. Is that right?

## Context
- The Jackal turns on a 1 GU radius at 8 su/frame, so it fits most corridors.
- 0035's creep is designed to free a vehicle pressed against a wall without reversing.

## Options
1. **(a)** an idle vehicle doesn't yield, and infantry yields to vehicles; **(b)** an idle vehicle drives forward up to one body length to clear the way.
2. **(a)** only on W; **(b)** automatic three-point turns.
3. **(a)** a forward loop; **(b)** reversing when the point is behind it and close.

Proposed: (a) on all three.
