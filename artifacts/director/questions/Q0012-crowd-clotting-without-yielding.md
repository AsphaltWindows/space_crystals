# Q0012: Crowds clot when units never make way. Should friendly units yield or swap?

- **Date**: 2026-11-23T18:20:00-05:00
- **Blocking**: no. The implementation follows the docs: no pushing, no yielding, no swapping. Units that stay blocked give up. This question adds test evidence to the open part 2 of Q0007.
- **Docs**: `systems/units.md > UnitCollision > Ground Collision`, `systems/controls/behaviors.md > Failure`

## Question
Should **friendly** units make room for each other? The options:
- an idle or arrived unit steps aside when a friendly unit is blocked by it;
- two friendly units that block each other head-on swap places.

Or should the current rule stay: nothing ever makes way, and blocked units give up after a while?

## Context
Hard collision with "idle units never move aside" is now built and tested. In playtest-like runs:
- **Head-on crowds:** 18 Peacekeepers meeting 18 Peacekeepers head-on in an open field leave about 4 units stuck. They give up their move order and stop short.
- **Random-order stress test:** with 200 units and many crossing orders, roughly 1 move in 3 to 1 in 5 ends in "gave up" instead of "arrived".
- **One-tile corridor:** two units meeting head-on with no other route both give up, as Q0007's default says. With a second corridor available, they take the detour instead.
- **Group arrivals:** a unit that reaches its group's crowd behind members already standing at their destinations now counts as arrived, wherever it stops. This is a stand-in until the design decides.

Most RTS games (StarCraft II, Age of Empires II) let a moving unit push idle friendly units aside. That removes most clots.

## Options
- **A (current)**: nothing makes way. Players untangle clots themselves.
- **B**: idle and arrived **friendly** units step aside, one short sidestep, when a friendly mover is blocked by them. Enemies never yield. This is the StarCraft-like choice and removes most clots.
- **C**: B, plus two friendly movers blocked head-on swap places.
- **D**: units that are moving may pass through friendly units at reduced speed, i.e. softer collision. This is a bigger departure from "hard collision".
