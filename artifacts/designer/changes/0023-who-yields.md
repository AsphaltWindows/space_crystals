# 0023 — Who yields: idle units only; modes don't matter; deployed Artillery Rig is a fixed obstacle

- **Date**: 2026-11-23T18:34:06-05:00
- **Answers**: —
- **Docs**: `systems/units.md > UnitCollision > Ground Collision > Friendly yielding` (Who yields, Unit modes don't matter, Units that can't move never yield)

## What changed and why
Follow-up to 0022, decided by the user:
- Only units with **no active command** yield. **Hold Position** units don't (deliberate walls stay possible), nor do units busy with a stationary task (attacking in place, mining, constructing). This confirms the default suggested in 0022.
- **Modes don't count as tasks:** an idle Peacekeeper with Riot Shield raised still yields.
- **Units that can't move never yield:** a deployed Artillery Rig is treated like a building, a fixed obstacle.

## Implementation impact
- Yield check: idle (empty command queue) AND able to move in its current mode. Riot Shield doesn't block yielding, and a deployed or deploying Artillery Rig does.
- Still TBD: sidestep distance (your placeholder), allies.

## Verification
- Walk a unit through your own idle Peacekeepers with shields raised: they step aside (at their shielded speed).
- Through a deployed Artillery Rig: the mover paths around it.
- Through units on Hold Position: they don't budge and the mover paths around or gives up.
