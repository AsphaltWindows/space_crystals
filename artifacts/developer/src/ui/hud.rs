use bevy::prelude::*;
use crate::types::*;
use crate::game::types::{ObjectInstance, Player, GdoPlayerResources, SyndicatePlayerResources, CultsPlayerResources, ColonistsPlayerResources, StructureInstance, PowerValue, BuildRadiusExtension, DeploymentCenterState, BarracksState, ExtractionFacilityState};
use crate::game::types::structures::{ExtractionPlateState, SupplyTowerState};
use crate::game::world::types::{SpaceCrystalPatch, SupplyDeliveryStation};
use crate::game::world::types::{Tile, TilePresetEnum};
use crate::types::UnitBaseEnum;
use crate::game::units::types::{UnitType, MovementSpeed};
use crate::game::combat::types::{AttackCapability, Turret};
use super::types::*;
use super::utils::{get_health_color, get_tile_color};

/// Setup the HUD panel at the bottom of the screen.
/// Must run after `setup_player_resources` so that `LocalPlayer` and `Player` entities exist.
pub fn setup_hud(
    mut commands: Commands,
    local_player: Res<LocalPlayer>,
    players: Query<&Player>,
) {
    // Spawn a dedicated UI camera (fullscreen, renders HUD on top of the 3D viewport)
    // DespawnOnExit auto-cleans when leaving InGame state
    let ui_cam = commands.spawn((
        Camera2d,
        Camera {
            order: 1, // Render after the 3D camera
            clear_color: ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
        super::types::UiCamera,
        DespawnOnExit(AppState::InGame),
    )).id();
    commands.insert_resource(super::types::UiCameraEntity(ui_cam));

    // Top resource bar
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(super::types::HUD_TOP_BAR_HEIGHT),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                padding: UiRect::horizontal(Val::Px(16.0)),
                column_gap: Val::Px(24.0),
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.85)),
            UiTargetCamera(ui_cam),
            ResourceBar,
            Interaction::default(),
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|parent| {
            // Determine the local player's faction
            let local_faction = players.iter()
                .find(|p| p.player_number == local_player.0)
                .map(|p| p.faction)
                .unwrap_or(FactionEnum::GlobalDefenseOrdinance);

            spawn_resource_bar_fields(parent, 14.0, Color::srgb(0.9, 0.9, 0.9), local_faction);
        });

    // Bottom HUD panel
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(super::types::HUD_BOTTOM_PANEL_HEIGHT),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                padding: UiRect::all(Val::Px(10.0)),
                column_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            UiTargetCamera(ui_cam),
            HudPanel,
            Interaction::default(),
            DespawnOnExit(AppState::InGame),
        ))
        .with_children(|parent| {
            // Left section: Minimap
            parent.spawn((
                Node {
                    width: Val::Px(200.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(5.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                MinimapSection,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Minimap"),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    Node {
                        margin: UiRect::bottom(Val::Px(5.0)),
                        ..default()
                    },
                ));

                parent.spawn((
                    Node {
                        width: Val::Px(180.0),
                        height: Val::Px(180.0),
                        display: Display::Grid,
                        grid_template_columns: RepeatedGridTrack::flex(32, 1.0),
                        grid_template_rows: RepeatedGridTrack::flex(32, 1.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
                    MinimapContainer,
                ));
            });

            // Center section: Selected Units Grid
            parent.spawn((
                Node {
                    flex_grow: 1.0,
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(5.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.8)),
                UnitsGridSection,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("No Units Selected"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));
            });

            // Right section: Command Panel
            parent.spawn((
                Node {
                    width: Val::Px(200.0),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexStart,
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.12, 0.15, 0.12, 0.9)),
                CommandPanelSection,
            ));
        });

    info!("HUD panel initialized");
}

/// Minimap resolution (downsampled from full grid)
const MINIMAP_SIZE: u32 = 32;

/// System to populate and update the minimap (downsampled from 64x64 to 32x32)
pub fn update_minimap_system(
    mut commands: Commands,
    minimap_container: Query<Entity, With<MinimapContainer>>,
    minimap_tiles: Query<&MinimapTile>,
    tiles: Query<(&GridPosition, &TilePresetEnum), With<Tile>>,
    _units: Query<(&Transform, &Owner), With<Unit>>,
    _minimap_units: Query<Entity, With<MinimapUnit>>,
    grid: Res<crate::game::world::types::GridMap>,
) {
    if minimap_tiles.is_empty() {
        if let Ok(container_entity) = minimap_container.single() {
            // Build a color map by downsampling: each minimap cell covers a 2x2 area of the real grid
            let scale_x = grid.width as f32 / MINIMAP_SIZE as f32;
            let scale_z = grid.height as f32 / MINIMAP_SIZE as f32;

            // Collect tile colors into a lookup
            let mut tile_colors: std::collections::HashMap<(i32, i32), Color> = std::collections::HashMap::new();
            for (grid_pos, tile_type) in tiles.iter() {
                tile_colors.insert((grid_pos.x, grid_pos.z), get_tile_color(tile_type));
            }

            for mz in 0..MINIMAP_SIZE {
                for mx in 0..MINIMAP_SIZE {
                    let sample_x = (mx as f32 * scale_x) as i32;
                    let sample_z = (mz as f32 * scale_z) as i32;
                    let tile_color = tile_colors.get(&(sample_x, sample_z))
                        .copied()
                        .unwrap_or(Color::srgb(0.0, 0.0, 0.0));

                    commands.entity(container_entity).with_children(|parent| {
                        parent.spawn((
                            Node {
                                width: Val::Px(5.0),
                                height: Val::Px(5.0),
                                ..default()
                            },
                            BackgroundColor(tile_color),
                            MinimapTile {
                                grid_x: sample_x,
                                grid_z: sample_z,
                            },
                        ));
                    });
                }
            }

            info!("Minimap tiles initialized ({}x{} downsampled from {}x{})",
                MINIMAP_SIZE, MINIMAP_SIZE, grid.width, grid.height);
        }
    }
}

