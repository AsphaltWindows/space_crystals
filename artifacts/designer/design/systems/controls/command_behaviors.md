# Controls: Command Behaviors

One behavior per command (systems/controls/commands.md).
See also: `systems/controls/behaviors.md`, `systems/controls/command_behaviors.md`, `systems/controls/action_channels.md` (split from `behaviors.md`, 2026-11-27).

## MovingToLocation
Base behavior for moving to a static target location.

### Reads: BaseCommandState.TargetLocation

### BaseBehaviorState contains:
- Plan: sequence of (Locomotion, Orientation) channel state pairs, respecting LocomotionOrientationConstraints[MovementModel]
- CurrentStep: index into the plan
- ExpectedPosition: the unit's expected position at the current step

### Algorithm:
1. Compute path from unit's current position to TargetLocation
2. Translate path into a plan of (Locomotion, Orientation) pairs
3. Store plan in BehaviorState, set CurrentStep = 0, compute ExpectedPosition
4. Each tick:
   - If unit position deviates from ExpectedPosition: recompute plan from step 1
   - If current step is complete: advance CurrentStep, update ExpectedPosition
   - Set action channels to the current step's values
5. Final step: Locomotion = Stopping, Orientation = Maintaining
6. When stopped: behavior complete

### GliderMovement exception:
Final step is Locomotion = Moving(idle circle over TargetLocation), Orientation = Turning(circle path). Behavior does not complete — continues circling until a new command is issued.

### Failure:
If the unit repeatedly fails to make progress toward TargetLocation, set Locomotion = Stopping. When stopped, behavior complete.

**"Repeatedly fails to make progress"** (placeholder values, user 2026-11-23, tune later):
- **Stuck**: the unit has gone **1.5 s (24 frames)** without getting any closer to its target.
- **Re-route attempts**: a stuck unit looks for another route, at most **3 times**, waiting **0.5 s, 1 s and 2 s** before each attempt.
- **Give up**: if it is still stuck after the third attempt, the behavior fails as above (about 7 s worst case).

This definition applies wherever a behavior says "repeatedly fails to make progress" (MovingToObject, AttackingObject, etc.).

### Unreachable target location:
If TargetLocation can't be reached (impassable terrain such as water or a cliff, inside a structure, or an area cut off from the unit), the unit moves to the **reachable point nearest to TargetLocation** and the behavior completes there (user, 2026-11-23). The order is never refused for being unreachable.

### Occupied target location:
If the unit is stuck **within 3 grid units** of TargetLocation and what blocks it are units that aren't moving (e.g. a crowd standing at the destination, units on Hold Position, a group that already converged there), it counts as **arrived**: it stops and the behavior completes normally, without re-route attempts (user, 2026-11-23; radius raised from 2 to **3 grid units** by the user, 2026-11-24, after crowd tests in which large units stuck just beyond 2 gave up; placeholder).

## MovingToObject
Base behavior for moving to a mobile or static object instance. Similar to MovingToLocation but the target position may change over time, requiring smarter replanning.

### Reads: BaseCommandState.TargetObject

### BaseBehaviorState contains:
- Plan: sequence of (Locomotion, Orientation) channel state pairs
- CurrentStep: index into the plan
- ExpectedPosition: the unit's expected position at the current step
- Additional intermediate state for optimizing recomputation when TargetObject moves (to be defined)

### Algorithm:
Same as MovingToLocation, except:
- Target position is derived from TargetObject's current location
- Recomputation is also triggered when TargetObject has moved sufficiently from its position at plan computation time
- Optimization of recomputation frequency is stubbed out for future refinement

### Proximity:
"Within proximity" = a gap of at most **8 space units** between the unit's Silhouette and the TargetObject's shape, measured edge to edge like range (systems/combat.md > Range Measurement). (User, 2026-11-23.)

