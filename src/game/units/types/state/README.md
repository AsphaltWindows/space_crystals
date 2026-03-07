# game/units/types/state/

Unit command, behavior, and action channel types for the Space Crystals RTS.

## Files

- **commands.rs** — Unit command types (UnitCommand, CommandMode, CommandType, CommandQueue, BaseCommandState, TurretCommandState), unit state types (UnitState, HoldingPosition)
- **behavior.rs** — Behavior state types (BaseBehaviorState, TurretBehaviorState) and action channel components (LocomotionChannel, OrientationChannel, BaseAttackChannel, TurretOrientationChannel, TurretAttackChannel)
- **types.rs** — Shared data components (AgentCarryState — tracks crystals/supplies carried by Agent units)
- **utils.rs** — Shared utility functions (convention file)
