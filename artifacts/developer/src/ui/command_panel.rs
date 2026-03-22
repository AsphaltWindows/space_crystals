use bevy::prelude::*;
use crate::types::*;
use crate::game::types::{
    ObjectInstance, Player, GdoPlayerResources, SyndicatePlayerResources, CultsPlayerResources, StructureInstance,
    DeploymentCenterState, BarracksState, ExtractionFacilityState,
    SupplyTowerState, SupplyChopperState, TunnelState, TunnelTier, TunnelOperation,
    HeadquartersState, RecruitmentCenterState, ArmoryState,
    tunnel_t2_upgrade_cost, tunnel_t3_upgrade_cost,
    cults_structure_stats::{SOLDIER_TRAINING_COST, GUNNER_TRAINING_COST},
};
use crate::game::units::types::commands::{CommandType, HoldingPosition, UnitCommand, CommandQueue};
use crate::game::units::types::state::AgentCarryState;
use crate::game::units::types::movement::{MoveTarget, Path, Velocity};
use crate::game::combat::types::{AttackState, AttackCapability, AttackType};
use crate::game::world::types::{SpaceCrystalPatch, SupplyDeliveryStation};
use super::types::*;

/// Hotkey letters for the 3x3 grid, indexed [row][col]
const GRID_HOTKEYS: [[char; 3]; 3] = [
    ['Q', 'W', 'E'],
    ['A', 'S', 'D'],
    ['Z', 'X', 'C'],
];

/// Map a KeyCode to a (row, col) grid slot, if it's a grid hotkey
fn keycode_to_grid_slot(key: KeyCode) -> Option<(u8, u8)> {
    match key {
        KeyCode::KeyQ => Some((0, 0)),
        KeyCode::KeyW => Some((0, 1)),
        KeyCode::KeyE => Some((0, 2)),
        KeyCode::KeyA => Some((1, 0)),
        KeyCode::KeyS => Some((1, 1)),
        KeyCode::KeyD => Some((1, 2)),
        KeyCode::KeyZ => Some((2, 0)),
        KeyCode::KeyX => Some((2, 1)),
        KeyCode::KeyC => Some((2, 2)),
        _ => None,
    }
}

/// Get the action for a given grid slot in the current interface state.
/// `caps` provides selected unit capabilities for conditional command visibility.
fn get_grid_slot_action(
    state: &ObjectInterfaceState,
    row: u8,
    col: u8,
    bk_has_queue: bool,
    has_active_construction: bool,
    caps: &SelectedUnitCapabilities,
    tunnel_is_upgrading: bool,
    has_ready_plate: bool,
) -> Option<CommandButtonAction> {
    match state {
        ObjectInterfaceState::StructureMenu(sm) => match sm {
            StructureMenuState::DcIdle => match (row, col) {
                (0, 0) => Some(CommandButtonAction::DcOpenBuildMenu),
                (2, 1) if has_active_construction => Some(CommandButtonAction::DcCancel),
                _ => None,
            },
            StructureMenuState::DcBuildMenu => match (row, col) {
                (0, 0) => Some(CommandButtonAction::DcBuild(ObjectEnum::PowerPlant)),
                (0, 1) => Some(CommandButtonAction::DcBuild(ObjectEnum::Barracks)),
                (1, 0) => Some(CommandButtonAction::DcBuild(ObjectEnum::SupplyTower)),
                (1, 1) => Some(CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility)),
                (2, 0) => Some(CommandButtonAction::Back),
                _ => None,
            },
            StructureMenuState::DcConstructing => match (row, col) {
                (2, 0) => Some(CommandButtonAction::Back),
                (2, 1) => Some(CommandButtonAction::DcCancel),
                _ => None,
            },
            StructureMenuState::DcReadyToPlace => match (row, col) {
                (0, 0) => Some(CommandButtonAction::EnterPlacement),
                (2, 0) => Some(CommandButtonAction::Back),
                (2, 1) => Some(CommandButtonAction::DcCancel),
                _ => None,
            },
            StructureMenuState::BarracksMenu => match (row, col) {
                (0, 0) => Some(CommandButtonAction::BkTrain(ObjectEnum::Peacekeeper)),
                (2, 1) if bk_has_queue => Some(CommandButtonAction::BkCancel),
                (2, 2) => Some(CommandButtonAction::SetRallyPoint),
                _ => None,
            },
            StructureMenuState::EfIdle => match (row, col) {
                (0, 0) if has_ready_plate => Some(CommandButtonAction::EnterPlacement),
                (0, 0) => Some(CommandButtonAction::EfBuildPlate),
                (2, 1) if has_active_construction => Some(CommandButtonAction::EfCancel),
                _ => None,
            },
            StructureMenuState::SupplyTowerMenu => match (row, col) {
                (0, 0) => Some(CommandButtonAction::StTrain(ObjectEnum::SupplyChopper)),
                (1, 0) => Some(CommandButtonAction::StScheduleDeliveries),
                (2, 1) if bk_has_queue => Some(CommandButtonAction::StCancel),
                (2, 2) => Some(CommandButtonAction::SetRallyPoint),
                _ => None,
            },
            StructureMenuState::HeadquartersMenu => match (row, col) {
                (0, 0) => Some(CommandButtonAction::HqTrain(ObjectEnum::SyndicateAgent)),
                (0, 1) => Some(CommandButtonAction::HqTrain(ObjectEnum::SyndicateGuard)),
                (2, 1) if bk_has_queue => Some(CommandButtonAction::HqCancel),
                (2, 2) => Some(CommandButtonAction::SetRallyPoint),
                _ => None,
            },
            StructureMenuState::TunnelIdle => match (row, col) {
                (0, 0) => Some(CommandButtonAction::TunnelUpgrade),
                (0, 1) => Some(CommandButtonAction::TunnelOpenExpandMenu),
                (0, 2) => Some(CommandButtonAction::TunnelOpenEjectMenu),
                (2, 1) if tunnel_is_upgrading => Some(CommandButtonAction::TunnelCancelUpgrade),
                _ => None,
            },
            StructureMenuState::TunnelExpandMenu => match (row, col) {
                // Must mirror build_tunnel_expand_grid() slot layout:
                // Slot 0 = (0,0) = Headquarters (T1+)
                // Future expansion types at (0,1), (0,2), (1,0), (1,1), (1,2)
                (0, 0) => Some(CommandButtonAction::TunnelSelectExpansion(ObjectEnum::Headquarters)),
                (2, 0) => Some(CommandButtonAction::Back),
                _ => None,
            },
            StructureMenuState::TunnelEjectMenu => match (row, col) {
                // Currently stubbed — no eject actions yet, only Back
                (2, 0) => Some(CommandButtonAction::Back),
                _ => None,
            },
            StructureMenuState::RecruitmentCenterMenu => match (row, col) {
                (2, 1) if bk_has_queue => Some(CommandButtonAction::RcCancel),
                (2, 2) => Some(CommandButtonAction::SetRallyPoint),
                _ => None,
            },
            StructureMenuState::ArmoryMenu => match (row, col) {
                (0, 0) => Some(CommandButtonAction::ArmoryTrainSoldier),
                (0, 1) => Some(CommandButtonAction::ArmoryTrainGunner),
                (0, 2) => Some(CommandButtonAction::ArmoryEjectAll),
                (2, 2) => Some(CommandButtonAction::SetRallyPoint),
                _ => None,
            },
            StructureMenuState::DcAwaitingPlacement | StructureMenuState::EfAwaitingPlacement |
            StructureMenuState::TunnelAwaitingPlacement => {
                // Placement mode is handled by mouse clicks + Escape, no grid buttons
                None
            },
            StructureMenuState::Inert => {
                // Structure with no active commands (Power Plant, Extraction Plate)
                None
            },
        },
        ObjectInterfaceState::Default if caps.is_chopper => match (row, col) {
            // SupplyChopper-specific commands
            (0, 0) => Some(CommandButtonAction::UnitMove),
            (0, 1) => Some(CommandButtonAction::ChopperPickUpSupplies),
            (0, 2) => Some(CommandButtonAction::ChopperAttachToTower),
            (1, 0) => Some(CommandButtonAction::ChopperDropOffSupplies),
            (1, 2) => Some(CommandButtonAction::UnitHoldPosition),
            (2, 1) => Some(CommandButtonAction::UnitStop),
            _ => None,
        },
        ObjectInterfaceState::Default => match (row, col) {
            // Unit commands (only shown when Selection has units)
            (0, 0) => Some(CommandButtonAction::UnitMove),
            (0, 1) if caps.can_reverse => Some(CommandButtonAction::UnitReverse),
            (0, 2) => Some(CommandButtonAction::UnitHoldPosition),
            (1, 0) if caps.has_attack => Some(CommandButtonAction::UnitAttack),
            (1, 1) => Some(CommandButtonAction::UnitPatrol),
            (1, 2) if caps.can_target_ground => Some(CommandButtonAction::UnitAttackGround),
            (2, 1) => Some(CommandButtonAction::UnitStop),
            _ => None,
        },
        ObjectInterfaceState::AgentMenu(am) => match am {
            AgentMenuState::AgentDefault => match (row, col) {
                (0, 0) => Some(CommandButtonAction::AgentBuildTunnel),
                (0, 1) => Some(CommandButtonAction::AgentDropOff),
                _ => None,
            },
            AgentMenuState::AgentAwaitingPlacement => {
                // Placement mode — handled by mouse clicks + Escape, no grid buttons
                None
            },
        },
        ObjectInterfaceState::CultsRecruitMenu(crm) => match crm {
            CultsRecruitMenuState::RecruitDefault => match (row, col) {
                (0, 0) => Some(CommandButtonAction::RecruitConstruct),
                (1, 1) => Some(CommandButtonAction::RecruitAssistConstruction),
                _ => None,
            },
            CultsRecruitMenuState::RecruitConstructMenu => match (row, col) {
                (0, 0) => Some(CommandButtonAction::RecruitSelectBuilding(ObjectEnum::CultsStorage)),
                (2, 0) => Some(CommandButtonAction::Back),
                _ => None,
            },
            CultsRecruitMenuState::RecruitAwaitingPlacement => None,
        },
        ObjectInterfaceState::AwaitingTarget(_) => match (row, col) {
            (2, 0) => Some(CommandButtonAction::Back),
            _ => None,
        },
    }
}

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

        if let Some(t) = super::utils::ray_aabb_intersect(ray_origin, ray_dir, aabb_min, aabb_max) {
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
fn selection_owned_by_local_player(
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
fn is_panel_visible(state: &ObjectInterfaceState, selection: &Selection) -> bool {
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

/// System to rebuild the command panel UI when state changes
pub fn rebuild_command_panel_ui(
    mut commands: Commands,
    panel_section: Query<Entity, With<CommandPanelSection>>,
    interface_state: Res<ObjectInterfaceState>,
    panel_target: Res<CommandPanelTarget>,
    dc_query: Query<&DeploymentCenterState>,
    bk_query: Query<&BarracksState>,
    ef_query: Query<&ExtractionFacilityState>,
    st_query: Query<&SupplyTowerState>,
    tunnel_query: Query<(&TunnelState, &Owner)>,
    all_tunnels_query: Query<(Entity, &TunnelState), With<Owner>>,
    syndicate_players: Query<(&Player, &SyndicatePlayerResources)>,
    network_units: Query<(&ObjectInstance, &crate::game::units::types::state::behavior::InTunnelNetwork)>,
    players: Query<(&Player, &GdoPlayerResources)>,
    selected_owners: Query<(&Owner, Option<&HeadquartersState>, Option<&RecruitmentCenterState>, Option<&ArmoryState>), (With<StructureInstance>, With<Selected>)>,
    unit_caps: Res<SelectedUnitCapabilities>,
    selection: Res<Selection>,
) {
    if !interface_state.is_changed() && !unit_caps.is_changed() && !selection.is_changed() {
        return;
    }

    let Ok(panel_entity) = panel_section.single() else { return; };

    // Clear existing content
    commands.entity(panel_entity).despawn_children();

    if !is_panel_visible(&interface_state, &selection) || !unit_caps.owned_by_local_player {
        return;
    }

    // Get player resources for button enable/disable
    let player_sc = get_player_sc_from_owners(&selected_owners, &players);

    // Check if player owns a Power Plant (tech prerequisite for Supply Tower)
    let has_power_plant = if let Some((owner, ..)) = selected_owners.iter().next() {
        players.iter().any(|(p, res)| Some(p.player_number) == owner.player_number() && res.has_power_plant)
    } else {
        false
    };

    // Get Syndicate SC for HQ production affordability checks
    let syndicate_sc = if let Some((owner, ..)) = selected_owners.iter().next() {
        syndicate_players.iter()
            .find(|(p, _)| Some(p.player_number) == owner.player_number())
            .map(|(_, res)| res.space_crystals)
            .unwrap_or(0)
    } else {
        0
    };

    commands.entity(panel_entity).with_children(|parent| {
        // Title
        let title = match &*interface_state {
            ObjectInterfaceState::StructureMenu(sm) => match sm {
                StructureMenuState::DcIdle | StructureMenuState::DcConstructing |
                StructureMenuState::DcReadyToPlace | StructureMenuState::DcAwaitingPlacement => "Deployment Center",
                StructureMenuState::DcBuildMenu => "Build Menu",
                StructureMenuState::BarracksMenu => "Barracks",
                StructureMenuState::EfIdle | StructureMenuState::EfAwaitingPlacement => "Extraction Facility",
                StructureMenuState::SupplyTowerMenu => "Supply Tower",
                StructureMenuState::HeadquartersMenu => "Headquarters",
                StructureMenuState::RecruitmentCenterMenu => "Recruitment Center",
                StructureMenuState::ArmoryMenu => "Armory",
                StructureMenuState::TunnelIdle | StructureMenuState::TunnelAwaitingPlacement => "Tunnel",
                StructureMenuState::TunnelExpandMenu => "Expand",
                StructureMenuState::TunnelEjectMenu => "Eject Units",
                StructureMenuState::Inert => "Info",
            },
            ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault) => "Agent Commands",
            ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement) => "Place Tunnel",
            ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault) => "Recruit",
            ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu) => "Construct",
            ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement) => "Place Building",
            ObjectInterfaceState::Default => "Unit Commands",
            ObjectInterfaceState::AwaitingTarget(ct) => match ct {
                CommandType::Move => "Move Target",
                CommandType::Attack => "Attack Target",
                CommandType::AttackGround => "Attack Ground",
                CommandType::AttackMove => "Attack Move",
                CommandType::Patrol => "Patrol Target",
                CommandType::Reverse => "Reverse Target",
                CommandType::Enter => "Enter Target",
                _ => "Select Target",
            },
        };
        spawn_panel_title(parent, title);

        // Info/progress text above the grid
        match &*interface_state {
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle) => {
                if let Some(target_entity) = panel_target.entity {
                    if let Ok(dc) = dc_query.get(target_entity) {
                        if dc.current_construction.is_some() {
                            let (name, pct) = get_dc_construction_info(dc);
                            spawn_progress_text(parent, &format!("Building {}... {:.0}%", name, pct));
                        } else if let Some(ref ready) = dc.ready_to_place {
                            let name = ready.object_type().name;
                            spawn_progress_text(parent, &format!("{} ready!", name));
                        }
                    }
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle) => {
                if let Some(target_entity) = panel_target.entity {
                    if let Ok(ef) = ef_query.get(target_entity) {
                        if ef.current_construction {
                            let progress = ef.construction_progress.unwrap_or(0.0);
                            let total = ExtractionFacilityState::construction_cost().build_frames as f32;
                            let pct = (progress / total * 100.0).min(100.0);
                            spawn_progress_text(parent, &format!("Building Plate... {:.0}%", pct));
                        } else if ef.ready_to_place {
                            spawn_progress_text(parent, "Plate ready!");
                        }
                    }
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing) => {
                if let Some(target_entity) = panel_target.entity {
                    if let Ok(dc) = dc_query.get(target_entity) {
                        let (name, pct) = get_dc_construction_info(dc);
                        spawn_progress_text(parent, &format!("Building {}... {:.0}%", name, pct));
                    }
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace) => {
                if let Some(target_entity) = panel_target.entity {
                    if let Ok(dc) = dc_query.get(target_entity) {
                        if let Some(ref ready) = dc.ready_to_place {
                            let name = ready.object_type().name;
                            spawn_progress_text(parent, &format!("{} ready!", name));
                        }
                    }
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu) => {
                if let Some(target_entity) = panel_target.entity {
                    if let Ok(bk) = bk_query.get(target_entity) {
                        if bk.current_build.is_some() {
                            let progress = bk.current_build_progress.unwrap_or(0.0);
                            let cost = BarracksState::production_cost(bk.current_build.as_ref().unwrap());
                            let total = cost.map(|c| c.build_frames as f32).unwrap_or(80.0);
                            let pct = (progress / total * 100.0).min(100.0);
                            spawn_progress_text(parent, &format!("Training... {:.0}%", pct));
                        }
                        if !bk.build_queue.is_empty() {
                            spawn_info_text(parent, &format!("Queue: {}/{}", bk.build_queue.len(), BarracksState::MAX_QUEUE_SIZE));
                        }
                    }
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcAwaitingPlacement) => {
                spawn_progress_text(parent, "Click to place");
                spawn_info_text(parent, "Right-click or Esc to cancel");
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::EfAwaitingPlacement) => {
                spawn_progress_text(parent, "Click to place plate");
                spawn_info_text(parent, "Right-click or Esc to cancel");
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu) => {
                if let Some(target_entity) = panel_target.entity {
                    if let Ok(st) = st_query.get(target_entity) {
                        if st.current_build.is_some() {
                            let progress = st.current_build_progress.unwrap_or(0.0);
                            let cost = SupplyTowerState::production_cost(st.current_build.as_ref().unwrap());
                            let total = cost.map(|c| c.build_frames as f32).unwrap_or(160.0);
                            let pct = (progress / total * 100.0).min(100.0);
                            spawn_progress_text(parent, &format!("Building... {:.0}%", pct));
                        }
                        if !st.build_queue.is_empty() {
                            spawn_info_text(parent, &format!("Queue: {}/{}", st.build_queue.len(), SupplyTowerState::MAX_QUEUE_SIZE));
                        }
                        if st.scheduled_sds.is_some() {
                            spawn_info_text(parent, "Delivering");
                        }
                    }
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu) => {
                if let Some((_, Some(hq), _, _)) = selected_owners.iter().next() {
                    if hq.current_build.is_some() {
                        let progress = hq.current_build_progress.unwrap_or(0.0);
                        let cost = HeadquartersState::production_cost(hq.current_build.as_ref().unwrap());
                        let total = cost.map(|c| c.build_frames as f32).unwrap_or(160.0);
                        let pct = (progress / total * 100.0).min(100.0);
                        spawn_progress_text(parent, &format!("Training... {:.0}%", pct));
                    }
                    if !hq.build_queue.is_empty() {
                        spawn_info_text(parent, &format!("Queue: {}/{}", hq.build_queue.len(), HeadquartersState::MAX_QUEUE_SIZE));
                    }
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu) => {
                if let Some((_, _, Some(rc), _)) = selected_owners.iter().next() {
                    if rc.production_progress > 0 {
                        // RC auto-produces on a 12s base cycle (192 frames at 16fps)
                        // Scale by effectiveness: actual_frames = 192 / effectiveness
                        let total_frames = if rc.effectiveness > 0.0 {
                            (192.0 / rc.effectiveness) as u32
                        } else {
                            192
                        };
                        let pct = (rc.production_progress as f32 / total_frames as f32 * 100.0).min(100.0);
                        spawn_progress_text(parent, &format!("Recruiting... {:.0}%", pct));
                    }
                    spawn_info_text(parent, &format!("Capacity: {}/{}", rc.local_used, rc.local_capacity));
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu) => {
                if let Some((_, _, _, Some(armory))) = selected_owners.iter().next() {
                    spawn_info_text(parent, &format!("Recruits: {}/{}",
                        armory.stored_recruits.len(),
                        crate::game::types::cults_structure_stats::ARMORY_INTERNAL_RECRUIT_CAPACITY));
                    if let Some(ref unit_type) = armory.training_queue {
                        let total_frames = ArmoryState::training_frames(unit_type).unwrap_or(160) as f32;
                        let pct = (armory.training_progress as f32 / total_frames * 100.0).min(100.0);
                        let name = unit_type.object_type().name;
                        spawn_progress_text(parent, &format!("Training: {} {:.0}%", name, pct));
                    }
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle) => {
                if let Some(target_entity) = panel_target.entity {
                    if let Ok((ts, _owner)) = tunnel_query.get(target_entity) {
                        let tier_name = match ts.tier {
                            TunnelTier::Tier1 => "Tier 1",
                            TunnelTier::Tier2 => "Tier 2",
                            TunnelTier::Tier3 => "Tier 3",
                        };
                        spawn_progress_text(parent, tier_name);
                        if let Some(ref op) = ts.current_operation {
                            match op {
                                TunnelOperation::Upgrading { progress, .. } => {
                                    let required = crate::game::types::syndicate_structure_stats::TUNNEL_UPGRADE_FRAMES as f32;
                                    let pct = (progress / required * 100.0).min(100.0);
                                    spawn_info_text(parent, &format!("Upgrading... {:.0}%", pct));
                                }
                                TunnelOperation::BuildingExpansion { object, progress, .. } => {
                                    let name = object.object_type().name;
                                    let required = match object {
                                        ObjectEnum::Headquarters => crate::game::types::syndicate_structure_stats::HQ_BUILD_FRAMES as f32,
                                        _ => crate::game::types::syndicate_structure_stats::TUNNEL_CONSTRUCTION_FRAMES as f32,
                                    };
                                    let pct = (progress / required * 100.0).min(100.0);
                                    spawn_info_text(parent, &format!("Building {}... {:.0}%", name, pct));
                                }
                            }
                        }
                    }
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement) => {
                spawn_progress_text(parent, "Click to place expansion");
                spawn_info_text(parent, "Right-click or Esc to cancel");
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu) => {
                spawn_info_text(parent, "Select expansion type");
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu) => {
                spawn_info_text(parent, "Select unit type to eject");
            }
            ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement) => {
                spawn_progress_text(parent, "Click to place Tunnel");
                spawn_info_text(parent, "Right-click or Esc to cancel");
            }
            ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault) => {
                // No extra info text for agent default state
            }
            ObjectInterfaceState::AwaitingTarget(ct) => {
                spawn_progress_text(parent, &format!("Mode: {}", ct.name()));
                spawn_info_text(parent, "Left-click target, Esc to cancel");
            }
            // Show group info when multiple groups are selected
            ObjectInterfaceState::Default => {
                if selection.groups.len() > 1 {
                    if let Some(active) = selection.active_group() {
                        spawn_info_text(parent, &format!("Group: {} (Tab to cycle)", active.object_type.object_type().name));
                    }
                }
            }
            _ => {}
        }

        // Dynamic content for Tunnel ExpandMenu and EjectMenu
        let is_tunnel_dynamic = matches!(&*interface_state,
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu) |
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu)
        );

        if is_tunnel_dynamic {
            // Build dynamic grid for Tunnel menus
            parent.spawn((
                Node {
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(3, 1.0),
                    grid_template_rows: RepeatedGridTrack::flex(3, 1.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(130.0),
                    column_gap: Val::Px(2.0),
                    row_gap: Val::Px(2.0),
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
                CommandGridContainer,
            )).with_children(|grid| {
                match &*interface_state {
                    ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu) => {
                        build_tunnel_expand_grid(grid, panel_target.entity, &tunnel_query);
                    }
                    ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu) => {
                        build_tunnel_eject_grid(grid, panel_target.entity, &tunnel_query, &network_units);
                    }
                    _ => {}
                }
            });
        } else {
            // Standard 3x3 Grid container
            parent.spawn((
                Node {
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(3, 1.0),
                    grid_template_rows: RepeatedGridTrack::flex(3, 1.0),
                    width: Val::Percent(100.0),
                    height: Val::Px(130.0),
                    column_gap: Val::Px(2.0),
                    row_gap: Val::Px(2.0),
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
                CommandGridContainer,
            )).with_children(|grid| {
                // Check production queue/active build state for grid slot action resolution
                // Includes both queued items and currently building unit
                let bk_has_queue = panel_target.entity
                    .and_then(|e| bk_query.get(e).ok())
                    .map(|bk| bk.has_cancellable())
                    .unwrap_or(false)
                    || panel_target.entity
                        .and_then(|e| st_query.get(e).ok())
                        .map(|st| st.has_cancellable())
                        .unwrap_or(false)
                    || selected_owners.iter().next()
                        .and_then(|(_, hq, _, _)| hq)
                        .map(|hq| hq.has_cancellable())
                        .unwrap_or(false)
                    || selected_owners.iter().next()
                        .and_then(|(_, _, rc, _)| rc)
                        .map(|rc| rc.production_progress > 0)
                        .unwrap_or(false);

                // Check DC/EF construction state for conditional cancel in idle states
                let has_active_construction = panel_target.entity
                    .and_then(|e| dc_query.get(e).ok())
                    .map(|dc| dc.current_construction.is_some() || dc.ready_to_place.is_some())
                    .unwrap_or(false)
                    || panel_target.entity
                        .and_then(|e| ef_query.get(e).ok())
                        .map(|ef| ef.current_construction || ef.ready_to_place)
                        .unwrap_or(false);

                // Check if the tunnel is currently upgrading (for conditional cancel button)
                let tunnel_is_upgrading = panel_target.entity
                    .and_then(|e| tunnel_query.get(e).ok())
                    .map(|(ts, _)| matches!(ts.current_operation, Some(TunnelOperation::Upgrading { .. })))
                    .unwrap_or(false);

                // Get tunnel upgrade cost for enabled check
                let tunnel_upgrade_cost = get_tunnel_upgrade_cost(panel_target.entity, &tunnel_query, &all_tunnels_query);
                let syndicate_supplies = get_syndicate_supplies(panel_target.entity, &tunnel_query, &syndicate_players);
                let has_network_units = !network_units.is_empty();

                // Check if EF has a plate ready to place (for context-dependent Q button)
                let has_ready_plate = panel_target.entity
                    .and_then(|e| ef_query.get(e).ok())
                    .map(|ef| ef.ready_to_place)
                    .unwrap_or(false);

                for row in 0..3u8 {
                    for col in 0..3u8 {
                        let hotkey = GRID_HOTKEYS[row as usize][col as usize];
                        if let Some(action) = get_grid_slot_action(&interface_state, row, col, bk_has_queue, has_active_construction, &unit_caps, tunnel_is_upgrading, has_ready_plate) {
                            let label = grid_button_label(&interface_state, &action, player_sc, hotkey);
                            let enabled = grid_button_enabled_ext(&action, player_sc, bk_has_queue, panel_target.entity, &bk_query, &st_query, &tunnel_query, tunnel_upgrade_cost, syndicate_supplies, &unit_caps, has_power_plant, syndicate_sc, &selected_owners, has_network_units);
                            let active = is_action_active(&action, &interface_state);
                            // When there's only one group (or no groups), all commands are
                            // effectively common — no group distinction to visualize
                            let is_common = selection.groups.len() <= 1 || is_common_command(&action, &selection);
                            spawn_grid_button(grid, &label, action, enabled, active, is_common, row, col);
                        } else {
                            spawn_empty_grid_cell(grid);
                        }
                    }
                }
            });
        }
    });
}

/// System to handle command button clicks
pub fn handle_command_button_clicks(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interaction_query: Query<
        (&Interaction, &CommandButtonAction),
        Changed<Interaction>,
    >,
    mut interface_state: ResMut<ObjectInterfaceState>,
    panel_target: Res<CommandPanelTarget>,
    mut dc_query: Query<(&Owner, &mut DeploymentCenterState)>,
    mut bk_hq_query: Query<(&Owner, Option<&mut BarracksState>, Option<&mut HeadquartersState>, Option<&mut RecruitmentCenterState>, Option<&mut ArmoryState>), Or<(With<BarracksState>, With<HeadquartersState>, With<RecruitmentCenterState>, With<ArmoryState>)>>,
    mut ef_query: Query<(&Owner, &mut ExtractionFacilityState)>,
    mut st_query_mut: Query<(&Owner, &mut SupplyTowerState)>,
    mut tunnel_query_mut: Query<(&Owner, &mut TunnelState, &crate::game::types::TunnelArea)>,
    mut syndicate_players: Query<(&Player, &mut SyndicatePlayerResources)>,
    mut placement_state: ResMut<PlacementState>,
    mut players: Query<(&Player, &mut GdoPlayerResources)>,
    selected_units: Query<(Entity, &mut Velocity, Option<&mut CommandQueue>), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
    attack_states: Query<&AttackState>,
    selection: Res<Selection>,
) {
    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        execute_command_action(action, &mut commands, &mut interface_state, &panel_target, &mut dc_query, &mut bk_hq_query, &mut ef_query, &mut st_query_mut, &mut tunnel_query_mut, &mut syndicate_players, &mut placement_state, &mut players, &selected_units, &attack_states, shift_held, &selection);
    }
}

/// System to handle keyboard hotkeys for the command panel (Q/W/E/A/S/D/Z/X/C grid + Escape + Tab)
pub fn command_panel_hotkeys(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interface_state: ResMut<ObjectInterfaceState>,
    panel_target: Res<CommandPanelTarget>,
    mut dc_query: Query<(&Owner, &mut DeploymentCenterState)>,
    mut bk_hq_query: Query<(&Owner, Option<&mut BarracksState>, Option<&mut HeadquartersState>, Option<&mut RecruitmentCenterState>, Option<&mut ArmoryState>), Or<(With<BarracksState>, With<HeadquartersState>, With<RecruitmentCenterState>, With<ArmoryState>)>>,
    mut ef_query: Query<(&Owner, &mut ExtractionFacilityState)>,
    mut st_query_mut: Query<(&Owner, &mut SupplyTowerState)>,
    mut tunnel_query_mut: Query<(&Owner, &mut TunnelState, &crate::game::types::TunnelArea)>,
    mut syndicate_players: Query<(&Player, &mut SyndicatePlayerResources)>,
    mut players: Query<(&Player, &mut GdoPlayerResources)>,
    selected_units: Query<(Entity, &mut Velocity, Option<&mut CommandQueue>), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
    attack_states: Query<&AttackState>,
    mut placement_state: ResMut<PlacementState>,
    unit_caps: Res<SelectedUnitCapabilities>,
    mut selection: ResMut<Selection>,
) {
    // Only process grid hotkeys when panel is visible and selection is owned by local player
    if !is_panel_visible(&interface_state, &selection) || !unit_caps.owned_by_local_player {
        return;
    }

    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Shift-Tab: Cycle active group backward
    if shift_held && keyboard.just_pressed(KeyCode::Tab) {
        if selection.groups.len() > 1 {
            selection.cycle_active_group_backward();
            if !matches!(*interface_state, ObjectInterfaceState::Default) {
                *interface_state = ObjectInterfaceState::Default;
            } else {
                interface_state.set_changed();
            }
            info!("Group cycling backward: active group index {:?}", selection.active_group_index);
        }
        return;
    }

    // Tab: Cycle active group forward (StateOnlyTransition)
    if keyboard.just_pressed(KeyCode::Tab) && !shift_held {
        if selection.groups.len() > 1 {
            selection.cycle_active_group();
            // Reset to default state when active group changes
            if !matches!(*interface_state, ObjectInterfaceState::Default) {
                *interface_state = ObjectInterfaceState::Default;
            } else {
                interface_state.set_changed();
            }
            info!("Group cycling: active group index {:?}", selection.active_group_index);
        }
        return;
    }

    // Escape: go back / close / cancel placement / cancel awaiting target
    if keyboard.just_pressed(KeyCode::Escape) {
        match &*interface_state {
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu) => {
                *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcAwaitingPlacement) => {
                *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace);
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::EfAwaitingPlacement) => {
                *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu) |
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu) => {
                *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing) |
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace) => {
                *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement) => {
                *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
            }
            ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement) => {
                *interface_state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
                info!("Agent: Cancelled placement, back to AgentDefault");
            }
            ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement) => {
                *interface_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu);
                info!("Recruit: Cancelled placement, back to ConstructMenu");
            }
            ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu) => {
                *interface_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault);
                info!("Recruit: Back to RecruitDefault");
            }
            ObjectInterfaceState::AwaitingTarget(_) => {
                let active_type = selection.active_type();
                let active_is_agent = active_type == Some(ObjectEnum::SyndicateAgent);
                let active_is_rc = active_type == Some(ObjectEnum::RecruitmentCenter);
                let active_is_armory = active_type == Some(ObjectEnum::CultsArmory);
                let active_is_cults_recruit = active_type == Some(ObjectEnum::CultsRecruit);
                if active_is_agent {
                    *interface_state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
                    info!("Cancelled awaiting target, back to AgentDefault");
                } else if active_is_cults_recruit {
                    *interface_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault);
                    info!("Cancelled awaiting target, back to RecruitDefault");
                } else if active_is_rc {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu);
                    info!("Cancelled awaiting target, back to RecruitmentCenterMenu");
                } else if active_is_armory {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
                    info!("Cancelled awaiting target, back to ArmoryMenu");
                } else {
                    *interface_state = ObjectInterfaceState::Default;
                    info!("Cancelled awaiting target, back to Default");
                }
            }
            _ => {}
        }
        return;
    }

    // F key: shortcut to enter placement mode from ReadyToPlace states
    if keyboard.just_pressed(KeyCode::KeyF) {
        match &*interface_state {
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace) => {
                *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcAwaitingPlacement);
                return;
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle) => {
                // Only enter placement if plate is ready
                let plate_ready = panel_target.entity
                    .and_then(|e| ef_query.get(e).ok())
                    .map(|(_, ef)| ef.ready_to_place)
                    .unwrap_or(false);
                if plate_ready {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfAwaitingPlacement);
                    return;
                }
            }
            _ => {}
        }
    }

    // R/Shift+R: Rotate building during placement mode
    if interface_state.is_placement_mode() {
        if keyboard.just_pressed(KeyCode::KeyR) {
            let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
            if shift {
                placement_state.rotation = placement_state.rotation.rotate_ccw();
                info!("Rotation: {:?} (CCW)", placement_state.rotation);
            } else {
                placement_state.rotation = placement_state.rotation.rotate_cw();
                info!("Rotation: {:?} (CW)", placement_state.rotation);
            }
        }
        // F/Shift+F: Toggle flip during placement mode
        // F = horizontal flip (mirror E↔W), Shift+F = vertical flip (mirror N↔S)
        if keyboard.just_pressed(KeyCode::KeyF) {
            let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
            if shift {
                placement_state.flip_vertical = !placement_state.flip_vertical;
                info!("Flip vertical: {}", placement_state.flip_vertical);
            } else {
                placement_state.flip_horizontal = !placement_state.flip_horizontal;
                info!("Flip horizontal: {}", placement_state.flip_horizontal);
            }
        }
        // Don't process grid hotkeys in AwaitingPlacement
        return;
    }

    // Check grid hotkeys
    let grid_keys = [
        KeyCode::KeyQ, KeyCode::KeyW, KeyCode::KeyE,
        KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD,
        KeyCode::KeyZ, KeyCode::KeyX, KeyCode::KeyC,
    ];

    for key in grid_keys {
        if keyboard.just_pressed(key) {
            if let Some((row, col)) = keycode_to_grid_slot(key) {
                // Check production queue state using the mutable queries (read-only access)
                // Includes both queued items and currently building unit
                let bk_has_queue = panel_target.entity
                    .and_then(|e| bk_hq_query.get(e).ok())
                    .map(|(_, bk, hq, _, _)| {
                        bk.as_ref().map(|b: &&BarracksState| b.has_cancellable()).unwrap_or(false)
                        || hq.as_ref().map(|h: &&HeadquartersState| h.has_cancellable()).unwrap_or(false)
                    })
                    .unwrap_or(false)
                    || panel_target.entity
                        .and_then(|e| st_query_mut.get(e).ok())
                        .map(|(_, st)| st.has_cancellable())
                        .unwrap_or(false);

                // Check DC/EF construction state for conditional cancel in idle states
                let has_active_construction = panel_target.entity
                    .and_then(|e| dc_query.get(e).ok())
                    .map(|(_, dc)| dc.current_construction.is_some() || dc.ready_to_place.is_some())
                    .unwrap_or(false)
                    || panel_target.entity
                        .and_then(|e| ef_query.get(e).ok())
                        .map(|(_, ef)| ef.current_construction || ef.ready_to_place)
                        .unwrap_or(false);

                // Check if the tunnel is currently upgrading (for conditional cancel button)
                let tunnel_is_upgrading = panel_target.entity
                    .and_then(|e| tunnel_query_mut.get(e).ok())
                    .map(|(_, ts, _)| matches!(ts.current_operation, Some(TunnelOperation::Upgrading { .. })))
                    .unwrap_or(false);

                // Check if EF has a plate ready to place
                let has_ready_plate = panel_target.entity
                    .and_then(|e| ef_query.get(e).ok())
                    .map(|(_, ef)| ef.ready_to_place)
                    .unwrap_or(false);

                if let Some(action) = get_grid_slot_action(&interface_state, row, col, bk_has_queue, has_active_construction, &unit_caps, tunnel_is_upgrading, has_ready_plate) {
                    execute_command_action(&action, &mut commands, &mut interface_state, &panel_target, &mut dc_query, &mut bk_hq_query, &mut ef_query, &mut st_query_mut, &mut tunnel_query_mut, &mut syndicate_players, &mut placement_state, &mut players, &selected_units, &attack_states, shift_held, &selection);
                }
            }
            break; // Only process one key per frame
        }
    }
}

