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
  - `command_panel/` — Grid-based command panel (directory split from monolithic file)
    - `grid_layout.rs` — `GRID_HOTKEYS` constant, `keycode_to_grid_slot()`, `get_grid_slot_action()` (state-dependent slot→action mapping)
    - `hotkeys.rs` — `command_panel_hotkeys()` system, `handle_command_button_clicks()`, `right_click_cancel_submenu()`
    - `ui_rebuild.rs` — `rebuild_command_panel_ui()` system (builds grid UI nodes)
    - `tunnel_actions.rs` — `build_tunnel_expand_grid()`, `build_tunnel_eject_grid()` (dynamic tunnel menu grids)
    - `actions.rs` — `execute_command_action()` (dispatches `CommandButtonAction` to game logic)
    - `helpers.rs` — `spawn_grid_button()`, `spawn_empty_grid_cell()`, label/enable helpers
    - `state.rs` — `is_panel_visible()`, panel state management
    - `pointer.rs` — Pointer/cursor interaction
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
4. **Grid slots**: Commands mapped to grid positions via `get_grid_slot_action()` — state-dependent. Being expanded from 3x3 to 4x3 (Q-W-E-R / A-S-D-F / Z-X-C-V). Tunnel dynamic grids in tunnel_actions.rs use slot/cols and slot%cols math — must update when column count changes.
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

## Entity-Click Handler Order in right_click_move_command (command_dispatch.rs)
- Line 96: Attack (right-click enemy or AwaitingTarget[Attack])
- Line 123: AwaitingTarget[DropOff] left-click entity
- Line 155: AwaitingTarget[Enter] left-click entity
- Line 183: AwaitingTarget[PickUpSupplies] left-click entity
- Line 215: AwaitingTarget[AttachToTower] left-click entity
- Line 247: AwaitingTarget[DropOffSupplies] left-click entity
- Line 270: Chopper right-click smart-command (SDS, SupplyTower)
- Line 343: Agent right-click smart-command (Crystal→Gather, SDS→Gather, own Tunnel→DropOff/Enter)
- Line 440: Recruit right-click Armory→EnterArmory
- Line 468: Guard right-click own Tunnel→Enter
- Line 502: Fall-through to ground Move/command handler
- **Missing**: No left-click entity handler for AwaitingTarget[Gather] mode — falls through to ground handler which resets mode
- `selected_units` query: (Entity, &Transform, &UnitBaseEnum, &Owner, Option<&AttackState>, Option<&SupplyChopperState>, &ObjectInstance, Option<&AgentCarryState>, &mut CommandQueue)
- `target_info` query: (Option<&SupplyDeliveryStation>, Option<&SupplyTowerState>, &Owner, Option<&SpaceCrystalPatch>, Option<&TunnelState>, Option<&ArmoryState>)

## Resource Nodes Implementation
- ObjectType definitions in `game/types/objects.rs` lines 302-315 (SCP: 1x1, SDS: 2x2, both indestructible, sight_range 0)
- SCP spawn: `game/world/resources.rs` lines 11-76 — single tile, no SightRange component
- SDS spawn: `game/world/resources.rs` lines 542-599 — marks only 1 tile non-traversible despite 2x2 size
- SDS delivery timer: `game/world/resources.rs` lines 602-617 — countdown only when supplies==0
- Extraction plate mining: `game/world/faction.rs` ~line 504 — decrements remaining_amount
- Agent gathering does NOT decrement remaining_amount (behaviors.rs line 688-689)
- Info panel for resources: `ui/hud.rs` lines 669-815 — no FogOfWarMap visibility checks
- Depleted patch despawn only on plate destruction (combat/systems/core.rs:573-576), not on depletion itself

