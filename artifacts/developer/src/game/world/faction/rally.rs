use bevy::prelude::*;
use crate::types::{Owner, MainCamera, Selected, SelectionBounds, LocalPlayer};
use crate::game::types::*;
use crate::ui::types::{ObjectInterfaceState, StructureMenuState, CursorOverUi};

// =====================================================
// RALLY POINT CLEANUP SYSTEM
// =====================================================

/// System that clears invalid rally point Object targets when the referenced entity is destroyed
pub fn rally_target_cleanup_system(
    mut barracks_query: Query<&mut BarracksState>,
    all_entities: Query<Entity>,
) {
    for mut bk_state in barracks_query.iter_mut() {
        if let Some(RallyTarget::Object(entity)) = &bk_state.rally_point {
            if all_entities.get(*entity).is_err() {
                bk_state.rally_point = None;
            }
        }
    }
}

// =====================================================
// PRODUCTION RALLY POINT SYSTEM
// =====================================================

/// System to set rally point via right-click on any production structure that is selected.
/// Handles Barracks, Headquarters, and Supply Tower.
pub fn production_rally_point_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut barracks_query: Query<(Entity, &mut BarracksState), With<Selected>>,
    mut hq_query: Query<(Entity, &mut HeadquartersState, &TunnelExpansionMarker), With<Selected>>,
    mut st_query: Query<(Entity, &mut SupplyTowerState), With<Selected>>,
    mut rc_query: Query<(Entity, &mut RecruitmentCenterState), (With<Selected>, Without<BarracksState>, Without<HeadquartersState>, Without<SupplyTowerState>, Without<ArmoryState>)>,
    mut armory_query: Query<(Entity, &mut ArmoryState), (With<Selected>, Without<BarracksState>, Without<HeadquartersState>, Without<SupplyTowerState>, Without<RecruitmentCenterState>)>,
    potential_targets: Query<(Entity, &Transform, &Owner, &SelectionBounds), With<ObjectInstance>>,
    cursor_over_ui: Res<CursorOverUi>,
    local_player: Res<LocalPlayer>,
    panel_state: Res<ObjectInterfaceState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_markers: Query<(Entity, &RallyPointMarker)>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }

    if cursor_over_ui.0 {
        return;
    }

    // Only handle when in a production structure menu state
    let is_production_menu = matches!(*panel_state,
        ObjectInterfaceState::StructureMenu(
            StructureMenuState::BarracksMenu |
            StructureMenuState::HeadquartersMenu |
            StructureMenuState::SupplyTowerMenu |
            StructureMenuState::RecruitmentCenterMenu |
            StructureMenuState::ArmoryMenu
        )
    );
    if !is_production_menu {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = cameras.single() else { return };

    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };

    // Determine the rally target from the click
    let rally_target = compute_rally_target_from_click(
        cursor_pos, &ray, camera, camera_transform,
        &potential_targets, &local_player,
    );

    let Some(rally_target) = rally_target else { return };

    // For Object targets, look up the target entity's world position for the visual marker
    let object_world_pos = if let RallyTarget::Object(target_entity) = &rally_target {
        potential_targets.iter()
            .find(|(e, _, _, _)| *e == *target_entity)
            .map(|(_, t, _, _)| t.translation)
    } else {
        None
    };

    // Set rally point on all selected production structures of the active group type
    match *panel_state {
        ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu) => {
            for (entity, mut bk_state) in &mut barracks_query {
                bk_state.rally_point = Some(rally_target.clone());
                info!("Barracks: Rally point set");
                spawn_or_update_rally_marker(&mut commands, &mut meshes, &mut materials, &existing_markers, entity, &rally_target, object_world_pos);
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu) => {
            for (entity, mut hq_state, expansion_marker) in &mut hq_query {
                // If clicking the parent tunnel, clear rally point (unit stays in network)
                if let RallyTarget::Object(target_e) = &rally_target {
                    if *target_e == expansion_marker.parent_tunnel {
                        hq_state.rally_point = None;
                        despawn_rally_marker_for(&mut commands, &existing_markers, entity);
                        info!("Headquarters: Rally point cleared (target is parent tunnel)");
                        continue;
                    }
                }
                hq_state.rally_point = Some(rally_target.clone());
                info!("Headquarters: Rally point set");
                spawn_or_update_rally_marker(&mut commands, &mut meshes, &mut materials, &existing_markers, entity, &rally_target, object_world_pos);
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu) => {
            for (entity, mut st_state) in &mut st_query {
                st_state.rally_point = Some(rally_target.clone());
                info!("Supply Tower: Rally point set");
                spawn_or_update_rally_marker(&mut commands, &mut meshes, &mut materials, &existing_markers, entity, &rally_target, object_world_pos);
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu) => {
            for (entity, mut rc_state) in &mut rc_query {
                // RC rally_point is Option<Vec3>, not Option<RallyTarget> — resolve to world position
                let location = match &rally_target {
                    RallyTarget::Object(e) => potential_targets.iter()
                        .find(|(te, _, _, _)| *te == *e)
                        .map(|(_, t, _, _)| t.translation),
                    RallyTarget::Location(loc) => Some(*loc),
                };
                rc_state.rally_point = location;
                info!("Recruitment Center: Rally point set");
                spawn_or_update_rally_marker(&mut commands, &mut meshes, &mut materials, &existing_markers, entity, &rally_target, object_world_pos);
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu) => {
            for (entity, mut armory_state) in &mut armory_query {
                armory_state.rally_point = Some(rally_target.clone());
                info!("Armory: Rally point set");
                spawn_or_update_rally_marker(&mut commands, &mut meshes, &mut materials, &existing_markers, entity, &rally_target, object_world_pos);
            }
        }
        _ => {}
    }
}

/// Compute a RallyTarget from a click position — checks entities first, then ground plane.
fn compute_rally_target_from_click(
    _cursor_pos: Vec2,
    ray: &Ray3d,
    _camera: &Camera,
    _camera_transform: &GlobalTransform,
    potential_targets: &Query<(Entity, &Transform, &Owner, &SelectionBounds), With<ObjectInstance>>,
    _local_player: &Res<LocalPlayer>,
) -> Option<RallyTarget> {
    // Check for entity under cursor via 3D ray-AABB intersection
    let ray_origin = ray.origin;
    let ray_dir = *ray.direction;
    let click_pad = 0.3;
    let mut best_distance = f32::MAX;
    let mut clicked_entity: Option<Entity> = None;

    for (target_entity, target_transform, _target_owner, bounds) in potential_targets.iter() {
        let center = target_transform.translation;
        let aabb_min = Vec3::new(
            center.x - bounds.half_x - click_pad,
            center.y - bounds.half_y - click_pad,
            center.z - bounds.half_z - click_pad,
        );
        let aabb_max = Vec3::new(
            center.x + bounds.half_x + click_pad,
            center.y + bounds.half_y + click_pad,
            center.z + bounds.half_z + click_pad,
        );

        if let Some(t) = crate::ui::utils::ray_aabb_intersect(ray_origin, ray_dir, aabb_min, aabb_max) {
            if t < best_distance {
                clicked_entity = Some(target_entity);
                best_distance = t;
            }
        }
    }

    if let Some(target_entity) = clicked_entity {
        return Some(RallyTarget::Object(target_entity));
    }

    // Ground click — set rally to location
    if ray.direction.y.abs() > 0.001 {
        let t = -ray.origin.y / ray.direction.y;
        if t > 0.0 {
            let world_hit = ray.origin + *ray.direction * t;
            return Some(RallyTarget::Location(world_hit));
        }
    }

    None
}

/// Spawn or update a visual rally point marker for a production structure.
/// Despawns any existing marker for the owner, then spawns a new one at the rally location.
/// For Object targets, `object_world_pos` provides the target entity's position for the marker.
pub fn spawn_or_update_rally_marker(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    existing_markers: &Query<(Entity, &RallyPointMarker)>,
    owner_structure: Entity,
    rally_target: &RallyTarget,
    object_world_pos: Option<Vec3>,
) {
    // Despawn existing marker for this structure
    despawn_rally_marker_for(commands, existing_markers, owner_structure);

    // Determine world position for the marker
    let marker_pos = match rally_target {
        RallyTarget::Location(pos) => *pos,
        RallyTarget::Object(_) => {
            // For object targets, place marker at the target's position if provided
            if let Some(pos) = object_world_pos {
                pos
            } else {
                return;
            }
        }
    };

    // Spawn a small cylinder as the rally point indicator
    let marker_mesh = meshes.add(Cylinder::new(0.3, 0.1));
    let marker_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 1.0, 0.2, 0.7),
        emissive: LinearRgba::new(0.0, 2.0, 0.0, 1.0),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        Mesh3d(marker_mesh),
        MeshMaterial3d(marker_material),
        Transform::from_translation(Vec3::new(marker_pos.x, 0.15, marker_pos.z)),
        RallyPointMarker { owner_structure },
    ));
}

