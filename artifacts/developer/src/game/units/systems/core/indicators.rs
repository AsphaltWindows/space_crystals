use bevy::prelude::*;
use crate::types::*;
use crate::game::units::types::*;
use crate::game::units::types::types::{CommandIndicator, CommandIndicatorType, command_indicator_color, command_has_indicator};
use crate::game::combat::types::{SeparationRadius, SEPARATION_FORCE_SCALE};

/// Create an emissive color variant from a base indicator color (scaled down for glow).
/// Returns `LinearRgba` directly, which is the type expected by `StandardMaterial::emissive` in Bevy 0.17.
fn emissive_from_color(color: Color) -> LinearRgba {
    match color {
        Color::Srgba(c) => LinearRgba::rgb(c.red * 0.8, c.green * 0.8, c.blue * 0.8),
        _ => LinearRgba::WHITE,
    }
}

/// Describes a desired indicator for a specific unit+command combination.
struct DesiredIndicator {
    owner_unit: Entity,
    indicator_type: CommandIndicatorType,
    target_entity: Option<Entity>,
    target_position: Option<Vec3>,
    color: Color,
    patrol_index: u8,
}

/// Cached material handles for command indicators, keyed by color.
/// Only 3 colors exist (green, red, orange) so a fixed struct suffices.
pub(crate) struct CachedIndicatorMaterials {
    green: Handle<StandardMaterial>,
    red: Handle<StandardMaterial>,
    orange: Handle<StandardMaterial>,
}

/// System to synchronize command indicators for all selected units.
/// Runs every frame: diffs existing indicators against desired state,
/// despawning stale ones and spawning new ones.
/// Mesh and material handles are cached via Local to avoid per-spawn asset allocation.
pub fn command_indicator_sync_system(
    mut commands: Commands,
    selected_units: Query<(Entity, &UnitCommand), (With<Unit>, With<Selected>)>,
    existing_indicators: Query<(Entity, &CommandIndicator)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cached_location_mesh: Local<Option<Handle<Mesh>>>,
    mut cached_object_mesh: Local<Option<Handle<Mesh>>>,
    mut cached_materials: Local<Option<CachedIndicatorMaterials>>,
) {
    // Build desired indicator set from selected units' commands
    let mut desired: Vec<DesiredIndicator> = Vec::new();

    for (unit_entity, cmd) in &selected_units {
        if !command_has_indicator(cmd) {
            continue;
        }

        let color = command_indicator_color(cmd);

        match cmd {
            UnitCommand::Move(pos) | UnitCommand::AttackMove(pos)
            | UnitCommand::AttackLocation(pos) | UnitCommand::Reverse(pos)
            | UnitCommand::BuildTunnel(pos) => {
                desired.push(DesiredIndicator {
                    owner_unit: unit_entity,
                    indicator_type: CommandIndicatorType::Location,
                    target_entity: None,
                    target_position: Some(*pos),
                    color,
                    patrol_index: 0,
                });
            }
            UnitCommand::AttackTarget(target) | UnitCommand::Enter(target) => {
                desired.push(DesiredIndicator {
                    owner_unit: unit_entity,
                    indicator_type: CommandIndicatorType::Object,
                    target_entity: Some(*target),
                    target_position: None,
                    color,
                    patrol_index: 0,
                });
            }
            UnitCommand::Patrol { start, end, .. } => {
                desired.push(DesiredIndicator {
                    owner_unit: unit_entity,
                    indicator_type: CommandIndicatorType::Location,
                    target_entity: None,
                    target_position: Some(*start),
                    color,
                    patrol_index: 0,
                });
                desired.push(DesiredIndicator {
                    owner_unit: unit_entity,
                    indicator_type: CommandIndicatorType::Location,
                    target_entity: None,
                    target_position: Some(*end),
                    color,
                    patrol_index: 1,
                });
            }
            _ => {}
        }
    }

    // Diff: remove indicators not matching desired set
    let mut kept: Vec<Entity> = Vec::new();
    for (indicator_entity, indicator) in &existing_indicators {
        let still_desired = desired.iter().any(|d| {
            d.owner_unit == indicator.owner_unit
                && d.indicator_type == indicator.indicator_type
                && d.target_entity == indicator.target_entity
                && d.patrol_index == indicator.patrol_index
        });

        if still_desired {
            kept.push(indicator_entity);
        } else {
            commands.entity(indicator_entity).despawn();
        }
    }

    // Spawn missing indicators
    for d in desired.iter() {
        let already_exists = existing_indicators.iter().any(|(ent, ind)| {
            kept.contains(&ent)
                && ind.owner_unit == d.owner_unit
                && ind.indicator_type == d.indicator_type
                && ind.target_entity == d.target_entity
                && ind.patrol_index == d.patrol_index
        });

        if already_exists {
            continue;
        }

        let indicator_component = CommandIndicator {
            owner_unit: d.owner_unit,
            indicator_type: d.indicator_type,
            target_entity: d.target_entity,
            patrol_index: d.patrol_index,
        };

        let mats = cached_materials.get_or_insert_with(|| {
            let mut make = |color: Color| materials.add(StandardMaterial {
                base_color: color,
                emissive: emissive_from_color(color),
                unlit: true,
                ..default()
            });
            CachedIndicatorMaterials {
                green: make(Color::srgb(0.0, 1.0, 0.0)),
                red: make(Color::srgb(1.0, 0.2, 0.0)),
                orange: make(Color::srgb(1.0, 0.6, 0.0)),
            }
        });
        let material = match d.color {
            Color::Srgba(c) if c.red > 0.9 && c.green < 0.3 => mats.red.clone(),
            Color::Srgba(c) if c.red > 0.9 && c.green > 0.5 => mats.orange.clone(),
            _ => mats.green.clone(),
        };

        match d.indicator_type {
            CommandIndicatorType::Location => {
                let pos = d.target_position.unwrap_or(Vec3::ZERO);
                let mesh = cached_location_mesh.get_or_insert_with(|| {
                    meshes.add(Cylinder::new(0.3, 0.05))
                }).clone();
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(pos.x, 0.05, pos.z),
                    indicator_component,
                ));
            }
            CommandIndicatorType::Object => {
                if let Some(target_entity) = d.target_entity {
                    let mesh = cached_object_mesh.get_or_insert_with(|| {
                        meshes.add(Torus::new(0.4, 0.56))
                    }).clone();
                    let indicator_id = commands.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(material.clone()),
                        Transform::from_xyz(0.0, -0.3, 0.0)
                            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                        indicator_component,
                    )).id();
                    commands.entity(target_entity).add_child(indicator_id);
                }
            }
        }
    }
}

