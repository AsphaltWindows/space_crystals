use bevy::prelude::*;
use crate::types::*;
use crate::game::combat::types::AttackState;
use crate::game::world::types::{Tile, TilePreset, GridMap};
use crate::game::units::types::*;
use crate::game::units::utils::{world_to_grid, smooth_path};
use crate::ui::types::ObjectInterfaceState;
use crate::game::units::types::commands::{CommandType, BaseCommandState, CommandQueue};
use crate::game::units::utils::issue_or_queue_command;

/// System to handle command input (hotkeys)
/// These legacy hotkeys (M/A/G/P/H/S) only fire when the command panel is hidden.
/// When the panel shows UnitCommands, the grid hotkeys (Q/W/E/A/S/D) handle commands instead.
pub fn command_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interface_state: ResMut<ObjectInterfaceState>,
    selection: Res<Selection>,
) {
    // When panel is showing content, grid hotkeys handle commands
    let panel_visible = match &*interface_state {
        ObjectInterfaceState::Default => !selection.groups.is_empty(),
        _ => true,
    };

    if panel_visible {
        // Only allow Escape to cancel awaiting target regardless of panel state
        if keyboard.just_pressed(KeyCode::Escape) {
            if interface_state.is_awaiting_target() {
                *interface_state = ObjectInterfaceState::Default;
                info!("Command mode: Default");
            }
        }
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyM) {
        *interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::Move);
        info!("Command mode: Move");
    } else if keyboard.just_pressed(KeyCode::KeyA) {
        *interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::Attack);
        info!("Command mode: Attack");
    } else if keyboard.just_pressed(KeyCode::KeyG) {
        *interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::AttackGround);
        info!("Command mode: Attack Ground");
    } else if keyboard.just_pressed(KeyCode::KeyT) {
        *interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::AttackMove);
        info!("Command mode: Attack Move");
    } else if keyboard.just_pressed(KeyCode::KeyP) {
        *interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::Patrol);
        info!("Command mode: Patrol");
    } else if keyboard.just_pressed(KeyCode::KeyH) {
        // Handled in hold_position_system
    } else if keyboard.just_pressed(KeyCode::KeyS) {
        // Handled in stop_command_system
    } else if keyboard.just_pressed(KeyCode::Escape) {
        if interface_state.is_awaiting_target() {
            *interface_state = ObjectInterfaceState::Default;
            info!("Command mode: Default");
        }
    }
}

/// System to handle Hold Position command (H key)
/// Only fires when command panel is hidden; otherwise grid hotkeys handle it.
/// Respects attack phase interruptibility — non-interruptible phases block the command.
pub fn hold_position_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected_units: Query<(Entity, Option<&AttackState>, &mut CommandQueue), (With<Unit>, With<Selected>)>,
    interface_state: Res<ObjectInterfaceState>,
    selection: Res<Selection>,
) {
    // Panel hidden = Default state with empty selection
    let panel_hidden = matches!(*interface_state, ObjectInterfaceState::Default) && selection.groups.is_empty();
    if !panel_hidden { return; }
    if keyboard.just_pressed(KeyCode::KeyH) {
        let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
        let count = selected_units.iter().count();
        if count > 0 {
            for (entity, attack_state_opt, mut command_queue) in &mut selected_units {
                // Skip units in non-interruptible attack phases
                if let Some(attack_state) = attack_state_opt {
                    if !attack_state.phase.is_interruptible() {
                        continue;
                    }
                }
                let mut entity_cmds = commands.entity(entity);
                if !shift_held {
                    entity_cmds
                        .remove::<MoveTarget>()
                        .remove::<Path>()
                        .insert(HoldingPosition);
                }
                issue_or_queue_command(&mut entity_cmds, &mut command_queue, UnitCommand::HoldPosition, shift_held);
            }
            info!("Hold Position: {} unit(s)", count);
        }
    }
}

