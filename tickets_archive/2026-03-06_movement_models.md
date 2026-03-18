# Ticket: Movement Models and Locomotion-Orientation Constraints

## Current State
No movement model definitions exist. Units have no physics parameters for movement.

## Desired State
Five movement model structs, each with their specific parameter sets, plus per-model LocomotionOrientationConstraint tables defining valid Locomotion+Orientation state combinations and maxTurnRate constraints.

### MovementModelEnum
TurnRate, FixedTurnRadius, SpeedTurnRadius, Drag, Glider

### TurnRateMovement
- TurnRate: number
- Acceleration: number
- Deceleration: number
- MaxSpeed: number
- Used by: LightInfantry, HeavyInfantry, Mech

### FixedTurnRadiusMovement
- MinimumTurnRadius: number
- ForwardAcceleration: number
- ForwardMaxSpeed: number
- ReverseAcceleration: number
- ReverseMaxSpeed: number
- Deceleration: number
- Used by: WheeledVehicle

### SpeedTurnRadiusMovement
- SpeedToTurnRadiusRatio: number
- Acceleration: number
- Deceleration: number
- MaxSpeed: number
- Used by: TrackedVehicle, DrillUnit

### DragMovement
- ForwardAcceleration: number
- OmniDirectionalAcceleration: number
- DragRatio: number
- TurnRate: number
- MaxSpeed is derived: (OmniDirectionalAcceleration + ForwardAcceleration) / DragRatio
- Used by: HoverVehicle, HoverCraft

### GliderMovement
- IdleSpeed: number
- MaxSpeed: number
- Acceleration: number
- Deceleration: number
- MaxCentripetalAcceleration: number
- Turn radius is derived: v^2 / MaxCentripetalAcceleration
- Used by: Glider

### LocomotionOrientationConstraints

Each movement model defines which Locomotion+Orientation combinations are valid and the maxTurnRate for turning combinations.

**TurnRateMovement constraints:**
- Stationary+Turning: maxTurnRate = TurnRate
- Stationary+Maintaining: valid
- Moving+Turning: maxTurnRate = TurnRate
- Moving+Maintaining: valid
- Stopping+Turning: maxTurnRate = TurnRate
- Stopping+Maintaining: valid

**FixedTurnRadiusMovement constraints:**
- Stationary+Turning: INVALID
- Stationary+Maintaining: valid
- Moving+Turning: maxTurnRate = currentSpeed / MinimumTurnRadius
- Moving+Maintaining: valid
- Reversing+Turning: maxTurnRate = currentSpeed / MinimumTurnRadius
- Reversing+Maintaining: valid
- Stopping+Turning: INVALID
- Stopping+Maintaining: valid

**SpeedTurnRadiusMovement constraints:**
- Stationary+Turning: unconstrained
- Stationary+Maintaining: valid
- Moving+Turning: maxTurnRate = f(currentSpeed, SpeedToTurnRadiusRatio)
- Moving+Maintaining: valid
- Reversing+Turning: maxTurnRate = f(currentSpeed, SpeedToTurnRadiusRatio)
- Reversing+Maintaining: valid
- Stopping+Turning: unconstrained
- Stopping+Maintaining: valid

**DragMovement constraints:**
- Stationary+Turning: maxTurnRate = TurnRate
- Stationary+Maintaining: valid
- Moving+Turning: maxTurnRate = TurnRate
- Moving+Maintaining: valid
- Stopping+Turning: maxTurnRate = TurnRate
- Stopping+Maintaining: valid

**GliderMovement constraints:**
- Moving+Turning: maxTurnRate = f(currentSpeed, MaxCentripetalAcceleration)
- Moving+Maintaining: valid
- (No stationary, stopping, or reversing states — glider must always be moving)

## Justification
Defined in `features/unit_system.md` with constraint tables from `design/combat.md`. Movement models are the core physics layer that determines how each unit navigates. The LocomotionOrientationConstraints are essential for the command/behavior system to know which state combinations are legal for a given unit.

## QA Steps
1. Verify all 5 movement model structs exist with the exact fields listed above.
2. Verify DragMovement does NOT have an explicit MaxSpeed field (it is derived).
3. Verify GliderMovement does NOT have an explicit TurnRadius field (it is derived).
4. Verify LocomotionOrientationConstraint data exists for each of the 5 movement models.
5. For TurnRateMovement: confirm 6 valid combinations (Stationary/Moving/Stopping x Turning/Maintaining), all turning combos use TurnRate as maxTurnRate.
6. For FixedTurnRadiusMovement: confirm Stationary+Turning and Stopping+Turning are marked invalid. Confirm Moving+Turning and Reversing+Turning use currentSpeed/MinimumTurnRadius.
7. For SpeedTurnRadiusMovement: confirm Stationary+Turning and Stopping+Turning are unconstrained. Confirm Moving/Reversing+Turning use f(currentSpeed, SpeedToTurnRadiusRatio).
8. For DragMovement: confirm all turning combos use TurnRate as maxTurnRate (same pattern as TurnRate model).
9. For GliderMovement: confirm only Moving+Turning and Moving+Maintaining are valid (no stationary/stopping/reversing states).

## Expected Experience
Inspecting the code shows 5 distinct movement model structs, each with clearly named numeric fields. A constraint table (or equivalent data structure) for each model cleanly lists which Locomotion+Orientation combinations are valid and what the maxTurnRate formula is for each turning combination. The GliderMovement constraints are notably minimal (only 2 valid combinations).
