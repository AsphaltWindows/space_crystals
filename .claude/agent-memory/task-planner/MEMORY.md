# Task Planner Memory

## Module Structure
- `src/types.rs` — crate-wide shared components/enums (Owner, GridPosition, Selectable, FactionEnum, ObjectEnum, constants)
- `src/game/types/` — game domain types (factions.rs, objects.rs, structures.rs)
- `src/game/world/` — map, resources, faction setup systems
- `src/game/units/` — unit systems, commands, pathfinding
- `src/game/combat/` — combat, turret, projectile systems
- `src/ui/` — HUD, command panel

## Known Issues
- FactionEnum duplicated in `src/types.rs:123` AND `src/game/types/factions.rs:6` — entity_hierarchy task addresses this
- Fixed timestep (16 FPS) is in FactionPlugin instead of a simulation plugin — simulation_core task addresses this

## Patterns
- Bevy 0.14 fixed timestep: `Time::<Fixed>::from_hz(f64)`
- Components derive `Component, Clone, Copy, Debug, PartialEq, Eq, Hash` when appropriate
- Every subsystem has a Plugin struct registered in main.rs
- Marker components are simple `#[derive(Component)] pub struct Name;`

## Tool Quirks
- Write tool requires Read first even for new files — use Bash heredoc for new file creation