/// System to handle Stop command (S key)
/// Only fires when command panel is hidden; otherwise grid hotkeys handle it.
/// Respects attack phase interruptibility — non-interruptible phases block the command.
pub fn stop_command_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected_units: Query<
        (Entity, &mut Velocity, Option<&AttackState>, &mut CommandQueue),
        (With<Unit>, With<Selected>)
    >,
    interface_state: Res<ObjectInterfaceState>,
    selection: Res<Selection>,
) {
    let panel_hidden = matches!(*interface_state, ObjectInterfaceState::Default) && selection.groups.is_empty();
    if !panel_hidden { return; }
    if keyboard.just_pressed(KeyCode::KeyS) {
        let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
        let count = selected_units.iter().count();
        if count > 0 {
            for (entity, mut velocity, attack_state_opt, mut command_queue) in &mut selected_units {
                // Skip units in non-interruptible attack phases
                if let Some(attack_state) = attack_state_opt {
                    if !attack_state.phase.is_interruptible() {
                        continue;
                    }
                }
                let mut entity_cmds = commands.entity(entity);
                if !shift_held {
                    velocity.0 = Vec3::ZERO;
                    entity_cmds
                        .remove::<MoveTarget>()
                        .remove::<Path>()
                        .remove::<HoldingPosition>();
                }
                issue_or_queue_command(&mut entity_cmds, &mut command_queue, UnitCommand::Stop, shift_held);
            }
            info!("Stop: {} unit(s)", count);
        }
    }
}

/// System to handle patrol command execution
pub fn patrol_command_system(
    _time: Res<Time>,
    mut commands: Commands,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    grid: Res<GridMap>,
    occupancy: Res<crate::game::units::types::OccupancyMap>,
    mut units: Query<
        (Entity, &Transform, &mut UnitCommand, &UnitBaseEnum),
        With<Unit>
    >,
) {
    for (entity, transform, mut command, unit_base) in &mut units {
        if let UnitCommand::Patrol { start, end, going_to_end } = *command {
            let current_pos = transform.translation;
            let target = if going_to_end { end } else { start };

            let distance = Vec3::new(
                target.x - current_pos.x,
                0.0,
                target.z - current_pos.z,
            ).length();

            if distance < 0.5 {
                let new_going_to_end = !going_to_end;
                let new_target = if new_going_to_end { end } else { start };

                let start_grid = world_to_grid(current_pos);
                let target_grid = world_to_grid(new_target);

                if let Some(path) = crate::game::units::pathfinding::find_path_for_domain(start_grid, target_grid, &tiles, unit_base, grid.width as i32, grid.height as i32, &occupancy, (start_grid.x, start_grid.z)) {
                    let smoothed_waypoints = smooth_path(path);

                    commands.entity(entity).insert((
                        MoveTarget(new_target),
                        Path {
                            waypoints: smoothed_waypoints,
                            current_waypoint: 0,
                        },
                    ));
                }

                *command = UnitCommand::Patrol {
                    start,
                    end,
                    going_to_end: new_going_to_end,
                };
            }
        }
    }
}

/// System that syncs BaseCommandState from the current UnitCommand each tick.
/// Maps each UnitCommand variant to the appropriate CommandType + target fields.
pub fn command_state_sync_system(
    mut units: Query<(&UnitCommand, &mut BaseCommandState), With<Unit>>,
) {
    for (command, mut state) in &mut units {
        let (cmd_type, target_loc, target_ent) = match command {
            UnitCommand::Idle => (CommandType::Default, None, None),
            UnitCommand::Move(pos) => (CommandType::Move, Some(*pos), None),
            UnitCommand::AttackTarget(entity) => (CommandType::Attack, None, Some(*entity)),
            UnitCommand::AttackLocation(pos) => (CommandType::AttackGround, Some(*pos), None),
            UnitCommand::AttackMove(pos) => (CommandType::AttackMove, Some(*pos), None),
            UnitCommand::Patrol { end, .. } => (CommandType::Patrol, Some(*end), None),
            UnitCommand::HoldPosition => (CommandType::HoldPosition, None, None),
            UnitCommand::Stop => (CommandType::Stop, None, None),
            UnitCommand::Reverse(pos) => (CommandType::Reverse, Some(*pos), None),
            UnitCommand::Enter(entity) => (CommandType::Enter, None, Some(*entity)),
            UnitCommand::Build { target, .. } => (CommandType::Build, Some(*target), None),
            UnitCommand::Gather(entity) => (CommandType::Gather, None, Some(*entity)),
            UnitCommand::DropOffResources(entity) => (CommandType::DropOff, None, Some(*entity)),
            UnitCommand::BuildTunnel(pos) => (CommandType::BuildTunnel, Some(*pos), None),
            UnitCommand::PickUpSupplies(entity) => (CommandType::PickUpSupplies, None, Some(*entity)),
            UnitCommand::AttachToTower(entity) => (CommandType::AttachToTower, None, Some(*entity)),
            UnitCommand::DropOffSupplies(entity) => (CommandType::DropOffSupplies, None, Some(*entity)),
        };
        state.command_type = cmd_type;
        state.target_location = target_loc;
        state.target_entity = target_ent;
    }
}

