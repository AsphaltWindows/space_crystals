use bevy::prelude::*;
use crate::types::*;
use crate::game::types::{
    Player, GdoPlayerResources, SyndicatePlayerResources, CultsPlayerResources, StructureInstance,
    DeploymentCenterState, BarracksState, ExtractionFacilityState,
    SupplyTowerState, TunnelState,
    HeadquartersState, RecruitmentCenterState, ArmoryState,
};
use crate::game::entities::EntityRegistry;
use crate::game::units::types::commands::{CommandType, HoldingPosition, UnitCommand, CommandQueue};
use crate::game::units::types::movement::{MoveTarget, Path, Velocity};
use crate::game::combat::types::AttackState;
use super::super::types::*;
use super::helpers::{find_player_resources_mut, find_syndicate_resources_mut, active_group_entities, command_target_entities};
use super::tunnel_actions::{execute_tunnel_upgrade, execute_tunnel_cancel_upgrade, execute_tunnel_select_expansion, execute_tunnel_eject_unit};

/// Execute a command action (shared between button clicks and hotkeys).
/// Thin dispatcher that delegates to category-specific sub-dispatchers.
pub(super) fn execute_command_action(
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
    selected_units: &Query<(Entity, &mut Velocity, Option<&mut CommandQueue>, Option<&AttackState>), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
    shift_held: bool,
    selection: &Selection,
    registry: &EntityRegistry,
) {
    match action {
        // DC build/cancel/placement actions
        CommandButtonAction::DcOpenBuildMenu |
        CommandButtonAction::DcBuild(_) |
        CommandButtonAction::DcCancel => {
            execute_dc_action(action, interface_state, panel_target, dc_query, players, registry);
        }
        // Production queue actions (Barracks, HQ, RC, EF, SupplyTower, Armory)
        CommandButtonAction::BkTrain(_) | CommandButtonAction::BkCancel |
        CommandButtonAction::HqTrain(_) | CommandButtonAction::HqCancel |
        CommandButtonAction::RcCancel |
        CommandButtonAction::EfBuildPlate | CommandButtonAction::EfCancel |
        CommandButtonAction::StTrain(_) | CommandButtonAction::StCancel |
        CommandButtonAction::StScheduleDeliveries |
        CommandButtonAction::ArmoryTrainSoldier | CommandButtonAction::ArmoryTrainGunner |
        CommandButtonAction::ArmoryEjectAll => {
            execute_production_action(action, interface_state, panel_target, bk_hq_query, ef_query, st_query, syndicate_players, players, selection, commands, registry);
        }
        // Tunnel-specific actions
        CommandButtonAction::TunnelUpgrade | CommandButtonAction::TunnelOpenExpandMenu |
        CommandButtonAction::TunnelOpenEjectMenu | CommandButtonAction::TunnelSelectExpansion(_) |
        CommandButtonAction::TunnelCancelUpgrade | CommandButtonAction::TunnelEjectUnit(_) => {
            execute_tunnel_action(action, interface_state, panel_target, tunnel_query, syndicate_players, placement_state, commands);
        }
        // Unit command actions (movement, combat, agent, recruit, rally)
        CommandButtonAction::UnitMove | CommandButtonAction::UnitAttack |
        CommandButtonAction::UnitAttackGround | CommandButtonAction::UnitAttackMove |
        CommandButtonAction::UnitPatrol | CommandButtonAction::UnitHoldPosition |
        CommandButtonAction::UnitStop | CommandButtonAction::UnitReverse |
        CommandButtonAction::UnitEnter | CommandButtonAction::UnitGather |
        CommandButtonAction::ChopperPickUpSupplies | CommandButtonAction::ChopperAttachToTower |
        CommandButtonAction::ChopperDropOffSupplies |
        CommandButtonAction::AgentBuildTunnel | CommandButtonAction::AgentDropOff |
        CommandButtonAction::RecruitConstruct | CommandButtonAction::RecruitSelectBuilding(_) |
        CommandButtonAction::RecruitAssistConstruction |
        CommandButtonAction::SetRallyPoint => {
            execute_unit_command_action(action, interface_state, commands, selection, selected_units, shift_held, placement_state);
        }
        // Navigation/placement mode transitions
        CommandButtonAction::Back | CommandButtonAction::EnterPlacement => {
            execute_navigation_action(action, interface_state);
        }
    }
}

