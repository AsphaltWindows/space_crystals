# Task Splitter Insights

## Upcoming Features (from forum topics, 2026-03-19)

Operator has posted 6 forum topics requesting designer to produce feature_requests. When these arrive, key splitting notes:

### Dependencies to track
- dc_ef_no_auto_enter_construction_submenu must complete BEFORE dc_default_state_cancel_commands
- QA re-tagging depends on automated_qa_ui_state_queries being completed first

### Large feature areas incoming
1. **Syndicate Agent** (6 subsystems): Agent interface state, spawn/commands, groupability, resource gathering (GatheringResource + DroppingOffResources behaviors), tunnel building (BuildTunnel behavior), worker-built arrival validation. Split along behavior/system boundaries.
2. **Syndicate Tunnels** (5 items): Tunnel interface (4 states: Default/Expand/Eject/AwaitingPlacement), underground occupancy bug fix (blocker!), HQ production interface, Enter command + EnteringTunnel behavior, rally point system.
3. **Unit Control** (3 items): BasicCombatUnitInterfaceState (command panel + right-click), SelectionPanel (multi-select UI), CommonCommand classification fix.
4. **GDO Structures** (3 items): DC cancel commands, Supply Tower interface, Guard unit.
5. **DC/EF Rework** (3 concerns): Stop auto-enter construction submenu, EF flat interface redesign, real-time progress bar fix.
6. **Technical** (2 items): Viewport black line visual glitch (debugging), QA tag re-tagging (markdown-only).

### Splitting heuristics
- Each behavior (GatheringResource, DroppingOffResources, BuildTunnel, EnteringTunnel) = separate task
- Each ObjectInterfaceState = separate task
- Bug fixes that are blockers = single focused task
- New unit implementations (Guard) = self-contained task (ObjectEnum + type data + spawn function)
- Grid layout rearrangements within a single interface state = single task (all slot changes are in one match block in command_panel.rs)
- When a "button removal" is really just rearranging existing resolution logic (e.g., AttackMove via Attack+ground), don't split into separate tasks

## Codebase Patterns (command_panel.rs)
- Unit command grid is in `ObjectInterfaceState::Default` match at ~line 142
- Structure commands use `StructureMenuState` variants
- Agent commands use `AgentMenuState` variants
- `SelectedUnitCapabilities` (has_attack, can_target_ground, can_reverse, agent_carrying) drives conditional slot visibility
- `execute_command_action()` at ~line 1008 handles all button/hotkey actions
- AwaitingTarget cancel (Escape) is at ~line 903

## Codebase Patterns (core.rs - right_click_move_command)
- Entity-click section (~line 256): handles Attack, DropOff target click, chopper targets (right-click+Default), agent targets (right-click+Default)
- Ground-based commands (~line 418+): handles Move, AttackMove, Patrol, Reverse, Enter, Build, Gather/DropOff (all just reset state for non-ground commands)
- AwaitingTarget left-click on entities: NOT handled for Enter/Gather/DropOff modes — they fall through to ground handler which resets state
- Agent right-click resolution is fully implemented (Crystal→Gather, SDS→Gather, Tunnel→DropOff/Enter, Enemy→Attack, Ground→Move)
- BasicCombatUnit right-click own Tunnel → Enter IS handled (line ~444, Guard-specific block)

## Codebase Patterns (Enter command pipeline gap)
- `UnitCommand::Enter(entity)` is inserted by right-click handlers but NO system converts it to `EnteringTunnelBehavior`
- `entering_tunnel_behavior_system` has a bug: calls `despawn()` instead of inserting `InTunnelNetwork` + `Visibility::Hidden`
- Production system (faction.rs ~line 472) correctly uses `InTunnelNetwork` + `Visibility::Hidden` — Enter must match this pattern

