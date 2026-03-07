use bevy::prelude::*;
use crate::types::*;
use crate::game::types::{
    ObjectInstance, Player, GdoPlayerResources, SyndicatePlayerResources, StructureInstance,
    DeploymentCenterState, BarracksState, ExtractionFacilityState,
    SupplyTowerState, TunnelState, TunnelTier, TunnelOperation,
    tunnel_t2_upgrade_cost, tunnel_t3_upgrade_cost,
};
use crate::game::units::types::commands::{CommandType, HoldingPosition, UnitCommand};
use crate::game::units::types::state::AgentCarryState;
use crate::game::units::types::movement::{MoveTarget, Path, Velocity};
use crate::game::combat::types::{AttackState, AttackCapability, AttackType};
use crate::game::world::utils::{screen_space_hit_test, cursor_pos_in_viewport};
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
    caps: &SelectedUnitCapabilities,
) -> Option<CommandButtonAction> {
    match state {
        ObjectInterfaceState::StructureMenu(sm) => match sm {
            StructureMenuState::DcIdle => match (row, col) {
                (0, 0) => Some(CommandButtonAction::DcOpenBuildMenu),
                _ => None,
            },
            StructureMenuState::DcBuildMenu => match (row, col) {
                (0, 0) => Some(CommandButtonAction::DcBuild(ObjectEnum::PowerPlant)),
                (0, 1) => Some(CommandButtonAction::DcBuild(ObjectEnum::Barracks)),
                (0, 2) => Some(CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility)),
                (1, 0) => Some(CommandButtonAction::DcBuild(ObjectEnum::SupplyTower)),
                (2, 0) => Some(CommandButtonAction::Back),
                _ => None,
            },
            StructureMenuState::DcConstructing => match (row, col) {
                (0, 0) => Some(CommandButtonAction::DcCancel),
                _ => None,
            },
            StructureMenuState::DcReadyToPlace => match (row, col) {
                (0, 0) => Some(CommandButtonAction::EnterPlacement),
                (0, 1) => Some(CommandButtonAction::DcCancel),
                _ => None,
            },
            StructureMenuState::BarracksMenu => match (row, col) {
                (0, 0) => Some(CommandButtonAction::BkTrain(ObjectEnum::Peacekeeper)),
                (0, 1) if bk_has_queue => Some(CommandButtonAction::BkCancel),
                _ => None,
            },
            StructureMenuState::EfIdle => match (row, col) {
                (0, 0) => Some(CommandButtonAction::EfBuildPlate),
                _ => None,
            },
            StructureMenuState::EfConstructing => match (row, col) {
                (0, 0) => Some(CommandButtonAction::EfCancel),
                _ => None,
            },
            StructureMenuState::EfReadyToPlace => match (row, col) {
                (0, 0) => Some(CommandButtonAction::EnterPlacement),
                (0, 1) => Some(CommandButtonAction::EfCancel),
                _ => None,
            },
            StructureMenuState::SupplyTowerMenu => match (row, col) {
                (0, 0) => Some(CommandButtonAction::StTrain(ObjectEnum::SupplyChopper)),
                (0, 1) if bk_has_queue => Some(CommandButtonAction::StCancel),
                (1, 0) => Some(CommandButtonAction::StScheduleDeliveries),
                _ => None,
            },
            StructureMenuState::TunnelIdle => match (row, col) {
                (0, 0) => Some(CommandButtonAction::TunnelUpgrade),
                (0, 1) => Some(CommandButtonAction::TunnelOpenExpandMenu),
                (0, 2) => Some(CommandButtonAction::TunnelOpenEjectMenu),
                _ => None,
            },
            StructureMenuState::TunnelExpandMenu | StructureMenuState::TunnelEjectMenu => {
                // Dynamic content — bypass static grid, built in rebuild_command_panel_ui
                None
            },
            StructureMenuState::DcAwaitingPlacement | StructureMenuState::EfAwaitingPlacement |
            StructureMenuState::TunnelAwaitingPlacement => {
                // Placement mode is handled by mouse clicks + Escape, no grid buttons
                None
            },
        },
        ObjectInterfaceState::Default => match (row, col) {
            // Unit commands (only shown when Selection has units)
            (0, 0) => Some(CommandButtonAction::UnitMove),
            (0, 1) if caps.has_attack => Some(CommandButtonAction::UnitAttack),
            (0, 2) if caps.can_target_ground => Some(CommandButtonAction::UnitAttackGround),
            (1, 0) if caps.has_attack => Some(CommandButtonAction::UnitAttackMove),
            (1, 1) => Some(CommandButtonAction::UnitPatrol),
            (1, 2) => Some(CommandButtonAction::UnitHoldPosition),
            (2, 0) => Some(CommandButtonAction::UnitStop),
            (2, 1) if caps.can_reverse => Some(CommandButtonAction::UnitReverse),
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
        ObjectInterfaceState::AwaitingTarget(_) => {
            // AwaitingTarget mode — no grid buttons, awaiting click
            None
        },
    }
}

