# Task Splitter Insights

## Active: Declarative Entity Definition Refactor (split 2026-03-25)

Architecture_request processed and split into 13 developer_tasks. Key details:
- **Structure**: `src/game/entities/{faction}/{units|buildings}/{entity}.rs` — one file per entity
- **Pattern**: EntityDefinition/UnitDefinition/StructureDefinition traits, explicit registry via `reg.register::<>()`
- **Task breakdown (13 tasks)**:
  1. entity-definition-foundation — traits, types, EntityRegistry, directory structure
  2. entity-def-pilot-peacekeeper — validate pattern end-to-end
  3-9. Per-faction entity migrations (gdo-units, gdo-buildings, syndicate-units, syndicate-buildings, cults-units, cults-buildings, neutral)
  10-12. Consumer wiring (grid-layout, spawn-dispatch, costs) — depend on all migrations
  13. entity-definition-cleanup — remove legacy code, update imports
- **Dependencies**: 1→2→(3-9 parallel)→(10-12 parallel)→13
- **Impact on future splits**: New entity feature_requests become trivially decomposable (one file = one task)
- **21 ObjectEnum variants** across GDO (2 units + 6 buildings), Syndicate (2 units + 2 buildings), Cults (3 units + 3 buildings), Neutral (2 resources)

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
- **action-channels r1 (rework)**: QA found 2 bugs after initial implementation: (1) Targeted attack (Attack+click or right-click enemy) causes unit to stop in place — root cause is `attack_channel_sync_system` forcing LocomotionChannel::Stationary during Aiming/Firing/Cooldown even when unit is out of range; (2) Attack-Move fires once then resumes moving — root cause is leash check in `attack_move_behavior_system` disengaging too aggressively during multi-phase attack cycle. Split into 2 tasks: action-channels-targeted-attack-fix, action-channels-attackmove-target-lock.

## Architecture Refactoring Splitting Heuristics
- File-split refactors (single file -> directory module) should be ONE developer_task, not multiple. The entire split must be atomic since intermediate states won't compile.
- The architecture doc provides exact line ranges -- include these in the task description for the developer.
- Test-only import fixes after refactoring = single task (all test files must compile together, no intermediate valid state).
- For core.rs split: 8 submodules, tests distributed to each submodule. Single task: core_systems_split.

## Codebase Patterns (action channels architecture)
- Behaviors (Phase 2) write to LocomotionChannel/OrientationChannel
- Movement systems (Phase 3) currently use MoveTarget/Path, NOT channels — needs refactoring
- Combat uses AttackState/AttackPhase (combat/types.rs), NOT BaseAttackChannel — needs integration
- Turret systems (combat/turret.rs) don't use TurretAttackChannel/TurretOrientationChannel — needs integration
- PhaseActionConstraints pattern (base_action_constraints) should be replaced by cross-channel constraint enforcement