## Already-Implemented Features
- **agent-groupable-construction**: Both Groupable=false and single-Agent construction enforcement are fully implemented with tests. Created a single verification task.
- **agent-resource-gathering**: Fully implemented — GatheringResourceBehavior, DroppingOffResourcesBehavior, AgentCarryState, all constants, both behavior systems with side occupancy, right-click integration, resource transfer. Created a single verification task.
- **agent-tunnel-building**: Fully implemented — BuildingTunnelBehavior, BuildTunnelPhase (MovingToSite/Constructing), build_tunnel_behavior_system, ConstructionHP with hp_fraction, spawn_tunnel_under_construction, tunnel_construction_cost scaling, Agent untargetable via Visibility::Hidden, destruction/completion handling, single-agent enforcement, extensive tests. Created a single verification task.
- **tunnel-interface**: Fully implemented — All 4 states (TunnelIdle, TunnelExpandMenu, TunnelEjectMenu, TunnelAwaitingPlacement), upgrade/cancel with cost formula, dynamic expand/eject grids, ghost placement system, ejection_tick_system with 8-frame cooldown, EjectRequest/EjectionQueue components, tier-based transit filtering. Created a single verification task.
- **underground-surface-walkability**: Fully implemented — `rebuild_occupancy_map` in core.rs skips `DomainEnum::Underground` structures when populating `blocked_tiles`/`structure_tiles`. `spawn_headquarters` assigns `DomainEnum::Underground`. Created a single verification task.
- **hq-production-interface**: Fully implemented — HeadquartersState (BuildQueue, CurrentBuild, RallyPoint), HeadquartersMenu grid (Q=Agent, W=Guard, X=Cancel, C=Rally), HqTrain/HqCancel execution with crystal deduction/refund, headquarters_production_tick_system, production_rally_point_system for right-click, set_rally_point_click_system for AwaitingTarget flow, availability checks. Created a single verification task.
- **guard-unit**: Fully implemented — ObjectEnum::SyndicateGuard (36x36, groupable, sight=5), guard_type_data (HeavyInfantry, HP=80, armor 1/1), guard_attack_data (FullyConnected, Ground, damage=6, range=3, aim=2/fire=1/cooldown=1/reload=4), spawn_syndicate_guard(), HQ production (125 SC, 120 frames), HQ menu slot W, GUARD_TUNNEL_SPACE_COST=2, GUARD_CONTROL_COST=1, extensive tests. Created a single verification task.
- **peacekeeper-unit**: Fully implemented — ObjectEnum::Peacekeeper (24x24, groupable, sight=5), peacekeeper_type_data (LightInfantry, GDO, HP=50, armor 1/1), peacekeeper_attack_data (FullyConnected, Ground, damage=10, range=4, aim=2/fire=1/cooldown=2/reload=12), spawn_peacekeeper(), Barracks production (50 SC, 80 frames), PEACEKEEPER_CONTROL_COST=1, PEACEKEEPER_RUGGED_BONUS=0.5, extensive tests. Created a single verification task.