### Completion:
- **Static TargetObject** (structure, resource, Supply Delivery Station): behavior complete when the unit is within proximity of the TargetObject.
- **Unit TargetObject: Follow** (user, 2026-11-23). The behavior does **not** complete on reaching the target. While within proximity, the unit stops and waits. When the target moves out of proximity, the unit follows again. Following continues until the player gives a new order, or the target dies, leaves the map (e.g. enters a transport, a building or the Tunnel Network) or **goes out of the owner's vision** (user, 2026-11-25; Q0051, same as AttackingObject > Target lost from vision); then the behavior completes, and the unit stops where it is (or goes on to its next queued order).
- **Blocked near the target**: if the unit is stuck within **3 grid units** (same radius as MovingToLocation; 2026-11-24) of its TargetObject, blocked by units that aren't moving, that counts as having reached it, like MovingToLocation > Occupied target location. For a static target the behavior completes. For a followed unit the unit waits there (no re-route attempts) and keeps following. (User, 2026-11-23.)

### Failure:
Same as MovingToLocation.

## AttackingObject
Base behavior for attacking a destructible object instance. The unit moves toward the target and engages when the target becomes attackable.

### Reads: BaseCommandState.TargetObject

### BaseBehaviorState contains:
- MovingToObject sub-behavior state (for approach)
- AttackPhaseState: current phase of the attack sequence when engaged

### Algorithm:
1. Begin MovingToObject toward TargetObject
2. Each tick: check if TargetObject is within Range, beyond MinRange, and within turret arc (turret units) or unit facing (infantry)
3. When targetable:
   - Locomotion = Stopping, Orientation = Maintaining (or Turning toward target for infantry)
   - Begin attack sequence on BaseAttackChannel (infantry) or set TurretCommandState.LockedTarget = TargetObject (turret units)
4. If target moves out of range or arc: resume MovingToObject toward TargetObject
5. If target is destroyed: behavior complete
6. **Target lost from vision** (user, 2026-11-25; Q0051, replaces the 2026-11-23 walk to the last seen position): if TargetObject is no longer visible to the unit's owner (it went into the fog of war), the **behavior completes at once**, exactly as when the target is destroyed. The unit **stops where it is** and is idle, or goes on to its next queued order if it has one. It does not walk to the last seen position or track the target through the fog; pursuing into the fog is the player's call. Once idle, normal idle auto-targeting (Idle behavior, with IdleLeashDistance) applies if the enemy is visible again. (The fog delay, core/vision.md > Visible > Fog delay, still keeps a target on tiles that just left sight visible for its 0.5 s.)

### GliderMovement exception:
Gliders cannot stop. Instead of stopping when in range, the Glider performs strafing runs:
1. Fly toward TargetObject
2. When in range and turret arc: fire while passing through
3. Continue past the target
4. Loop around for another pass
5. Repeat until target is destroyed or a new command is issued

### Failure:
If the unit repeatedly fails to make progress toward the target, behavior complete.


### Blocked by its own side
(user, 2026-11-25; Q0057; applies systems/controls/behaviors.md > Core principle) An attacker whose way to its target is blocked by other units, its own side's included:
- **Within 3 grid units of the target** (MovingToObject > Blocked near the target): the approach counts as arrived. If it is **not in weapon range** there (typically melee units behind the front rank), it **gives up the target**: an explicit Attack completes and the unit is idle; an AttackMove treats it as a failed chase (AttackMovingToLocation > Algorithm, "A failed chase counts as can't reach"). An idle unit then re-acquires targets by itself (systems/controls/behaviors.md > BaseAutoTargeting > Unreachable auto-targets).
- **Farther out**: it pathfinds around the occupied space if there is a way round (systems/units.md > UnitCollision > Ground Collision). If there isn't, the normal **Failure** rule applies (re-route attempts, then give up after about 7 s); it does **not** wait behind the blockers indefinitely.
- No special crowd logic (rings, queueing, re-checking for spots opening along a row): getting units into good positions is the player's job.

### Target inside MinRange
(user, 2026-11-24) On an explicit Attack whose target is inside the weapon's MinRange, a unit that can move **backs off** until the target is outside MinRange, then attacks. A unit that can't move (e.g. a deployed Artillery Rig) can't carry it out: the order fails with feedback ("Target too close"; same channel as other refusals).
- **How far it backs off** (user, 2026-11-25; Q0054): to the **middle of the band** between MinRange and Range, not just past MinRange, so a target that keeps stepping closer doesn't make it stop and start on every step.
- **A unit that can't back off** (cornered, or blocked behind): it **gives up the order** with the same "Target too close" feedback, like an immobile unit (user, 2026-11-25; Q0054; for now).

