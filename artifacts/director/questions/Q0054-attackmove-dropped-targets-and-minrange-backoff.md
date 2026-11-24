# Q0054: AttackMove: a dropped target isn't chased again on that leg; the MinRange back-off spot and a cornered unit

- **Date**: 2026-11-24T18:53:37-05:00
- **Blocking**: no. M3-L2 is built with the current behaviour listed under each point.
- **Docs**: `systems/controls/behaviors.md > AttackMovingToLocation` (PathReference; Algorithm step 3; 0180), `> AttackingObject > Target inside MinRange` (0182).

## Question
1. **An AttackMove that keeps dropping and re-taking a target.**
   - **The loop found.** A unit took a target at the edge of its sight. The approach led beyond the leash, so it dropped the target, walked back toward the route, saw the target again, and took it again, every other tick, forever (soak seed 1). A first fix made two targets take turns forever instead (seed 29).
   - **Current behaviour:** a target the unit dropped while it was still alive is left alone for the rest of that leg. The unit fights it only if it comes within weapon range. So an enemy that steps back out of range isn't chased again on that leg, even if the leash would now allow it.
   - The design doesn't have this rule, so is it acceptable? Or should the unit re-take a dropped target in some other case, for example after some ticks, or once the target has moved a set distance?
2. **Where a mobile unit backs off to** when an explicit Attack targets an enemy inside its MinRange (0182 says "back off until outside MinRange, then attack").
   - **Current behaviour:** it backs off to the middle of the band between MinRange and Range, not just past MinRange. That way a target that keeps stepping closer doesn't make it stop and start on every step.
3. **A unit that can't back off** (cornered, or blocked behind) while its explicit Attack target is inside MinRange.
   - **Current behaviour:** it gives up the order, as an immobile unit does, with the "Target too close" feedback.

## Context
All three came up while building 0180 and 0182 (M3-L2). The design has no unit with a MinRange above 0 yet; the rules were tested with a test-only unit.

## Options
- **Point 1:**
  - A, as now: left alone for the rest of the leg, unless it comes within weapon range.
  - B, left alone for N ticks, then it may be taken again.
  - C, left alone until it moves a set distance from where it was dropped.
- **Point 2:**
  - A, as now: the middle of the band.
  - B, just past MinRange.
- **Point 3:**
  - A, as now: give up, with the feedback.
  - B, keep trying to back off while the order stands.