## Already-Processed Feature Requests
- **unit-command-system**: Most command types and issuing paths already implemented (UnitCommand enum, CommandType enum, CommandQueue, BaseCommandState, right-click/left-click/hotkey issuing). Gaps: (1) No system maps UnitCommand → BaseCommandState fields, (2) No shift-click queuing (commands always replace). Split into 2 tasks: command-to-state-mapping, shift-click-command-queuing.
- **control-state-selection**: Mostly implemented — Selection, SelectionGroup, BoxSelection (5-tier), ControlGroups (assign/add/recall/center), Tab forward cycling, selection_validation_system, selection_group_sync_system all exist. Gaps: (1) Ctrl+Shift+Num for Add (currently Shift+Num, and Ctrl+Shift falls into Ctrl=Assign branch), (2) Shift-Tab backward cycling missing, (3) ObjectInterfaceState reset on selection/ActiveGroup change + tick validation not implemented. Split into 2 tasks: control-selection-keybinding-fixes, control-selection-state-validation.
- **command-panel-framework**: Nearly fully implemented — 3x3 grid, hotkeys, Z=Back, X=Cancel, C=Rally, CommonCommands vs GroupCommands (green vs yellow tinting), `is_common_command()`, `command_target_entities()`, Tab group cycling, Escape cancel from all states, placement right-click cancel. Only gap: right-click cancel from non-placement structure sub-menus and AwaitingTarget(SetRallyPoint/ScheduleDeliveries). Split into 1 task: command-panel-rightclick-cancel.
- **scale-camera-system**: Scale system (FRAMES_PER_SECOND=16, GridUnit, SpaceUnit, SPACE_UNITS_PER_GRID_UNIT=64) already fully implemented. Only camera rework needed: remove zoom controls, enforce fixed 28 GridUnit horizontal view. Split into 1 task: camera-fixed-zoom.
- **tile-terrain-system**: Fully implemented — TilePresetEnum (5 presets), TilePreset component (5 properties), TilePlacement (type+location+elevation 0-16), spawn_grid with procedural terrain, distinct colors per tile type. Created a single verification task.
- **factions-resources**: Fully implemented — All 4 faction resource structs (GdoPlayerResources, SyndicatePlayerResources, CultsPlayerResources, ColonistsPlayerResources), HUD display per faction, power_ratio() slowdown, unit control/tunnel space/beacon capacity caps, extensive tests. Created a single verification task.
- **unit-bases-movement-collision**: Fully implemented — All 9 UnitBaseEnum variants with data(), all 5 MovementModel param structs with locomotion/orientation constraints, TurretAttributesData, OccupancyMap ground AABB collision, air_unit_separation_system with SeparationRadius, directional armor. Created a single verification task.
- **resource-nodes**: Fully implemented — SpaceCrystalPatch component (remaining_amount, initial_amount, has_plate), SupplyDeliveryStation component (delivery_size, delivery_interval, current_supplies, time_until_next_delivery), spawn functions for both, sds_delivery_timer system, info panel display in hud.rs, both indestructible/neutral/SightRange=0. Created a single verification task.
- **fog-of-war-elevation**: Fully implemented — FogOfWarMap resource (per-player VisibilityStateEnum per tile), update_fog_of_war system (SightRange+Owner vision, Visible→Explored transitions with LastKnownStructures snapshots), apply_fog_rendering (tile darkening, enemy unit hiding), apply_structure_fog_rendering (Unexplored=hidden, Explored=last-known), ElevationMap resource, elevation_modifier() (air exempt, underground uses surface, binary), elevation integrated into combat systems. Created a single verification task.
- **locomotion-orientation-constraints**: Fully implemented — Locomotion/Orientation enums for constraint lookup, TurnRateConstraint enum (Invalid/Valid/FixedRate/SpeedDependent/Unconstrained), locomotion_orientation_constraint() on all 5 movement param structs, max_turn_rate_at_speed() for speed-dependent models, extensive tests. Created a single verification task.
- **base-behaviors**: Fully implemented — All 9 behaviors: MovingToLocation (units/systems/behaviors.rs), MovingToObject, ReversingToLocation, StoppingBehavior in units module; AttackingObject, AttackingLocation, AttackMovingToLocation (6gu leash), HoldingPosition (facing arc for non-CanTurnInPlace), Patrolling (PatrolEngaged state save/restore) in combat/systems/behaviors.rs. All registered in respective plugins. Created a single verification task.
- **combat-attack-system**: Fully implemented — AttackCapability (all fields), AttackType (4 variants with subtypes), AttackPhase (4 phases + None, interruptibility, base_action_constraints), AttackTarget (UnitTarget/LocationTarget), AttackState, Armor (point/full/directional), Silhouette, DamageEvent (SingleTarget/AoE), attack_command_system, attack_phase_system (full cycle with all 4 attack types), apply_damage_system (SingleTarget point armor + AoE overlap-based + directional armor), directional_armor_multiplier, is_domain_compatible, is_valid_target, projectile_movement/impact systems, turret_autonomous_scanning_system, base_auto_target_system, idle_leash_system, remove_dead_entities_system, visual effects (attack lines, explosions, CombatAssetCache). Created a single verification task.

## Already-Processed Feature Requests (continued)
- **action-channels**: Channel components (LocomotionChannel, OrientationChannel, BaseAttackChannel, TurretOrientationChannel, TurretAttackChannel) fully defined in `game/units/types/state/behavior.rs` and spawned on units. Behaviors write to locomotion/orientation channels. BUT: no consumer systems read channels to drive movement/rotation (current systems use MoveTarget/Path pattern), and combat doesn't write to attack channels (uses AttackState/AttackPhase separately). Split into 2 tasks: action-channel-locomotion-orientation (consumer systems for movement/rotation), action-channel-attack-integration (connect combat to attack channels + turret channels).

## Codebase Patterns (action channels architecture)
- Behaviors (Phase 2) write to LocomotionChannel/OrientationChannel
- Movement systems (Phase 3) currently use MoveTarget/Path, NOT channels — needs refactoring
- Combat uses AttackState/AttackPhase (combat/types.rs), NOT BaseAttackChannel — needs integration
- Turret systems (combat/turret.rs) don't use TurretAttackChannel/TurretOrientationChannel — needs integration
- PhaseActionConstraints pattern (base_action_constraints) should be replaced by cross-channel constraint enforcement