/// Sub-dispatcher for Deployment Center build/cancel/placement actions.
fn execute_dc_action(
    action: &CommandButtonAction,
    interface_state: &mut ResMut<ObjectInterfaceState>,
    panel_target: &Res<CommandPanelTarget>,
    dc_query: &mut Query<(&Owner, &mut DeploymentCenterState)>,
    players: &mut Query<(&Player, &mut GdoPlayerResources)>,
    registry: &EntityRegistry,
) {
    match action {
        CommandButtonAction::DcOpenBuildMenu => {
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
                if *object_type == ObjectEnum::SupplyTower {
                    let has_power_plant = players.iter()
                        .any(|(p, res)| Some(p.player_number) == owner.player_number() && res.has_power_plant);
                    if !has_power_plant {
                        info!("DC: Supply Tower requires a Power Plant");
                        return;
                    }
                }
                if let Some(cost) = registry.cost(object_type) {
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
                    if let Some(cost) = registry.cost(&building_type) {
                        if let Some(refund) = dc_state.cancellation_refund(&cost) {
                            if let Some(mut res) = find_player_resources_mut(owner, players) {
                                res.space_crystals += refund as i32;
                                info!("DC: Cancelled construction, refunded {} SC", refund);
                            }
                        }
                    }
                    dc_state.current_construction = None;
                    dc_state.construction_progress = None;
                    **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
                } else if let Some(ready_type) = &dc_state.ready_to_place.clone() {
                    if let Some(cost) = registry.cost(ready_type) {
                        if let Some(refund) = dc_state.cancellation_refund(&cost) {
                            if let Some(mut res) = find_player_resources_mut(owner, players) {
                                res.space_crystals += refund as i32;
                                info!("DC: Cancelled placement, refunded {} SC (75%)", refund);
                            }
                        }
                    }
                    dc_state.ready_to_place = None;
                    **interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
                }
            }
        }
        _ => {}
    }
}