## Test Harness Infrastructure
- `TestHarness::set_selection()` (shared/testing/harness.rs:264) must update BOTH `Selected` ECS markers AND the `Selection` resource groups
- `Selection::build_from_entities()` takes `&[(Entity, ObjectEnum, bool)]` — groupability comes from `ObjectEnum::object_type().groupable`
- `ObjectInstance` component (objects.rs:55) stores `object_type: ObjectEnum` on each entity — read this in set_selection to build groups
- Without Selection resource sync, `update_command_panel_state` reads empty `selection.groups`, returns `active_type() == None`, resets interface state to Default
- `spawn_unit_at_grid` does NOT insert `VisibilityStateEnum` on spawned entities — `base_auto_target_system` queries `&VisibilityStateEnum` as required component, so targets without it are completely invisible to auto-targeting
- `spawn_peacekeeper` (game/utils.rs:399) does NOT insert `VisibilityStateEnum` — this is a real-game spawn function, visibility is set by fog-of-war systems at runtime
- For tests, must insert `VisibilityStateEnum::Visible` on spawned entities if combat/targeting is being tested
- Stop on stationary unit completes instantly: `stopping_behavior_system` checks velocity < 0.01, sets Stationary; `behavior_completion_system` then transitions Stop -> Idle in same frame

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
- **Behavior completion race**: `behavior_completion_system` runs unordered with behavior systems in Phase 2. If it runs before a behavior system initializes `BaseBehaviorState` from `None` to a real variant, it will prematurely complete the command. Guard: check `BaseBehaviorState::None` and skip. This is more robust than `.after()` ordering since it handles the semantic issue regardless of scheduling.

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

