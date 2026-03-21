# locomotion-orientation-constraints

## Metadata
- **From**: designer
- **To**: task_splitter

## Content

Implement LocomotionOrientationConstraints for all movement models as defined in `artifacts/designer/design/combat.md`.

These constraints define which combinations of Locomotion and Orientation channel states are valid for each movement model, and the maximum turn rate allowed in each combination.

**TurnRateMovement:**
- Stationary + Turning: maxTurnRate = TurnRate
- Stationary + Maintaining: valid
- Moving + Turning: maxTurnRate = TurnRate
- Moving + Maintaining: valid
- Stopping + Turning: maxTurnRate = TurnRate
- Stopping + Maintaining: valid

**FixedTurnRadiusMovement:**
- Stationary + Turning: INVALID (cannot turn in place)
- Stationary + Maintaining: valid
- Moving + Turning: maxTurnRate = currentSpeed / MinimumTurnRadius
- Moving + Maintaining: valid
- Reversing + Turning: maxTurnRate = currentSpeed / MinimumTurnRadius
- Reversing + Maintaining: valid
- Stopping + Turning: INVALID
- Stopping + Maintaining: valid

**SpeedTurnRadiusMovement:**
- Stationary + Turning: unconstrained (spins treads in place)
- Stationary + Maintaining: valid
- Moving + Turning: maxTurnRate = f(currentSpeed, SpeedToTurnRadiusRatio)
- Moving + Maintaining: valid
- Reversing + Turning: maxTurnRate = f(currentSpeed, SpeedToTurnRadiusRatio)
- Reversing + Maintaining: valid
- Stopping + Turning: unconstrained
- Stopping + Maintaining: valid

**DragMovement:**
- Stationary + Turning: maxTurnRate = TurnRate
- Stationary + Maintaining: valid
- Moving + Turning: maxTurnRate = TurnRate
- Moving + Maintaining: valid
- Stopping + Turning: maxTurnRate = TurnRate
- Stopping + Maintaining: valid

**GliderMovement:**
- Moving + Turning: maxTurnRate = f(currentSpeed, MaxCentripetalAcceleration)
- Moving + Maintaining: valid
(Only these two — Gliders must always be Moving)

## QA Instructions

1. WheeledVehicle (FixedTurnRadius): verify it cannot rotate while stationary — must move forward or reverse to turn.
2. WheeledVehicle: verify turn rate increases with speed (maxTurnRate = speed / MinimumTurnRadius).
3. TrackedVehicle (SpeedTurnRadius): verify it CAN spin in place while stationary (unconstrained).
4. TrackedVehicle: verify turn radius widens at higher speeds.
5. Glider: verify it is always in Moving state (never Stationary or Stopping) and turn rate is governed by centripetal acceleration.
6. Infantry (TurnRate): verify turn rate is constant regardless of movement state.
7. HoverVehicle (Drag): verify turn rate is constant (TurnRate) in all locomotion states.
8. Attempt invalid combinations (e.g., FixedTurnRadius + Stationary + Turning) — verify the system prevents them.
