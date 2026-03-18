# Units Systems

ECS systems for unit behavior, movement, commands, and selection display.

## Files

- **core.rs** - Core unit systems: movement (legacy), rotation, selection display, right-click command issuing
- **commands.rs** - Command input systems: hotkey handling, hold position, stop, patrol
- **behaviors.rs** - Behavior systems: MovingToLocation, MovingToObject, ReversingToLocation, StoppingBehavior. Write to LocomotionChannel/OrientationChannel action channels.