## Declarative Entity Definitions System
- Architecture doc: `artifacts/software_architect/architecture/declarative-entity-definitions.md`
- Foundation: `src/game/entities/mod.rs` (traits + types + registry), `src/game/entities/registry.rs` (builder + plugin)
- Trait hierarchy: `EntityDefinition` (base) → `UnitDefinition` / `StructureDefinition` (Option B from arch doc)
- `EntityRegistry` is a Bevy `Resource` with `HashMap<ObjectEnum, RegistryEntry>`, inserted at startup via `EntitiesPlugin`
- Directory structure: `entities/{gdo,syndicate,cults,neutral}/{units,buildings}/` — one file per entity
- Wired into `src/game/mod.rs` as `pub mod entities;` and `EntitiesPlugin` added to main.rs plugin list
- Pending migration tasks: pilot (peacekeeper), GDO units, GDO buildings, Syndicate units — all depend on foundation
- **Peacekeeper pilot spawn caveat**: `EntityDefinition::spawn()` takes `&mut EntityCommands` (not `&mut Commands`), and `SpawnParams` lacks mesh/material asset handles. Spawn impl should insert all gameplay ECS components; visual components (Mesh3d, MeshMaterial3d) need caller-side or separate system handling.
- Peacekeeper command slots use `ObjectInterfaceState::Default` (standard unit): Move(0,0), Attack(0,2), AttackMove(1,0), Patrol(1,1), AttackGround(1,2), HoldPosition(2,0), Stop(2,1). No UnitReverse (LightInfantry can't reverse).
- `SupplyTowerState.attached_chopper` and `SupplyChopperState.attached_tower` form a bidirectional link

## PointerDisplayType System
- New `PointerDisplayType` enum + resource in `ui/types.rs`, init in `ui/mod.rs`
- `resolve_pointer_display_type` system in `command_panel.rs`, runs after `update_command_panel_state`
- Mirrors right-click resolution logic from `core.rs:179-583` but read-only (no commands issued)
- Reads: `ObjectInterfaceState`, `CursorTarget`, `Selection`, `SelectedUnitCapabilities`, `LocalPlayer`
- Sibling task `pointer_display_rendering` consumes the resource for visual updates

## HUD Bar Content System Architecture
- `update_selected_units_grid_system` (hud.rs:312-973) is a monolithic 660-line system handling all selection display
- 5-section scaffold already implemented: Minimap, Portrait, InfoPanel, SelectionPanel, CommandPanel
- `UnitsGridSection` marker is a backward-compat shim on `InfoPanelSection` entity (hud.rs:159) — to be removed
- `hud_section_visibility_system` (hud.rs:218-253) correctly manages section visibility, no changes needed
- `selection_portrait_click_system` (hud.rs:1215) handles click on `SelectionPortrait` components — must run after SelectionPanel system
- Health bar update (no-rebuild path) at lines 958-972: split between InfoPanel (single-select) and SelectionPanel (multi-select)
- Helper functions: `spawn_stat_text` (line 1473), `spawn_selection_portrait` (line 978), `spawn_resource_portrait` (line 1127)

## Movement System Architecture (MoveTarget/Path pipeline)
- All MoveTarget/Path-driven movement systems live in `game/units/systems/core.rs`, registered in Phase 3 of `UnitsPlugin` (mod.rs:43-55)
- Each movement model gets its own system, filtered by its param component type (e.g., `&TurnRateMovementParams` in query)
- `unit_movement_system` is the fallback — uses `Without<TurnRateMovementParams>` (and should also exclude other param types) + `&MovementSpeed`
- Query exclusion pattern: systems exclude each other via `Without<OtherParamsType>` to prevent double-processing
- `grid_position_sync_system` must have `.after()` constraints for ALL movement systems
- `channel_fallback_locomotion_system` also needs `Without<>` for all specific param types
- Shared constants: `WAYPOINT_ARRIVAL_THRESHOLD = 0.5` (core.rs:21)
- Ground collision pattern: `occupancy.check_movement_collision(entity, x, z, half_w, half_h)` → on collision, remove Path, insert NeedsRepath, zero velocity

## Cults Faction Structure
- First Cults structure: RecruitmentCenter (4x4, AAAA, primary structure like DC)
- `setup_cults_game_start()` (faction.rs:172) spawns RC at grid (50,50) with full Commands/mesh/material params
- `CultsPlayerResources` (factions.rs:171) has `unit_control_used`/`unit_control_available` fields ready for RC integration
- No `cults_structure_stats` module exists yet — will be added alongside `gdo_structure_stats` and `syndicate_structure_stats` in structures.rs
- Cults faction color: `Color::srgb(0.5, 0.2, 0.6)` (purple) — shared/types.rs:298
- `RecruitmentCenterCounter` Resource needed for build_order priority (first-built-first-claim semantics per design doc)
- `RecruitmentCenterState` (structures.rs:494) already has all fields: `claimed_tiles`, `effectiveness`, `local_capacity`, `local_used`, `production_progress`, `build_order`, `rally_point`
- `spawn_recruitment_center()` (utils.rs:947) takes build_order param, spawns with GridPosition, ObjectInstance, StructureInstance
- `TileClaimMap` resource to be added in `game/world/types.rs` — HashMap<(i32,i32), Entity>
- 10x10 area: RC at GridPosition(x,z) → area from (x-3, z-3) to (x+6, z+6) inclusive
- Tile recruitable: all TilePresetEnum except Water. Query: `(&GridPosition, &TilePreset), With<Tile>`
- `remove_dead_entities_system` (combat/systems/core.rs:757) despawns dead entities — claiming system should self-heal stale claims

## RC Production System Notes
- RC production uses integer frame counting (`production_progress: u32`), NOT float progress like barracks
- RC `rally_point` is `Option<Vec3>` — simpler than `Option<RallyTarget>` used by Barracks/HQ
- RC has no build queue — continuous auto-production, one at a time
- No power ratio for Cults — production rate scales by `effectiveness` only
- `CultsRecruit` ObjectEnum variant needs to be added (doesn't exist yet)
- `OriginatingCenters` component shared between production task and unit_control_tracking task — whichever lands first defines it
- HUD TheCults branch (hud.rs:1343-1356) already displays UC from CultsPlayerResources — just needs aggregation system to populate it

## RC Interface Implementation Notes
- RC interface follows HeadquartersMenu pattern: single StructureMenuState, cancel + rally grid
- RC `rally_point` is `Option<Vec3>` (NOT `Option<RallyTarget>`) — must convert RallyTarget to Vec3 in rally systems
- `set_rally_point_click_system` (core.rs:952) unconditionally returns to `Default` after setting rally — needs RC-aware return logic
- `right_click_cancel_target` (command_panel.rs:1059) + `RallyTargetKind` enum needs RC variant for correct Escape/right-click return
- `update_command_panel_state` query on `selected_structures` (line 300) needs `Option<&RecruitmentCenterState>` added
- `production_rally_point_system` (faction.rs:703) handles right-click rally for BK/HQ/ST — needs RC branch
- `bk_has_queue` param in `get_grid_slot_action` is overloaded for multiple cancel buttons — consider renaming to `has_cancel_target` or adding `rc_has_production` param

## Cults Recruit Interface Pattern
- CultsRecruit needs its own `ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState)` — separate from AgentMenu
- CultsRecruitMenuState: RecruitDefault, RecruitConstructMenu, RecruitAwaitingPlacement
- Cults placement validation uses `can_worker_place_structure()` (same as Agent tunnel — no build area, no fog)
- `ObjectEnum::CultsRecruit` may not exist yet — added by `recruitment_center_auto_production` task
- `is_unit()` in objects.rs needs CultsRecruit added
- `UnitCommand::ConstructBuilding(Entity)` is the walk-to-site command for Cults building
- `CommandType::AssistConstruction` for the assist construction AwaitingTarget mode
- `placement_click_system` (faction.rs:1433) needs a `selected_recruits` query added for Cults placement
- Cults buildings cost Recruits (consumed on completion), not crystals — no resource deduction at placement time
- `spawn_cults_storage()` exists (utils.rs:988); need `spawn_cults_storage_under_construction()` variant

## Cults Construction System Notes
- `CultsConstructionState` component: assigned_recruits (Vec<Entity>), construction_progress (u32), total_construction_frames (u32)
- `UnitCommand::ConstructBuilding(Entity)` does NOT exist yet — must be added to commands.rs
- Generic `construction_hp_tick_system` (faction.rs:946) must be filtered with `Without<CultsConstructionState>` to avoid double-progression
- Recruits are hidden (Visibility::Hidden) while inside construction, NOT despawned — allows cancellation refund
- On completion: recruits despawned; on cancel/destruction: recruits restored with Visibility::Inherited + UnitCommand::Idle
- Cancel system must run BEFORE `remove_dead_entities_system` (combat/mod.rs:43) — same ordering as `cults_unit_death_tracking_system`
- `spawn_cults_recruit()` (utils.rs:1043) currently spawns with minimal components — no LocomotionChannel/OrientationChannel
- Registration: cults_construction_tick_system in FixedUpdate/DiagCategory::Construction (world/mod.rs:109-122)

## Armory Structure Notes
- `SymmetryTypeEnum::ABCB` does NOT exist yet — must be added to enum in shared/types.rs:446
- ABCB allows non-square sizes (like ABAB/ABAC/ABCD) — do NOT add to `requires_square` match in validate_size()
- `RallyPointTarget` does NOT exist — use `RallyTarget` (structures.rs:57)
- `ArmoryState` goes in structures.rs after RecruitmentCenterState (~line 528)
- Constants go in `cults_structure_stats` module (structures.rs:468)
- `spawn_cults_armory()` follows `spawn_cults_storage()` pattern (utils.rs:988) — same 3x2 with rotation/flip

## Entity Definition Migration — Structure Pattern
- Structures use `StructureDefinition` trait (not `UnitDefinition`) — `register_structure::<T>()` in registry
- `EntityDefinition::spawn()` takes `&mut EntityCommands` — insert gameplay components only, skip Mesh3d/MeshMaterial3d (no asset handles in SpawnParams)
- `SpawnParams` lacks extra fields like `attached_patch: Entity` (ExtractionPlate) — spawn impl may need Entity::PLACEHOLDER or skip state component
- Structures with rotation: use `params.rotation.unwrap_or_default()` for StructureInstance
- Command slots: include ALL states (DC has 4 states = many slots); Inert structures (PP, EP) return `&[]`
- GDO buildings mod.rs at `entities/factions/gdo/buildings/mod.rs` — add `pub mod` for each new file
- ExtractionPlate cost comes from `ExtractionFacilityState::construction_cost()` (75 crystals, 96 frames), NOT from DC

## Armory Enter Mechanic Pattern
- `UnitCommand::Enter` is gated by `is_syndicate` in `is_available()` — Cults units need a separate `EnterArmory(Entity)` variant
- `spawn_cults_recruit()` (utils.rs:1043) lacks `LocomotionChannel`/`OrientationChannel`/`Velocity` — must be added for any movement behavior to work
- `target_info` query in `right_click_move_command` (core.rs:249) needs `Option<&ArmoryState>` added — affects ~10+ destructure sites
- Pattern for new enter behaviors: dispatch system (validate + insert marker) → behavior system (move + arrive + hide/store)
- Armory stores recruits in `ArmoryState.stored_recruits: Vec<Entity>` (not a network like tunnels)

## Armory Interface & Production Notes
- `ArmoryMenu` StructureMenuState needed — follows Barracks/HQ single-state pattern
- `execute_command_action` needs `CultsPlayerResources` query added (not currently accessible) + separate `armory_query` to avoid borrow conflicts with `bk_hq_query`
- `ArmoryEjectionQueue` component (separate from tunnel `EjectionQueue`) for the eject-all mechanic
- `CultsSoldier`/`CultsGunner` ObjectEnum variants needed as stubs for training output
- Armory `rally_point` is `Option<RallyTarget>` (like Barracks), NOT `Option<Vec3>` (like RC) — can reuse Barracks rally pattern directly
- `RallyTargetKind::Armory` variant + `StructureMenuState::ArmoryMenu` in `production_rally_point_system` is_production_menu check
- Training consumes a stored recruit (pop from stored_recruits) AND deducts crystals; spawns a NEW entity on completion
- `find_cults_resources_mut` helper needed (mirrors `find_syndicate_resources_mut`)

## Shift-Queue Command Dispatch Gap
- Ground-click handlers (core.rs:740-880) compute paths inline for non-shift commands but only push UnitCommand to CommandQueue for shift commands
- `command_dequeue_system` pops UnitCommand but no system then computes path/inserts MoveTarget+Path for dequeued movement commands
- Fix pattern: `move_command_dispatch_system` with `Without<MoveTarget>` filter, runs after `command_dequeue_system`, computes path for Move/AttackMove/Patrol/Reverse
- Affects all ground-click movement commands (Move, Patrol, AttackMove, Reverse) equally
- Entity-click commands (Gather, Enter, etc.) already use `issue_or_queue_command` correctly and their dispatch systems handle the behavior setup

## Attack Command Approach Movement Pattern
- `right_click_move_command` (core.rs:339-358) issues `UnitCommand::AttackTarget` + clears movement state, but does NOT compute approach path inline
- Move command (core.rs:730-768) does compute path inline — both MoveTarget+Path and UnitCommand inserted in same deferred batch
- `attacking_object_behavior_system` (combat/systems/behaviors.rs:22) handles approach pathing for AttackTarget, runs in Combat (after Movement)
- Two-step approach relies on Bevy's auto `apply_deferred` between Movement and Combat sets
- If command timing gap is the bug, fix by inlining initial approach path in the attack branch (matching Move pattern)
- `attack_channel_sync_system` (combat/core.rs:575) forces LocomotionChannel::Stationary during Aiming/Firing/Cooldown — safe during AttackPhase::None (approach)
- `behavior_completion_system` only matches Move/Reverse/Stop — does NOT interfere with AttackTarget
- `DiagCategory::Commands` is unordered relative to Movement and Combat — potential source of timing issues

## HUD Bar Layout Rework
- Current bottom panel: 3 sections — Minimap (200px) | UnitsGridSection (flex-grow) | CommandPanel (200px)
- Target: 5 fixed sections — Minimap (200px) | Portrait (150px) | InfoPanel (flex-grow) | SelectionPanel (250px) | CommandPanel (200px)
- `setup_hud` in hud.rs lines 65-161 builds the bottom panel
- `update_selected_units_grid_system` (hud.rs:223) queries `With<UnitsGridSection>` — during scaffold phase, InfoPanel must carry BOTH `InfoPanelSection` and `UnitsGridSection` markers
- Visibility toggle: use `Visibility::Hidden`/`Visibility::Inherited` (not `Display::None`) to preserve fixed positions
- `Selection::total_entity_count()` (shared/types.rs:155) for count checks
- `SelectedUnitCapabilities::owned_by_local_player` already computed by `update_command_panel_state`
- HUD systems registered in `DiagCategory::UiHud` (ui/mod.rs:37-52)

## InfoPanel Fog Gating Pattern
- `update_selected_units_grid_system` (hud.rs:223) needs `Res<FogOfWarMap>` + `Res<LocalPlayer>` + `&GridPosition` in queries to gate info fields
- SCP and SDS both spawn with `GridPosition` (resources.rs:66, 596)
- ExtractionPlate gets patch visibility via `sc_patches.get(ep.attached_patch)` — needs `&GridPosition` added to that query
- `FogOfWarMap` imported from `crate::game::world::types`; `VisibilityStateEnum` already in scope via wildcard types import
- Beware `with_children` closure borrow conflicts — compute visibility bools before entering the closure

## QA Test Infrastructure
- `tests/qa/` integration tests have 13 compilation errors (as of 2026-03-28) due to entity definitions refactoring
- Errors span 6 files: agent_tunnel_building, guard_unit, syndicate_agent_unit, gdo_supply_tower_and_chopper, unit_cap_systems, tunnel_expansions_and_starting_condition
- `tests/scenarios/main.rs` has 4 errors: 3 missing spawn functions + 3 type inference issues
- Old API → New API: `guard_type_data()` → `SyndicateGuardDef::base_stats()`, `*State::production_cost()` → `<EntityDef>::cost()`
- `spawn_syndicate_guard()` → `TestHarness::spawn_unit_at_grid(ObjectEnum::SyndicateGuard, ...)`
- Entity def traits need importing: `EntityDefinition` for `::base_stats()`, `::cost()`; `UnitDefinition` for `::attack_data()`
- `SelectedUnitCapabilities` struct changes require updating all initializers in tests/qa/ files (4 sites across 2 files)
- `auto_capabilities.txt` is parsed by `scripts/qa_router.sh` — non-comment non-empty lines become grep patterns for routing qa_items

## Behaviors.rs Split Details
- File is 4,481 lines; test module starts at line 1670 (~2,810 lines of tests)
- Code is NOT contiguous by domain: `building_tunnel_behavior_system` (1111-1289) sits after resource gathering code (757-1110), not after `building_behavior_system` (692-755)
- `tunnel_side_world_position` must be `pub` (not `pub(super)`) because it is re-exported via `units/utils.rs` line 11 to `game/world/faction.rs`
- `find_nearest_own_tunnel` and `drop_off_side_for_carry` can be `pub(super)` -- only used within behaviors submodules
- No constants are shared across submodule boundaries -- each domain's constants are self-contained
- `entering_tunnel_behavior_system` does NOT use `tunnel_side_world_position` -- it uses tunnel center directly (line 551)

## Core.rs Split Details
- File is 6,054 lines; test module starts at line 2916 (~3,138 lines of tests)
- 7 code sections are fully self-contained -- no cross-submodule function calls between them
- `WAYPOINT_ARRIVAL_THRESHOLD` only used within legacy_movement section (lines 1105-2163)
- `CommandContext` only used within command_dispatch section (lines 244-952)
- `emissive_from_color`, `DesiredIndicator`, `CachedIndicatorMaterials` only used within indicators section
- Test `spawn_unit_at` helper (line 2922) used by multiple test sections -- must be duplicated or shared via pub test helper
- No rally.rs tests exist in the test block -- section has zero dedicated tests
- External refs: only `units/mod.rs` references `systems::core::*` paths (17 system registrations); combat/systems/behaviors.rs imports from combat::core (different module)
- Test sections are NOT contiguous by submodule -- command_dispatch tests are split across 4 non-adjacent ranges

## Faction.rs Split Details
- File is 4,387 lines; test module starts at line 2586 (~1,801 lines of tests)
- `get_power_ratio_for_owner` (line 1158, private) is used by 5 systems across 3 target submodules (construction, production, resources) -- must become `pub(super)` in power.rs
- `debug_construction_input` (line 1017) is dead code -- not registered in any plugin, private
- `cults_construction_cancel_system` is registered in `combat/mod.rs` (line 43) via full path, NOT in `world/mod.rs` -- must be re-exported from faction/mod.rs
- `issue_rally_command` (line 425, private) is only called within production systems -- stays private in production.rs
- 4 external import paths reference `crate::game::world::faction::` -- core.rs (rally markers), ui/mod.rs (setup_player_resources x2), combat/mod.rs (cults_construction_cancel)
- Test block is NOT contiguous by submodule -- test functions are interleaved across domains; careful mapping needed
- Shared test helpers (`spawn_test_tile`, `spawn_test_rc`, `setup_production_app`, `setup_construction_app`) are used by multiple test groups -- must be duplicated per submodule or shared
- `super` in `faction/setup.rs` resolves to `faction/mod.rs`, NOT `game::world` -- use `super::super::types` or absolute paths for world-level imports
- Test helper functions (`spawn_test_tile`, `spawn_test_rc`, etc.) are private to the test module -- move to the submodule whose tests use them
- `supply_tower_production_tick_system` is registered in `DiagCategory::SupplyDelivery` set (separate from other production systems in `DiagCategory::Construction`)

## owned_by_local_player Overwrite Bug (state.rs)
- `compute_selected_unit_capabilities()` returns `SelectedUnitCapabilities::default()` which has `owned_by_local_player: false`
- `update_command_panel_state` sets `unit_caps.owned_by_local_player = true` (line 155-157) but then `*unit_caps = new_caps` (line 290-292) overwrites it to false
- This affects ALL player-owned units, not just Peacekeeper — `rebuild_command_panel_ui` returns early at line 43 because `!unit_caps.owned_by_local_player`
- Fix: re-set `owned_by_local_player = owned` AFTER the `*unit_caps = new_caps` assignment

## Info Panel / Selection Panel Icon Cross-Contamination (hud.rs)
- `UnitIcon`, `StructureIcon`, `ResourceIcon` markers are shared between info panel (lines 449-451, 529, 777, 905, 980) and selection panel (lines 1192-1196, 1349)
- Info panel queries are global (no parent scoping), so they see + despawn selection panel icons
- Fix: add `InfoPanelIcon` marker to info panel spawns, filter queries with `With<InfoPanelIcon>`

## execute_command_action Decomposition Details
- Function spans lines 1260-1813 (553 lines) in command_panel.rs
- 5 natural groups: DC (3 arms, ~70 lines), Production (13 arms, ~180 lines), Unit Commands (16 arms, ~110 lines), Tunnel (6 arms, ~18 lines, delegates to tunnel_actions), Placement/Nav (5 arms, ~70 lines)
- ArmoryTrainSoldier/Gunner use `commands.queue(move |world: &mut World| ...)` for CultsPlayerResources access -- deferred world mutation pattern
- SetRallyPoint logically fits with unit commands (AwaitingTarget pattern)
- `active_group_entities()` and `command_target_entities()` from helpers.rs needed by production and unit command groups respectively
- Callers: `handle_command_button_clicks` (line 941) and `command_panel_hotkeys` (line 1160) -- both pass full param set

## Faction Module Split Notes
- faction.rs has 33 pub functions and 9 private functions across 2,585 lines of code + 1,801 lines of tests
- `get_power_ratio_for_owner` is the key cross-submodule dependency: lives in power.rs, needed by construction.rs, production.rs, resources.rs
- `cults_construction_cancel_system` is registered in `combat/mod.rs:43` (NOT world/mod.rs) -- re-export must cover `crate::game::world::faction::cults_construction_cancel_system` path
- `spawn_or_update_rally_marker` and `despawn_rally_marker_for` imported by `units/systems/core.rs:16` -- re-export required
- Test helpers are self-contained within their domain (no cross-domain test helpers)
- `debug_construction_input` is private and unregistered -- stays private in construction.rs
- `issue_rally_command` callers are all in production.rs -- no cross-module concern

## Test Soft Assertion Patterns
- Three patterns exist in tests/qa/: (A) `world.get::<T>()` read-only guards, (B) `world.get_mut::<T>()` setup guards, (C) enum variant matching on local fields
- Pattern A+B: convert `if let Some(x) = world.get/get_mut::<T>(entity)` to `.expect("Entity should have T")`
- Pattern C: `if let Some(EnumVariant { .. }) = field.option` needs `match` or assert + destructure approach
- `world.query::<&T>()` is the Bevy 0.17 replacement for `world.iter_entities().filter_map(|e| world.get::<T>(e.id()))`
- `#[ignore = "reason"]` is the proper Rust syntax for ignore annotations (not `#[ignore] // reason`)

## Directive: Never Run `cargo clean`
- Always use incremental builds. Diagnose build issues directly instead of wiping cache.

## File Reorganization Tasks Must Be Atomic
- Directory split tasks (e.g., converting foo.rs to foo/mod.rs) must: (1) check for pre-existing partial state and clean up, (2) create all new files with correct imports, (3) verify cargo check, (4) only then delete the original .rs file
- If cargo check fails after creating the directory, roll back (delete directory, keep original .rs intact)
- Include these rollback/recovery steps in planned_tasks for any file reorganization work
- Root cause of past conflicts was partial state from interrupted executions, not true concurrency

## Upcoming Architecture: Declarative Entity Definitions
- Forum consensus (2026-03-25): Move toward declarative Rust structs for entity definitions, NOT external YAML/RON files
- Proposed structure: `src/game/entities/{faction}/{units|buildings}/{entity}.rs` — one file per entity
- Each entity file contains: stats (UnitTypeData, AttackAttributesData), costs, command panel layout, capability flags
- What stays OUT: behavior implementations, state machine logic, spawn component assembly, ECS systems
- Registry pattern: `EntityRegistry` resource maps `ObjectEnum` → `&EntityDef`, populated at startup from const definitions
- Explicit registration preferred (mod.rs collects exports) over inventory/linkme crates
- Phase 1: stats + costs; Phase 2: command panel layouts; Phase 3: spawn bundles (evaluate later)
- This will replace scattered match arms in objects.rs, unit_data.rs, grid_layout.rs, structures.rs, utils.rs
- Production building deduplication (Barracks/HQ/Armory/SupplyTower) is a subset of this initiative

## Targeted Attack Channel Fix Pattern
- `attacking_object_behavior_system` (combat/systems/behaviors.rs:22) needs `&mut LocomotionChannel` + `&mut OrientationChannel` added to query
- In-range: write `LocomotionChannel::Stationary` + `OrientationChannel::Turning(target_pos)` for infantry
- Out-of-range: write `LocomotionChannel::Moving(smoothed_path)` + `OrientationChannel::Turning(target_pos)` alongside legacy MoveTarget/Path
- `attack_channel_sync_system` (core.rs:575) needs `&Transform` + `&AttackCapability` added to unit query to compute range
- Aiming+out-of-range: do NOT overwrite LocomotionChannel (let behavior's Moving persist). Aiming+in-range: force Stationary (existing).
- Firing/Cooldown: always force Stationary (existing behavior, no change needed)
- `attack_phase_system` (core.rs:101-102) already resets AttackPhase to None when out of range during Aiming — cooperates with fix

## Known HUD/UI Bugs (2026-03-25 forum)
- **EF Build label**: `grid_button_label()` helpers.rs:327 catch-all `DcBuild(_) => "Build"` doesn't name ExtractionFacility — needs specific arm before catch-all (cost: 200 SC per extraction.rs:235)
- **Icon component cross-contamination**: `update_info_panel_section_system` (hud.rs:449-451) queries global `UnitIcon`/`StructureIcon`/`ResourceIcon` but selection panel (hud.rs:1192-1196) also spawns entities with those markers. Causes incorrect rebuild logic when both panels active.
- **Peacekeeper command panel**: Code path looks correct (Unit component, Default state, 6 registry slots, is_panel_visible=true). Needs runtime investigation — possible ownership or selection detection issue.