## New Batch of Foundational Features (2026-03-20)
Large batch of ~28 foundational feature requests arrived covering: tile terrain, fog of war, resources, factions, unit bases/movement/collision, combat, locomotion, control/selection, commands, behaviors, turrets, auto-targeting, action channels, command indicators, and faction-specific units/structures. These are core engine systems — process most foundational first.

## Codebase Patterns (main.rs - Camera)
- Camera spawns at (0, 40, 25) looking at origin, uses Camera3d + MainCamera
- `camera_movement`: arrow key panning
- `camera_zoom`: Q/E zoom (needs removal per spec)
- `update_camera_viewport`: adjusts viewport to exclude HUD (top bar + bottom panel)
- GamePlugin registers camera systems in DiagCategory::Camera

## Already-Processed Feature Requests (batch 4 - GDO structures)
- **gdo-power-plant**: Fully implemented — ObjectEnum::PowerPlant, ObjectType (2x2, destructible, sight_range=3, groupable), StructureType (point_armor=1, full_armor=4, AAAA symmetry), spawn_power_plant() with all components (PowerValue(20), BuildRadiusExtension(1)), DC construction cost (150 SC, 160 frames), ConstructionHP rule, compute_power_grid system, has_power_plant flag, no ObjectInterfaceState (info only), extensive tests. Created a single verification task.
- **gdo-deployment-center**: Nearly fully implemented — ObjectType (4x4, destructible, sight_range=6, groupable=false), StructureType (AAAA), DC_MAX_HP=1000, DC_POINT_ARMOR=1, DC_FULL_ARMOR=16, DC_BUILD_RADIUS=12, DC_POWER=20, DeploymentCenterState (current_construction, construction_progress, ready_to_place), construction_cost (PP=150/160, BK=200/160, ST=200/160←WRONG should be 240), cancellation_refund (full/75%), spawn_deployment_center, has_power_plant prerequisite for SupplyTower. ONE FIX: SupplyTower build_frames 160→240. Created a single verification/fix task.
- **gdo-build-area**: Fully implemented — GdoBuildArea resource (HashSet cells), expand_build_area() Chebyshev distance, can_place_building() (build area overlap + visibility + buildable + structure overlap + EP special case), DC seeding (extension=12), placement expansion (PP=1, BK=2, EF=2, ST=1, EP=0), manage_build_area_overlay (green semi-transparent overlay during AwaitingPlacement), build_area_mesh(). Created a single verification task.
- **gdo-barracks**: Fully implemented — BarracksState (rally_point, build_queue max 5, current_build, progress), spawn_barracks (3x2 ABAC, HP=300, PowerValue=-30, BuildRadiusExtension=2), barracks_production_tick_system (power_ratio, B-side spawn, issue_rally_command), BarracksMenu (Q=BkTrain Peacekeeper 50SC/80f, X=BkCancel, C=SetRallyPoint), production_rally_point_system for right-click, DC construction 200SC/160f. Created a single verification task.

## Already-Processed Feature Requests (batch 3)
- **turret-behavior-system**: Components (TurretCommandState, TurretBehaviorState, TurretOrientationChannel, TurretAttackChannel) all exist. turret_autonomous_scanning_system exists but writes to AttackState instead of TurretCommandState. No engagement system reads TurretCommandState to drive channels. No base behavior relay to TurretCommandState. Split into 3 tasks: turret-autonomous-scanning-rework (fix state target), turret-engagement-system (new system: locked_target → channels), turret-base-behavior-target-relay (base behaviors set/clear locked_target).

## Already-Processed Feature Requests (batch 8 - GDO structures cont.)
- **extraction-plate**: Fully implemented — ObjectEnum::ExtractionPlate (1x1, AAAA, HP=85, armor 2/2, sight=0, groupable, BuildRadiusExtension=0), ExtractionPlateState (attached_patch, mining_timer), mining constants (rate=10, residual=1, interval=48), extraction_plate_mining_system, spawn_extraction_plate, destruction cleanup (uncover/despawn patch), depleted_patch_despawn_system, EF construction (75 SC, 96 frames), InfoPanel display, placement validation. Created a single verification task.

## Already-Processed Feature Requests (batch 5 - UI/control fixes)
- **common-vs-group-commands**: Bug fix — `is_common_command()` in command_panel.rs uses hardcoded whitelist instead of checking per-group capabilities. Split into 1 task: common-command-classification-fix (rewrite is_common_command to check all SelectionGroups, update SelectedUnitCapabilities to use active group caps for button visibility).

