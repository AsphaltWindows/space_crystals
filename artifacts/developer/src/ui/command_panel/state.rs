use bevy::prelude::*;
use crate::types::*;
use crate::game::types::{
    ObjectInstance, BarracksState, ExtractionFacilityState,
    DeploymentCenterState, SupplyTowerState, TunnelState, ArmoryState,
    SupplyChopperState, StructureInstance,
};
use crate::game::units::types::state::AgentCarryState;
use crate::game::combat::types::AttackCapability;
use super::super::types::*;
use super::helpers::compute_selected_unit_capabilities;

/// System to update the CursorTarget resource each frame based on cursor position.
/// Raycasts from cursor through camera to detect entities and ground under cursor.
/// Always detects entities/ground regardless of UI overlap — consumers decide
/// whether to respect cursor_over_ui for their specific input handling.
pub fn update_cursor_target(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    potential_targets: Query<(Entity, &Transform, &Owner, &SelectionBounds), With<ObjectInstance>>,
    local_player: Res<LocalPlayer>,
    mut cursor_target: ResMut<CursorTarget>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    // Reset each frame
    *cursor_target = CursorTarget::default();

    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = cameras.single() else { return };

    let Some(cursor_pos) = window.cursor_position() else { return };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };

    // Check entities via 3D ray-AABB intersection.
    // Uses the same ray from viewport_to_world that ground detection uses,
    // avoiding screen-space projection issues with camera viewport offsets.
    // The AABB is padded in world space for click-friendliness.
    let ray_origin = ray.origin;
    let ray_dir = *ray.direction;
    let click_pad = 0.3; // World-space padding for easier clicking

    let mut best_distance = f32::MAX;
    let mut best_entity = None;
    let mut best_owner = None;

    for (entity, transform, owner, bounds) in potential_targets.iter() {
        let center = transform.translation;
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
                best_distance = t;
                best_entity = Some(entity);
                best_owner = Some(*owner);
            }
        }
    }

    if let Some(entity) = best_entity {
        let owner = best_owner.unwrap();
        cursor_target.entity = Some(entity);
        if owner.is_neutral() {
            cursor_target.kind = CursorTargetEnum::NeutralObject;
        } else if owner.player_number() == Some(local_player.0) {
            cursor_target.kind = CursorTargetEnum::FriendlyObject;
        } else {
            cursor_target.kind = CursorTargetEnum::EnemyObject;
        }
    }

    // Debug: log cursor target details on mouse click frames to help diagnose hit detection
    let is_click = buttons.just_pressed(MouseButton::Left) || buttons.just_pressed(MouseButton::Right);
    if is_click {
        info!(
            "CursorTarget click debug: cursor=({:.0},{:.0}) kind={:?} entity={:?} targets_checked={}",
            cursor_pos.x, cursor_pos.y,
            cursor_target.kind,
            cursor_target.entity,
            potential_targets.iter().count(),
        );
    }

    // Ground plane intersection (y = 0) — always compute location for ground clicks
    if ray.direction.y.abs() > 0.001 {
        let t = -ray.origin.y / ray.direction.y;
        if t > 0.0 {
            cursor_target.location = Some(ray.origin + *ray.direction * t);
            if cursor_target.kind == CursorTargetEnum::None {
                cursor_target.kind = CursorTargetEnum::Ground;
            }
        }
    }
}

/// Check whether all entities in the selection are owned by the local player.
/// Returns false if any entity is enemy-owned, neutral, or missing an Owner component.
pub(super) fn selection_owned_by_local_player(
    selection: &Selection,
    local_player: &LocalPlayer,
    owner_query: &Query<&Owner>,
) -> bool {
    selection.groups.iter().all(|group| {
        group.entities.iter().all(|entity| {
            owner_query.get(*entity).map_or(false, |owner| {
                owner.player_number() == Some(local_player.0)
            })
        })
    })
}

