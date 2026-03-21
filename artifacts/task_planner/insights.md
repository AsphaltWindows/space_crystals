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
- `rebuild_occupancy_map` NOW correctly filters by domain — underground structures are skipped (line 1094 checks `DomainEnum::Underground`). No existing system-level tests for this function though.
- `is_common_command()` checks action type without considering selection composition — causes incorrect green/yellow tinting
- Escape handler in `command_panel_hotkeys` (line ~872) must consider active selection type when choosing return state — generic `Default` return is wrong when agents are selected
- `Selection::active_type()` returns `Option<ObjectEnum>` — standard way to detect what kind of unit/structure is in the active group

## right_click_move_command Structure (core.rs)
- Line 179: function signature — 11 query/resource params
- Lines 196-249: Early returns (no click, cursor over UI, placement mode, SetRallyPoint, ScheduleDeliveries)
- Lines 256-390: Entity-click section (attack, chopper targets, agent targets)
- Lines 392-583: Ground-click section (move, reverse, enter, build, gather/dropoff fallthrough)
- Line 1375: Test module begins
- `AgentMenuState` must be explicitly imported in core.rs (not in the default import line 10)
- Agent-specific AwaitingTarget modes (DropOff, Gather, Enter) should return to AgentMenu(AgentDefault), not Default

