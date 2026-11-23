# Q0007: Group moves and crowding

- **Date**: 2026-11-23T15:46:00-05:00
- **Blocking**: no, but the answer is needed before the steering/collision slice of M1. Proceeding on the defaults below.
- **Docs**: `systems/units.md > UnitCollision > Ground Collision`, `systems/controls/behaviors.md > MovingToLocation`, `systems/controls/selection_and_hud.md` (no limit on selection size)

## Question
1. **Group destination.** Several units are ordered to one point, and they can't all stand on it. Where does each one end up?
2. **Head-on in a narrow passage.** Collision is hard, and idle units never move aside (`Ground Collision`). Two units meeting head-on in a passage too narrow to pass can therefore block each other for good. Two Guards in a one-tile corridor is an example. Should both give up and stop, or should two friendly moving units swap places?

## Context
Units are now being designed for M1. The docs define hard collision and say idle units don't make way. They say nothing about where the members of a group end up, or what happens when two moving units block each other with no way around.

## Options
**Group destination:**
- **A (default)**: each unit gets its own spot on a compact grid around the click point. The spots are spaced by the group's largest unit, and units fill the nearest free spots. Each unit moves at its own speed, so the group does not wait for its slowest member.
- **B**: all units head for the exact point. Those that can't reach it stop wherever the crowd blocks them.
- **C**: units keep their formation relative to each other, as moved.

**Head-on:**
- **A (default)**: both stop after the "fails to make progress" rule (see Q0008) and their orders complete. The player has to sort it out.
- **B**: two friendly units that are both moving swap places. Enemies still block each other.

**Also to confirm:** idle units never make way, not even your own units standing in a chokepoint. This is how the docs read today, and it is what will be built.
