# game/units/types/

Unit-related type definitions for the Space Crystals RTS.

## Files

- **state/** — Unit command, behavior state, and action channel types (commands, queues, behavior states, locomotion/orientation/attack channels)
- **movement.rs** — Movement types (MoveTarget, Velocity, MovementSpeed, Path, PathNode), 5 movement model parameter structs (TurnRate, FixedTurnRadius, SpeedTurnRadius, Drag, Glider), UnitBaseData with per-base lookup
- **unit_data.rs** — Unit definition types (UnitType, UnitTypeData, TurretAttributesData, AttackAttributesData, UnitControlCost)
- **types.rs** — Shared types: CommandIndicator, CommandIndicatorType, OccupancyMap (tile/AABB collision data), CollisionBody, NeedsRepath marker, indicator helpers
- **utils.rs** — Shared utility functions (convention file)