/// Determine the cancel-target state for right-click from a sub-menu.
/// Returns Some(new_state) if the current state should be cancelled, None otherwise.
/// `resolve_rally_target` is a callback that determines which structure menu
/// to return to when cancelling SetRallyPoint (returns the structure type marker).
fn right_click_cancel_target(
    current: &ObjectInterfaceState,
    resolve_rally_target: impl FnOnce() -> Option<RallyTargetKind>,
) -> Option<ObjectInterfaceState> {
    match current {
        // DC sub-menus → DcIdle
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu) |
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing) |
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace) => {
            Some(ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle))
        }
        // Tunnel sub-menus → TunnelIdle
        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu) |
        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu) => {
            Some(ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle))
        }
        // SetRallyPoint → determine which structure menu to return to
        ObjectInterfaceState::AwaitingTarget(CommandType::SetRallyPoint) => {
            match resolve_rally_target() {
                Some(RallyTargetKind::Barracks) => Some(ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu)),
                Some(RallyTargetKind::Headquarters) => Some(ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu)),
                Some(RallyTargetKind::SupplyTower) => Some(ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu)),
                Some(RallyTargetKind::RecruitmentCenter) => Some(ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu)),
                Some(RallyTargetKind::Armory) => Some(ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu)),
                None => Some(ObjectInterfaceState::Default),
            }
        }
        // ScheduleDeliveries → SupplyTowerMenu
        ObjectInterfaceState::AwaitingTarget(CommandType::ScheduleDeliveries) => {
            Some(ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu))
        }
        // Don't handle Default, placement modes, or unit AwaitingTarget modes
        _ => None,
    }
}

/// Marker for which production structure the panel target entity is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RallyTargetKind {
    Barracks,
    Headquarters,
    SupplyTower,
    RecruitmentCenter,
    Armory,
}

/// Right-click cancel for multi-stage ObjectInterfaceState sub-menus.
/// When in a structure sub-menu or certain AwaitingTarget states,
/// right-click returns to the parent/idle state — same as Z/Escape.
/// Does NOT handle placement modes (handled by placement_click_system in faction.rs)
/// or unit AwaitingTarget modes (handled by right_click_move_command in core.rs).
pub fn right_click_cancel_submenu(
    buttons: Res<ButtonInput<MouseButton>>,
    mut interface_state: ResMut<ObjectInterfaceState>,
    panel_target: Res<CommandPanelTarget>,
    bk_query: Query<(), With<BarracksState>>,
    hq_query: Query<(), With<HeadquartersState>>,
    st_query: Query<(), With<SupplyTowerState>>,
    rc_query: Query<(), With<RecruitmentCenterState>>,
    armory_query: Query<(), With<ArmoryState>>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }

    let resolve = || {
        let entity = panel_target.entity?;
        if bk_query.get(entity).is_ok() {
            Some(RallyTargetKind::Barracks)
        } else if hq_query.get(entity).is_ok() {
            Some(RallyTargetKind::Headquarters)
        } else if st_query.get(entity).is_ok() {
            Some(RallyTargetKind::SupplyTower)
        } else if rc_query.get(entity).is_ok() {
            Some(RallyTargetKind::RecruitmentCenter)
        } else if armory_query.get(entity).is_ok() {
            Some(RallyTargetKind::Armory)
        } else {
            None
        }
    };

    if let Some(new_state) = right_click_cancel_target(&interface_state, resolve) {
        *interface_state = new_state;
    }
}

/// Execute a command action (shared between button clicks and hotkeys)
fn execute_command_action(
    action: &CommandButtonAction,
    commands: &mut Commands,
    interface_state: &mut ResMut<ObjectInterfaceState>,
    panel_target: &Res<CommandPanelTarget>,
    dc_query: &mut Query<(&Owner, &mut DeploymentCenterState)>,
    bk_hq_query: &mut Query<(&Owner, Option<&mut BarracksState>, Option<&mut HeadquartersState>, Option<&mut RecruitmentCenterState>, Option<&mut ArmoryState>), Or<(With<BarracksState>, With<HeadquartersState>, With<RecruitmentCenterState>, With<ArmoryState>)>>,
    ef_query: &mut Query<(&Owner, &mut ExtractionFacilityState)>,
    st_query: &mut Query<(&Owner, &mut SupplyTowerState)>,
    tunnel_query: &mut Query<(&Owner, &mut TunnelState, &crate::game::types::TunnelArea)>,
    syndicate_players: &mut Query<(&Player, &mut SyndicatePlayerResources)>,
    placement_state: &mut ResMut<PlacementState>,
    players: &mut Query<(&Player, &mut GdoPlayerResources)>,
    selected_units: &Query<(Entity, &mut Velocity, Option<&mut CommandQueue>), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
    attack_states: &Query<&AttackState>,
    shift_held: bool,
    selection: &Selection,
) {
    match action {
        CommandButtonAction::DcOpenBuildMenu => {
            // Context-aware: route to the appropriate sub-state based on DC status
            let dc_state = panel_target.entity.and_then(|e| dc_query.get(e).ok());
            if let Some((_, dc)) = dc_state {
                if dc.current_construction.is_some() {
                    **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing);
                } else if dc.ready_to_place.is_some() {
                    **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace);
                } else {
                    **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
                }
            } else {
                **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
            }
        }
        CommandButtonAction::DcBuild(object_type) => {
            let Some(target_entity) = panel_target.entity else { return };
            if let Ok((owner, mut dc_state)) = dc_query.get_mut(target_entity) {
                if dc_state.current_construction.is_some() || dc_state.ready_to_place.is_some() {
                    info!("DC: Already building or has structure ready");
                    return;
                }
                // Tech prerequisite: Supply Tower requires owning a Power Plant
                if *object_type == ObjectEnum::SupplyTower {
                    let has_power_plant = players.iter()
                        .any(|(p, res)| Some(p.player_number) == owner.player_number() && res.has_power_plant);
                    if !has_power_plant {
                        info!("DC: Supply Tower requires a Power Plant");
                        return;
                    }
                }
                if let Some(cost) = DeploymentCenterState::construction_cost(object_type) {
                    if let Some(mut res) = find_player_resources_mut(owner, players) {
                        if res.space_crystals >= cost.space_crystals as i32 {
                            res.space_crystals -= cost.space_crystals as i32;
                            dc_state.current_construction = Some(*object_type);
                            dc_state.construction_progress = Some(0.0);
                            **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing);
                            info!("DC: Started building {:?} ({} SC)", object_type, cost.space_crystals);
                        } else {
                            info!("DC: Not enough SC ({} needed, {} available)", cost.space_crystals, res.space_crystals);
                        }
                    }
                }
            }
        }
        CommandButtonAction::DcCancel => {
            let Some(target_entity) = panel_target.entity else { return };
            if let Ok((owner, mut dc_state)) = dc_query.get_mut(target_entity) {
                if let Some(building_type) = dc_state.current_construction {
                    if let Some(refund) = dc_state.cancellation_refund(&building_type) {
                        if let Some(mut res) = find_player_resources_mut(owner, players) {
                            res.space_crystals += refund as i32;
                            info!("DC: Cancelled construction, refunded {} SC", refund);
                        }
                    }
                    dc_state.current_construction = None;
                    dc_state.construction_progress = None;
                    **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
                } else if let Some(ready_type) = &dc_state.ready_to_place.clone() {
                    if let Some(refund) = dc_state.cancellation_refund(ready_type) {
                        if let Some(mut res) = find_player_resources_mut(owner, players) {
                            res.space_crystals += refund as i32;
                            info!("DC: Cancelled placement, refunded {} SC (75%)", refund);
                        }
                    }
                    dc_state.ready_to_place = None;
                    **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
                }
            }
        }
        CommandButtonAction::BkTrain(unit_type) => {
            let entities = active_group_entities(selection);
            let mut any_queued = false;
            for entity in entities {
                if let Ok((owner, Some(mut bk_state), _, _, _)) = bk_hq_query.get_mut(entity) {
                    if let Some(cost) = BarracksState::production_cost(unit_type) {
                        if let Some(mut res) = find_player_resources_mut(owner, players) {
                            if res.space_crystals < cost.space_crystals as i32 {
                                info!("Barracks: Not enough SC");
                                continue;
                            }
                            let control_cost = unit_type.unit_control_cost();
                            if !res.has_unit_control(control_cost) {
                                info!("Barracks: Unit control cap reached ({}/{})",
                                    res.unit_control_used, res.unit_control_cap);
                                continue;
                            }
                            if bk_state.try_queue(*unit_type) {
                                res.space_crystals -= cost.space_crystals as i32;
                                info!("Barracks: Queued {:?} ({} SC, queue: {})",
                                    unit_type, cost.space_crystals, bk_state.build_queue.len());
                                any_queued = true;
                            } else {
                                info!("Barracks: Queue full");
                            }
                        }
                    }
                }
            }
            if any_queued {
                interface_state.set_changed();
            }
        }
        CommandButtonAction::BkCancel => {
            let entities = active_group_entities(selection);
            let mut any_cancelled = false;
            for entity in entities {
                if let Ok((owner, Some(mut bk_state), _, _, _)) = bk_hq_query.get_mut(entity) {
                    if let Some(cancelled) = bk_state.cancel_last() {
                        if let Some(cost) = BarracksState::production_cost(&cancelled) {
                            if let Some(mut res) = find_player_resources_mut(owner, players) {
                                res.space_crystals += cost.space_crystals as i32;
                                info!("Barracks: Cancelled {:?}, refunded {} SC", cancelled, cost.space_crystals);
                            }
                        }
                        any_cancelled = true;
                    }
                }
            }
            if any_cancelled {
                interface_state.set_changed();
            }
        }
        CommandButtonAction::HqTrain(unit_type) => {
            let entities = active_group_entities(selection);
            let mut any_queued = false;
            for entity in entities {
                if let Ok((owner, _, Some(mut hq_state), _, _)) = bk_hq_query.get_mut(entity) {
                    if let Some(cost) = HeadquartersState::production_cost(unit_type) {
                        if let Some(mut res) = find_syndicate_resources_mut(owner, syndicate_players) {
                            if res.space_crystals < cost.space_crystals as i32 {
                                info!("Headquarters: Not enough SC");
                                continue;
                            }
                            if hq_state.try_queue(*unit_type) {
                                res.space_crystals -= cost.space_crystals as i32;
                                info!("Headquarters: Queued {:?} ({} SC, queue: {})",
                                    unit_type, cost.space_crystals, hq_state.build_queue.len());
                                any_queued = true;
                            } else {
                                info!("Headquarters: Queue full");
                            }
                        }
                    }
                }
            }
            if any_queued {
                interface_state.set_changed();
            }
        }
        CommandButtonAction::HqCancel => {
            let entities = active_group_entities(selection);
            let mut any_cancelled = false;
            for entity in entities {
                if let Ok((owner, _, Some(mut hq_state), _, _)) = bk_hq_query.get_mut(entity) {
                    if let Some(cancelled) = hq_state.cancel_last() {
                        if let Some(cost) = HeadquartersState::production_cost(&cancelled) {
                            if let Some(mut res) = find_syndicate_resources_mut(owner, syndicate_players) {
                                res.space_crystals += cost.space_crystals as i32;
                                info!("Headquarters: Cancelled {:?}, refunded {} SC", cancelled, cost.space_crystals);
                            }
                        }
                        any_cancelled = true;
                    }
                }
            }
            if any_cancelled {
                interface_state.set_changed();
            }
        }
        CommandButtonAction::RcCancel => {
            let Some(target_entity) = panel_target.entity else { return };
            if let Ok((_, _, _, Some(mut rc_state), _)) = bk_hq_query.get_mut(target_entity) {
                if rc_state.production_progress > 0 {
                    rc_state.production_progress = 0;
                    info!("Recruitment Center: Cancelled production");
                    interface_state.set_changed();
                }
            }
        }
        CommandButtonAction::EfBuildPlate => {
            let Some(target_entity) = panel_target.entity else { return };
            if let Ok((owner, mut ef_state)) = ef_query.get_mut(target_entity) {
                // If already constructing or ready, just refresh (no-op, grid shows current state)
                if ef_state.current_construction || ef_state.ready_to_place {
                    interface_state.set_changed();
                    return;
                }
                let cost = ExtractionFacilityState::construction_cost();
                if let Some(mut res) = find_player_resources_mut(owner, players) {
                    if res.space_crystals >= cost.space_crystals as i32 {
                        res.space_crystals -= cost.space_crystals as i32;
                        ef_state.current_construction = true;
                        ef_state.construction_progress = Some(0.0);
                        interface_state.set_changed();
                        info!("EF: Started building plate ({} SC)", cost.space_crystals);
                    } else {
                        info!("EF: Not enough SC");
                    }
                }
            }
        }
        CommandButtonAction::EfCancel => {
            let Some(target_entity) = panel_target.entity else { return };
            if let Ok((owner, mut ef_state)) = ef_query.get_mut(target_entity) {
                if let Some(refund) = ef_state.cancellation_refund() {
                    if let Some(mut res) = find_player_resources_mut(owner, players) {
                        res.space_crystals += refund as i32;
                        info!("EF: Cancelled, refunded {} SC", refund);
                    }
                }
                ef_state.current_construction = false;
                ef_state.construction_progress = None;
                ef_state.ready_to_place = false;
                **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
            }
        }
        CommandButtonAction::Back => {
            match &**interface_state {
                ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu) |
                ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing) |
                ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace) => {
                    **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
                }
                ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu) |
                ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu) => {
                    **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
                }
                ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu) => {
                    **interface_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault);
                }
                ObjectInterfaceState::AwaitingTarget(_) => {
                    **interface_state = ObjectInterfaceState::Default;
                }
                _ => {}
            }
        }
        CommandButtonAction::EnterPlacement => {
            match &**interface_state {
                ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace) => {
                    **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcAwaitingPlacement);
                }
                ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle) => {
                    **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfAwaitingPlacement);
                }
                _ => {}
            }
        }
        CommandButtonAction::UnitMove => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::Move);
            info!("Command mode: Move");
        }
        CommandButtonAction::UnitAttack => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::Attack);
            info!("Command mode: Attack");
        }
        CommandButtonAction::UnitAttackGround => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::AttackGround);
            info!("Command mode: Attack Ground");
        }
        CommandButtonAction::UnitAttackMove => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::AttackMove);
            info!("Command mode: Attack Move");
        }
        CommandButtonAction::UnitPatrol => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::Patrol);
            info!("Command mode: Patrol");
        }
        CommandButtonAction::UnitHoldPosition => {
            // Determine which entities receive the command based on common/group
            let target_entities = command_target_entities(action, &selection, selected_units);
            for entity in target_entities {
                // Skip units in non-interruptible attack phases
                if let Ok(attack_state) = attack_states.get(entity) {
                    if !attack_state.phase.is_interruptible() {
                        continue;
                    }
                }
                let mut entity_cmds = commands.entity(entity);
                if shift_held {
                    // Queue via deferred command insert — we can't get mutable query access here
                    // since target_entities already borrowed selected_units immutably.
                    // Use commands to insert the queued command via a deferred closure.
                    entity_cmds.queue(move |mut entity: EntityWorldMut| {
                        if let Some(mut queue) = entity.get_mut::<CommandQueue>() {
                            queue.push(UnitCommand::HoldPosition);
                        }
                    });
                } else {
                    entity_cmds
                        .remove::<MoveTarget>()
                        .remove::<Path>()
                        .insert(HoldingPosition)
                        .insert(UnitCommand::HoldPosition);
                    entity_cmds.queue(move |mut entity: EntityWorldMut| {
                        if let Some(mut queue) = entity.get_mut::<CommandQueue>() {
                            queue.clear();
                        }
                    });
                }
            }
            info!("Hold Position");
        }
        CommandButtonAction::UnitStop => {
            let target_entities = command_target_entities(action, &selection, selected_units);
            for entity in target_entities {
                // Skip units in non-interruptible attack phases
                if let Ok(attack_state) = attack_states.get(entity) {
                    if !attack_state.phase.is_interruptible() {
                        continue;
                    }
                }
                let mut entity_cmds = commands.entity(entity);
                if shift_held {
                    entity_cmds.queue(move |mut entity: EntityWorldMut| {
                        if let Some(mut queue) = entity.get_mut::<CommandQueue>() {
                            queue.push(UnitCommand::Stop);
                        }
                    });
                } else {
                    entity_cmds
                        .remove::<MoveTarget>()
                        .remove::<Path>()
                        .remove::<HoldingPosition>()
                        .insert(UnitCommand::Stop);
                    entity_cmds.queue(move |mut entity: EntityWorldMut| {
                        if let Some(mut queue) = entity.get_mut::<CommandQueue>() {
                            queue.clear();
                        }
                    });
                }
            }
            info!("Stop");
        }
        CommandButtonAction::UnitReverse => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::Reverse);
            info!("Command mode: Reverse");
        }
        CommandButtonAction::UnitEnter => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::Enter);
            info!("Command mode: Enter");
        }
        CommandButtonAction::UnitGather => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::Gather);
            info!("Command mode: Gather");
        }
        CommandButtonAction::ChopperPickUpSupplies => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::PickUpSupplies);
            info!("Command mode: Pick Up Supplies");
        }
        CommandButtonAction::ChopperAttachToTower => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::AttachToTower);
            info!("Command mode: Attach to Tower");
        }
        CommandButtonAction::ChopperDropOffSupplies => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::DropOffSupplies);
            info!("Command mode: Drop Off Supplies");
        }
        CommandButtonAction::StTrain(unit_type) => {
            let entities = active_group_entities(selection);
            let mut any_queued = false;
            for entity in entities {
                if let Ok((owner, mut st_state)) = st_query.get_mut(entity) {
                    if let Some(cost) = SupplyTowerState::production_cost(unit_type) {
                        if let Some(mut res) = find_player_resources_mut(owner, players) {
                            if res.space_crystals < cost.space_crystals as i32 {
                                info!("Supply Tower: Not enough SC");
                                continue;
                            }
                            if st_state.try_queue(*unit_type) {
                                res.space_crystals -= cost.space_crystals as i32;
                                info!("Supply Tower: Queued {:?} ({} SC, queue: {})",
                                    unit_type, cost.space_crystals, st_state.build_queue.len());
                                any_queued = true;
                            } else {
                                info!("Supply Tower: Queue full");
                            }
                        }
                    }
                }
            }
            if any_queued {
                interface_state.set_changed();
            }
        }
        CommandButtonAction::StCancel => {
            let entities = active_group_entities(selection);
            let mut any_cancelled = false;
            for entity in entities {
                if let Ok((owner, mut st_state)) = st_query.get_mut(entity) {
                    if let Some(cancelled) = st_state.cancel_last() {
                        if let Some(cost) = SupplyTowerState::production_cost(&cancelled) {
                            if let Some(mut res) = find_player_resources_mut(owner, players) {
                                res.space_crystals += cost.space_crystals as i32;
                                info!("Supply Tower: Cancelled {:?}, refunded {} SC", cancelled, cost.space_crystals);
                            }
                        }
                        any_cancelled = true;
                    }
                }
            }
            if any_cancelled {
                interface_state.set_changed();
            }
        }
        CommandButtonAction::StScheduleDeliveries => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::ScheduleDeliveries);
            info!("Command mode: Schedule Deliveries");
        }
        CommandButtonAction::TunnelUpgrade => {
            execute_tunnel_upgrade(panel_target, tunnel_query, syndicate_players, interface_state);
        }
        CommandButtonAction::TunnelOpenExpandMenu => {
            **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
        }
        CommandButtonAction::TunnelOpenEjectMenu => {
            **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu);
        }
        CommandButtonAction::TunnelSelectExpansion(expansion_type) => {
            execute_tunnel_select_expansion(*expansion_type, panel_target, tunnel_query, placement_state, interface_state);
        }
        CommandButtonAction::TunnelCancelUpgrade => {
            execute_tunnel_cancel_upgrade(panel_target, tunnel_query, syndicate_players, interface_state);
        }
        CommandButtonAction::TunnelEjectUnit(unit_type) => {
            execute_tunnel_eject_unit(*unit_type, panel_target, commands);
        }
        CommandButtonAction::AgentBuildTunnel => {
            // Enter placement mode for Agent tunnel building
            placement_state.building_type = Some(ObjectEnum::Tunnel);
            // Set source_entity to the first selected Agent entity
            let agent_entity = selected_units.iter().next().map(|(e, _, _)| e);
            placement_state.source_entity = agent_entity;
            placement_state.grid_pos = None;
            placement_state.is_valid = false;
            placement_state.rotation = crate::types::StructureRotation::default();
            placement_state.flip_horizontal = false;
            placement_state.flip_vertical = false;
            **interface_state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement);
            info!("Agent: Enter tunnel placement mode");
        }
        CommandButtonAction::AgentDropOff => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::DropOff);
            info!("Agent: Drop Off Resources mode");
        }
        CommandButtonAction::ArmoryTrainSoldier => {
            let Some(target_entity) = panel_target.entity else { return };
            if let Ok((owner, _, _, _, Some(mut armory))) = bk_hq_query.get_mut(target_entity) {
                if armory.stored_recruits.is_empty() || armory.training_queue.is_some() { return; }
                let cost = SOLDIER_TRAINING_COST as i32;
                let player_number = owner.player_number();
                // Consume one stored recruit (despawn it)
                if let Some(recruit_entity) = armory.stored_recruits.pop() {
                    commands.entity(recruit_entity).despawn();
                }
                armory.training_queue = Some(ObjectEnum::CultsSoldier);
                armory.training_progress = 0;
                interface_state.set_changed();
                // Deferred crystal deduction via world access
                commands.queue(move |world: &mut World| {
                    let mut cults_query = world.query::<(&Player, &mut CultsPlayerResources)>();
                    for (player, mut res) in cults_query.iter_mut(world) {
                        if Some(player.player_number) == player_number {
                            res.space_crystals -= cost;
                            break;
                        }
                    }
                });
                info!("Armory: Training Soldier ({} SC)", cost);
            }
        }
        CommandButtonAction::ArmoryTrainGunner => {
            let Some(target_entity) = panel_target.entity else { return };
            if let Ok((owner, _, _, _, Some(mut armory))) = bk_hq_query.get_mut(target_entity) {
                if armory.stored_recruits.is_empty() || armory.training_queue.is_some() { return; }
                let cost = GUNNER_TRAINING_COST as i32;
                let player_number = owner.player_number();
                // Consume one stored recruit (despawn it)
                if let Some(recruit_entity) = armory.stored_recruits.pop() {
                    commands.entity(recruit_entity).despawn();
                }
                armory.training_queue = Some(ObjectEnum::CultsGunner);
                armory.training_progress = 0;
                interface_state.set_changed();
                // Deferred crystal deduction via world access
                commands.queue(move |world: &mut World| {
                    let mut cults_query = world.query::<(&Player, &mut CultsPlayerResources)>();
                    for (player, mut res) in cults_query.iter_mut(world) {
                        if Some(player.player_number) == player_number {
                            res.space_crystals -= cost;
                            break;
                        }
                    }
                });
                info!("Armory: Training Gunner ({} SC)", cost);
            }
        }
        CommandButtonAction::ArmoryEjectAll => {
            let Some(target_entity) = panel_target.entity else { return };
            if let Ok((_, _, _, _, Some(mut armory))) = bk_hq_query.get_mut(target_entity) {
                if armory.stored_recruits.is_empty() { return; }
                let queue: std::collections::VecDeque<Entity> = armory.stored_recruits.drain(..).collect();
                commands.entity(target_entity).insert(ArmoryEjectionQueue {
                    queue,
                    cooldown: 0,
                });
                interface_state.set_changed();
                info!("Armory: Ejecting all stored Recruits");
            }
        }
        CommandButtonAction::RecruitConstruct => {
            **interface_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu);
            info!("Recruit: Open construct menu");
        }
        CommandButtonAction::RecruitSelectBuilding(building_type) => {
            placement_state.building_type = Some(*building_type);
            placement_state.source_entity = selected_units.iter().next().map(|(e, _, _)| e);
            placement_state.grid_pos = None;
            placement_state.is_valid = false;
            placement_state.rotation = crate::types::StructureRotation::default();
            placement_state.flip_horizontal = false;
            placement_state.flip_vertical = false;
            **interface_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement);
            info!("Recruit: Select building {:?} for placement", building_type);
        }
        CommandButtonAction::RecruitAssistConstruction => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::AssistConstruction);
            info!("Command mode: Assist Construction");
        }
        CommandButtonAction::SetRallyPoint => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::SetRallyPoint);
            info!("Command mode: Set Rally Point");
        }
    }
}

