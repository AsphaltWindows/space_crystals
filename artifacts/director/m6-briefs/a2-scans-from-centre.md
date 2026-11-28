# Task A2: Target scans agree with the unit's own sight (0467 part 1)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a mechanic phase, and your budget is 150 tool calls.**

## The design
- `systems/controls/behaviors.md > BaseAutoTargeting > Active during Idle`: target scans measure SightRange from the unit's centre, as vision does, so that a unit's scan and its own eyesight always agree. This applies to every target scan: idle, HoldPosition, AttackMove and Patrol.
- `core/vision.md`, the intro: vision is a disc of radius SightRange around the object's centre.
- `systems/combat.md > Range Measurement`: weapon range stays edge to edge.
- `artifacts/designer/changes/0467-scans-from-centre-full-tanks-tech-gates.md`: part 1 only. Parts 2 and 3 need nothing.
- `factions/syndicate/structures/null_chamber.md`: the buff widens vision only, and scans keep the unbuffed SightRange (0463).

## The Director's reading
0467 lets the Director choose how the measure reaches the target, as long as the choice is consistent. Vision sees a **tile** when the tile's sight point (its centre, `TILE_SIGHT_POINT`) lies within the disc around the eye. The radius is SightRange with the ElevationModifier between the eye and that tile's level (`crates/sim/src/vision/sight.rs`, `stamp`). An object is seen when its box covers a seen tile (`crates/sim/src/vision/player.rs`, `sees_object_box`).

A distance from the centre to the target's edge would still differ from that by up to about half a tile: the size of the gap Q0190 was about. So:

**A scan's SightRange reaches a target when the unit's own eye, at its unbuffed SightRange, sees a tile the target's box covers, by the same test vision uses.** Put that test in one function that both vision's `stamp` and the scan call, so the two can't drift apart. MinRange and weapon Range stay edge to edge.

## Goal
1. **The scan,** in `crates/sim/src/orders/world/scan.rs`:
   - `World::within` and `World::gap_within`, for `Scan::Sight`, use the shared test in place of `gap <= sight range`.
   - MinRange's check stays on the gap between the shapes.
   - Remove the doc's placeholder reading ("edge to edge, like Range") and cite the design section instead.
   - **Check the call sites:** `orders/travel.rs` (near line 325), `orders/engage.rs` (near 569 and 626) and `World::pick`. Check that every SightRange-based target scan goes through these two functions: `grep -rn 'sight_range' crates/sim/src/orders crates/sim/src/combat`. A scan that measures SightRange some other way (a structure's, a turret's) changes the same way, through the same function.
   - **HoldPosition** scans by weapon range (`systems/controls/command_behaviors.md > HoldingPosition` 3), so it stays edge to edge, unless it also uses SightRange somewhere.
   - **A target that leaves SightRange** (when an engagement drops it) is judged by the same test.
   - **Keep it cheap:** reject by distance first, so the tile test runs only for targets near the edge of the reach. Don't run a perf A/B now; that happens at the milestone boundary.
2. **Tests:**
   - Unit tests of the shared function: a tile whose sight point lies on the boundary counts, as vision says. Vision's own tests must still pass unchanged.
   - Scan tests in `scan.rs`'s tests: a target is taken exactly when the unit's own eye first sees it, for a unit wider than the target and for a structure target.
   - **0467's Verification 1:** a lone Guard engages at the same distance buffed by a Null Chamber and unbuffed. SB's test of the buffed and unbuffed gap with a spotter is the model: `grep -rln 'null_chamber\|NullChamber' crates/sim/tests`.
3. **Goldens.** Units now engage about half their width later, so most combat goldens move. Re-bless them in the commit of the change. In your report, list the goldens that moved. For one of them, give the first tick at which it differs and what differs there (a unit taking its target later).
4. **The computer players:** leave `crates/ai` alone unless its tests fail. Their games' rows will move, and the Director runs those games after the merge.

## Don't
- Don't change weapon range, MinRange, or what vision sees: vision's tests and its tiles stay as they are.
- Don't edit anything under `artifacts/`.

## Report
At most 10 lines, then **Handoff**:
- the shared function, and the scans that use it;
- the tests;
- the goldens that moved, with the first differing tick for one;
- the gate's results.
