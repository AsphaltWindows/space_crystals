# Q0015: Magic box and friendly yielding: where they meet, and the box's exact shape

- **Date**: 2026-11-23T19:45:00-05:00
- **Blocking**: no. The work goes ahead on the assumptions listed under each point.
- **Docs**: `systems/controls/commands.md > Group Destinations (Magic Box)` (0021), `systems/units.md > UnitCollision > Ground Collision > Friendly yielding` (0022, 0023)

## Question
1. **A converging group against yielding.** When a click falls inside the box, 0021 says all units go to the exact clicked point and "stop where the crowd blocks them". Under 0022, a unit that has arrived is idle, so it would step aside for every group member still coming in. The units would keep displacing each other at the point until the last one arrives.

   **Assumption:** a unit that arrived under a group order doesn't yield to members of that same order who are still arriving. Those members stop against it, as "stop where the crowd blocks them" describes. To everyone else it is an ordinary idle unit and yields as 0022 says. A later order clears this.
2. **What the box is drawn around.** 0021 calls the box "the bounding rectangle of the ordered units' positions". Taken literally, that means the unit centres, and then units standing in a straight line have a box with zero height. A right-click "in the middle" of the line almost never lands inside it, so 0021's own Verification fails in play: the line slides a few pixels instead of converging.

   **Assumption:** the box covers the units' footprints, their whole collision boxes, so a click on or between the units of a line counts as inside. The group centre is the centre of that box.
3. **Shift-queued group moves.** The box and offsets currently come from where the units stand when the order is given. Say a group is queued to go to A and then back to its starting area. The second move lands inside the original box, so the units converge instead of returning in formation. The alternative is to build the box for a queued move from the destinations of the move queued before it.

   **Assumption:** positions at the time the order is given (current behaviour).

## Context
Group destinations (0021) and friendly yielding (0022, 0023) are both being built now. Point 1 is where the two rules meet. Point 2 was found while writing tests for 0021's Verification. Point 3 is a rarer case.

## Options
- **Point 1:** A (the assumption) or B (yield to everyone, including the converging group, and accept the shuffling).
- **Point 2:** A (footprints) or B (centres, the literal reading).
- **Point 3:** A (positions now) or B (the previous queued destinations).