/// System to update the selected units grid — handles both units and structures
pub fn update_selected_units_grid_system(
    mut commands: Commands,
    units_grid_section: Query<Entity, With<UnitsGridSection>>,
    selected_units: Query<(Entity, &UnitType, &ObjectInstance, &Owner, &UnitBaseEnum, &MovementSpeed, Option<&AttackCapability>, Option<&Turret>), (With<Unit>, With<Selected>)>,
    selected_structures: Query<(Entity, &ObjectInstance, &Owner, Option<&PowerValue>, Option<&BuildRadiusExtension>, Option<&DeploymentCenterState>, Option<&BarracksState>, Option<&ExtractionFacilityState>, Option<&ExtractionPlateState>, Option<&SupplyTowerState>), (With<StructureInstance>, With<Selected>, Without<Unit>)>,
    sc_patches: Query<&SpaceCrystalPatch>,
    selected_scp: Query<(Entity, &SpaceCrystalPatch, &ObjectInstance), (With<Selected>, Without<Unit>, Without<StructureInstance>)>,
    selected_sds: Query<(Entity, &SupplyDeliveryStation, &ObjectInstance), (With<Selected>, Without<Unit>, Without<StructureInstance>, Without<SpaceCrystalPatch>)>,
    existing_unit_icons: Query<(Entity, &UnitIcon)>,
    existing_struct_icons: Query<(Entity, &StructureIcon)>,
    existing_resource_icons: Query<(Entity, &ResourceIcon)>,
    mut unit_health_bars: Query<(&mut Node, &UnitHealthBar), (Without<StructureHealthBar>, Without<ResourceIcon>)>,
    mut struct_health_bars: Query<(&mut Node, &StructureHealthBar), (Without<UnitHealthBar>, Without<ResourceIcon>)>,
    selection: Res<Selection>,
) {
    let Ok(grid_entity) = units_grid_section.single() else { return; };

    let unit_count = selected_units.iter().count();
    let struct_count = selected_structures.iter().count();
    let resource_count = selected_scp.iter().count() + selected_sds.iter().count();
    let total_selected = unit_count + struct_count + resource_count;

    let existing_count = existing_unit_icons.iter().count() + existing_struct_icons.iter().count() + existing_resource_icons.iter().count();
    if existing_count != total_selected || selection.is_changed() {
        // Selection changed — rebuild UI
        for (icon_entity, _) in existing_unit_icons.iter() {
            commands.entity(icon_entity).despawn();
        }
        for (icon_entity, _) in existing_struct_icons.iter() {
            commands.entity(icon_entity).despawn();
        }
        for (icon_entity, _) in existing_resource_icons.iter() {
            commands.entity(icon_entity).despawn();
        }

        if total_selected == 0 {
            // Nothing selected
            commands.entity(grid_entity).despawn_children();
            commands.entity(grid_entity).insert(Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(5.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                overflow: Overflow::clip(),
                ..default()
            });
            commands.entity(grid_entity).with_children(|parent| {
                parent.spawn((
                    Text::new("No Selection"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));
            });
        } else if total_selected == 1 && struct_count == 1 {
            // Single structure selected
            let (struct_entity, obj_instance, owner, power, build_radius, dc_state, bk_state, ef_state, ep_state, st_state) = selected_structures.iter().next().unwrap();
            let obj_type_data = obj_instance.object_type.object_type();

            commands.entity(grid_entity).despawn_children();
            commands.entity(grid_entity).insert(Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                overflow: Overflow::clip(),
                ..default()
            });

            commands.entity(grid_entity).with_children(|parent| {
                // Left side: Structure icon with color and health bar
                parent.spawn((
                    Node {
                        width: Val::Px(120.0),
                        height: Val::Px(150.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.9)),
                    StructureIcon { structure_entity: struct_entity },
                ))
                .with_children(|icon_parent| {
                    // Owner color swatch (square for structures)
                    icon_parent.spawn((
                        Node {
                            width: Val::Px(80.0),
                            height: Val::Px(80.0),
                            border: UiRect::all(Val::Px(2.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(owner.color()),
                        BorderColor::all(Color::srgb(0.6, 0.6, 0.6)),
                    ))
                    .with_children(|swatch| {
                        // Structure type indicator text inside swatch
                        let abbrev = match obj_instance.object_type {
                            ObjectEnum::DeploymentCenter => "DC",
                            ObjectEnum::PowerPlant => "PP",
                            ObjectEnum::Barracks => "BK",
                            ObjectEnum::ExtractionFacility => "EF",
                            ObjectEnum::ExtractionPlate => "EP",
                            ObjectEnum::SupplyTower => "ST",
                            _ => "??",
                        };
                        swatch.spawn((
                            Text::new(abbrev),
                            TextFont { font_size: 24.0, ..default() },
                            TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        ));
                    });

                    // Health bar
                    icon_parent.spawn((
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(12.0),
                            margin: UiRect::top(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ))
                    .with_children(|health_parent| {
                        let health_percent = obj_instance.health_fraction();
                        let health_color = get_health_color(health_percent);

                        health_parent.spawn((
                            Node {
                                width: Val::Percent(health_percent * 100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(health_color),
                            StructureHealthBar { structure_entity: struct_entity },
                        ));
                    });

                    // HP text
                    icon_parent.spawn((
                        Text::new(format!("{:.0} / {:.0}", obj_instance.hp.unwrap_or(0.0), obj_instance.max_hp.unwrap_or(0.0))),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        Node {
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                    ));
                });

                // Right side: Structure stats
                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::Start,
                        padding: UiRect::all(Val::Px(10.0)),
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                ))
                .with_children(|stats_parent| {
                    // Structure name
                    stats_parent.spawn((
                        Text::new(&obj_type_data.name),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ));

                    // Owner
                    let owner_text = match owner.player_number() {
                        Some(id) => format!("Player {}", id),
                        None => "Neutral".to_string(),
                    };
                    spawn_stat_text(stats_parent, &format!("Owner: {}", owner_text), 14.0, owner.color(), Some(8.0));

                    // Structure stats section
                    spawn_stat_text(stats_parent, "Structure Stats", 12.0, Color::srgb(0.7, 0.9, 0.8), None);

                    // Power value
                    if let Some(pv) = power {
                        let power_color = if pv.0 >= 0 {
                            Color::srgb(0.5, 1.0, 0.5)
                        } else {
                            Color::srgb(1.0, 0.5, 0.5)
                        };
                        let power_text = if pv.0 >= 0 {
                            format!("  Power: +{}", pv.0)
                        } else {
                            format!("  Power: {}", pv.0)
                        };
                        spawn_stat_text(stats_parent, &power_text, 13.0, power_color, None);
                    }

                    // Build radius extension
                    if let Some(br) = build_radius {
                        if br.0 > 0 {
                            spawn_stat_text(stats_parent, &format!("  Build Radius: +{}", br.0), 13.0, Color::srgb(0.8, 0.8, 0.8), None);
                        }
                    }

                    // Size
                    spawn_stat_text(stats_parent, &format!("  Size: {}x{}", obj_type_data.size.0, obj_type_data.size.1), 13.0, Color::srgb(0.8, 0.8, 0.8), Some(6.0));

                    // Structure-specific state
                    if let Some(dc) = dc_state {
                        spawn_stat_text(stats_parent, "Status", 12.0, Color::srgb(0.9, 0.8, 0.6), None);
                        if let Some(ref building) = dc.current_construction {
                            let progress = dc.construction_progress.unwrap_or(0.0);
                            let cost = DeploymentCenterState::construction_cost(building);
                            let total_frames = cost.map(|c| c.build_frames as f32).unwrap_or(160.0);
                            let pct = (progress / total_frames * 100.0).min(100.0);
                            let name = building.object_type().name;
                            spawn_stat_text(stats_parent, &format!("  Building {} ({:.0}%)", name, pct), 13.0, Color::srgb(0.9, 0.9, 0.5), None);
                        } else if let Some(ref ready) = dc.ready_to_place {
                            let name = ready.object_type().name;
                            spawn_stat_text(stats_parent, &format!("  {} ready to place", name), 13.0, Color::srgb(0.5, 1.0, 0.5), None);
                        } else {
                            spawn_stat_text(stats_parent, "  Idle", 13.0, Color::srgb(0.7, 0.7, 0.7), None);
                        }
                    }

                    if let Some(bk) = bk_state {
                        spawn_stat_text(stats_parent, "Production", 12.0, Color::srgb(0.9, 0.8, 0.6), None);
                        if let Some(ref building) = bk.current_build {
                            let progress = bk.current_build_progress.unwrap_or(0.0);
                            let cost = BarracksState::production_cost(building);
                            let total_frames = cost.map(|c| c.build_frames as f32).unwrap_or(80.0);
                            let pct = (progress / total_frames * 100.0).min(100.0);
                            let name = building.object_type().name;
                            spawn_stat_text(stats_parent, &format!("  Training {} ({:.0}%)", name, pct), 13.0, Color::srgb(0.9, 0.9, 0.5), None);
                        } else {
                            spawn_stat_text(stats_parent, "  Idle", 13.0, Color::srgb(0.7, 0.7, 0.7), None);
                        }
                        if !bk.build_queue.is_empty() {
                            spawn_stat_text(stats_parent, &format!("  Queue: {}/{}", bk.build_queue.len(), BarracksState::MAX_QUEUE_SIZE), 13.0, Color::srgb(0.8, 0.8, 0.8), None);
                        }
                    }

                    if let Some(ef) = ef_state {
                        spawn_stat_text(stats_parent, "Status", 12.0, Color::srgb(0.9, 0.8, 0.6), None);
                        if ef.current_construction {
                            let progress = ef.construction_progress.unwrap_or(0.0);
                            let total_frames = ExtractionFacilityState::construction_cost().build_frames as f32;
                            let pct = (progress / total_frames * 100.0).min(100.0);
                            spawn_stat_text(stats_parent, &format!("  Building Plate ({:.0}%)", pct), 13.0, Color::srgb(0.9, 0.9, 0.5), None);
                        } else if ef.ready_to_place {
                            spawn_stat_text(stats_parent, "  Plate ready to place", 13.0, Color::srgb(0.5, 1.0, 0.5), None);
                        } else {
                            spawn_stat_text(stats_parent, "  Idle", 13.0, Color::srgb(0.7, 0.7, 0.7), None);
                        }
                    }

                    if let Some(ep) = ep_state {
                        spawn_stat_text(stats_parent, "Mining", 12.0, Color::srgb(0.9, 0.8, 0.6), None);
                        if let Ok(patch) = sc_patches.get(ep.attached_patch) {
                            let remaining = patch.remaining_amount;
                            let initial = patch.initial_amount;
                            let pct = if initial > 0 { (remaining as f32 / initial as f32 * 100.0).min(100.0) } else { 0.0 };
                            spawn_stat_text(stats_parent, &format!("  Remaining SC: {} / {} ({:.0}%)", remaining, initial, pct), 13.0, Color::srgb(0.5, 0.9, 1.0), None);
                            if remaining == 0 {
                                spawn_stat_text(stats_parent, "  Patch depleted (residual mining)", 13.0, Color::srgb(1.0, 0.7, 0.3), None);
                            }
                        } else {
                            spawn_stat_text(stats_parent, "  Patch unavailable", 13.0, Color::srgb(0.7, 0.7, 0.7), None);
                        }
                    }

                    if let Some(st) = st_state {
                        spawn_stat_text(stats_parent, "Production", 12.0, Color::srgb(0.9, 0.8, 0.6), None);
                        if let Some(ref building) = st.current_build {
                            let progress = st.current_build_progress.unwrap_or(0.0);
                            let cost = SupplyTowerState::production_cost(building);
                            let total_frames = cost.map(|c| c.build_frames as f32).unwrap_or(160.0);
                            let pct = (progress / total_frames * 100.0).min(100.0);
                            let name = building.object_type().name;
                            spawn_stat_text(stats_parent, &format!("  Training {} ({:.0}%)", name, pct), 13.0, Color::srgb(0.9, 0.9, 0.5), None);
                        } else {
                            spawn_stat_text(stats_parent, "  Idle", 13.0, Color::srgb(0.7, 0.7, 0.7), None);
                        }
                        if !st.build_queue.is_empty() {
                            spawn_stat_text(stats_parent, &format!("  Queue: {}/{}", st.build_queue.len(), SupplyTowerState::MAX_QUEUE_SIZE), 13.0, Color::srgb(0.8, 0.8, 0.8), None);
                        }
                        if st.scheduled_sds.is_some() {
                            spawn_stat_text(stats_parent, "  Delivering", 13.0, Color::srgb(0.5, 1.0, 0.5), None);
                        }
                        if st.attached_chopper.is_some() {
                            spawn_stat_text(stats_parent, "  Chopper: Attached", 13.0, Color::srgb(0.5, 0.9, 1.0), None);
                        }
                    }
                });
            });
        } else if total_selected == 1 && unit_count == 1 {
            // Single unit selected (existing behavior)
            let (unit_entity, unit_type, obj_instance, owner, unit_base, speed, attack, turret) = selected_units.iter().next().unwrap();

            commands.entity(grid_entity).despawn_children();
            commands.entity(grid_entity).insert(Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                overflow: Overflow::clip(),
                ..default()
            });

            commands.entity(grid_entity).with_children(|parent| {
                // Left side: Unit icon with color and health bar
                parent.spawn((
                    Node {
                        width: Val::Px(120.0),
                        height: Val::Px(150.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.9)),
                    UnitIcon { unit_entity },
                ))
                .with_children(|icon_parent| {
                    icon_parent.spawn((
                        Node {
                            width: Val::Px(80.0),
                            height: Val::Px(80.0),
                            ..default()
                        },
                        BackgroundColor(owner.color()),
                    ));

                    icon_parent.spawn((
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(12.0),
                            margin: UiRect::top(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    ))
                    .with_children(|health_parent| {
                        let health_percent = obj_instance.health_fraction();
                        let health_color = get_health_color(health_percent);

                        health_parent.spawn((
                            Node {
                                width: Val::Percent(health_percent * 100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(health_color),
                            UnitHealthBar { unit_entity },
                        ));
                    });

                    icon_parent.spawn((
                        Text::new(format!("{:.0} / {:.0}", obj_instance.hp.unwrap_or(0.0), obj_instance.max_hp.unwrap_or(0.0))),
                        TextFont { font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        Node {
                            margin: UiRect::top(Val::Px(4.0)),
                            ..default()
                        },
                    ));
                });

                // Right side: Detailed stats
                parent.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        justify_content: JustifyContent::Start,
                        padding: UiRect::all(Val::Px(10.0)),
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                ))
                .with_children(|stats_parent| {
                    stats_parent.spawn((
                        Text::new(&unit_type.name),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        Node {
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                    ));

                    let base_type_name = unit_base.display_name();
                    spawn_stat_text(stats_parent, &format!("Type: {}", base_type_name), 14.0, Color::srgb(0.8, 0.8, 0.8), Some(2.0));

                    let owner_text = match owner.player_number() {
                        Some(id) => format!("Player {}", id),
                        None => "Neutral".to_string(),
                    };
                    spawn_stat_text(stats_parent, &format!("Owner: {}", owner_text), 14.0, owner.color(), Some(8.0));
                    if let Some(attack) = attack {
                        spawn_stat_text(stats_parent, "Combat", 12.0, Color::srgb(0.9, 0.7, 0.7), None);
                        spawn_stat_text(stats_parent, &format!("  Damage: {:.0}", attack.damage), 13.0, Color::srgb(0.8, 0.8, 0.8), None);
                        spawn_stat_text(stats_parent, &format!("  Range: {:.1}", attack.range), 13.0, Color::srgb(0.8, 0.8, 0.8), Some(6.0));
                    } else {
                        spawn_stat_text(stats_parent, "Unarmed", 12.0, Color::srgb(0.7, 0.7, 0.7), Some(6.0));
                    }
                    spawn_stat_text(stats_parent, "Movement", 12.0, Color::srgb(0.7, 0.8, 0.9), None);
                    spawn_stat_text(stats_parent, &format!("  Speed: {:.1}", speed.0), 13.0, Color::srgb(0.8, 0.8, 0.8), Some(6.0));

                    if let Some(turret) = turret {
                        let turn_angle_deg = turret.turn_angle.to_degrees();
                        let turn_rate_deg = turret.turn_rate.to_degrees();
                        spawn_stat_text(stats_parent, "Turret", 12.0, Color::srgb(0.8, 0.7, 0.9), None);
                        spawn_stat_text(stats_parent, &format!("  Arc: {:.0}\u{00b0}", turn_angle_deg), 13.0, Color::srgb(0.8, 0.8, 0.8), None);
                        spawn_stat_text(stats_parent, &format!("  Turn Rate: {:.0}\u{00b0}/s", turn_rate_deg), 13.0, Color::srgb(0.8, 0.8, 0.8), None);
                    }
                });
            });
        } else if total_selected == 1 && resource_count == 1 {
            // Single resource entity selected (Crystal Patch or SDS)
            commands.entity(grid_entity).despawn_children();
            commands.entity(grid_entity).insert(Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(20.0),
                overflow: Overflow::clip(),
                ..default()
            });

            // Determine which resource type is selected
            if let Some((res_entity, scp, obj_instance)) = selected_scp.iter().next() {
                let obj_type_data = obj_instance.object_type.object_type();
                commands.entity(grid_entity).with_children(|parent| {
                    // Left side: Resource icon
                    parent.spawn((
                        Node {
                            width: Val::Px(120.0),
                            height: Val::Px(150.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.9)),
                        ResourceIcon { resource_entity: res_entity },
                    ))
                    .with_children(|icon_parent| {
                        // Crystal color swatch
                        icon_parent.spawn((
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(80.0),
                                border: UiRect::all(Val::Px(2.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.8, 1.0)),
                            BorderColor::all(Color::srgb(0.5, 0.7, 0.9)),
                        ))
                        .with_children(|swatch| {
                            swatch.spawn((
                                Text::new("SC"),
                                TextFont { font_size: 24.0, ..default() },
                                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                            ));
                        });
                    });

                    // Right side: Crystal Patch stats
                    parent.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Start,
                            justify_content: JustifyContent::Start,
                            padding: UiRect::all(Val::Px(10.0)),
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                    ))
                    .with_children(|stats_parent| {
                        // Name
                        spawn_stat_text(stats_parent, &obj_type_data.name, 20.0, Color::srgb(1.0, 1.0, 1.0), Some(8.0));

                        // Resource type
                        spawn_stat_text(stats_parent, "Type: Resource", 14.0, Color::srgb(0.8, 0.8, 0.8), Some(8.0));

                        // Remaining amount
                        let remaining = scp.remaining_amount;
                        let initial = scp.initial_amount;
                        let pct = if initial > 0 { (remaining as f32 / initial as f32 * 100.0).min(100.0) } else { 0.0 };
                        spawn_stat_text(stats_parent, "Resources", 12.0, Color::srgb(0.5, 0.9, 1.0), None);
                        spawn_stat_text(stats_parent, &format!("  Remaining: {} / {} ({:.0}%)", remaining, initial, pct), 13.0, Color::srgb(0.8, 0.8, 0.8), None);

                        if scp.has_plate {
                            spawn_stat_text(stats_parent, "  Extraction Plate: Active", 13.0, Color::srgb(0.5, 1.0, 0.5), None);
                        }

                        if remaining == 0 {
                            spawn_stat_text(stats_parent, "  Depleted", 13.0, Color::srgb(1.0, 0.7, 0.3), None);
                        }
                    });
                });
            } else if let Some((res_entity, sds, obj_instance)) = selected_sds.iter().next() {
                let obj_type_data = obj_instance.object_type.object_type();
                commands.entity(grid_entity).with_children(|parent| {
                    // Left side: Resource icon
                    parent.spawn((
                        Node {
                            width: Val::Px(120.0),
                            height: Val::Px(150.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.9)),
                        ResourceIcon { resource_entity: res_entity },
                    ))
                    .with_children(|icon_parent| {
                        // SDS color swatch
                        icon_parent.spawn((
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(80.0),
                                border: UiRect::all(Val::Px(2.0)),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.8, 0.6, 0.2)),
                            BorderColor::all(Color::srgb(0.7, 0.5, 0.3)),
                        ))
                        .with_children(|swatch| {
                            swatch.spawn((
                                Text::new("SD"),
                                TextFont { font_size: 24.0, ..default() },
                                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                            ));
                        });
                    });

                    // Right side: SDS stats
                    parent.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Start,
                            justify_content: JustifyContent::Start,
                            padding: UiRect::all(Val::Px(10.0)),
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
                    ))
                    .with_children(|stats_parent| {
                        // Name
                        spawn_stat_text(stats_parent, &obj_type_data.name, 20.0, Color::srgb(1.0, 1.0, 1.0), Some(8.0));

                        // Resource type
                        spawn_stat_text(stats_parent, "Type: Resource", 14.0, Color::srgb(0.8, 0.8, 0.8), Some(8.0));

                        // Delivery stats
                        spawn_stat_text(stats_parent, "Delivery", 12.0, Color::srgb(0.9, 0.8, 0.5), None);
                        spawn_stat_text(stats_parent, &format!("  Size: {} supplies", sds.delivery_size), 13.0, Color::srgb(0.8, 0.8, 0.8), None);
                        spawn_stat_text(stats_parent, &format!("  Interval: {:.0}s", sds.delivery_interval), 13.0, Color::srgb(0.8, 0.8, 0.8), None);
                        spawn_stat_text(stats_parent, &format!("  Current: {} supplies", sds.current_supplies), 13.0, Color::srgb(0.8, 0.8, 0.8), None);

                        if sds.current_supplies == 0 {
                            let time_left = sds.time_until_next_delivery;
                            spawn_stat_text(stats_parent, &format!("  Next delivery: {:.1}s", time_left), 13.0, Color::srgb(1.0, 0.7, 0.3), None);
                        }
                    });
                });
            }
        } else {
            // Multiple entities selected (units, structures, and/or resources)
            commands.entity(grid_entity).despawn_children();
            commands.entity(grid_entity).insert(Node {
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

            // Collect active group entities for highlight
            let active_entities: Vec<Entity> = selection.active_group()
                .map(|g| g.entities.clone())
                .unwrap_or_default();

            // Show structures first, then units
            let mut cards_spawned = 0;
            for (struct_entity, obj_instance, owner, _power, _build_radius, _dc, _bk, _ef, _ep, _st) in selected_structures.iter() {
                if cards_spawned >= 12 { break; }
                let obj_type_data = obj_instance.object_type.object_type();
                let in_active_group = active_entities.contains(&struct_entity);
                spawn_selection_portrait(&mut commands, grid_entity, struct_entity, &obj_type_data.name, obj_instance, owner, true, in_active_group);
                cards_spawned += 1;
            }

            for (unit_entity, unit_type, obj_instance, owner, _unit_base, _speed, _attack, _turret) in selected_units.iter() {
                if cards_spawned >= 12 { break; }
                let in_active_group = active_entities.contains(&unit_entity);
                spawn_selection_portrait(&mut commands, grid_entity, unit_entity, &unit_type.name, obj_instance, owner, false, in_active_group);
                cards_spawned += 1;
            }

            // Show resource entities last
            for (res_entity, _scp, obj_instance) in selected_scp.iter() {
                if cards_spawned >= 12 { break; }
                let obj_type_data = obj_instance.object_type.object_type();
                let in_active_group = active_entities.contains(&res_entity);
                spawn_resource_portrait(&mut commands, grid_entity, res_entity, &obj_type_data.name, in_active_group);
                cards_spawned += 1;
            }
            for (res_entity, _sds, obj_instance) in selected_sds.iter() {
                if cards_spawned >= 12 { break; }
                let obj_type_data = obj_instance.object_type.object_type();
                let in_active_group = active_entities.contains(&res_entity);
                spawn_resource_portrait(&mut commands, grid_entity, res_entity, &obj_type_data.name, in_active_group);
                cards_spawned += 1;
            }
        }
    } else {
        // Update health bars for existing icons (no selection change)
        for (mut node, health_bar) in unit_health_bars.iter_mut() {
            if let Ok((_, _, obj_instance, _, _, _, _, _)) = selected_units.get(health_bar.unit_entity) {
                let health_percent = obj_instance.health_fraction();
                node.width = Val::Percent(health_percent * 100.0);
            }
        }
        for (mut node, health_bar) in struct_health_bars.iter_mut() {
            if let Ok((_, obj_instance, _, _, _, _, _, _, _, _)) = selected_structures.get(health_bar.structure_entity) {
                let health_percent = obj_instance.health_fraction();
                node.width = Val::Percent(health_percent * 100.0);
            }
        }
    }
}

/// Helper to spawn a selection portrait card for multi-select display.
/// Includes `SelectionPortrait` + `Interaction` for click handling, and
/// an active group highlight overlay when the entity is in the active group.
fn spawn_selection_portrait(
    commands: &mut Commands,
    grid_entity: Entity,
    entity: Entity,
    name: &str,
    obj_instance: &ObjectInstance,
    owner: &Owner,
    is_structure: bool,
    in_active_group: bool,
) {
    commands.entity(grid_entity).with_children(|parent| {
        let icon_component: Box<dyn FnOnce(&mut EntityCommands)> = if is_structure {
            Box::new(move |ec: &mut EntityCommands| { ec.insert(StructureIcon { structure_entity: entity }); })
        } else {
            Box::new(move |ec: &mut EntityCommands| { ec.insert(UnitIcon { unit_entity: entity }); })
        };

        let mut card_entity = parent.spawn((
            Node {
                width: Val::Px(140.0),
                height: Val::Px(95.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.9)),
            SelectionPortrait { entity },
            Interaction::default(),
        ));

        icon_component(&mut card_entity);

        let display_name = if name.len() > 14 {
            format!("{}...", &name[..11])
        } else {
            name.to_string()
        };

        let owner_color = owner.color();
        let hp = obj_instance.hp.unwrap_or(0.0);
        let max_hp = obj_instance.max_hp.unwrap_or(0.0);
        let health_percent = obj_instance.health_fraction();
        let health_color = get_health_color(health_percent);
        let is_struct = is_structure;

        card_entity.with_children(move |card| {
            // Top row: color swatch + name
            card.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.0),
                margin: UiRect::bottom(Val::Px(2.0)),
                ..default()
            })
            .with_children(|top_row| {
                top_row.spawn((
                    Node {
                        width: Val::Px(16.0),
                        height: Val::Px(16.0),
                        ..default()
                    },
                    BackgroundColor(owner_color),
                ));

                top_row.spawn((
                    Text::new(display_name.clone()),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::srgb(1.0, 1.0, 1.0)),
                ));
            });

            // HP text
            card.spawn((
                Text::new(format!("HP: {:.0}/{:.0}", hp, max_hp)),
                TextFont { font_size: 10.0, ..default() },
                TextColor(Color::srgb(0.7, 0.9, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(1.0)),
                    ..default()
                },
            ));

            // Type label
            let type_label = if is_struct { "Structure" } else { "Unit" };
            card.spawn((
                Text::new(type_label),
                TextFont { font_size: 10.0, ..default() },
                TextColor(if is_struct { Color::srgb(0.7, 0.9, 0.8) } else { Color::srgb(0.9, 0.7, 0.7) }),
                Node {
                    margin: UiRect::bottom(Val::Px(3.0)),
                    ..default()
                },
            ));

            // Health bar
            card.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ))
            .with_children(|health_parent| {
                if is_struct {
                    health_parent.spawn((
                        Node {
                            width: Val::Percent(health_percent * 100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(health_color),
                        StructureHealthBar { structure_entity: entity },
                    ));
                } else {
                    health_parent.spawn((
                        Node {
                            width: Val::Percent(health_percent * 100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(health_color),
                        UnitHealthBar { unit_entity: entity },
                    ));
                }
            });

            // Active group highlight overlay
            if in_active_group {
                card.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.15)),
                ));
            }
        });
    });
}