## AttackingLocation
Base behavior for attacking a ground location. Only available to units whose AttackType has CanTargetGround = true.

### Reads: BaseCommandState.TargetLocation

### BaseBehaviorState contains:
- MovingToLocation sub-behavior state (for approach)
- AttackPhaseState: current phase of the attack sequence when engaged

### Algorithm:
Same as AttackingObject, except:
- Target position is BaseCommandState.TargetLocation (static)
- Uses MovingToLocation sub-behavior instead of MovingToObject
- Range/arc check is against the target location
- TurretCommandState.LockedTarget is not set (turret fires at location, not a unit)
- **It keeps firing at the location** (user, 2026-11-26; Q0083; replaces "complete after the attack effect is applied"): like an attack on a unit that never dies, the behavior repeats its attack cycle at the location until the player gives another order (or Stop). Queued orders after an Attack Ground therefore never start on their own. (A Specter's Attack Ground follows its own ammo rules: the location always "persists", so after its bomb it flies home to re-arm, factions/syndicate/units/specter.md.)

### GliderMovement exception:
Same strafing run pattern as AttackingObject, targeting the ground location.

## ReversingToLocation
Base behavior for reversing to a static target location. Only available to bases with CanReverse = true. Identical to MovingToLocation except the plan uses Reversing locomotion instead of Moving.

### Reads: BaseCommandState.TargetLocation

### BaseBehaviorState contains:
- Same as MovingToLocation

### Algorithm:
Same as MovingToLocation, except:
- Plan uses Reversing(path) instead of Moving(path)
- LocomotionOrientationConstraints[MovementModel] are evaluated for Reversing combinations

### GliderMovement: not applicable (CanReverse = false)

### Failure:
Same as MovingToLocation.

## AttackMovingToLocation
Base behavior for moving to a target location while engaging enemies encountered along the way. The unit follows its path but breaks off to attack enemies that enter its SightRange, returning to the path if the chase strays too far.

### Reads: BaseCommandState.TargetLocation

### Global Constant: AttackMoveLeashDistance - 6 grid units

### BaseBehaviorState contains:
- MovingToLocation sub-behavior state (for pathing to TargetLocation)
- AttackingObject sub-behavior state (when engaged with a target)
- CurrentTarget: ObjectInstance | None
- PathReference: the path planned when the AttackMove started (used for leash distance calculation). It is **not** replaced after fights (the walking path is re-planned, the leash line isn't), so fights can't drag the unit progressively off course. It **is** replaced when the unit has to re-route because its way is **blocked** (a detour), so the unit can fight and chase normally along the detour (user, 2026-11-24).

### Algorithm:
1. Begin MovingToLocation toward TargetLocation
2. Each tick while moving:
   - Scan for ValidTarget enemies within SightRange
   - If enemy detected: set CurrentTarget, switch to AttackingObject sub-behavior targeting CurrentTarget, **but only if it is a chase the unit can finish** (below)
   - **Only take targets you can reach** (user, 2026-11-25; Q0058, replaces Q0054's "left alone for the rest of the leg"): an enemy **within weapon range** is always taken. An enemy out of weapon range is taken only if the **path** to a firing position (the path the unit plans when it takes a target anyway) **stays within AttackMoveLeashDistance of PathReference and keeps the target within the unit's SightRange**. Otherwise the unit doesn't take it and walks on. So the unit never starts a chase it would have to abandon, and its choice depends only on the current situation. Easy to explain: *"an attack-moving unit fights what it can reach without leaving its route or losing sight of it."*
   - **Re-checking a rejected enemy** (user, 2026-11-25): only when something relevant changes, i.e. the enemy moves a meaningful distance, **the unit itself has moved a meaningful distance since it judged that enemy** (placeholder 1 GU; Q0074, so the same situation always gets the same response; to be revisited if too expensive), or the obstacle knowledge the path used changes (systems/controls/behaviors.md > PathKnowledge). Otherwise a rejected enemy costs nothing more. **Performance is a priority here** (large armies attack-moving past many unreachable targets); the exact triggers and caching are the Director's call, with performance and predictability as the goals.
   - For turret units: set TurretCommandState.LockedTarget = CurrentTarget
3. While engaged with CurrentTarget:
   - **The leash limits chasing only** (user, 2026-11-24): a CurrentTarget **within weapon range is always fought**, wherever the unit is. The unit does not **move** to chase a target if that would take it beyond AttackMoveLeashDistance (perpendicular distance from PathReference). If the target is out of weapon range and can only be reached by going beyond the leash: disengage, clear CurrentTarget, resume MovingToLocation from current position toward TargetLocation (recompute path). So an attack-moving unit is never shot without fighting back by an enemy it can reach from where it is; it only ignores enemies that outrange it from beyond the leash.
   - **A failed chase counts as "can't reach"** (user, 2026-11-25; Q0073): if a chase is given up by the Failure rule (e.g. the way is blocked by the unit's own side, which the path check can't see, AttackingObject > Blocked by its own side), the enemy is treated like one the reach rule rejected: judged again only on the same change triggers (it moves a meaningful distance, the obstacle knowledge changes, or PathReference is replaced). Within weapon range it is still always fought. This stops the give-up/re-take cycle in crowded corridors.
   - A target dropped at the leash (the case above) is judged again like any other enemy by the reach rule in step 2, so it is not re-taken while the chase still can't be finished. There is **no** memory of dropped targets (user, 2026-11-25: the same enemy, or an identical one swapped in unseen, must get the same response).
   - If CurrentTarget is destroyed: clear CurrentTarget, resume MovingToLocation from current position toward TargetLocation (recompute path)
4. When unit arrives at TargetLocation: behavior complete

### GliderMovement exception:
Glider follows its path, performing strafing runs on enemies within SightRange. Leash is measured the same way. On arrival at TargetLocation, transitions to idle circling.

### Failure:
Same as MovingToLocation.

## Patrolling
Base behavior for moving back and forth between two locations while engaging enemies encountered along the way. Combines AttackMovingToLocation with automatic waypoint cycling.

### Reads: BaseCommandState.TargetLocation (patrol destination), unit's position when the Patrol starts executing (patrol origin)

### BaseBehaviorState contains:
- AttackMovingToLocation sub-behavior state
- ConsecutiveFailedLegs: number
- PatrolOrigin: Coordinates (unit's position when the Patrol **starts executing**, not when it was issued; for a Shift-queued Patrol that is where the previous queued order ended. User, 2026-11-23)
- PatrolDestination: Coordinates (BaseCommandState.TargetLocation)
- CurrentLeg: origin-to-destination | destination-to-origin

### Algorithm:
1. Set PatrolOrigin = unit's current position, PatrolDestination = TargetLocation
2. Begin AttackMovingToLocation toward PatrolDestination
3. When AttackMovingToLocation completes (arrived at destination): swap legs, begin AttackMovingToLocation toward the other endpoint
4. Repeat indefinitely until a new command is issued

### Failed legs (user, 2026-11-23):
- A leg that fails (the unit gives up per MovingToLocation > Failure) **counts as arrived**: the unit turns around and patrols back toward the other endpoint.
- If **two legs in a row** fail (stuck in both directions), the Patrol ends and the unit goes idle.
- Unreachable or crowded endpoints are not failures (MovingToLocation > Unreachable / Occupied target location), and neither is stopping to fight.

### Behavior never completes on its own (except two failed legs in a row) — continues cycling until interrupted by a new command.

## HoldingPosition
Base behavior for remaining stationary. The unit does not move for any reason but will turn and fight if able.

### Reads: BaseCommandState (no target parameters needed)

### Algorithm:
1. Locomotion = Stationary at all times
2. For turret units: TurretCommandState is unchanged — turret continues autonomous scanning and engaging targets within range and arc
3. For non-turret units with CanTurnInPlace = true: scan for ValidTarget enemies within weapon range. If enemy detected, Orientation = Turning(enemy position), engage via BaseAttackChannel
4. For non-turret units with CanTurnInPlace = false: can only engage ValidTarget enemies that happen to be in the unit's current facing and weapon range
5. Behavior never completes — continues until a new command is issued

## StoppingBehavior
Base behavior for ceasing all current activity.

### Reads: BaseCommandState (no target parameters needed)

### Algorithm:
1. Locomotion = Stopping, Orientation = Maintaining
2. Clear TurretCommandState.LockedTarget (turret falls back to autonomous scanning)
3. When stopped: behavior complete, unit has no active behavior
