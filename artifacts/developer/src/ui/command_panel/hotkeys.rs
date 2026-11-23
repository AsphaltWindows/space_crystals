use bevy::prelude::*;
use crate::types::*;
use crate::game::types::{
    Player, GdoPlayerResources, SyndicatePlayerResources, StructureInstance,
    DeploymentCenterState, BarracksState, ExtractionFacilityState,
    SupplyTowerState, TunnelState, TunnelOperation,
    HeadquartersState, RecruitmentCenterState, ArmoryState,
};
use crate::game::entities::EntityRegistry;
use crate::game::entities::types::SlotConditionContext;
use crate::game::units::types::commands::{CommandType, CommandQueue};
use crate::game::units::types::movement::Velocity;
use crate::game::combat::types::AttackState;
use super::super::types::*;
use super::state::is_panel_visible;
use super::grid_layout::{keycode_to_grid_slot, get_grid_slot_action};
use super::actions::execute_command_action;

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
    selected_units: Query<(Entity, &mut Velocity, Option<&mut CommandQueue>, Option<&AttackState>), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
    selection: Res<Selection>,
    registry: Res<EntityRegistry>,
) {
    let shift_held = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        execute_command_action(action, &mut commands, &mut interface_state, &panel_target, &mut dc_query, &mut bk_hq_query, &mut ef_query, &mut st_query_mut, &mut tunnel_query_mut, &mut syndicate_players, &mut placement_state, &mut players, &selected_units, shift_held, &selection, &registry);
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
    selected_units: Query<(Entity, &mut Velocity, Option<&mut CommandQueue>, Option<&AttackState>), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
    mut placement_state: ResMut<PlacementState>,
    unit_caps: Res<SelectedUnitCapabilities>,
    mut selection: ResMut<Selection>,
    registry: Res<EntityRegistry>,
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
        KeyCode::KeyQ, KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyR,
        KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD, KeyCode::KeyF,
        KeyCode::KeyZ, KeyCode::KeyX, KeyCode::KeyC, KeyCode::KeyV,
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

                let ctx = SlotConditionContext {
                    has_queue: bk_has_queue,
                    has_active_construction,
                    tunnel_is_upgrading,
                    has_ready_plate,
                };
                if let Some(action) = get_grid_slot_action(&interface_state, row, col, selection.active_type(), &registry, &ctx) {
                    execute_command_action(&action, &mut commands, &mut interface_state, &panel_target, &mut dc_query, &mut bk_hq_query, &mut ef_query, &mut st_query_mut, &mut tunnel_query_mut, &mut syndicate_players, &mut placement_state, &mut players, &selected_units, shift_held, &selection, &registry);
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
pub(super) fn right_click_cancel_target(
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
pub(super) enum RallyTargetKind {
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