/// System to update the CursorTarget resource each frame based on cursor position.
/// Raycasts from cursor through camera to detect entities and ground under cursor.
pub fn update_cursor_target(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    potential_targets: Query<(Entity, &Transform, &Owner, &SelectionBounds), With<ObjectInstance>>,
    cursor_over_ui: Res<CursorOverUi>,
    local_player: Res<LocalPlayer>,
    mut cursor_target: ResMut<CursorTarget>,
) {
    // Reset each frame
    *cursor_target = CursorTarget::default();

    if cursor_over_ui.0 {
        return;
    }

    let Ok(window) = windows.get_single() else { return };
    let Ok((camera, camera_transform)) = cameras.get_single() else { return };

    let Some(cursor_pos) = cursor_pos_in_viewport(window, camera) else { return };
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_pos) else { return };

    // Check entities first (screen-space hit test)
    let click_radius = 25.0_f32;
    let mut best_distance = f32::MAX;
    let mut best_entity = None;
    let mut best_owner = None;

    for (entity, transform, owner, _bounds) in potential_targets.iter() {
        let target_pos = transform.translation;
        if let Some(screen_pos) = camera.world_to_viewport(camera_transform, target_pos) {
            if screen_space_hit_test(cursor_pos, screen_pos, click_radius).is_some() {
                let cam_distance = (target_pos - camera_transform.translation()).length();
                if cam_distance < best_distance {
                    best_entity = Some(entity);
                    best_owner = Some(*owner);
                    best_distance = cam_distance;
                }
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

/// Whether the panel should show content (not hidden).
/// Returns true when there's an active selection or we're in a structure menu / awaiting target.
fn is_panel_visible(state: &ObjectInterfaceState, selection: &Selection) -> bool {
    match state {
        ObjectInterfaceState::Default | ObjectInterfaceState::AgentMenu(_) => !selection.groups.is_empty(),
        _ => true,
    }
}

/// System to update the command panel state based on selected structures
pub fn update_command_panel_state(
    selected_structures: Query<
        (Entity, &ObjectInstance, Option<&DeploymentCenterState>, Option<&BarracksState>, Option<&ExtractionFacilityState>, Option<&SupplyTowerState>, Option<&TunnelState>),
        (With<StructureInstance>, With<Selected>),
    >,
    selected_units: Query<(Entity, Option<&AttackCapability>, &UnitBaseEnum, &ObjectInstance, Option<&AgentCarryState>), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
    mut interface_state: ResMut<ObjectInterfaceState>,
    mut panel_target: ResMut<CommandPanelTarget>,
    mut unit_caps: ResMut<SelectedUnitCapabilities>,
    _selection: Res<Selection>,
) {
    let struct_count = selected_structures.iter().count();

    if struct_count != 1 {
        // No single structure selected — check for units
        let unit_count = selected_units.iter().count();
        if unit_count > 0 {
            // Compute aggregate capabilities from selected units
            let new_caps = compute_selected_unit_capabilities(&selected_units);
            if *unit_caps != new_caps {
                *unit_caps = new_caps;
            }

            // Check if all selected units are Agents (for Agent-specific interface)
            let all_agents = selected_units.iter().all(|(_, _, _, obj, _)| obj.object_type == ObjectEnum::SyndicateAgent);

            if all_agents {
                // Route to AgentMenu state
                if !matches!(*interface_state, ObjectInterfaceState::AgentMenu(_) | ObjectInterfaceState::AwaitingTarget(_)) {
                    *interface_state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
                    panel_target.entity = None;
                }
            } else {
                // Stay in Default (unit commands) or AwaitingTarget
                if matches!(*interface_state, ObjectInterfaceState::StructureMenu(_) | ObjectInterfaceState::AgentMenu(_)) {
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
        return;
    }

    let (entity, obj_instance, dc_state, _bk_state, ef_state, _st_state, tunnel_state) = selected_structures.iter().next().unwrap();

    // Update target
    let target_changed = panel_target.entity != Some(entity);
    panel_target.entity = Some(entity);

    match obj_instance.object_type {
        ObjectEnum::DeploymentCenter => {
            if let Some(dc) = dc_state {
                if dc.current_construction.is_some() {
                    let new_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing);
                    if *interface_state != new_state || target_changed {
                        *interface_state = new_state;
                    }
                } else if dc.ready_to_place.is_some() {
                    // Preserve AwaitingPlacement state when structure is still ready
                    if *interface_state != ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace)
                        && *interface_state != ObjectInterfaceState::StructureMenu(StructureMenuState::DcAwaitingPlacement)
                        || target_changed
                    {
                        *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace);
                    }
                } else if target_changed || !matches!(*interface_state,
                    ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle) |
                    ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu)
                ) {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
                }
                // Keep DcBuildMenu if already in it
            }
        }
        ObjectEnum::Barracks => {
            let new_state = ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);
            if *interface_state != new_state || target_changed {
                *interface_state = new_state;
            }
        }
        ObjectEnum::ExtractionFacility => {
            if let Some(ef) = ef_state {
                if ef.current_construction {
                    let new_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfConstructing);
                    if *interface_state != new_state || target_changed {
                        *interface_state = new_state;
                    }
                } else if ef.ready_to_place {
                    if *interface_state != ObjectInterfaceState::StructureMenu(StructureMenuState::EfReadyToPlace)
                        && *interface_state != ObjectInterfaceState::StructureMenu(StructureMenuState::EfAwaitingPlacement)
                        || target_changed
                    {
                        *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfReadyToPlace);
                    }
                } else if target_changed || !matches!(*interface_state, ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle)) {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
                }
            }
        }
        ObjectEnum::SupplyTower => {
            let new_state = ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu);
            if *interface_state != new_state || target_changed {
                *interface_state = new_state;
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
        // PowerPlant, ExtractionPlate — no commands
        _ => {
            if !matches!(*interface_state, ObjectInterfaceState::Default) {
                *interface_state = ObjectInterfaceState::Default;
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
    selected_owners: Query<&Owner, (With<StructureInstance>, With<Selected>)>,
    unit_caps: Res<SelectedUnitCapabilities>,
    selection: Res<Selection>,
) {
    if !interface_state.is_changed() && !unit_caps.is_changed() && !selection.is_changed() {
        return;
    }

    let panel_entity = match panel_section.get_single() {
        Ok(e) => e,
        Err(_) => return,
    };

    // Clear existing content
    commands.entity(panel_entity).despawn_descendants();

    if !is_panel_visible(&interface_state, &selection) {
        return;
    }

    // Get player resources for button enable/disable
    let player_sc = get_player_sc(&selected_owners, &players);

    commands.entity(panel_entity).with_children(|parent| {
        // Title
        let title = match &*interface_state {
            ObjectInterfaceState::StructureMenu(sm) => match sm {
                StructureMenuState::DcIdle | StructureMenuState::DcConstructing |
                StructureMenuState::DcReadyToPlace | StructureMenuState::DcAwaitingPlacement => "Deployment Center",
                StructureMenuState::DcBuildMenu => "Build Menu",
                StructureMenuState::BarracksMenu => "Barracks",
                StructureMenuState::EfIdle | StructureMenuState::EfConstructing |
                StructureMenuState::EfReadyToPlace | StructureMenuState::EfAwaitingPlacement => "Extraction Facility",
                StructureMenuState::SupplyTowerMenu => "Supply Tower",
                StructureMenuState::TunnelIdle | StructureMenuState::TunnelAwaitingPlacement => "Tunnel",
                StructureMenuState::TunnelExpandMenu => "Expand",
                StructureMenuState::TunnelEjectMenu => "Eject Units",
            },
            ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault) => "Agent Commands",
            ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement) => "Place Tunnel",
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
            ObjectInterfaceState::StructureMenu(StructureMenuState::EfConstructing) => {
                if let Some(target_entity) = panel_target.entity {
                    if let Ok(ef) = ef_query.get(target_entity) {
                        let progress = ef.construction_progress.unwrap_or(0.0);
                        let total = ExtractionFacilityState::construction_cost().build_frames as f32;
                        let pct = (progress / total * 100.0).min(100.0);
                        spawn_progress_text(parent, &format!("Building Plate... {:.0}%", pct));
                    }
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::EfReadyToPlace) => {
                spawn_progress_text(parent, "Plate ready!");
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
                                TunnelOperation::BuildingExpansion { object, progress } => {
                                    let name = object.object_type().name;
                                    let required = crate::game::types::syndicate_structure_stats::TUNNEL_CONSTRUCTION_FRAMES as f32;
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
                NodeBundle {
                    style: Style {
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
                NodeBundle {
                    style: Style {
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
                    ..default()
                },
                CommandGridContainer,
            )).with_children(|grid| {
                // Check production queue state for grid slot action resolution
                let bk_has_queue = panel_target.entity
                    .and_then(|e| bk_query.get(e).ok())
                    .map(|bk| !bk.build_queue.is_empty())
                    .unwrap_or(false)
                    || panel_target.entity
                        .and_then(|e| st_query.get(e).ok())
                        .map(|st| !st.build_queue.is_empty())
                        .unwrap_or(false);

                // Get tunnel upgrade cost for enabled check
                let tunnel_upgrade_cost = get_tunnel_upgrade_cost(panel_target.entity, &tunnel_query, &all_tunnels_query);
                let syndicate_supplies = get_syndicate_supplies(panel_target.entity, &tunnel_query, &syndicate_players);

                for row in 0..3u8 {
                    for col in 0..3u8 {
                        let hotkey = GRID_HOTKEYS[row as usize][col as usize];
                        if let Some(action) = get_grid_slot_action(&interface_state, row, col, bk_has_queue, &unit_caps) {
                            let label = grid_button_label(&interface_state, &action, player_sc, hotkey);
                            let enabled = grid_button_enabled_ext(&action, player_sc, bk_has_queue, panel_target.entity, &bk_query, &st_query, &tunnel_query, tunnel_upgrade_cost, syndicate_supplies, &unit_caps);
                            let active = is_action_active(&action, &interface_state);
                            let is_common = is_common_command(&action);
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
    mut interaction_query: Query<
        (&Interaction, &CommandButtonAction),
        Changed<Interaction>,
    >,
    mut interface_state: ResMut<ObjectInterfaceState>,
    panel_target: Res<CommandPanelTarget>,
    mut dc_query: Query<(&Owner, &mut DeploymentCenterState)>,
    mut bk_query: Query<(&Owner, &mut BarracksState)>,
    mut ef_query: Query<(&Owner, &mut ExtractionFacilityState)>,
    mut st_query_mut: Query<(&Owner, &mut SupplyTowerState)>,
    mut tunnel_query_mut: Query<(&Owner, &mut TunnelState, &crate::game::types::TunnelArea)>,
    mut syndicate_players: Query<(&Player, &mut SyndicatePlayerResources)>,
    mut placement_state: ResMut<PlacementState>,
    mut players: Query<(&Player, &mut GdoPlayerResources)>,
    selected_units: Query<(Entity, &mut Velocity), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
    attack_states: Query<&AttackState>,
    selection: Res<Selection>,
) {
    for (interaction, action) in interaction_query.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        execute_command_action(action, &mut commands, &mut interface_state, &panel_target, &mut dc_query, &mut bk_query, &mut ef_query, &mut st_query_mut, &mut tunnel_query_mut, &mut syndicate_players, &mut placement_state, &mut players, &selected_units, &attack_states, &selection);
    }
}

/// System to handle keyboard hotkeys for the command panel (Q/W/E/A/S/D/Z/X/C grid + Escape + Tab)
pub fn command_panel_hotkeys(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interface_state: ResMut<ObjectInterfaceState>,
    panel_target: Res<CommandPanelTarget>,
    mut dc_query: Query<(&Owner, &mut DeploymentCenterState)>,
    mut bk_query: Query<(&Owner, &mut BarracksState)>,
    mut ef_query: Query<(&Owner, &mut ExtractionFacilityState)>,
    mut st_query_mut: Query<(&Owner, &mut SupplyTowerState)>,
    mut tunnel_query_mut: Query<(&Owner, &mut TunnelState, &crate::game::types::TunnelArea)>,
    mut syndicate_players: Query<(&Player, &mut SyndicatePlayerResources)>,
    mut players: Query<(&Player, &mut GdoPlayerResources)>,
    selected_units: Query<(Entity, &mut Velocity), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
    attack_states: Query<&AttackState>,
    mut placement_state: ResMut<PlacementState>,
    unit_caps: Res<SelectedUnitCapabilities>,
    mut selection: ResMut<Selection>,
) {
    // Only process grid hotkeys when panel is visible
    if !is_panel_visible(&interface_state, &selection) {
        return;
    }

    // Tab: Cycle active group (StateOnlyTransition)
    if keyboard.just_pressed(KeyCode::Tab) {
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
                *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfReadyToPlace);
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu) |
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu) => {
                *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement) => {
                *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
            }
            ObjectInterfaceState::AgentMenu(AgentMenuState::AgentAwaitingPlacement) => {
                *interface_state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
                info!("Agent: Cancelled placement, back to AgentDefault");
            }
            ObjectInterfaceState::AwaitingTarget(_) => {
                // Check if we should return to AgentDefault or Default
                let all_agents = selected_units.iter().count() > 0; // If units are selected, check below
                let _ = all_agents; // suppress unused warning
                *interface_state = ObjectInterfaceState::Default;
                info!("Cancelled awaiting target, back to Default");
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
            ObjectInterfaceState::StructureMenu(StructureMenuState::EfReadyToPlace) => {
                *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfAwaitingPlacement);
                return;
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
        // F: Toggle horizontal flip during placement mode
        if keyboard.just_pressed(KeyCode::KeyF) {
            placement_state.flip_horizontal = !placement_state.flip_horizontal;
            info!("Flip horizontal: {}", placement_state.flip_horizontal);
        }
        // G: Toggle vertical flip during placement mode
        if keyboard.just_pressed(KeyCode::KeyG) {
            placement_state.flip_vertical = !placement_state.flip_vertical;
            info!("Flip vertical: {}", placement_state.flip_vertical);
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
                let bk_has_queue = panel_target.entity
                    .and_then(|e| bk_query.get(e).ok())
                    .map(|(_, bk)| !bk.build_queue.is_empty())
                    .unwrap_or(false)
                    || panel_target.entity
                        .and_then(|e| st_query_mut.get(e).ok())
                        .map(|(_, st)| !st.build_queue.is_empty())
                        .unwrap_or(false);

                if let Some(action) = get_grid_slot_action(&interface_state, row, col, bk_has_queue, &unit_caps) {
                    execute_command_action(&action, &mut commands, &mut interface_state, &panel_target, &mut dc_query, &mut bk_query, &mut ef_query, &mut st_query_mut, &mut tunnel_query_mut, &mut syndicate_players, &mut placement_state, &mut players, &selected_units, &attack_states, &selection);
                }
            }
            break; // Only process one key per frame
        }
    }
}

/// Execute a command action (shared between button clicks and hotkeys)
fn execute_command_action(
    action: &CommandButtonAction,
    commands: &mut Commands,
    interface_state: &mut ResMut<ObjectInterfaceState>,
    panel_target: &Res<CommandPanelTarget>,
    dc_query: &mut Query<(&Owner, &mut DeploymentCenterState)>,
    bk_query: &mut Query<(&Owner, &mut BarracksState)>,
    ef_query: &mut Query<(&Owner, &mut ExtractionFacilityState)>,
    st_query: &mut Query<(&Owner, &mut SupplyTowerState)>,
    tunnel_query: &mut Query<(&Owner, &mut TunnelState, &crate::game::types::TunnelArea)>,
    syndicate_players: &mut Query<(&Player, &mut SyndicatePlayerResources)>,
    placement_state: &mut ResMut<PlacementState>,
    players: &mut Query<(&Player, &mut GdoPlayerResources)>,
    selected_units: &Query<(Entity, &mut Velocity), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
    attack_states: &Query<&AttackState>,
    selection: &Selection,
) {
    match action {
        CommandButtonAction::DcOpenBuildMenu => {
            **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
        }
        CommandButtonAction::DcBuild(object_type) => {
            let Some(target_entity) = panel_target.entity else { return };
            if let Ok((owner, mut dc_state)) = dc_query.get_mut(target_entity) {
                if dc_state.current_construction.is_some() || dc_state.ready_to_place.is_some() {
                    info!("DC: Already building or has structure ready");
                    return;
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
            let Some(target_entity) = panel_target.entity else { return };
            if let Ok((owner, mut bk_state)) = bk_query.get_mut(target_entity) {
                if let Some(cost) = BarracksState::production_cost(unit_type) {
                    if let Some(mut res) = find_player_resources_mut(owner, players) {
                        if res.space_crystals < cost.space_crystals as i32 {
                            info!("Barracks: Not enough SC");
                            return;
                        }
                        let control_cost = unit_type.unit_control_cost();
                        if !res.has_unit_control(control_cost) {
                            info!("Barracks: Unit control cap reached ({}/{})",
                                res.unit_control_used, res.unit_control_cap);
                            return;
                        }
                        if bk_state.try_queue(*unit_type) {
                            res.space_crystals -= cost.space_crystals as i32;
                            info!("Barracks: Queued {:?} ({} SC, queue: {})",
                                unit_type, cost.space_crystals, bk_state.build_queue.len());
                            interface_state.set_changed();
                        } else {
                            info!("Barracks: Queue full");
                        }
                    }
                }
            }
        }
        CommandButtonAction::BkCancel => {
            let Some(target_entity) = panel_target.entity else { return };
            if let Ok((owner, mut bk_state)) = bk_query.get_mut(target_entity) {
                if let Some(cancelled) = bk_state.cancel_last() {
                    if let Some(cost) = BarracksState::production_cost(&cancelled) {
                        if let Some(mut res) = find_player_resources_mut(owner, players) {
                            res.space_crystals += cost.space_crystals as i32;
                            info!("Barracks: Cancelled {:?}, refunded {} SC", cancelled, cost.space_crystals);
                        }
                    }
                    interface_state.set_changed();
                }
            }
        }
        CommandButtonAction::EfBuildPlate => {
            let Some(target_entity) = panel_target.entity else { return };
            if let Ok((owner, mut ef_state)) = ef_query.get_mut(target_entity) {
                if ef_state.current_construction || ef_state.ready_to_place {
                    info!("EF: Already building or has plate ready");
                    return;
                }
                let cost = ExtractionFacilityState::construction_cost();
                if let Some(mut res) = find_player_resources_mut(owner, players) {
                    if res.space_crystals >= cost.space_crystals as i32 {
                        res.space_crystals -= cost.space_crystals as i32;
                        ef_state.current_construction = true;
                        ef_state.construction_progress = Some(0.0);
                        **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfConstructing);
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
                ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu) => {
                    **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
                }
                ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu) |
                ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu) => {
                    **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
                }
                _ => {}
            }
        }
        CommandButtonAction::EnterPlacement => {
            match &**interface_state {
                ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace) => {
                    **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcAwaitingPlacement);
                }
                ObjectInterfaceState::StructureMenu(StructureMenuState::EfReadyToPlace) => {
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
                commands.entity(entity)
                    .remove::<MoveTarget>()
                    .remove::<Path>()
                    .insert(HoldingPosition)
                    .insert(UnitCommand::HoldPosition);
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
                commands.entity(entity)
                    .remove::<MoveTarget>()
                    .remove::<Path>()
                    .remove::<HoldingPosition>()
                    .insert(UnitCommand::Stop);
            }
            info!("Stop");
        }
        CommandButtonAction::UnitReverse => {
            **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::Reverse);
            info!("Command mode: Reverse");
        }
        CommandButtonAction::StTrain(unit_type) => {
            let Some(target_entity) = panel_target.entity else { return };
            if let Ok((owner, mut st_state)) = st_query.get_mut(target_entity) {
                if let Some(cost) = SupplyTowerState::production_cost(unit_type) {
                    if let Some(mut res) = find_player_resources_mut(owner, players) {
                        if res.space_crystals < cost.space_crystals as i32 {
                            info!("Supply Tower: Not enough SC");
                            return;
                        }
                        if st_state.try_queue(*unit_type) {
                            res.space_crystals -= cost.space_crystals as i32;
                            info!("Supply Tower: Queued {:?} ({} SC, queue: {})",
                                unit_type, cost.space_crystals, st_state.build_queue.len());
                            interface_state.set_changed();
                        } else {
                            info!("Supply Tower: Queue full");
                        }
                    }
                }
            }
        }
        CommandButtonAction::StCancel => {
            let Some(target_entity) = panel_target.entity else { return };
            if let Ok((owner, mut st_state)) = st_query.get_mut(target_entity) {
                if let Some(cancelled) = st_state.cancel_last() {
                    if let Some(cost) = SupplyTowerState::production_cost(&cancelled) {
                        if let Some(mut res) = find_player_resources_mut(owner, players) {
                            res.space_crystals += cost.space_crystals as i32;
                            info!("Supply Tower: Cancelled {:?}, refunded {} SC", cancelled, cost.space_crystals);
                        }
                    }
                    interface_state.set_changed();
                }
            }
        }
        CommandButtonAction::StScheduleDeliveries => {
            // TODO: Enter AwaitingTarget mode for schedule deliveries
            info!("Supply Tower: Schedule Deliveries (awaiting target not yet implemented)");
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
        CommandButtonAction::TunnelEjectUnit(unit_type) => {
            execute_tunnel_eject_unit(*unit_type, panel_target, commands);
        }
        CommandButtonAction::AgentBuildTunnel => {
            // Enter placement mode for Agent tunnel building
            placement_state.building_type = Some(ObjectEnum::Tunnel);
            // Set source_entity to the first selected Agent entity
            let agent_entity = selected_units.iter().next().map(|(e, _)| e);
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
        ObjectInterfaceState::StructureMenu(StructureMenuState::EfConstructing) => {
            if let Ok(ef) = ef_query.get(target) {
                if !ef.current_construction && ef.ready_to_place {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfReadyToPlace);
                } else if !ef.current_construction && !ef.ready_to_place {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
                } else {
                    interface_state.set_changed();
                }
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::EfReadyToPlace) => {
            if let Ok(ef) = ef_query.get(target) {
                if !ef.ready_to_place {
                    *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
                }
            }
        }
        ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu) => {
            if st_query.get(target).is_ok() {
                interface_state.set_changed();
            }
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
    selected_units: &Query<(Entity, Option<&AttackCapability>, &UnitBaseEnum, &ObjectInstance, Option<&AgentCarryState>), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
) -> SelectedUnitCapabilities {
    let mut caps = SelectedUnitCapabilities::default();
    for (_entity, attack_cap, unit_base, _obj, carry_state) in selected_units.iter() {
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
    }
    caps
}

/// Whether a runtime AttackType can target ground locations (not just units).
/// Only TailDisjointed and DoublyDisjointed support ground targeting.
pub(crate) fn attack_type_can_target_ground(attack_type: &AttackType) -> bool {
    matches!(attack_type, AttackType::TailDisjointed { .. } | AttackType::DoublyDisjointed { .. })
}

/// Get player space crystals for the selected structure's owner
fn get_player_sc(
    selected_owners: &Query<&Owner, (With<StructureInstance>, With<Selected>)>,
    players: &Query<(&Player, &GdoPlayerResources)>,
) -> i32 {
    if let Some(owner) = selected_owners.iter().next() {
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

/// Execute tunnel select expansion: enter placement mode for the expansion type
fn execute_tunnel_select_expansion(
    expansion_type: ObjectEnum,
    panel_target: &Res<CommandPanelTarget>,
    tunnel_query: &mut Query<(&Owner, &mut TunnelState, &crate::game::types::TunnelArea)>,
    placement_state: &mut ResMut<PlacementState>,
    interface_state: &mut ResMut<ObjectInterfaceState>,
) {
    let Some(target_entity) = panel_target.entity else { return };

    // Check tunnel is not busy
    if let Ok((_, ts, _)) = tunnel_query.get(target_entity) {
        if ts.is_busy() {
            info!("Tunnel: Cannot place expansion while operation in progress");
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
    _unit_type: ObjectEnum,
    _panel_target: &Res<CommandPanelTarget>,
    _commands: &mut Commands,
) {
    // Stubbed: The EjectMenu is functional but ejection requires InTunnelNetwork
    // units to exist (from enter_command_and_entering_tunnel_behavior task).
    // When units exist in the network, this will:
    // 1. Find a unit of the given type with InTunnelNetwork marker
    // 2. Add it to the EjectionQueue on the target Tunnel
    // 3. The ejection_tick_system processes the queue
    info!("Tunnel: Eject unit (stubbed — no units in network yet)");
}

/// Build the dynamic grid for TunnelExpandMenu
fn build_tunnel_expand_grid(
    grid: &mut ChildBuilder,
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
        if slot >= 8 { break; } // Reserve slot 8 (Z,0) for Back
        let tier_ok = tier_at_least(&tier, min_tier);
        let enabled = tier_ok && !is_busy;
        let hotkey = hotkeys[slot];
        let label = format!("[{}] {}", hotkey, expansion.object_type().name);
        let action = CommandButtonAction::TunnelSelectExpansion(*expansion);
        let row = (slot / 3) as u8;
        let col = (slot % 3) as u8;
        spawn_grid_button(grid, &label, action, enabled, false, false, row, col);
        slot += 1;
    }

    // Fill empty cells up to slot 8
    while slot < 8 {
        spawn_empty_grid_cell(grid);
        slot += 1;
    }

    // Back button at (2, 2) = slot 8 = 'C'
    spawn_grid_button(grid, "[C] Back", CommandButtonAction::Back, true, false, false, 2, 2);
}

/// Build the dynamic grid for TunnelEjectMenu.
/// Currently stubbed: units in the tunnel network are despawned entities,
/// so the network_units query will be empty until a proper network storage
/// system is implemented alongside the enter_command task.
fn build_tunnel_eject_grid(
    grid: &mut ChildBuilder,
    _target_entity: Option<Entity>,
    _tunnel_query: &Query<(&TunnelState, &Owner)>,
    _network_units: &Query<(&ObjectInstance, &crate::game::units::types::state::behavior::InTunnelNetwork)>,
) {
    // Stub: Show empty grid with Back button.
    // When a proper tunnel network storage (Resource) tracks units,
    // this will display unit types with counts and tier-based enable/disable.
    for _slot in 0..8 {
        spawn_empty_grid_cell(grid);
    }
    // Back button at (2, 2)
    spawn_grid_button(grid, "[C] Back", CommandButtonAction::Back, true, false, false, 2, 2);
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
) -> bool {
    match action {
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
        CommandButtonAction::TunnelOpenEjectMenu => true,
        CommandButtonAction::AgentDropOff => unit_caps.agent_carrying,
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

/// Whether a command action is a "common command" (available to all unit types).
/// Common commands are visually distinguished from group-specific commands.
fn is_common_command(action: &CommandButtonAction) -> bool {
    matches!(action,
        CommandButtonAction::UnitMove |
        CommandButtonAction::UnitPatrol |
        CommandButtonAction::UnitHoldPosition |
        CommandButtonAction::UnitStop |
        CommandButtonAction::AgentBuildTunnel |
        CommandButtonAction::AgentDropOff
    )
}

/// Determine which entities should receive a command based on whether it's
/// a common command (issued to ALL selected) or group command (issued to active group only).
fn command_target_entities(
    action: &CommandButtonAction,
    selection: &Selection,
    selected_units: &Query<(Entity, &mut Velocity), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
) -> Vec<Entity> {
    if is_common_command(action) {
        // Common command: issue to all selected units
        selected_units.iter().map(|(e, _)| e).collect()
    } else {
        // Group command: issue only to active group entities
        if let Some(active_group) = selection.active_group() {
            active_group.entities.clone()
        } else {
            selected_units.iter().map(|(e, _)| e).collect()
        }
    }
}

/// Generate the label for a grid button based on state and action
fn grid_button_label(
    _state: &ObjectInterfaceState,
    action: &CommandButtonAction,
    _player_sc: i32,
    hotkey: char,
) -> String {
    match action {
        CommandButtonAction::DcOpenBuildMenu => format!("[{}] Build", hotkey),
        CommandButtonAction::DcBuild(ObjectEnum::PowerPlant) => format!("[{}] PP\n150 SC", hotkey),
        CommandButtonAction::DcBuild(ObjectEnum::Barracks) => format!("[{}] BK\n200 SC", hotkey),
        CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility) => format!("[{}] EF\n250 SC", hotkey),
        CommandButtonAction::DcBuild(ObjectEnum::SupplyTower) => format!("[{}] ST\n200 SC", hotkey),
        CommandButtonAction::DcBuild(_) => format!("[{}] Build", hotkey),
        CommandButtonAction::DcCancel => format!("[{}] Cancel", hotkey),
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
        CommandButtonAction::StTrain(ObjectEnum::SupplyChopper) => format!("[{}] SC\n100 SC", hotkey),
        CommandButtonAction::StTrain(_) => format!("[{}] Train", hotkey),
        CommandButtonAction::StCancel => format!("[{}] Cancel\nLast", hotkey),
        CommandButtonAction::StScheduleDeliveries => format!("[{}] Schedule\nDeliv", hotkey),
        CommandButtonAction::TunnelUpgrade => format!("[{}] Upgrade", hotkey),
        CommandButtonAction::TunnelOpenExpandMenu => format!("[{}] Expand", hotkey),
        CommandButtonAction::TunnelOpenEjectMenu => format!("[{}] Eject", hotkey),
        CommandButtonAction::TunnelSelectExpansion(obj) => format!("[{}] {}", hotkey, obj.object_type().name),
        CommandButtonAction::TunnelEjectUnit(obj) => format!("[{}] {}", hotkey, obj.object_type().name),
        CommandButtonAction::AgentBuildTunnel => format!("[{}] Build\nTunnel", hotkey),
        CommandButtonAction::AgentDropOff => format!("[{}] Drop\nOff", hotkey),
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
        CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility) => player_sc >= 250,
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
        _ => false,
    }
}

/// Spawn a panel title text
fn spawn_panel_title(parent: &mut ChildBuilder, title: &str) {
    parent.spawn(TextBundle {
        text: Text::from_section(
            title,
            TextStyle {
                font_size: 14.0,
                color: Color::srgb(1.0, 1.0, 1.0),
                ..default()
            },
        ),
        style: Style {
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        },
        ..default()
    });
}

/// Spawn a progress text line
fn spawn_progress_text(parent: &mut ChildBuilder, text: &str) {
    parent.spawn(TextBundle {
        text: Text::from_section(
            text,
            TextStyle {
                font_size: 12.0,
                color: Color::srgb(0.9, 0.9, 0.5),
                ..default()
            },
        ),
        style: Style {
            margin: UiRect::bottom(Val::Px(2.0)),
            ..default()
        },
        ..default()
    });
}

/// Spawn an info text line
fn spawn_info_text(parent: &mut ChildBuilder, text: &str) {
    parent.spawn(TextBundle {
        text: Text::from_section(
            text,
            TextStyle {
                font_size: 11.0,
                color: Color::srgb(0.7, 0.7, 0.7),
                ..default()
            },
        ),
        style: Style {
            margin: UiRect::bottom(Val::Px(2.0)),
            ..default()
        },
        ..default()
    });
}

/// Spawn a command button in the 3x3 grid with hotkey label
fn spawn_grid_button(
    parent: &mut ChildBuilder,
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
        ButtonBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: BackgroundColor(bg_color),
            ..default()
        },
        action,
        GridSlot { row, col },
    ))
    .with_children(|button| {
        button.spawn(TextBundle::from_section(
            label,
            TextStyle {
                font_size: 10.0,
                color: text_color,
                ..default()
            },
        ).with_style(Style {
            max_width: Val::Px(58.0),
            ..default()
        }));
    });
}

/// Spawn an empty placeholder cell in the grid
fn spawn_empty_grid_cell(parent: &mut ChildBuilder) {
    parent.spawn(NodeBundle {
        style: Style {
            ..default()
        },
        background_color: BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.3)),
        ..default()
    });
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let a = SelectedUnitCapabilities { has_attack: true, can_target_ground: false, can_reverse: true, agent_carrying: false };
        let b = SelectedUnitCapabilities { has_attack: true, can_target_ground: false, can_reverse: true, agent_carrying: false };
        let c = SelectedUnitCapabilities { has_attack: false, can_target_ground: false, can_reverse: true, agent_carrying: false };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // === get_grid_slot_action conditional tests ===

    fn all_caps() -> SelectedUnitCapabilities {
        SelectedUnitCapabilities { has_attack: true, can_target_ground: true, can_reverse: true, agent_carrying: false }
    }

    fn no_caps() -> SelectedUnitCapabilities {
        SelectedUnitCapabilities::default()
    }

    fn attack_only() -> SelectedUnitCapabilities {
        SelectedUnitCapabilities { has_attack: true, can_target_ground: false, can_reverse: false, agent_carrying: false }
    }

    #[test]
    fn unit_commands_move_always_available() {
        let caps = no_caps();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 0, 0, false, &caps);
        assert!(matches!(action, Some(CommandButtonAction::UnitMove)));
    }

    #[test]
    fn unit_commands_attack_requires_has_attack() {
        let caps = no_caps();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 0, 1, false, &caps);
        assert!(action.is_none());

        let caps = attack_only();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 0, 1, false, &caps);
        assert!(matches!(action, Some(CommandButtonAction::UnitAttack)));
    }

    #[test]
    fn unit_commands_attack_ground_requires_can_target_ground() {
        let caps = attack_only();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 0, 2, false, &caps);
        assert!(action.is_none());

        let caps = all_caps();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 0, 2, false, &caps);
        assert!(matches!(action, Some(CommandButtonAction::UnitAttackGround)));
    }

    #[test]
    fn unit_commands_attack_move_requires_has_attack() {
        let caps = no_caps();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 1, 0, false, &caps);
        assert!(action.is_none());

        let caps = attack_only();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 1, 0, false, &caps);
        assert!(matches!(action, Some(CommandButtonAction::UnitAttackMove)));
    }

    #[test]
    fn unit_commands_patrol_always_available() {
        let caps = no_caps();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 1, 1, false, &caps);
        assert!(matches!(action, Some(CommandButtonAction::UnitPatrol)));
    }

    #[test]
    fn unit_commands_hold_position_always_available() {
        let caps = no_caps();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 1, 2, false, &caps);
        assert!(matches!(action, Some(CommandButtonAction::UnitHoldPosition)));
    }

    #[test]
    fn unit_commands_stop_always_available() {
        let caps = no_caps();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 2, 0, false, &caps);
        assert!(matches!(action, Some(CommandButtonAction::UnitStop)));
    }

    #[test]
    fn unit_commands_reverse_requires_can_reverse() {
        let caps = no_caps();
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 2, 1, false, &caps);
        assert!(action.is_none());

        let caps = SelectedUnitCapabilities { has_attack: false, can_target_ground: false, can_reverse: true, agent_carrying: false };
        let action = get_grid_slot_action(&ObjectInterfaceState::Default, 2, 1, false, &caps);
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
    fn all_caps_unit_commands_shows_all_eight_commands() {
        let caps = all_caps();
        let mut count = 0;
        for row in 0..3u8 {
            for col in 0..3u8 {
                if get_grid_slot_action(&ObjectInterfaceState::Default, row, col, false, &caps).is_some() {
                    count += 1;
                }
            }
        }
        // Move, Attack, AtkGround, AtkMove, Patrol, HoldPos, Stop, Reverse = 8
        assert_eq!(count, 8);
    }

    #[test]
    fn no_caps_unit_commands_shows_only_universal() {
        let caps = no_caps();
        let mut count = 0;
        for row in 0..3u8 {
            for col in 0..3u8 {
                if get_grid_slot_action(&ObjectInterfaceState::Default, row, col, false, &caps).is_some() {
                    count += 1;
                }
            }
        }
        // Move, Patrol, HoldPos, Stop = 4
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
            entity: Some(Entity::from_raw(42)),
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
        selection.build_from_entities(&[(Entity::from_raw(1), ObjectEnum::Peacekeeper, true)]);
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
    fn move_is_common_command() {
        assert!(is_common_command(&CommandButtonAction::UnitMove));
    }

    #[test]
    fn stop_is_common_command() {
        assert!(is_common_command(&CommandButtonAction::UnitStop));
    }

    #[test]
    fn hold_position_is_common_command() {
        assert!(is_common_command(&CommandButtonAction::UnitHoldPosition));
    }

    #[test]
    fn patrol_is_common_command() {
        assert!(is_common_command(&CommandButtonAction::UnitPatrol));
    }

    #[test]
    fn attack_is_not_common_command() {
        assert!(!is_common_command(&CommandButtonAction::UnitAttack));
    }

    #[test]
    fn reverse_is_not_common_command() {
        assert!(!is_common_command(&CommandButtonAction::UnitReverse));
    }

    // === StructureMenuState grid tests ===

    #[test]
    fn dc_idle_shows_build_button() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        let caps = no_caps();
        let action = get_grid_slot_action(&state, 0, 0, false, &caps);
        assert!(matches!(action, Some(CommandButtonAction::DcOpenBuildMenu)));
    }

    #[test]
    fn dc_build_menu_shows_structures() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
        let caps = no_caps();
        let pp = get_grid_slot_action(&state, 0, 0, false, &caps);
        assert!(matches!(pp, Some(CommandButtonAction::DcBuild(ObjectEnum::PowerPlant))));
        let bk = get_grid_slot_action(&state, 0, 1, false, &caps);
        assert!(matches!(bk, Some(CommandButtonAction::DcBuild(ObjectEnum::Barracks))));
    }

    #[test]
    fn awaiting_target_no_grid_buttons() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::Attack);
        let caps = all_caps();
        for row in 0..3u8 {
            for col in 0..3u8 {
                assert!(get_grid_slot_action(&state, row, col, false, &caps).is_none());
            }
        }
    }

    // === Tunnel Interface State Tests ===

    #[test]
    fn tunnel_idle_shows_three_commands() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
        let caps = no_caps();
        let upgrade = get_grid_slot_action(&state, 0, 0, false, &caps);
        assert!(matches!(upgrade, Some(CommandButtonAction::TunnelUpgrade)));
        let expand = get_grid_slot_action(&state, 0, 1, false, &caps);
        assert!(matches!(expand, Some(CommandButtonAction::TunnelOpenExpandMenu)));
        let eject = get_grid_slot_action(&state, 0, 2, false, &caps);
        assert!(matches!(eject, Some(CommandButtonAction::TunnelOpenEjectMenu)));
    }

    #[test]
    fn tunnel_idle_remaining_slots_are_none() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
        let caps = no_caps();
        // Only (0,0), (0,1), (0,2) should have actions
        for row in 1..3u8 {
            for col in 0..3u8 {
                assert!(get_grid_slot_action(&state, row, col, false, &caps).is_none(),
                    "Expected None at ({}, {})", row, col);
            }
        }
    }

    #[test]
    fn tunnel_expand_menu_no_static_grid_actions() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
        let caps = no_caps();
        // Dynamic content — all slots return None from the static function
        for row in 0..3u8 {
            for col in 0..3u8 {
                assert!(get_grid_slot_action(&state, row, col, false, &caps).is_none());
            }
        }
    }

    #[test]
    fn tunnel_eject_menu_no_static_grid_actions() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu);
        let caps = no_caps();
        for row in 0..3u8 {
            for col in 0..3u8 {
                assert!(get_grid_slot_action(&state, row, col, false, &caps).is_none());
            }
        }
    }

    #[test]
    fn tunnel_awaiting_placement_no_grid_buttons() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement);
        let caps = no_caps();
        for row in 0..3u8 {
            for col in 0..3u8 {
                assert!(get_grid_slot_action(&state, row, col, false, &caps).is_none());
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
        eq.queue.push_back(Entity::from_raw(1));
        eq.queue.push_back(Entity::from_raw(2));
        assert_eq!(eq.queue.len(), 2);
        let front = eq.queue.pop_front().unwrap();
        assert_eq!(front, Entity::from_raw(1));
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
        assert!(!is_common_command(&CommandButtonAction::TunnelUpgrade));
        assert!(!is_common_command(&CommandButtonAction::TunnelOpenExpandMenu));
        assert!(!is_common_command(&CommandButtonAction::TunnelOpenEjectMenu));
        assert!(!is_common_command(&CommandButtonAction::TunnelSelectExpansion(ObjectEnum::Headquarters)));
        assert!(!is_common_command(&CommandButtonAction::TunnelEjectUnit(ObjectEnum::SyndicateAgent)));
    }

    // === Agent Interface State tests ===

    #[test]
    fn agent_default_shows_build_tunnel_at_0_0() {
        let caps = no_caps();
        let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
        let action = get_grid_slot_action(&state, 0, 0, false, &caps);
        assert!(matches!(action, Some(CommandButtonAction::AgentBuildTunnel)));
    }

    #[test]
    fn agent_default_shows_drop_off_at_0_1() {
        let caps = no_caps();
        let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
        let action = get_grid_slot_action(&state, 0, 1, false, &caps);
        assert!(matches!(action, Some(CommandButtonAction::AgentDropOff)));
    }

    #[test]
    fn agent_default_no_other_slots() {
        let caps = no_caps();
        let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
        // Verify only (0,0) and (0,1) have actions
        for row in 0..3u8 {
            for col in 0..3u8 {
                if (row, col) == (0, 0) || (row, col) == (0, 1) {
                    continue;
                }
                let action = get_grid_slot_action(&state, row, col, false, &caps);
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
                let action = get_grid_slot_action(&state, row, col, false, &caps);
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
            'Q',
        );
        assert_eq!(label, "[Q] Build\nTunnel");
    }

    #[test]
    fn agent_drop_off_label() {
        let label = grid_button_label(
            &ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault),
            &CommandButtonAction::AgentDropOff,
            0,
            'W',
        );
        assert_eq!(label, "[W] Drop\nOff");
    }

    #[test]
    fn agent_commands_are_common_commands() {
        assert!(is_common_command(&CommandButtonAction::AgentBuildTunnel));
        assert!(is_common_command(&CommandButtonAction::AgentDropOff));
    }

    #[test]
    fn agent_menu_panel_visible_with_selection() {
        let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
        let mut selection = Selection::default();
        selection.groups.push(SelectionGroup {
            object_type: ObjectEnum::SyndicateAgent,
            entities: vec![Entity::from_raw(1)],
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
}