## Rework: agent_resource_gathering_r1 (2026-03-28)
- Right-click Gather on crystal patches reported as non-functional. Code in command_dispatch.rs (line ~343) looks structurally correct. Likely a hit detection issue (AABB sizing, system ordering, or cursor target not detecting crystal patches as entities). Split as single debugging task: agent_rightclick_gather_fix.
- Note: agent_tunnel_building_r1 is also pending with similar symptoms (Build Tunnel ghost placement doesn't appear). These may share a root cause in entity detection/spawning.

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

## Already-Processed Feature Requests (batch 19 - Design doc clarification)
- **tunnel-transit-light-infantry**: Design doc clarification only — LightInfantry was already correctly included in TransitTier::Tier1 in code (structures.rs line 482) with passing tests. Created a single verification task.

## Already-Processed Feature Requests (batch 20 - Camera)
- **camera-pan-snap**: Design doc formalization of existing behavior. Camera instant-snap centering already implemented in two places: Alt-click portrait (hud.rs selection_portrait_click_system) and double-tap control group recall (resources.rs). Both use same z_offset formula (cam.y * 25.0 / 40.0). Created a single verification task.

## Already-Processed Feature Requests (rework - add_cults_armory_r1)
- **add_cults_armory_r1**: QA blocked — Cults starts with RC at (50,50) but camera doesn't center (setup.rs `center_camera_on_start` has `_ => return` for Cults), enemy test units overlap RC position, no crystal patches nearby. Single task: cults_game_start_fixes (camera centering, enemy relocation, crystal patch placement).

## Codebase Patterns (game start setup)
- `center_camera_on_start` only handles GDO (DeploymentCenter) and Syndicate (Tunnel) — Cults/Colonists fall through to `_ => return`
- Enemy test units hardcoded at (50,50) area — conflicts with Cults RC at same position
- Crystal patches clustered near GDO start (30,30) — no patches near Cults/Syndicate starts
- Each faction's game start function is in setup.rs, registered in mod.rs with `.after(map::spawn_grid)`

## Design Gap: ExtractionFacility Construction (2026-03-21) — RESOLVED
- Designer added EF to DC Constructs list (200 SC, 320 frames) + ExtractionPlate Power -3
- Split into 2 tasks: dc_buildmenu_add_ef (grid slot + construction_cost), extraction_plate_power_cost (PowerValue(-3) on spawn)

## Already-Processed Feature Requests (rework 1 - factions_resources)
- **factions_resources_r1**: QA rework — GDO/Syndicate passed, but 3 items failed: (1) GDO UC cap untestable (blocked on EF buildability), (2) Cults not startable, (3) Colonists not startable. Resource structs and HUD display code already exist for all 4 factions. setup_player_resources in faction.rs is hardcoded for 2-player GDO vs Syndicate. Split into 2 tasks: cults-colonists-game-start (expand AVAILABLE_FACTIONS + setup_player_resources for all 4 factions + stub game start functions), gdo-unit-control-cap-test (integration test for production blocking at cap since in-game QA blocked on EF).

## Codebase Patterns (faction setup)
- setup_player_resources (faction.rs) hardcoded for GDO vs Syndicate binary — needs refactor for 4-faction support
- AVAILABLE_FACTIONS in menu.rs gates which factions are selectable (currently [GDO, Syndicate])
- Each faction needs: InvisibleEntity+FactionEnum+DisplayHud entity, Player+DisplayHudInfo+FactionResources entity, and a setup_X_game_start function registered in world/mod.rs

## Already-Processed Feature Requests (rework 2 - tile terrain)
- **tile_terrain_system_r1**: QA rework — tile presets work but elevation not rendering (all tiles at Y=0). Root cause: spawn_grid hardcodes elevation=0 and Transform Y=0.0. Split into 1 task: tile_elevation_rendering (assign varied elevations per tile type, map elevation to Transform Y).

## Already-Processed Feature Requests (rework 3 - unit bases movement)
- **unit_bases_movement_collision_r1**: QA rework — LightInfantry (Peacekeeper) works but remaining 8 base types untestable. Root causes: (1) spawn_test_units only spawns Peacekeepers + 3 placeholder enemy units with wrong ObjectEnum/stats, (2) NO movement systems for FixedTurnRadius/SpeedTurnRadius/Drag/Glider models (only TurnRate implemented), (3) no crushing mechanic. Split into 4 tasks: test_unit_spawner_all_bases (spawn representative units for all 9 base types), vehicle_turn_movement_systems (FixedTurnRadius + SpeedTurnRadius), drag_glider_movement_systems (Drag + Glider), unit_crushing_mechanic (TrackedVehicle/Mech crush LightInfantry).

## Codebase Patterns (movement systems)
- `unit_movement_system`: generic fallback for non-TurnRate entities (simple waypoint following, no model-specific physics)
- `turn_rate_movement_system`: TurnRate model (Peacekeeper, Agent, Guard, Mech)
- `channel_turnrate_locomotion_system` + `channel_fallback_locomotion_system`: channel-driven variants
- Movement param components: TurnRateMovementParams, FixedTurnRadiusMovementParams, SpeedTurnRadiusMovementParams, DragMovementParams, GliderMovementParams — all defined in movement.rs but only TurnRate has a movement system
- All movement systems registered in UnitsPlugin Phase 3, grid_position_sync_system must run after all of them
- SupplyChopper spawns with DragMovementParams but uses generic unit_movement_system (no drag physics)

## Already-Processed Feature Requests (batch 21 - EP power penalty)
- **extraction_plate_power_penalty**: Single task — extraction_plate_mining_system doesn't use power_ratio. Other production systems (barracks, DC, ST, HQ) all apply get_power_ratio_for_owner() to their progress counters. Mining timer needs same treatment (may require f32 conversion). Split into 1 task: extraction_plate_power_slowdown.

## Already-Processed Feature Requests (batch 22 - ownership guard)
- **command_panel_ownership_guard**: Bug fix — CommandPanel, hotkeys, and right-click resolution all operate on any selected entity regardless of ownership. `is_panel_visible()` checks for empty selection and resource-only but NOT ownership. `right_click_move_command()` checks target ownership (own tunnel, own ST) but not selected unit ownership. Single coherent concern → 1 task: command_panel_ownership_guard (add LocalPlayer+Owner guards to is_panel_visible, update_command_panel_state, command_panel_hotkeys, right_click_move_command, execute_command_action).

## Already-Processed Feature Requests (batch 23 - camera starting position)
- **camera_starting_position**: Camera spawns at (0,40,25) but primary structures are at grid (30,30) GDO / (40,40) Syndicate. Single task: add startup system that queries local player's primary structure (DC or Tunnel) and snaps camera to it. Uses existing snap formula from hud.rs/resources.rs.

## Already-Processed Feature Requests (batch 24 - camera map override)
- **camera_starting_position_map_override**: Extends camera starting position with map-defined per-player-slot positions. No MapStartingPositions resource or player slot concept exists yet. Single task: camera_map_starting_position (add MapStartingPositions resource + startup system with map-override-then-primary-structure fallback). Supersedes the previous camera_starting_position task scope — the new task includes the fallback behavior.

## Already-Processed Feature Requests (batch 25 - Cults structures)
- **cults_recruitment_center_and_storage**: First Cults faction structures. No Cults objects existed in ObjectEnum. Split into 5 tasks:
  1. recruitment_center_structure — ObjectEnum::RecruitmentCenter, type data (4x4, AAAA), spawn function, RecruitmentCenterState component, Cults game start update
  2. storage_structure — ObjectEnum::CultsStorage, type data (3x2, ABAB), spawn function
  3. recruitment_area_tile_claiming — TileClaimMap resource, 10x10 area scanning, first-built priority claiming, effectiveness calculation, reclaim on destruction
  4. recruitment_center_auto_production — Auto-production tick system (effectiveness-scaled rate/capacity), stub CultsRecruit unit, HUD UC aggregation from centers
  5. cults_unit_control_tracking — OriginatingCenters component, lineage persistence through training, death → local_used decrement

## Codebase Patterns (Cults faction)
- CultsPlayerResources already exists with space_crystals, unit_control_used, unit_control_available — HUD display already wired
- setup_cults_game_start exists and spawns a RecruitmentCenter at grid (50,30) with build_order from RecruitmentCenterCounter
- TilePreset.recruitable field exists on all tile types (all except Water are recruitable)
- RecruitmentCenter and CultsStorage ObjectEnum variants NOW EXIST with full type data, spawn functions, and tests
- RecruitmentCenterState component fully defined (rally_point, claimed_tiles, effectiveness, local_capacity, local_used, production_progress, build_order)
- RecruitmentCenterCounter resource for build-order priority tracking
- No StructureMenuState variant for RecruitmentCenter yet (no command panel)
- No CultsRecruit unit exists yet (stub to be created by recruitment_center_auto_production task)
- No TileClaimMap or tile claiming system yet
- No OriginatingCenters / UC tracking yet

## Already-Processed Feature Requests (batch 26 - Cults objects formalized)
- **cults_objects_formalized**: Formal design doc for Cults structures + building mechanics. Previous batch 25 tasks (recruitment_center_structure, storage_structure, recruitment_area_tile_claiming, recruitment_center_auto_production, cults_unit_control_tracking) cover structure data + tile claiming + auto-production + UC tracking. This feature adds: (1) RecruitmentCenter ObjectInterfaceState, (2) Cults building mechanics (Recruit-based construction). Split into 3 tasks:
  1. recruitment_center_interface — StructureMenuState::RecruitmentCenterMenu, command panel (X=Cancel, C=Rally), right-click rally, AwaitingTarget resolution
  2. cults_building_placement — Recruit Build Command flow (Construct submenu, building selection, ghost placement, walk-to-site command, Assist Construction AwaitingTarget)
  3. cults_construction_system — CultsConstructionState component, Recruit enter behavior, construction tick (proportional to Recruit count), completion consumption, cancellation refund

## Already-Processed Feature Requests (batch 27 - Cults Armory)
- **add_cults_armory**: New Cults training building (3x2, ABCB). Recruits enter entrance side (A), stored internally (max 10), trained into Soldiers/Gunners consuming crystals, or ejected from exit side (C). Many values TBD (placeholder constants). No Soldier/Gunner ObjectEnum variants exist yet — stubs needed. Split into 3 tasks:
  1. armory_structure — ObjectEnum::CultsArmory, type data (3x2 ABCB), ArmoryState component, spawn function, placeholder constants
  2. armory_enter_mechanic — Recruit right-click→Enter Armory, walk to A side, enter and hide, StoredRecruits tracking, cap at 10
  3. armory_interface_and_production — StructureMenuState::ArmoryMenu (Q=Soldier, W=Gunner, E=EjectAll, C=Rally), training tick system, eject system, stub CultsSoldier/CultsGunner ObjectEnum variants, info panel, availability gating

## Already-Processed Feature Requests (batch 28 - elevation rendering fix)
- **fix_elevation_rendering_glitches**: Rendering bug — Plane3d tile meshes create visible gaps/seams at different elevation heights. Single task: elevation_cuboid_meshes (replace Plane3d with Cuboid in map.rs spawn_grid, rendering-only fix, no gameplay changes).

## Already-Processed Feature Requests (batch 29 - grid lines elevation)
- **fix_grid_lines_elevation**: Regression fix — draw_grid_lines() hardcodes y=0.005 but elevation system raises tiles above that. Single task: grid_lines_elevation_fix (add ElevationMap param to draw_grid_lines, draw per-cell segments at max neighbor elevation + offset).

## Already-Processed Feature Requests (rework 4 - unit command system)
- **unit_command_system_r1**: QA rework — Move commands work but 3 issues: (1) right-click enemy → unit stops instead of pursuing (AttackTarget issued but approach not working), (2) shift-click queuing → only first command executed (queue not dequeuing or ground handler not using issue_or_queue_command), (3) hotkeys non-functional (likely owned_by_local_player guard or system ordering). Split into 3 tasks: attack_command_rightclick_fix, shift_click_queue_fix, hotkey_command_fix.

## Splitting Lessons (rework tasks)
- QA rework failures that report N distinct bugs should be split into N separate tasks even when they're in the same subsystem — each bug is independently investigable and fixable
- For bug-fix tasks, include specific file locations, line numbers, and investigation hypotheses to help the developer narrow scope quickly

## Already-Processed Feature Requests (batch 30 - extraction plate depletion fix)
- **extraction_plate_persists_after_depletion**: Bug fix — depleted_patch_despawn_system (faction.rs ~line 659) despawns both patch AND plate when remaining_amount==0. Per design, plate must persist and mine at residual rate. Single task: extraction_plate_depletion_lifecycle (fix despawn system to skip plate, handle missing patch entity in mining system, handle missing patch in destruction cleanup).

## Duplicate Feature Requests Resolved
- **designer-fix_extraction_plate_depletion_lifecycle**: Duplicate of designer-extraction_plate_persists_after_depletion. Moved to done with no new tasks. Cleaned up accidentally-created duplicate messages from completion_aggregator/task_planner.

## Duplicate Feature Requests Pending
- (none currently)

## Already-Processed Feature Requests (batch 32 - fog selectability)
- **fog_of_war_selectability**: Two changes: (1) Explored areas should block selection of all objects except SDS for GDO players, (2) SDS InfoPanel rework with visibility-context-aware display (Visible=4 fields, Explored/GDO=3 fields, NextDeliveryTime always shown). Split into 2 tasks: explored_selectability_rules (modify fog filters in both selection_system and box_selection_system in resources.rs), sds_infopanel_visibility_rework (rework SDS InfoPanel display in hud.rs with SelectedFaction gating).

## Already-Processed Feature Requests (batch 31 - InfoPanel fog gating)
- **infopanel_fog_of_war_visibility**: Single task — infopanel_fog_visibility_gate. Add FogOfWarMap + GridPosition + local player lookups to update_selected_units_grid_system in hud.rs. Gate SpaceCrystalPatch RemainingAmount and SupplyDeliveryStation CurrentSupplies behind Visible check. Also covers ExtractionPlate's attached patch display.
- **resource_nodes_r1**: QA rework — duplicate of infopanel_fog_of_war_visibility. Both manifests point to same task (task_splitter-infopanel_fog_visibility_gate.md). When task completes, both features will be marked done by completion_aggregator.

## Splitting Lessons (duplicates)
- ALWAYS check insights for known duplicates BEFORE sending any messages. Reading insights must happen first to avoid creating duplicate tasks that need cleanup.

## Already-Processed Feature Requests (rework 5 - GDO Barracks)
- **gdo_barracks_r1**: QA rework — rally point on enemy doesn't trigger attack approach. Root cause: `issue_rally_command` in faction.rs (line ~470) inserts `UnitCommand::AttackTarget` but NOT `MoveTarget`/`Path` for initial approach. The right-click attack fix in core.rs (lines ~355-375) added this for direct right-click but rally command wasn't updated. Same function is shared by Barracks, HQ, and Armory. Rally target cleanup (destruction → reset to None) already implemented but untestable until attacks work. Single task: rally_attack_approach_path.

## Codebase Patterns (rally command)
- `issue_rally_command` in faction.rs is shared by Barracks (line 404), Headquarters (line 571), and Armory (line 2531)
- `rally_target_cleanup_system` (line 675) handles entity destruction → rally_point=None for Barracks; HQ cleanup likely similar

## Upcoming: Refactoring Architecture Requests (2026-03-23)
- Forum topics opened by operator requesting software_architect to review codebase for refactoring
- Key refactoring targets identified by developer + task_planner: core.rs (right-click monolith), command_panel.rs (16-param limit, monolithic actions), faction.rs (all-factions production mixing), dual movement pipeline (MoveTarget vs LocomotionChannel)
- When architecture_requests arrive for these, expected splitting: trait/dispatch extraction for core.rs, per-state module extraction for command_panel.rs, per-faction file extraction for faction.rs, 2-3 tasks for movement pipeline unification
- Also: automated QA infrastructure architecture_request expected — testing harness already exists (TestApp, 1866+ tests), so new work likely focused on routing/capabilities rather than building from scratch

## Already-Processed Architecture Requests (batch 1)
- **automated-qa-unblock**: Fix test compilation errors + verify qa_router capabilities. auto_capabilities.txt was already updated by software_architect directly. Only developer work: add `owned_by_local_player: true` to 4 SelectedUnitCapabilities initializers in tests/qa/ (2 in agent_object_interface_state.rs lines 653/667, 2 in basic_combat_unit_interface_state.rs lines 439/462), plus verify/fix auto_capabilities.txt content. Split into 1 task: qa_test_compilation_fix.

## Already-Processed Architecture Requests (batch 2)
- **right-click-command-dispatch**: Refactor monolithic `right_click_move_command` (lines 243-1008, ~765 lines) into dispatch pattern. Split into 2 sequential tasks:
  1. command_context_and_issue_helper — Extract CommandContext struct + try_issue_command helper (Phases 2+3 from arch doc). Foundation work that deduplicates 10+ repeated interruptibility/clear/issue patterns and reduces parameter passing.
  2. extract_command_handlers_dispatch — Extract all entity-click handlers (10 handlers: attack, dropoff, enter, pickup, attach, dropoff_supplies, chopper/agent/recruit/guard right-click) and ground-click handlers (6 handlers: move, patrol, attack, attack_move, attack_ground, reverse) into standalone functions. Main function becomes <100-line dispatcher.
- Phase 4 (TargetKind enum) explicitly deferred per arch doc — optional follow-up if Phases 1-3 suffice.

## Splitting Lessons (architecture refactors)
- For pure refactors of a single function, split foundation work (helpers, structs) from extraction work — foundation first reduces complexity of extraction
- Don't over-split refactors that touch the same function — merge conflicts between tasks are a real risk. 2 tasks is better than 4 for a single-function refactor.
- File-to-directory module splits should be a SINGLE task — splitting one file into N submodules is atomic work that cannot be parallelized (all pieces consume the same source file)

## Already-Processed Architecture Requests (batch 3 - file splits)
- **behaviors-split**: Split behaviors.rs (4,481 lines) into behaviors/ directory with 6 submodules (movement, tunnel_interaction, construction, resource_gathering, supply_chopper, mod.rs). Pure file reorganization. Single task because the work is atomic — all submodules consume the same source file.
- **command-panel-split**: Split command_panel.rs (6,357 lines) into command_panel/ directory with 9 files (mod.rs + grid_layout, state, ui_rebuild, hotkeys, actions, tunnel_actions, pointer, helpers). Also decomposes execute_command_action (560 lines) into 5 sub-dispatchers. Split into 2 sequential tasks: command_panel_directory_split (file reorganization + test redistribution), command_panel_action_decomposition (function decomposition into sub-dispatchers). Two tasks justified because function decomposition is a distinct analytical concern from mechanical file moves.
- **core-systems-split**: Processed earlier (see log).
- **faction-split**: Split faction.rs (4,387 lines) into faction/ directory with 9 files (mod.rs + setup, power, production, construction, resources, rally, placement, ejection). Single task: faction_module_split. Key cross-module dependency: spawn_or_update_rally_marker and despawn_rally_marker_for must be re-exported from mod.rs (imported by units/systems/core.rs). All 4 large-file splits now processed.

## Already-Processed Architecture Requests (batch 4 - test suite cleanup)
- **test-suite-cleanup**: 9 failing integration tests across 5 categories, 33 soft assertions, dead code, ignore annotations. Split into 4 tasks:
  1. test_harness_selection_fix (Category D) — fix TestHarness::set_selection() to populate Selection resource groups (DONE at task_planner)
  2. fix_stale_test_values (Categories C+E) — fix ST_BUILD_FRAMES 160→240 + production ejection test expectation (DONE at task_planner)
  3. fix_auto_targeting_and_stop_tests (Categories A+B) — fix VisibilityStateEnum on spawned enemies + stop command test expectations (active at task_planner)
  4. harden_assertions_and_cleanup (Phase 2+3) — 33 soft assertions→expect(), dead code removal, ignore annotations (active at task_planner)
- NOTE: This architecture_request was stuck in active/ because prior session sent all 4 developer_tasks but failed to send feature_request + feature_tasks manifest to completion_aggregator. Fixed by sending both and moving to done.

## Splitting Lessons (stuck messages)
- When resuming a stuck architecture_request/feature_request from active/, always check what was already sent (task_planner done/pending/active) before creating duplicate tasks. Only send what's missing.

## Already-Processed Feature Requests (batch 33 - command panel grid expansion)
- **expand_command_panel_grid**: Pure layout expansion — 3x3 grid to 4x3, adding R/F/V hotkey column. All changes in command_panel/ module (grid_layout.rs, ui_rebuild.rs, hotkeys.rs). New slots unassigned for all current states. Single task because all changes are tightly coupled in one module. NOTE: R and F keys have existing non-grid uses (placement rotation/flip, ready-to-place shortcut) but these are gated by early returns in hotkeys.rs before grid_keys processing.

## Already-Processed Feature Requests (rework 6 - core_systems_split)
- **core_systems_split_r1**: QA rework — 19 integration tests failing with "got Some(Idle)" after issuing commands. Root cause: `behavior_completion_system` (movement.rs:357) resets `UnitCommand::Move` to `Idle` when `LocomotionChannel::Stationary` + zero velocity, but runs in same unordered Phase 2 group as movement behavior systems. Freshly issued commands haven't been processed by `moving_to_location_system` yet, so they have default Stationary+zero state. Fix: add `BaseBehaviorState::None` guard so completion only fires for behaviors that have actually started. Single task: fix_behavior_completion_race.

## Codebase Patterns (behavior completion)
- `behavior_completion_system` runs in Phase 2 (behavior systems), NOT Phase 3 (movement systems)
- It checks `LocomotionChannel::Stationary` + near-zero velocity to determine "complete"
- `BaseBehaviorState::None` is the initial state before any behavior system processes the command
- Movement behavior systems set real states (TurnRate, FixedTurnRadius, etc.) when they first handle a command

## Already-Processed Feature Requests (rework 7 - core_systems_split_r2)
- **core_systems_split_r2**: r1 fixed 13/19 tests, 6 remain in 3 categories:
  1. Interface state reset after selection (2 tests): set_selection() only adds Selected component, doesn't populate Selection.groups. update_command_panel_state reads groups for branching — empty groups causes Default reset. (test_harness_selection_fix task was completed in prior round but NOT actually implemented)
  2. Auto-targeting not engaging (3 tests): spawn_peacekeeper/etc do NOT insert VisibilityStateEnum. base_auto_target_system queries it, is_valid_target requires Visible. (fix_auto_targeting_and_stop_tests task was completed but NOT actually implemented)
  3. ST_BUILD_FRAMES=160 should be 240 (1 test): (fix_stale_test_values task was completed but NOT actually implemented)
  - Split into 2 tasks: fix_test_harness_selection_groups (cat 1), fix_visibility_and_st_constant (cats 2+3)

## Splitting Lessons (failed prior fixes)
- Developer can mark tasks complete without implementing them. When r2/r3 reworks arrive for the same bugs, include very specific code locations and exact changes needed — don't just describe the problem, prescribe the fix with line numbers and concrete code changes.

## Forum: Declarative Rust Entity Definitions (2026-03-25) — CONCLUDED
- Operator pivoted from YAML to declarative Rust structs with per-faction/per-entity file structure
- Proposed: src/game/entities/{faction}/units/{entity}.rs and buildings/{entity}.rs
- Team consensus: Phase 1 = stats/costs, Phase 2 = command panel layouts, defer behaviors
- Software architect's final proposal: EntityDefinition/UnitDefinition/StructureDefinition trait hierarchy, explicit registration (reg.register::<>()), per-entity files contain stats+costs+command_slots+spawn
- Discovery: explicit mod.rs re-exports, no magic macros
- Spawn functions move INTO entity files (half-data half-logic, but colocation preferred)
- Production buildings: generic ProductionBuildingState parameterized by entity def data (eliminates ~300 lines duplication)
- Awaiting architecture_request from software_architect to formalize and split into tasks
- Expected splitting: 1 trait/registry infrastructure task, 1 proof-of-concept migration (Peacekeeper), then N per-entity migration tasks (parallelizable), then grid_layout consolidation, then emptied-code cleanup

## Already-Processed Feature Requests (batch 34 - HUD Bar layout)
- **hud_bar_layout**: HUD Bar restructure from 3 sections (Minimap, UnitsGridSection, CommandPanelSection) to 5 fixed-position sections (Minimap, Portrait, InfoPanel, SelectionPanel, CommandPanel) with visibility rules. Split into 2 tasks:
  1. hud-bar-five-section-scaffold — Restructure setup_hud layout, add PortraitSection/InfoPanelSection/SelectionPanelSection markers, add hud_section_visibility_system (Visibility::Hidden for space reservation)
  2. hud-bar-content-redistribution — Migrate update_selected_units_grid_system into 3 per-section systems (portrait, info, selection), remove old UnitsGridSection

## Codebase Patterns (HUD layout)
- setup_hud in hud.rs: spawns UI camera (Camera2d, order=1), top ResourceBar, bottom HudPanel
- Bottom panel: currently 3 children — Minimap (200px), UnitsGridSection (flex-grow), CommandPanelSection (200px)
- update_selected_units_grid_system is monolithic (~500+ lines) handling 0-selected, single-unit, single-structure, single-resource, and multi-select display
- Existing markers: HudPanel, MinimapSection, UnitsGridSection, CommandPanelSection (in ui/types.rs)
- Use Visibility::Hidden (not Display::None) for fixed-position slot reservation

## Already-Processed Feature Requests (rework 8 - Agent groupable construction)
- **agent_groupable_construction_r1**: QA rework — Groupable=false works but Tunnel build command shows no placement overlay. Root cause: `manage_placement_ghost` in placement.rs (line 36-56) match on panel_state missing `AgentAwaitingPlacement` arm — falls to `_ => None`. Single task: agent_tunnel_placement_ghost_fix (add match arm returning `placement_state.building_type`).

## Codebase Patterns (placement ghost)
- `manage_placement_ghost` (placement.rs:36-56) determines building_type via match on panel_state
- Each AwaitingPlacement variant needs an explicit match arm — new placement modes must be added here
- The rest of the placement flow (validation, confirmation, left-click) already handles AgentAwaitingPlacement correctly
- Related reworks pending: agent_resource_gathering_r1, agent_tunnel_building_r1 — both share same root cause as agent_interface_r1

## Already-Processed Feature Requests (rework 9 - Agent interface)
- **agent_interface_r1**: QA rework — 3 failures: (1) Build Tunnel placement ghost missing, (2) right-click Gather broken, (3) right-click Attack broken. Investigation revealed:
  - Placement ghost: ALREADY FIXED — AgentAwaitingPlacement arm exists in manage_placement_ghost (placement.rs line 55)
  - Right-click Attack: ALREADY FIXED — generic attack handler (command_dispatch.rs lines 96-120) works for all units
  - Right-click Gather (and DropOff, BuildTunnel): ROOT CAUSE FOUND — Agent commands only insert UnitCommand component, NOT the corresponding behavior components. Behavior systems query for behavior components (GatheringResourceBehavior, DroppingOffResourcesBehavior, BuildingTunnelBehavior), so commands are silently ignored.
  - Compare to SupplyChopper pattern which inserts both UnitCommand AND behavior component at right-click time.
  - Split into 1 task: agent_command_behavior_bridge (insert behavior components in command_dispatch.rs and placement.rs)

## Codebase Patterns (command-to-behavior bridge)
- Move: behavior system reads UnitCommand::Move directly (no behavior component needed)
- Enter: has dedicated bridge system (enter_command_validation_system in tunnel_interaction.rs) that reads UnitCommand::Enter and inserts EnteringTunnelBehavior
- SupplyChopper: behavior components inserted alongside commands at right-click time in command_dispatch.rs
- Agent commands (Gather, DropOff, BuildTunnel): NO bridge — commands issued but behavior components never inserted
- Pattern: for immediate commands (!shift_held), insert behavior alongside command. Queued commands (shift-held) need future dequeue bridge.

## Shared Root Cause: agent_resource_gathering_r1, agent_tunnel_building_r1
- Both remaining reworks (pending) share the SAME root cause as agent_interface_r1
- agent_resource_gathering_r1: right-click Gather broken → same missing GatheringResourceBehavior insertion
- agent_tunnel_building_r1: Build Tunnel placement broken → placement ghost is already fixed, actual construction broken by missing BuildingTunnelBehavior insertion
- When agent_command_behavior_bridge task is implemented, all three reworks should pass QA
- **UPDATE 2026-03-28**: agent_command_behavior_bridge AND agent_tunnel_placement_ghost_fix both completed. agent_tunnel_building_r1 rework was filed BEFORE ghost fix landed (336s earlier). Created single verification task.
- **UPDATE 2026-03-28**: agent_resource_gathering_r1 was also already processed with a single debugging task (agent_rightclick_gather_fix)