/// Air unit soft separation system -- applies gentle repulsion between nearby air units
/// to prevent stacking. Runs after movement systems so it doesn't interfere with pathfinding.
pub fn air_unit_separation_system(
    time: Res<Time>,
    mut air_units: Query<
        (Entity, &mut Transform, &SeparationRadius, &DomainEnum),
        (With<Unit>, Without<InTunnelNetwork>)
    >,
) {
    let delta = time.delta_secs();
    if delta < 0.0001 {
        return;
    }

    let air_positions: Vec<(Entity, Vec3, f32)> = air_units
        .iter()
        .filter(|(_, _, _, domain)| **domain == DomainEnum::Air)
        .map(|(e, t, sr, _)| (e, t.translation, sr.0))
        .collect();

    for (entity, mut transform, sep_radius, domain) in &mut air_units {
        if *domain != DomainEnum::Air {
            continue;
        }

        let current_pos = transform.translation;
        let mut repulsion = Vec3::ZERO;

        for &(other_entity, other_pos, other_radius) in &air_positions {
            if entity == other_entity {
                continue;
            }

            let diff = Vec3::new(
                current_pos.x - other_pos.x,
                0.0,
                current_pos.z - other_pos.z,
            );
            let dist = diff.length();

            let threshold = sep_radius.0.max(other_radius);

            if dist < threshold && dist > 0.01 {
                let direction = diff / dist;
                let strength = 1.0 - (dist / threshold);
                repulsion += direction * strength * SEPARATION_FORCE_SCALE;
            }
        }

        if repulsion.length_squared() > 0.0001 {
            transform.translation.x += repulsion.x * delta;
            transform.translation.z += repulsion.z * delta;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    // === Emissive color tests ===

    #[test]
    fn emissive_from_green_color() {
        let green = Color::srgb(0.0, 1.0, 0.0);
        let emissive = emissive_from_color(green);
        assert!((emissive.red - 0.0).abs() < 0.001);
        assert!((emissive.green - 0.8).abs() < 0.001);
        assert!((emissive.blue - 0.0).abs() < 0.001);
    }

    #[test]
    fn emissive_from_red_color() {
        let red = Color::srgb(1.0, 0.2, 0.0);
        let emissive = emissive_from_color(red);
        assert!((emissive.red - 0.8).abs() < 0.001);
        assert!((emissive.green - 0.16).abs() < 0.001);
        assert!((emissive.blue - 0.0).abs() < 0.001);
    }

    #[test]
    fn emissive_from_orange_color() {
        let orange = Color::srgb(1.0, 0.6, 0.0);
        let emissive = emissive_from_color(orange);
        assert!((emissive.red - 0.8).abs() < 0.001);
        assert!((emissive.green - 0.48).abs() < 0.001);
        assert!((emissive.blue - 0.0).abs() < 0.001);
    }

    #[test]
    fn emissive_from_non_srgba_returns_white() {
        let linear_color = Color::LinearRgba(LinearRgba::rgb(0.5, 0.5, 0.5));
        let emissive = emissive_from_color(linear_color);
        assert_eq!(emissive, LinearRgba::WHITE);
    }

    // === Indicator sync tests ===

    fn create_indicator_test_world() -> World {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        world
    }

    fn spawn_selected_unit_with_command(world: &mut World, cmd: UnitCommand) -> Entity {
        world.spawn((
            Unit,
            Selected,
            Transform::from_xyz(0.0, 0.5, 0.0),
            cmd,
        )).id()
    }

    #[test]
    fn sync_spawns_location_indicator_for_move() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::Move(Vec3::new(5.0, 0.0, 5.0)));
        world.run_system_once(command_indicator_sync_system).unwrap();

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].indicator_type, CommandIndicatorType::Location);
    }

    #[test]
    fn sync_spawns_object_indicator_for_attack() {
        let mut world = create_indicator_test_world();
        let target = world.spawn(Transform::from_xyz(10.0, 0.5, 10.0)).id();
        spawn_selected_unit_with_command(&mut world, UnitCommand::AttackTarget(target));
        world.run_system_once(command_indicator_sync_system).unwrap();

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].indicator_type, CommandIndicatorType::Object);
        assert_eq!(indicators[0].target_entity, Some(target));
    }

    #[test]
    fn sync_spawns_two_indicators_for_patrol() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::Patrol {
            start: Vec3::new(0.0, 0.0, 0.0),
            end: Vec3::new(10.0, 0.0, 10.0),
            going_to_end: true,
        });
        world.run_system_once(command_indicator_sync_system).unwrap();

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 2);
        let indices: Vec<u8> = indicators.iter().map(|i| i.patrol_index).collect();
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
    }

    #[test]
    fn sync_no_indicator_for_idle() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::Idle);
        world.run_system_once(command_indicator_sync_system).unwrap();
        assert_eq!(world.query::<&CommandIndicator>().iter(&world).count(), 0);
    }

    #[test]
    fn sync_no_indicator_for_hold_position() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::HoldPosition);
        world.run_system_once(command_indicator_sync_system).unwrap();
        assert_eq!(world.query::<&CommandIndicator>().iter(&world).count(), 0);
    }

    #[test]
    fn sync_removes_indicator_when_deselected() {
        let mut world = create_indicator_test_world();
        let unit = spawn_selected_unit_with_command(&mut world, UnitCommand::Move(Vec3::new(5.0, 0.0, 5.0)));
        world.run_system_once(command_indicator_sync_system).unwrap();
        assert_eq!(world.query::<&CommandIndicator>().iter(&world).count(), 1);

        world.entity_mut(unit).remove::<Selected>();
        world.run_system_once(command_indicator_sync_system).unwrap();
        assert_eq!(world.query::<&CommandIndicator>().iter(&world).count(), 0);
    }

    #[test]
    fn sync_multiple_selected_units_show_all_indicators() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::Move(Vec3::new(5.0, 0.0, 5.0)));
        spawn_selected_unit_with_command(&mut world, UnitCommand::Move(Vec3::new(10.0, 0.0, 10.0)));
        let target = world.spawn(Transform::from_xyz(15.0, 0.5, 15.0)).id();
        spawn_selected_unit_with_command(&mut world, UnitCommand::AttackTarget(target));
        world.run_system_once(command_indicator_sync_system).unwrap();

        assert_eq!(world.query::<&CommandIndicator>().iter(&world).count(), 3);
    }

    #[test]
    fn sync_indicator_tracks_owner_unit() {
        let mut world = create_indicator_test_world();
        let unit = spawn_selected_unit_with_command(&mut world, UnitCommand::Move(Vec3::ZERO));
        world.run_system_once(command_indicator_sync_system).unwrap();

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].owner_unit, unit);
    }

    #[test]
    fn sync_enter_command_creates_object_indicator() {
        let mut world = create_indicator_test_world();
        let tunnel = world.spawn(Transform::from_xyz(5.0, 0.5, 5.0)).id();
        spawn_selected_unit_with_command(&mut world, UnitCommand::Enter(tunnel));
        world.run_system_once(command_indicator_sync_system).unwrap();

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].indicator_type, CommandIndicatorType::Object);
        assert_eq!(indicators[0].target_entity, Some(tunnel));
    }

    #[test]
    fn sync_attack_location_creates_location_indicator() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::AttackLocation(Vec3::new(8.0, 0.0, 3.0)));
        world.run_system_once(command_indicator_sync_system).unwrap();

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].indicator_type, CommandIndicatorType::Location);
    }

    #[test]
    fn sync_reverse_creates_location_indicator() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::Reverse(Vec3::new(3.0, 0.0, 2.0)));
        world.run_system_once(command_indicator_sync_system).unwrap();

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].indicator_type, CommandIndicatorType::Location);
    }

    #[test]
    fn sync_attack_move_creates_location_indicator() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::AttackMove(Vec3::new(7.0, 0.0, 7.0)));
        world.run_system_once(command_indicator_sync_system).unwrap();

        let indicators: Vec<_> = world.query::<&CommandIndicator>().iter(&world).collect();
        assert_eq!(indicators.len(), 1);
        assert_eq!(indicators[0].indicator_type, CommandIndicatorType::Location);
    }

    // === Memory leak fix tests ===

    #[test]
    fn indicator_sync_shares_mesh_within_single_run() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::Move(Vec3::new(1.0, 0.0, 1.0)));
        spawn_selected_unit_with_command(&mut world, UnitCommand::Move(Vec3::new(5.0, 0.0, 5.0)));
        spawn_selected_unit_with_command(&mut world, UnitCommand::Move(Vec3::new(9.0, 0.0, 9.0)));
        world.run_system_once(command_indicator_sync_system).unwrap();

        assert_eq!(world.query::<&CommandIndicator>().iter(&world).count(), 3);
        let mesh_count = world.resource::<Assets<Mesh>>().len();
        assert_eq!(mesh_count, 1, "All Location indicators should share 1 cached mesh");
    }

    #[test]
    fn indicator_sync_location_and_object_use_two_meshes() {
        let mut world = create_indicator_test_world();
        spawn_selected_unit_with_command(&mut world, UnitCommand::Move(Vec3::new(1.0, 0.0, 1.0)));
        let target = world.spawn(Transform::from_xyz(10.0, 0.5, 10.0)).id();
        spawn_selected_unit_with_command(&mut world, UnitCommand::AttackTarget(target));
        world.run_system_once(command_indicator_sync_system).unwrap();

        let mesh_count = world.resource::<Assets<Mesh>>().len();
        assert_eq!(mesh_count, 2, "Should cache exactly 2 mesh types (Cylinder + Torus)");
    }

    // === Air Unit Soft Separation Tests ===

    fn spawn_air_unit_at(world: &mut World, x: f32, z: f32, radius: f32) -> Entity {
        world.spawn((
            Transform::from_xyz(x, 1.5, z),
            Unit,
            DomainEnum::Air,
            SeparationRadius(radius),
        )).id()
    }

    fn spawn_ground_unit_at(world: &mut World, x: f32, z: f32) -> Entity {
        world.spawn((
            Transform::from_xyz(x, 0.5, z),
            Unit,
            DomainEnum::Ground,
        )).id()
    }

    fn create_separation_test_world() -> World {
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());
        world
    }

    #[test]
    fn separation_radius_component_stores_value() {
        let sr = SeparationRadius(1.5);
        assert!((sr.0 - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn separation_force_scale_is_positive() {
        assert!(SEPARATION_FORCE_SCALE > 0.0);
    }

    #[test]
    fn two_overlapping_air_units_drift_apart() {
        let mut world = create_separation_test_world();
        let e1 = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let e2 = spawn_air_unit_at(&mut world, 0.3, 0.0, 1.25);
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system).unwrap();

        let pos1 = world.entity(e1).get::<Transform>().unwrap().translation;
        let pos2 = world.entity(e2).get::<Transform>().unwrap().translation;
        let separation = (pos2.x - pos1.x).abs();
        assert!(separation > 0.3, "Units should have drifted apart: separation={}", separation);
    }

    #[test]
    fn air_units_beyond_radius_no_repulsion() {
        let mut world = create_separation_test_world();
        let e1 = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let e2 = spawn_air_unit_at(&mut world, 5.0, 0.0, 1.25);
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system).unwrap();

        let pos1 = world.entity(e1).get::<Transform>().unwrap().translation;
        let pos2 = world.entity(e2).get::<Transform>().unwrap().translation;
        assert!((pos1.x - 0.0).abs() < 0.001);
        assert!((pos2.x - 5.0).abs() < 0.001);
    }

    #[test]
    fn air_units_ignore_ground_units() {
        let mut world = create_separation_test_world();
        let air = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let _ground = spawn_ground_unit_at(&mut world, 0.1, 0.0);
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system).unwrap();

        let pos = world.entity(air).get::<Transform>().unwrap().translation;
        assert!((pos.x - 0.0).abs() < 0.001, "Air unit should not be pushed by ground unit");
    }

    #[test]
    fn separation_preserves_y_height() {
        let mut world = create_separation_test_world();
        let e1 = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let _e2 = spawn_air_unit_at(&mut world, 0.3, 0.0, 1.25);
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system).unwrap();

        let pos = world.entity(e1).get::<Transform>().unwrap().translation;
        assert!((pos.y - 1.5).abs() < 0.001, "Y should be unchanged");
    }

    #[test]
    fn separation_uses_max_radius() {
        let mut world = create_separation_test_world();
        let e1 = spawn_air_unit_at(&mut world, 0.0, 0.0, 0.5);
        let e2 = spawn_air_unit_at(&mut world, 1.0, 0.0, 2.0);
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system).unwrap();

        let pos1 = world.entity(e1).get::<Transform>().unwrap().translation;
        let pos2 = world.entity(e2).get::<Transform>().unwrap().translation;
        let separation = (pos2.x - pos1.x).abs();
        assert!(separation > 1.0, "Units should drift apart when within max radius");
    }

    #[test]
    fn three_air_units_spread_evenly() {
        let mut world = create_separation_test_world();
        let e1 = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let _e2 = spawn_air_unit_at(&mut world, 0.1, 0.0, 1.25);
        let e3 = spawn_air_unit_at(&mut world, 0.2, 0.0, 1.25);
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system).unwrap();

        let pos1 = world.entity(e1).get::<Transform>().unwrap().translation;
        let pos3 = world.entity(e3).get::<Transform>().unwrap().translation;
        assert!(pos1.x < 0.0, "Leftmost unit should move further left");
        assert!(pos3.x > 0.2, "Rightmost unit should move further right");
    }

    #[test]
    fn in_tunnel_units_excluded_from_separation() {
        let mut world = create_separation_test_world();
        let air = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let _tunneled = world.spawn((
            Transform::from_xyz(0.1, 1.5, 0.0),
            Unit, DomainEnum::Air, SeparationRadius(1.25),
            InTunnelNetwork { owner_player: 0 },
        )).id();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system).unwrap();

        let pos = world.entity(air).get::<Transform>().unwrap().translation;
        assert!((pos.x - 0.0).abs() < 0.001, "Should not be pushed by in-tunnel unit");
    }

    #[test]
    fn separation_linear_falloff_stronger_when_closer() {
        let mut world = create_separation_test_world();
        let e1_close = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let _e2_close = spawn_air_unit_at(&mut world, 0.1, 0.0, 1.25);
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system).unwrap();
        let push_close = (world.entity(e1_close).get::<Transform>().unwrap().translation.x - 0.0).abs();

        let mut world2 = create_separation_test_world();
        let e1_far = spawn_air_unit_at(&mut world2, 0.0, 0.0, 1.25);
        let _e2_far = spawn_air_unit_at(&mut world2, 1.0, 0.0, 1.25);
        world2.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world2.run_system_once(air_unit_separation_system).unwrap();
        let push_far = (world2.entity(e1_far).get::<Transform>().unwrap().translation.x - 0.0).abs();

        assert!(push_close > push_far, "Closer units should receive stronger repulsion: close={}, far={}", push_close, push_far);
    }

    #[test]
    fn no_separation_when_zero_delta() {
        let mut world = create_separation_test_world();
        let e1 = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let _e2 = spawn_air_unit_at(&mut world, 0.1, 0.0, 1.25);
        world.run_system_once(air_unit_separation_system).unwrap();

        let pos = world.entity(e1).get::<Transform>().unwrap().translation;
        assert!((pos.x - 0.0).abs() < 0.001, "No movement with zero delta");
    }

    #[test]
    fn separation_works_on_z_axis() {
        let mut world = create_separation_test_world();
        let e1 = spawn_air_unit_at(&mut world, 0.0, 0.0, 1.25);
        let _e2 = world.spawn((
            Transform::from_xyz(0.0, 1.5, 0.3),
            Unit, DomainEnum::Air, SeparationRadius(1.25),
        )).id();
        world.resource_mut::<Time<()>>().advance_by(std::time::Duration::from_millis(100));
        world.run_system_once(air_unit_separation_system).unwrap();

        let pos = world.entity(e1).get::<Transform>().unwrap().translation;
        assert!(pos.z < 0.0, "Unit should be pushed in negative Z direction");
    }
}