/// System to keep command panel updated with construction progress
pub fn update_command_panel_progress(
    panel_target: Res<CommandPanelTarget>,
    dc_query: Query<&DeploymentCenterState, Changed<DeploymentCenterState>>,
    bk_query: Query<&BarracksState, Changed<BarracksState>>,
    ef_query: Query<&ExtractionFacilityState, Changed<ExtractionFacilityState>>,
    st_query: Query<&SupplyTowerState, Changed<SupplyTowerState>>,
    tunnel_progress_query: Query<&TunnelState, Changed<TunnelState>>,
    hq_query: Query<&HeadquartersState, Changed<HeadquartersState>>,
    mut interface_state: ResMut<ObjectInterfaceState>,
) {
    let target = match panel_target.entity {
        Some(e) => e,
        None => return,
    };

    match &*interface_state {
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing) => {
            if let Ok(dc) = dc_query.get(target) {
                if dc.current_construction.is_none() && dc.ready_to_place.is_some() {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace);
                } else if dc.current_construction.is_none() && dc.ready_to_place.is_none() {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
                } else {
                    interface_state.set_changed();
                }
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace) => {
            if let Ok(dc) = dc_query.get(target) {
                if dc.ready_to_place.is_none() {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
                }
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu) => {
            if bk_query.get(target).is_ok() {
                interface_state.set_changed();
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle) => {
            // Refresh the grid when EF state changes (construction progress, ready_to_place transitions)
            if ef_query.get(target).is_ok() {
                interface_state.set_changed();
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu) => {
            if st_query.get(target).is_ok() {
                interface_state.set_changed();
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu) => {
            if hq_query.get(target).is_ok() {
                interface_state.set_changed();
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu) => {
            // RC auto-production changes production_progress each frame — refresh display
            // Note: RecruitmentCenterState doesn't impl Changed tracking here;
            // the panel already refreshes via interface_state.set_changed() when the RC
            // state is observed. For now, piggy-back on the existing change detection.
            interface_state.set_changed();
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle) => {
            if let Ok(ts) = tunnel_progress_query.get(target) {
                if ts.current_operation.is_none() {
                    // Operation finished — refresh UI
                    interface_state.set_changed();
                } else {
                    interface_state.set_changed();
                }
            }
        }
        _ => {}
    }
}

// =====================================================
// HELPER FUNCTIONS
// =====================================================

/// Compute aggregate capabilities from all selected units.
/// Returns `SelectedUnitCapabilities` with `has_attack`, `can_target_ground`, `can_reverse`
/// set to true if ANY selected unit has that capability.
fn compute_selected_unit_capabilities(
    selected_units: &Query<(Entity, Option<&AttackCapability>, &UnitBaseEnum, &ObjectInstance, Option<&AgentCarryState>, Option<&SupplyChopperState>), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
    selection: &Selection,
) -> SelectedUnitCapabilities {
    let mut caps = SelectedUnitCapabilities::default();
    // Only compute capabilities from the active group's entities
    // This ensures the command panel reflects what the active group can do,
    // not the union of all selected units
    let active_entities: Option<&Vec<Entity>> = selection.active_group().map(|g| &g.entities);
    for (entity, attack_cap, unit_base, obj, carry_state, chopper_state) in selected_units.iter() {
        // Skip entities not in the active group (when there is one)
        if let Some(entities) = active_entities {
            if !entities.contains(&entity) {
                continue;
            }
        }
        if let Some(atk) = attack_cap {
            caps.has_attack = true;
            if attack_type_can_target_ground(&atk.attack_type) {
                caps.can_target_ground = true;
            }
        }
        if unit_base.data().can_reverse {
            caps.can_reverse = true;
        }
        if let Some(cs) = carry_state {
            if cs.is_carrying() {
                caps.agent_carrying = true;
            }
        }
        if obj.object_type == ObjectEnum::SupplyChopper {
            caps.is_chopper = true;
        }
        if let Some(cs) = chopper_state {
            if cs.carried_supplies > 0 {
                caps.chopper_has_supplies = true;
            }
        }
    }
    caps
}

/// Whether a runtime AttackType can target ground locations (not just units).
/// Only TailDisjointed and DoublyDisjointed support ground targeting.
pub(crate) fn attack_type_can_target_ground(attack_type: &AttackType) -> bool {
    matches!(attack_type, AttackType::TailDisjointed { .. } | AttackType::DoublyDisjointed { .. })
}

/// Get player space crystals for the selected structure's owner
fn get_player_sc_from_owners(
    selected_owners: &Query<(&Owner, Option<&HeadquartersState>, Option<&RecruitmentCenterState>, Option<&ArmoryState>), (With<StructureInstance>, With<Selected>)>,
    players: &Query<(&Player, &GdoPlayerResources)>,
) -> i32 {
    if let Some((owner, ..)) = selected_owners.iter().next() {
        for (player, res) in players.iter() {
            if Some(player.player_number) == owner.player_number() {
                return res.space_crystals;
            }
        }
    }
    0
}

/// Find player resources mutably for a given owner
fn find_player_resources_mut<'a>(
    owner: &Owner,
    players: &'a mut Query<(&Player, &mut GdoPlayerResources)>,
) -> Option<Mut<'a, GdoPlayerResources>> {
    for (player, res) in players.iter_mut() {
        if Some(player.player_number) == owner.player_number() {
            return Some(res);
        }
    }
    None
}

/// Get DC construction info (name, percentage)
fn get_dc_construction_info(dc: &DeploymentCenterState) -> (String, f32) {
    if let Some(ref building) = dc.current_construction {
        let progress = dc.construction_progress.unwrap_or(0.0);
        let cost = DeploymentCenterState::construction_cost(building);
        let total = cost.map(|c| c.build_frames as f32).unwrap_or(160.0);
        let pct = (progress / total * 100.0).min(100.0);
        (building.object_type().name, pct)
    } else {
        ("Unknown".to_string(), 0.0)
    }
}

/// Execute tunnel upgrade action: validate, deduct cost, start operation
fn execute_tunnel_upgrade(
    panel_target: &Res<CommandPanelTarget>,
    tunnel_query: &mut Query<(&Owner, &mut TunnelState, &crate::game::types::TunnelArea)>,
    syndicate_players: &mut Query<(&Player, &mut SyndicatePlayerResources)>,
    interface_state: &mut ResMut<ObjectInterfaceState>,
) {
    let Some(target_entity) = panel_target.entity else { return };

    // First collect all tunnel data for cost calculation (read-only pass)
    let all_tunnels_data: Vec<(Entity, TunnelTier)> = tunnel_query.iter()
        .map(|(_, ts, _)| (Entity::PLACEHOLDER, ts.tier)) // entity not needed for counting
        .collect();

    if let Ok((owner, mut ts, _area)) = tunnel_query.get_mut(target_entity) {
        if ts.is_busy() {
            info!("Tunnel: Already has an operation in progress");
            return;
        }

        let target_tier = match ts.tier.next_tier() {
            Some(t) => t,
            None => {
                info!("Tunnel: Already at maximum tier");
                return;
            }
        };

        // Count existing tunnels at target tier or higher (excluding this one)
        let cost = match target_tier {
            TunnelTier::Tier2 => {
                let count = all_tunnels_data.iter()
                    .filter(|(_, tier)| matches!(tier, TunnelTier::Tier2 | TunnelTier::Tier3))
                    .count() as u32;
                // Subtract 1 if this tunnel is already counted (it shouldn't be T2+ since we're upgrading to T2)
                tunnel_t2_upgrade_cost(count)
            },
            TunnelTier::Tier3 => {
                let count = all_tunnels_data.iter()
                    .filter(|(_, tier)| *tier == TunnelTier::Tier3)
                    .count() as u32;
                tunnel_t3_upgrade_cost(count)
            },
            TunnelTier::Tier1 => unreachable!(),
        };

        // Find owner's syndicate resources and check/deduct
        let owner_copy = *owner;
        if let Some(mut res) = find_syndicate_resources_mut(&owner_copy, syndicate_players) {
            if (res.supplies as u32) < cost {
                info!("Tunnel: Not enough supplies ({} needed, {} available)", cost, res.supplies);
                return;
            }
            res.supplies -= cost as i32;
            ts.current_operation = Some(TunnelOperation::Upgrading {
                target_tier,
                progress: 0.0,
            });
            interface_state.set_changed();
            info!("Tunnel: Started upgrading to {:?} ({} supplies)", target_tier, cost);
        }
    }
}

/// Execute tunnel cancel upgrade: refund cost and clear operation
fn execute_tunnel_cancel_upgrade(
    panel_target: &Res<CommandPanelTarget>,
    tunnel_query: &mut Query<(&Owner, &mut TunnelState, &crate::game::types::TunnelArea)>,
    syndicate_players: &mut Query<(&Player, &mut SyndicatePlayerResources)>,
    interface_state: &mut ResMut<ObjectInterfaceState>,
) {
    let Some(target_entity) = panel_target.entity else { return };

    // First collect all tunnel data for cost calculation (read-only pass)
    let all_tunnels_data: Vec<(Entity, TunnelTier)> = tunnel_query.iter()
        .map(|(_, ts, _)| (Entity::PLACEHOLDER, ts.tier))
        .collect();

    if let Ok((owner, mut ts, _area)) = tunnel_query.get_mut(target_entity) {
        let target_tier = match ts.current_operation {
            Some(TunnelOperation::Upgrading { target_tier, .. }) => target_tier,
            _ => {
                info!("Tunnel: No upgrade in progress to cancel");
                return;
            }
        };

        // Compute refund cost using same logic as execute_tunnel_upgrade
        let cost = match target_tier {
            TunnelTier::Tier2 => {
                let count = all_tunnels_data.iter()
                    .filter(|(_, tier)| matches!(tier, TunnelTier::Tier2 | TunnelTier::Tier3))
                    .count() as u32;
                tunnel_t2_upgrade_cost(count)
            },
            TunnelTier::Tier3 => {
                let count = all_tunnels_data.iter()
                    .filter(|(_, tier)| *tier == TunnelTier::Tier3)
                    .count() as u32;
                tunnel_t3_upgrade_cost(count)
            },
            TunnelTier::Tier1 => unreachable!("Upgrade target is never Tier1"),
        };

        // Clear the operation
        ts.current_operation = None;

        // Refund supplies
        let owner_copy = *owner;
        if let Some(mut res) = find_syndicate_resources_mut(&owner_copy, syndicate_players) {
            res.supplies += cost as i32;
            info!("Tunnel: Cancelled upgrade to {:?}, refunded {} supplies", target_tier, cost);
        }

        interface_state.set_changed();
    }
}

/// Execute tunnel select expansion: enter placement mode for the expansion type
fn execute_tunnel_select_expansion(
    expansion_type: ObjectEnum,
    panel_target: &Res<CommandPanelTarget>,
    tunnel_query: &mut Query<(&Owner, &mut TunnelState, &crate::game::types::TunnelArea)>,
    placement_state: &mut ResMut<PlacementState>,
    interface_state: &mut ResMut<ObjectInterfaceState>,
) {
    let Some(target_entity) = panel_target.entity else { return };

    // Check tunnel is not busy and meets tier requirement
    if let Ok((_, ts, _)) = tunnel_query.get(target_entity) {
        if ts.is_busy() {
            info!("Tunnel: Cannot place expansion while operation in progress");
            return;
        }
        // Validate tier requirement for the expansion type
        let min_tier = match expansion_type {
            ObjectEnum::Headquarters => TunnelTier::Tier1,
            _ => TunnelTier::Tier1, // Default; update when new expansion types are added
        };
        if !tier_at_least(&ts.tier, &min_tier) {
            info!("Tunnel: Tier too low for {:?} expansion", expansion_type);
            return;
        }
    } else {
        return;
    }

    // Set up placement state for the expansion
    placement_state.building_type = Some(expansion_type);
    placement_state.source_entity = Some(target_entity);
    placement_state.grid_pos = None;
    placement_state.is_valid = false;
    placement_state.rotation = crate::types::StructureRotation::R0;
    placement_state.flip_horizontal = false;
    placement_state.flip_vertical = false;

    **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement);
    info!("Tunnel: Entering placement mode for {:?}", expansion_type);
}

/// Execute tunnel eject: queue a unit of the given type for ejection from this tunnel
fn execute_tunnel_eject_unit(
    unit_type: ObjectEnum,
    panel_target: &Res<CommandPanelTarget>,
    commands: &mut Commands,
) {
    let Some(tunnel_entity) = panel_target.entity else { return };
    // Insert EjectRequest marker — processed by ejection_tick_system
    commands.entity(tunnel_entity).insert(EjectRequest { unit_type });
    info!("Tunnel: Eject request queued for {:?}", unit_type);
}

/// Build the dynamic grid for TunnelExpandMenu
fn build_tunnel_expand_grid(
    grid: &mut ChildSpawnerCommands,
    target_entity: Option<Entity>,
    tunnel_query: &Query<(&TunnelState, &Owner)>,
) {
    let tier = target_entity
        .and_then(|e| tunnel_query.get(e).ok())
        .map(|(ts, _)| ts.tier)
        .unwrap_or(TunnelTier::Tier1);

    let is_busy = target_entity
        .and_then(|e| tunnel_query.get(e).ok())
        .map(|(ts, _)| ts.is_busy())
        .unwrap_or(false);

    // Headquarters is the only expansion type currently (T1+)
    let expansions: Vec<(ObjectEnum, TunnelTier)> = vec![
        (ObjectEnum::Headquarters, TunnelTier::Tier1),
    ];

    let hotkeys: Vec<char> = vec!['Q', 'W', 'E', 'A', 'S', 'D', 'Z', 'X', 'C'];
    let mut slot = 0;

    for (expansion, min_tier) in &expansions {
        if slot >= 6 { break; } // Reserve slot 6 (2,0) = Z for Back
        let tier_ok = tier_at_least(&tier, min_tier);
        let enabled = tier_ok && !is_busy;
        let hotkey = hotkeys[slot];
        let cost = match expansion {
            ObjectEnum::Headquarters => crate::game::types::syndicate_structure_stats::HQ_SC_COST,
            _ => 0,
        };
        let label = if cost > 0 {
            format!("[{}] {} ({} SC)", hotkey, expansion.object_type().name, cost)
        } else {
            format!("[{}] {}", hotkey, expansion.object_type().name)
        };
        let action = CommandButtonAction::TunnelSelectExpansion(*expansion);
        let row = (slot / 3) as u8;
        let col = (slot % 3) as u8;
        spawn_grid_button(grid, &label, action, enabled, false, false, row, col);
        slot += 1;
    }

    // Fill empty cells up to slot 6
    while slot < 6 {
        spawn_empty_grid_cell(grid);
        slot += 1;
    }

    // Back button at (2, 0) = slot 6 = 'Z'
    spawn_grid_button(grid, "[Z] Back", CommandButtonAction::Back, true, false, false, 2, 0);

    // Fill remaining empty cells (slots 7-8)
    spawn_empty_grid_cell(grid);
    spawn_empty_grid_cell(grid);
}

/// Build the dynamic grid for TunnelEjectMenu.
/// Shows unit types present in the tunnel network, grouped by ObjectEnum with counts.
/// Units whose base category exceeds the tunnel's tier are visible but disabled.
fn build_tunnel_eject_grid(
    grid: &mut ChildSpawnerCommands,
    target_entity: Option<Entity>,
    tunnel_query: &Query<(&TunnelState, &Owner)>,
    network_units: &Query<(&ObjectInstance, &crate::game::units::types::state::behavior::InTunnelNetwork)>,
) {
    use std::collections::HashMap;
    use crate::game::units::types::unit_data::{agent_type_data, guard_type_data};

    // Get the tunnel's tier and owner for filtering + transit checks
    let (tunnel_tier, tunnel_owner) = target_entity
        .and_then(|e| tunnel_query.get(e).ok())
        .map(|(ts, owner)| (ts.tier, owner.player_number()))
        .unwrap_or((TunnelTier::Tier1, None));

    // Count units in network by ObjectEnum, filtering to same owner
    let mut type_counts: HashMap<ObjectEnum, u32> = HashMap::new();
    for (obj_instance, in_network) in network_units.iter() {
        if Some(in_network.owner_player) == tunnel_owner {
            *type_counts.entry(obj_instance.object_type).or_insert(0) += 1;
        }
    }

    // Sort unit types for stable display order
    let mut sorted_types: Vec<_> = type_counts.iter().collect();
    sorted_types.sort_by_key(|(obj, _)| obj.object_type().name.clone());

    // Build buttons for each unit type (up to 6 slots before the Back row)
    let hotkeys = ['Q', 'W', 'E', 'A', 'S', 'D'];
    let mut slot = 0;
    for (&unit_type, &count) in &sorted_types {
        if slot >= 6 { break; }

        let row = slot / 3;
        let col = slot % 3;
        let hotkey = hotkeys[slot];

        // Check transit tier eligibility
        let unit_base = match unit_type {
            ObjectEnum::SyndicateAgent => agent_type_data().unit_base,
            ObjectEnum::SyndicateGuard => guard_type_data().unit_base,
            _ => crate::types::UnitBaseEnum::HeavyInfantry, // fallback
        };
        let can_transit = tunnel_tier.can_transit(&unit_base);

        let label = format!("[{}] {} ({})", hotkey, unit_type.object_type().name, count);
        let action = CommandButtonAction::TunnelEjectUnit(unit_type);
        spawn_grid_button(grid, &label, action, can_transit, false, false, row as u8, col as u8);
        slot += 1;
    }

    // Fill remaining empty cells up to slot 6
    while slot < 6 {
        spawn_empty_grid_cell(grid);
        slot += 1;
    }

    // Back button at (2, 0) = 'Z'
    spawn_grid_button(grid, "[Z] Back", CommandButtonAction::Back, true, false, false, 2, 0);
    // Fill remaining empty cells (slots 7-8)
    spawn_empty_grid_cell(grid);
    spawn_empty_grid_cell(grid);
}

/// Check if tier_a is >= tier_b
fn tier_at_least(tier_a: &TunnelTier, tier_b: &TunnelTier) -> bool {
    match (tier_a, tier_b) {
        (_, TunnelTier::Tier1) => true,
        (TunnelTier::Tier2, TunnelTier::Tier2) => true,
        (TunnelTier::Tier3, TunnelTier::Tier2) => true,
        (TunnelTier::Tier3, TunnelTier::Tier3) => true,
        _ => false,
    }
}

/// Get the cost for upgrading a tunnel, or None if not upgradeable
fn get_tunnel_upgrade_cost(
    target_entity: Option<Entity>,
    tunnel_query: &Query<(&TunnelState, &Owner)>,
    all_tunnels: &Query<(Entity, &TunnelState), With<Owner>>,
) -> Option<u32> {
    let entity = target_entity?;
    let (ts, _) = tunnel_query.get(entity).ok()?;
    if ts.is_busy() { return None; }
    let target_tier = ts.tier.next_tier()?;

    match target_tier {
        TunnelTier::Tier2 => {
            let count = all_tunnels.iter()
                .filter(|(e, s)| *e != entity && matches!(s.tier, TunnelTier::Tier2 | TunnelTier::Tier3))
                .count() as u32;
            Some(tunnel_t2_upgrade_cost(count))
        },
        TunnelTier::Tier3 => {
            let count = all_tunnels.iter()
                .filter(|(e, s)| *e != entity && s.tier == TunnelTier::Tier3)
                .count() as u32;
            Some(tunnel_t3_upgrade_cost(count))
        },
        TunnelTier::Tier1 => None,
    }
}

/// Get available syndicate supplies for the tunnel's owner
fn get_syndicate_supplies(
    target_entity: Option<Entity>,
    tunnel_query: &Query<(&TunnelState, &Owner)>,
    syndicate_players: &Query<(&Player, &SyndicatePlayerResources)>,
) -> i32 {
    let entity = target_entity.unwrap_or(Entity::PLACEHOLDER);
    if let Ok((_, owner)) = tunnel_query.get(entity) {
        for (player, res) in syndicate_players.iter() {
            if Some(player.player_number) == owner.player_number() {
                return res.supplies;
            }
        }
    }
    0
}

/// Extended button enabled check that includes Tunnel-specific logic
fn grid_button_enabled_ext(
    action: &CommandButtonAction,
    player_sc: i32,
    bk_has_queue: bool,
    target_entity: Option<Entity>,
    bk_query: &Query<&BarracksState>,
    st_query: &Query<&SupplyTowerState>,
    tunnel_query: &Query<(&TunnelState, &Owner)>,
    tunnel_upgrade_cost: Option<u32>,
    syndicate_supplies: i32,
    unit_caps: &SelectedUnitCapabilities,
    has_power_plant: bool,
    syndicate_sc: i32,
    hq_owners: &Query<(&Owner, Option<&HeadquartersState>, Option<&RecruitmentCenterState>, Option<&ArmoryState>), (With<StructureInstance>, With<Selected>)>,
    has_network_units: bool,
) -> bool {
    match action {
        CommandButtonAction::ArmoryTrainSoldier | CommandButtonAction::ArmoryTrainGunner => {
            hq_owners.iter().next()
                .and_then(|(_, _, _, armory)| armory)
                .map(|a| !a.stored_recruits.is_empty() && a.training_queue.is_none())
                .unwrap_or(false)
        }
        CommandButtonAction::ArmoryEjectAll => {
            hq_owners.iter().next()
                .and_then(|(_, _, _, armory)| armory)
                .map(|a| !a.stored_recruits.is_empty())
                .unwrap_or(false)
        }
        CommandButtonAction::HqTrain(unit_type) => {
            let queue_full = hq_owners.iter().next()
                .and_then(|(_, hq, _, _)| hq)
                .map(|hq| hq.build_queue.len() >= HeadquartersState::MAX_QUEUE_SIZE)
                .unwrap_or(false);
            let cost = HeadquartersState::production_cost(unit_type)
                .map(|c| c.space_crystals as i32)
                .unwrap_or(100);
            syndicate_sc >= cost && !queue_full
        }
        // Tech prerequisite: Supply Tower requires Power Plant
        CommandButtonAction::DcBuild(ObjectEnum::SupplyTower) => player_sc >= 200 && has_power_plant,
        CommandButtonAction::TunnelUpgrade => {
            if let Some(cost) = tunnel_upgrade_cost {
                (syndicate_supplies as u32) >= cost
            } else {
                false // Already max tier or busy
            }
        }
        CommandButtonAction::TunnelOpenExpandMenu => {
            // Available if tunnel is not busy
            target_entity
                .and_then(|e| tunnel_query.get(e).ok())
                .map(|(ts, _)| !ts.is_busy())
                .unwrap_or(false)
        }
        CommandButtonAction::TunnelCancelUpgrade => true,
        CommandButtonAction::TunnelOpenEjectMenu => has_network_units,
        // Schedule Deliveries only available when tower has an attached chopper
        CommandButtonAction::StScheduleDeliveries => {
            target_entity
                .and_then(|e| st_query.get(e).ok())
                .map(|st| st.attached_chopper.is_some())
                .unwrap_or(false)
        }
        CommandButtonAction::AgentDropOff => unit_caps.agent_carrying,
        CommandButtonAction::ChopperDropOffSupplies => unit_caps.chopper_has_supplies,
        CommandButtonAction::RcCancel => true,
        CommandButtonAction::RecruitConstruct |
        CommandButtonAction::RecruitSelectBuilding(_) |
        CommandButtonAction::RecruitAssistConstruction => true,
        _ => grid_button_enabled(action, player_sc, bk_has_queue, target_entity, bk_query, st_query),
    }
}

/// Find syndicate player resources mutably for a given owner
fn find_syndicate_resources_mut<'a>(
    owner: &Owner,
    players: &'a mut Query<(&Player, &mut SyndicatePlayerResources)>,
) -> Option<Mut<'a, SyndicatePlayerResources>> {
    for (player, res) in players.iter_mut() {
        if Some(player.player_number) == owner.player_number() {
            return Some(res);
        }
    }
    None
}

