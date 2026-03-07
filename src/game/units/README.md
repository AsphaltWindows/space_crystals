# units/

Unit management including spawning, movement, pathfinding, command handling, collision, and behavior systems.

## Structure

- **types/** — UnitBase, UnitType, movement components, UnitCommand, CommandMode, behavior states, action channels, OccupancyMap (collision/pathfinding), NeedsRepath marker
- **utils.rs** — Grid conversion, pathfinding helpers, path smoothing, attack capability creation, movement state clearing
- **systems/** — Unit systems: core (occupancy rebuild, movement with collision, rotation, selection, repath), commands (input, hold, stop, patrol), behaviors (MovingToLocation, MovingToObject, ReversingToLocation, StoppingBehavior)
- **pathfinding.rs** — A* pathfinding algorithm with occupancy-aware tile blocking
