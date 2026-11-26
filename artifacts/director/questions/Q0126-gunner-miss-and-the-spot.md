# Q0126 — The Gunner's miss: no unit can leave the spot in 2 frames; and do buildings count as "at the spot"?

- **Date**: 2026-11-25T19:52:26-05:00
- **Blocking**: no — proceeding on assumption: built literally (point 1 a), buildings count (point 2 a), and the tie rule as below (point 3 a).
- **Docs**: `systems/combat.md > TailDisjointed`; `factions/cults/units/gunner.md > AttackAttributes:`; entry 0304

## Question
Slice M5-O3 built 0304 as written. At the end of Aiming the Gunner locks the target's position, a point. At the end of Firing, 2 frames later, it hits whatever covers that point, of any owner, and if nothing does the shot is a miss. Three points came up:
1. **The miss never happens with today's numbers.** In 2 frames no ground unit moves farther than its own half-width, so a target always still covers the point when the shot lands. Should a moving target be able to dodge the Gunner?
   - The closest case is the Drone: 16 su of movement against an 18 su half-width.
   - A Peacekeeper or a Recruit moves 8 su against 12.
   - So "a target that moves during the Firing phase is missed" can't happen in play now. The tests make it happen only with a teleport, or with a test attack whose Firing lasts 12 frames.
2. **Buildings at the spot.** `systems/combat.md > TailDisjointed` says "whatever units are at the target location", and 0304 says "whatever unit stands at the spot". Read literally, a Gunner ordered to attack a building would never damage it. The build counts structures as at the spot. Is that right?
3. **Two objects covering the spot,** for example a unit standing against a building's edge. The build hits the one the shot was aimed at if it still covers the point, otherwise the one with the lowest id. Is that acceptable?

## Context
- The Gunner's numbers: AimDuration 4 frames, FiringDuration 2 frames, Range 5, MinRange 2, Damage 16.
- Units' Silhouettes: 24x24 for infantry, 36x36 for the Drone.
- **The design intends dodging:** "Targets can dodge by moving out of the locked location" (`systems/combat.md > TailDisjointed`).
- **Already in place:** the client's tracer ends at the locked point, so a miss would show on screen as a shot landing on empty ground. Attack Ground works, and so does "Target too close" for a location.

## Options
1. **(a)** Keep the numbers. The Gunner hits whatever it aims at for now, and the miss matters for slower attacks later, such as the Artillery Rig.
   **(b)** Lengthen the Gunner's FiringDuration so that a walking infantry unit can step off the point: 4 frames moves 16 su at 4 su a frame, against a 12 su half-width. Cooldown could shrink by the same amount to keep the rate of fire.
   **(c)** Lock the position earlier, at the start of Aiming. That gives 6 frames, 24 su at 4 su a frame.
   **(d)** Treat "at the spot" as a small circle around the point, so that any unit that moved more than a few su is missed. Misses then become common against anything moving.
2. **(a)** Structures count, as built. **(b)** Only units count, so a Gunner can't damage buildings.
3. **(a)** As built. **(b)** Every object covering the point takes the hit. **(c)** Always the lowest id.

Proposed: (a) for points 2 and 3. Point 1 is a balance choice for the user; the build keeps (a) until it is answered.
