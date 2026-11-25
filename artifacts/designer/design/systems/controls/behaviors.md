# Controls: Behaviors and Action Channels

## Core principle: predictable, not optimal
(user, 2026-11-25) **Every unit's default behavior must be understandable and predictable** in every situation. We do **not** write algorithms that make default behavior optimal for whatever situation the units find themselves in, and we don't aim for silky-smooth crowd pathing. Avoiding bottlenecks and getting the best out of units is **the player's job**. Different units need different amounts of oversight and control to get good results, and that is part of deciding which units to make for a situation.

When a behavior question comes up, prefer the **simple, explainable rule** over a cleverer one that handles more cases, as long as the simple rule fails in ways the player can see and fix (e.g. a unit that gives up and goes idle, then re-acquires a target by itself).

## BaseBehaviorState[UnitBase]
Internal data maintained by the base behavior as it executes its algorithm in response to the BaseCommandState. Includes computed data such as planned paths, progress tracking, and any other cached information the behavior needs across frames. Parameterized by UnitBase as different bases may need to store different kinds of intermediate results (e.g., a Glider's circling parameters vs. a Wheeled Vehicle's turn radius planning).

## TurretBehaviorState
Internal data maintained by the turret behavior as it executes its algorithm in response to the TurretCommandState. May include scan state, last known target position, or other cached information.

## BaseBehavior
The algorithm the unit base executes based on its BaseCommandState, in response to game circumstances. Reads and writes BaseBehaviorState as internal working data. Composed of a sequence of base actions. Also responsible for updating TurretCommandState when appropriate (e.g., assigning a target for the turret to engage).

## TurretBehavior
The algorithm the unit's turret executes based on its TurretCommandState. When the turret has a locked target, it engages that target. When the turret has no locked target, it autonomously scans for and engages the best available target within range and turn angle. Reads and writes TurretBehaviorState as internal working data. Composed of a sequence of turret actions.

## TurretAutonomousScanning
When the turret has no locked target (TurretCommandState.LockedTarget = None), it autonomously selects the best available target from ValidTarget enemies within weapon range and turret arc.

### Target Selection Priority:
1. **Threatening units first**: Prefer ValidTargets whose AttackAttributes.TargetDomain includes this unit's domain (Ground, Air, or Universal). These are units that have the capability to attack this unit.
2. **Least rotation, in coarse steps**: Among equal-priority targets, prefer the target requiring the least turret rotation from the turret's current facing, compared in **45° steps** (user, 2026-11-24; placeholder): the rotation needed (either direction, 0–180°) is grouped as 0–45°, 45–90°, 90–135°, 135–180°, and targets in the same step count as equal rotation.
3. **Closest distance**: Among targets in the same rotation step, prefer the closest target by distance. (Example: an enemy at max range straight ahead and an adjacent enemy 5° to the side are in the same step, so the adjacent one is chosen.) Remaining ties: a fixed deterministic order.

