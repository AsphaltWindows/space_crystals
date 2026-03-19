# Task Planner Insights

## Codebase Map

### Core Modules
- `artifacts/developer/src/game/` — Core game logic
  - `types/` — Shared type definitions (factions, structures, object enums)
  - `units/` — Unit systems and behaviors
    - `types/state/commands.rs` — `UnitCommand` enum, `CommandType` enum, `is_available()` checks
    - `types/state/behavior.rs` — `BaseBehaviorState` enum (movement models), action channels (`LocomotionChannel`, `OrientationChannel`, `BaseAttackChannel`)
    - `types/unit_data.rs` — Per-unit stats, type data functions (e.g. `agent_type_data()`, `guard_type_data()`)
    - `systems/commands.rs` — Command input handling, hotkey systems
    - `systems/behaviors.rs` — All behavior systems (moving, attacking, entering tunnel, gathering, building, etc.)
  - `world/` — Map, resources, faction initialization
    - `faction.rs` — `setup_syndicate_game_start()`, `setup_gdo_game_start()`
  - `combat/` — Attack states, turrets, projectiles
- `artifacts/developer/src/ui/` — HUD and interface
  - `types.rs` — `ObjectInterfaceState`, `StructureMenuState`, `AgentMenuState`, `SelectedUnitCapabilities`
  - `command_panel.rs` — 3x3 grid layout (Q/W/E, A/S/D, Z/X/C), `get_grid_slot_action()`, `CommandButtonAction`
- `artifacts/developer/src/shared/types.rs` — `Selection`, `SelectionGroup`, `Selected`, `ControlGroups`
- `artifacts/developer/src/simulation/` — Core simulation loop, diagnostics

### Key Types
- **Factions**: `SyndicatePlayerResources` (space_crystals, supplies, tunnel_space), `GdoPlayerResources`
- **Structures**: `TunnelState` (tier, upgrades), `HeadquartersState` (build_queue, rally_point), `TunnelTier` enum
- **Unit Commands**: `UnitCommand` enum — Move, Attack, Enter, Gather, Build, BuildTunnel, etc.
- **Interface States**: `ObjectInterfaceState` — Default, AwaitingTarget(CommandType), StructureMenu(...), AgentMenu(...)
- **Selection**: `Selection` resource with `groups: Vec<SelectionGroup>`, `active_group_index`

### Syndicate Units
- **Agent**: LightInfantry (per forum, may be HeavyInfantry in code — CHECK), melee, gathers resources, builds tunnels, ungroupable
- **Guard**: HeavyInfantry, ranged (3 GU), groupable, produced from HQ

### Plugin Pattern
- Each feature area implements `Plugin` trait, registers systems with `add_systems()`
- Systems use Bevy scheduling phases and diagnostic categories

## Architectural Patterns

1. **Command flow**: Hotkey/click → `CommandButtonAction` → `command_input_system()` → `UnitCommand` dispatched
2. **Behavior flow**: `UnitCommand` converted to `BaseBehaviorState` → behavior systems process each tick → action channels drive locomotion/orientation/attack
3. **Interface state machine**: `ObjectInterfaceState` (resource) drives command panel layout; transitions between Default/AwaitingTarget/StructureMenu/AgentMenu
4. **3x3 grid slots**: Commands mapped to grid positions via `get_grid_slot_action()` — state-dependent
5. **Conditional commands**: `UnitCommand::is_available()` checks unit capabilities (has_attack, can_target_ground, can_reverse, is_syndicate)
6. **Selection groups**: Units grouped by `ObjectEnum` type; ungroupable entities get their own group

## Common Pitfalls
- `update_command_panel_state()` auto-forces structure interface states — must be careful with state preservation
- `rebuild_occupancy_map` doesn't filter by domain — underground structures incorrectly block surface movement
- `is_common_command()` checks action type without considering selection composition — causes incorrect green/yellow tinting

## Dependencies Between Systems
- Command panel state depends on Selection + ObjectInterfaceState
- Behavior systems depend on UnitCommand being dispatched
- Tunnel entry depends on TunnelTier::transit_tier() checks
- Production (HQ) depends on SyndicatePlayerResources and parent Tunnel entity