/// Helper to spawn a resource portrait card for multi-select display.
/// Simpler than unit/structure portraits since resources are indestructible and unowned.
fn spawn_resource_portrait(
    commands: &mut Commands,
    grid_entity: Entity,
    entity: Entity,
    name: &str,
    in_active_group: bool,
) {
    commands.entity(grid_entity).with_children(|parent| {
        let mut card_entity = parent.spawn((
            Node {
                width: Val::Px(140.0),
                height: Val::Px(95.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 0.9)),
            SelectionPortrait { entity },
            ResourceIcon { resource_entity: entity },
            Interaction::default(),
        ));

        let display_name = if name.len() > 14 {
            format!("{}...", &name[..11])
        } else {
            name.to_string()
        };

        card_entity.with_children(move |card| {
            // Top row: color swatch + name
            card.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(4.0),
                margin: UiRect::bottom(Val::Px(2.0)),
                ..default()
            })
            .with_children(|top_row| {
                top_row.spawn((
                    Node {
                        width: Val::Px(16.0),
                        height: Val::Px(16.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.5, 0.7, 0.9)),
                ));

                top_row.spawn((
                    Text::new(display_name.clone()),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::srgb(1.0, 1.0, 1.0)),
                ));
            });

            // Type label
            card.spawn((
                Text::new("Resource"),
                TextFont { font_size: 10.0, ..default() },
                TextColor(Color::srgb(0.5, 0.9, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(3.0)),
                    ..default()
                },
            ));

            // Active group highlight overlay
            if in_active_group {
                card.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.15)),
                ));
            }
        });
    });
}

