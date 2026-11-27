# 0454 — MinRange back-off to MinRange + 1 GU; a unit that can't aim repositions; hull-aiming units stop in range

- **Date**: 2026-11-27T13:10:00-05:00
- **Answers**: Q0179
- **Docs**:
  - `systems/controls/command_behaviors.md > AttackingObject > Target inside MinRange` (How far it backs off)
  - `systems/controls/command_behaviors.md > AttackingObject > Can't aim from where it stands` (new)
  - `systems/controls/command_behaviors.md > AttackingObject > Algorithm`, step 4 (hull-aiming units)

## What changed and why
1. **The back-off** (user's decision). On an explicit Attack at a target inside MinRange, every unit now backs off to **MinRange + 1 GU**, never beyond Range. This replaces Q0054's "middle of the band". A lone Hand of Heaven (MinRange 4) now stops at 5 GU, well inside its 7 GU sight, and fires. A target that keeps stepping closer may make a unit back off again; the user accepted that.
2. **A unit that can't aim from where it stands** (user's decision). On an explicit Attack, a unit whose weapon can't bear from where it stands keeps trying to reach a position it can attack from. One case is a hull-aiming unit whose turn is stopped by a wall or cliff. The unit moves to a nearby spot that is in range, beyond MinRange, and has room to turn, then aims and fires. It neither waits in place forever nor gives up for this reason. For auto-acquired targets (idle, AttackMove, Patrol) the repositioning is the same, but the normal leash rules apply: past the leash, it disengages. How the spot is chosen is the Director's call.
3. **Step 4** (answered from the docs, as built). A unit that aims with its hull, today only the Hand of Heaven, stops once the target is in range and beyond MinRange, wherever its arc points, and turns its hull to aim. Step 4 now says so.

## Implementation impact
- **Changed:** the back-off distance, for all units.
- **New:** repositioning when blocked from aiming. It replaces the current wait-forever.
- **Unchanged:** M5-N10's step 4 behaviour, now written into the docs.

## Verification
1. Order a lone Hand of Heaven to attack an enemy 2 GU away. It backs off to about 5 GU, keeps the target in sight, and fires.
2. Do the same with a Rocketman (MinRange 2). It backs off to about 3 GU.
3. Park a Hand of Heaven along a cliff edge, facing along it, and order it to attack a target across the cliff. It moves a short way clear, turns, and fires.
4. Repeat 3 with an idle Hand of Heaven auto-acquiring the target. It repositions within its 4 GU leash, or disengages if the only spot is farther away.
