# locomotion-orientation-verify

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-locomotion-orientation-constraints.md

## Task

Verify the existing LocomotionOrientationConstraints implementation in `artifacts/developer/src/game/units/types/movement.rs`. The feature is already fully implemented:

- `Locomotion` enum (Stationary, Moving, Stopping, Reversing) and `Orientation` enum (Turning, Maintaining) for constraint lookup
- `TurnRateConstraint` enum (Invalid, Valid, FixedRate, SpeedDependent, Unconstrained)
- `locomotion_orientation_constraint()` methods on all 5 movement param structs:
  - TurnRateMovementParams: all states valid, FixedRate(turn_rate) when Turning
  - FixedTurnRadiusMovementParams: Stationary+Turning and Stopping+Turning are Invalid, Moving/Reversing+Turning are SpeedDependent (speed / min_turn_radius)
  - SpeedTurnRadiusMovementParams: Stationary/Stopping+Turning are Unconstrained, Moving/Reversing+Turning are SpeedDependent (speed / (speed * ratio))
  - DragMovementParams: all states valid, FixedRate(turn_rate) when Turning
  - GliderMovementParams: only Moving+Turning (SpeedDependent) and Moving+Maintaining are valid, all others Invalid
- `max_turn_rate_at_speed()` on FixedTurnRadius, SpeedTurnRadius, and Glider params
- Extensive tests covering all Locomotion x Orientation combinations for each model

**Verification**: Run `cargo test` and confirm all locomotion_orientation tests pass. Review the constraint tables match the design doc at `artifacts/designer/design/combat.md` (LocomotionOrientationConstraints sections). No new code should be needed.