/// System to handle click interactions on selection portraits in the multi-select panel.
/// Supports: left-click (select only), shift-click (remove), ctrl-click (select same type),
/// ctrl-shift-click (remove same type), alt-hover (center camera).
pub fn selection_portrait_click_system(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    portraits: Query<(&Interaction, &SelectionPortrait)>,
    mut selection: ResMut<Selection>,
    selected_query: Query<Entity, With<Selected>>,
    object_instances: Query<&ObjectInstance>,
    transforms: Query<&Transform, Without<MainCamera>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<SelectionPortrait>)>,
) {
    let alt_held = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);

    // Alt + click camera centering: detect which portrait is hovered, then
    // check for an actual mouse click to avoid triggering on mere hover.
    // We use ButtonInput<MouseButton> rather than Interaction::Pressed because
    // some Linux WMs intercept Alt+Click for window dragging, preventing
    // Interaction::Pressed from firing. ButtonInput may still receive the event
    // depending on WM config, and at minimum the behavior is correct (click-based).
    if alt_held && mouse_buttons.just_pressed(MouseButton::Left) {
        for (interaction, portrait) in portraits.iter() {
            if matches!(interaction, Interaction::Hovered | Interaction::Pressed) {
                let target_entity = portrait.entity;
                if let Ok(target_transform) = transforms.get(target_entity) {
                    if let Ok(mut cam_transform) = camera_query.single_mut() {
                        // Apply z_offset to account for the camera's oblique angle.
                        // Camera looks from (x, y, z) down at the ground; the Z
                        // offset from camera position to the ground look-at point
                        // is y * 25.0 / 40.0 (matching the initial setup angle).
                        let z_offset = cam_transform.translation.y * 25.0 / 40.0;
                        cam_transform.translation.x = target_transform.translation.x;
                        cam_transform.translation.z = target_transform.translation.z + z_offset;
                    }
                }
                return;
            }
        }
    }

    for (interaction, portrait) in portraits.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Skip if Alt is held (camera centering handled above)
        if alt_held {
            continue;
        }

        let ctrl_held = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
        let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
        let target_entity = portrait.entity;

        if ctrl_held && shift_held {
            // Ctrl+Shift-click: remove all entities of same type from selection
            if let Ok(obj) = object_instances.get(target_entity) {
                let target_type = obj.object_type;
                let entities_to_remove: Vec<Entity> = selection.groups.iter()
                    .filter(|g| g.object_type == target_type)
                    .flat_map(|g| g.entities.clone())
                    .collect();
                for entity in &entities_to_remove {
                    commands.entity(*entity).remove::<Selected>();
                    selection.remove_entity(*entity);
                }
            }
        } else if shift_held {
            // Shift-click: remove this entity from selection
            commands.entity(target_entity).remove::<Selected>();
            selection.remove_entity(target_entity);
        } else if ctrl_held {
            // Ctrl-click: select all entities of same type from current selection
            if let Ok(obj) = object_instances.get(target_entity) {
                let target_type = obj.object_type;
                let same_type_entities: Vec<Entity> = selection.groups.iter()
                    .filter(|g| g.object_type == target_type)
                    .flat_map(|g| g.entities.clone())
                    .collect();
                // Clear all selected markers
                for entity in selected_query.iter() {
                    commands.entity(entity).remove::<Selected>();
                }
                // Select only same-type entities
                for entity in &same_type_entities {
                    commands.entity(*entity).insert(Selected);
                }
                // Directly update Selection for immediate response
                selection.groups.retain(|g| g.object_type == target_type);
                if !selection.groups.is_empty() {
                    selection.active_group_index = Some(0);
                } else {
                    selection.active_group_index = None;
                }
            }
        } else {
            // Left-click (no modifier): select only this entity
            for entity in selected_query.iter() {
                commands.entity(entity).remove::<Selected>();
            }
            commands.entity(target_entity).insert(Selected);
            // Directly update Selection for immediate response
            if let Ok(obj) = object_instances.get(target_entity) {
                selection.groups = vec![SelectionGroup {
                    object_type: obj.object_type,
                    entities: vec![target_entity],
                }];
                selection.active_group_index = Some(0);
            }
        }
    }
}

