use bevy::prelude::*;

/// Component representing a unit's current command
#[derive(Component, Clone, Debug)]
pub enum UnitCommand {
    Idle,
    Move(Vec3),
    AttackTarget(Entity),
    AttackLocation(Vec3),
    Patrol { start: Vec3, end: Vec3, going_to_end: bool },
    HoldPosition,
    Stop,
}

/// Resource tracking the current command mode for issuing commands
#[derive(Resource, Default)]
pub struct CommandMode {
    pub mode: CommandType,
}

/// Types of command modes for input
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum CommandType {
    #[default]
    Default,    // Right-click: Move to ground, Attack enemy
    Move,       // M key: Move command
    Attack,     // A key: Attack command
    AttackGround, // G key: Attack ground command
    Patrol,     // P key: Patrol command
}

impl CommandType {
    /// Get the display name for this command type
    pub fn name(&self) -> &str {
        match self {
            CommandType::Default => "Default",
            CommandType::Move => "Move",
            CommandType::Attack => "Attack",
            CommandType::AttackGround => "Attack Ground",
            CommandType::Patrol => "Patrol",
        }
    }

    /// Get the hotkey for this command type
    pub fn hotkey(&self) -> &str {
        match self {
            CommandType::Default => "",
            CommandType::Move => "M",
            CommandType::Attack => "A",
            CommandType::AttackGround => "G",
            CommandType::Patrol => "P",
        }
    }
}

/// Unit state for behavior management
#[derive(Component, Default, Debug)]
pub enum UnitState {
    #[default]
    Idle,
    Busy,
    HoldingPosition,
}

/// Component marking a unit as holding position
#[derive(Component)]
pub struct HoldingPosition;

/// Plugin for command system
pub struct CommandsPlugin;

impl Plugin for CommandsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CommandMode::default())
            .add_systems(Update, (
                command_input_system,
                command_ui_system,
                hold_position_system,
                stop_command_system,
                patrol_command_system,
            ));
    }
}

/// System to handle command input (hotkeys)
fn command_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut command_mode: ResMut<CommandMode>,
) {
    // Check hotkeys
    if keyboard.just_pressed(KeyCode::KeyM) {
        command_mode.mode = CommandType::Move;
        info!("Command mode: Move");
    } else if keyboard.just_pressed(KeyCode::KeyA) {
        command_mode.mode = CommandType::Attack;
        info!("Command mode: Attack");
    } else if keyboard.just_pressed(KeyCode::KeyG) {
        command_mode.mode = CommandType::AttackGround;
        info!("Command mode: Attack Ground");
    } else if keyboard.just_pressed(KeyCode::KeyP) {
        command_mode.mode = CommandType::Patrol;
        info!("Command mode: Patrol");
    } else if keyboard.just_pressed(KeyCode::KeyH) {
        // H key: Hold Position (immediate command, no mode change)
        // This will be handled in the hold_position_system
    } else if keyboard.just_pressed(KeyCode::KeyS) {
        // S key: Stop (immediate command, no mode change)
        // This will be handled in the stop_command_system
    } else if keyboard.just_pressed(KeyCode::Escape) {
        // ESC: Cancel command mode
        if command_mode.mode != CommandType::Default {
            command_mode.mode = CommandType::Default;
            info!("Command mode: Default");
        }
    }
}

/// System to handle Hold Position command (H key)
fn hold_position_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    selected_units: Query<Entity, (With<crate::units::Unit>, With<crate::resources::Selected>)>,
) {
    if keyboard.just_pressed(KeyCode::KeyH) {
        let count = selected_units.iter().count();
        if count > 0 {
            for entity in selected_units.iter() {
                commands.entity(entity)
                    .remove::<crate::units::MoveTarget>()
                    .remove::<crate::pathfinding::Path>()
                    .insert(HoldingPosition)
                    .insert(UnitCommand::HoldPosition);
            }
            info!("Hold Position: {} unit(s)", count);
        }
    }
}

/// System to handle Stop command (S key)
fn stop_command_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected_units: Query<
        (Entity, &mut crate::units::Velocity),
        (With<crate::units::Unit>, With<crate::resources::Selected>)
    >,
) {
    if keyboard.just_pressed(KeyCode::KeyS) {
        let count = selected_units.iter().count();
        if count > 0 {
            for (entity, mut velocity) in selected_units.iter_mut() {
                velocity.0 = Vec3::ZERO;
                commands.entity(entity)
                    .remove::<crate::units::MoveTarget>()
                    .remove::<crate::pathfinding::Path>()
                    .remove::<HoldingPosition>()
                    .insert(UnitCommand::Stop);
            }
            info!("Stop: {} unit(s)", count);
        }
    }
}

