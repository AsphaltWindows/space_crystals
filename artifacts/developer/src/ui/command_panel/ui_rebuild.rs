use bevy::prelude::*;
use crate::types::*;
use crate::game::types::{
    ObjectInstance, Player, GdoPlayerResources, SyndicatePlayerResources, StructureInstance,
    DeploymentCenterState, BarracksState, ExtractionFacilityState,
    SupplyTowerState, TunnelState, TunnelTier, TunnelOperation,
    HeadquartersState, RecruitmentCenterState, ArmoryState,
};
use crate::game::entities::EntityRegistry;
use crate::game::entities::types::SlotConditionContext;
use crate::game::units::types::commands::CommandType;
use super::super::types::*;
use super::state::is_panel_visible;
use super::grid_layout::{GRID_HOTKEYS, get_grid_slot_action};
use super::helpers::{get_player_sc_from_owners, grid_button_label, grid_button_enabled_ext, is_action_active, is_common_command, get_dc_construction_info, spawn_panel_title, spawn_progress_text, spawn_info_text, spawn_grid_button, spawn_empty_grid_cell};
use super::tunnel_actions::{build_tunnel_expand_grid, build_tunnel_eject_grid, get_tunnel_upgrade_cost, get_syndicate_supplies};

pub fn rebuild_command_panel_ui(
    mut commands: Commands,
    panel_section: Query<Entity, With<CommandPanelSection>>,
    interface_state: Res<ObjectInterfaceState>,
    panel_target: Res<CommandPanelTarget>,
    building_states: Query<(Option<&DeploymentCenterState>, Option<&BarracksState>, Option<&ExtractionFacilityState>, Option<&SupplyTowerState>)>,
    tunnel_query: Query<(&TunnelState, &Owner)>,
    all_tunnels_query: Query<(Entity, &TunnelState), With<Owner>>,
    syndicate_players: Query<(&Player, &SyndicatePlayerResources)>,
    network_units: Query<(&ObjectInstance, &crate::game::units::types::state::behavior::InTunnelNetwork)>,
    players: Query<(&Player, &GdoPlayerResources)>,
    selected_owners: Query<(&Owner, Option<&HeadquartersState>, Option<&RecruitmentCenterState>, Option<&ArmoryState>), (With<StructureInstance>, With<Selected>)>,
    unit_caps: Res<SelectedUnitCapabilities>,
    selection: Res<Selection>,
    registry: Res<EntityRegistry>,
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

        // Extract building states for the panel target entity (combined query)
        let target_dc = panel_target.entity.and_then(|e| building_states.get(e).ok()).and_then(|(dc, _, _, _)| dc);
        let target_bk = panel_target.entity.and_then(|e| building_states.get(e).ok()).and_then(|(_, bk, _, _)| bk);
        let target_ef = panel_target.entity.and_then(|e| building_states.get(e).ok()).and_then(|(_, _, ef, _)| ef);
        let target_st = panel_target.entity.and_then(|e| building_states.get(e).ok()).and_then(|(_, _, _, st)| st);

        // Info/progress text above the grid
        match &*interface_state {
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle) => {
                if let Some(dc) = target_dc {
                    if dc.current_construction.is_some() {
                        let (name, pct) = get_dc_construction_info(dc, &registry);
                        spawn_progress_text(parent, &format!("Building {}... {:.0}%", name, pct));
                    } else if let Some(ref ready) = dc.ready_to_place {
                        let name = ready.object_type().name;
                        spawn_progress_text(parent, &format!("{} ready!", name));
                    }
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle) => {
                if let Some(ef) = target_ef {
                    if ef.current_construction {
                        let progress = ef.construction_progress.unwrap_or(0.0);
                        let total = registry.cost(&ObjectEnum::ExtractionPlate).map(|c| c.build_frames as f32).unwrap_or(96.0);
                        let pct = (progress / total * 100.0).min(100.0);
                        spawn_progress_text(parent, &format!("Building Plate... {:.0}%", pct));
                    } else if ef.ready_to_place {
                        spawn_progress_text(parent, "Plate ready!");
                    }
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing) => {
                if let Some(dc) = target_dc {
                    let (name, pct) = get_dc_construction_info(dc, &registry);
                    spawn_progress_text(parent, &format!("Building {}... {:.0}%", name, pct));
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace) => {
                if let Some(dc) = target_dc {
                    if let Some(ref ready) = dc.ready_to_place {
                        let name = ready.object_type().name;
                        spawn_progress_text(parent, &format!("{} ready!", name));
                    }
                }
            }
            ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu) => {
                if let Some(bk) = target_bk {
                    if bk.current_build.is_some() {
                        let progress = bk.current_build_progress.unwrap_or(0.0);
                        let cost = registry.cost(bk.current_build.as_ref().unwrap());
                        let total = cost.map(|c| c.build_frames as f32).unwrap_or(80.0);
                        let pct = (progress / total * 100.0).min(100.0);
                        spawn_progress_text(parent, &format!("Training... {:.0}%", pct));
                    }
                    if !bk.build_queue.is_empty() {
                        spawn_info_text(parent, &format!("Queue: {}/{}", bk.build_queue.len(), BarracksState::MAX_QUEUE_SIZE));
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
                if let Some(st) = target_st {
                    if st.current_build.is_some() {
                        let progress = st.current_build_progress.unwrap_or(0.0);
                        let cost = registry.cost(st.current_build.as_ref().unwrap());
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
            ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu) => {
                if let Some((_, Some(hq), _, _)) = selected_owners.iter().next() {
                    if hq.current_build.is_some() {
                        let progress = hq.current_build_progress.unwrap_or(0.0);
                        let cost = registry.cost(hq.current_build.as_ref().unwrap());
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
                    grid_template_columns: RepeatedGridTrack::flex(4, 1.0),
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
            // Standard 4x3 Grid container
            parent.spawn((
                Node {
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(4, 1.0),
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
                let bk_has_queue = target_bk.map(|bk| bk.has_cancellable()).unwrap_or(false)
                    || target_st.map(|st| st.has_cancellable()).unwrap_or(false)
                    || selected_owners.iter().next()
                        .and_then(|(_, hq, _, _)| hq)
                        .map(|hq| hq.has_cancellable())
                        .unwrap_or(false)
                    || selected_owners.iter().next()
                        .and_then(|(_, _, rc, _)| rc)
                        .map(|rc| rc.production_progress > 0)
                        .unwrap_or(false);

                // Check DC/EF construction state for conditional cancel in idle states
                let has_active_construction = target_dc
                    .map(|dc| dc.current_construction.is_some() || dc.ready_to_place.is_some())
                    .unwrap_or(false)
                    || target_ef
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
                let has_ready_plate = target_ef.map(|ef| ef.ready_to_place).unwrap_or(false);

                let ctx = SlotConditionContext {
                    has_queue: bk_has_queue,
                    has_active_construction,
                    tunnel_is_upgrading,
                    has_ready_plate,
                };
                for row in 0..3u8 {
                    for col in 0..4u8 {
                        let hotkey = GRID_HOTKEYS[row as usize][col as usize];
                        if let Some(action) = get_grid_slot_action(&interface_state, row, col, selection.active_type(), &registry, &ctx) {
                            let label = grid_button_label(&interface_state, &action, player_sc, hotkey);
                            let enabled = grid_button_enabled_ext(&action, player_sc, bk_has_queue, panel_target.entity, &building_states, &tunnel_query, tunnel_upgrade_cost, syndicate_supplies, &unit_caps, has_power_plant, syndicate_sc, &selected_owners, has_network_units, &registry);
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
