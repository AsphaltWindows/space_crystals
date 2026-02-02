use bevy::prelude::*;
use crate::map::{Tile, TileProperties, TileType, GridPosition};
use crate::units::{Unit, Owner, UnitHealth, UnitType, UnitBase, MovementSpeed};
use crate::combat::AttackCapability;
use crate::turret::Turret;
use crate::resources::Selected;

/// Plugin for HUD (Heads-Up Display) systems
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud)
            .add_systems(Update, (
                update_minimap_system,
                update_selected_units_grid_system,
            ));
    }
}

/// Marker component for the main HUD panel
#[derive(Component)]
struct HudPanel;

/// Marker component for the minimap section
#[derive(Component)]
pub struct MinimapSection;

/// Marker component for the selected units grid section
#[derive(Component)]
pub struct UnitsGridSection;

/// Marker component for minimap tiles
#[derive(Component)]
struct MinimapTile {
    grid_x: i32,
    grid_z: i32,
}

/// Marker component for minimap unit indicators
#[derive(Component)]
struct MinimapUnit {
    unit_entity: Entity,
}

/// Container for minimap tiles
#[derive(Component)]
struct MinimapContainer;

/// Marker for unit icon in selected units grid
#[derive(Component)]
struct UnitIcon {
    unit_entity: Entity,
}

/// Marker for unit health bar
#[derive(Component)]
struct UnitHealthBar {
    unit_entity: Entity,
}

/// Setup the HUD panel at the bottom of the screen
fn setup_hud(mut commands: Commands) {
    // Create main HUD container at bottom of screen
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(220.0),
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(0.0),
                    left: Val::Px(0.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    column_gap: Val::Px(10.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
                ..default()
            },
            HudPanel,
        ))
        .with_children(|parent| {
            // Left section: Minimap
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(5.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                    ..default()
                },
                MinimapSection,
            ))
            .with_children(|parent| {
                // Minimap title
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        "Minimap",
                        TextStyle {
                            font_size: 12.0,
                            color: Color::srgb(0.7, 0.7, 0.7),
                            ..default()
                        },
                    ),
                    style: Style {
                        margin: UiRect::bottom(Val::Px(5.0)),
                        ..default()
                    },
                    ..default()
                });

                // Minimap grid container
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(180.0),
                            height: Val::Px(180.0),
                            display: Display::Grid,
                            grid_template_columns: RepeatedGridTrack::flex(20, 1.0),
                            grid_template_rows: RepeatedGridTrack::flex(20, 1.0),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                        ..default()
                    },
                    MinimapContainer,
                ));
            });

            // Center section: Selected Units Grid (also shows detailed stats for single unit)
            parent.spawn((
                NodeBundle {
                    style: Style {
                        flex_grow: 1.0,
                        height: Val::Percent(100.0),
                        padding: UiRect::all(Val::Px(5.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.8)),
                    ..default()
                },
                UnitsGridSection,
            ))
            .with_children(|parent| {
                // Placeholder text for units grid
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        "No Units Selected",
                        TextStyle {
                            font_size: 16.0,
                            color: Color::srgb(0.7, 0.7, 0.7),
                            ..default()
                        },
                    ),
                    ..default()
                });
            });
        });

    info!("HUD panel initialized");
}

/// System to populate and update the minimap
fn update_minimap_system(
    mut commands: Commands,
    minimap_container: Query<Entity, With<MinimapContainer>>,
    minimap_tiles: Query<&MinimapTile>,
    tiles: Query<(&GridPosition, &TileType), With<Tile>>,
    units: Query<(&Transform, &Owner), With<Unit>>,
    minimap_units: Query<Entity, With<MinimapUnit>>,
) {
    // Initialize minimap tiles only if they don't exist yet
    if minimap_tiles.is_empty() {
        if let Ok(container_entity) = minimap_container.get_single() {
            // Create minimap tiles for each grid position
            for (grid_pos, tile_type) in tiles.iter() {
                let tile_color = get_tile_color(tile_type);

                commands.entity(container_entity).with_children(|parent| {
                    parent.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Px(9.0),
                                height: Val::Px(9.0),
                                ..default()
                            },
                            background_color: BackgroundColor(tile_color),
                            ..default()
                        },
                        MinimapTile {
                            grid_x: grid_pos.x,
                            grid_z: grid_pos.z,
                        },
                    ));
                });
            }

            info!("Minimap tiles initialized (one-time)");
        }
    }

    // TODO: Update unit positions on minimap
    // This would create small colored dots for units
    // For now, we'll just render the terrain
}