/// Despawn any existing rally point marker for the given structure entity.
pub fn despawn_rally_marker_for(
    commands: &mut Commands,
    existing_markers: &Query<(Entity, &RallyPointMarker)>,
    owner_structure: Entity,
) {
    for (marker_entity, marker) in existing_markers.iter() {
        if marker.owner_structure == owner_structure {
            commands.entity(marker_entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headquarters_default_has_no_rally_point() {
        let state = HeadquartersState::default();
        assert!(state.rally_point.is_none());
    }

    #[test]
    fn headquarters_rally_point_location_means_eject() {
        let state = HeadquartersState {
            rally_point: Some(RallyTarget::Location(Vec3::new(10.0, 0.0, 10.0))),
            ..default()
        };
        // A Location rally point means the unit should eject to surface
        let should_eject = matches!(state.rally_point, Some(RallyTarget::Location(_)));
        assert!(should_eject);
    }

    #[test]
    fn headquarters_no_rally_means_tunnel_network() {
        let state = HeadquartersState::default();
        // No rally point means the unit stays in the tunnel network
        let should_eject = match &state.rally_point {
            Some(RallyTarget::Location(_)) => true,
            Some(RallyTarget::Object(_)) => true, // non-parent-tunnel object
            None => false,
        };
        assert!(!should_eject);
    }

    #[test]
    fn headquarters_rally_target_parent_tunnel_clears() {
        // Simulate the "rally on parent tunnel = clear" logic
        let parent_tunnel = Entity::from_raw_u32(42).unwrap();
        let rally_target = RallyTarget::Object(parent_tunnel);

        let mut state = HeadquartersState {
            rally_point: Some(rally_target),
            ..default()
        };

        // The logic: if rally target entity == parent_tunnel, clear
        if let Some(RallyTarget::Object(entity)) = &state.rally_point {
            if *entity == parent_tunnel {
                state.rally_point = None;
            }
        }

        assert!(state.rally_point.is_none(), "Rally should be cleared when target is parent tunnel");
    }

    #[test]
    fn headquarters_rally_target_other_entity_does_not_clear() {
        let parent_tunnel = Entity::from_raw_u32(42).unwrap();
        let other_entity = Entity::from_raw_u32(99).unwrap();
        let rally_target = RallyTarget::Object(other_entity);

        let mut state = HeadquartersState {
            rally_point: Some(rally_target),
            ..default()
        };

        // The logic: if rally target entity != parent_tunnel, keep it
        if let Some(RallyTarget::Object(entity)) = &state.rally_point {
            if *entity == parent_tunnel {
                state.rally_point = None;
            }
        }

        assert!(state.rally_point.is_some(), "Rally should NOT be cleared when target is different entity");
    }

    #[test]
    fn rally_point_marker_stores_owner() {
        let owner = Entity::from_raw_u32(10).unwrap();
        let marker = RallyPointMarker { owner_structure: owner };
        assert_eq!(marker.owner_structure, owner);
    }
}