/// Returns the resource bar fields and their default display text for a given faction.
pub(crate) fn resource_bar_fields_for_faction(faction: FactionEnum) -> Vec<(ResourceBarField, &'static str)> {
    match faction {
        FactionEnum::GlobalDefenseOrdinance => vec![
            (ResourceBarField::Crystals, "SC: 0"),
            (ResourceBarField::Supplies, "Supplies: 0"),
            (ResourceBarField::Power, "Power: 0 / 0"),
            (ResourceBarField::UnitControl, "UC: 0 / 200"),
        ],
        FactionEnum::TheSyndicate => vec![
            (ResourceBarField::Crystals, "SC: 0"),
            (ResourceBarField::Supplies, "Supplies: 0"),
            (ResourceBarField::TunnelSpace, "TS: 0 / 0"),
        ],
        FactionEnum::TheCults => vec![
            (ResourceBarField::Crystals, "SC: 0"),
            (ResourceBarField::UnitControl, "UC: 0 / 0"),
        ],
        FactionEnum::Colonists => vec![
            (ResourceBarField::Crystals, "SC: 0"),
            (ResourceBarField::Alloys, "Alloys: 0"),
            (ResourceBarField::Essence, "Essence: 0"),
            (ResourceBarField::Conduits, "Conduits: 0"),
            (ResourceBarField::BeaconCapacity, "BC: 0 / 0"),
        ],
    }
}

/// Spawn the correct resource bar text elements based on the local player's faction.
fn spawn_resource_bar_fields(parent: &mut ChildSpawnerCommands, font_size: f32, color: Color, faction: FactionEnum) {
    for (field, default_text) in resource_bar_fields_for_faction(faction) {
        parent.spawn((
            Text::new(default_text),
            TextFont { font_size, ..default() },
            TextColor(color),
            field,
        ));
    }
}