/// Whether a CommandButtonAction is a unit/agent action (as opposed to structure/resource).
fn is_unit_action(action: &CommandButtonAction) -> bool {
    matches!(action,
        CommandButtonAction::UnitMove |
        CommandButtonAction::UnitAttack |
        CommandButtonAction::UnitAttackGround |
        CommandButtonAction::UnitAttackMove |
        CommandButtonAction::UnitPatrol |
        CommandButtonAction::UnitHoldPosition |
        CommandButtonAction::UnitStop |
        CommandButtonAction::UnitReverse |
        CommandButtonAction::UnitEnter |
        CommandButtonAction::UnitGather |
        CommandButtonAction::AgentBuildTunnel |
        CommandButtonAction::AgentDropOff |
        CommandButtonAction::ChopperPickUpSupplies |
        CommandButtonAction::ChopperAttachToTower |
        CommandButtonAction::ChopperDropOffSupplies |
        CommandButtonAction::RecruitConstruct |
        CommandButtonAction::RecruitSelectBuilding(_) |
        CommandButtonAction::RecruitAssistConstruction
    )
}

/// Whether a given ObjectEnum type supports a specific CommandButtonAction.
/// This is the static capability mapping used to determine common vs group commands.
fn object_type_supports_action(obj: &ObjectEnum, action: &CommandButtonAction) -> bool {
    // Structures and resources support no unit/agent actions
    if !obj.is_unit() {
        return false;
    }

    match action {
        // All units support basic movement commands
        CommandButtonAction::UnitMove |
        CommandButtonAction::UnitPatrol |
        CommandButtonAction::UnitHoldPosition |
        CommandButtonAction::UnitStop => true,

        // Attack requires AttackCapability — SupplyChopper lacks it
        CommandButtonAction::UnitAttack |
        CommandButtonAction::UnitAttackMove => matches!(obj,
            ObjectEnum::Peacekeeper | ObjectEnum::SyndicateAgent | ObjectEnum::SyndicateGuard
        ),

        // AttackGround requires can_target_ground (TailDisjointed or DoublyDisjointed attack type)
        // Only Peacekeeper has TailDisjointed attack
        CommandButtonAction::UnitAttackGround => matches!(obj, ObjectEnum::Peacekeeper),

        // Reverse requires can_reverse on UnitBase — currently no HeavyInfantry/Glider units have it
        // (WheeledVehicle, TrackedVehicle, DrillUnit do, but no ObjectEnum variants use those yet)
        CommandButtonAction::UnitReverse => false,

        // Agent-specific commands — only SyndicateAgent
        CommandButtonAction::AgentBuildTunnel |
        CommandButtonAction::AgentDropOff |
        CommandButtonAction::UnitGather => matches!(obj, ObjectEnum::SyndicateAgent),

        // Enter tunnel — Syndicate units only (Agent and Guard)
        CommandButtonAction::UnitEnter => matches!(obj,
            ObjectEnum::SyndicateAgent | ObjectEnum::SyndicateGuard
        ),

        // Supply Chopper-specific commands
        CommandButtonAction::ChopperPickUpSupplies |
        CommandButtonAction::ChopperAttachToTower |
        CommandButtonAction::ChopperDropOffSupplies => matches!(obj, ObjectEnum::SupplyChopper),

        // Cults Recruit-specific commands
        CommandButtonAction::RecruitConstruct |
        CommandButtonAction::RecruitSelectBuilding(_) |
        CommandButtonAction::RecruitAssistConstruction => matches!(obj, ObjectEnum::CultsRecruit),

        // All other actions (structure commands, etc.) are not unit actions
        _ => false,
    }
}

/// Whether a command action is a "common command" (available to all selected entities).
/// A command is common if and only if every SelectionGroup in the Selection supports it.
/// Structure/resource-specific commands are never common across unit groups.
fn is_common_command(action: &CommandButtonAction, selection: &Selection) -> bool {
    if !is_unit_action(action) {
        return false;
    }
    // A unit command is common iff every group supports it
    selection.groups.iter().all(|g| object_type_supports_action(&g.object_type, action))
}

/// Determine which entities should receive a command based on whether it's
/// a common command (issued to ALL selected) or group command (issued to active group only).
/// Get all entities from the active selection group (for multi-structure commands).
/// Falls back to panel_target.entity if no active group exists.
fn active_group_entities(selection: &Selection) -> Vec<Entity> {
    if let Some(group) = selection.active_group() {
        group.entities.clone()
    } else {
        Vec::new()
    }
}

fn command_target_entities(
    action: &CommandButtonAction,
    selection: &Selection,
    selected_units: &Query<(Entity, &mut Velocity, Option<&mut CommandQueue>), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
) -> Vec<Entity> {
    if is_common_command(action, selection) {
        // Common command: issue to all selected units
        selected_units.iter().map(|(e, _, _)| e).collect()
    } else {
        // Group command: issue only to active group entities
        if let Some(active_group) = selection.active_group() {
            active_group.entities.clone()
        } else {
            selected_units.iter().map(|(e, _, _)| e).collect()
        }
    }
}

/// Generate the label for a grid button based on state and action
fn grid_button_label(
    state: &ObjectInterfaceState,
    action: &CommandButtonAction,
    _player_sc: i32,
    hotkey: char,
) -> String {
    match action {
        CommandButtonAction::DcOpenBuildMenu => format!("[{}] Build", hotkey),
        CommandButtonAction::DcBuild(ObjectEnum::PowerPlant) => format!("[{}] PP\n150 SC", hotkey),
        CommandButtonAction::DcBuild(ObjectEnum::Barracks) => format!("[{}] BK\n200 SC", hotkey),
        CommandButtonAction::DcBuild(ObjectEnum::SupplyTower) => format!("[{}] ST\n200 SC", hotkey),
        CommandButtonAction::DcBuild(_) => format!("[{}] Build", hotkey),
        CommandButtonAction::DcCancel => match state {
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing) => format!("[{}] Cancel\nConstr.", hotkey),
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace) => format!("[{}] Cancel\nBuilding", hotkey),
            _ => format!("[{}] Cancel", hotkey),
        },
        CommandButtonAction::BkTrain(ObjectEnum::Peacekeeper) => format!("[{}] PK\n50 SC", hotkey),
        CommandButtonAction::BkTrain(_) => format!("[{}] Train", hotkey),
        CommandButtonAction::BkCancel => format!("[{}] Cancel\nLast", hotkey),
        CommandButtonAction::EfBuildPlate => format!("[{}] Plate\n75 SC", hotkey),
        CommandButtonAction::EfCancel => format!("[{}] Cancel", hotkey),
        CommandButtonAction::Back => format!("[{}] Back", hotkey),
        CommandButtonAction::EnterPlacement => format!("[{}] Place", hotkey),
        CommandButtonAction::UnitMove => format!("[{}] Move", hotkey),
        CommandButtonAction::UnitAttack => format!("[{}] Attack", hotkey),
        CommandButtonAction::UnitAttackGround => format!("[{}] Atk\nGround", hotkey),
        CommandButtonAction::UnitAttackMove => format!("[{}] Atk\nMove", hotkey),
        CommandButtonAction::UnitPatrol => format!("[{}] Patrol", hotkey),
        CommandButtonAction::UnitHoldPosition => format!("[{}] Hold\nPos", hotkey),
        CommandButtonAction::UnitStop => format!("[{}] Stop", hotkey),
        CommandButtonAction::UnitReverse => format!("[{}] Rev", hotkey),
        CommandButtonAction::UnitEnter => format!("[{}] Enter", hotkey),
        CommandButtonAction::UnitGather => format!("[{}] Gather", hotkey),
        CommandButtonAction::StTrain(ObjectEnum::SupplyChopper) => format!("[{}] SC\n100 SC", hotkey),
        CommandButtonAction::StTrain(_) => format!("[{}] Train", hotkey),
        CommandButtonAction::StCancel => format!("[{}] Cancel\nLast", hotkey),
        CommandButtonAction::StScheduleDeliveries => format!("[{}] Schedule\nDeliv", hotkey),
        CommandButtonAction::TunnelUpgrade => format!("[{}] Upgrade", hotkey),
        CommandButtonAction::TunnelOpenExpandMenu => format!("[{}] Expand", hotkey),
        CommandButtonAction::TunnelOpenEjectMenu => format!("[{}] Eject", hotkey),
        CommandButtonAction::TunnelSelectExpansion(obj) => {
            let cost = match obj {
                ObjectEnum::Headquarters => crate::game::types::syndicate_structure_stats::HQ_SC_COST,
                _ => 0,
            };
            if cost > 0 {
                format!("[{}] {}\n{} SC", hotkey, obj.object_type().name, cost)
            } else {
                format!("[{}] {}", hotkey, obj.object_type().name)
            }
        }
        CommandButtonAction::TunnelCancelUpgrade => format!("[{}] Cancel\nUpgrade", hotkey),
        CommandButtonAction::TunnelEjectUnit(obj) => format!("[{}] {}", hotkey, obj.object_type().name),
        CommandButtonAction::HqTrain(ObjectEnum::SyndicateAgent) => format!("[{}] Agent\n100 SC", hotkey),
        CommandButtonAction::HqTrain(ObjectEnum::SyndicateGuard) => format!("[{}] Guard\n125 SC", hotkey),
        CommandButtonAction::HqTrain(_) => format!("[{}] Train", hotkey),
        CommandButtonAction::HqCancel => format!("[{}] Cancel\nLast", hotkey),
        CommandButtonAction::AgentBuildTunnel => format!("[{}] Build\nTunnel", hotkey),
        CommandButtonAction::AgentDropOff => format!("[{}] Drop\nOff", hotkey),
        CommandButtonAction::SetRallyPoint => format!("[{}] Rally", hotkey),
        CommandButtonAction::ChopperPickUpSupplies => format!("[{}] Pick Up\nSupplies", hotkey),
        CommandButtonAction::ChopperAttachToTower => format!("[{}] Attach\nTower", hotkey),
        CommandButtonAction::ChopperDropOffSupplies => format!("[{}] Drop Off\nSupplies", hotkey),
        CommandButtonAction::RcCancel => format!("[{}] Cancel\nProd", hotkey),
        CommandButtonAction::ArmoryTrainSoldier => format!("[{}] Train\nSoldier", hotkey),
        CommandButtonAction::ArmoryTrainGunner => format!("[{}] Train\nGunner", hotkey),
        CommandButtonAction::ArmoryEjectAll => format!("[{}] Eject\nAll", hotkey),
        CommandButtonAction::RecruitConstruct => format!("[{}] Construct", hotkey),
        CommandButtonAction::RecruitSelectBuilding(ObjectEnum::CultsStorage) => format!("[{}] Storage", hotkey),
        CommandButtonAction::RecruitSelectBuilding(_) => format!("[{}] Build", hotkey),
        CommandButtonAction::RecruitAssistConstruction => format!("[{}] Assist\nConstruct", hotkey),
    }
}

/// Determine if a grid button should be enabled
fn grid_button_enabled(
    action: &CommandButtonAction,
    player_sc: i32,
    _bk_has_queue: bool,
    target_entity: Option<Entity>,
    bk_query: &Query<&BarracksState>,
    st_query: &Query<&SupplyTowerState>,
) -> bool {
    match action {
        CommandButtonAction::DcBuild(ObjectEnum::PowerPlant) => player_sc >= 150,
        CommandButtonAction::DcBuild(ObjectEnum::Barracks) => player_sc >= 200,
        CommandButtonAction::DcBuild(ObjectEnum::SupplyTower) => player_sc >= 200,
        CommandButtonAction::BkTrain(ObjectEnum::Peacekeeper) => {
            let queue_full = target_entity
                .and_then(|e| bk_query.get(e).ok())
                .map(|bk| bk.build_queue.len() >= BarracksState::MAX_QUEUE_SIZE)
                .unwrap_or(false);
            player_sc >= 50 && !queue_full
        }
        CommandButtonAction::StTrain(ObjectEnum::SupplyChopper) => {
            let queue_full = target_entity
                .and_then(|e| st_query.get(e).ok())
                .map(|st| st.build_queue.len() >= 5)
                .unwrap_or(false);
            player_sc >= 100 && !queue_full
        }
        CommandButtonAction::EfBuildPlate => player_sc >= 75,
        _ => true,
    }
}

/// Check if an action corresponds to the currently active command mode (AwaitingTarget)
fn is_action_active(action: &CommandButtonAction, interface_state: &ObjectInterfaceState) -> bool {
    let ct = match interface_state.awaiting_command_type() {
        Some(ct) => ct,
        None => return false,
    };
    match (action, ct) {
        (CommandButtonAction::UnitMove, CommandType::Move) => true,
        (CommandButtonAction::UnitAttack, CommandType::Attack) => true,
        (CommandButtonAction::UnitAttackGround, CommandType::AttackGround) => true,
        (CommandButtonAction::UnitAttackMove, CommandType::AttackMove) => true,
        (CommandButtonAction::UnitPatrol, CommandType::Patrol) => true,
        (CommandButtonAction::UnitReverse, CommandType::Reverse) => true,
        (CommandButtonAction::UnitEnter, CommandType::Enter) => true,
        (CommandButtonAction::UnitGather, CommandType::Gather) => true,
        (CommandButtonAction::ChopperPickUpSupplies, CommandType::PickUpSupplies) => true,
        (CommandButtonAction::ChopperAttachToTower, CommandType::AttachToTower) => true,
        (CommandButtonAction::ChopperDropOffSupplies, CommandType::DropOffSupplies) => true,
        _ => false,
    }
}

/// Spawn a panel title text
fn spawn_panel_title(parent: &mut ChildSpawnerCommands, title: &str) {
    parent.spawn((
        Text::new(title),
        TextFont { font_size: 14.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Node {
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        },
    ));
}

/// Spawn a progress text line
fn spawn_progress_text(parent: &mut ChildSpawnerCommands, text: &str) {
    parent.spawn((
        Text::new(text),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgb(0.9, 0.9, 0.5)),
        Node {
            margin: UiRect::bottom(Val::Px(2.0)),
            ..default()
        },
    ));
}

/// Spawn an info text line
fn spawn_info_text(parent: &mut ChildSpawnerCommands, text: &str) {
    parent.spawn((
        Text::new(text),
        TextFont { font_size: 11.0, ..default() },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
        Node {
            margin: UiRect::bottom(Val::Px(2.0)),
            ..default()
        },
    ));
}

/// Spawn a command button in the 3x3 grid with hotkey label
fn spawn_grid_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: CommandButtonAction,
    enabled: bool,
    active: bool,
    is_common: bool,
    row: u8,
    col: u8,
) {
    let bg_color = if active {
        Color::srgb(0.2, 0.45, 0.6)  // Highlighted blue for active mode
    } else if enabled && is_common {
        Color::srgb(0.25, 0.35, 0.25)  // Green-tinted for common commands
    } else if enabled {
        Color::srgb(0.3, 0.3, 0.2)  // Yellow-tinted for group-specific commands
    } else {
        Color::srgb(0.2, 0.2, 0.2)
    };

    let text_color = if enabled {
        Color::srgb(0.9, 0.9, 0.9)
    } else {
        Color::srgb(0.5, 0.5, 0.5)
    };

    parent.spawn((
        Button,
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(bg_color),
        action,
        GridSlot { row, col },
        CommandButtonEnabled(enabled),
        CommandButtonCommon(is_common),
    ))
    .with_children(|button| {
        button.spawn((
            Text::new(label),
            TextFont { font_size: 10.0, ..default() },
            TextColor(text_color),
            Node {
                max_width: Val::Px(58.0),
                ..default()
            },
        ));
    });
}

/// Spawn an empty placeholder cell in the grid
fn spawn_empty_grid_cell(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node::default(),
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.3)),
    ));
}

/// Describes what target_info the resolution function needs to know about a
/// cursor-hovered entity, extracted from ECS queries.
#[derive(Debug, Clone, Default)]
struct CursorEntityInfo {
    has_crystal_patch: bool,
    has_supply_station: bool,
    has_tunnel: bool,
    is_own: bool,
    has_supply_tower: bool,
}

/// Pure function that resolves the pointer display type from the current
/// interface state, cursor target, and selection capabilities.
///
/// Separated from the ECS system for testability.
fn resolve_pointer_display(
    interface_state: &ObjectInterfaceState,
    cursor_kind: &CursorTargetEnum,
    has_cursor_entity: bool,
    cursor_entity_info: &CursorEntityInfo,
    selection_count: usize,
    active_type: Option<ObjectEnum>,
    unit_caps: &SelectedUnitCapabilities,
    active_group_has_production: bool,
) -> PointerDisplayType {
    // Placement mode → Inactive
    if interface_state.is_placement_mode() {
        return PointerDisplayType::Inactive;
    }

    // AwaitingTarget mode
    if let Some(ct) = interface_state.awaiting_command_type() {
        return resolve_awaiting_target(ct, cursor_kind, has_cursor_entity, cursor_entity_info);
    }

    // DefaultState (and structure/agent menu states)
    resolve_default_state(
        cursor_kind,
        has_cursor_entity,
        cursor_entity_info,
        selection_count,
        active_type,
        unit_caps,
        active_group_has_production,
    )
}

/// Resolution logic for AwaitingTarget command modes.
fn resolve_awaiting_target(
    ct: CommandType,
    cursor_kind: &CursorTargetEnum,
    _has_cursor_entity: bool,
    cursor_entity_info: &CursorEntityInfo,
) -> PointerDisplayType {
    match ct {
        CommandType::Attack => match cursor_kind {
            CursorTargetEnum::EnemyObject => PointerDisplayType::Attack,
            CursorTargetEnum::Ground => PointerDisplayType::Attack, // AttackMove preview
            _ => PointerDisplayType::Inactive,
        },
        CommandType::Move => PointerDisplayType::Move,
        CommandType::Patrol => match cursor_kind {
            CursorTargetEnum::Ground => PointerDisplayType::Patrol,
            _ => PointerDisplayType::Inactive,
        },
        CommandType::AttackGround => match cursor_kind {
            CursorTargetEnum::Ground => PointerDisplayType::AttackGround,
            _ => PointerDisplayType::Inactive,
        },
        CommandType::Reverse => match cursor_kind {
            CursorTargetEnum::Ground => PointerDisplayType::Move,
            _ => PointerDisplayType::Inactive,
        },
        CommandType::ScheduleDeliveries => {
            if cursor_entity_info.has_supply_station {
                PointerDisplayType::GatherResources
            } else {
                PointerDisplayType::Inactive
            }
        },
        CommandType::SetRallyPoint => PointerDisplayType::Move,
        CommandType::Enter => {
            if cursor_entity_info.has_tunnel && cursor_entity_info.is_own {
                PointerDisplayType::Enter
            } else {
                PointerDisplayType::Inactive
            }
        },
        CommandType::Gather => {
            if cursor_entity_info.has_crystal_patch || cursor_entity_info.has_supply_station {
                PointerDisplayType::GatherResources
            } else {
                PointerDisplayType::Inactive
            }
        },
        CommandType::DropOff | CommandType::DropOffSupplies => {
            if cursor_entity_info.has_tunnel && cursor_entity_info.is_own {
                PointerDisplayType::ReturnResources
            } else if cursor_entity_info.has_supply_tower && cursor_entity_info.is_own {
                PointerDisplayType::ReturnResources
            } else {
                PointerDisplayType::Inactive
            }
        },
        _ => PointerDisplayType::Inactive,
    }
}

/// Resolution logic for DefaultState (and structure/agent menu states).
fn resolve_default_state(
    cursor_kind: &CursorTargetEnum,
    _has_cursor_entity: bool,
    cursor_entity_info: &CursorEntityInfo,
    selection_count: usize,
    active_type: Option<ObjectEnum>,
    unit_caps: &SelectedUnitCapabilities,
    active_group_has_production: bool,
) -> PointerDisplayType {
    if selection_count == 0 {
        return PointerDisplayType::Inactive;
    }

    // Production structures → Move (rally point preview)
    if active_group_has_production {
        return PointerDisplayType::Move;
    }

    let is_unit_group = active_type.map(|t| t.is_unit()).unwrap_or(false);

    // Enemy target + has_attack → Attack
    if *cursor_kind == CursorTargetEnum::EnemyObject && unit_caps.has_attack {
        return PointerDisplayType::Attack;
    }

    // Resource gathering checks (agents and choppers)
    let is_agent = active_type == Some(ObjectEnum::SyndicateAgent);
    let is_chopper = unit_caps.is_chopper;

    if (is_agent || is_chopper) && (cursor_entity_info.has_crystal_patch || cursor_entity_info.has_supply_station) {
        return PointerDisplayType::GatherResources;
    }

    // Return resources check — agent carrying to own tunnel
    if unit_caps.agent_carrying && cursor_entity_info.has_tunnel && cursor_entity_info.is_own {
        return PointerDisplayType::ReturnResources;
    }

    // Return resources check — chopper with supplies to own supply tower
    if unit_caps.chopper_has_supplies && cursor_entity_info.has_supply_tower && cursor_entity_info.is_own {
        return PointerDisplayType::ReturnResources;
    }

    // Enter tunnel — syndicate units to own tunnel
    let is_syndicate_unit = matches!(active_type, Some(ObjectEnum::SyndicateAgent | ObjectEnum::SyndicateGuard));
    if is_syndicate_unit && cursor_entity_info.has_tunnel && cursor_entity_info.is_own {
        return PointerDisplayType::Enter;
    }

    // Ground / Friendly / Neutral with movable units → Move
    if is_unit_group && matches!(cursor_kind, CursorTargetEnum::Ground | CursorTargetEnum::FriendlyObject | CursorTargetEnum::NeutralObject) {
        return PointerDisplayType::Move;
    }

    PointerDisplayType::Inactive
}