/// Sub-dispatcher for production queue actions (Barracks, HQ, RC, EF, SupplyTower, Armory).
fn execute_production_action(
    action: &CommandButtonAction,
    interface_state: &mut ResMut<ObjectInterfaceState>,
    panel_target: &Res<CommandPanelTarget>,
    bk_hq_query: &mut Query<(&Owner, Option<&mut BarracksState>, Option<&mut HeadquartersState>, Option<&mut RecruitmentCenterState>, Option<&mut ArmoryState>), Or<(With<BarracksState>, With<HeadquartersState>, With<RecruitmentCenterState>, With<ArmoryState>)>>,
    ef_query: &mut Query<(&Owner, &mut ExtractionFacilityState)>,
    st_query: &mut Query<(&Owner, &mut SupplyTowerState)>,
    syndicate_players: &mut Query<(&Player, &mut SyndicatePlayerResources)>,
    players: &mut Query<(&Player, &mut GdoPlayerResources)>,
    selection: &Selection,
    commands: &mut Commands,
    registry: &EntityRegistry,
) {
    match action {
        CommandButtonAction::BkTrain(unit_type) => {
            let entities = active_group_entities(selection);
            let mut any_queued = false;
            for entity in entities {
                if let Ok((owner, Some(mut bk_state), _, _, _)) = bk_hq_query.get_mut(entity) {
                    if let Some(cost) = registry.cost(unit_type) {
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
                        if let Some(cost) = registry.cost(&cancelled) {
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
                    if let Some(cost) = registry.cost(unit_type) {
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
                        if let Some(cost) = registry.cost(&cancelled) {
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
                if ef_state.current_construction || ef_state.ready_to_place {
                    interface_state.set_changed();
                    return;
                }
                let Some(cost) = registry.cost(&ObjectEnum::ExtractionPlate) else { return };
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
                let ef_cost = registry.cost(&ObjectEnum::ExtractionPlate);
                if let Some(refund) = ef_cost.as_ref().and_then(|c| ef_state.cancellation_refund(c)) {
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
        CommandButtonAction::StTrain(unit_type) => {
            let entities = active_group_entities(selection);
            let mut any_queued = false;
            for entity in entities {
                if let Ok((owner, mut st_state)) = st_query.get_mut(entity) {
                    if let Some(cost) = registry.cost(unit_type) {
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
                        if let Some(cost) = registry.cost(&cancelled) {
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
        CommandButtonAction::ArmoryTrainSoldier => {
            let Some(target_entity) = panel_target.entity else { return };
            if let Ok((owner, _, _, _, Some(mut armory))) = bk_hq_query.get_mut(target_entity) {
                if armory.stored_recruits.is_empty() || armory.training_queue.is_some() { return; }
                let cost = registry.cost(&ObjectEnum::CultsSoldier).map(|c| c.space_crystals as i32).unwrap_or(0);
                let player_number = owner.player_number();
                if let Some(recruit_entity) = armory.stored_recruits.pop() {
                    commands.entity(recruit_entity).despawn();
                }
                armory.training_queue = Some(ObjectEnum::CultsSoldier);
                armory.training_progress = 0;
                interface_state.set_changed();
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
                let cost = registry.cost(&ObjectEnum::CultsGunner).map(|c| c.space_crystals as i32).unwrap_or(0);
                let player_number = owner.player_number();
                if let Some(recruit_entity) = armory.stored_recruits.pop() {
                    commands.entity(recruit_entity).despawn();
                }
                armory.training_queue = Some(ObjectEnum::CultsGunner);
                armory.training_progress = 0;
                interface_state.set_changed();
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
        _ => {}
    }
}

/// Sub-dispatcher for tunnel-specific actions.
fn execute_tunnel_action(
    action: &CommandButtonAction,
    interface_state: &mut ResMut<ObjectInterfaceState>,
    panel_target: &Res<CommandPanelTarget>,
    tunnel_query: &mut Query<(&Owner, &mut TunnelState, &crate::game::types::TunnelArea)>,
    syndicate_players: &mut Query<(&Player, &mut SyndicatePlayerResources)>,
    placement_state: &mut ResMut<PlacementState>,
    commands: &mut Commands,
) {
    match action {
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
        _ => {}
    }
}

/// Sub-dispatcher for unit command actions (movement, combat, agent, recruit, rally).
fn execute_unit_command_action(
    action: &CommandButtonAction,
    interface_state: &mut ResMut<ObjectInterfaceState>,
    commands: &mut Commands,
    selection: &Selection,
    selected_units: &Query<(Entity, &mut Velocity, Option<&mut CommandQueue>, Option<&AttackState>), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
    shift_held: bool,
    placement_state: &mut ResMut<PlacementState>,
) {
    match action {
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
            let target_entities = command_target_entities(action, &selection, selected_units);
            for entity in target_entities {
                if let Ok((_, _, _, Some(attack_state))) = selected_units.get(entity) {
                    if !attack_state.phase.is_interruptible() {
                        continue;
                    }
                }
                let mut entity_cmds = commands.entity(entity);
                if shift_held {
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
                if let Ok((_, _, _, Some(attack_state))) = selected_units.get(entity) {
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
        CommandButtonAction::AgentBuildTunnel => {
            placement_state.building_type = Some(ObjectEnum::Tunnel);
            let agent_entity = selected_units.iter().next().map(|(e, _, _, _)| e);
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
        CommandButtonAction::RecruitConstruct => {
            **interface_state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu);
            info!("Recruit: Open construct menu");
        }
        CommandButtonAction::RecruitSelectBuilding(building_type) => {
            placement_state.building_type = Some(*building_type);
            placement_state.source_entity = selected_units.iter().next().map(|(e, _, _, _)| e);
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
        _ => {}
    }
}

/// Sub-dispatcher for navigation and placement mode transitions (Back, EnterPlacement).
fn execute_navigation_action(
    action: &CommandButtonAction,
    interface_state: &mut ResMut<ObjectInterfaceState>,
) {
    match action {
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
        _ => {}
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
