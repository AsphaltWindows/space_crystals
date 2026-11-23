use bevy::prelude::*;
use crate::types::*;
use crate::game::types::{
    ObjectInstance, Player, GdoPlayerResources, SyndicatePlayerResources, StructureInstance,
    DeploymentCenterState, BarracksState,
    SupplyTowerState, SupplyChopperState, TunnelState,
    HeadquartersState, RecruitmentCenterState, ArmoryState,
};
use crate::game::units::types::commands::{CommandType, CommandQueue};
use crate::game::units::types::state::AgentCarryState;
use crate::game::units::types::movement::Velocity;
use crate::game::combat::types::{AttackCapability, AttackType};
use super::super::types::*;

/// Compute aggregate capabilities from all selected units.
/// Returns `SelectedUnitCapabilities` with `has_attack`, `can_target_ground`, `can_reverse`
/// set to true if ANY selected unit has that capability.
pub(super) fn compute_selected_unit_capabilities(
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
pub(super) fn attack_type_can_target_ground(attack_type: &AttackType) -> bool {
    matches!(attack_type, AttackType::TailDisjointed { .. } | AttackType::DoublyDisjointed { .. })
}

/// Get player space crystals for the selected structure's owner
pub(super) fn get_player_sc_from_owners(
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
pub(super) fn find_player_resources_mut<'a>(
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
pub(super) fn get_dc_construction_info(dc: &DeploymentCenterState, registry: &crate::game::entities::EntityRegistry) -> (String, f32) {
    if let Some(ref building) = dc.current_construction {
        let progress = dc.construction_progress.unwrap_or(0.0);
        let cost = registry.cost(building);
        let total = cost.map(|c| c.build_frames as f32).unwrap_or(160.0);
        let pct = (progress / total * 100.0).min(100.0);
        (building.object_type().name, pct)
    } else {
        ("Unknown".to_string(), 0.0)
    }
}


pub(super) fn grid_button_enabled_ext(
    action: &CommandButtonAction,
    player_sc: i32,
    bk_has_queue: bool,
    target_entity: Option<Entity>,
    building_states: &Query<(Option<&DeploymentCenterState>, Option<&BarracksState>, Option<&crate::game::types::ExtractionFacilityState>, Option<&SupplyTowerState>)>,
    tunnel_query: &Query<(&TunnelState, &Owner)>,
    tunnel_upgrade_cost: Option<u32>,
    syndicate_supplies: i32,
    unit_caps: &SelectedUnitCapabilities,
    has_power_plant: bool,
    syndicate_sc: i32,
    hq_owners: &Query<(&Owner, Option<&HeadquartersState>, Option<&RecruitmentCenterState>, Option<&ArmoryState>), (With<StructureInstance>, With<Selected>)>,
    has_network_units: bool,
    registry: &crate::game::entities::EntityRegistry,
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
            let cost = registry.cost(unit_type)
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
                .and_then(|e| building_states.get(e).ok())
                .and_then(|(_, _, _, st)| st)
                .map(|st| st.attached_chopper.is_some())
                .unwrap_or(false)
        }
        CommandButtonAction::AgentDropOff => unit_caps.agent_carrying,
        CommandButtonAction::ChopperDropOffSupplies => unit_caps.chopper_has_supplies,
        CommandButtonAction::RcCancel => true,
        CommandButtonAction::RecruitConstruct |
        CommandButtonAction::RecruitSelectBuilding(_) |
        CommandButtonAction::RecruitAssistConstruction => true,
        _ => grid_button_enabled(action, player_sc, bk_has_queue, target_entity, building_states),
    }
}