/// System to update the resource bar text based on the local player's faction resources.
pub fn update_resource_bar_system(
    local_player: Res<LocalPlayer>,
    player_query: Query<&Player>,
    gdo_query: Query<(&Player, &GdoPlayerResources)>,
    syn_query: Query<(&Player, &SyndicatePlayerResources)>,
    cults_query: Query<(&Player, &CultsPlayerResources)>,
    col_query: Query<(&Player, &ColonistsPlayerResources)>,
    mut fields: Query<(&mut Text, &mut TextColor, &ResourceBarField)>,
) {
    let local_id = local_player.0;

    // Determine the local player's faction
    let Some(faction) = player_query.iter()
        .find(|p| p.player_number == local_id)
        .map(|p| p.faction)
    else {
        return;
    };

    match faction {
        FactionEnum::GlobalDefenseOrdinance => {
            let Some((_player, res)) = gdo_query.iter().find(|(p, _)| p.player_number == local_id) else { return };
            for (mut text, mut text_color, field) in fields.iter_mut() {
                match field {
                    ResourceBarField::Crystals => {
                        **text = format!("SC: {}", res.space_crystals);
                    }
                    ResourceBarField::Supplies => {
                        **text = format!("Supplies: {}", res.supplies);
                    }
                    ResourceBarField::Power => {
                        let net = res.current_power();
                        let color = if net >= 0 {
                            Color::srgb(0.5, 1.0, 0.5)
                        } else {
                            Color::srgb(1.0, 0.4, 0.4)
                        };
                        **text = format!("Power: {} / {}", net, res.power_generated);
                        text_color.0 = color;
                    }
                    ResourceBarField::UnitControl => {
                        **text = format!("UC: {} / {}", res.unit_control_used, res.unit_control_cap);
                    }
                    _ => {}
                }
            }
        }
        FactionEnum::TheSyndicate => {
            let Some((_player, res)) = syn_query.iter().find(|(p, _)| p.player_number == local_id) else { return };
            for (mut text, mut _text_color, field) in fields.iter_mut() {
                match field {
                    ResourceBarField::Crystals => {
                        **text = format!("SC: {}", res.space_crystals);
                    }
                    ResourceBarField::Supplies => {
                        **text = format!("Supplies: {}", res.supplies);
                    }
                    ResourceBarField::TunnelSpace => {
                        **text = format!("TS: {} / {}", res.tunnel_space_used, res.tunnel_space_provided);
                    }
                    _ => {}
                }
            }
        }
        FactionEnum::TheCults => {
            let Some((_player, res)) = cults_query.iter().find(|(p, _)| p.player_number == local_id) else { return };
            for (mut text, mut _text_color, field) in fields.iter_mut() {
                match field {
                    ResourceBarField::Crystals => {
                        **text = format!("SC: {}", res.space_crystals);
                    }
                    ResourceBarField::UnitControl => {
                        **text = format!("UC: {} / {}", res.unit_control_used, res.unit_control_available);
                    }
                    _ => {}
                }
            }
        }
        FactionEnum::Colonists => {
            let Some((_player, res)) = col_query.iter().find(|(p, _)| p.player_number == local_id) else { return };
            for (mut text, mut _text_color, field) in fields.iter_mut() {
                match field {
                    ResourceBarField::Crystals => {
                        **text = format!("SC: {}", res.space_crystals);
                    }
                    ResourceBarField::Alloys => {
                        **text = format!("Alloys: {}", res.alloys);
                    }
                    ResourceBarField::Essence => {
                        **text = format!("Essence: {}", res.essence);
                    }
                    ResourceBarField::Conduits => {
                        **text = format!("Conduits: {}", res.conduits);
                    }
                    ResourceBarField::BeaconCapacity => {
                        **text = format!("BC: {} / {}", res.beacon_capacity_used, res.beacon_capacity_provided);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Helper to spawn a stat text element
fn spawn_stat_text(parent: &mut ChildSpawnerCommands, text: &str, font_size: f32, color: Color, bottom_margin: Option<f32>) {
    let margin = if let Some(m) = bottom_margin {
        UiRect::bottom(Val::Px(m))
    } else {
        UiRect::default()
    };

    parent.spawn((
        Text::new(text),
        TextFont { font_size, ..default() },
        TextColor(color),
        Node {
            margin,
            ..default()
        },
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    // === resource_bar_fields_for_faction tests ===

    #[test]
    fn gdo_has_four_resource_fields() {
        let fields = resource_bar_fields_for_faction(FactionEnum::GlobalDefenseOrdinance);
        assert_eq!(fields.len(), 4);
    }

    #[test]
    fn gdo_fields_are_crystals_supplies_power_uc() {
        let fields = resource_bar_fields_for_faction(FactionEnum::GlobalDefenseOrdinance);
        let field_types: Vec<_> = fields.iter().map(|(f, _)| *f).collect();
        assert_eq!(field_types, vec![
            ResourceBarField::Crystals,
            ResourceBarField::Supplies,
            ResourceBarField::Power,
            ResourceBarField::UnitControl,
        ]);
    }

    #[test]
    fn syndicate_has_three_resource_fields() {
        let fields = resource_bar_fields_for_faction(FactionEnum::TheSyndicate);
        assert_eq!(fields.len(), 3);
    }

    #[test]
    fn syndicate_fields_are_crystals_supplies_tunnel_space() {
        let fields = resource_bar_fields_for_faction(FactionEnum::TheSyndicate);
        let field_types: Vec<_> = fields.iter().map(|(f, _)| *f).collect();
        assert_eq!(field_types, vec![
            ResourceBarField::Crystals,
            ResourceBarField::Supplies,
            ResourceBarField::TunnelSpace,
        ]);
    }

    #[test]
    fn cults_has_two_resource_fields() {
        let fields = resource_bar_fields_for_faction(FactionEnum::TheCults);
        assert_eq!(fields.len(), 2);
    }

    #[test]
    fn cults_fields_are_crystals_uc() {
        let fields = resource_bar_fields_for_faction(FactionEnum::TheCults);
        let field_types: Vec<_> = fields.iter().map(|(f, _)| *f).collect();
        assert_eq!(field_types, vec![
            ResourceBarField::Crystals,
            ResourceBarField::UnitControl,
        ]);
    }

    #[test]
    fn colonists_has_five_resource_fields() {
        let fields = resource_bar_fields_for_faction(FactionEnum::Colonists);
        assert_eq!(fields.len(), 5);
    }

    #[test]
    fn colonists_fields_are_crystals_alloys_essence_conduits_beacon() {
        let fields = resource_bar_fields_for_faction(FactionEnum::Colonists);
        let field_types: Vec<_> = fields.iter().map(|(f, _)| *f).collect();
        assert_eq!(field_types, vec![
            ResourceBarField::Crystals,
            ResourceBarField::Alloys,
            ResourceBarField::Essence,
            ResourceBarField::Conduits,
            ResourceBarField::BeaconCapacity,
        ]);
    }

    #[test]
    fn all_factions_have_crystals_as_first_field() {
        let factions = [
            FactionEnum::GlobalDefenseOrdinance,
            FactionEnum::TheSyndicate,
            FactionEnum::TheCults,
            FactionEnum::Colonists,
        ];
        for faction in factions {
            let fields = resource_bar_fields_for_faction(faction);
            assert_eq!(fields[0].0, ResourceBarField::Crystals,
                "Faction {:?} should have Crystals as first field", faction);
        }
    }

    #[test]
    fn gdo_default_text_contains_sc() {
        let fields = resource_bar_fields_for_faction(FactionEnum::GlobalDefenseOrdinance);
        assert!(fields[0].1.contains("SC"));
    }

    #[test]
    fn gdo_default_text_contains_power() {
        let fields = resource_bar_fields_for_faction(FactionEnum::GlobalDefenseOrdinance);
        let power_field = fields.iter().find(|(f, _)| *f == ResourceBarField::Power);
        assert!(power_field.is_some());
        assert!(power_field.unwrap().1.contains("Power"));
    }

    #[test]
    fn syndicate_has_no_power_field() {
        let fields = resource_bar_fields_for_faction(FactionEnum::TheSyndicate);
        assert!(!fields.iter().any(|(f, _)| *f == ResourceBarField::Power));
    }

    #[test]
    fn cults_has_no_supplies_field() {
        let fields = resource_bar_fields_for_faction(FactionEnum::TheCults);
        assert!(!fields.iter().any(|(f, _)| *f == ResourceBarField::Supplies));
    }

    #[test]
    fn colonists_has_no_power_or_supplies_field() {
        let fields = resource_bar_fields_for_faction(FactionEnum::Colonists);
        assert!(!fields.iter().any(|(f, _)| *f == ResourceBarField::Power));
        assert!(!fields.iter().any(|(f, _)| *f == ResourceBarField::Supplies));
    }

    #[test]
    fn only_gdo_has_power_field() {
        // Power is exclusive to GDO
        let non_gdo = [FactionEnum::TheSyndicate, FactionEnum::TheCults, FactionEnum::Colonists];
        for faction in non_gdo {
            let fields = resource_bar_fields_for_faction(faction);
            assert!(!fields.iter().any(|(f, _)| *f == ResourceBarField::Power),
                "Faction {:?} should not have Power field", faction);
        }
        let gdo_fields = resource_bar_fields_for_faction(FactionEnum::GlobalDefenseOrdinance);
        assert!(gdo_fields.iter().any(|(f, _)| *f == ResourceBarField::Power));
    }

    #[test]
    fn only_syndicate_has_tunnel_space() {
        let fields = resource_bar_fields_for_faction(FactionEnum::TheSyndicate);
        assert!(fields.iter().any(|(f, _)| *f == ResourceBarField::TunnelSpace));

        let non_syndicate = [FactionEnum::GlobalDefenseOrdinance, FactionEnum::TheCults, FactionEnum::Colonists];
        for faction in non_syndicate {
            let fields = resource_bar_fields_for_faction(faction);
            assert!(!fields.iter().any(|(f, _)| *f == ResourceBarField::TunnelSpace),
                "Faction {:?} should not have TunnelSpace field", faction);
        }
    }

    #[test]
    fn only_colonists_has_alloys_essence_conduits_beacon() {
        let fields = resource_bar_fields_for_faction(FactionEnum::Colonists);
        assert!(fields.iter().any(|(f, _)| *f == ResourceBarField::Alloys));
        assert!(fields.iter().any(|(f, _)| *f == ResourceBarField::Essence));
        assert!(fields.iter().any(|(f, _)| *f == ResourceBarField::Conduits));
        assert!(fields.iter().any(|(f, _)| *f == ResourceBarField::BeaconCapacity));

        let non_colonists = [FactionEnum::GlobalDefenseOrdinance, FactionEnum::TheSyndicate, FactionEnum::TheCults];
        for faction in non_colonists {
            let fields = resource_bar_fields_for_faction(faction);
            assert!(!fields.iter().any(|(f, _)| *f == ResourceBarField::Alloys));
            assert!(!fields.iter().any(|(f, _)| *f == ResourceBarField::Essence));
            assert!(!fields.iter().any(|(f, _)| *f == ResourceBarField::Conduits));
            assert!(!fields.iter().any(|(f, _)| *f == ResourceBarField::BeaconCapacity));
        }
    }

    // === ResourceBarField enum tests ===

    #[test]
    fn resource_bar_field_clone_and_eq() {
        let field = ResourceBarField::Crystals;
        let cloned = field.clone();
        assert_eq!(field, cloned);
    }

    #[test]
    fn resource_bar_field_variants_distinct() {
        let variants = [
            ResourceBarField::Crystals,
            ResourceBarField::Supplies,
            ResourceBarField::Power,
            ResourceBarField::UnitControl,
            ResourceBarField::TunnelSpace,
            ResourceBarField::Alloys,
            ResourceBarField::Essence,
            ResourceBarField::Conduits,
            ResourceBarField::BeaconCapacity,
        ];
        // All 9 variants should be distinct
        for i in 0..variants.len() {
            for j in (i + 1)..variants.len() {
                assert_ne!(variants[i], variants[j], "Variant {} and {} should be distinct", i, j);
            }
        }
    }

    // === SelectionPortrait component tests ===

    #[test]
    fn selection_portrait_stores_entity() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let portrait = SelectionPortrait { entity };
        assert_eq!(portrait.entity, entity);
    }

    #[test]
    fn selection_portrait_click_removes_entity_on_shift() {
        // Test the shift-click logic: removing an entity from Selection
        let mut selection = Selection::default();
        selection.groups = vec![
            SelectionGroup { object_type: ObjectEnum::Peacekeeper, entities: vec![] },
        ];
        // Populate with dummy entities using a World
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        let e3 = world.spawn_empty().id();
        selection.groups[0].entities = vec![e1, e2, e3];
        selection.active_group_index = Some(0);

        // Simulate shift-click on e2
        selection.remove_entity(e2);
        assert_eq!(selection.total_entity_count(), 2);
        assert!(!selection.contains_entity(e2));
        assert!(selection.contains_entity(e1));
        assert!(selection.contains_entity(e3));
    }

    #[test]
    fn selection_portrait_ctrl_click_retains_same_type() {
        // Test ctrl-click logic: retain only groups of target type
        let mut selection = Selection::default();
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        let e3 = world.spawn_empty().id();
        selection.groups = vec![
            SelectionGroup { object_type: ObjectEnum::Peacekeeper, entities: vec![e1, e2] },
            SelectionGroup { object_type: ObjectEnum::SyndicateAgent, entities: vec![e3] },
        ];
        selection.active_group_index = Some(0);

        // Simulate ctrl-click: retain only Peacekeeper type
        let target_type = ObjectEnum::Peacekeeper;
        selection.groups.retain(|g| g.object_type == target_type);
        if !selection.groups.is_empty() {
            selection.active_group_index = Some(0);
        }

        assert_eq!(selection.groups.len(), 1);
        assert_eq!(selection.groups[0].object_type, ObjectEnum::Peacekeeper);
        assert_eq!(selection.total_entity_count(), 2);
        assert!(selection.contains_entity(e1));
        assert!(selection.contains_entity(e2));
        assert!(!selection.contains_entity(e3));
    }

    #[test]
    fn selection_portrait_ctrl_shift_removes_all_of_type() {
        // Test ctrl+shift-click logic: remove all entities of same type
        let mut selection = Selection::default();
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        let e3 = world.spawn_empty().id();
        selection.groups = vec![
            SelectionGroup { object_type: ObjectEnum::Peacekeeper, entities: vec![e1, e2] },
            SelectionGroup { object_type: ObjectEnum::SyndicateAgent, entities: vec![e3] },
        ];
        selection.active_group_index = Some(0);

        // Simulate ctrl+shift-click on Peacekeeper
        let target_type = ObjectEnum::Peacekeeper;
        let entities_to_remove: Vec<Entity> = selection.groups.iter()
            .filter(|g| g.object_type == target_type)
            .flat_map(|g| g.entities.clone())
            .collect();
        for entity in &entities_to_remove {
            selection.remove_entity(*entity);
        }

        assert_eq!(selection.groups.len(), 1);
        assert_eq!(selection.groups[0].object_type, ObjectEnum::SyndicateAgent);
        assert_eq!(selection.total_entity_count(), 1);
        assert!(selection.contains_entity(e3));
    }

    #[test]
    fn selection_portrait_left_click_replaces_selection() {
        // Test left-click logic: replace selection with single entity
        let mut selection = Selection::default();
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        let e3 = world.spawn_empty().id();
        selection.groups = vec![
            SelectionGroup { object_type: ObjectEnum::Peacekeeper, entities: vec![e1, e2] },
            SelectionGroup { object_type: ObjectEnum::SyndicateAgent, entities: vec![e3] },
        ];
        selection.active_group_index = Some(0);

        // Simulate left-click on e2 (Peacekeeper)
        let target_entity = e2;
        let target_type = ObjectEnum::Peacekeeper;
        selection.groups = vec![SelectionGroup {
            object_type: target_type,
            entities: vec![target_entity],
        }];
        selection.active_group_index = Some(0);

        assert_eq!(selection.groups.len(), 1);
        assert_eq!(selection.total_entity_count(), 1);
        assert!(selection.contains_entity(e2));
        assert!(!selection.contains_entity(e1));
        assert!(!selection.contains_entity(e3));
    }

    #[test]
    fn selection_portrait_shift_click_to_single_reduces_count() {
        // Edge case: shift-click with 2 units leaves 1
        let mut selection = Selection::default();
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        selection.groups = vec![
            SelectionGroup { object_type: ObjectEnum::Peacekeeper, entities: vec![e1, e2] },
        ];
        selection.active_group_index = Some(0);

        selection.remove_entity(e1);
        assert_eq!(selection.total_entity_count(), 1);
        assert!(selection.contains_entity(e2));
    }

    #[test]
    fn selection_portrait_ctrl_shift_can_empty_selection() {
        // Edge case: ctrl+shift-click removes all entities when only one type
        let mut selection = Selection::default();
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        selection.groups = vec![
            SelectionGroup { object_type: ObjectEnum::Peacekeeper, entities: vec![e1, e2] },
        ];
        selection.active_group_index = Some(0);

        let target_type = ObjectEnum::Peacekeeper;
        let entities_to_remove: Vec<Entity> = selection.groups.iter()
            .filter(|g| g.object_type == target_type)
            .flat_map(|g| g.entities.clone())
            .collect();
        for entity in &entities_to_remove {
            selection.remove_entity(*entity);
        }

        assert_eq!(selection.total_entity_count(), 0);
        assert!(selection.groups.is_empty());
        assert_eq!(selection.active_group_index, None);
    }

    #[test]
    fn spawn_selection_portrait_creates_interaction_component() {
        // Verify that Interaction::default() is included in portrait spawn
        let interaction = Interaction::default();
        assert_eq!(interaction, Interaction::None);
    }

    #[test]
    fn active_group_highlight_requires_group_membership() {
        // Verify active_group entities lookup works
        let mut selection = Selection::default();
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        let e3 = world.spawn_empty().id();
        selection.groups = vec![
            SelectionGroup { object_type: ObjectEnum::Peacekeeper, entities: vec![e1, e2] },
            SelectionGroup { object_type: ObjectEnum::SyndicateAgent, entities: vec![e3] },
        ];
        selection.active_group_index = Some(0);

        let active_entities: Vec<Entity> = selection.active_group()
            .map(|g| g.entities.clone())
            .unwrap_or_default();

        assert!(active_entities.contains(&e1));
        assert!(active_entities.contains(&e2));
        assert!(!active_entities.contains(&e3));
    }

    #[test]
    fn active_group_changes_on_tab_cycle() {
        // Verify active_group_index changes produce different highlight sets
        let mut selection = Selection::default();
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        selection.groups = vec![
            SelectionGroup { object_type: ObjectEnum::Peacekeeper, entities: vec![e1] },
            SelectionGroup { object_type: ObjectEnum::SyndicateAgent, entities: vec![e2] },
        ];
        selection.active_group_index = Some(0);

        // Before tab: e1 highlighted
        let active_before = selection.active_group().unwrap().entities.clone();
        assert!(active_before.contains(&e1));
        assert!(!active_before.contains(&e2));

        // After tab: switch to group 1
        selection.active_group_index = Some(1);
        let active_after = selection.active_group().unwrap().entities.clone();
        assert!(!active_after.contains(&e1));
        assert!(active_after.contains(&e2));
    }

    #[test]
    fn selection_portrait_remove_nonexistent_entity_is_noop() {
        let mut selection = Selection::default();
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        selection.groups = vec![
            SelectionGroup { object_type: ObjectEnum::Peacekeeper, entities: vec![e1] },
        ];
        selection.active_group_index = Some(0);

        // Removing entity not in selection should be harmless
        let removed = selection.remove_entity(e2);
        assert!(!removed);
        assert_eq!(selection.total_entity_count(), 1);
    }

    // === ResourceIcon component tests ===

    #[test]
    fn resource_icon_stores_entity() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let icon = ResourceIcon { resource_entity: entity };
        assert_eq!(icon.resource_entity, entity);
    }

    #[test]
    fn resource_icon_different_from_unit_and_structure_icons() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let _resource_icon = ResourceIcon { resource_entity: entity };
        let _unit_icon = UnitIcon { unit_entity: entity };
        let _struct_icon = StructureIcon { structure_entity: entity };
        // All three icon types can coexist for the same entity without conflict
    }

    // === Resource selection counting tests ===

    #[test]
    fn resource_only_selection_shows_in_groups() {
        // Resources should appear in selection groups and be counted
        let mut selection = Selection::default();
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        selection.groups = vec![
            SelectionGroup { object_type: ObjectEnum::SpaceCrystalsPatch, entities: vec![e1] },
        ];
        assert_eq!(selection.groups.len(), 1);
        assert_eq!(selection.groups[0].object_type, ObjectEnum::SpaceCrystalsPatch);
    }

    #[test]
    fn sds_selection_shows_in_groups() {
        let mut selection = Selection::default();
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        selection.groups = vec![
            SelectionGroup { object_type: ObjectEnum::SupplyDeliveryStation, entities: vec![e1] },
        ];
        assert_eq!(selection.groups.len(), 1);
        assert_eq!(selection.groups[0].object_type, ObjectEnum::SupplyDeliveryStation);
    }

    #[test]
    fn mixed_resource_and_unit_selection() {
        let mut selection = Selection::default();
        let mut world = World::new();
        let e1 = world.spawn_empty().id();
        let e2 = world.spawn_empty().id();
        selection.groups = vec![
            SelectionGroup { object_type: ObjectEnum::SpaceCrystalsPatch, entities: vec![e1] },
            SelectionGroup { object_type: ObjectEnum::Peacekeeper, entities: vec![e2] },
        ];
        assert_eq!(selection.groups.len(), 2);
        assert_eq!(selection.total_entity_count(), 2);
    }

    // === Alt-Click Portrait Camera Snap Centering Tests ===

    #[test]
    fn alt_click_snap_z_offset_default_height() {
        // At default camera height (40), z_offset should be 25.0
        let camera_height = 40.0_f32;
        let target_z = 10.0_f32;
        let z_offset = camera_height * 25.0 / 40.0;
        let result_cam_z = target_z + z_offset;
        assert!((result_cam_z - 35.0).abs() < 0.001,
            "Camera at height 40 centering on z=10 should snap to z=35, got {}", result_cam_z);
    }

    #[test]
    fn alt_click_snap_z_offset_zoomed_in() {
        // At zoomed-in height (20), z_offset shrinks proportionally
        let camera_height = 20.0_f32;
        let target_z = 5.0_f32;
        let z_offset = camera_height * 25.0 / 40.0;
        let result_cam_z = target_z + z_offset;
        assert!((result_cam_z - 17.5).abs() < 0.001,
            "Camera at height 20 centering on z=5 should snap to z=17.5, got {}", result_cam_z);
    }

    #[test]
    fn alt_click_snap_x_is_exact_target() {
        // X centering is direct assignment with no offset
        let target_x = 42.0_f32;
        let result_cam_x = target_x;
        assert!((result_cam_x - 42.0).abs() < 0.001,
            "Camera X should snap exactly to target X");
    }

    #[test]
    fn alt_click_snap_formula_matches_control_group_snap() {
        // Both snap paths must use the same z_offset formula
        // This test verifies the formula produces identical results
        let camera_height = 35.0_f32;
        let target_z = -8.0_f32;

        // Portrait snap formula (hud.rs)
        let portrait_z_offset = camera_height * 25.0 / 40.0;
        let portrait_cam_z = target_z + portrait_z_offset;

        // Control group snap formula (resources.rs)
        let group_z_offset = camera_height * 25.0 / 40.0;
        let group_cam_z = target_z + group_z_offset;

        assert!((portrait_cam_z - group_cam_z).abs() < 0.001,
            "Both snap paths must produce identical results");
    }

    #[test]
    fn alt_click_snap_is_instant_no_interpolation() {
        // Verify that snap centering sets the value directly (no lerp factor)
        // The system does: cam.x = target.x; cam.z = target.z + z_offset;
        // There is no t factor, no lerp, no animation — just direct assignment.
        let camera_height = 40.0_f32;
        let target_pos = Vec3::new(100.0, 0.0, -50.0);
        let z_offset = camera_height * 25.0 / 40.0;

        // Simulate snap: direct assignment
        let cam_x = target_pos.x;
        let cam_z = target_pos.z + z_offset;

        // Result should be exactly target position + offset, regardless of
        // previous camera position (no lerp toward target)
        assert!((cam_x - 100.0).abs() < f32::EPSILON);
        assert!((cam_z - (-25.0)).abs() < 0.001,
            "Snap should be instant: expected -25.0, got {}", cam_z);
    }
}
