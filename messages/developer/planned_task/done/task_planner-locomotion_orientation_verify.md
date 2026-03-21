# locomotion-orientation-verify

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-locomotion-orientation-constraints.md

## Task

Verify the existing LocomotionOrientationConstraints implementation in `artifacts/developer/src/game/units/types/movement.rs`. The feature is already fully implemented:

- `Locomotion` enum (Stationary, Moving, Stopping, Reversing) and `Orientation` enum (Turning, Maintaining) for constraint lookup
- `TurnRateConstraint` enum (Invalid, Valid, FixedRate, SpeedDependent, Unconstrained)
- `locomotion_orientation_constraint()` methods on all 5 movement param structs
- `max_turn_rate_at_speed()` on FixedTurnRadius, SpeedTurnRadius, and Glider params
- Extensive tests covering all Locomotion x Orientation combinations for each model

**Verification**: Run `cargo test` and confirm all locomotion_orientation tests pass. Review the constraint tables match the design doc at `artifacts/designer/design/combat.md` (LocomotionOrientationConstraints sections). No new code should be needed.

## Technical Context

### Primary File
- **`artifacts/developer/src/game/units/types/movement.rs`** — Contains the entire implementation. No other files need changes.

### Key Types (lines 90-130)
- `Locomotion` enum (line 95): Stationary, Moving, Stopping, Reversing
- `Orientation` enum (line 105): Turning, Maintaining
- `TurnRateConstraint` enum (line 112): Invalid, Valid, FixedRate(f32), SpeedDependent, Unconstrained

### Constraint Methods (lines 185-280)
Five `locomotion_orientation_constraint()` implementations, one per movement model struct:
1. **TurnRateMovementParams** (line 190): Reversing=Invalid, Maintaining=Valid, Turning=FixedRate(turn_rate)
2. **FixedTurnRadiusMovementParams** (line 203): Maintaining=Valid, Stationary/Stopping+Turning=Invalid, Moving/Reversing+Turning=SpeedDependent
3. **SpeedTurnRadiusMovementParams** (line 226): Maintaining=Valid, Stationary/Stopping+Turning=Unconstrained, Moving/Reversing+Turning=SpeedDependent
4. **DragMovementParams** (line 249): Same pattern as TurnRate — Reversing=Invalid, Maintaining=Valid, Turning=FixedRate(turn_rate)
5. **GliderMovementParams** (line 266): Only Moving+Maintaining=Valid and Moving+Turning=SpeedDependent; all others Invalid

### Design Doc Reference
- `artifacts/designer/design/combat.md` lines 135-182 — Contains constraint tables for all 5 models under `## LocomotionOrientationConstraints[ModelName]` headers
- **Note**: TurnRate and Drag tables in the design doc omit Reversing states (not listed = not applicable). The code correctly returns `Invalid` for these.
- GliderMovement only lists Moving states in the design doc; code correctly returns Invalid for all non-Moving states.

### Tests (lines 421+, test module)
Comprehensive test coverage already exists:
- `turn_rate_all_maintaining_valid` (731), `turn_rate_all_turning_fixed_rate` (739), `turn_rate_reversing_invalid` (747)
- `fixed_turn_radius_all_maintaining_valid` (755), `fixed_turn_radius_stationary_stopping_turning_invalid` (763), `fixed_turn_radius_moving_reversing_turning_speed_dependent` (771), `fixed_turn_radius_max_turn_rate_at_speed` (778)
- `speed_turn_radius_all_maintaining_valid` (787), `speed_turn_radius_stationary_stopping_turning_unconstrained` (796), `speed_turn_radius_moving_reversing_turning_speed_dependent` (803), `speed_turn_radius_max_turn_rate_at_speed` (810)
- `drag_all_maintaining_valid` (819), `drag_all_turning_fixed_rate` (827), `drag_reversing_invalid` (835)
- `glider_moving_maintaining_valid` (850), `glider_moving_turning_speed_dependent` (856), `glider_non_moving_all_invalid` (862), `glider_turn_radius_derived` (873)

### Verification Steps
1. Run `cargo test -p space_crystals -- locomotion_orientation` (or broader `cargo test`) and confirm all pass
2. Spot-check constraint tables in code vs design doc sections (lines listed above)
3. If all tests pass and tables match: mark task complete with no code changes

## Dependencies

None — this is a standalone verification task with no dependencies on other tasks.