/// System to update the selected units grid
fn update_selected_units_grid_system(
    mut commands: Commands,
    units_grid_section: Query<Entity, With<UnitsGridSection>>,
    selected_units: Query<(Entity, &UnitType, &UnitHealth, &Owner, &UnitBase, &MovementSpeed, &AttackCapability, Option<&Turret>), (With<Unit>, With<Selected>)>,
    existing_icons: Query<(Entity, &UnitIcon)>,
    mut health_bars: Query<(&mut Style, &UnitHealthBar)>,
) {
    let grid_entity = match units_grid_section.get_single() {
        Ok(entity) => entity,
        Err(_) => return,
    };

    // Get currently selected units
    let selected: Vec<_> = selected_units.iter().collect();

    // Clear existing icons if selection changed
    let existing_count = existing_icons.iter().count();
    if existing_count != selected.len() {
        // Clear all existing icons
        for (icon_entity, _) in existing_icons.iter() {
            commands.entity(icon_entity).despawn_recursive();
        }

        // Create new icons for selected units
        if selected.is_empty() {
            // Show placeholder text
            commands.entity(grid_entity).despawn_descendants();
            commands.entity(grid_entity).insert(Style {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(5.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            });
            commands.entity(grid_entity).with_children(|parent| {
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        "No Units Selected",
                        TextStyle {
                            font_size: 16.0,
                            color: Color::srgb(0.7, 0.7, 0.7),
                            ..default()
                        },
                    ),
                    ..default()
                });
            });
        } else if selected.len() == 1 {
            // Single unit selected - show detailed stats in a card layout
            let (unit_entity, unit_type, unit_health, owner, unit_base, speed, attack, turret) = selected[0];

            commands.entity(grid_entity).despawn_descendants();
            commands.entity(grid_entity).insert(Style {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                ..default()
            });

            commands.entity(grid_entity).with_children(|parent| {
                // Left side: Unit icon with color and health bar
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Px(120.0),
                            height: Val::Px(150.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::all(Val::Px(8.0)),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.9)),
                        ..default()
                    },
                    UnitIcon {
                        unit_entity,
                    },
                ))
                .with_children(|icon_parent| {
                    // Unit color indicator (large square)
                    icon_parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(80.0),
                            height: Val::Px(80.0),
                            ..default()
                        },
                        background_color: BackgroundColor(owner.color()),
                        ..default()
                    });

                    // Health bar container
                    icon_parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(100.0),
                            height: Val::Px(12.0),
                            margin: UiRect::top(Val::Px(8.0)),
                            ..default()
                        },
                        background_color: BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                        ..default()
                    })
                    .with_children(|health_parent| {
                        let health_percent = unit_health.current / unit_health.max;
                        let health_color = get_health_color(health_percent);

                        health_parent.spawn((
                            NodeBundle {
                                style: Style {
                                    width: Val::Percent(health_percent * 100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                background_color: BackgroundColor(health_color),
                                ..default()
                            },
                            UnitHealthBar {
                                unit_entity,
                            },
                        ));
                    });

                    // Health text
                    icon_parent.spawn(TextBundle {
                        text: Text::from_section(
                            format!("{:.0} / {:.0}", unit_health.current, unit_health.max),
                            TextStyle {
                                font_size: 12.0,
                                color: Color::srgb(0.8, 0.8, 0.8),
                                ..default()
                            },
                        ),
                        style: Style {
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                        ..default()
                    });
                });

                // Right side: Detailed stats in organized groups
                parent.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::Start,
                        padding: UiRect::all(Val::Px(10.0)),
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                    ..default()
                })
                .with_children(|stats_parent| {
                    // Unit name (large, prominent)
                    stats_parent.spawn(TextBundle {
                        text: Text::from_section(
                            &unit_type.name,
                            TextStyle {
                                font_size: 20.0,
                                color: Color::srgb(1.0, 1.0, 1.0),
                                ..default()
                            },
                        ),
                        style: Style {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                        ..default()
                    });

                    // Unit Base Type
                    let base_type_name = match unit_base {
                        UnitBase::LightInfantry => "Light Infantry",
                        UnitBase::WheeledVehicle { .. } => "Wheeled Vehicle",
                        UnitBase::TrackedVehicle { .. } => "Tracked Vehicle",
                        UnitBase::DrillUnit { .. } => "Drill Unit",
                        UnitBase::HoverVehicle { .. } => "Hover Vehicle",
                        UnitBase::Mech { .. } => "Mech",
                    };
                    stats_parent.spawn(TextBundle {
                        text: Text::from_section(
                            format!("Type: {}", base_type_name),
                            TextStyle {
                                font_size: 14.0,
                                color: Color::srgb(0.8, 0.8, 0.8),
                                ..default()
                            },
                        ),
                        style: Style {
                            margin: UiRect::bottom(Val::Px(2.0)),
                            ..default()
                        },
                        ..default()
                    });

                    // Owner
                    let owner_text = match owner {
                        Owner::Player(id) => format!("Player {}", id),
                        Owner::Neutral => "Neutral".to_string(),
                    };
                    stats_parent.spawn(TextBundle {
                        text: Text::from_section(
                            format!("Owner: {}", owner_text),
                            TextStyle {
                                font_size: 14.0,
                                color: owner.color(),
                                ..default()
                            },
                        ),
                        style: Style {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                        ..default()
                    });

                    // Combat stats header
                    stats_parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Combat",
                            TextStyle {
                                font_size: 12.0,
                                color: Color::srgb(0.9, 0.7, 0.7),
                                ..default()
                            },
                        ),
                        ..default()
                    });

                    // Damage
                    stats_parent.spawn(TextBundle {
                        text: Text::from_section(
                            format!("  Damage: {:.0}", attack.damage),
                            TextStyle {
                                font_size: 13.0,
                                color: Color::srgb(0.8, 0.8, 0.8),
                                ..default()
                            },
                        ),
                        ..default()
                    });

                    // Range
                    stats_parent.spawn(TextBundle {
                        text: Text::from_section(
                            format!("  Range: {:.1}", attack.range),
                            TextStyle {
                                font_size: 13.0,
                                color: Color::srgb(0.8, 0.8, 0.8),
                                ..default()
                            },
                        ),
                        style: Style {
                            margin: UiRect::bottom(Val::Px(6.0)),
                            ..default()
                        },
                        ..default()
                    });

                    // Movement header
                    stats_parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Movement",
                            TextStyle {
                                font_size: 12.0,
                                color: Color::srgb(0.7, 0.8, 0.9),
                                ..default()
                            },
                        ),
                        ..default()
                    });

                    // Speed
                    stats_parent.spawn(TextBundle {
                        text: Text::from_section(
                            format!("  Speed: {:.1}", speed.0),
                            TextStyle {
                                font_size: 13.0,
                                color: Color::srgb(0.8, 0.8, 0.8),
                                ..default()
                            },
                        ),
                        style: Style {
                            margin: UiRect::bottom(Val::Px(6.0)),
                            ..default()
                        },
                        ..default()
                    });

                    // Turret info if present
                    if let Some(turret) = turret {
                        let turn_angle_deg = turret.turn_angle.to_degrees();
                        let turn_rate_deg = turret.turn_rate.to_degrees();

                        stats_parent.spawn(TextBundle {
                            text: Text::from_section(
                                "Turret",
                                TextStyle {
                                    font_size: 12.0,
                                    color: Color::srgb(0.8, 0.7, 0.9),
                                    ..default()
                                },
                            ),
                            ..default()
                        });

                        stats_parent.spawn(TextBundle {
                            text: Text::from_section(
                                format!("  Arc: {:.0}°", turn_angle_deg),
                                TextStyle {
                                    font_size: 13.0,
                                    color: Color::srgb(0.8, 0.8, 0.8),
                                    ..default()
                                },
                            ),
                            ..default()
                        });

                        stats_parent.spawn(TextBundle {
                            text: Text::from_section(
                                format!("  Turn Rate: {:.0}°/s", turn_rate_deg),
                                TextStyle {
                                    font_size: 13.0,
                                    color: Color::srgb(0.8, 0.8, 0.8),
                                    ..default()
                                },
                            ),
                            ..default()
                        });
                    }
                });
            });
        } else {
            // Multiple units selected - show grid with stats under each icon
            commands.entity(grid_entity).despawn_descendants();

            // Use 4 columns for expanded cards with stats
            commands.entity(grid_entity).insert(Style {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(5.0)),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(4, 1.0),
                column_gap: Val::Px(8.0),
                row_gap: Val::Px(8.0),
                overflow: Overflow::clip(),
                ..default()
            });

            // Add unit cards with stats (limit to 12 units)
            for (unit_entity, unit_type, unit_health, owner, _unit_base, _speed, attack, _turret) in selected.iter().take(12) {
                commands.entity(grid_entity).with_children(|parent| {
                    parent.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Px(140.0),
                                height: Val::Px(95.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Start,
                                padding: UiRect::all(Val::Px(4.0)),
                                ..default()
                            },
                            background_color: BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.9)),
                            ..default()
                        },
                        UnitIcon {
                            unit_entity: *unit_entity,
                        },
                    ))
                    .with_children(|card| {
                        // Top row: color square + name
                        card.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                column_gap: Val::Px(4.0),
                                margin: UiRect::bottom(Val::Px(2.0)),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|top_row| {
                            // Unit color indicator (smaller)
                            top_row.spawn(NodeBundle {
                                style: Style {
                                    width: Val::Px(16.0),
                                    height: Val::Px(16.0),
                                    ..default()
                                },
                                background_color: BackgroundColor(owner.color()),
                                ..default()
                            });

                            // Unit name (truncated if needed)
                            let display_name = if unit_type.name.len() > 14 {
                                format!("{}...", &unit_type.name[..11])
                            } else {
                                unit_type.name.clone()
                            };
                            top_row.spawn(TextBundle {
                                text: Text::from_section(
                                    display_name,
                                    TextStyle {
                                        font_size: 11.0,
                                        color: Color::srgb(1.0, 1.0, 1.0),
                                        ..default()
                                    },
                                ),
                                ..default()
                            });
                        });

                        // Health row: "HP: current/max"
                        card.spawn(TextBundle {
                            text: Text::from_section(
                                format!("HP: {:.0}/{:.0}", unit_health.current, unit_health.max),
                                TextStyle {
                                    font_size: 10.0,
                                    color: Color::srgb(0.7, 0.9, 0.7),
                                    ..default()
                                },
                            ),
                            style: Style {
                                margin: UiRect::bottom(Val::Px(1.0)),
                                ..default()
                            },
                            ..default()
                        });

                        // Stats row: "Dmg: X  Rng: Y"
                        card.spawn(TextBundle {
                            text: Text::from_section(
                                format!("Dmg: {:.0}  Rng: {:.1}", attack.damage, attack.range),
                                TextStyle {
                                    font_size: 10.0,
                                    color: Color::srgb(0.9, 0.7, 0.7),
                                    ..default()
                                },
                            ),
                            style: Style {
                                margin: UiRect::bottom(Val::Px(3.0)),
                                ..default()
                            },
                            ..default()
                        });

                        // Health bar container
                        card.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Px(6.0),
                                ..default()
                            },
                            background_color: BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                            ..default()
                        })
                        .with_children(|health_parent| {
                            let health_percent = unit_health.current / unit_health.max;
                            let health_color = get_health_color(health_percent);

                            health_parent.spawn((
                                NodeBundle {
                                    style: Style {
                                        width: Val::Percent(health_percent * 100.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    background_color: BackgroundColor(health_color),
                                    ..default()
                                },
                                UnitHealthBar {
                                    unit_entity: *unit_entity,
                                },
                            ));
                        });
                    });
                });
            }
        }
    } else {
        // Update health bars for existing icons
        for (mut style, health_bar) in health_bars.iter_mut() {
            if let Ok((_, _, unit_health, _, _, _, _, _)) = selected_units.get(health_bar.unit_entity) {
                let health_percent = unit_health.current / unit_health.max;
                style.width = Val::Percent(health_percent * 100.0);
            }
        }
    }
}

/// Get color for health bar based on health percentage
fn get_health_color(health_percent: f32) -> Color {
    if health_percent > 0.6 {
        Color::srgb(0.2, 0.8, 0.2) // Green
    } else if health_percent > 0.3 {
        Color::srgb(0.8, 0.8, 0.2) // Yellow
    } else {
        Color::srgb(0.8, 0.2, 0.2) // Red
    }
}

/// Get color for tile type on minimap
fn get_tile_color(tile_type: &TileType) -> Color {
    match tile_type {
        TileType::Plane => Color::srgb(0.3, 0.6, 0.3),        // Green
        TileType::RuggedTerrain => Color::srgb(0.5, 0.4, 0.3), // Brown
        TileType::Cliff => Color::srgb(0.4, 0.4, 0.4),         // Gray
        TileType::Mountain => Color::srgb(0.5, 0.5, 0.5),      // Light gray
        TileType::Water => Color::srgb(0.2, 0.3, 0.6),         // Blue
    }
}
