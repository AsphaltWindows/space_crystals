use bevy::prelude::*;
use crate::types::*;
use crate::game::combat::types::AttackState;
use crate::game::world::types::{Tile, TilePreset, GridMap};
use crate::game::units::types::*;
use crate::game::units::utils::{world_to_grid, smooth_path};
use crate::ui::types::ObjectInterfaceState;
use crate::game::units::types::commands::CommandType;


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
    selected_units: Query<(Entity, Option<&AttackState>), (With<Unit>, With<Selected>)>,
    interface_state: Res<ObjectInterfaceState>,
    selection: Res<Selection>,
) {
    // Panel hidden = Default state with empty selection
    let panel_hidden = matches!(*interface_state, ObjectInterfaceState::Default) && selection.groups.is_empty();
    if !panel_hidden { return; }
    if keyboard.just_pressed(KeyCode::KeyH) {
        let count = selected_units.iter().count();
        if count > 0 {
            for (entity, attack_state_opt) in &selected_units {
                // Skip units in non-interruptible attack phases
                if let Some(attack_state) = attack_state_opt {
                    if !attack_state.phase.is_interruptible() {
                        continue;
                    }
                }
                commands.entity(entity)
                    .remove::<MoveTarget>()
                    .remove::<Path>()
                    .insert(HoldingPosition)
                    .insert(UnitCommand::HoldPosition);
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
        (Entity, &mut Velocity, Option<&AttackState>),
        (With<Unit>, With<Selected>)
    >,
    interface_state: Res<ObjectInterfaceState>,
    selection: Res<Selection>,
) {
    let panel_hidden = matches!(*interface_state, ObjectInterfaceState::Default) && selection.groups.is_empty();
    if !panel_hidden { return; }
    if keyboard.just_pressed(KeyCode::KeyS) {
        let count = selected_units.iter().count();
        if count > 0 {
            for (entity, mut velocity, attack_state_opt) in &mut selected_units {
                // Skip units in non-interruptible attack phases
                if let Some(attack_state) = attack_state_opt {
                    if !attack_state.phase.is_interruptible() {
                        continue;
                    }
                }
                velocity.0 = Vec3::ZERO;
                commands.entity(entity)
                    .remove::<MoveTarget>()
                    .remove::<Path>()
                    .remove::<HoldingPosition>()
                    .insert(UnitCommand::Stop);
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
