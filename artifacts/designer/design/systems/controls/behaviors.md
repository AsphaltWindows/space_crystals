# Controls: Behaviors

See also: `systems/controls/behaviors.md`, `systems/controls/command_behaviors.md`, `systems/controls/action_channels.md` (split from `behaviors.md`, 2026-11-27).

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
1. **Threatening units first**: Prefer ValidTargets whose AttackAttributes.TargetDomain includes this unit's domain (Ground, Air, or Universal). These are units that have the capability to attack this unit. **Buildings that fire through their occupants** (e.g. the Cults Garrison; user, 2026-11-27; Q0139) count as threatening while **any occupant's weapon** (as fired from inside, e.g. a Recruit's Militant stand-in) could attack this unit's domain.
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