## Placement Validation
- `can_worker_place_structure()` in `artifacts/developer/src/game/world/utils.rs` (line 328) — validates tiles are buildable and no structure overlap
- Requires `Query<(&GridPosition, &TilePreset), With<Tile>>` and `Query<(&GridPosition, &StructureInstance, &ObjectInstance)>`
- `world_to_grid(pos, 1.0)` maps Vec3(0.5, 0, 0.5) → grid (32, 32) due to half_size=32 offset
- Tunnel size is (4, 4), NOT (1, 1)
- Existing `building_behavior_arrived_structure_overlap_cancels` test (line 1443) spawns structure without `ObjectInstance` — likely pre-existing test bug (entity won't match the structures query)
- Tests using `App::new()` need `MinimalPlugins` + `Assets<Mesh>` + `Assets<StandardMaterial>` init for `building_tunnel_behavior_system`

## Tunnel Interface Implementation Map
- All 4 Tunnel interface states (TunnelIdle, TunnelExpandMenu, TunnelEjectMenu, TunnelAwaitingPlacement) are fully implemented
- Static grid layout in `get_grid_slot_action()` (command_panel.rs ~112), dynamic grids via `build_tunnel_expand_grid()` / `build_tunnel_eject_grid()`
- Ejection pipeline: `EjectRequest` marker → `ejection_tick_system` (faction.rs ~1816) → `EjectionQueue` with 8-frame cooldown → spawn at Side A
- Upgrade cost functions: `tunnel_t2_upgrade_cost()`, `tunnel_t3_upgrade_cost()` in command_panel.rs
- ~30 unit tests covering grid slots, labels, placement mode, ejection queue, panel visibility

## Command-to-Behavior Dispatch Gap
- Right-click handler (`core.rs`) inserts `UnitCommand::Enter(entity)` but does NOT insert behavior markers
- Same gap exists for `BuildTunnel` — `BuildingTunnelBehavior` is only present in test spawns, never inserted at runtime
- Pattern: a separate dispatch system should read `UnitCommand` and insert the corresponding behavior marker component
- ObjectEnum → UnitBaseEnum mapping: `SyndicateAgent => agent_type_data().unit_base`, `SyndicateGuard => guard_type_data().unit_base` (see command_panel.rs:1870-1873)
- ObjectEnum → is_syndicate: match `SyndicateAgent | SyndicateGuard` (used throughout core.rs)

## Camera System (main.rs)
- `GamePlugin` in `main.rs` registers camera systems in `DiagCategory::Camera` set
- Camera spawns at (0, 40, 25) looking at origin — angled view, not top-down
- `update_camera_viewport` sets viewport to exclude HUD (top 32px, bottom 220px)
- `camera_movement` handles arrow key panning
- `viewport_to_world` ray-casting used in map.rs, command_panel.rs, faction.rs — works with both perspective and orthographic projections
- `TestApp` (shared/testing/test_app.rs) spawns a dummy MainCamera — needs to stay in sync with main camera components
- World coordinate: 1 GridUnit = 1.0 world unit (cell_size=1.0), 64x64 grid centered at origin

## Dependencies Between Systems
- Command panel state depends on Selection + ObjectInterfaceState
- Behavior systems depend on UnitCommand being dispatched
- Tunnel entry depends on TunnelTier::transit_tier() checks
- Production (HQ) depends on SyndicatePlayerResources and parent Tunnel entity
- `InTunnelNetwork` filter: `air_unit_separation_system` uses `Without<InTunnelNetwork>` (core.rs:1353)

## Entity-Click Handler Order in right_click_move_command (core.rs)
- Line 260: Attack (right-click enemy or AwaitingTarget[Attack])
- Line 287: AwaitingTarget[DropOff] left-click entity
- Line 313: Chopper right-click (SDS, SupplyTower)
- Line 352: Agent right-click (Crystal, SDS, own Tunnel → DropOff/Enter)
- Line 415: Fall-through to ground Move
- New Enter handlers should be inserted: AwaitingTarget[Enter] after DropOff handler; Guard right-click Enter after Agent block
- `selected_units` query tuple: (Entity, &Transform, &UnitBaseEnum, &Owner, Option<&AttackState>, Option<&SupplyChopperState>, &ObjectInstance, Option<&AgentCarryState>)
- `target_info` query tuple: (Option<&SupplyDeliveryStation>, Option<&SupplyTowerState>, &Owner, Option<&SpaceCrystalPatch>, Option<&TunnelState>)

## Resource Nodes Implementation
- ObjectType definitions in `game/types/objects.rs` lines 302-315 (SCP: 1x1, SDS: 2x2, both indestructible, sight_range 0)
- SCP spawn: `game/world/resources.rs` lines 11-76 — single tile, no SightRange component
- SDS spawn: `game/world/resources.rs` lines 542-599 — marks only 1 tile non-traversible despite 2x2 size
- SDS delivery timer: `game/world/resources.rs` lines 602-617 — countdown only when supplies==0
- Extraction plate mining: `game/world/faction.rs` ~line 504 — decrements remaining_amount
- Agent gathering does NOT decrement remaining_amount (behaviors.rs line 688-689)
- Info panel for resources: `ui/hud.rs` lines 669-815 — no FogOfWarMap visibility checks
- Depleted patch despawn only on plate destruction (combat/systems/core.rs:573-576), not on depletion itself

## Control/Selection Systems
- `control_group_system` (resources.rs:645): handles Ctrl+Num assign, Shift+Num add, plain Num recall. Branch order matters for modifier combos.
- `active_group_cycle_system` (resources.rs:811): Tab cycling for resource-only groups. Commandable groups' Tab cycling handled in `command_panel_hotkeys` (command_panel.rs:852).
- `Selection::cycle_active_group()` (shared/types.rs:220): forward cycling. No backward method yet.
- System ordering: control_group_system -> selection_group_sync_system -> active_group_cycle_system (all in Update/DiagCategory::Faction)

## Command-to-State Pipeline
- `CommandType` enum (commands.rs:76) is MISSING `HoldPosition` and `Stop` variants — these need to be added for the command-state mapping to work
- `BaseCommandState` is spawned on all units (utils.rs:472) but never populated at runtime — no sync system exists yet
- `CommandQueue` is spawned on all units (utils.rs:471) but nothing dequeues from it — no dequeue system exists yet
- Behavior completion: many behaviors set `UnitCommand::Idle` on completion (building, gathering, dropping off, building tunnel), but `moving_to_location_system` does NOT — it explicitly says a "separate completion system" must handle it (behaviors.rs:128-131)
- `PickUpSupplies` and `AttachToTower` UnitCommand variants have no corresponding `CommandType` entries — chopper-specific, may need new variants or map to Default
- New systems should go in `CommandsPlugin` (mod.rs:57) under `DiagCategory::Commands`

## Interface State Management
- `update_command_panel_state` (command_panel.rs:281) forces structure/agent menu states based on active Selection group type — acts as a "convenience" layer
- `ObjectInterfaceState` is initialized in `HudPlugin` (ui/mod.rs:18)
- Selection-related systems in `FactionPlugin` (world/mod.rs:73-91) run in `DiagCategory::Faction`; UI systems in `DiagCategory::UiHud` — no cross-set ordering by default
- `StructureMenuState` and `AgentMenuState` must be imported from `crate::ui::types` when used in resources.rs (only `ObjectInterfaceState` is currently imported on line 5)
- System order chain: selection_validation → selection_group_sync → active_group_cycle → [new reset/validation systems] → update_command_panel_state

## Command Issuing Code Paths
- `right_click_move_command` (core.rs:179): does NOT have keyboard access — only `ButtonInput<MouseButton>`. 13+ UnitCommand insertion points across entity-click and ground-click branches.
- `hold_position_system` (commands.rs:65) and `stop_command_system` (commands.rs:99): both have `keyboard: Res<ButtonInput<KeyCode>>` already.
- `execute_command_action` (command_panel.rs:1107): called from both `handle_command_button_clicks` (mouse, no keyboard) and `command_panel_hotkeys` (keyboard, already computes `shift_held` at line 851). Only HoldPosition and Stop branches issue commands directly; other unit commands just set AwaitingTarget mode.
- `bevy::ecs::world::CommandQueue` (Bevy internal) vs game's `CommandQueue` (commands.rs:150) — completely different types, both appear in test code. Watch for namespace confusion.

## Behavior System Architecture
- **Unit behaviors** (units/systems/behaviors.rs, 2750 lines): 4 movement systems write to action channels (LocomotionChannel, OrientationChannel). Use `BaseBehaviorState` variants for path tracking.
- **Combat behaviors** (combat/systems/behaviors.rs, 728 lines): 5 combat systems use `MoveTarget`/`Path` components for approach (legacy movement system) rather than action channels for locomotion.
- **Two movement pipelines**: Unit behaviors write to action channels (newer pattern), combat behaviors use `MoveTarget`/`Path` components (older pattern). Both coexist. `unit_movement_system` reads `MoveTarget`/`Path`; `turn_rate_movement_system` also reads `MoveTarget`/`Path` (NOT channels yet). Neither existing movement system reads from channels.
- **Channel consumer gap**: No system currently reads `LocomotionChannel`/`OrientationChannel` to drive movement. Task `action-channel-locomotion-orientation` creates these consumer systems.
- **LocomotionChannel path tracking**: `LocomotionChannel::Moving(Vec<Vec3>)` stores full path but no index. Behaviors rewrite channels every tick, so consumers can treat `path[0]` as current target.
- **Constraint methods**: Each movement param struct has `locomotion_orientation_constraint(loco, orient) -> TurnRateConstraint`. FixedTurnRadius prohibits stationary/stopping + turning. SpeedTurnRadius allows unconstrained turning when stationary.
- **AttackMove leash**: Implementation uses radial distance from origin, not perpendicular distance from path (spec simplification).
- **Patrol engagement**: Directly switches to `AttackTarget` command rather than delegating to AttackMove sub-behavior (no leash during patrol engagement).
- **Moving completion gap**: `moving_to_location_system` does NOT set `UnitCommand::Idle` — a separate completion system is needed (see comment at behaviors.rs:128-131).

## Combat-Channel Integration Points
- `attack_phase_system` (core.rs:38) drives AttackState through None→Aiming→Firing→Cooldown→Reloading. Does NOT write to any channel components yet.
- `turret_autonomous_scanning_system` (core.rs:265) sets `AttackState` on turret units. Does NOT write to `TurretAttackChannel`.
- `base_auto_target_system` (core.rs:355) sets `AttackState` on non-turret idle units. Does NOT write to `BaseAttackChannel`.
- `turret_aiming_system` (turret.rs:6) reads AttackState to set Turret.target_angle. Does NOT write to `TurretOrientationChannel`.
- Non-turret units spawn with `BaseAttackChannel` but NOT `TurretAttackChannel`/`TurretOrientationChannel`.
- Turret units spawn with `TurretAttackChannel`+`TurretOrientationChannel`+`TurretCommandState`+`TurretBehaviorState` but NOT `BaseAttackChannel`.
- Use `With<Turret>`/`Without<Turret>` filter to distinguish turret from non-turret units in queries.

## EF Flat Interface Pattern
- EF interface rework eliminates `EfConstructing`/`EfReadyToPlace` states, keeping only `EfIdle` and `EfAwaitingPlacement`
- Dynamic Q button in EfIdle: uses `has_ready_plate` bool to switch between `EfBuildPlate` and `EnterPlacement`
- `has_active_construction` already handles conditional X=EfCancel in the grid
- ~15 change sites in command_panel.rs, plus types.rs, resources.rs, faction.rs
- EfIdle info panel (lines 528-540) already handles both constructing and ready_to_place display

## SupplyChopper Interface Pattern
- SupplyChopper uses `ObjectInterfaceState::Default` (not a dedicated menu state like AgentMenu)
- `get_grid_slot_action()` does NOT receive `active_type` — to add chopper-specific grid, either pass it as param or add `is_chopper` to `SelectedUnitCapabilities`
- `update_command_panel_state()` line 410-416: choppers fall into the else branch (non-agent, non-structure) → Default state
- New `CommandButtonAction` variants for choppers should use `Chopper` prefix (e.g., `ChopperPickUpSupplies`)
- AwaitingTarget resolution for chopper commands returns to `Default` (not `AgentMenu`)
- `target_info` query in core.rs already includes `Option<&SupplyDeliveryStation>` and `Option<&SupplyTowerState>` — no query changes needed for AwaitingTarget handlers

## SupplyChopper Behavior Architecture
- Chopper spawns with `DragMovementParams`, `LocomotionChannel`, `OrientationChannel`, `BaseBehaviorState`, `SupplyChopperState` — all needed for channel-based movement (utils.rs:892-928)
- No `AttackCapability`/`AttackState`/attack channels — unarmed unit (utils.rs:928)
- Right-click issues `UnitCommand::PickUpSupplies`/`AttachToTower` (core.rs:349-392) but NO behavior markers are inserted yet — that's needed
- `command_state_sync_system` maps both to `CommandType::Default` (commands.rs:215-216) — no dedicated CommandType variants
- `SupplyDeliveryStation.current_supplies` is the supply pool; SDS is not a Bevy `StructureInstance` (it's a resource node with `ObjectInstance::indestructible`)
- `GdoPlayerResources.supplies` (factions.rs:89) is where delivered supplies land — NOT `space_crystals`
- `SC_MAX_HP = 150.0` (structures.rs:431)
- `SupplyTowerState.attached_chopper` and `SupplyChopperState.attached_tower` form a bidirectional link

## PointerDisplayType System
- New `PointerDisplayType` enum + resource in `ui/types.rs`, init in `ui/mod.rs`
- `resolve_pointer_display_type` system in `command_panel.rs`, runs after `update_command_panel_state`
- Mirrors right-click resolution logic from `core.rs:179-583` but read-only (no commands issued)
- Reads: `ObjectInterfaceState`, `CursorTarget`, `Selection`, `SelectedUnitCapabilities`, `LocalPlayer`
- Sibling task `pointer_display_rendering` consumes the resource for visual updates

## Directive: Never Run `cargo clean`
- Always use incremental builds. Diagnose build issues directly instead of wiping cache.