/// Whether the panel should show content (not hidden).
/// Returns true when there's an active selection or we're in a structure menu / awaiting target.
/// Resource-only selections (Crystal Patches, SDS) are treated as empty for panel purposes —
/// they show InfoPanel but not the command panel.
pub(super) fn is_panel_visible(state: &ObjectInterfaceState, selection: &Selection) -> bool {
    match state {
        ObjectInterfaceState::Default | ObjectInterfaceState::AgentMenu(_) | ObjectInterfaceState::CultsRecruitMenu(_) => {
            if selection.groups.is_empty() {
                return false;
            }
            // Hide panel when selection contains only resource entities (no commands)
            let all_resources = selection.groups.iter().all(|g| g.object_type.is_resource());
            !all_resources
        }
        _ => true,
    }
}

/// System to update the command panel state based on selected structures
pub fn update_command_panel_state(
    selected_structures: Query<
        (Entity, &ObjectInstance, Option<&DeploymentCenterState>, Option<&BarracksState>, Option<&ExtractionFacilityState>, Option<&SupplyTowerState>, Option<&TunnelState>, Option<&ArmoryState>),
        (With<StructureInstance>, With<Selected>),
    >,
    selected_units: Query<(Entity, Option<&AttackCapability>, &UnitBaseEnum, &ObjectInstance, Option<&AgentCarryState>, Option<&SupplyChopperState>), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
    mut interface_state: ResMut<ObjectInterfaceState>,
    mut panel_target: ResMut<CommandPanelTarget>,
    mut unit_caps: ResMut<SelectedUnitCapabilities>,
    selection: Res<Selection>,
    local_player: Res<LocalPlayer>,
    owner_query: Query<&Owner>,
) {
    // Ownership guard: if any selected entity is not owned by the local player,
    // reset state and hide the panel. This prevents commanding enemy or neutral units.
    let owned = selection.groups.is_empty() || selection_owned_by_local_player(&selection, &local_player, &owner_query);
    if unit_caps.owned_by_local_player != owned {
        unit_caps.owned_by_local_player = owned;
    }
    if !owned {
        if !matches!(*interface_state, ObjectInterfaceState::Default) {
            *interface_state = ObjectInterfaceState::Default;
        }
        panel_target.entity = None;
        return;
    }

    // Determine which branch to use based on the active group's type.
    // This ensures that when cycling active groups in mixed unit+structure selections,
    // the command panel updates to reflect the active group (not always the structure).
    let active_type = selection.active_type();
    let active_is_structure = active_type.map(|t| t.is_structure()).unwrap_or(false);

    // Try to find a structure entity from the active group for structure menu rendering
    let active_structure = if active_is_structure {
        selection.active_group().and_then(|g| {
            g.entities.iter().find_map(|&e| selected_structures.get(e).ok())
        })
    } else {
        None
    };

    if let Some((entity, obj_instance, dc_state, _bk_state, ef_state, _st_state, tunnel_state, armory_state)) = active_structure {
        // Active group is a structure — use structure menu branch
        let target_changed = panel_target.entity != Some(entity);
        panel_target.entity = Some(entity);

        match obj_instance.object_type {
            ObjectEnum::DeploymentCenter => {
                if let Some(_dc) = dc_state {
                    let in_valid_dc_state = matches!(*interface_state,
                        ObjectInterfaceState::StructureMenu(
                            StructureMenuState::DcIdle |
                            StructureMenuState::DcBuildMenu |
                            StructureMenuState::DcConstructing |
                            StructureMenuState::DcReadyToPlace |
                            StructureMenuState::DcAwaitingPlacement
                        )
                    );
                    if target_changed || !in_valid_dc_state {
                        *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
                    }
                }
            }
            ObjectEnum::Barracks => {
                let in_valid_state = matches!(*interface_state,
                    ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu) |
                    ObjectInterfaceState::AwaitingTarget(_)
                );
                if target_changed || !in_valid_state {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);
                }
            }
            ObjectEnum::ExtractionFacility => {
                if let Some(_ef) = ef_state {
                    let in_valid_ef_state = matches!(*interface_state,
                        ObjectInterfaceState::StructureMenu(
                            StructureMenuState::EfIdle |
                            StructureMenuState::EfAwaitingPlacement
                        )
                    );
                    if target_changed || !in_valid_ef_state {
                        *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
                    }
                }
            }
            ObjectEnum::SupplyTower => {
                let in_valid_state = matches!(*interface_state,
                    ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu) |
                    ObjectInterfaceState::AwaitingTarget(_)
                );
                if target_changed || !in_valid_state {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu);
                }
            }
            ObjectEnum::Tunnel => {
                if tunnel_state.is_some() {
                    let in_tunnel_state = matches!(*interface_state,
                        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle) |
                        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu) |
                        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu) |
                        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement)
                    );
                    if target_changed || !in_tunnel_state {
                        *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
                    }
                }
            }
            ObjectEnum::Headquarters => {
                let in_valid_state = matches!(*interface_state,
                    ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu) |
                    ObjectInterfaceState::AwaitingTarget(_)
                );
                if target_changed || !in_valid_state {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
                }
            }
            ObjectEnum::RecruitmentCenter => {
                let in_valid_state = matches!(*interface_state,
                    ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu) |
                    ObjectInterfaceState::AwaitingTarget(_)
                );
                if target_changed || !in_valid_state {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu);
                }
            }
            ObjectEnum::CultsArmory => {
                if armory_state.is_some() {
                    let in_valid_state = matches!(*interface_state,
                        ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu) |
                        ObjectInterfaceState::AwaitingTarget(_)
                    );
                    if target_changed || !in_valid_state {
                        *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
                    }
                }
            }
            // PowerPlant, ExtractionPlate — no active commands
            _ => {
                let new_state = ObjectInterfaceState::StructureMenu(StructureMenuState::Inert);
                if *interface_state != new_state || target_changed {
                    *interface_state = new_state;
                }
            }
        }
    } else {
        // Active group is NOT a structure — handle units, agents, or empty selection
        let unit_count = selected_units.iter().count();
        if unit_count > 0 {
            // Compute capabilities from active group's entities only (not all selected)
            let new_caps = compute_selected_unit_capabilities(&selected_units, &selection);
            if *unit_caps != new_caps {
                *unit_caps = new_caps;
            }
            // Preserve ownership flag — compute_selected_unit_capabilities doesn't track it
            if unit_caps.owned_by_local_player != owned {
                unit_caps.owned_by_local_player = owned;
            }

            // Check if active group is Agents (for Agent-specific interface)
            let active_is_agent = active_type == Some(ObjectEnum::SyndicateAgent);

            let active_is_cults_recruit = active_type == Some(ObjectEnum::CultsRecruit);

            if active_is_agent {
                // Route to AgentMenu state
                if !matches!(*interface_state, ObjectInterfaceState::AgentMenu(_) | ObjectInterfaceState::AwaitingTarget(_)) {
                    *interface_state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
                    panel_target.entity = None;
                }
            } else if active_is_cults_recruit {
                // Route to CultsRecruitMenu state
                if !matches!(*interface_state, ObjectInterfaceState::CultsRecruitMenu(_) | ObjectInterfaceState::AwaitingTarget(_)) {
                    *interface_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault);
                    panel_target.entity = None;
                }
            } else {
                // Stay in Default (unit commands) or AwaitingTarget
                if matches!(*interface_state, ObjectInterfaceState::StructureMenu(_) | ObjectInterfaceState::AgentMenu(_) | ObjectInterfaceState::CultsRecruitMenu(_)) {
                    *interface_state = ObjectInterfaceState::Default;
                    panel_target.entity = None;
                }
            }
        } else if !matches!(*interface_state, ObjectInterfaceState::Default) || panel_target.entity.is_some() {
            *interface_state = ObjectInterfaceState::Default;
            panel_target.entity = None;
            // Reset capabilities when no units selected
            if *unit_caps != SelectedUnitCapabilities::default() {
                *unit_caps = SelectedUnitCapabilities::default();
            }
        }
    }
}
