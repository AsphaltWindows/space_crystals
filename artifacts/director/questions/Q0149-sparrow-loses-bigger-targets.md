# Q0149 — With sight 8, a lone Sparrow still loses bigger targets: its loop grows with the target's size

- **Date**: 2026-11-26T15:01:09-05:00
- **Blocking**: no. Slice M5-H2 built 0403 as written, and nothing is tuned.
- **Docs**: `factions/cults/units/sparrow.md > Placeholder stats` (SightRange 8, 0403) and its Strafing rule ("after a pass it flies on until the target is out of range, then loops back at full speed"); design change 0200 (a unit that loses its target stops).

## Question
0403 raised the Sparrow's sight to 8 GU (512 su) so that a lone Sparrow keeps making passes. That works for small targets but not bigger ones. Should a lone Sparrow keep any target, whatever its size?

## Context
- Range is measured edge to edge. So the pass ends when the target's **far edge** is out of range, and the loop's farthest point grows with the target's size.
- **Measured** with no spotter and the target standing at a tile centre:

  | Target | Size | Farthest point of the loop | Result |
  |---|---|---|---|
  | Peacekeeper | small | 497–502 su | kept; dies on the 5th pass |
  | Agent, Guard | 36 su | 514–516 su | lost after 2–4 passes |
  | Seeker, Hand of Heaven, Prayer-mobile | 56–64 su | 512–522 su | lost after 1–2 passes |

  Buildings are bigger still, so a lone Sparrow loses them sooner.
- **Several Sparrows,** or any spotter, share their vision and keep the target.

## Options
- **(a) Accept it.** Big targets need a second Sparrow or a spotter, as Attack Move already does (0403).
- **(b) More sight,** for example 9 GU (576 su) or 10 GU (640 su). This holds up to about 64 su targets, not buildings.
- **(c) The pass ends when the target's centre is out of range,** not its far edge, so the loop no longer depends on the target's size. This is a rule change for strafing.
- **(d) A glider keeps its attack target through its loop,** sight or no sight, while within some distance (Q0148 part 1's option b).