### Algorithm:
1. Gather all ValidTarget enemies within Range and within TurnAngle arc
2. Partition into threatening (can target this unit's domain) and non-threatening
3. From the highest-priority non-empty partition, select the target in the lowest 45° rotation step, breaking ties by closest distance
4. Set TurretCommandState.LockedTarget to the selected target
5. If no valid targets exist, turret remains inactive (TurretAttackChannel = TurretInactive)

## BaseAutoTargeting
When the unit has no active behavior (idle) or is executing HoldingPosition, the base autonomously scans for and engages ValidTarget enemies. For turret units, the base sets TurretCommandState.LockedTarget. For non-turret units (infantry), the base engages directly via BaseAttackChannel.

### Target Selection Priority:
Same as TurretAutonomousScanning: threatening targets first, then least rotation (unit facing for infantry, turret angle for turret units), then closest distance.

### Active during Idle:
When a unit has no active command (command queue empty), it scans for ValidTarget enemies within SightRange. On acquiring a target, the unit switches to an AttackingObject sub-behavior targeting the acquired enemy. The unit's position at the time of acquisition is recorded as the IdleOrigin. If the unit's distance from IdleOrigin exceeds IdleLeashDistance, the unit disengages and returns to IdleOrigin via MovingToLocation. If the target is destroyed or leaves SightRange, the unit returns to idle at its current position (new IdleOrigin).

### Global Constant: IdleLeashDistance - 4 grid units

### Targets inside MinRange
(user, 2026-11-24) Auto-targeting (BaseAutoTargeting and TurretAutonomousScanning) **skips** enemies inside the weapon's MinRange and picks one it can hit; with none, it does nothing.

### Unreachable auto-targets
(user, 2026-11-24; for now) An auto-acquired target the unit can't reach (e.g. on a cliff with no ramp, in sight but out of weapon range) is handled by the normal rules: the unit tries to approach, gives up by the give-up rule, and **retries after 32 ticks (2 s)**. Skipping unreachable targets in favour of the next one was considered and not adopted for now.

### Active during HoldPosition:
Same target scanning as idle, but Locomotion is locked to Stationary. The unit never chases. For turret units, the turret engages within range and arc. For non-turret units with CanTurnInPlace, Orientation = Turning(enemy position) and engage via BaseAttackChannel. For non-turret units without CanTurnInPlace, can only engage targets already in the unit's current facing and weapon range. If the target leaves range, disengage immediately.

### Not active during:
- Move — pure movement, no base engagement. Turret units' TurretAutonomousScanning still operates independently via TurretBehavior, but the base does not acquire targets or set TurretCommandState.LockedTarget.
- AttackTarget — already has an explicit target
- AttackGround — deliberate location-targeted attack, auto-acquiring a different target would override player intent
- Stop — halting all activity

### Not applicable to:
- AttackMove, Patrol — these behaviors have their own explicit enemy scanning logic in their behavior algorithms

## PathKnowledge
Paths are planned only from what the unit's **owner knows** (user, 2026-11-23), so pathing never leaks hidden information:
- **Terrain is always known**: tile presets, corner heights, cliffs, water and Rugged tiles are part of the map.
- **Own structures** are always known.
- **Other players' structures** are known only once seen. A structure the player has seen that is now covered by fog of war is treated as a **wall at its last-known position** (core/vision.md > Explored), even if it has since been destroyed, until the player sees otherwise. Structures the player has never seen are treated as open ground.
- A unit that walks toward an obstacle it didn't know about discovers it and re-routes (exactly when: see When to recalculate a route below).

*Implementation suggestion from the user (not a design requirement; the Director decides):* a per-player cache of known obstacles, updated when a structure enters or leaves that player's view. A cache change since a stuck unit's plan was made could justify a new path calculation.

- **A destination inside a remembered obstacle** (user, 2026-11-24): a click inside (or unreachable behind) a remembered structure resolves, when the order is given, to the nearest reachable spot by what the player knows (systems/controls/commands.md > Group Destinations: destinations are resolved when the order is given). If the unit later sees the obstacle is gone, it **still stops at that resolved spot**; it doesn't continue to the clicked point.
- **When to recalculate a route** (user, 2026-11-24): **the Director's discretion**, e.g. whether to re-plan when a shortcut opens up (a remembered obstacle seen gone), or when a new obstacle comes into view rather than only once the unit is actually blocked. The user's goals for these choices: **avoid performance issues** and keep movement **predictable for the player**. (The user's first instinct: recalculate only when a unit actually gets blocked or stuck.)
- **Map-placed objects** (crystal patches, Supply Delivery Stations) are known from the start, like terrain (user, 2026-11-23). This covers **every neutral object placed by the map maker** (user, 2026-11-24), not only resources. **Players' starting layouts are never known in advance**: an enemy's starting base is known only once seen (it would otherwise reveal their start position and faction). A patch depleted out of the player's view is still treated as present until the player sees it gone.

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
- Behavior complete after attack effect is applied

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
   - If enemy detected: set CurrentTarget, switch to AttackingObject sub-behavior targeting CurrentTarget
   - For turret units: set TurretCommandState.LockedTarget = CurrentTarget
3. While engaged with CurrentTarget:
   - **The leash limits chasing only** (user, 2026-11-24): a CurrentTarget **within weapon range is always fought**, wherever the unit is. The unit does not **move** to chase a target if that would take it beyond AttackMoveLeashDistance (perpendicular distance from PathReference). If the target is out of weapon range and can only be reached by going beyond the leash: disengage, clear CurrentTarget, resume MovingToLocation from current position toward TargetLocation (recompute path). So an attack-moving unit is never shot without fighting back by an enemy it can reach from where it is; it only ignores enemies that outrange it from beyond the leash.
   - **A dropped target is left alone for the rest of the leg** (user, 2026-11-25; Q0054, for now): a target the unit disengaged from (above) while it was still alive is not taken again as CurrentTarget until the unit reaches TargetLocation (the end of this leg), **unless it comes within weapon range**, where it is fought as always. This stops a unit from dropping and re-taking the same target forever at the leash edge.
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

## BaseActionChannels
The unit base operates on three concurrent action channels. Each channel has one active value at a time. Valid combinations of channel states are constrained by UnitBase and current attack phase.

## LocomotionChannel
Controls the unit base's movement.

### Value - LocomotionEnum

## Moving
The unit base is moving along a path from BehaviorState.

### Path - from BaseBehaviorState

## Reversing
The unit base is moving backward along a path. Only available to bases with CanReverse = true (WheeledVehicle, TrackedVehicle, DrillUnit).

### Path - from BaseBehaviorState

## Stopping
The unit base is slowing down to a stop.

## Stationary
The unit base is not moving.

## OrientationChannel
Controls the unit base's facing direction.

### Value - OrientationEnum

## Turning
The unit base is rotating toward a target position. The engine computes the required angle and applies the base's turn rate each tick. For FixedTurnRadiusMovement bases, orientation is constrained by locomotion.

### TargetPosition - Coordinates

## Maintaining
The unit base is holding its current facing direction.

## BaseAttackChannel
Controls the unit base's attack state. Only active for non-turret units. For turret units this channel is always None.

### Value - BaseAttackEnum | None

## BaseAiming
The unit base is aiming at an attack target. Orientation channel is overridden to face the target. Locomotion must be Stationary. If the unit is given a different command, the attack sequence is interrupted.

### Target - AttackTarget

## BaseFiring
The unit base is executing the firing phase. Locomotion must be Stationary. Orientation is locked.

### Target - AttackTarget

## BaseCooldown
The unit base is in the cooldown phase after firing. Locomotion must be Stationary. Orientation is locked.

## BaseReloading
The unit base is in the reloading phase between attacks. Locomotion and Orientation are free.

## TurretActionChannels
The turret operates on two concurrent action channels, running independently of the base action channels. Only present on turret-bearing units.

## TurretOrientationChannel
Controls the turret's facing direction, independent of the unit base's orientation.

### Value - TurretOrientationEnum

## TurretTurning
The turret is rotating toward a target position. The engine computes the required angle and applies the turret's turn rate each tick, constrained by TurnAngle.

### TargetPosition - Coordinates

## TurretMaintaining
The turret is holding its current facing direction.

## TurretAttackChannel
Controls the turret's attack state.

### Value - TurretAttackEnum

## TurretAiming
The turret is aiming at an attack target. TurretOrientation is overridden to face the target.

### Target - AttackTarget

## TurretFiring
The turret is executing the firing phase of its attack sequence.

### Target - AttackTarget

## TurretCooldown
The turret is in the cooldown phase after firing.

## TurretReloading
The turret is in the reloading phase between attacks.

## TurretInactive
The turret is not performing any attack action.