## Already-Processed Feature Requests (batch 6 - UI)
- **selection-panel**: Fully implemented — `update_selected_units_grid_system` renders portrait grid when 2+ selected (UnitsGridSection, UnitIcon, SelectionPortrait components), active group highlighting, `selection_portrait_click_system` handles all 5 interactions (left-click=select only, shift=remove, ctrl=select type, ctrl-shift=remove type, alt=center camera), edge cases (reduce to 1/0). Created a single verification task.
- **command-indicators**: Fully implemented — CommandIndicatorType (Location/Object), CommandIndicator component, command_indicator_color() (Green/Red/Orange), command_has_indicator(), command_indicator_sync_system (diff-based spawn/despawn, cached meshes/materials, Patrol dual indicators), extensive tests. Created a single verification task.

## Already-Processed Feature Requests (batch 7 - auto-targeting)
- **base-auto-targeting**: Substantially implemented — base_auto_target_system (idle scanning + IdleOrigin), idle_leash_system (4gu leash), hold_position_behavior_system (stationary + facing arc). Gaps: (1) target priority is distance-only (needs threatening→rotation→distance), (2) idle scanning uses attack_cap.range not SightRange, (3) AttackMove erroneously included in base_auto_target match, (4) no ValidTarget/domain compatibility filtering. Split into 1 task: base-auto-target-refinements.

## Already-Processed Feature Requests (batch 9 - Syndicate rally)
- **syndicate-rally-point**: Nearly fully implemented — HeadquartersState.rally_point, production_rally_point_system (right-click), set_rally_point_click_system (C→AwaitingTarget→left-click), issue_rally_command (Location→Move, enemy Object→AttackTarget, friendly Object→Move), parent tunnel detection (clears rally). ONE BUG: eject decision in headquarters_production_tick_system uses `_ => true` which catches `None` (no rally) — should be `None => false` (stay in tunnel). Split into 1 task: syndicate-rally-point-eject-fix.

## Already-Processed Feature Requests (batch 11 - Syndicate tunnels)
- **tunnel-network-mechanics**: Fully implemented — TunnelTier (Tier1/2/3) with HP (600/800/1000), armor (1/16), space (20/30/40), area radius (3/4/5). TransitTier filtering (Infantry/+Vehicles/+Air). TunnelArea with overlaps() and validate_tunnel_upgrade(). Cost scaling: tunnel_construction_cost (0,1,2...), tunnel_t2_upgrade_cost (2,4,6...), tunnel_t3_upgrade_cost (3,6,9...). Side B/C drop-off with occupancy (drop_off_side_for_carry). TunnelState/TunnelOperation (one op at a time). spawn_tunnel with ABCD/SightRange=5/groupable=false. InTunnelNetwork marker. Created a single verification task.

## Already-Processed Feature Requests (batch 10 - unit interface)
- **basic-combat-unit-interface**: Fully implemented — Grid layout (Q=Move, W=Reverse, E=HoldPosition, A=Attack, S=Patrol, D=AttackGround, X=Stop), right-click resolution (Enemy→Attack, Ground→Move, own Tunnel→Enter for Syndicate, other→Move), all AwaitingTarget resolutions (Attack entity→Attack, Attack ground→AttackMove, Move→Move, Patrol→Patrol, AttackGround→AttackLocation, Reverse→Reverse), cancel via Z=Back/Escape. Created a single verification task.

## Splitting Lessons
- Design doc labels (A, B, C, X) map to grid slot hotkeys: A→(0,0)=Q, B→(0,1)=W, C→(0,2)=E, X→(2,1)=X
- When a feature request says "two panel commands", check if current implementation over-implements (had 7 buttons when spec says 2)
- AwaitingTarget modes that need entity clicks (not ground) require explicit handling in the entity-click section of right_click_move_command

## Already-Processed Feature Requests (batch 12 - DC/EF construction rework)
- **dc-ef-construction-rework**: DC BuildMenu incorrectly includes ExtractionFacility (EF is a separate structure per design). EF uses submenu pattern (EfIdle/EfConstructing/EfReadyToPlace) but design requires flat DefaultState with dynamic buttons. Split into 2 tasks: dc-buildmenu-remove-ef (remove EF from DC build menu), ef-flat-interface-rework (collapse EF states into EfIdle with dynamic Q button).