/// System to handle patrol command execution
fn patrol_command_system(
    time: Res<Time>,
    mut commands: Commands,
    tiles: Query<(&crate::map::GridPosition, &crate::map::TileProperties), With<crate::map::Tile>>,
    mut units: Query<
        (Entity, &Transform, &mut UnitCommand, &crate::units::UnitBase),
        With<crate::units::Unit>
    >,
) {
    for (entity, transform, mut command, unit_base) in units.iter_mut() {
        if let UnitCommand::Patrol { start, end, going_to_end } = *command {
            let current_pos = transform.translation;
            let target = if going_to_end { end } else { start };

            // Check if we've reached the target
            let distance = Vec3::new(
                target.x - current_pos.x,
                0.0,
                target.z - current_pos.z,
            ).length();

            if distance < 0.5 {
                // Reached target, switch direction
                let new_going_to_end = !going_to_end;
                let new_target = if new_going_to_end { end } else { start };

                // Calculate new path
                let start_grid = crate::pathfinding::world_to_grid(current_pos);
                let target_grid = crate::pathfinding::world_to_grid(new_target);

                if let Some(path) = crate::pathfinding::find_path(start_grid, target_grid, &tiles, unit_base) {
                    let smoothed_waypoints = crate::pathfinding::smooth_path(path);

                    commands.entity(entity).insert((
                        crate::units::MoveTarget(new_target),
                        crate::pathfinding::Path {
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

/// System to render command UI
fn command_ui_system(
    mut commands: Commands,
    command_mode: Res<CommandMode>,
    existing_ui: Query<Entity, With<CommandUIRoot>>,
    asset_server: Res<AssetServer>,
) {
    // Only rebuild UI when command mode changes
    if !command_mode.is_changed() {
        return;
    }

    // Remove existing UI
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Create command UI panel
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    right: Val::Px(20.0),
                    bottom: Val::Px(20.0),
                    width: Val::Px(300.0),
                    height: Val::Px(120.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    display: Display::Grid,
                    grid_template_columns: vec![
                        GridTrack::px(90.0),
                        GridTrack::px(90.0),
                        GridTrack::px(90.0),
                    ],
                    grid_template_rows: vec![
                        GridTrack::px(40.0),
                        GridTrack::px(40.0),
                    ],
                    row_gap: Val::Px(5.0),
                    column_gap: Val::Px(5.0),
                    ..default()
                },
                background_color: Color::srgba(0.1, 0.1, 0.1, 0.8).into(),
                ..default()
            },
            CommandUIRoot,
        ))
        .with_children(|parent| {
            // Move button
            create_command_button(parent, "Move", "M", command_mode.mode == CommandType::Move);

            // Attack button
            create_command_button(parent, "Attack", "A", command_mode.mode == CommandType::Attack);

            // Attack Ground button
            create_command_button(parent, "Atk Gnd", "G", command_mode.mode == CommandType::AttackGround);

            // Patrol button
            create_command_button(parent, "Patrol", "P", command_mode.mode == CommandType::Patrol);

            // Hold Position button
            create_command_button(parent, "Hold", "H", false);

            // Stop button
            create_command_button(parent, "Stop", "S", false);
        });
}

/// Helper function to create a command button
fn create_command_button(parent: &mut ChildBuilder, label: &str, hotkey: &str, is_active: bool) {
    let bg_color = if is_active {
        Color::srgb(0.3, 0.5, 0.3) // Green when active
    } else {
        Color::srgb(0.2, 0.2, 0.2) // Dark gray
    };

    parent.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: bg_color.into(),
        ..default()
    })
    .with_children(|button| {
        // Label text
        button.spawn(TextBundle::from_section(
            label,
            TextStyle {
                font_size: 14.0,
                color: Color::WHITE,
                ..default()
            },
        ));

        // Hotkey text
        button.spawn(TextBundle::from_section(
            format!("[{}]", hotkey),
            TextStyle {
                font_size: 10.0,
                color: Color::srgb(0.7, 0.7, 0.7),
                ..default()
            },
        ));
    });
}

/// Marker component for command UI root
#[derive(Component)]
struct CommandUIRoot;
