# Q0018: Behaviour gaps found while building chase, auto-targeting and AttackMove

- **Date**: 2026-11-23T21:40:00-05:00
- **Blocking**: no. Behaviours are built with the placeholder listed under each point. Each one is a single named rule in the code.
- **Docs**: `systems/controls/behaviors.md` (BaseAutoTargeting > Target Selection Priority, AttackingObject, AttackMovingToLocation), `core/vision.md`, `systems/combat.md` (MinRange)

## Question
Please confirm or correct each of these:
1. **Losing sight for a moment.** Sight is measured in whole tiles, so a target right at the edge of sight blinks in and out of view.
   - Current behaviour: an Attack or Follow switches to "go to the last seen position" (0034) the first tick the target is unseen. When the target reappears, auto-targeting picks it up again, so chases at the edge of sight look jerky.
   - An idle unit's chase also resets the point it is leashed to.
   - Should a target stay "held" for a short grace period, such as 0.5 s unseen, before the order switches to the last seen position?
2. **"Least rotation" in target priority.** Comparing exact angles means two enemies almost never tie on rotation. Distance therefore almost never decides, and a far enemy straight ahead beats an adjacent one slightly to the side.
   - Current behaviour: exact angles, as written.
   - Alternative: compare rotation in coarse steps (for example 45° sectors, or "needs no turn" versus "needs a turn") and let distance decide within a step.
3. **The AttackMove leash after a fight.** The leash is measured from the unit's planned route (PathReference). If the route were re-planned after each fight, the leash would move with the unit and stop limiting chases.
   - Current behaviour: PathReference stays the route planned when the AttackMove started.
   - Is that the intent?
4. **An auto-acquired target that can't be reached.** An example is an enemy on a cliff with no ramp, inside sight but out of weapon range.
   - Current behaviour: the unit tries to approach, fails by the give-up rule, and retries after 32 ticks (`GIVE_UP_RETRY_TICKS`).
   - Alternatives:
     - ignore targets that can't be reached;
     - ignore them for longer;
     - pick the next target.
5. **A target inside MinRange.** No M1 unit has a MinRange, but artillery will.
   - Current behaviour: the attack fails and the order gives up.
   - Alternatives:
     - the unit backs off until the target is outside MinRange;
     - auto-targeting skips targets inside MinRange.
6. **A unit walled out of its group spot after a fight.** A late unit can find its magic-box spot surrounded by its own group.
   - Current behaviour: it counts as arrived within 2 grid units (0026), and gives up beyond that.
   - Is that acceptable?

## Context
These came up while building the behaviours (M1-E). Points 1 and 2 are the most visible: chases at the edge of sight, and which enemy a unit shoots first.

## Options
For each point, either the current behaviour (A) or a different rule you give. The main alternatives:
- **Point 1 (B):** a grace period before the target counts as lost.
- **Point 2 (B):** coarse rotation steps, then distance.
- **Point 4 (B):** skip unreachable targets for a while and pick the next one.
- **Point 5 (B):** back off to MinRange.