## Already-Processed Feature Requests (batch 13 - GDO EF structure)
- **gdo-extraction-facility**: Fully implemented — ObjectEnum::ExtractionFacility (3x3, AAAA, destructible, sight=3, groupable=false), EF_MAX_HP=500, EF_POINT_ARMOR=1, EF_FULL_ARMOR=9, EF_BUILD_RADIUS=2, EF_POWER=-15, spawn_extraction_facility(), ExtractionFacilityState (current_construction/construction_progress/ready_to_place), ef_construction_tick_system, EP construction 75SC/96f, cancellation (full/75%). Created a single verification task.

## Codebase Patterns (structure armor)
- Structure armor constants (e.g., EF_POINT_ARMOR, DC_POINT_ARMOR) are defined but NOT attached as Armor components in spawn functions. Only units get Armor components. This is consistent across all structures — not a bug, just not yet integrated into the damage system for structures.

## Already-Processed Feature Requests (batch 14 - Supply Tower interface)
- **supply-tower-interface**: Nearly fully implemented — SupplyTowerState (attached_chopper, landed_chopper, scheduled_sds, build_queue, current_build, rally_point), SupplyTowerMenu grid (Q=StTrain SupplyChopper 100SC/160f, S=StScheduleDeliveries, X=StCancel, C=SetRallyPoint), StTrain/StCancel actions with crystal deduct/refund, schedule_deliveries_click_system (validates SDS target), set_rally_point_click_system (ST branch), production_rally_point_system (right-click rally), supply_tower_production_tick_system (power_ratio, spawn + rally command), button availability (S checks attached_chopper.is_some()), info panel (progress/queue/delivering). ONE BUG: free chopper spawned on placement (faction.rs ~line 1410) not linked — tower.attached_chopper and chopper.attached_tower both None. Created 1 fix task: supply-tower-placement-attach-chopper.

## Already-Processed Feature Requests (batch 15 - Supply Chopper)
- **supply-chopper**: Substantially implemented — ObjectEnum::SupplyChopper (60x60, HP=150, armor 1/1, sight=5, groupable, unarmed), SupplyChopperState (carried_supplies, attached_tower), spawn_supply_chopper() with DragMovementParams (HoverCraft), UnitCommand::PickUpSupplies/AttachToTower, right-click resolution (SDS→PickUpSupplies, own ST→AttachToTower), ST production (100SC/160f), free chopper spawn on tower placement. Gaps: (1) No command panel grid for SupplyChopper selection (no CommandType::PickUpSupplies/AttachToTower, no AwaitingTarget resolution, no grid buttons), (2) No behavior systems to execute PickUpSupplies/AttachToTower commands (fly to target, pickup/dropoff, attach/detach, repair). Split into 2 tasks: supply-chopper-interface, supply-chopper-behaviors.

## Already-Processed Feature Requests (batch 17 - Supply Chopper Commands)
- **supply-chopper-commands**: Adds DropOffSupplies command variant, state-dependent right-click resolution (carrying supplies → DropOffSupplies, not carrying → AttachToTower), carrying-units gating on PickUpSupplies/AttachToTower, command panel grid with 4 AwaitingTarget modes (PickUpSupplies/AttachToTower/DropOffSupplies/Move), attachment-breaking on player commands. Split into 2 tasks: supply-chopper-dropoff-command (UnitCommand variant + right-click updates + attachment breaking), supply-chopper-command-panel (CommandButtonAction variants + grid + AwaitingTarget resolutions + availability gating).

## Already-Processed Feature Requests (batch 16 - Syndicate HQ structure)
- **syndicate-headquarters-structure**: Fully implemented — ObjectEnum::Headquarters (2x2, destructible, Underground), HQ_MAX_HP=400, HQ_POINT_ARMOR=1, HQ_FULL_ARMOR=4, HQ_SC_COST=200, HQ_BUILD_FRAMES=400, spawn_headquarters() with DomainEnum::Underground + TunnelExpansionMarker, HeadquartersState (rally/queue max 5/build/progress), production catalog (Agent 100SC/160f, Guard 125SC/120f), pre-built in starting tunnel, tunnel expand menu integration. Created a single verification task.

## Already-Processed Feature Requests (batch 18 - Pointer Display)
- **pointer-display-types**: No existing implementation — PointerDisplayType is entirely new. Split into 2 tasks: pointer_display_type_resolution (enum + resource + resolution system with full rule matrix for DefaultState/AwaitingTarget/Placement), pointer_display_rendering (visual cursor update system). Resolution mirrors right-click logic in core.rs but produces display state only.
