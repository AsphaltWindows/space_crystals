# Q0051: A target that outruns sight is lost at once, despite the fog delay

- **Date**: 2026-11-24T17:05:30-05:00
- **Blocking**: no. M3-L1 is built with the current behaviour below.
- **Docs**: `core/vision.md > Visible > Fog delay` (0178); `systems/controls/behaviors.md > AttackingObject` and `> FollowingObject` (target leaves vision, 0034).

## Question
0178 makes a tile stay Visible for 8 ticks after it leaves sight, so that "edge-of-sight targets no longer flicker, so Attack/Follow don't drop to last-seen position on one-tick gaps — without a separate held-target rule".

The delay covers only tiles that **were** in sight. A target that moves ahead onto tiles nobody saw is lost at once, and 0034 sends the chaser to its last-seen position.

For example, a Peacekeeper attacking a fleeing Peacekeeper stops to shoot, the target walks past the edge of its sight, and the chase ends at tick 17. Is that intended?

## Context
M3-L1 built 0178 per tile, as its Implementation impact says. A test pins the case above.

## Options
- **A, as now: per tile only.** A target that outruns sight is lost at once. This leaks nothing.
- **B: the held target lingers.** An object a unit is attacking or following stays visible to that unit's owner for 8 ticks after it leaves sight, wherever it moves. This is the separate held-target rule 0178 wanted to avoid, but it is narrow. During those ticks the player sees one enemy moving in the fog.
- **C: every object lingers.** Any enemy object leaving all of a player's sight stays visible for 8 ticks, wherever it moves. Simple and uniform, but every unit leaving sight shows 0.5 s of its movement in the fog.
