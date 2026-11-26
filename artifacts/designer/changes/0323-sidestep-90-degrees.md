# 0323 — Moving units' contact sidestep: 90°, one body width (infantry, hover, tracked); wheeled vehicles turn and graze

- **Date**: 2026-11-26T17:20:00-05:00
- **Answers**: — (part of the Q0111 work; Q0111 itself stays open, see below)
- **Docs**: `systems/units.md > UnitCollision > Ground Collision` (Moving units steer around blockers)

## What changed and why
User: "The sidestep should probably be 90 degrees and the size of the unit itself", for "infantry, hover vehicles and treaded vehicles".
- When a moving unit of those bases is blocked on contact, it steps **straight sideways**, at 90° to its heading, by **its own width**. This replaces the 45° step.
- The point is to clear the lane it was in, instead of angling forward into a crowd.
- Unchanged: right first, the other side after 1 s, sliding along the blocker's face, the 4-body-length look-ahead, and the stuck rule after that.
- **WheeledVehicles** turn and graze around the obstacle instead, as the user said earlier.
- **Mechs**: TBD.

## Implementation impact
- This changes `docs/movement.md > 6.3`'s contact step for those bases.
- Please measure it on Q0111's 30-layout grid and the `dev_skirmish` soak.
- The next steps for Q0111 are not yet confirmed: a 2-square crystal hatch, then the "stuck gathering Agents make way" backstop if still needed. A follow-up entry will answer Q0111. Measuring this change first is useful either way.

## Verification
- Two Agents meeting head-on in a lane with room to the side: one steps fully sideways by its own width, and both pass.
- A Jackal turns and grazes instead.