/// System that dequeues the next command from CommandQueue when the current command is Idle.
/// Runs before command_state_sync_system so dequeued commands get their state mapped same tick.
pub fn command_dequeue_system(
    mut units: Query<(&mut UnitCommand, &mut CommandQueue), With<Unit>>,
) {
    for (mut command, mut queue) in &mut units {
        if matches!(*command, UnitCommand::Idle) && !queue.is_empty() {
            if let Some(next_command) = queue.pop_front() {
                *command = next_command;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    use bevy::MinimalPlugins;

    /// Helper: spawn a unit entity with Unit marker, UnitCommand, BaseCommandState, and CommandQueue
    fn spawn_test_unit(app: &mut App, command: UnitCommand) -> Entity {
        app.world_mut().spawn((
            Unit,
            command,
            BaseCommandState::default(),
            CommandQueue::new(),
        )).id()
    }

    fn run_sync_system(app: &mut App) {
        app.add_systems(Update, command_state_sync_system);
        app.update();
    }

    fn run_dequeue_system(app: &mut App) {
        app.add_systems(Update, command_dequeue_system);
        app.update();
    }

    fn run_both_systems(app: &mut App) {
        app.add_systems(Update, (
            command_dequeue_system,
            command_state_sync_system.after(command_dequeue_system),
        ));
        app.update();
    }

    // === command_state_sync_system tests ===

    #[test]
    fn sync_idle_maps_to_default() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = spawn_test_unit(&mut app, UnitCommand::Idle);
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::Default);
        assert!(state.target_location.is_none());
        assert!(state.target_entity.is_none());
    }

    #[test]
    fn sync_move_maps_correctly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let pos = Vec3::new(10.0, 0.0, 5.0);
        let entity = spawn_test_unit(&mut app, UnitCommand::Move(pos));
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::Move);
        assert_eq!(state.target_location, Some(pos));
        assert!(state.target_entity.is_none());
    }

    #[test]
    fn sync_attack_target_maps_correctly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let target = app.world_mut().spawn_empty().id();
        let entity = spawn_test_unit(&mut app, UnitCommand::AttackTarget(target));
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::Attack);
        assert!(state.target_location.is_none());
        assert_eq!(state.target_entity, Some(target));
    }

    #[test]
    fn sync_attack_location_maps_correctly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let pos = Vec3::new(3.0, 0.0, 7.0);
        let entity = spawn_test_unit(&mut app, UnitCommand::AttackLocation(pos));
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::AttackGround);
        assert_eq!(state.target_location, Some(pos));
        assert!(state.target_entity.is_none());
    }

    #[test]
    fn sync_attack_move_maps_correctly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let pos = Vec3::new(5.0, 0.0, 5.0);
        let entity = spawn_test_unit(&mut app, UnitCommand::AttackMove(pos));
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::AttackMove);
        assert_eq!(state.target_location, Some(pos));
        assert!(state.target_entity.is_none());
    }

    #[test]
    fn sync_patrol_maps_correctly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let end = Vec3::new(20.0, 0.0, 20.0);
        let entity = spawn_test_unit(&mut app, UnitCommand::Patrol {
            start: Vec3::ZERO,
            end,
            going_to_end: true,
        });
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::Patrol);
        assert_eq!(state.target_location, Some(end));
        assert!(state.target_entity.is_none());
    }

    #[test]
    fn sync_hold_position_maps_correctly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = spawn_test_unit(&mut app, UnitCommand::HoldPosition);
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::HoldPosition);
        assert!(state.target_location.is_none());
        assert!(state.target_entity.is_none());
    }

    #[test]
    fn sync_stop_maps_correctly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = spawn_test_unit(&mut app, UnitCommand::Stop);
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::Stop);
        assert!(state.target_location.is_none());
        assert!(state.target_entity.is_none());
    }

    #[test]
    fn sync_reverse_maps_correctly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let pos = Vec3::new(1.0, 0.0, 2.0);
        let entity = spawn_test_unit(&mut app, UnitCommand::Reverse(pos));
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::Reverse);
        assert_eq!(state.target_location, Some(pos));
        assert!(state.target_entity.is_none());
    }

    #[test]
    fn sync_enter_maps_correctly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let tunnel = app.world_mut().spawn_empty().id();
        let entity = spawn_test_unit(&mut app, UnitCommand::Enter(tunnel));
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::Enter);
        assert!(state.target_location.is_none());
        assert_eq!(state.target_entity, Some(tunnel));
    }

    #[test]
    fn sync_build_maps_correctly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let pos = Vec3::new(8.0, 0.0, 8.0);
        let entity = spawn_test_unit(&mut app, UnitCommand::Build {
            target: pos,
            object: crate::types::ObjectEnum::Tunnel,
        });
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::Build);
        assert_eq!(state.target_location, Some(pos));
        assert!(state.target_entity.is_none());
    }

    #[test]
    fn sync_gather_maps_correctly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let crystal = app.world_mut().spawn_empty().id();
        let entity = spawn_test_unit(&mut app, UnitCommand::Gather(crystal));
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::Gather);
        assert!(state.target_location.is_none());
        assert_eq!(state.target_entity, Some(crystal));
    }

    #[test]
    fn sync_drop_off_maps_correctly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let tunnel = app.world_mut().spawn_empty().id();
        let entity = spawn_test_unit(&mut app, UnitCommand::DropOffResources(tunnel));
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::DropOff);
        assert!(state.target_location.is_none());
        assert_eq!(state.target_entity, Some(tunnel));
    }

    #[test]
    fn sync_build_tunnel_maps_correctly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let pos = Vec3::new(15.0, 0.0, 15.0);
        let entity = spawn_test_unit(&mut app, UnitCommand::BuildTunnel(pos));
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::BuildTunnel);
        assert_eq!(state.target_location, Some(pos));
        assert!(state.target_entity.is_none());
    }

    // === command_dequeue_system tests ===

    #[test]
    fn dequeue_pops_when_idle_with_queue() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = spawn_test_unit(&mut app, UnitCommand::Idle);
        {
            let mut queue = app.world_mut().get_mut::<CommandQueue>(entity).unwrap();
            queue.push(UnitCommand::Move(Vec3::new(1.0, 0.0, 0.0)));
            queue.push(UnitCommand::HoldPosition);
        }
        run_dequeue_system(&mut app);

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Move(_)));
        let queue = app.world().get::<CommandQueue>(entity).unwrap();
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn dequeue_does_nothing_when_not_idle() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = spawn_test_unit(&mut app, UnitCommand::Move(Vec3::ZERO));
        {
            let mut queue = app.world_mut().get_mut::<CommandQueue>(entity).unwrap();
            queue.push(UnitCommand::HoldPosition);
        }
        run_dequeue_system(&mut app);

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Move(_)));
        let queue = app.world().get::<CommandQueue>(entity).unwrap();
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn dequeue_does_nothing_when_queue_empty() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = spawn_test_unit(&mut app, UnitCommand::Idle);
        run_dequeue_system(&mut app);

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Idle));
    }

    #[test]
    fn dequeue_fifo_order() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = spawn_test_unit(&mut app, UnitCommand::Idle);
        {
            let mut queue = app.world_mut().get_mut::<CommandQueue>(entity).unwrap();
            queue.push(UnitCommand::Move(Vec3::new(1.0, 0.0, 0.0)));
            queue.push(UnitCommand::AttackMove(Vec3::new(2.0, 0.0, 0.0)));
            queue.push(UnitCommand::Stop);
        }

        // First dequeue
        run_dequeue_system(&mut app);
        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Move(_)));

        // Simulate command completion by setting Idle
        *app.world_mut().get_mut::<UnitCommand>(entity).unwrap() = UnitCommand::Idle;
        app.update(); // run dequeue again
        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::AttackMove(_)));

        // Next
        *app.world_mut().get_mut::<UnitCommand>(entity).unwrap() = UnitCommand::Idle;
        app.update();
        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Stop));

        // Queue empty, stays idle
        *app.world_mut().get_mut::<UnitCommand>(entity).unwrap() = UnitCommand::Idle;
        app.update();
        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Idle));
    }

    // === Integration: both systems together ===

    #[test]
    fn dequeue_and_sync_in_same_tick() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = spawn_test_unit(&mut app, UnitCommand::Idle);
        {
            let mut queue = app.world_mut().get_mut::<CommandQueue>(entity).unwrap();
            queue.push(UnitCommand::AttackLocation(Vec3::new(5.0, 0.0, 5.0)));
        }
        run_both_systems(&mut app);

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::AttackLocation(_)));
        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::AttackGround);
        assert_eq!(state.target_location, Some(Vec3::new(5.0, 0.0, 5.0)));
    }

    // === issue_or_queue_command tests ===

    #[test]
    fn issue_or_queue_non_shift_replaces_command_and_clears_queue() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = spawn_test_unit(&mut app, UnitCommand::Move(Vec3::ZERO));
        // Pre-fill the queue
        {
            let mut queue = app.world_mut().get_mut::<CommandQueue>(entity).unwrap();
            queue.push(UnitCommand::HoldPosition);
            queue.push(UnitCommand::Stop);
        }

        // Simulate non-shift command issue via deferred commands
        {
            let world = app.world_mut();
            let mut command_queue_comp = world.get_mut::<CommandQueue>(entity).unwrap();
            command_queue_comp.clear();
            // Insert new command directly
            let mut entity_ref = world.entity_mut(entity);
            entity_ref.insert(UnitCommand::AttackMove(Vec3::new(5.0, 0.0, 5.0)));
        }

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::AttackMove(_)));
        let queue = app.world().get::<CommandQueue>(entity).unwrap();
        assert!(queue.is_empty(), "Queue should be cleared on non-shift command");
    }

    #[test]
    fn issue_or_queue_shift_appends_without_changing_current() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = spawn_test_unit(&mut app, UnitCommand::Move(Vec3::ZERO));

        // Simulate shift-queue: push to queue, don't change current command
        {
            let mut queue = app.world_mut().get_mut::<CommandQueue>(entity).unwrap();
            queue.push(UnitCommand::HoldPosition);
        }

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Move(_)), "Current command should remain Move");
        let queue = app.world().get::<CommandQueue>(entity).unwrap();
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn multiple_shift_clicks_accumulate_in_order() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = spawn_test_unit(&mut app, UnitCommand::Idle);

        // Simulate 3 shift-clicks
        {
            let mut queue = app.world_mut().get_mut::<CommandQueue>(entity).unwrap();
            queue.push(UnitCommand::Move(Vec3::new(1.0, 0.0, 0.0)));
            queue.push(UnitCommand::AttackMove(Vec3::new(2.0, 0.0, 0.0)));
            queue.push(UnitCommand::HoldPosition);
        }

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Idle), "Current command should remain Idle");
        let queue = app.world().get::<CommandQueue>(entity).unwrap();
        assert_eq!(queue.len(), 3, "Queue should have 3 entries");

        // Verify FIFO order by dequeuing
        run_dequeue_system(&mut app);
        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Move(_)), "First dequeued should be Move");
        let queue = app.world().get::<CommandQueue>(entity).unwrap();
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn non_shift_after_shift_queue_clears_queue() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = spawn_test_unit(&mut app, UnitCommand::Idle);

        // Shift-queue some commands
        {
            let mut queue = app.world_mut().get_mut::<CommandQueue>(entity).unwrap();
            queue.push(UnitCommand::Move(Vec3::new(1.0, 0.0, 0.0)));
            queue.push(UnitCommand::HoldPosition);
        }

        // Non-shift command: should clear queue and replace current
        {
            let world = app.world_mut();
            let mut queue = world.get_mut::<CommandQueue>(entity).unwrap();
            queue.clear();
            let mut entity_ref = world.entity_mut(entity);
            entity_ref.insert(UnitCommand::Stop);
        }

        let cmd = app.world().get::<UnitCommand>(entity).unwrap();
        assert!(matches!(cmd, UnitCommand::Stop));
        let queue = app.world().get::<CommandQueue>(entity).unwrap();
        assert!(queue.is_empty(), "Queue should be empty after non-shift command");
    }

    // === SupplyChopper command mapping tests ===

    #[test]
    fn sync_pick_up_supplies_maps_to_pick_up_supplies() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let target = app.world_mut().spawn_empty().id();
        let entity = spawn_test_unit(&mut app, UnitCommand::PickUpSupplies(target));
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::PickUpSupplies);
        assert!(state.target_location.is_none());
        assert_eq!(state.target_entity, Some(target));
    }

    #[test]
    fn sync_attach_to_tower_maps_to_attach_to_tower() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let target = app.world_mut().spawn_empty().id();
        let entity = spawn_test_unit(&mut app, UnitCommand::AttachToTower(target));
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::AttachToTower);
        assert!(state.target_location.is_none());
        assert_eq!(state.target_entity, Some(target));
    }

    #[test]
    fn sync_drop_off_supplies_maps_to_drop_off_supplies() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let target = app.world_mut().spawn_empty().id();
        let entity = spawn_test_unit(&mut app, UnitCommand::DropOffSupplies(target));
        run_sync_system(&mut app);

        let state = app.world().get::<BaseCommandState>(entity).unwrap();
        assert_eq!(state.command_type, CommandType::DropOffSupplies);
        assert!(state.target_location.is_none());
        assert_eq!(state.target_entity, Some(target));
    }
}