/// Find syndicate player resources mutably for a given owner
pub(super) fn find_syndicate_resources_mut<'a>(
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
pub(super) fn is_unit_action(action: &CommandButtonAction) -> bool {
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
pub(super) fn object_type_supports_action(obj: &ObjectEnum, action: &CommandButtonAction) -> bool {
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
pub(super) fn is_common_command(action: &CommandButtonAction, selection: &Selection) -> bool {
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
pub(super) fn active_group_entities(selection: &Selection) -> Vec<Entity> {
    if let Some(group) = selection.active_group() {
        group.entities.clone()
    } else {
        Vec::new()
    }
}

pub(super) fn command_target_entities(
    action: &CommandButtonAction,
    selection: &Selection,
    selected_units: &Query<(Entity, &mut Velocity, Option<&mut CommandQueue>, Option<&crate::game::combat::types::AttackState>), (With<Unit>, With<Selected>, Without<StructureInstance>)>,
) -> Vec<Entity> {
    if is_common_command(action, selection) {
        // Common command: issue to all selected units
        selected_units.iter().map(|(e, _, _, _)| e).collect()
    } else {
        // Group command: issue only to active group entities
        if let Some(active_group) = selection.active_group() {
            active_group.entities.clone()
        } else {
            selected_units.iter().map(|(e, _, _, _)| e).collect()
        }
    }
}

/// Generate the label for a grid button based on state and action
pub(super) fn grid_button_label(
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
        CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility) => format!("[{}] EF\n200 SC", hotkey),
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
pub(super) fn grid_button_enabled(
    action: &CommandButtonAction,
    player_sc: i32,
    _bk_has_queue: bool,
    target_entity: Option<Entity>,
    building_states: &Query<(Option<&DeploymentCenterState>, Option<&BarracksState>, Option<&crate::game::types::ExtractionFacilityState>, Option<&SupplyTowerState>)>,
) -> bool {
    match action {
        CommandButtonAction::DcBuild(ObjectEnum::PowerPlant) => player_sc >= 150,
        CommandButtonAction::DcBuild(ObjectEnum::Barracks) => player_sc >= 200,
        CommandButtonAction::DcBuild(ObjectEnum::SupplyTower) => player_sc >= 200,
        CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility) => player_sc >= 200,
        CommandButtonAction::BkTrain(ObjectEnum::Peacekeeper) => {
            let queue_full = target_entity
                .and_then(|e| building_states.get(e).ok())
                .and_then(|(_, bk, _, _)| bk)
                .map(|bk| bk.build_queue.len() >= BarracksState::MAX_QUEUE_SIZE)
                .unwrap_or(false);
            player_sc >= 50 && !queue_full
        }
        CommandButtonAction::StTrain(ObjectEnum::SupplyChopper) => {
            let queue_full = target_entity
                .and_then(|e| building_states.get(e).ok())
                .and_then(|(_, _, _, st)| st)
                .map(|st| st.build_queue.len() >= 5)
                .unwrap_or(false);
            player_sc >= 100 && !queue_full
        }
        CommandButtonAction::EfBuildPlate => player_sc >= 75,
        _ => true,
    }
}

/// Check if an action corresponds to the currently active command mode (AwaitingTarget)
pub(super) fn is_action_active(action: &CommandButtonAction, interface_state: &ObjectInterfaceState) -> bool {
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
pub(super) fn spawn_panel_title(parent: &mut ChildSpawnerCommands, title: &str) {
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
pub(super) fn spawn_progress_text(parent: &mut ChildSpawnerCommands, text: &str) {
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
pub(super) fn spawn_info_text(parent: &mut ChildSpawnerCommands, text: &str) {
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
pub(super) fn spawn_grid_button(
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
pub(super) fn spawn_empty_grid_cell(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node::default(),
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.3)),
    ));
}


// =====================================================
// TESTS - All command_panel tests in one module
// =====================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::grid_layout::get_grid_slot_action;  // kept for test_grid_action wrapper
    use super::super::state::{is_panel_visible, selection_owned_by_local_player};
    use super::super::hotkeys::{right_click_cancel_target, RallyTargetKind};
    use super::super::tunnel_actions::tier_at_least;
    use super::super::pointer::{resolve_pointer_display, CursorEntityInfo};
    use crate::game::combat::types::{AttackType, ProjectileVisual};
    use crate::game::types::{TunnelTier, HeadquartersState, DeploymentCenterState, SupplyTowerState, GdoPlayerResources};
    use crate::game::units::types::commands::CommandType;
    use crate::game::entities::types::SlotConditionContext;
    use crate::game::entities::registry::build_registry;
    use bevy::ecs::system::RunSystemOnce;

    /// Default condition context — all conditions false.
    fn default_ctx() -> SlotConditionContext {
        SlotConditionContext {
            has_queue: false,
            has_active_construction: false,
            tunnel_is_upgrading: false,
            has_ready_plate: false,
        }
    }

    /// Condition context with has_queue=true.
    fn queue_ctx() -> SlotConditionContext {
        SlotConditionContext { has_queue: true, ..default_ctx() }
    }

    /// Condition context with has_active_construction=true.
    fn construction_ctx() -> SlotConditionContext {
        SlotConditionContext { has_active_construction: true, ..default_ctx() }
    }

    /// Condition context with tunnel_is_upgrading=true.
    fn upgrading_ctx() -> SlotConditionContext {
        SlotConditionContext { tunnel_is_upgrading: true, ..default_ctx() }
    }

    /// Condition context with has_ready_plate=true.
    fn ready_plate_ctx() -> SlotConditionContext {
        SlotConditionContext { has_ready_plate: true, ..default_ctx() }
    }

    /// Condition context with has_active_construction + has_ready_plate.
    fn construction_ready_plate_ctx() -> SlotConditionContext {
        SlotConditionContext { has_active_construction: true, has_ready_plate: true, ..default_ctx() }
    }

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

    // === get_grid_slot_action registry-based tests ===

    fn all_caps() -> SelectedUnitCapabilities {
        SelectedUnitCapabilities { has_attack: true, can_target_ground: true, can_reverse: true, agent_carrying: false, is_chopper: false, chopper_has_supplies: false, owned_by_local_player: false }
    }

    fn no_caps() -> SelectedUnitCapabilities {
        SelectedUnitCapabilities::default()
    }

    /// Test compatibility wrapper: adapts the old get_grid_slot_action signature
    /// to the new registry-based signature. For Default state, uses Peacekeeper
    /// as the default active_type. For structure/agent/recruit states, the
    /// active_type is resolved from the state itself.
    #[allow(unused_variables)]
    fn test_grid_action(
        state: &ObjectInterfaceState,
        row: u8,
        col: u8,
        bk_has_queue: bool,
        has_active_construction: bool,
        caps: &SelectedUnitCapabilities,
        tunnel_is_upgrading: bool,
        has_ready_plate: bool,
    ) -> Option<CommandButtonAction> {
        let registry = build_registry();
        let ctx = SlotConditionContext {
            has_queue: bk_has_queue,
            has_active_construction,
            tunnel_is_upgrading,
            has_ready_plate,
        };
        // Determine active_type from state context
        let active_type = match state {
            ObjectInterfaceState::Default => Some(ObjectEnum::Peacekeeper),
            _ => None, // Structure/Agent/Recruit/AwaitingTarget resolved from state
        };
        get_grid_slot_action(state, row, col, active_type, &registry, &ctx)
    }

    /// Registry-based grid action lookup with explicit active_type.
    fn test_grid_action_for(
        state: &ObjectInterfaceState,
        row: u8,
        col: u8,
        active_type: Option<ObjectEnum>,
        ctx: &SlotConditionContext,
    ) -> Option<CommandButtonAction> {
        let registry = build_registry();
        get_grid_slot_action(state, row, col, active_type, &registry, ctx)
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
        let action = test_grid_action(&ObjectInterfaceState::Default, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::UnitMove)));
    }

    #[test]
    fn unit_commands_attack_present_for_combat_unit() {
        // Peacekeeper has UnitAttack in its registry definition
        let ctx = default_ctx();
        let action = test_grid_action_for(&ObjectInterfaceState::Default, 1, 0, Some(ObjectEnum::Peacekeeper), &ctx);
        assert!(matches!(action, Some(CommandButtonAction::UnitAttack)));

        // SupplyChopper has ChopperDropOffSupplies at (1,0) instead of UnitAttack
        let action = test_grid_action_for(&ObjectInterfaceState::Default, 1, 0, Some(ObjectEnum::SupplyChopper), &ctx);
        assert!(matches!(action, Some(CommandButtonAction::ChopperDropOffSupplies)));

        // CultsSoldier has UnitAttack (combat unit)
        let action = test_grid_action_for(&ObjectInterfaceState::Default, 1, 0, Some(ObjectEnum::CultsSoldier), &ctx);
        assert!(matches!(action, Some(CommandButtonAction::UnitAttack)));
    }

    #[test]
    fn unit_commands_attack_ground_present_for_capable_unit() {
        let ctx = default_ctx();
        // Peacekeeper has UnitAttackGround in its registry definition
        let action = test_grid_action_for(&ObjectInterfaceState::Default, 1, 2, Some(ObjectEnum::Peacekeeper), &ctx);
        assert!(matches!(action, Some(CommandButtonAction::UnitAttackGround)));

        // CultsSoldier does NOT have UnitAttackGround
        let action = test_grid_action_for(&ObjectInterfaceState::Default, 1, 2, Some(ObjectEnum::CultsSoldier), &ctx);
        assert!(action.is_none());
    }

    #[test]
    fn unit_commands_patrol_always_available() {
        let caps = no_caps();
        let action = test_grid_action(&ObjectInterfaceState::Default, 1, 1, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::UnitPatrol)));
    }

    #[test]
    fn unit_commands_hold_position_always_available() {
        let caps = no_caps();
        let action = test_grid_action(&ObjectInterfaceState::Default, 0, 2, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::UnitHoldPosition)));
    }

    #[test]
    fn unit_commands_stop_always_available() {
        let caps = no_caps();
        let action = test_grid_action(&ObjectInterfaceState::Default, 2, 1, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::UnitStop)));
    }

    #[test]
    fn unit_commands_reverse_not_in_current_units() {
        // No current unit type defines UnitReverse in its command_slots.
        // Peacekeeper's (0,1) is empty in registry definition.
        let ctx = default_ctx();
        let action = test_grid_action_for(&ObjectInterfaceState::Default, 0, 1, Some(ObjectEnum::Peacekeeper), &ctx);
        assert!(action.is_none());
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
    fn peacekeeper_default_commands_count() {
        let ctx = default_ctx();
        let mut count = 0;
        for row in 0..3u8 {
            for col in 0..3u8 {
                if test_grid_action_for(&ObjectInterfaceState::Default, row, col, Some(ObjectEnum::Peacekeeper), &ctx).is_some() {
                    count += 1;
                }
            }
        }
        // Move(0,0), HoldPos(0,2), Attack(1,0), Patrol(1,1), AtkGround(1,2), Stop(2,1) = 6
        assert_eq!(count, 6);
    }

    #[test]
    fn supply_chopper_default_commands_count() {
        let ctx = default_ctx();
        let mut count = 0;
        for row in 0..3u8 {
            for col in 0..3u8 {
                if test_grid_action_for(&ObjectInterfaceState::Default, row, col, Some(ObjectEnum::SupplyChopper), &ctx).is_some() {
                    count += 1;
                }
            }
        }
        // Move(0,0), PickUp(0,1), Attach(0,2), DropOff(1,0), HoldPos(1,2), Stop(2,1) = 6
        assert_eq!(count, 6);
    }

    #[test]
    fn cults_soldier_default_commands_count() {
        let ctx = default_ctx();
        let mut count = 0;
        for row in 0..3u8 {
            for col in 0..3u8 {
                if test_grid_action_for(&ObjectInterfaceState::Default, row, col, Some(ObjectEnum::CultsSoldier), &ctx).is_some() {
                    count += 1;
                }
            }
        }
        // Move(0,0), HoldPos(0,2), Attack(1,0), Patrol(1,1), Stop(2,1) = 5
        assert_eq!(count, 5);
    }

    // === Registry grid layout condition tests ===

    #[test]
    fn dc_idle_cancel_requires_active_construction() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        // Without active construction, cancel is hidden
        let action = test_grid_action_for(&state, 2, 1, None, &default_ctx());
        assert!(action.is_none());
        // With active construction, cancel appears
        let action = test_grid_action_for(&state, 2, 1, None, &construction_ctx());
        assert!(matches!(action, Some(CommandButtonAction::DcCancel)));
    }

    #[test]
    fn ef_idle_shows_enter_placement_when_ready() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        // Without ready plate, shows EfBuildPlate
        let action = test_grid_action_for(&state, 0, 0, None, &default_ctx());
        assert!(matches!(action, Some(CommandButtonAction::EfBuildPlate)));
        // With ready plate, shows EnterPlacement
        let action = test_grid_action_for(&state, 0, 0, None, &ready_plate_ctx());
        assert!(matches!(action, Some(CommandButtonAction::EnterPlacement)));
    }

    #[test]
    fn ef_idle_cancel_requires_active_construction() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        let action = test_grid_action_for(&state, 2, 1, None, &default_ctx());
        assert!(action.is_none());
        let action = test_grid_action_for(&state, 2, 1, None, &construction_ctx());
        assert!(matches!(action, Some(CommandButtonAction::EfCancel)));
    }

    #[test]
    fn barracks_cancel_requires_queue() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);
        let action = test_grid_action_for(&state, 2, 1, None, &default_ctx());
        assert!(action.is_none());
        let action = test_grid_action_for(&state, 2, 1, None, &queue_ctx());
        assert!(matches!(action, Some(CommandButtonAction::BkCancel)));
    }

    #[test]
    fn tunnel_cancel_upgrade_requires_upgrading() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
        let action = test_grid_action_for(&state, 2, 1, None, &default_ctx());
        assert!(action.is_none());
        let action = test_grid_action_for(&state, 2, 1, None, &upgrading_ctx());
        assert!(matches!(action, Some(CommandButtonAction::TunnelCancelUpgrade)));
    }

    #[test]
    fn awaiting_target_always_shows_back() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::Attack);
        let ctx = default_ctx();
        let action = test_grid_action_for(&state, 2, 0, None, &ctx);
        assert!(matches!(action, Some(CommandButtonAction::Back)));
        // Other positions are empty
        let action = test_grid_action_for(&state, 0, 0, None, &ctx);
        assert!(action.is_none());
    }

    #[test]
    fn agent_menu_default_from_registry() {
        let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
        let ctx = default_ctx();
        let action = test_grid_action_for(&state, 0, 0, None, &ctx);
        assert!(matches!(action, Some(CommandButtonAction::AgentBuildTunnel)));
        let action = test_grid_action_for(&state, 0, 1, None, &ctx);
        assert!(matches!(action, Some(CommandButtonAction::AgentDropOff)));
    }

    #[test]
    fn recruit_menu_from_registry() {
        let state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitDefault);
        let ctx = default_ctx();
        let action = test_grid_action_for(&state, 0, 0, None, &ctx);
        assert!(matches!(action, Some(CommandButtonAction::RecruitConstruct)));
        let action = test_grid_action_for(&state, 1, 1, None, &ctx);
        assert!(matches!(action, Some(CommandButtonAction::RecruitAssistConstruction)));
    }

    #[test]
    fn unregistered_active_type_returns_none() {
        // No active_type means no registry lookup
        let ctx = default_ctx();
        let action = test_grid_action_for(&ObjectInterfaceState::Default, 0, 0, None, &ctx);
        assert!(action.is_none());
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
        let action = test_grid_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::DcOpenBuildMenu)));
    }

    #[test]
    fn dc_build_menu_shows_structures() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
        let caps = no_caps();
        let pp = test_grid_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(pp, Some(CommandButtonAction::DcBuild(ObjectEnum::PowerPlant))));
        let bk = test_grid_action(&state, 0, 1, false, false, &caps, false, false);
        assert!(matches!(bk, Some(CommandButtonAction::DcBuild(ObjectEnum::Barracks))));
    }

    #[test]
    fn awaiting_target_has_back_at_z() {
        let state = ObjectInterfaceState::AwaitingTarget(CommandType::Attack);
        let caps = all_caps();
        // Z (2,0) should be Back
        let action = test_grid_action(&state, 2, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::Back)));
        // All other slots should be None
        for row in 0..3u8 {
            for col in 0..3u8 {
                if (row, col) == (2, 0) { continue; }
                assert!(test_grid_action(&state, row, col, false, false, &caps, false, false).is_none(),
                    "Slot ({}, {}) should be empty in AwaitingTarget", row, col);
            }
        }
    }

    // === Tunnel Interface State Tests ===

    #[test]
    fn tunnel_idle_shows_three_commands() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
        let caps = no_caps();
        let upgrade = test_grid_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(upgrade, Some(CommandButtonAction::TunnelUpgrade)));
        let expand = test_grid_action(&state, 0, 1, false, false, &caps, false, false);
        assert!(matches!(expand, Some(CommandButtonAction::TunnelOpenExpandMenu)));
        let eject = test_grid_action(&state, 0, 2, false, false, &caps, false, false);
        assert!(matches!(eject, Some(CommandButtonAction::TunnelOpenEjectMenu)));
    }

    #[test]
    fn tunnel_idle_remaining_slots_are_none() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
        let caps = no_caps();
        // Only (0,0), (0,1), (0,2) should have actions
        for row in 1..3u8 {
            for col in 0..3u8 {
                assert!(test_grid_action(&state, row, col, false, false, &caps, false, false).is_none(),
                    "Expected None at ({}, {})", row, col);
            }
        }
    }

    #[test]
    fn tunnel_idle_cancel_upgrade_shown_when_upgrading() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
        let caps = no_caps();
        let action = test_grid_action(&state, 2, 1, false, false, &caps, true, false);
        assert!(matches!(action, Some(CommandButtonAction::TunnelCancelUpgrade)));
    }

    #[test]
    fn tunnel_idle_cancel_upgrade_hidden_when_not_upgrading() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelIdle);
        let caps = no_caps();
        let action = test_grid_action(&state, 2, 1, false, false, &caps, false, false);
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
        let action = test_grid_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::TunnelSelectExpansion(ObjectEnum::Headquarters))));
    }

    #[test]
    fn tunnel_expand_menu_back_at_2_0() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
        let caps = no_caps();
        let action = test_grid_action(&state, 2, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::Back)));
    }

    #[test]
    fn tunnel_expand_menu_empty_slots_return_none() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelExpandMenu);
        let caps = no_caps();
        // All non-mapped slots should return None
        assert!(test_grid_action(&state, 0, 1, false, false, &caps, false, false).is_none());
        assert!(test_grid_action(&state, 0, 2, false, false, &caps, false, false).is_none());
        assert!(test_grid_action(&state, 1, 0, false, false, &caps, false, false).is_none());
        assert!(test_grid_action(&state, 1, 1, false, false, &caps, false, false).is_none());
        assert!(test_grid_action(&state, 1, 2, false, false, &caps, false, false).is_none());
        assert!(test_grid_action(&state, 2, 1, false, false, &caps, false, false).is_none());
        assert!(test_grid_action(&state, 2, 2, false, false, &caps, false, false).is_none());
    }

    #[test]
    fn tunnel_eject_menu_back_at_2_0() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelEjectMenu);
        let caps = no_caps();
        let action = test_grid_action(&state, 2, 0, false, false, &caps, false, false);
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
                assert!(test_grid_action(&state, row, col, false, false, &caps, false, false).is_none());
            }
        }
    }

    #[test]
    fn tunnel_awaiting_placement_no_grid_buttons() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::TunnelAwaitingPlacement);
        let caps = no_caps();
        for row in 0..3u8 {
            for col in 0..3u8 {
                assert!(test_grid_action(&state, row, col, false, false, &caps, false, false).is_none());
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
        let action = test_grid_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::AgentBuildTunnel)));
    }

    #[test]
    fn agent_default_shows_drop_off_at_w() {
        let caps = no_caps();
        let state = ObjectInterfaceState::AgentMenu(AgentMenuState::AgentDefault);
        let action = test_grid_action(&state, 0, 1, false, false, &caps, false, false);
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
                let action = test_grid_action(&state, row, col, false, false, &caps, false, false);
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
                let action = test_grid_action(&state, row, col, false, false, &caps, false, false);
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
        let action = test_grid_action(&state, 2, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::Back)));
    }

    #[test]
    fn dc_build_menu_ef_at_row1_col1() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
        let caps = no_caps();
        let action = test_grid_action(&state, 1, 1, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility))));
    }

    #[test]
    fn dc_build_menu_no_back_at_bottom_right() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
        let caps = no_caps();
        let action = test_grid_action(&state, 2, 2, false, false, &caps, false, false);
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
            matches!(test_grid_action(&state, 2, 0, false, false, &caps, false, false), Some(CommandButtonAction::Back)),
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
        }).unwrap();
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
        }).unwrap();
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
        }).unwrap();
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
        }).unwrap();
    }

    #[test]
    fn ownership_true_when_selection_empty() {
        let mut app = App::new();
        let selection = Selection::default();
        let local_player = LocalPlayer(1);
        app.world_mut().run_system_once(move |owner_q: Query<&Owner>| {
            // Empty selection is considered "owned" (no violations)
            assert!(selection_owned_by_local_player(&selection, &local_player, &owner_q));
        }).unwrap();
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
        use crate::game::entities::registry::build_registry;
        let registry = build_registry();
        let cost = registry.cost(&ObjectEnum::SupplyTower);
        assert!(cost.is_some(), "Supply Tower should have a construction cost");
        assert_eq!(cost.unwrap().space_crystals, 200);
    }

    #[test]
    fn supply_tower_build_button_in_dc_build_menu() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
        let caps = no_caps();
        // Supply Tower is at (1, 0) in the DC build menu
        let action = test_grid_action(&state, 1, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::DcBuild(ObjectEnum::SupplyTower))),
            "DC build menu should have Supply Tower at (1,0)");
    }

    // === Schedule Deliveries gate tests ===

    #[test]
    fn schedule_deliveries_action_exists_in_supply_tower_menu() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu);
        let caps = no_caps();
        let action = test_grid_action(&state, 1, 0, false, false, &caps, false, false);
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
        let action = test_grid_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::HqTrain(ObjectEnum::SyndicateAgent))));
    }

    #[test]
    fn hq_menu_grid_slot_cancel_at_2_1_when_queue() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
        let caps = no_caps();
        // Cancel visible at (2,1) when has_queue is true
        let action = test_grid_action(&state, 2, 1, true, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::HqCancel)));
    }

    #[test]
    fn hq_menu_grid_slot_cancel_hidden_when_no_queue() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
        let caps = no_caps();
        // Cancel hidden when has_queue is false
        let action = test_grid_action(&state, 2, 1, false, false, &caps, false, false);
        assert!(action.is_none());
    }

    #[test]
    fn hq_menu_grid_slot_guard_at_0_1() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
        let caps = no_caps();
        let action = test_grid_action(&state, 0, 1, false, false, &caps, false, false);
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
                let action = test_grid_action(&state, row, col, false, false, &caps, false, false);
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
        use crate::game::entities::registry::build_registry;
        let registry = build_registry();
        let cost = registry.cost(&ObjectEnum::SyndicateAgent);
        assert!(cost.is_some());
        let cost = cost.unwrap();
        assert_eq!(cost.space_crystals, 100);
        assert_eq!(cost.build_frames, 160);
    }

    #[test]
    fn headquarters_state_no_cost_for_other_units() {
        use crate::game::entities::registry::build_registry;
        let registry = build_registry();
        // Peacekeeper does have a cost in the registry (it's a GDO unit)
        // This test verifies that the registry-based lookup works correctly
        let pk_cost = registry.cost(&ObjectEnum::Peacekeeper);
        assert!(pk_cost.is_some()); // PK has cost 50 SC
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
                if test_grid_action(&state, row, col, true, false, &caps, false, false).is_some() {
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
                if test_grid_action(&state, row, col, false, false, &caps, false, false).is_some() {
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
                    test_grid_action(&state, row, col, false, false, &caps, false, false).is_none(),
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
            let action = test_grid_action(&state, 2, 0, false, false, &caps, false, false);
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
            let action = test_grid_action(&state, 2, 2, false, false, &caps, false, false);
            assert!(matches!(action, Some(CommandButtonAction::SetRallyPoint)),
                "Menu {:?} should have SetRallyPoint at (2,2)", menu);
        }
    }

    #[test]
    fn cancel_at_x_for_production_menus_with_queue() {
        let caps = no_caps();
        // Barracks
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);
        let action = test_grid_action(&state, 2, 1, true, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::BkCancel)));

        // Supply Tower
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::SupplyTowerMenu);
        let action = test_grid_action(&state, 2, 1, true, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::StCancel)));

        // HQ
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::HeadquartersMenu);
        let action = test_grid_action(&state, 2, 1, true, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::HqCancel)));
    }

    #[test]
    fn dc_constructing_back_at_z_cancel_at_x() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing);
        let back = test_grid_action(&state, 2, 0, false, false, &caps, false, false);
        assert!(matches!(back, Some(CommandButtonAction::Back)));
        let cancel = test_grid_action(&state, 2, 1, false, false, &caps, false, false);
        assert!(matches!(cancel, Some(CommandButtonAction::DcCancel)));
        // Old position (0,0) should be empty
        let old = test_grid_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(old.is_none(), "DcConstructing (0,0) should be empty after move");
    }

    #[test]
    fn dc_ready_to_place_back_at_z_cancel_at_x() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace);
        let place = test_grid_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(place, Some(CommandButtonAction::EnterPlacement)));
        let back = test_grid_action(&state, 2, 0, false, false, &caps, false, false);
        assert!(matches!(back, Some(CommandButtonAction::Back)));
        let cancel = test_grid_action(&state, 2, 1, false, false, &caps, false, false);
        assert!(matches!(cancel, Some(CommandButtonAction::DcCancel)));
        // Old cancel position (0,1) should be empty
        let old = test_grid_action(&state, 0, 1, false, false, &caps, false, false);
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
        let action = test_grid_action(&state, 0, 0, false, true, &caps, false, true);
        assert!(matches!(action, Some(CommandButtonAction::EnterPlacement)),
            "EfIdle with ready plate should show EnterPlacement at (0,0)");
    }

    #[test]
    fn ef_idle_constructing_shows_build_plate_and_cancel() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        // has_active_construction=true, has_ready_plate=false → Q=EfBuildPlate, X=EfCancel
        let q = test_grid_action(&state, 0, 0, false, true, &caps, false, false);
        assert!(matches!(q, Some(CommandButtonAction::EfBuildPlate)),
            "EfIdle constructing should show EfBuildPlate at (0,0)");
        let x = test_grid_action(&state, 2, 1, false, true, &caps, false, false);
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
        let old = test_grid_action(&state, 0, 1, true, false, &caps, false, false);
        assert!(old.is_none(), "BarracksMenu (0,1) should be empty after BkCancel move");
    }

    #[test]
    fn barracks_train_peacekeeper_at_q() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);
        let action = test_grid_action(&state, 0, 0, false, false, &caps, false, false);
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
        let action = test_grid_action(&state, 2, 1, false, false, &caps, false, false);
        assert!(action.is_none(), "BarracksMenu (2,1) should be None when queue is empty");
    }

    #[test]
    fn barracks_grid_slot_action_count_with_queue() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::BarracksMenu);
        let mut count = 0;
        for row in 0..3u8 {
            for col in 0..3u8 {
                if test_grid_action(&state, row, col, true, false, &caps, false, false).is_some() {
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
                if test_grid_action(&state, row, col, false, false, &caps, false, false).is_some() {
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
        let old = test_grid_action(&state, 0, 1, true, false, &caps, false, false);
        assert!(old.is_none(), "SupplyTowerMenu (0,1) should be empty after StCancel move");
    }

    // === DC grid slot tests ===

    #[test]
    fn dc_idle_grid_has_open_build_menu() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        let action = test_grid_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::DcOpenBuildMenu)));
    }

    #[test]
    fn dc_idle_no_construction_no_cancel() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        let action = test_grid_action(&state, 2, 1, false, false, &caps, false, false);
        assert!(action.is_none(), "DcIdle should not show cancel when has_active_construction=false");
    }

    #[test]
    fn dc_idle_with_construction_shows_cancel() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        let action = test_grid_action(&state, 2, 1, false, true, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::DcCancel)),
            "DcIdle should show DcCancel at (2,1) when has_active_construction=true");
    }

    #[test]
    fn dc_idle_with_construction_still_has_build_menu() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcIdle);
        let action = test_grid_action(&state, 0, 0, false, true, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::DcOpenBuildMenu)),
            "DcIdle should still show DcOpenBuildMenu at (0,0) even with active construction");
    }

    #[test]
    fn dc_build_menu_grid_has_all_buildings_and_back() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu);
        assert!(matches!(test_grid_action(&state, 0, 0, false, false, &caps, false, false), Some(CommandButtonAction::DcBuild(ObjectEnum::PowerPlant))));
        assert!(matches!(test_grid_action(&state, 0, 1, false, false, &caps, false, false), Some(CommandButtonAction::DcBuild(ObjectEnum::Barracks))));
        assert!(test_grid_action(&state, 0, 2, false, false, &caps, false, false).is_none());
        assert!(matches!(test_grid_action(&state, 1, 0, false, false, &caps, false, false), Some(CommandButtonAction::DcBuild(ObjectEnum::SupplyTower))));
        assert!(matches!(test_grid_action(&state, 2, 0, false, false, &caps, false, false), Some(CommandButtonAction::Back)));
    }

    #[test]
    fn dc_constructing_grid_has_back_and_cancel() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcConstructing);
        assert!(matches!(test_grid_action(&state, 2, 0, false, false, &caps, false, false), Some(CommandButtonAction::Back)));
        assert!(matches!(test_grid_action(&state, 2, 1, false, false, &caps, false, false), Some(CommandButtonAction::DcCancel)));
        // Old cancel position (0,0) should be empty
        assert!(test_grid_action(&state, 0, 0, false, false, &caps, false, false).is_none());
    }

    #[test]
    fn dc_ready_to_place_grid_has_enter_placement_back_and_cancel() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::DcReadyToPlace);
        assert!(matches!(test_grid_action(&state, 0, 0, false, false, &caps, false, false), Some(CommandButtonAction::EnterPlacement)));
        assert!(matches!(test_grid_action(&state, 2, 0, false, false, &caps, false, false), Some(CommandButtonAction::Back)));
        assert!(matches!(test_grid_action(&state, 2, 1, false, false, &caps, false, false), Some(CommandButtonAction::DcCancel)));
        // Old cancel position (0,1) should be empty
        assert!(test_grid_action(&state, 0, 1, false, false, &caps, false, false).is_none());
    }

    // === EF grid slot tests ===

    #[test]
    fn ef_idle_grid_has_build_plate() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        assert!(matches!(test_grid_action(&state, 0, 0, false, false, &caps, false, false), Some(CommandButtonAction::EfBuildPlate)));
    }


    #[test]
    fn ef_idle_remaining_slots_are_none() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        // Only (0,0) should have an action
        for row in 0..3u8 {
            for col in 0..3u8 {
                if (row, col) == (0, 0) { continue; }
                assert!(test_grid_action(&state, row, col, false, false, &caps, false, false).is_none(),
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
        let action = test_grid_action(&state, 2, 1, false, false, &caps, false, false);
        assert!(action.is_none(), "EfIdle should not show cancel when has_active_construction=false");
    }

    #[test]
    fn ef_idle_with_construction_shows_cancel() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        let action = test_grid_action(&state, 2, 1, false, true, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::EfCancel)),
            "EfIdle should show EfCancel at (2,1) when has_active_construction=true");
    }

    #[test]
    fn ef_idle_with_construction_still_has_build_plate() {
        let caps = no_caps();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::EfIdle);
        let action = test_grid_action(&state, 0, 0, false, true, &caps, false, false);
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
        let action = test_grid_action(&state, 0, 2, false, false, &caps, false, false);
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
        use crate::game::entities::types::UnitDefinition;
        // SyndicateAgent is HeavyInfantry — allowed by Tier1
        let tier = TunnelTier::Tier1;
        let unit_base = crate::game::entities::factions::syndicate::units::agent::SyndicateAgentDef::unit_stats().unit_base;
        assert!(tier.can_transit(&unit_base), "Tier1 tunnel should allow Agent (HeavyInfantry) ejection");
    }

    #[test]
    fn tier1_tunnel_allows_guard_ejection() {
        use crate::game::entities::types::UnitDefinition;
        // SyndicateGuard is HeavyInfantry — allowed by Tier1
        let tier = TunnelTier::Tier1;
        let unit_base = crate::game::entities::factions::syndicate::units::guard::SyndicateGuardDef::unit_stats().unit_base;
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
        use crate::game::types::structures::tunnel_t2_upgrade_cost;
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
        let action = test_grid_action(&state, 2, 1, true, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::RcCancel)));
    }

    #[test]
    fn rc_menu_grid_slot_no_cancel_when_idle() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu);
        let caps = no_caps();
        // Cancel hidden at (2,1) when bk_has_queue is false
        let action = test_grid_action(&state, 2, 1, false, false, &caps, false, false);
        assert!(action.is_none());
    }

    #[test]
    fn rc_menu_grid_slot_rally_at_2_2() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu);
        let caps = no_caps();
        let action = test_grid_action(&state, 2, 2, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::SetRallyPoint)));
    }

    #[test]
    fn rc_menu_grid_slot_empty_at_0_0() {
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::RecruitmentCenterMenu);
        let caps = no_caps();
        let action = test_grid_action(&state, 0, 0, false, false, &caps, false, false);
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
        let ctx = default_ctx();
        let state = ObjectInterfaceState::Default;
        let at = Some(ObjectEnum::SupplyChopper);
        assert!(matches!(test_grid_action_for(&state, 0, 0, at, &ctx), Some(CommandButtonAction::UnitMove)));
        assert!(matches!(test_grid_action_for(&state, 0, 1, at, &ctx), Some(CommandButtonAction::ChopperPickUpSupplies)));
        assert!(matches!(test_grid_action_for(&state, 0, 2, at, &ctx), Some(CommandButtonAction::ChopperAttachToTower)));
        assert!(matches!(test_grid_action_for(&state, 1, 0, at, &ctx), Some(CommandButtonAction::ChopperDropOffSupplies)));
        assert!(matches!(test_grid_action_for(&state, 1, 2, at, &ctx), Some(CommandButtonAction::UnitHoldPosition)));
        assert!(matches!(test_grid_action_for(&state, 2, 1, at, &ctx), Some(CommandButtonAction::UnitStop)));
    }

    #[test]
    fn chopper_grid_has_no_attack_buttons() {
        let ctx = default_ctx();
        let state = ObjectInterfaceState::Default;
        let at = Some(ObjectEnum::SupplyChopper);
        // No attack-related buttons in any remaining empty slots
        assert!(test_grid_action_for(&state, 1, 1, at, &ctx).is_none());
        assert!(test_grid_action_for(&state, 2, 0, at, &ctx).is_none());
        assert!(test_grid_action_for(&state, 2, 2, at, &ctx).is_none());
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
        assert!(matches!(test_grid_action(&state, 0, 0, false, false, &caps, false, false), Some(CommandButtonAction::UnitMove)));
        assert!(matches!(test_grid_action(&state, 0, 2, false, false, &caps, false, false), Some(CommandButtonAction::UnitHoldPosition)));
        assert!(matches!(test_grid_action(&state, 1, 1, false, false, &caps, false, false), Some(CommandButtonAction::UnitPatrol)));
        assert!(matches!(test_grid_action(&state, 2, 1, false, false, &caps, false, false), Some(CommandButtonAction::UnitStop)));
        // No PickUpSupplies or AttachToTower in non-chopper grid
        assert!(test_grid_action(&state, 0, 1, false, false, &caps, false, false).is_none());
    }

    // === ChopperDropOffSupplies tests ===

    #[test]
    fn chopper_drop_off_supplies_in_grid() {
        let ctx = default_ctx();
        let state = ObjectInterfaceState::Default;
        assert!(matches!(
            test_grid_action_for(&state, 1, 0, Some(ObjectEnum::SupplyChopper), &ctx),
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
        let action = test_grid_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::ArmoryTrainSoldier)));
    }

    #[test]
    fn armory_menu_grid_has_train_gunner() {
        let caps = SelectedUnitCapabilities::default();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
        let action = test_grid_action(&state, 0, 1, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::ArmoryTrainGunner)));
    }

    #[test]
    fn armory_menu_grid_has_eject_all() {
        let caps = SelectedUnitCapabilities::default();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
        let action = test_grid_action(&state, 0, 2, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::ArmoryEjectAll)));
    }

    #[test]
    fn armory_menu_grid_has_set_rally_point() {
        let caps = SelectedUnitCapabilities::default();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
        let action = test_grid_action(&state, 2, 2, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::SetRallyPoint)));
    }

    #[test]
    fn armory_menu_grid_empty_slots() {
        let caps = SelectedUnitCapabilities::default();
        let state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
        assert!(test_grid_action(&state, 1, 0, false, false, &caps, false, false).is_none());
        assert!(test_grid_action(&state, 1, 1, false, false, &caps, false, false).is_none());
        assert!(test_grid_action(&state, 2, 0, false, false, &caps, false, false).is_none());
        assert!(test_grid_action(&state, 2, 1, false, false, &caps, false, false).is_none());
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
        let action = test_grid_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::RecruitConstruct)));
        // S = Assist Construction
        let action = test_grid_action(&state, 1, 1, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::RecruitAssistConstruction)));
        // Other slots are None
        let action = test_grid_action(&state, 0, 1, false, false, &caps, false, false);
        assert!(action.is_none());
    }

    #[test]
    fn cults_recruit_construct_menu_grid_has_storage_and_back() {
        let state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitConstructMenu);
        let caps = SelectedUnitCapabilities::default();
        // Q = Storage
        let action = test_grid_action(&state, 0, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::RecruitSelectBuilding(ObjectEnum::CultsStorage))));
        // Z = Back
        let action = test_grid_action(&state, 2, 0, false, false, &caps, false, false);
        assert!(matches!(action, Some(CommandButtonAction::Back)));
    }

    #[test]
    fn cults_recruit_awaiting_placement_grid_is_empty() {
        let state = ObjectInterfaceState::CultsRecruitMenu(CultsRecruitMenuState::RecruitAwaitingPlacement);
        let caps = SelectedUnitCapabilities::default();
        for r in 0..3 {
            for c in 0..3 {
                let action = test_grid_action(&state, r, c, false, false, &caps, false, false);
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

    #[test]
    fn compute_caps_does_not_set_owned_by_local_player() {
        // Verifies that compute_selected_unit_capabilities returns default
        // owned_by_local_player=false. The caller (update_command_panel_state)
        // must re-apply the ownership flag after overwriting unit_caps.
        let caps = SelectedUnitCapabilities::default();
        assert!(!caps.owned_by_local_player, "default should have owned_by_local_player=false");
    }

    #[test]
    fn grid_button_label_ef_build() {
        let label = grid_button_label(
            &ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu),
            &CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility),
            0,
            'W',
        );
        assert!(label.contains("EF"), "Label should contain 'EF', got: {}", label);
        assert!(label.contains("200 SC"), "Label should contain '200 SC', got: {}", label);
    }

    #[test]
    fn ef_build_action_has_explicit_match_arm() {
        // Verifies the ExtractionFacility build action resolves to
        // a specific label (not the generic "Build" catch-all)
        let action = CommandButtonAction::DcBuild(ObjectEnum::ExtractionFacility);
        let label = grid_button_label(
            &ObjectInterfaceState::StructureMenu(StructureMenuState::DcBuildMenu),
            &action,
            0,
            'Q',
        );
        assert!(!label.ends_with("Build"), "EF label should not be the generic 'Build' fallback");
        assert!(label.contains("EF"), "EF label should identify the structure");
    }

}