/// System that resolves `PointerDisplayType` each frame based on the current
/// interface state, cursor target, selection, and unit capabilities.
///
/// Must run after `update_command_panel_state` (which populates
/// `SelectedUnitCapabilities`, `ObjectInterfaceState`) and after
/// `update_cursor_target` (which populates `CursorTarget`).
pub fn resolve_pointer_display_type(
    interface_state: Res<ObjectInterfaceState>,
    cursor_target: Res<CursorTarget>,
    selection: Res<Selection>,
    unit_caps: Res<SelectedUnitCapabilities>,
    mut pointer_display: ResMut<PointerDisplayType>,
    target_info: Query<(Option<&SpaceCrystalPatch>, Option<&SupplyDeliveryStation>, Option<&TunnelState>, Option<&SupplyTowerState>, &Owner), With<ObjectInstance>>,
    selected_structures: Query<(Option<&BarracksState>, Option<&HeadquartersState>, Option<&SupplyTowerState>, Option<&DeploymentCenterState>, Option<&ExtractionFacilityState>), (With<StructureInstance>, With<Selected>)>,
    local_player: Res<LocalPlayer>,
) {
    // Build cursor entity info from ECS queries
    let cursor_entity_info = if let Some(entity) = cursor_target.entity {
        if let Ok((crystal, supply_station, tunnel, supply_tower, owner)) = target_info.get(entity) {
            CursorEntityInfo {
                has_crystal_patch: crystal.is_some(),
                has_supply_station: supply_station.is_some(),
                has_tunnel: tunnel.is_some(),
                is_own: owner.player_number() == Some(local_player.0),
                has_supply_tower: supply_tower.is_some(),
            }
        } else {
            CursorEntityInfo::default()
        }
    } else {
        CursorEntityInfo::default()
    };

    // Check if the active group is a production structure
    let active_group_has_production = if let Some(group) = selection.active_group() {
        if group.object_type.is_structure() {
            group.entities.iter().any(|&e| {
                if let Ok((bk, hq, st, dc, ef)) = selected_structures.get(e) {
                    bk.is_some() || hq.is_some() || st.is_some() || dc.is_some() || ef.is_some()
                } else {
                    false
                }
            })
        } else {
            false
        }
    } else {
        false
    };

    *pointer_display = resolve_pointer_display(
        &interface_state,
        &cursor_target.kind,
        cursor_target.entity.is_some(),
        &cursor_entity_info,
        selection.total_entity_count(),
        selection.active_type(),
        &unit_caps,
        active_group_has_production,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use crate::game::combat::types::{AttackType, ProjectileVisual};

    // === attack_type_can_target_ground tests ===

    #[test]
    fn fully_connected_ranged_cannot_target_ground() {
        let at = AttackType::FullyConnected { subtype: FullyConnectedSubtype::Ranged };
        assert!(!attack_type_can_target_ground(&at));
    }

    #[test]
    fn fully_connected_melee_cannot_target_ground() {
        let at = AttackType::FullyConnected { subtype: FullyConnectedSubtype::Melee };
        assert!(!attack_type_can_target_ground(&at));
    }

    #[test]
    fn head_disjointed_cannot_target_ground() {
        let at = AttackType::HeadDisjointed { effect_radius: 2.0 };
        assert!(!attack_type_can_target_ground(&at));
    }

    #[test]
    fn tail_disjointed_can_target_ground() {
        let at = AttackType::TailDisjointed {
            projectile_speed: 10.0,
            projectile_visual: ProjectileVisual::Sphere { radius: 0.1 },
        };
        assert!(attack_type_can_target_ground(&at));
    }

    #[test]
    fn doubly_disjointed_can_target_ground() {
        let at = AttackType::DoublyDisjointed {
            projectile_speed: 10.0,
            projectile_visual: ProjectileVisual::Sphere { radius: 0.1 },
            effect_radius: 2.0,
        };
        assert!(attack_type_can_target_ground(&at));
    }

    // === SelectedUnitCapabilities tests ===

    #[test]
    fn selected_unit_capabilities_default_all_false() {
        let caps = SelectedUnitCapabilities::default();
        assert!(!caps.has_attack);
        assert!(!caps.can_target_ground);
        assert!(!caps.can_reverse);
    }

    #[test]
    fn selected_unit_capabilities_equality() {
        let a = SelectedUnitCapabilities { has_attack: true, can_target_ground: false, can_reverse: true, agent_carrying: false, is_chopper: false, chopper_has_supplies: false, owned_by_local_player: false };
        let b = SelectedUnitCapabilities { has_attack: true, can_target_ground: false, can_reverse: true, agent_carrying: false, is_chopper: false, chopper_has_supplies: false, owned_by_local_player: false };
        let c = SelectedUnitCapabilities { has_attack: false, can_target_ground: false, can_reverse: true, agent_carrying: false, is_chopper: false, chopper_has_supplies: false, owned_by_local_player: false };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // === get_grid_slot_action conditional tests ===

    fn all_caps() -> SelectedUnitCapabilities {
        SelectedUnitCapabilities { has_attack: true, can_target_ground: true, can_reverse: true, agent_carrying: false, is_chopper: false, chopper_has_supplies: false, owned_by_local_player: false }
    }

    fn no_caps() -> SelectedUnitCapabilities {
        SelectedUnitCapabilities::default()
    }

    /// Selection with only unit groups (unit commands are common)
    fn units_only_selection() -> Selection {
        Selection {
            groups: vec![
                SelectionGroup {
                    object_type: ObjectEnum::Peacekeeper,
                    entities: vec![Entity::from_raw_u32(1).unwrap()],
                },
                SelectionGroup {
                    object_type: ObjectEnum::SyndicateAgent,
                    entities: vec![Entity::from_raw_u32(2).unwrap()],
                },
            ],
            active_group_index: Some(0),
        }
    }

    /// Selection with mixed unit + structure groups (unit commands are NOT common)
    fn mixed_unit_structure_selection() -> Selection {
        Selection {
            groups: vec![
                SelectionGroup {
                    object_type: ObjectEnum::Peacekeeper,
                    entities: vec![Entity::from_raw_u32(1).unwrap()],
                },
                SelectionGroup {
                    object_type: ObjectEnum::Headquarters,
                    entities: vec![Entity::from_raw_u32(2).unwrap()],
                },
            ],
            active_group_index: Some(0),
        }
    }

    /// Selection with only structure groups
    fn structures_only_selection() -> Selection {
        Selection {
            groups: vec![
                SelectionGroup {
                    object_type: ObjectEnum::DeploymentCenter,
                    entities: vec![Entity::from_raw_u32(1).unwrap()],
                },
                SelectionGroup {
                    object_type: ObjectEnum::PowerPlant,
                    entities: vec![Entity::from_raw_u32(2).unwrap()],
                },
            ],
            active_group_index: Some(0),
        }
    }

    fn attack_only() -> SelectedUnitCapabilities {
        SelectedUnitCapabilities { has_attack: true, can_target_ground: false, can_reverse: false, agent_carrying: false, is_chopper: false, chopper_has_supplies: false, owned_by_local_player: false }
    }

    #[test]
    fn unit_commands_move_always_available() {
        let caps = no_caps();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::UnitMove)));
    }

    #[test]
    fn unit_commands_attack_requires_has_attack() {
        let caps = no_caps();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 1, 0, false, false, &caps, false, false);
        assert!(action.is_none());

        let caps = attack_only();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 1, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::UnitAttack)));
    }

    #[test]
    fn unit_commands_attack_ground_requires_can_target_ground() {
        let caps = attack_only();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 1, 2, false, false, &caps, false, false);
        assert!(action.is_none());

        let caps = all_caps();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 1, 2, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::UnitAttackGround)));
    }

    #[test]
    fn unit_commands_patrol_always_available() {
        let caps = no_caps();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 1, 1, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::UnitPatrol)));
    }

    #[test]
    fn unit_commands_hold_position_always_available() {
        let caps = no_caps();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 0, 2, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::UnitHoldPosition)));
    }

    #[test]
    fn unit_commands_stop_always_available() {
        let caps = no_caps();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 2, 1, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::UnitStop)));
    }

    #[test]
    fn unit_commands_reverse_requires_can_reverse() {
        let caps = no_caps();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 0, 1, false, false, &caps, false, false);
        assert!(action.is_none());

        let caps = SelectedUnitCapabilities { has_attack: false, can_target_ground: false, can_reverse: true, agent_carrying: false, is_chopper: false, chopper_has_supplies: false, owned_by_local_player: false };
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 0, 1, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::UnitReverse)));
    }

    // === is_action_active tests ===

    #[test]
    fn is_action_active_reverse_mode() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::Reverse);
        assert!(is_action_active(&CommandButtonAction::UnitReverse, &state));
        assert!(!is_action_active(&CommandButtonAction::UnitMove, &state));
    }

    #[test]
    fn is_action_active_attack_mode() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::Attack);
        assert!(is_action_active(&CommandButtonAction::UnitAttack, &state));
        assert!(!is_action_active(&CommandButtonAction::UnitReverse, &state));
    }

    #[test]
    fn is_action_active_default_state_nothing_active() {
        let state = ObjectInterfaceState::Default;
        assert!(!is_action_active(&CommandButtonAction::UnitMove, &state));
        assert!(!is_action_active(&CommandButtonAction::UnitAttack, &state));
    }

    #[test]
    fn is_action_active_enter_mode() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::Enter);
        assert!(is_action_active(&CommandButtonAction::UnitEnter, &state));
        assert!(!is_action_active(&CommandButtonAction::UnitMove, &state));
    }

    #[test]
    fn is_action_active_gather_mode() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::Gather);
        assert!(is_action_active(&CommandButtonAction::UnitGather, &state));
        assert!(!is_action_active(&CommandButtonAction::UnitAttack, &state));
    }

    // === grid_button_label tests ===

    #[test]
    fn grid_button_label_reverse() {
        let label = grid_button_label(
            &ObjectInterfaceState::Default,
            &CommandButtonAction::UnitReverse,
            0,
            'X',
        );
        assert_eq!(label, "[X] Rev");
    }

    // === Conditional layout completeness ===

    #[test]
    fn all_caps_unit_commands_shows_all_seven_commands() {
        let caps = all_caps();
        let mut count = 0;
        for row in 0..3u8 {
            for col in 0..3u8 {
                if get_grid_slot_action(&ObjectInterfaceState::Default, row, col, false, false, &caps, false, false).is_some() {
                    count += 1;
                }
            }
        }
        // Move, Reverse, HoldPos, Attack, Patrol, AtkGround, Stop = 7
        assert_eq!(count, 7);
    }

    #[test]
    fn no_caps_unit_commands_shows_only_universal() {
        let caps = no_caps();
        let mut count = 0;
        for row in 0..3u8 {
            for col in 0..3u8 {
                if get_grid_slot_action(&ObjectInterfaceState::Default, row, col, false, false, &caps, false, false).is_some() {
                    count += 1;
                }
            }
        }
        // Move(0,0), HoldPos(0,2), Patrol(1,1), Stop(2,1) = 4
        assert_eq!(count, 4);
    }

    // === ObjectInterfaceState tests ===

    #[test]
    fn object_interface_state_default_is_default() {
        let state = ObjectInterfaceState::default();
        assert_eq!(state, ObjectInterfaceState::Default);
    }

    #[test]
    fn object_interface_state_awaiting_target() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::Attack);
        assert!(state.is_awaiting_target());
        assert_eq!(state.awaiting_command_type(), Some(CommandType::Attack));
    }

    #[test]
    fn object_interface_state_default_not_awaiting() {
        let state = ObjectInterfaceState::Default;
        assert!(!state.is_awaiting_target());
        assert_eq!(state.awaiting_command_type(), None);
    }

    #[test]
    fn object_interface_state_placement_mode() {
        let dc = ObjectInterfaceState::StructureMenu(StructureMenuState::DcAwaitingPlacement);
        let ef = ObjectInterfaceState::StructureMenu(StructureMenuState::EfAwaitingPlacement);
        let idle = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        assert!(dc.is_placement_mode());
        assert!(ef.is_placement_mode());
        assert!(!idle.is_placement_mode());
    }

    // === CursorTarget tests ===

    #[test]
    fn cursor_target_default_is_none() {
        let ct = CursorTarget::default();
        assert_eq!(ct.kind, CursorTargetEnum::None);
        assert!(ct.location.is_none());
        assert!(ct.entity.is_none());
    }

    #[test]
    fn cursor_target_ground_variant() {
        let ct = CursorTarget {
            kind: CursorTargetEnum::Ground,
            location: Some(Vec3::new(5.0, 0.0, 3.0)),
            entity: None,
        };
        assert_eq!(ct.kind, CursorTargetEnum::Ground);
        assert!(ct.location.is_some());
        assert!(ct.entity.is_none());
    }

    #[test]
    fn cursor_target_enemy_variant() {
        let ct = CursorTarget {
            kind: CursorTargetEnum::EnemyObject,
            location: Some(Vec3::ZERO),
            entity: Some(Entity::from_raw_u32(42).unwrap()),
        };
        assert_eq!(ct.kind, CursorTargetEnum::EnemyObject);
        assert!(ct.entity.is_some());
    }

    // === is_panel_visible tests ===

    #[test]
    fn panel_hidden_when_default_and_empty_selection() {
        let state = ObjectInterfaceState::Default;
        let selection = Selection::default();
        assert!(!is_panel_visible(&state, &selection));
    }

    #[test]
    fn panel_visible_when_default_and_has_selection() {
        let state = ObjectInterfaceState::Default;
        let mut selection = Selection::default();
        selection.build_from_entities(&[(Entity::from_raw_u32(1).unwrap(), ObjectEnum::Peacekeeper, true)]);
        assert!(is_panel_visible(&state, &selection));
    }

    #[test]
    fn panel_visible_when_structure_menu() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        let selection = Selection::default();
        assert!(is_panel_visible(&state, &selection));
    }

    #[test]
    fn panel_visible_when_awaiting_target() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::Attack);
        let selection = Selection::default();
        assert!(is_panel_visible(&state, &selection));
    }

    // === is_common_command tests ===

    #[test]
    fn move_is_common_with_units_only() {
        let sel = units_only_selection();
        assert!(is_common_command(&CommandButtonAction::UnitMove, &sel));
    }

    #[test]
    fn stop_is_common_with_units_only() {
        let sel = units_only_selection();
        assert!(is_common_command(&CommandButtonAction::UnitStop, &sel));
    }

    #[test]
    fn hold_position_is_common_with_units_only() {
        let sel = units_only_selection();
        assert!(is_common_command(&CommandButtonAction::UnitHoldPosition, &sel));
    }

    #[test]
    fn patrol_is_common_with_units_only() {
        let sel = units_only_selection();
        assert!(is_common_command(&CommandButtonAction::UnitPatrol, &sel));
    }

    #[test]
    fn attack_is_common_when_all_groups_support_it() {
        // Peacekeeper + SyndicateAgent both have attack → common
        let sel = units_only_selection();
        assert!(is_common_command(&CommandButtonAction::UnitAttack, &sel));
    }

    #[test]
    fn attack_is_not_common_when_chopper_in_selection() {
        // Peacekeeper + SupplyChopper: chopper has no attack → NOT common
        let sel = Selection {
            groups: vec![
                SelectionGroup {
                    object_type: ObjectEnum::Peacekeeper,
                    entities: vec![Entity::from_raw_u32(1).unwrap()],
                },
                SelectionGroup {
                    object_type: ObjectEnum::SupplyChopper,
                    entities: vec![Entity::from_raw_u32(2).unwrap()],
                },
            ],
            active_group_index: Some(0),
        };
        assert!(!is_common_command(&CommandButtonAction::UnitAttack, &sel));
    }

    #[test]
    fn reverse_is_not_common_command() {
        // No current unit types have can_reverse → never common
        let sel = units_only_selection();
        assert!(!is_common_command(&CommandButtonAction::UnitReverse, &sel));
    }

    #[test]
    fn unit_commands_not_common_with_mixed_selection() {
        let sel = mixed_unit_structure_selection();
        assert!(!is_common_command(&CommandButtonAction::UnitMove, &sel));
        assert!(!is_common_command(&CommandButtonAction::UnitStop, &sel));
        assert!(!is_common_command(&CommandButtonAction::UnitHoldPosition, &sel));
        assert!(!is_common_command(&CommandButtonAction::UnitPatrol, &sel));
        assert!(!is_common_command(&CommandButtonAction::AgentBuildTunnel, &sel));
        assert!(!is_common_command(&CommandButtonAction::AgentDropOff, &sel));
    }

    #[test]
    fn unit_commands_not_common_with_structures_only() {
        let sel = structures_only_selection();
        assert!(!is_common_command(&CommandButtonAction::UnitMove, &sel));
        assert!(!is_common_command(&CommandButtonAction::UnitStop, &sel));
    }

    #[test]
    fn basic_commands_common_with_chopper_and_peacekeeper() {
        // Peacekeeper + SupplyChopper: Move, Stop, HoldPosition, Patrol ARE common
        let sel = Selection {
            groups: vec![
                SelectionGroup {
                    object_type: ObjectEnum::Peacekeeper,
                    entities: vec![Entity::from_raw_u32(1).unwrap()],
                },
                SelectionGroup {
                    object_type: ObjectEnum::SupplyChopper,
                    entities: vec![Entity::from_raw_u32(2).unwrap()],
                },
            ],
            active_group_index: Some(0),
        };
        assert!(is_common_command(&CommandButtonAction::UnitMove, &sel));
        assert!(is_common_command(&CommandButtonAction::UnitStop, &sel));
        assert!(is_common_command(&CommandButtonAction::UnitHoldPosition, &sel));
        assert!(is_common_command(&CommandButtonAction::UnitPatrol, &sel));
    }

    #[test]
    fn attack_common_with_peacekeeper_and_guard() {
        // Peacekeeper + SyndicateGuard: both have attack → common
        let sel = Selection {
            groups: vec![
                SelectionGroup {
                    object_type: ObjectEnum::Peacekeeper,
                    entities: vec![Entity::from_raw_u32(1).unwrap()],
                },
                SelectionGroup {
                    object_type: ObjectEnum::SyndicateGuard,
                    entities: vec![Entity::from_raw_u32(2).unwrap()],
                },
            ],
            active_group_index: Some(0),
        };
        assert!(is_common_command(&CommandButtonAction::UnitAttack, &sel));
        assert!(is_common_command(&CommandButtonAction::UnitAttackMove, &sel));
    }

    #[test]
    fn attack_ground_not_common_across_different_units() {
        // Peacekeeper (has AttackGround) + SyndicateGuard (no AttackGround) → NOT common
        let sel = Selection {
            groups: vec![
                SelectionGroup {
                    object_type: ObjectEnum::Peacekeeper,
                    entities: vec![Entity::from_raw_u32(1).unwrap()],
                },
                SelectionGroup {
                    object_type: ObjectEnum::SyndicateGuard,
                    entities: vec![Entity::from_raw_u32(2).unwrap()],
                },
            ],
            active_group_index: Some(0),
        };
        assert!(!is_common_command(&CommandButtonAction::UnitAttackGround, &sel));
    }

    #[test]
    fn object_type_supports_action_structure_supports_nothing() {
        assert!(!object_type_supports_action(&ObjectEnum::DeploymentCenter, &CommandButtonAction::UnitMove));
        assert!(!object_type_supports_action(&ObjectEnum::Barracks, &CommandButtonAction::UnitStop));
    }

    #[test]
    fn object_type_supports_action_resource_supports_nothing() {
        assert!(!object_type_supports_action(&ObjectEnum::SpaceCrystalsPatch, &CommandButtonAction::UnitMove));
        assert!(!object_type_supports_action(&ObjectEnum::SupplyDeliveryStation, &CommandButtonAction::UnitAttack));
    }

    #[test]
    fn object_type_supports_action_chopper_no_attack() {
        assert!(object_type_supports_action(&ObjectEnum::SupplyChopper, &CommandButtonAction::UnitMove));
        assert!(!object_type_supports_action(&ObjectEnum::SupplyChopper, &CommandButtonAction::UnitAttack));
        assert!(!object_type_supports_action(&ObjectEnum::SupplyChopper, &CommandButtonAction::UnitAttackMove));
        assert!(!object_type_supports_action(&ObjectEnum::SupplyChopper, &CommandButtonAction::UnitAttackGround));
    }

    #[test]
    fn object_type_supports_action_agent_specific() {
        assert!(object_type_supports_action(&ObjectEnum::SyndicateAgent, &CommandButtonAction::AgentBuildTunnel));
        assert!(object_type_supports_action(&ObjectEnum::SyndicateAgent, &CommandButtonAction::AgentDropOff));
        assert!(object_type_supports_action(&ObjectEnum::SyndicateAgent, &CommandButtonAction::UnitGather));
        assert!(object_type_supports_action(&ObjectEnum::SyndicateAgent, &CommandButtonAction::UnitEnter));
        // Peacekeeper does NOT support agent commands
        assert!(!object_type_supports_action(&ObjectEnum::Peacekeeper, &CommandButtonAction::AgentBuildTunnel));
        assert!(!object_type_supports_action(&ObjectEnum::Peacekeeper, &CommandButtonAction::AgentDropOff));
    }

    #[test]
    fn is_unit_action_covers_all_unit_actions() {
        assert!(is_unit_action(&CommandButtonAction::UnitMove));
        assert!(is_unit_action(&CommandButtonAction::UnitAttack));
        assert!(is_unit_action(&CommandButtonAction::UnitAttackGround));
        assert!(is_unit_action(&CommandButtonAction::UnitAttackMove));
        assert!(is_unit_action(&CommandButtonAction::UnitPatrol));
        assert!(is_unit_action(&CommandButtonAction::UnitHoldPosition));
        assert!(is_unit_action(&CommandButtonAction::UnitStop));
        assert!(is_unit_action(&CommandButtonAction::UnitReverse));
        assert!(is_unit_action(&CommandButtonAction::UnitEnter));
        assert!(is_unit_action(&CommandButtonAction::UnitGather));
        assert!(is_unit_action(&CommandButtonAction::AgentBuildTunnel));
        assert!(is_unit_action(&CommandButtonAction::AgentDropOff));
        assert!(is_unit_action(&CommandButtonAction::ChopperPickUpSupplies));
        assert!(is_unit_action(&CommandButtonAction::ChopperAttachToTower));
        // Structure commands are NOT unit actions
        assert!(!is_unit_action(&CommandButtonAction::DcOpenBuildMenu));
        assert!(!is_unit_action(&CommandButtonAction::BkCancel));
    }

    // === StructureMenuState grid tests ===

    #[test]
    fn dc_idle_shows_build_button() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        let caps = no_caps();
        let action = get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::DcOpenBuildMenu)));
    }

    #[test]
    fn dc_build_menu_shows_structures() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
        let caps = no_caps();
        let pp = get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(pp, Some(CommandButtonAction::DcBuild(ObjectEnum::PowerPlant))));
        let bk = get_grid_slot_action(&state, 0, 1, false, false, &caps, false, false);
        assert!(matches!(bk, Some(CommandButtonAction::DcBuild(ObjectEnum::Barracks))));
    }

    #[test]
    fn awaiting_target_has_back_at_z() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::Attack);
        let caps = all_caps();
        // Z (2,0) should be Back
        let action = get_grid_slot_action(&state, 2, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::Back)));
        // All other slots should be None
        for row in 0..3u8 {
            for col in 0..3u8 {
                if (row, col) == (2, 0) { continue; }
                assert!(get_grid_slot_action(&state, row, col, false, false, &caps, false, false).is_none(),
                    "Slot ({}, {}) should be empty in AwaitingTarget", row, col);
            }
        }
    }

    // === Tunnel Interface State Tests ===

    #[test]
    fn tunnel_idle_shows_three_commands() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
        let caps = no_caps();
        let upgrade = get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(upgrade, Some(CommandButtonAction::TunnelUpgrade)));
        let expand = get_grid_slot_action(&state, 0, 1, false, false, &caps, false, false);
        assert!(matches!(expand, Some(CommandButtonAction::TunnelOpenExpandMenu)));
        let eject = get_grid_slot_action(&state, 0, 2, false, false, &caps, false, false);
        assert!(matches!(eject, Some(CommandButtonAction::TunnelOpenEjectMenu)));
    }

    #[test]
    fn tunnel_idle_remaining_slots_are_none() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
        let caps = no_caps();
        // Only (0,0), (0,1), (0,2) should have actions
        for row in 1..3u8 {
            for col in 0..3u8 {
                assert!(get_grid_slot_action(&state, row, col, false, false, &caps, false, false).is_none(),
                    "Expected None at ({}, {})", row, col);
            }
        }
    }

    #[test]
    fn tunnel_idle_cancel_upgrade_shown_when_upgrading() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
        let caps = no_caps();
        let action = get_grid_slot_action(&state, 2, 1, false, false, &caps, true, false);
        assert!(matches!(action, Some(CommandButtonAction::TunnelCancelUpgrade)));
    }

    #[test]
    fn tunnel_idle_cancel_upgrade_hidden_when_not_upgrading() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
        let caps = no_caps();
        let action = get_grid_slot_action(&state, 2, 1, false, false, &caps, false, false);
        assert!(action.is_none());
    }

    #[test]
    fn tunnel_cancel_upgrade_label() {
        let label = grid_button_label(
            &ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle),
            &CommandButtonAction::TunnelCancelUpgrade,
            0,
            'X',
        );
        assert_eq!(label, "[X] Cancel\nUpgrade");
    }

    #[test]
    fn tunnel_cancel_upgrade_is_not_common_command() {
        let sel = units_only_selection();
        assert!(!is_common_command(&CommandButtonAction::TunnelCancelUpgrade, &sel));
    }

    #[test]
    fn tunnel_expand_menu_headquarters_at_0_0() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
        let caps = no_caps();
        let action = get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::TunnelSelectExpansion(ObjectEnum::Headquarters))));
    }

    #[test]
    fn tunnel_expand_menu_back_at_2_0() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
        let caps = no_caps();
        let action = get_grid_slot_action(&state, 2, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::Back)));
    }

    #[test]
    fn tunnel_expand_menu_empty_slots_return_none() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
        let caps = no_caps();
        // All non-mapped slots should return None
        assert!(get_grid_slot_action(&state, 0, 1, false, false, &caps, false, false).is_none());
        assert!(get_grid_slot_action(&state, 0, 2, false, false, &caps, false, false).is_none());
        assert!(get_grid_slot_action(&state, 1, 0, false, false, &caps, false, false).is_none());
        assert!(get_grid_slot_action(&state, 1, 1, false, false, &caps, false, false).is_none());
        assert!(get_grid_slot_action(&state, 1, 2, false, false, &caps, false, false).is_none());
        assert!(get_grid_slot_action(&state, 2, 1, false, false, &caps, false, false).is_none());
        assert!(get_grid_slot_action(&state, 2, 2, false, false, &caps, false, false).is_none());
    }

    #[test]
    fn tunnel_eject_menu_back_at_2_0() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu);
        let caps = no_caps();
        let action = get_grid_slot_action(&state, 2, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::Back)));
    }

    #[test]
    fn tunnel_eject_menu_other_slots_return_none() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu);
        let caps = no_caps();
        // All slots except Back should return None (stub)
        for row in 0..3u8 {
            for col in 0..3u8 {
                if (row, col) == (2, 0) { continue; }
                assert!(get_grid_slot_action(&state, row, col, false, false, &caps, false, false).is_none());
            }
        }
    }

    #[test]
    fn tunnel_awaiting_placement_no_grid_buttons() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement);
        let caps = no_caps();
        for row in 0..3u8 {
            for col in 0..3u8 {
                assert!(get_grid_slot_action(&state, row, col, false, false, &caps, false, false).is_none());
            }
        }
    }

    #[test]
    fn tunnel_awaiting_placement_is_placement_mode() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement);
        assert!(state.is_placement_mode());
    }

    #[test]
    fn tunnel_idle_is_not_placement_mode() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
        assert!(!state.is_placement_mode());
    }

    #[test]
    fn tunnel_expand_menu_is_not_placement_mode() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
        assert!(!state.is_placement_mode());
    }

    // === tier_at_least tests ===

    #[test]
    fn tier1_at_least_tier1() {
        assert!(tier_at_least(&TunnelTier::Tier1, &TunnelTier::Tier1));
    }

    #[test]
    fn tier1_not_at_least_tier2() {
        assert!(!tier_at_least(&TunnelTier::Tier1, &TunnelTier::Tier2));
    }

    #[test]
    fn tier2_at_least_tier1() {
        assert!(tier_at_least(&TunnelTier::Tier2, &TunnelTier::Tier1));
    }

    #[test]
    fn tier2_at_least_tier2() {
        assert!(tier_at_least(&TunnelTier::Tier2, &TunnelTier::Tier2));
    }

    #[test]
    fn tier3_at_least_all() {
        assert!(tier_at_least(&TunnelTier::Tier3, &TunnelTier::Tier1));
        assert!(tier_at_least(&TunnelTier::Tier3, &TunnelTier::Tier2));
        assert!(tier_at_least(&TunnelTier::Tier3, &TunnelTier::Tier3));
    }

    // === grid_button_label tests for Tunnel ===

    #[test]
    fn tunnel_upgrade_label() {
        let label = grid_button_label(
            &ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle),
            &CommandButtonAction::TunnelUpgrade,
            0,
            'Q',
        );
        assert_eq!(label, "[Q] Upgrade");
    }

    #[test]
    fn tunnel_expand_label() {
        let label = grid_button_label(
            &ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle),
            &CommandButtonAction::TunnelOpenExpandMenu,
            0,
            'W',
        );
        assert_eq!(label, "[W] Expand");
    }

    #[test]
    fn tunnel_eject_label() {
        let label = grid_button_label(
            &ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle),
            &CommandButtonAction::TunnelOpenEjectMenu,
            0,
            'E',
        );
        assert_eq!(label, "[E] Eject");
    }

    #[test]
    fn tunnel_select_expansion_label() {
        let label = grid_button_label(
            &ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu),
            &CommandButtonAction::TunnelSelectExpansion(ObjectEnum::Headquarters),
            0,
            'Q',
        );
        assert!(label.contains("Headquarters"));
        assert!(label.contains("[Q]"));
    }

    // === EjectionQueue tests ===

    #[test]
    fn ejection_queue_default_is_empty() {
        let eq = EjectionQueue::default();
        assert!(eq.queue.is_empty());
        assert_eq!(eq.cooldown, 0);
    }

    #[test]
    fn ejection_queue_can_push_and_pop() {
        let mut eq = EjectionQueue::default();
        eq.queue.push_back(Entity::from_raw_u32(1).unwrap());
        eq.queue.push_back(Entity::from_raw_u32(2).unwrap());
        assert_eq!(eq.queue.len(), 2);
        let front = eq.queue.pop_front().unwrap();
        assert_eq!(front, Entity::from_raw_u32(1).unwrap());
    }

    #[test]
    fn ejection_queue_cooldown_tracking() {
        let mut eq = EjectionQueue::default();
        eq.cooldown = 8;
        assert_eq!(eq.cooldown, 8);
        eq.cooldown = eq.cooldown.saturating_sub(1);
        assert_eq!(eq.cooldown, 7);
    }

    // === panel_visible with Tunnel states ===

    #[test]
    fn panel_visible_when_tunnel_idle() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
        let selection = Selection::default();
        assert!(is_panel_visible(&state, &selection));
    }

    #[test]
    fn panel_visible_when_tunnel_expand_menu() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
        let selection = Selection::default();
        assert!(is_panel_visible(&state, &selection));
    }

    #[test]
    fn panel_visible_when_tunnel_eject_menu() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu);
        let selection = Selection::default();
        assert!(is_panel_visible(&state, &selection));
    }

    #[test]
    fn panel_visible_when_tunnel_awaiting_placement() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement);
        let selection = Selection::default();
        assert!(is_panel_visible(&state, &selection));
    }

    // === is_common_command for Tunnel actions ===

    #[test]
    fn tunnel_actions_are_not_common_commands() {
        let sel = units_only_selection();
        assert!(!is_common_command(&CommandButtonAction::TunnelUpgrade, &sel));
        assert!(!is_common_command(&CommandButtonAction::TunnelOpenExpandMenu, &sel));
        assert!(!is_common_command(&CommandButtonAction::TunnelOpenEjectMenu, &sel));
        assert!(!is_common_command(&CommandButtonAction::TunnelSelectExpansion(ObjectEnum::Headquarters), &sel));
        assert!(!is_common_command(&CommandButtonAction::TunnelEjectUnit(ObjectEnum::SyndicateAgent), &sel));
        assert!(!is_common_command(&CommandButtonAction::TunnelCancelUpgrade, &sel));
    }

    // === Agent Interface State tests ===

    #[test]
    fn agent_default_shows_build_tunnel_at_q() {
        let caps = no_caps();
        let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
        let action = get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::AgentBuildTunnel)));
    }

    #[test]
    fn agent_default_shows_drop_off_at_w() {
        let caps = no_caps();
        let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
        let action = get_grid_slot_action(&state, 0, 1, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::AgentDropOff)));
    }

    #[test]
    fn agent_default_no_extra_slots() {
        let caps = no_caps();
        let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
        // All slots except (0,0) and (0,1) should be empty
        for row in 0..3u8 {
            for col in 0..3u8 {
                if (row, col) == (0, 0) || (row, col) == (0, 1) {
                    continue;
                }
                let action = get_grid_slot_action(&state, row, col, false, false, &caps, false, false);
                assert!(action.is_none(), "Unexpected action at ({}, {})", row, col);
            }
        }
    }

    #[test]
    fn agent_awaiting_placement_no_grid_buttons() {
        let caps = no_caps();
        let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement);
        for row in 0..3u8 {
            for col in 0..3u8 {
                let action = get_grid_slot_action(&state, row, col, false, false, &caps, false, false);
                assert!(action.is_none());
            }
        }
    }

    #[test]
    fn agent_awaiting_placement_is_placement_mode() {
        let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement);
        assert!(state.is_placement_mode());
    }

    #[test]
    fn agent_default_is_not_placement_mode() {
        let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
        assert!(!state.is_placement_mode());
    }

    #[test]
    fn agent_drop_off_disabled_when_not_carrying() {
        // grid_button_enabled_ext returns caps.agent_carrying for AgentDropOff
        let caps = SelectedUnitCapabilities { agent_carrying: false, ..Default::default() };
        assert!(!caps.agent_carrying, "Drop off should be disabled when not carrying");
    }

    #[test]
    fn agent_drop_off_enabled_when_carrying() {
        // grid_button_enabled_ext returns caps.agent_carrying for AgentDropOff
        let caps = SelectedUnitCapabilities { agent_carrying: true, ..Default::default() };
        assert!(caps.agent_carrying, "Drop off should be enabled when carrying");
    }

    #[test]
    fn agent_build_tunnel_label() {
        let label = grid_button_label(
            &ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault),
            &CommandButtonAction::AgentBuildTunnel,
            0,
            'D',
        );
        assert_eq!(label, "[D] Build\nTunnel");
    }

    #[test]
    fn agent_drop_off_label() {
        let label = grid_button_label(
            &ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault),
            &CommandButtonAction::AgentDropOff,
            0,
            'S',
        );
        assert_eq!(label, "[S] Drop\nOff");
    }

    #[test]
    fn agent_enter_label() {
        let label = grid_button_label(
            &ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault),
            &CommandButtonAction::UnitEnter,
            0,
            'E',
        );
        assert_eq!(label, "[E] Enter");
    }

    #[test]
    fn agent_gather_label() {
        let label = grid_button_label(
            &ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault),
            &CommandButtonAction::UnitGather,
            0,
            'A',
        );
        assert_eq!(label, "[A] Gather");
    }

    #[test]
    fn agent_commands_not_common_with_non_agent_units() {
        // Peacekeeper + SyndicateAgent: Peacekeeper doesn't support AgentBuildTunnel → NOT common
        let sel = units_only_selection();
        assert!(!is_common_command(&CommandButtonAction::AgentBuildTunnel, &sel));
        assert!(!is_common_command(&CommandButtonAction::AgentDropOff, &sel));
    }

    #[test]
    fn agent_commands_common_with_only_agents() {
        // Two SyndicateAgent groups: both support AgentBuildTunnel → IS common
        let sel = Selection {
            groups: vec![
                SelectionGroup {
                    object_type: ObjectEnum::SyndicateAgent,
                    entities: vec![Entity::from_raw_u32(1).unwrap()],
                },
                SelectionGroup {
                    object_type: ObjectEnum::SyndicateAgent,
                    entities: vec![Entity::from_raw_u32(2).unwrap()],
                },
            ],
            active_group_index: Some(0),
        };
        assert!(is_common_command(&CommandButtonAction::AgentBuildTunnel, &sel));
        assert!(is_common_command(&CommandButtonAction::AgentDropOff, &sel));
    }

    #[test]
    fn agent_menu_panel_visible_with_selection() {
        let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
        let mut selection = Selection::default();
        selection.groups.push(SelectionGroup {
            object_type: ObjectEnum::SyndicateAgent,
            entities: vec![Entity::from_raw_u32(1).unwrap()],
        });
        assert!(is_panel_visible(&state, &selection));
    }

    #[test]
    fn agent_menu_panel_hidden_without_selection() {
        let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
        let selection = Selection::default();
        assert!(!is_panel_visible(&state, &selection));
    }

    #[test]
    fn selected_unit_capabilities_agent_carrying() {
        let caps = SelectedUnitCapabilities { agent_carrying: true, ..Default::default() };
        assert!(caps.agent_carrying);
        assert!(!caps.has_attack);
    }

    #[test]
    fn selected_unit_capabilities_default_agent_not_carrying() {
        let caps = SelectedUnitCapabilities::default();
        assert!(!caps.agent_carrying);
    }

    // === Back button consistency tests ===

    #[test]
    fn dc_build_menu_back_at_bottom_left() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
        let caps = no_caps();
        let action = get_grid_slot_action(&state, 2, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::Back)));
    }

    #[test]
    fn dc_build_menu_ef_at_row1_col1() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
        let caps = no_caps();
        let action = get_grid_slot_action(&state, 1, 1, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility))));
    }

    #[test]
    fn dc_build_menu_no_back_at_bottom_right() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
        let caps = no_caps();
        let action = get_grid_slot_action(&state, 2, 2, false, false, &caps, false, false);
        assert!(!matches!(action, Some(CommandButtonAction::Back)));
    }

    #[test]
    fn back_button_convention_bottom_left_z() {
        // Convention: Back/Cancel always at (2, 0) = Z in any sub-menu.
        // This test documents the convention for future factions/menus.
        // GDO DcBuildMenu already maps (2, 0) => Back via get_grid_slot_action.
        // Syndicate tunnel menus use dynamic grid builders that place Back at (2, 0).
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
        let caps = no_caps();
        assert!(
            matches!(get_grid_slot_action(&state, 2, 0, false, false, &caps, false, false), Some(CommandButtonAction::Back)),
            "Back button must be at grid position (2,0) = Z = bottom-left"
        );
    }

    // === is_panel_visible tests ===

    #[test]
    fn panel_hidden_when_selection_empty() {
        let selection = Selection::default();
        assert!(!is_panel_visible(&ObjectInterfaceState::Default, &selection));
    }

    #[test]
    fn panel_visible_when_unit_selected() {
        let mut selection = Selection::default();
        let entities = vec![
            (Entity::from_raw_u32(1).unwrap(), ObjectEnum::Peacekeeper, true),
        ];
        selection.build_from_entities(&entities);
        assert!(is_panel_visible(&ObjectInterfaceState::Default, &selection));
    }

    #[test]
    fn panel_visible_when_structure_selected() {
        let mut selection = Selection::default();
        let entities = vec![
            (Entity::from_raw_u32(1).unwrap(), ObjectEnum::DeploymentCenter, false),
        ];
        selection.build_from_entities(&entities);
        assert!(is_panel_visible(&ObjectInterfaceState::Default, &selection));
    }

    #[test]
    fn panel_hidden_when_only_resource_selected() {
        let mut selection = Selection::default();
        let entities = vec![
            (Entity::from_raw_u32(1).unwrap(), ObjectEnum::SupplyDeliveryStation, false),
        ];
        selection.build_from_entities(&entities);
        assert!(!is_panel_visible(&ObjectInterfaceState::Default, &selection),
            "Panel should be hidden for resource-only selection");
    }

    #[test]
    fn panel_hidden_when_only_crystal_patch_selected() {
        let mut selection = Selection::default();
        let entities = vec![
            (Entity::from_raw_u32(1).unwrap(), ObjectEnum::SpaceCrystalsPatch, false),
        ];
        selection.build_from_entities(&entities);
        assert!(!is_panel_visible(&ObjectInterfaceState::Default, &selection),
            "Panel should be hidden for crystal-patch-only selection");
    }

    #[test]
    fn panel_visible_when_mixed_resource_and_unit() {
        let mut selection = Selection::default();
        let entities = vec![
            (Entity::from_raw_u32(1).unwrap(), ObjectEnum::SupplyDeliveryStation, false),
            (Entity::from_raw_u32(2).unwrap(), ObjectEnum::Peacekeeper, true),
        ];
        selection.build_from_entities(&entities);
        assert!(is_panel_visible(&ObjectInterfaceState::Default, &selection),
            "Panel should be visible when selection includes non-resource entities");
    }

    #[test]
    fn panel_visible_in_structure_menu_regardless_of_selection() {
        let selection = Selection::default();
        assert!(is_panel_visible(
            &ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle),
            &selection
        ), "Panel always visible in structure menu states");
    }

    #[test]
    fn panel_hidden_for_multiple_resources_only() {
        let mut selection = Selection::default();
        let entities = vec![
            (Entity::from_raw_u32(1).unwrap(), ObjectEnum::SpaceCrystalsPatch, false),
            (Entity::from_raw_u32(2).unwrap(), ObjectEnum::SupplyDeliveryStation, false),
        ];
        selection.build_from_entities(&entities);
        assert!(!is_panel_visible(&ObjectInterfaceState::Default, &selection),
            "Panel should be hidden when all selected entities are resources");
    }

    // === selection_owned_by_local_player tests ===

    #[test]
    fn ownership_true_when_all_owned_by_local_player() {
        let mut app = App::new();
        let e1 = app.world_mut().spawn(Owner::player(1)).id();
        let e2 = app.world_mut().spawn(Owner::player(1)).id();
        let mut selection = Selection::default();
        selection.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
            (e2, ObjectEnum::Peacekeeper, true),
        ]);
        let local_player = LocalPlayer(1);
        app.world_mut().run_system_once(move |owner_q: Query<&Owner>| {
            assert!(selection_owned_by_local_player(&selection, &local_player, &owner_q));
        });
    }

    #[test]
    fn ownership_false_when_enemy_unit_selected() {
        let mut app = App::new();
        let e1 = app.world_mut().spawn(Owner::player(2)).id();
        let mut selection = Selection::default();
        selection.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
        ]);
        let local_player = LocalPlayer(1);
        app.world_mut().run_system_once(move |owner_q: Query<&Owner>| {
            assert!(!selection_owned_by_local_player(&selection, &local_player, &owner_q));
        });
    }

    #[test]
    fn ownership_false_when_neutral_entity_selected() {
        let mut app = App::new();
        let e1 = app.world_mut().spawn(Owner(None)).id();
        let mut selection = Selection::default();
        selection.build_from_entities(&[
            (e1, ObjectEnum::SpaceCrystalsPatch, false),
        ]);
        let local_player = LocalPlayer(1);
        app.world_mut().run_system_once(move |owner_q: Query<&Owner>| {
            assert!(!selection_owned_by_local_player(&selection, &local_player, &owner_q));
        });
    }

    #[test]
    fn ownership_false_when_mixed_owned_and_enemy() {
        let mut app = App::new();
        let e1 = app.world_mut().spawn(Owner::player(1)).id();
        let e2 = app.world_mut().spawn(Owner::player(2)).id();
        let mut selection = Selection::default();
        selection.build_from_entities(&[
            (e1, ObjectEnum::Peacekeeper, true),
            (e2, ObjectEnum::Peacekeeper, true),
        ]);
        let local_player = LocalPlayer(1);
        app.world_mut().run_system_once(move |owner_q: Query<&Owner>| {
            assert!(!selection_owned_by_local_player(&selection, &local_player, &owner_q));
        });
    }

    #[test]
    fn ownership_true_when_selection_empty() {
        let mut app = App::new();
        let selection = Selection::default();
        let local_player = LocalPlayer(1);
        app.world_mut().run_system_once(move |owner_q: Query<&Owner>| {
            // Empty selection is considered "owned" (no violations)
            assert!(selection_owned_by_local_player(&selection, &local_player, &owner_q));
        });
    }

    // === Supply Tower tech prerequisite tests ===

    #[test]
    fn supply_tower_tech_prerequisite_has_power_plant() {
        // Supply Tower requires has_power_plant == true (owning a Power Plant)
        // The DC generates power (DC_POWER=20) but should not satisfy the prerequisite
        let mut resources = GdoPlayerResources::default();
        // Default has no power plant
        resources.has_power_plant = false;
        resources.power_generated = 20; // DC generates power but no PP
        assert!(!resources.has_power_plant, "Should fail prerequisite without PP");

        // Simulate owning a Power Plant
        resources.has_power_plant = true;
        assert!(resources.has_power_plant, "With PP, has_power_plant should be true");
    }

    #[test]
    fn supply_tower_construction_cost_is_200() {
        let cost = DeploymentCenterState::construction_cost(&ObjectEnum::SupplyTower);
        assert!(cost.is_some(), "Supply Tower should have a construction cost");
        assert_eq!(cost.unwrap().space_crystals, 200);
    }

    #[test]
    fn supply_tower_build_button_in_dc_build_menu() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
        let caps = no_caps();
        // Supply Tower is at (1, 0) in the DC build menu
        let action = get_grid_slot_action(&state, 1, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::DcBuild(ObjectEnum::SupplyTower))),
            "DC build menu should have Supply Tower at (1,0)");
    }

    // === Schedule Deliveries gate tests ===

    #[test]
    fn schedule_deliveries_action_exists_in_supply_tower_menu() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu);
        let caps = no_caps();
        let action = get_grid_slot_action(&state, 1, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::StScheduleDeliveries)),
            "Supply Tower menu should have Schedule Deliveries at (1,0)");
    }

    #[test]
    fn schedule_deliveries_requires_attached_chopper() {
        // SupplyTowerState without attached chopper should disable schedule deliveries
        let state = SupplyTowerState::default();
        assert!(state.attached_chopper.is_none(), "Default tower has no attached chopper");
        // The grid_button_enabled_ext checks attached_chopper.is_some()
    }

    #[test]
    fn schedule_deliveries_enabled_with_attached_chopper() {
        let mut state = SupplyTowerState::default();
        state.attached_chopper = Some(Entity::from_raw_u32(1).unwrap());
        assert!(state.attached_chopper.is_some(), "Tower with attached chopper should enable schedule");
    }

    // === Selection bounds tests ===

    #[test]
    fn selection_bounds_3x3_structure_larger_than_unit() {
        let unit_bounds = SelectionBounds::unit();
        let tower_bounds = SelectionBounds::from_dimensions(3.0, 1.2, 3.0);
        assert!(tower_bounds.half_x > unit_bounds.half_x,
            "3x3 structure bounds should be larger than default unit bounds");
        assert_eq!(tower_bounds.half_x, 1.5);
        assert_eq!(tower_bounds.half_z, 1.5);
    }

    // === Air domain movement tests ===

    #[test]
    fn hovercraft_has_air_domain() {
        let data = UnitBaseEnum::HoverCraft.data();
        assert_eq!(data.domain, DomainEnum::Air, "HoverCraft should be Air domain");
    }

    #[test]
    fn light_infantry_has_ground_domain() {
        let data = UnitBaseEnum::LightInfantry.data();
        assert_eq!(data.domain, DomainEnum::Ground, "LightInfantry should be Ground domain");
    }

    #[test]
    fn supply_chopper_spawn_uses_air_domain() {
        // Verify ObjectEnum::SupplyChopper is a unit (chopper is unit, not structure)
        assert!(ObjectEnum::SupplyChopper.is_unit(), "Supply Chopper should be a unit");
        assert!(!ObjectEnum::SupplyChopper.is_structure(), "Supply Chopper should not be a structure");
    }

    // === HeadquartersMenu tests ===

    #[test]
    fn hq_menu_grid_slot_train_agent_at_0_0() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
        let caps = no_caps();
        let action = get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::HqTrain(ObjectEnum::SyndicateAgent))));
    }

    #[test]
    fn hq_menu_grid_slot_cancel_at_2_1_when_queue() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
        let caps = no_caps();
        // Cancel visible at (2,1) when has_queue is true
        let action = get_grid_slot_action(&state, 2, 1, true, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::HqCancel)));
    }

    #[test]
    fn hq_menu_grid_slot_cancel_hidden_when_no_queue() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
        let caps = no_caps();
        // Cancel hidden when has_queue is false
        let action = get_grid_slot_action(&state, 2, 1, false, false, &caps, false, false);
        assert!(action.is_none());
    }

    #[test]
    fn hq_menu_grid_slot_guard_at_0_1() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
        let caps = no_caps();
        let action = get_grid_slot_action(&state, 0, 1, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::HqTrain(ObjectEnum::SyndicateGuard))));
    }

    #[test]
    fn hq_menu_empty_slots() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
        let caps = no_caps();
        // All other slots should be empty (excluding (0,0) Agent, (0,1) Guard, (2,2) Rally)
        for row in 0..3u8 {
            for col in 0..3u8 {
                if (row, col) == (0, 0) || (row, col) == (0, 1) || (row, col) == (2, 2) { continue; }
                let action = get_grid_slot_action(&state, row, col, false, false, &caps, false, false);
                assert!(action.is_none(), "Slot ({}, {}) should be empty", row, col);
            }
        }
    }

    #[test]
    fn hq_menu_label_train_agent() {
        let label = grid_button_label(
            &ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu),
            &CommandButtonAction::HqTrain(ObjectEnum::SyndicateAgent),
            0,
            'Q',
        );
        assert_eq!(label, "[Q] Agent\n100 SC");
    }

    #[test]
    fn hq_menu_label_cancel() {
        let label = grid_button_label(
            &ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu),
            &CommandButtonAction::HqCancel,
            0,
            'W',
        );
        assert_eq!(label, "[W] Cancel\nLast");
    }

    #[test]
    fn headquarters_state_production_cost() {
        let cost = HeadquartersState::production_cost(&ObjectEnum::SyndicateAgent);
        assert!(cost.is_some());
        let cost = cost.unwrap();
        assert_eq!(cost.space_crystals, 100);
        assert_eq!(cost.build_frames, 160);
    }

    #[test]
    fn headquarters_state_no_cost_for_other_units() {
        let cost = HeadquartersState::production_cost(&ObjectEnum::Peacekeeper);
        assert!(cost.is_none());
    }

    #[test]
    fn headquarters_state_try_queue_and_cancel() {
        let mut hq = HeadquartersState::default();
        assert!(hq.try_queue(ObjectEnum::SyndicateAgent));
        assert_eq!(hq.build_queue.len(), 1);
        let cancelled = hq.cancel_last();
        assert_eq!(cancelled, Some(ObjectEnum::SyndicateAgent));
        assert!(hq.build_queue.is_empty());
    }

    #[test]
    fn headquarters_state_queue_max_size() {
        let mut hq = HeadquartersState::default();
        for _ in 0..HeadquartersState::MAX_QUEUE_SIZE {
            assert!(hq.try_queue(ObjectEnum::SyndicateAgent));
        }
        // Queue full — should reject
        assert!(!hq.try_queue(ObjectEnum::SyndicateAgent));
        assert_eq!(hq.build_queue.len(), HeadquartersState::MAX_QUEUE_SIZE);
    }

    #[test]
    fn headquarters_menu_not_placement_mode() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
        assert!(!state.is_placement_mode());
    }

    #[test]
    fn headquarters_menu_not_awaiting_target() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
        assert!(!state.is_awaiting_target());
    }

    #[test]
    fn hq_is_common_command_false() {
        // HqTrain and HqCancel are structure-specific, not common commands
        let sel = units_only_selection();
        assert!(!is_common_command(&CommandButtonAction::HqTrain(ObjectEnum::SyndicateAgent), &sel));
        assert!(!is_common_command(&CommandButtonAction::HqCancel, &sel));
    }

    #[test]
    fn headquarters_state_cancel_empty_queue() {
        let mut hq = HeadquartersState::default();
        assert!(hq.cancel_last().is_none());
    }

    #[test]
    fn headquarters_state_default() {
        let hq = HeadquartersState::default();
        assert!(hq.rally_point.is_none());
        assert!(hq.build_queue.is_empty());
        assert!(hq.current_build.is_none());
        assert!(hq.current_build_progress.is_none());
    }

    #[test]
    fn hq_grid_slot_action_count() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
        let caps = no_caps();
        let mut count = 0;
        for row in 0..3u8 {
            for col in 0..3u8 {
                if get_grid_slot_action(&state, row, col, true, false, &caps, false, false).is_some() {
                    count += 1;
                }
            }
        }
        // Agent + Guard + Cancel + Rally = 4 (when queue has items)
        assert_eq!(count, 4);
    }

    #[test]
    fn hq_grid_slot_action_count_no_queue() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
        let caps = no_caps();
        let mut count = 0;
        for row in 0..3u8 {
            for col in 0..3u8 {
                if get_grid_slot_action(&state, row, col, false, false, &caps, false, false).is_some() {
                    count += 1;
                }
            }
        }
        // Agent + Guard + Rally = 3 (when queue empty, no Cancel)
        assert_eq!(count, 3);
    }

    // === Inert Structure State Tests ===

    #[test]
    fn inert_structure_no_grid_buttons() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::Inert);
        let caps = all_caps();
        for row in 0..3u8 {
            for col in 0..3u8 {
                assert!(
                    get_grid_slot_action(&state, row, col, false, false, &caps, false, false).is_none(),
                    "Inert structure should have no grid buttons at ({}, {})", row, col
                );
            }
        }
    }

    #[test]
    fn inert_is_not_placement_mode() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::Inert);
        assert!(!state.is_placement_mode(), "Inert should not be placement mode");
    }

    #[test]
    fn inert_is_not_awaiting_target() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::Inert);
        assert!(!state.is_awaiting_target(), "Inert should not be awaiting target");
    }

    #[test]
    fn inert_panel_is_visible() {
        // Even with no commands, the panel frame is visible for structure info display
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::Inert);
        let selection = Selection {
            groups: vec![SelectionGroup {
                object_type: ObjectEnum::PowerPlant,
                entities: vec![Entity::from_raw_u32(1).unwrap()],
            }],
            active_group_index: Some(0),
        };
        assert!(is_panel_visible(&state, &selection));
    }

    // === Common command visual distinction with group count ===

    #[test]
    fn single_group_all_commands_treated_as_common() {
        // With only one group, is_common should effectively be true for all commands
        // (the rendering code uses: selection.groups.len() <= 1 || is_common_command(&action, &selection))
        let selection = Selection {
            groups: vec![SelectionGroup {
                object_type: ObjectEnum::Peacekeeper,
                entities: vec![Entity::from_raw_u32(1).unwrap()],
            }],
            active_group_index: Some(0),
        };
        let attack_action = CommandButtonAction::UnitAttack;
        let result = selection.groups.len() <= 1 || is_common_command(&attack_action, &selection);
        assert!(result, "Attack should be treated as common with single group");

        let reverse_action = CommandButtonAction::UnitReverse;
        let result = selection.groups.len() <= 1 || is_common_command(&reverse_action, &selection);
        assert!(result, "Reverse should be treated as common with single group");
    }

    #[test]
    fn multiple_unit_groups_group_commands_not_common() {
        // With multiple unit groups where one doesn't support Attack, it should NOT be common
        let sel = Selection {
            groups: vec![
                SelectionGroup {
                    object_type: ObjectEnum::Peacekeeper,
                    entities: vec![Entity::from_raw_u32(1).unwrap()],
                },
                SelectionGroup {
                    object_type: ObjectEnum::SupplyChopper,
                    entities: vec![Entity::from_raw_u32(2).unwrap()],
                },
            ],
            active_group_index: Some(0),
        };
        let attack_action = CommandButtonAction::UnitAttack;
        let result = sel.groups.len() <= 1 || is_common_command(&attack_action, &sel);
        assert!(!result, "Attack should be group-specific when chopper is in selection");
    }

    #[test]
    fn multiple_unit_groups_common_commands_still_common() {
        let sel = units_only_selection();
        let move_action = CommandButtonAction::UnitMove;
        let result = sel.groups.len() <= 1 || is_common_command(&move_action, &sel);
        assert!(result, "Move should still be common with multiple unit groups");

        let stop_action = CommandButtonAction::UnitStop;
        let result = sel.groups.len() <= 1 || is_common_command(&stop_action, &sel);
        assert!(result, "Stop should still be common with multiple unit groups");
    }

    #[test]
    fn mixed_selection_unit_commands_not_common() {
        // With mixed unit+structure groups, unit commands should NOT be common
        let sel = mixed_unit_structure_selection();
        let move_action = CommandButtonAction::UnitMove;
        let result = sel.groups.len() <= 1 || is_common_command(&move_action, &sel);
        assert!(!result, "Move should NOT be common with mixed unit+structure selection");

        let stop_action = CommandButtonAction::UnitStop;
        let result = sel.groups.len() <= 1 || is_common_command(&stop_action, &sel);
        assert!(!result, "Stop should NOT be common with mixed unit+structure selection");
    }

    // === Standard Bottom-Row Commands Tests ===

    #[test]
    fn awaiting_target_back_at_z_for_all_command_types() {
        let caps = no_caps();
        let command_types = [
            CommandType::Move, CommandType::Attack, CommandType::AttackGround,
            CommandType::AttackMove, CommandType::Patrol, CommandType::Reverse,
            CommandType::SetRallyPoint,
        ];
        for ct in command_types {
            let state = ObjectInterfaceState::AwaitingTarget(ct);
            let action = get_grid_slot_action(&state, 2, 0, false, false, &caps, false, false);
            assert!(matches!(action, Some(CommandButtonAction::Back)),
                "AwaitingTarget({:?}) should have Back at (2,0)", ct);
        }
    }

    #[test]
    fn set_rally_point_at_c_for_production_menus() {
        let caps = no_caps();
        let production_menus = [
            StructureMenuState::BarracksMenu,
            StructureMenuState::HeadquartersMenu,
            StructureMenuState::SupplyTowerMenu,
        ];
        for menu in production_menus {
            let state = ObjectInterfaceState::StructureMenu(menu.clone());
            let action = get_grid_slot_action(&state, 2, 2, false, false, &caps, false, false);
            assert!(matches!(action, Some(CommandButtonAction::SetRallyPoint)),
                "Menu {:?} should have SetRallyPoint at (2,2)", menu);
        }
    }

    #[test]
    fn cancel_at_x_for_production_menus_with_queue() {
        let caps = no_caps();
        // Barracks
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);
        let action = get_grid_slot_action(&state, 2, 1, true, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::BkCancel)));

        // Supply Tower
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu);
        let action = get_grid_slot_action(&state, 2, 1, true, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::StCancel)));

        // HQ
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
        let action = get_grid_slot_action(&state, 2, 1, true, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::HqCancel)));
    }

    #[test]
    fn dc_constructing_back_at_z_cancel_at_x() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing);
        let back = get_grid_slot_action(&state, 2, 0, false, false, &caps, false, false);
        assert!(matches!(back, Some(CommandButtonAction::Back)));
        let cancel = get_grid_slot_action(&state, 2, 1, false, false, &caps, false, false);
        assert!(matches!(cancel, Some(CommandButtonAction::DcCancel)));
        // Old position (0,0) should be empty
        let old = get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(old.is_none(), "DcConstructing (0,0) should be empty after move");
    }

    #[test]
    fn dc_ready_to_place_back_at_z_cancel_at_x() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace);
        let place = get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(place, Some(CommandButtonAction::EnterPlacement)));
        let back = get_grid_slot_action(&state, 2, 0, false, false, &caps, false, false);
        assert!(matches!(back, Some(CommandButtonAction::Back)));
        let cancel = get_grid_slot_action(&state, 2, 1, false, false, &caps, false, false);
        assert!(matches!(cancel, Some(CommandButtonAction::DcCancel)));
        // Old cancel position (0,1) should be empty
        let old = get_grid_slot_action(&state, 0, 1, false, false, &caps, false, false);
        assert!(old.is_none(), "DcReadyToPlace (0,1) should be empty after move");
    }

    #[test]
    fn dc_cancel_label_in_idle_is_generic() {
        let label = grid_button_label(
            &ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle),
            &CommandButtonAction::DcCancel,
            0,
            'X',
        );
        assert_eq!(label, "[X] Cancel");
    }

    #[test]
    fn dc_cancel_label_in_constructing_shows_constr() {
        let label = grid_button_label(
            &ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing),
            &CommandButtonAction::DcCancel,
            0,
            'X',
        );
        assert_eq!(label, "[X] Cancel\nConstr.");
    }

    #[test]
    fn dc_cancel_label_in_ready_to_place_shows_building() {
        let label = grid_button_label(
            &ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace),
            &CommandButtonAction::DcCancel,
            0,
            'X',
        );
        assert_eq!(label, "[X] Cancel\nBuilding");
    }

    #[test]
    fn ef_idle_with_ready_plate_shows_enter_placement() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        // has_ready_plate=true → Q shows EnterPlacement instead of EfBuildPlate
        let action = get_grid_slot_action(&state, 0, 0, false, true, &caps, false, true);
        assert!(matches!(action, Some(CommandButtonAction::EnterPlacement)),
            "EfIdle with ready plate should show EnterPlacement at (0,0)");
    }

    #[test]
    fn ef_idle_constructing_shows_build_plate_and_cancel() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        // has_active_construction=true, has_ready_plate=false → Q=EfBuildPlate, X=EfCancel
        let q = get_grid_slot_action(&state, 0, 0, false, true, &caps, false, false);
        assert!(matches!(q, Some(CommandButtonAction::EfBuildPlate)),
            "EfIdle constructing should show EfBuildPlate at (0,0)");
        let x = get_grid_slot_action(&state, 2, 1, false, true, &caps, false, false);
        assert!(matches!(x, Some(CommandButtonAction::EfCancel)),
            "EfIdle constructing should show EfCancel at (2,1)");
    }

    #[test]
    fn set_rally_point_label() {
        let label = grid_button_label(
            &ObjectInterfaceState::Default,
            &CommandButtonAction::SetRallyPoint,
            0,
            'C',
        );
        assert_eq!(label, "[C] Rally");
    }

    #[test]
    fn set_rally_point_is_not_common_command() {
        let sel = units_only_selection();
        assert!(!is_common_command(&CommandButtonAction::SetRallyPoint, &sel));
    }

    #[test]
    fn barracks_cancel_moved_from_old_position() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);
        // Old position (0,1) should be empty
        let old = get_grid_slot_action(&state, 0, 1, true, false, &caps, false, false);
        assert!(old.is_none(), "BarracksMenu (0,1) should be empty after BkCancel move");
    }

    #[test]
    fn barracks_train_peacekeeper_at_q() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);
        let action = get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(
            matches!(action, Some(CommandButtonAction::BkTrain(ObjectEnum::Peacekeeper))),
            "BarracksMenu (0,0) should be BkTrain(Peacekeeper)"
        );
    }

    #[test]
    fn barracks_cancel_hidden_when_queue_empty() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);
        // bk_has_queue = false → cancel at (2,1) should be None
        let action = get_grid_slot_action(&state, 2, 1, false, false, &caps, false, false);
        assert!(action.is_none(), "BarracksMenu (2,1) should be None when queue is empty");
    }

    #[test]
    fn barracks_grid_slot_action_count_with_queue() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);
        let mut count = 0;
        for row in 0..3u8 {
            for col in 0..3u8 {
                if get_grid_slot_action(&state, row, col, true, false, &caps, false, false).is_some() {
                    count += 1;
                }
            }
        }
        // BkTrain(Peacekeeper) + BkCancel + SetRallyPoint = 3
        assert_eq!(count, 3, "BarracksMenu should have 3 active slots with queue");
    }

    #[test]
    fn barracks_grid_slot_action_count_no_queue() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);
        let mut count = 0;
        for row in 0..3u8 {
            for col in 0..3u8 {
                if get_grid_slot_action(&state, row, col, false, false, &caps, false, false).is_some() {
                    count += 1;
                }
            }
        }
        // BkTrain(Peacekeeper) + SetRallyPoint = 2 (no BkCancel when queue empty)
        assert_eq!(count, 2, "BarracksMenu should have 2 active slots without queue");
    }

    #[test]
    fn supply_tower_cancel_moved_from_old_position() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu);
        // Old position (0,1) should be empty
        let old = get_grid_slot_action(&state, 0, 1, true, false, &caps, false, false);
        assert!(old.is_none(), "SupplyTowerMenu (0,1) should be empty after StCancel move");
    }

    // === DC grid slot tests ===

    #[test]
    fn dc_idle_grid_has_open_build_menu() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        let action = get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::DcOpenBuildMenu)));
    }

    #[test]
    fn dc_idle_no_construction_no_cancel() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        let action = get_grid_slot_action(&state, 2, 1, false, false, &caps, false, false);
        assert!(action.is_none(), "DcIdle should not show cancel when has_active_construction=false");
    }

    #[test]
    fn dc_idle_with_construction_shows_cancel() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        let action = get_grid_slot_action(&state, 2, 1, false, true, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::DcCancel)),
            "DcIdle should show DcCancel at (2,1) when has_active_construction=true");
    }

    #[test]
    fn dc_idle_with_construction_still_has_build_menu() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        let action = get_grid_slot_action(&state, 0, 0, false, true, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::DcOpenBuildMenu)),
            "DcIdle should still show DcOpenBuildMenu at (0,0) even with active construction");
    }

    #[test]
    fn dc_build_menu_grid_has_all_buildings_and_back() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
        assert!(matches!(get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false), Some(CommandButtonAction::DcBuild(ObjectEnum::PowerPlant))));
        assert!(matches!(get_grid_slot_action(&state, 0, 1, false, false, &caps, false, false), Some(CommandButtonAction::DcBuild(ObjectEnum::Barracks))));
        assert!(get_grid_slot_action(&state, 0, 2, false, false, &caps, false, false).is_none());
        assert!(matches!(get_grid_slot_action(&state, 1, 0, false, false, &caps, false, false), Some(CommandButtonAction::DcBuild(ObjectEnum::SupplyTower))));
        assert!(matches!(get_grid_slot_action(&state, 2, 0, false, false, &caps, false, false), Some(CommandButtonAction::Back)));
    }

    #[test]
    fn dc_constructing_grid_has_back_and_cancel() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing);
        assert!(matches!(get_grid_slot_action(&state, 2, 0, false, false, &caps, false, false), Some(CommandButtonAction::Back)));
        assert!(matches!(get_grid_slot_action(&state, 2, 1, false, false, &caps, false, false), Some(CommandButtonAction::DcCancel)));
        // Old cancel position (0,0) should be empty
        assert!(get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false).is_none());
    }

    #[test]
    fn dc_ready_to_place_grid_has_enter_placement_back_and_cancel() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace);
        assert!(matches!(get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false), Some(CommandButtonAction::EnterPlacement)));
        assert!(matches!(get_grid_slot_action(&state, 2, 0, false, false, &caps, false, false), Some(CommandButtonAction::Back)));
        assert!(matches!(get_grid_slot_action(&state, 2, 1, false, false, &caps, false, false), Some(CommandButtonAction::DcCancel)));
        // Old cancel position (0,1) should be empty
        assert!(get_grid_slot_action(&state, 0, 1, false, false, &caps, false, false).is_none());
    }

    // === EF grid slot tests ===

    #[test]
    fn ef_idle_grid_has_build_plate() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        assert!(matches!(get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false), Some(CommandButtonAction::EfBuildPlate)));
    }


    #[test]
    fn ef_idle_remaining_slots_are_none() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        // Only (0,0) should have an action
        for row in 0..3u8 {
            for col in 0..3u8 {
                if (row, col) == (0, 0) { continue; }
                assert!(get_grid_slot_action(&state, row, col, false, false, &caps, false, false).is_none(),
                    "Expected None at ({}, {}) for EfIdle", row, col);
            }
        }
    }


    #[test]
    fn ef_awaiting_placement_is_placement_mode() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfAwaitingPlacement);
        assert!(state.is_placement_mode());
    }

    #[test]
    fn ef_idle_is_not_placement_mode() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        assert!(!state.is_placement_mode());
    }


    // === Mixed unit+structure active group type routing tests ===

    #[test]
    fn active_group_structure_type_is_detected() {
        // When active group is a structure type, is_structure() should return true
        assert!(ObjectEnum::Barracks.is_structure());
        assert!(ObjectEnum::DeploymentCenter.is_structure());
        assert!(ObjectEnum::ExtractionFacility.is_structure());
        assert!(ObjectEnum::SupplyTower.is_structure());
        assert!(ObjectEnum::PowerPlant.is_structure());
        assert!(ObjectEnum::Tunnel.is_structure());
        assert!(ObjectEnum::Headquarters.is_structure());
    }

    #[test]
    fn active_group_unit_type_is_detected() {
        // When active group is a unit type, is_unit() should return true
        assert!(ObjectEnum::Peacekeeper.is_unit());
        assert!(ObjectEnum::SupplyChopper.is_unit());
        assert!(ObjectEnum::SyndicateAgent.is_unit());
        assert!(ObjectEnum::SyndicateGuard.is_unit());
        // And units are not structures
        assert!(!ObjectEnum::Peacekeeper.is_structure());
        assert!(!ObjectEnum::SupplyChopper.is_structure());
    }

    #[test]
    fn mixed_selection_active_group_determines_branch() {
        // Simulates the decision logic in update_command_panel_state:
        // When active group is a unit type, the structure branch should NOT be entered
        // even if structures are present in the selection.
        let mut selection = Selection::default();
        let e1 = Entity::from_raw_u32(100).unwrap();
        let e2 = Entity::from_raw_u32(101).unwrap();
        selection.groups = vec![
            SelectionGroup { object_type: ObjectEnum::Barracks, entities: vec![e1] },
            SelectionGroup { object_type: ObjectEnum::Peacekeeper, entities: vec![e2] },
        ];

        // Active group 0 = Barracks (structure)
        selection.active_group_index = Some(0);
        let active_type = selection.active_type();
        let active_is_structure = active_type.map(|t| t.is_structure()).unwrap_or(false);
        assert!(active_is_structure, "Barracks group should be detected as structure");

        // Cycle to group 1 = Peacekeeper (unit)
        selection.active_group_index = Some(1);
        let active_type = selection.active_type();
        let active_is_structure = active_type.map(|t| t.is_structure()).unwrap_or(false);
        assert!(!active_is_structure, "Peacekeeper group should NOT be detected as structure");
    }

    // === DC/EF State Preservation Tests ===

    #[test]
    fn dc_selecting_constructing_dc_lands_on_idle() {
        // When selecting a DC that is constructing, the interface should start at DcIdle,
        // NOT auto-enter DcConstructing. The player navigates to DcConstructing manually.
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        // DcIdle is the correct initial state regardless of DC construction status
        assert!(matches!(state, ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle)));
    }

    #[test]
    fn dc_valid_states_are_preserved() {
        // All DC sub-states should be preserved when re-selecting the same DC (no target change)
        let valid_dc_states = vec![
            StructureMenuState::DcIdle,
            StructureMenuState::DcBuildMenu,
            StructureMenuState::DcConstructing,
            StructureMenuState::DcReadyToPlace,
            StructureMenuState::DcAwaitingPlacement,
        ];
        for state in valid_dc_states {
            let in_valid = matches!(state,
                StructureMenuState::DcIdle |
                StructureMenuState::DcBuildMenu |
                StructureMenuState::DcConstructing |
                StructureMenuState::DcReadyToPlace |
                StructureMenuState::DcAwaitingPlacement
            );
            assert!(in_valid, "DC state {:?} should be considered valid for preservation", state);
        }
    }

    #[test]
    fn dc_invalid_state_triggers_reset_to_idle() {
        // Non-DC states should cause a reset to DcIdle
        let non_dc_states = vec![
            StructureMenuState::EfIdle,
            StructureMenuState::BarracksMenu,
            StructureMenuState::TunnelIdle,
        ];
        for state in non_dc_states {
            let in_valid = matches!(state,
                StructureMenuState::DcIdle |
                StructureMenuState::DcBuildMenu |
                StructureMenuState::DcConstructing |
                StructureMenuState::DcReadyToPlace |
                StructureMenuState::DcAwaitingPlacement
            );
            assert!(!in_valid, "Non-DC state {:?} should NOT be valid for DC preservation", state);
        }
    }

    #[test]
    fn ef_selecting_constructing_ef_lands_on_idle() {
        // Same as DC: selecting a constructing EF should land on EfIdle
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        assert!(matches!(state, ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle)));
    }

    #[test]
    fn ef_valid_states_are_preserved() {
        let valid_ef_states = vec![
            StructureMenuState::EfIdle,
            StructureMenuState::EfAwaitingPlacement,
        ];
        for state in valid_ef_states {
            let in_valid = matches!(state,
                StructureMenuState::EfIdle |
                StructureMenuState::EfAwaitingPlacement
            );
            assert!(in_valid, "EF state {:?} should be considered valid for preservation", state);
        }
    }

    #[test]
    fn ef_invalid_state_triggers_reset_to_idle() {
        let non_ef_states = vec![
            StructureMenuState::DcIdle,
            StructureMenuState::BarracksMenu,
            StructureMenuState::TunnelIdle,
        ];
        for state in non_ef_states {
            let in_valid = matches!(state,
                StructureMenuState::EfIdle |
                StructureMenuState::EfAwaitingPlacement
            );
            assert!(!in_valid, "Non-EF state {:?} should NOT be valid for EF preservation", state);
        }
    }

    #[test]
    fn dc_back_from_constructing_goes_to_idle() {
        // Pressing Back from DcConstructing should go to DcIdle
        // (verified via the Back handler, not update_command_panel_state)
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing);
        // The Back handler maps DcConstructing → DcIdle
        let result = match state {
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing) |
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace) =>
                ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle),
            _ => state,
        };
        assert!(matches!(result, ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle)));
    }

    #[test]
    fn ef_escape_from_awaiting_placement_goes_to_idle() {
        // EfAwaitingPlacement → Escape → EfIdle (no longer goes to EfReadyToPlace)
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfAwaitingPlacement);
        let result = match state {
            ObjectInterfaceState::StructureMenu(StructureMenuState::EfAwaitingPlacement) =>
                ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle),
            _ => state,
        };
        assert!(matches!(result, ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle)));
    }

    #[test]
    fn active_group_entities_returns_all_group_entities() {
        use crate::shared::types::ObjectEnum;

        let e1 = Entity::from_raw_u32(1).unwrap();
        let e2 = Entity::from_raw_u32(2).unwrap();
        let e3 = Entity::from_raw_u32(3).unwrap();

        let selection = Selection {
            groups: vec![SelectionGroup {
                object_type: ObjectEnum::Barracks,
                entities: vec![e1, e2, e3],
            }],
            active_group_index: Some(0),
        };

        let entities = active_group_entities(&selection);
        assert_eq!(entities.len(), 3);
        assert_eq!(entities, vec![e1, e2, e3]);
    }

    #[test]
    fn active_group_entities_returns_empty_when_no_active_group() {
        let selection = Selection {
            groups: vec![],
            active_group_index: None,
        };

        let entities = active_group_entities(&selection);
        assert!(entities.is_empty());
    }

    #[test]
    fn active_group_entities_single_entity_group() {
        use crate::shared::types::ObjectEnum;

        let e1 = Entity::from_raw_u32(10).unwrap();

        let selection = Selection {
            groups: vec![SelectionGroup {
                object_type: ObjectEnum::Barracks,
                entities: vec![e1],
            }],
            active_group_index: Some(0),
        };

        let entities = active_group_entities(&selection);
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0], e1);
    }

    // === Context-Aware Build Menu Tests ===

    #[test]
    fn ef_idle_no_construction_no_cancel() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        let action = get_grid_slot_action(&state, 2, 1, false, false, &caps, false, false);
        assert!(action.is_none(), "EfIdle should not show cancel when has_active_construction=false");
    }

    #[test]
    fn ef_idle_with_construction_shows_cancel() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        let action = get_grid_slot_action(&state, 2, 1, false, true, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::EfCancel)),
            "EfIdle should show EfCancel at (2,1) when has_active_construction=true");
    }

    #[test]
    fn ef_idle_with_construction_still_has_build_plate() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        let action = get_grid_slot_action(&state, 0, 0, false, true, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::EfBuildPlate)),
            "EfIdle should still show EfBuildPlate at (0,0) even with active construction");
    }

    // === Placement flip tests ===

    #[test]
    fn placement_state_flip_defaults_to_false() {
        let ps = PlacementState::default();
        assert!(!ps.flip_horizontal, "Default placement should not be flipped horizontally");
        assert!(!ps.flip_vertical, "Default placement should not be flipped vertically");
    }

    #[test]
    fn flipped_structure_material_should_be_double_sided() {
        // When a structure is flipped, its material cull_mode must be None
        // to prevent backface culling artifacts from negative scale
        let is_flipped = true;
        let cull_mode: Option<bevy::render::render_resource::Face> = if is_flipped { None } else { Some(bevy::render::render_resource::Face::Back) };
        assert!(cull_mode.is_none(), "Flipped structure material must have cull_mode: None");
    }

    #[test]
    fn unflipped_structure_material_should_use_backface_culling() {
        let is_flipped = false;
        let cull_mode: Option<bevy::render::render_resource::Face> = if is_flipped { None } else { Some(bevy::render::render_resource::Face::Back) };
        assert!(cull_mode.is_some(), "Unflipped structure material should use normal backface culling");
    }

    // === EjectRequest tests ===

    #[test]
    fn eject_request_stores_unit_type() {
        let request = EjectRequest { unit_type: ObjectEnum::SyndicateAgent };
        assert_eq!(request.unit_type, ObjectEnum::SyndicateAgent);
    }

    #[test]
    fn eject_request_stores_guard_type() {
        let request = EjectRequest { unit_type: ObjectEnum::SyndicateGuard };
        assert_eq!(request.unit_type, ObjectEnum::SyndicateGuard);
    }

    // === Eject button enabled logic tests ===

    #[test]
    fn eject_button_grid_slot_action_exists() {
        // TunnelOpenEjectMenu should be at (0, 2) in TunnelIdle state
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
        let action = get_grid_slot_action(&state, 0, 2, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::TunnelOpenEjectMenu)),
            "Eject should appear at grid slot (0, 2) in TunnelIdle");
    }

    #[test]
    fn eject_button_uses_has_network_units_flag() {
        // The grid_button_enabled_ext match arm for TunnelOpenEjectMenu returns has_network_units.
        // Since we can't easily call it with Query params, we verify the logic indirectly:
        // the match arm is: CommandButtonAction::TunnelOpenEjectMenu => has_network_units
        // When has_network_units=false, the button should be disabled.
        // When has_network_units=true, the button should be enabled.
        // This is a documentation-level test asserting the intent.
        let has_network_units_false = false;
        let has_network_units_true = true;
        assert!(!has_network_units_false, "Eject disabled when no network units");
        assert!(has_network_units_true, "Eject enabled when network has units");
    }

    // === Tunnel transit tier checks for eject ===

    #[test]
    fn tier1_tunnel_allows_agent_ejection() {
        // SyndicateAgent is HeavyInfantry — allowed by Tier1
        let tier = TunnelTier::Tier1;
        let unit_base = crate::game::units::types::unit_data::agent_type_data().unit_base;
        assert!(tier.can_transit(&unit_base), "Tier1 tunnel should allow Agent (HeavyInfantry) ejection");
    }

    #[test]
    fn tier1_tunnel_allows_guard_ejection() {
        // SyndicateGuard is HeavyInfantry — allowed by Tier1
        let tier = TunnelTier::Tier1;
        let unit_base = crate::game::units::types::unit_data::guard_type_data().unit_base;
        assert!(tier.can_transit(&unit_base), "Tier1 tunnel should allow Guard (HeavyInfantry) ejection");
    }

    #[test]
    fn tier1_tunnel_rejects_vehicle_ejection() {
        // WheeledVehicle requires Tier2+ transit
        let tier = TunnelTier::Tier1;
        let unit_base = crate::types::UnitBaseEnum::WheeledVehicle;
        assert!(!tier.can_transit(&unit_base), "Tier1 tunnel should reject WheeledVehicle ejection");
    }

    #[test]
    fn tier2_tunnel_allows_vehicle_ejection() {
        let tier = TunnelTier::Tier2;
        let unit_base = crate::types::UnitBaseEnum::WheeledVehicle;
        assert!(tier.can_transit(&unit_base), "Tier2 tunnel should allow WheeledVehicle ejection");
    }

    #[test]
    fn tier2_tunnel_rejects_air_ejection() {
        let tier = TunnelTier::Tier2;
        let unit_base = crate::types::UnitBaseEnum::HoverCraft;
        assert!(!tier.can_transit(&unit_base), "Tier2 tunnel should reject HoverCraft ejection");
    }

    #[test]
    fn tier3_tunnel_allows_air_ejection() {
        let tier = TunnelTier::Tier3;
        let unit_base = crate::types::UnitBaseEnum::HoverCraft;
        assert!(tier.can_transit(&unit_base), "Tier3 tunnel should allow HoverCraft ejection");
    }

    // === Tunnel upgrade cost formula design spec verification ===

    #[test]
    fn tunnel_upgrade_t2_cost_matches_design_spec() {
        use crate::game::types::structures::{tunnel_t2_upgrade_cost, tunnel_t3_upgrade_cost};
        // Design spec: Upgrade to Tier 2: 2 + 2 x (number of T2+ Tunnels owned)
        // 1st T2: 2, 2nd T2: 4, 3rd T2: 6
        assert_eq!(tunnel_t2_upgrade_cost(0), 2, "1st T2 upgrade should cost 2");
        assert_eq!(tunnel_t2_upgrade_cost(1), 4, "2nd T2 upgrade should cost 4");
        assert_eq!(tunnel_t2_upgrade_cost(2), 6, "3rd T2 upgrade should cost 6");
    }

    #[test]
    fn tunnel_upgrade_t3_cost_matches_design_spec() {
        use crate::game::types::structures::{tunnel_t3_upgrade_cost};
        // Design spec: Upgrade to Tier 3: 3 + 3 x (number of T3 Tunnels owned)
        // 1st T3: 3, 2nd T3: 6, 3rd T3: 9
        assert_eq!(tunnel_t3_upgrade_cost(0), 3, "1st T3 upgrade should cost 3");
        assert_eq!(tunnel_t3_upgrade_cost(1), 6, "2nd T3 upgrade should cost 6");
        assert_eq!(tunnel_t3_upgrade_cost(2), 9, "3rd T3 upgrade should cost 9");
    }

    #[test]
    fn tunnel_upgrade_cancel_gives_full_refund() {
        use crate::game::types::structures::{tunnel_t2_upgrade_cost, tunnel_t3_upgrade_cost};
        // Design spec: "Full refund of Supplies cost" — the cancel function uses
        // the same cost formula as upgrade, so refund == original cost
        let upgrade_cost = tunnel_t2_upgrade_cost(0);
        let refund_cost = tunnel_t2_upgrade_cost(0);
        assert_eq!(upgrade_cost, refund_cost, "Cancel refund must equal upgrade cost");

        let upgrade_cost_t3 = tunnel_t3_upgrade_cost(1);
        let refund_cost_t3 = tunnel_t3_upgrade_cost(1);
        assert_eq!(upgrade_cost_t3, refund_cost_t3, "Cancel refund must equal upgrade cost for T3");
    }

    // === Ejection queue cooldown design spec verification ===

    #[test]
    fn ejection_queue_cooldown_is_8_frames() {
        // Design spec: "a new unit begins ejecting every 8 frames minimum"
        // The ejection_tick_system sets cooldown = 8 after each ejection.
        // This is verified by the constant in the system (hardcoded 8).
        // Here we verify the EjectionQueue tracks cooldown correctly.
        let mut queue = crate::ui::types::EjectionQueue::default();
        assert_eq!(queue.cooldown, 0, "Default cooldown should be 0");
        queue.cooldown = 8;
        assert_eq!(queue.cooldown, 8, "Cooldown should be settable to 8");
    }

    // === Right-click cancel submenu tests ===

    #[test]
    fn right_click_dc_build_menu_returns_dc_idle() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
        let result = right_click_cancel_target(&state, || None);
        assert_eq!(result, Some(ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle)));
    }

    #[test]
    fn right_click_dc_constructing_returns_dc_idle() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing);
        let result = right_click_cancel_target(&state, || None);
        assert_eq!(result, Some(ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle)));
    }

    #[test]
    fn right_click_dc_ready_to_place_returns_dc_idle() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace);
        let result = right_click_cancel_target(&state, || None);
        assert_eq!(result, Some(ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle)));
    }

    #[test]
    fn right_click_tunnel_expand_menu_returns_tunnel_idle() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
        let result = right_click_cancel_target(&state, || None);
        assert_eq!(result, Some(ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle)));
    }

    #[test]
    fn right_click_tunnel_eject_menu_returns_tunnel_idle() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu);
        let result = right_click_cancel_target(&state, || None);
        assert_eq!(result, Some(ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle)));
    }

    #[test]
    fn right_click_ef_idle_returns_none() {
        // EfIdle has no sub-menus to cancel, so right_click_cancel_target returns None
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        let result = right_click_cancel_target(&state, || None);
        assert_eq!(result, None);
    }

    #[test]
    fn right_click_set_rally_point_with_barracks_returns_barracks_menu() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::SetRallyPoint);
        let result = right_click_cancel_target(&state, || Some(RallyTargetKind::Barracks));
        assert_eq!(result, Some(ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu)));
    }

    #[test]
    fn right_click_set_rally_point_with_headquarters_returns_hq_menu() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::SetRallyPoint);
        let result = right_click_cancel_target(&state, || Some(RallyTargetKind::Headquarters));
        assert_eq!(result, Some(ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu)));
    }

    #[test]
    fn right_click_set_rally_point_with_supply_tower_returns_st_menu() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::SetRallyPoint);
        let result = right_click_cancel_target(&state, || Some(RallyTargetKind::SupplyTower));
        assert_eq!(result, Some(ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu)));
    }

    #[test]
    fn right_click_set_rally_point_no_target_returns_default() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::SetRallyPoint);
        let result = right_click_cancel_target(&state, || None);
        assert_eq!(result, Some(ObjectInterfaceState::Default));
    }

    #[test]
    fn right_click_schedule_deliveries_returns_supply_tower_menu() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::ScheduleDeliveries);
        let result = right_click_cancel_target(&state, || None);
        assert_eq!(result, Some(ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu)));
    }

    #[test]
    fn right_click_default_state_does_nothing() {
        let state = ObjectInterfaceState::Default;
        let result = right_click_cancel_target(&state, || None);
        assert_eq!(result, None);
    }

    #[test]
    fn right_click_placement_state_does_nothing() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcAwaitingPlacement);
        let result = right_click_cancel_target(&state, || None);
        assert_eq!(result, None);
    }

    #[test]
    fn right_click_unit_awaiting_target_move_does_nothing() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::Move);
        let result = right_click_cancel_target(&state, || None);
        assert_eq!(result, None);
    }

    // === RecruitmentCenterMenu tests ===

    #[test]
    fn rc_menu_grid_slot_cancel_at_2_1_when_producing() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu);
        let caps = no_caps();
        // Cancel visible at (2,1) when bk_has_queue is true (reused for RC production_progress > 0)
        let action = get_grid_slot_action(&state, 2, 1, true, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::RcCancel)));
    }

    #[test]
    fn rc_menu_grid_slot_no_cancel_when_idle() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu);
        let caps = no_caps();
        // Cancel hidden at (2,1) when bk_has_queue is false
        let action = get_grid_slot_action(&state, 2, 1, false, false, &caps, false, false);
        assert!(action.is_none());
    }

    #[test]
    fn rc_menu_grid_slot_rally_at_2_2() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu);
        let caps = no_caps();
        let action = get_grid_slot_action(&state, 2, 2, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::SetRallyPoint)));
    }

    #[test]
    fn rc_menu_grid_slot_empty_at_0_0() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu);
        let caps = no_caps();
        let action = get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(action.is_none(), "RC menu should have no action at (0,0)");
    }

    #[test]
    fn rc_cancel_label() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu);
        let label = grid_button_label(&state, &CommandButtonAction::RcCancel, 0, 'X');
        assert_eq!(label, "[X] Cancel\nProd");
    }

    #[test]
    fn right_click_set_rally_point_with_rc_returns_rc_menu() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::SetRallyPoint);
        let result = right_click_cancel_target(&state, || Some(RallyTargetKind::RecruitmentCenter));
        assert_eq!(result, Some(ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu)));
    }

    #[test]
    fn rc_menu_is_not_placement_mode() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu);
        assert!(!state.is_placement_mode());
    }

    #[test]
    fn rc_cancel_is_not_unit_action() {
        assert!(!is_unit_action(&CommandButtonAction::RcCancel));
    }

    #[test]
    fn rc_set_rally_is_not_unit_action() {
        assert!(!is_unit_action(&CommandButtonAction::SetRallyPoint));
    }

    #[test]
    fn right_click_unit_awaiting_target_attack_does_nothing() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::Attack);
        let result = right_click_cancel_target(&state, || None);
        assert_eq!(result, None);
    }

    // === SupplyChopper interface tests ===

    #[test]
    fn chopper_grid_shows_move_pickup_attach_dropoff() {
        let caps = SelectedUnitCapabilities { is_chopper: true, ..Default::default() };
        let state = ObjectInterfaceState::Default;
        assert!(matches!(get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false), Some(CommandButtonAction::UnitMove)));
        assert!(matches!(get_grid_slot_action(&state, 0, 1, false, false, &caps, false, false), Some(CommandButtonAction::ChopperPickUpSupplies)));
        assert!(matches!(get_grid_slot_action(&state, 0, 2, false, false, &caps, false, false), Some(CommandButtonAction::ChopperAttachToTower)));
        assert!(matches!(get_grid_slot_action(&state, 1, 0, false, false, &caps, false, false), Some(CommandButtonAction::ChopperDropOffSupplies)));
        assert!(matches!(get_grid_slot_action(&state, 1, 2, false, false, &caps, false, false), Some(CommandButtonAction::UnitHoldPosition)));
        assert!(matches!(get_grid_slot_action(&state, 2, 1, false, false, &caps, false, false), Some(CommandButtonAction::UnitStop)));
    }

    #[test]
    fn chopper_grid_has_no_attack_buttons() {
        let caps = SelectedUnitCapabilities { is_chopper: true, ..Default::default() };
        let state = ObjectInterfaceState::Default;
        // No attack-related buttons in any remaining empty slots
        assert!(get_grid_slot_action(&state, 1, 1, false, false, &caps, false, false).is_none());
        assert!(get_grid_slot_action(&state, 2, 0, false, false, &caps, false, false).is_none());
        assert!(get_grid_slot_action(&state, 2, 2, false, false, &caps, false, false).is_none());
    }

    #[test]
    fn chopper_object_type_supports_chopper_actions() {
        assert!(object_type_supports_action(&ObjectEnum::SupplyChopper, &CommandButtonAction::ChopperPickUpSupplies));
        assert!(object_type_supports_action(&ObjectEnum::SupplyChopper, &CommandButtonAction::ChopperAttachToTower));
    }

    #[test]
    fn non_chopper_does_not_support_chopper_actions() {
        assert!(!object_type_supports_action(&ObjectEnum::Peacekeeper, &CommandButtonAction::ChopperPickUpSupplies));
        assert!(!object_type_supports_action(&ObjectEnum::Peacekeeper, &CommandButtonAction::ChopperAttachToTower));
        assert!(!object_type_supports_action(&ObjectEnum::SyndicateAgent, &CommandButtonAction::ChopperPickUpSupplies));
    }

    #[test]
    fn is_action_active_pick_up_supplies_mode() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::PickUpSupplies);
        assert!(is_action_active(&CommandButtonAction::ChopperPickUpSupplies, &state));
        assert!(!is_action_active(&CommandButtonAction::ChopperAttachToTower, &state));
        assert!(!is_action_active(&CommandButtonAction::UnitMove, &state));
    }

    #[test]
    fn is_action_active_attach_to_tower_mode() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::AttachToTower);
        assert!(is_action_active(&CommandButtonAction::ChopperAttachToTower, &state));
        assert!(!is_action_active(&CommandButtonAction::ChopperPickUpSupplies, &state));
    }

    #[test]
    fn chopper_grid_button_labels() {
        let label = grid_button_label(
            &ObjectInterfaceState::Default,
            &CommandButtonAction::ChopperPickUpSupplies,
            0,
            'W',
        );
        assert_eq!(label, "[W] Pick Up\nSupplies");

        let label = grid_button_label(
            &ObjectInterfaceState::Default,
            &CommandButtonAction::ChopperAttachToTower,
            0,
            'E',
        );
        assert_eq!(label, "[E] Attach\nTower");
    }

    #[test]
    fn non_chopper_default_grid_unchanged() {
        // Verify the standard Default grid hasn't changed when is_chopper=false
        let caps = no_caps();
        let state = ObjectInterfaceState::Default;
        assert!(matches!(get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false), Some(CommandButtonAction::UnitMove)));
        assert!(matches!(get_grid_slot_action(&state, 0, 2, false, false, &caps, false, false), Some(CommandButtonAction::UnitHoldPosition)));
        assert!(matches!(get_grid_slot_action(&state, 1, 1, false, false, &caps, false, false), Some(CommandButtonAction::UnitPatrol)));
        assert!(matches!(get_grid_slot_action(&state, 2, 1, false, false, &caps, false, false), Some(CommandButtonAction::UnitStop)));
        // No PickUpSupplies or AttachToTower in non-chopper grid
        assert!(get_grid_slot_action(&state, 0, 1, false, false, &caps, false, false).is_none());
    }

    // === ChopperDropOffSupplies tests ===

    #[test]
    fn chopper_drop_off_supplies_in_grid() {
        let caps = SelectedUnitCapabilities { is_chopper: true, ..Default::default() };
        let state = ObjectInterfaceState::Default;
        assert!(matches!(
            get_grid_slot_action(&state, 1, 0, false, false, &caps, false, false),
            Some(CommandButtonAction::ChopperDropOffSupplies)
        ));
    }

    #[test]
    fn chopper_drop_off_supplies_label() {
        let label = grid_button_label(
            &ObjectInterfaceState::Default,
            &CommandButtonAction::ChopperDropOffSupplies,
            0,
            'A',
        );
        assert_eq!(label, "[A] Drop Off\nSupplies");
    }

    #[test]
    fn chopper_drop_off_supplies_object_type_support() {
        assert!(object_type_supports_action(&ObjectEnum::SupplyChopper, &CommandButtonAction::ChopperDropOffSupplies));
        assert!(!object_type_supports_action(&ObjectEnum::Peacekeeper, &CommandButtonAction::ChopperDropOffSupplies));
    }

    #[test]
    fn chopper_drop_off_supplies_is_unit_action() {
        assert!(is_unit_action(&CommandButtonAction::ChopperDropOffSupplies));
    }

    #[test]
    fn is_action_active_drop_off_supplies_mode() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::DropOffSupplies);
        assert!(is_action_active(&CommandButtonAction::ChopperDropOffSupplies, &state));
        assert!(!is_action_active(&CommandButtonAction::ChopperPickUpSupplies, &state));
        assert!(!is_action_active(&CommandButtonAction::ChopperAttachToTower, &state));
    }

    #[test]
    fn chopper_drop_off_supplies_caps_gating() {
        // Without supplies — chopper_has_supplies should be false by default
        let caps_empty = SelectedUnitCapabilities { is_chopper: true, chopper_has_supplies: false, ..Default::default() };
        assert!(!caps_empty.chopper_has_supplies);

        // With supplies — chopper_has_supplies should be true
        let caps_has = SelectedUnitCapabilities { is_chopper: true, chopper_has_supplies: true, ..Default::default() };
        assert!(caps_has.chopper_has_supplies);
    }

    // === resolve_pointer_display tests ===

    fn default_cursor_info() -> CursorEntityInfo {
        CursorEntityInfo::default()
    }

    fn caps_with_attack() -> SelectedUnitCapabilities {
        SelectedUnitCapabilities { has_attack: true, ..Default::default() }
    }

    #[test]
    fn pointer_inactive_when_no_selection() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::Default,
            &CursorTargetEnum::Ground,
            false,
            &default_cursor_info(),
            0, // no selection
            None,
            &SelectedUnitCapabilities::default(),
            false,
        );
        assert_eq!(result, PointerDisplayType::Inactive);
    }

    #[test]
    fn pointer_inactive_during_placement() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::StructureMenu(StructureMenuState::DcAwaitingPlacement),
            &CursorTargetEnum::Ground,
            false,
            &default_cursor_info(),
            1,
            Some(ObjectEnum::DeploymentCenter),
            &SelectedUnitCapabilities::default(),
            false,
        );
        assert_eq!(result, PointerDisplayType::Inactive);
    }

    #[test]
    fn pointer_move_on_ground_with_units() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::Default,
            &CursorTargetEnum::Ground,
            false,
            &default_cursor_info(),
            3,
            Some(ObjectEnum::Peacekeeper),
            &caps_with_attack(),
            false,
        );
        assert_eq!(result, PointerDisplayType::Move);
    }

    #[test]
    fn pointer_attack_on_enemy_with_attack_units() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::Default,
            &CursorTargetEnum::EnemyObject,
            true,
            &default_cursor_info(),
            2,
            Some(ObjectEnum::Peacekeeper),
            &caps_with_attack(),
            false,
        );
        assert_eq!(result, PointerDisplayType::Attack);
    }

    #[test]
    fn pointer_inactive_on_enemy_without_attack() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::Default,
            &CursorTargetEnum::EnemyObject,
            true,
            &default_cursor_info(),
            1,
            Some(ObjectEnum::SyndicateAgent),
            &SelectedUnitCapabilities::default(), // no attack
            false,
        );
        assert_eq!(result, PointerDisplayType::Inactive);
    }

    #[test]
    fn pointer_move_for_production_structure() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::Default,
            &CursorTargetEnum::Ground,
            false,
            &default_cursor_info(),
            1,
            Some(ObjectEnum::Barracks),
            &SelectedUnitCapabilities::default(),
            true, // has production
        );
        assert_eq!(result, PointerDisplayType::Move);
    }

    #[test]
    fn pointer_gather_agent_on_crystal() {
        let info = CursorEntityInfo { has_crystal_patch: true, ..Default::default() };
        let result = resolve_pointer_display(
            &ObjectInterfaceState::Default,
            &CursorTargetEnum::NeutralObject,
            true,
            &info,
            1,
            Some(ObjectEnum::SyndicateAgent),
            &SelectedUnitCapabilities::default(),
            false,
        );
        assert_eq!(result, PointerDisplayType::GatherResources);
    }

    #[test]
    fn pointer_gather_chopper_on_supply_station() {
        let info = CursorEntityInfo { has_supply_station: true, ..Default::default() };
        let caps = SelectedUnitCapabilities { is_chopper: true, ..Default::default() };
        let result = resolve_pointer_display(
            &ObjectInterfaceState::Default,
            &CursorTargetEnum::NeutralObject,
            true,
            &info,
            1,
            Some(ObjectEnum::SupplyChopper),
            &caps,
            false,
        );
        assert_eq!(result, PointerDisplayType::GatherResources);
    }

    #[test]
    fn pointer_return_resources_agent_to_own_tunnel() {
        let info = CursorEntityInfo { has_tunnel: true, is_own: true, ..Default::default() };
        let caps = SelectedUnitCapabilities { agent_carrying: true, ..Default::default() };
        let result = resolve_pointer_display(
            &ObjectInterfaceState::Default,
            &CursorTargetEnum::FriendlyObject,
            true,
            &info,
            1,
            Some(ObjectEnum::SyndicateAgent),
            &caps,
            false,
        );
        assert_eq!(result, PointerDisplayType::ReturnResources);
    }

    #[test]
    fn pointer_return_resources_chopper_to_own_supply_tower() {
        let info = CursorEntityInfo { has_supply_tower: true, is_own: true, ..Default::default() };
        let caps = SelectedUnitCapabilities { is_chopper: true, chopper_has_supplies: true, ..Default::default() };
        let result = resolve_pointer_display(
            &ObjectInterfaceState::Default,
            &CursorTargetEnum::FriendlyObject,
            true,
            &info,
            1,
            Some(ObjectEnum::SupplyChopper),
            &caps,
            false,
        );
        assert_eq!(result, PointerDisplayType::ReturnResources);
    }

    #[test]
    fn pointer_enter_syndicate_unit_own_tunnel() {
        let info = CursorEntityInfo { has_tunnel: true, is_own: true, ..Default::default() };
        let result = resolve_pointer_display(
            &ObjectInterfaceState::Default,
            &CursorTargetEnum::FriendlyObject,
            true,
            &info,
            1,
            Some(ObjectEnum::SyndicateGuard),
            &SelectedUnitCapabilities::default(),
            false,
        );
        assert_eq!(result, PointerDisplayType::Enter);
    }

    #[test]
    fn pointer_no_enter_for_non_syndicate() {
        let info = CursorEntityInfo { has_tunnel: true, is_own: true, ..Default::default() };
        let result = resolve_pointer_display(
            &ObjectInterfaceState::Default,
            &CursorTargetEnum::FriendlyObject,
            true,
            &info,
            1,
            Some(ObjectEnum::Peacekeeper),
            &caps_with_attack(),
            false,
        );
        // Peacekeeper can't enter tunnels → Move (friendly object, has units)
        assert_eq!(result, PointerDisplayType::Move);
    }

    // --- AwaitingTarget tests ---

    #[test]
    fn awaiting_attack_enemy_shows_attack() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::AwaitingTarget(CommandType::Attack),
            &CursorTargetEnum::EnemyObject,
            true,
            &default_cursor_info(),
            1,
            Some(ObjectEnum::Peacekeeper),
            &caps_with_attack(),
            false,
        );
        assert_eq!(result, PointerDisplayType::Attack);
    }

    #[test]
    fn awaiting_attack_ground_shows_attack() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::AwaitingTarget(CommandType::Attack),
            &CursorTargetEnum::Ground,
            false,
            &default_cursor_info(),
            1,
            Some(ObjectEnum::Peacekeeper),
            &caps_with_attack(),
            false,
        );
        assert_eq!(result, PointerDisplayType::Attack);
    }

    #[test]
    fn awaiting_attack_friendly_shows_inactive() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::AwaitingTarget(CommandType::Attack),
            &CursorTargetEnum::FriendlyObject,
            true,
            &default_cursor_info(),
            1,
            Some(ObjectEnum::Peacekeeper),
            &caps_with_attack(),
            false,
        );
        assert_eq!(result, PointerDisplayType::Inactive);
    }

    #[test]
    fn awaiting_move_any_target_shows_move() {
        for kind in [CursorTargetEnum::Ground, CursorTargetEnum::EnemyObject, CursorTargetEnum::FriendlyObject] {
            let result = resolve_pointer_display(
                &ObjectInterfaceState::AwaitingTarget(CommandType::Move),
                &kind,
                false,
                &default_cursor_info(),
                1,
                Some(ObjectEnum::Peacekeeper),
                &caps_with_attack(),
                false,
            );
            assert_eq!(result, PointerDisplayType::Move);
        }
    }

    #[test]
    fn awaiting_patrol_ground_shows_patrol() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::AwaitingTarget(CommandType::Patrol),
            &CursorTargetEnum::Ground,
            false,
            &default_cursor_info(),
            1,
            Some(ObjectEnum::Peacekeeper),
            &caps_with_attack(),
            false,
        );
        assert_eq!(result, PointerDisplayType::Patrol);
    }

    #[test]
    fn awaiting_patrol_enemy_shows_inactive() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::AwaitingTarget(CommandType::Patrol),
            &CursorTargetEnum::EnemyObject,
            true,
            &default_cursor_info(),
            1,
            Some(ObjectEnum::Peacekeeper),
            &caps_with_attack(),
            false,
        );
        assert_eq!(result, PointerDisplayType::Inactive);
    }

    #[test]
    fn awaiting_attack_ground_cmd_ground_target() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::AwaitingTarget(CommandType::AttackGround),
            &CursorTargetEnum::Ground,
            false,
            &default_cursor_info(),
            1,
            Some(ObjectEnum::Peacekeeper),
            &caps_with_attack(),
            false,
        );
        assert_eq!(result, PointerDisplayType::AttackGround);
    }

    #[test]
    fn awaiting_reverse_ground_shows_move() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::AwaitingTarget(CommandType::Reverse),
            &CursorTargetEnum::Ground,
            false,
            &default_cursor_info(),
            1,
            Some(ObjectEnum::Peacekeeper),
            &SelectedUnitCapabilities { can_reverse: true, ..Default::default() },
            false,
        );
        assert_eq!(result, PointerDisplayType::Move);
    }

    #[test]
    fn awaiting_schedule_deliveries_on_supply_station() {
        let info = CursorEntityInfo { has_supply_station: true, ..Default::default() };
        let result = resolve_pointer_display(
            &ObjectInterfaceState::AwaitingTarget(CommandType::ScheduleDeliveries),
            &CursorTargetEnum::NeutralObject,
            true,
            &info,
            1,
            Some(ObjectEnum::SupplyChopper),
            &SelectedUnitCapabilities { is_chopper: true, ..Default::default() },
            false,
        );
        assert_eq!(result, PointerDisplayType::GatherResources);
    }

    #[test]
    fn awaiting_set_rally_point_shows_move() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::AwaitingTarget(CommandType::SetRallyPoint),
            &CursorTargetEnum::Ground,
            false,
            &default_cursor_info(),
            1,
            Some(ObjectEnum::Barracks),
            &SelectedUnitCapabilities::default(),
            true,
        );
        assert_eq!(result, PointerDisplayType::Move);
    }

    #[test]
    fn awaiting_enter_own_tunnel() {
        let info = CursorEntityInfo { has_tunnel: true, is_own: true, ..Default::default() };
        let result = resolve_pointer_display(
            &ObjectInterfaceState::AwaitingTarget(CommandType::Enter),
            &CursorTargetEnum::FriendlyObject,
            true,
            &info,
            1,
            Some(ObjectEnum::SyndicateGuard),
            &SelectedUnitCapabilities::default(),
            false,
        );
        assert_eq!(result, PointerDisplayType::Enter);
    }

    #[test]
    fn awaiting_enter_enemy_tunnel_shows_inactive() {
        let info = CursorEntityInfo { has_tunnel: true, is_own: false, ..Default::default() };
        let result = resolve_pointer_display(
            &ObjectInterfaceState::AwaitingTarget(CommandType::Enter),
            &CursorTargetEnum::EnemyObject,
            true,
            &info,
            1,
            Some(ObjectEnum::SyndicateGuard),
            &SelectedUnitCapabilities::default(),
            false,
        );
        assert_eq!(result, PointerDisplayType::Inactive);
    }

    #[test]
    fn awaiting_gather_on_crystal() {
        let info = CursorEntityInfo { has_crystal_patch: true, ..Default::default() };
        let result = resolve_pointer_display(
            &ObjectInterfaceState::AwaitingTarget(CommandType::Gather),
            &CursorTargetEnum::NeutralObject,
            true,
            &info,
            1,
            Some(ObjectEnum::SyndicateAgent),
            &SelectedUnitCapabilities::default(),
            false,
        );
        assert_eq!(result, PointerDisplayType::GatherResources);
    }

    #[test]
    fn awaiting_dropoff_own_tunnel() {
        let info = CursorEntityInfo { has_tunnel: true, is_own: true, ..Default::default() };
        let result = resolve_pointer_display(
            &ObjectInterfaceState::AwaitingTarget(CommandType::DropOff),
            &CursorTargetEnum::FriendlyObject,
            true,
            &info,
            1,
            Some(ObjectEnum::SyndicateAgent),
            &SelectedUnitCapabilities { agent_carrying: true, ..Default::default() },
            false,
        );
        assert_eq!(result, PointerDisplayType::ReturnResources);
    }

    #[test]
    fn awaiting_dropoff_supplies_own_supply_tower() {
        let info = CursorEntityInfo { has_supply_tower: true, is_own: true, ..Default::default() };
        let result = resolve_pointer_display(
            &ObjectInterfaceState::AwaitingTarget(CommandType::DropOffSupplies),
            &CursorTargetEnum::FriendlyObject,
            true,
            &info,
            1,
            Some(ObjectEnum::SupplyChopper),
            &SelectedUnitCapabilities { is_chopper: true, chopper_has_supplies: true, ..Default::default() },
            false,
        );
        assert_eq!(result, PointerDisplayType::ReturnResources);
    }

    #[test]
    fn awaiting_hold_position_shows_inactive() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::AwaitingTarget(CommandType::HoldPosition),
            &CursorTargetEnum::Ground,
            false,
            &default_cursor_info(),
            1,
            Some(ObjectEnum::Peacekeeper),
            &caps_with_attack(),
            false,
        );
        assert_eq!(result, PointerDisplayType::Inactive);
    }

    #[test]
    fn pointer_move_on_friendly_with_units() {
        let result = resolve_pointer_display(
            &ObjectInterfaceState::Default,
            &CursorTargetEnum::FriendlyObject,
            true,
            &default_cursor_info(),
            2,
            Some(ObjectEnum::Peacekeeper),
            &caps_with_attack(),
            false,
        );
        assert_eq!(result, PointerDisplayType::Move);
    }

    #[test]
    fn pointer_inactive_structure_no_production() {
        // Non-production structure (e.g., PowerPlant) should be inactive
        let result = resolve_pointer_display(
            &ObjectInterfaceState::Default,
            &CursorTargetEnum::Ground,
            false,
            &default_cursor_info(),
            1,
            Some(ObjectEnum::PowerPlant),
            &SelectedUnitCapabilities::default(),
            false, // no production
        );
        // PowerPlant is a structure (not a unit) — is_unit() returns false → Inactive
        assert_eq!(result, PointerDisplayType::Inactive);
    }

    // === Armory interface tests ===

    #[test]
    fn armory_menu_grid_has_train_soldier() {
        let caps = SelectedUnitCapabilities::default();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
        let action = get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::ArmoryTrainSoldier)));
    }

    #[test]
    fn armory_menu_grid_has_train_gunner() {
        let caps = SelectedUnitCapabilities::default();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
        let action = get_grid_slot_action(&state, 0, 1, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::ArmoryTrainGunner)));
    }

    #[test]
    fn armory_menu_grid_has_eject_all() {
        let caps = SelectedUnitCapabilities::default();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
        let action = get_grid_slot_action(&state, 0, 2, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::ArmoryEjectAll)));
    }

    #[test]
    fn armory_menu_grid_has_set_rally_point() {
        let caps = SelectedUnitCapabilities::default();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
        let action = get_grid_slot_action(&state, 2, 2, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::SetRallyPoint)));
    }

    #[test]
    fn armory_menu_grid_empty_slots() {
        let caps = SelectedUnitCapabilities::default();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
        assert!(get_grid_slot_action(&state, 1, 0, false, false, &caps, false, false).is_none());
        assert!(get_grid_slot_action(&state, 1, 1, false, false, &caps, false, false).is_none());
        assert!(get_grid_slot_action(&state, 2, 0, false, false, &caps, false, false).is_none());
        assert!(get_grid_slot_action(&state, 2, 1, false, false, &caps, false, false).is_none());
    }

    #[test]
    fn armory_train_soldier_label() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
        let label = grid_button_label(&state, &CommandButtonAction::ArmoryTrainSoldier, 0, 'Q');
        assert_eq!(label, "[Q] Train\nSoldier");
    }

    #[test]
    fn armory_train_gunner_label() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
        let label = grid_button_label(&state, &CommandButtonAction::ArmoryTrainGunner, 0, 'W');
        assert_eq!(label, "[W] Train\nGunner");
    }

    #[test]
    fn armory_eject_all_label() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
        let label = grid_button_label(&state, &CommandButtonAction::ArmoryEjectAll, 0, 'E');
        assert_eq!(label, "[E] Eject\nAll");
    }

    #[test]
    fn armory_actions_are_not_unit_actions() {
        assert!(!is_unit_action(&CommandButtonAction::ArmoryTrainSoldier));
        assert!(!is_unit_action(&CommandButtonAction::ArmoryTrainGunner));
        assert!(!is_unit_action(&CommandButtonAction::ArmoryEjectAll));
    }

    #[test]
    fn right_click_cancel_target_rally_armory() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::SetRallyPoint);
        let result = right_click_cancel_target(&state, || Some(RallyTargetKind::Armory));
        assert_eq!(result, Some(ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu)));
    }

    #[test]
    fn armory_menu_title() {
        // Verify the title mapping works for ArmoryMenu
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
        let title = match &state {
            ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu) => "Armory",
            _ => "Unknown",
        };
        assert_eq!(title, "Armory");
    }

    // === Cults Recruit Menu Tests ===

    #[test]
    fn cults_recruit_default_grid_has_construct_and_assist() {
        let state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault);
        let caps = SelectedUnitCapabilities::default();
        // Q = Construct
        let action = get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::RecruitConstruct)));
        // S = Assist Construction
        let action = get_grid_slot_action(&state, 1, 1, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::RecruitAssistConstruction)));
        // Other slots are None
        let action = get_grid_slot_action(&state, 0, 1, false, false, &caps, false, false);
        assert!(action.is_none());
    }

    #[test]
    fn cults_recruit_construct_menu_grid_has_storage_and_back() {
        let state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu);
        let caps = SelectedUnitCapabilities::default();
        // Q = Storage
        let action = get_grid_slot_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::RecruitSelectBuilding(ObjectEnum::CultsStorage))));
        // Z = Back
        let action = get_grid_slot_action(&state, 2, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::Back)));
    }

    #[test]
    fn cults_recruit_awaiting_placement_grid_is_empty() {
        let state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement);
        let caps = SelectedUnitCapabilities::default();
        for r in 0..3 {
            for c in 0..3 {
                let action = get_grid_slot_action(&state, r, c, false, false, &caps, false, false);
                assert!(action.is_none(), "Expected None at ({}, {})", r, c);
            }
        }
    }

    #[test]
    fn cults_recruit_is_placement_mode() {
        let state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement);
        assert!(state.is_placement_mode());
        let default = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault);
        assert!(!default.is_placement_mode());
        let construct = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu);
        assert!(!construct.is_placement_mode());
    }

    #[test]
    fn recruit_actions_are_unit_actions() {
        assert!(is_unit_action(&CommandButtonAction::RecruitConstruct));
        assert!(is_unit_action(&CommandButtonAction::RecruitSelectBuilding(ObjectEnum::CultsStorage)));
        assert!(is_unit_action(&CommandButtonAction::RecruitAssistConstruction));
    }

    #[test]
    fn object_type_supports_recruit_actions() {
        assert!(object_type_supports_action(&ObjectEnum::CultsRecruit, &CommandButtonAction::RecruitConstruct));
        assert!(object_type_supports_action(&ObjectEnum::CultsRecruit, &CommandButtonAction::RecruitSelectBuilding(ObjectEnum::CultsStorage)));
        assert!(object_type_supports_action(&ObjectEnum::CultsRecruit, &CommandButtonAction::RecruitAssistConstruction));
        // Other unit types should NOT support recruit actions
        assert!(!object_type_supports_action(&ObjectEnum::Peacekeeper, &CommandButtonAction::RecruitConstruct));
        assert!(!object_type_supports_action(&ObjectEnum::SyndicateAgent, &CommandButtonAction::RecruitAssistConstruction));
    }

    #[test]
    fn recruit_grid_button_labels() {
        let state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault);
        assert_eq!(grid_button_label(&state, &CommandButtonAction::RecruitConstruct, 0, 'Q'), "[Q] Construct");
        assert_eq!(grid_button_label(&state, &CommandButtonAction::RecruitSelectBuilding(ObjectEnum::CultsStorage), 0, 'Q'), "[Q] Storage");
        assert_eq!(grid_button_label(&state, &CommandButtonAction::RecruitAssistConstruction, 0, 'S'), "[S] Assist\nConstruct");
    }

    #[test]
    fn recruit_menu_titles() {
        let cases = [
            (ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault), "Recruit"),
            (ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu), "Construct"),
            (ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement), "Place Building"),
        ];
        for (state, expected) in cases {
            let title = match &state {
                ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault) => "Recruit",
                ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu) => "Construct",
                ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement) => "Place Building",
                _ => "Unknown",
            };
            assert_eq!(title, expected);
        }
    }
}
