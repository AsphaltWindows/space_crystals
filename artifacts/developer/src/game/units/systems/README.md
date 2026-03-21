# Units Systems

ECS systems for unit behavior, movement, commands, and selection display.

## Files

- **core.rs** - Core unit systems: movement (legacy MoveTarget/Path pipeline), channel-driven locomotion/orientation consumers (LocomotionChannel/OrientationChannel pipeline), rotation, selection display, right-click command issuing
- **commands.rs** - Command input systems: hotkey handling, hold position, stop, patrol
- **behaviors.rs** - Behavior systems: MovingToLocation, MovingToObject, ReversingToLocation, StoppingBehavior, GatheringResource, DroppingOffResources, BuildingTunnel, EnteringTunnel, Building, and Supply Chopper behaviors (PickUp, Attach, DropOff, Detach, Repair). Write to LocomotionChannel/OrientationChannel action channels.
