# Units Systems

ECS systems for unit behavior, movement, commands, and selection display.

## Directories

- **core/** - Core unit systems: movement (legacy MoveTarget/Path pipeline), channel-driven locomotion/orientation consumers, rotation, selection display, right-click command issuing, rally points, indicators, occupancy
- **behaviors/** - Behavior systems split by domain: movement, tunnel interaction, construction, resource gathering, supply chopper. Write to LocomotionChannel/OrientationChannel action channels.

## Files

- **commands.rs** - Command input systems: hotkey handling, hold position, stop, patrol
