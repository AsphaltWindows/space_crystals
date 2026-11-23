use bevy::prelude::*;
use crate::types::*;
use crate::game::types::{ObjectInstance, RecruitmentCenterState, SupplyTowerState, BarracksState, HeadquartersState, RallyTarget, TunnelExpansionMarker, RallyPointMarker};
use crate::ui::types::{CursorTarget, CursorTargetEnum, CursorOverUi, ObjectInterfaceState, CommandPanelTarget, StructureMenuState};
use crate::game::units::types::*;
use crate::game::world::faction::{spawn_or_update_rally_marker, despawn_rally_marker_for};

/// System to handle left-click target selection when in AwaitingTarget(SetRallyPoint) mode.
/// Sets the rally point on all selected production structures of the active group type
/// (Barracks, HQ, or Supply Tower).
pub fn set_rally_point_click_system(
    buttons: Res<ButtonInput<MouseButton>>,
    mut interface_state: ResMut<ObjectInterfaceState>,
    cursor_target: Res<CursorTarget>,
    cursor_over_ui: Res<CursorOverUi>,
    selection: Res<Selection>,
    mut barracks_query: Query<&mut BarracksState>,
    mut hq_query: Query<(&mut HeadquartersState, &TunnelExpansionMarker)>,
    mut st_query: Query<&mut SupplyTowerState>,
    mut rc_query: Query<&mut RecruitmentCenterState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_markers: Query<(Entity, &RallyPointMarker)>,
    object_transforms: Query<&Transform, With<ObjectInstance>>,
) {
    // Only process when in SetRallyPoint awaiting target mode
    if !matches!(*interface_state, ObjectInterfaceState::AwaitingTarget(CommandType::SetRallyPoint)) {
        return;
    }

    let is_left_click = buttons.just_pressed(MouseButton::Left);
    if !is_left_click {
        return;
    }

    // Block left-clicks over UI
    if cursor_over_ui.0 {
        return;
    }

    if cursor_target.kind == CursorTargetEnum::None {
        return;
    }

    let Some(group) = selection.active_group() else { return };

    // Determine rally target from cursor
    let rally_target = if let Some(entity) = cursor_target.entity {
        RallyTarget::Object(entity)
    } else if let Some(location) = cursor_target.location {
        RallyTarget::Location(location)
    } else {
        return;
    };

    // For Object targets, look up the target entity's world position for the visual marker
    let object_world_pos = if let RallyTarget::Object(entity) = &rally_target {
        object_transforms.get(*entity).ok().map(|t| t.translation)
    } else {
        None
    };

    // Set rally point on all structures in the active group
    for &target_entity in &group.entities {
        if let Ok(mut bk_state) = barracks_query.get_mut(target_entity) {
            bk_state.rally_point = Some(rally_target.clone());
            info!("Barracks: Rally point set via command mode");
            spawn_or_update_rally_marker(&mut commands, &mut meshes, &mut materials, &existing_markers, target_entity, &rally_target, object_world_pos);
        } else if let Ok((mut hq_state, expansion_marker)) = hq_query.get_mut(target_entity) {
            // If clicking the parent tunnel, clear rally point (unit stays in network)
            if let RallyTarget::Object(entity) = &rally_target {
                if *entity == expansion_marker.parent_tunnel {
                    hq_state.rally_point = None;
                    despawn_rally_marker_for(&mut commands, &existing_markers, target_entity);
                    info!("Headquarters: Rally point cleared via command mode (target is parent tunnel)");
                    continue;
                }
            }
            hq_state.rally_point = Some(rally_target.clone());
            info!("Headquarters: Rally point set via command mode");
            spawn_or_update_rally_marker(&mut commands, &mut meshes, &mut materials, &existing_markers, target_entity, &rally_target, object_world_pos);
        } else if let Ok(mut st_state) = st_query.get_mut(target_entity) {
            st_state.rally_point = Some(rally_target.clone());
            info!("Supply Tower: Rally point set via command mode");
            spawn_or_update_rally_marker(&mut commands, &mut meshes, &mut materials, &existing_markers, target_entity, &rally_target, object_world_pos);
        } else if let Ok(mut rc_state) = rc_query.get_mut(target_entity) {
            // RC rally_point is Option<Vec3>, not Option<RallyTarget> -- resolve to world position
            let location = match &rally_target {
                RallyTarget::Object(e) => object_transforms.get(*e).ok().map(|t| t.translation),
                RallyTarget::Location(loc) => Some(*loc),
            };
            rc_state.rally_point = location;
            info!("Recruitment Center: Rally point set via command mode");
            spawn_or_update_rally_marker(&mut commands, &mut meshes, &mut materials, &existing_markers, target_entity, &rally_target, object_world_pos);
        }
    }

    // Return to the appropriate structure menu after setting rally point
    let active_type = selection.active_type();
    if active_type == Some(ObjectEnum::RecruitmentCenter) {
        *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu);
    } else {
        *interface_state = ObjectInterfaceState::Default;
    }
}

/// System to handle left-click target selection when in AwaitingTarget(ScheduleDeliveries) mode.
/// Sets the scheduled SDS on the selected Supply Tower.
pub fn schedule_deliveries_click_system(
    buttons: Res<ButtonInput<MouseButton>>,
    mut interface_state: ResMut<ObjectInterfaceState>,
    cursor_target: Res<CursorTarget>,
    cursor_over_ui: Res<CursorOverUi>,
    panel_target: Res<CommandPanelTarget>,
    mut st_query: Query<&mut SupplyTowerState>,
    object_query: Query<&ObjectInstance>,
) {
    // Only process when in ScheduleDeliveries awaiting target mode
    if !matches!(*interface_state, ObjectInterfaceState::AwaitingTarget(CommandType::ScheduleDeliveries)) {
        return;
    }

    let is_left_click = buttons.just_pressed(MouseButton::Left);
    if !is_left_click {
        return;
    }

    // Block left-clicks over UI
    if cursor_over_ui.0 {
        return;
    }

    let Some(target_entity) = panel_target.entity else { return };

    // Must click on an entity
    let Some(clicked_entity) = cursor_target.entity else {
        info!("Schedule Deliveries: Must click on a Supply Delivery Station");
        return;
    };

    // Verify clicked entity is a Supply Delivery Station
    if let Ok(obj_instance) = object_query.get(clicked_entity) {
        if obj_instance.object_type != ObjectEnum::SupplyDeliveryStation {
            info!("Schedule Deliveries: Target is not a Supply Delivery Station");
            return;
        }
    } else {
        return;
    }

    // Set scheduled SDS on the Supply Tower
    if let Ok(mut st_state) = st_query.get_mut(target_entity) {
        st_state.scheduled_sds = Some(clicked_entity);
        info!("Supply Tower: Scheduled deliveries to SDS {:?}", clicked_entity);
    }

    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu);
}

// No dedicated rally tests exist in the original test block.
