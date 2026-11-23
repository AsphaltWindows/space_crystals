use bevy::prelude::*;
use crate::types::*;
use crate::game::types::{
    ObjectInstance,
    Player, SyndicatePlayerResources,
    TunnelState, TunnelTier, TunnelOperation,
    tunnel_t2_upgrade_cost, tunnel_t3_upgrade_cost,
};
use super::super::types::*;
use super::helpers::{find_syndicate_resources_mut, spawn_grid_button, spawn_empty_grid_cell};

pub(super) fn execute_tunnel_upgrade(
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
pub(super) fn execute_tunnel_cancel_upgrade(
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
pub(super) fn execute_tunnel_select_expansion(
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
pub(super) fn execute_tunnel_eject_unit(
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
pub(super) fn build_tunnel_expand_grid(
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

    let hotkeys: Vec<char> = vec!['Q', 'W', 'E', 'R', 'A', 'S', 'D', 'F', 'Z', 'X', 'C', 'V'];
    let mut slot = 0;

    for (expansion, min_tier) in &expansions {
        if slot >= 8 { break; } // Reserve row 2 for Back
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
        let row = (slot / 4) as u8;
        let col = (slot % 4) as u8;
        spawn_grid_button(grid, &label, action, enabled, false, false, row, col);
        slot += 1;
    }

    // Fill empty cells up to slot 8
    while slot < 8 {
        spawn_empty_grid_cell(grid);
        slot += 1;
    }

    // Back button at (2, 0) = slot 8 = 'Z'
    spawn_grid_button(grid, "[Z] Back", CommandButtonAction::Back, true, false, false, 2, 0);

    // Fill remaining empty cells (slots 9-11)
    spawn_empty_grid_cell(grid);
    spawn_empty_grid_cell(grid);
    spawn_empty_grid_cell(grid);
}

/// Build the dynamic grid for TunnelEjectMenu.
/// Shows unit types present in the tunnel network, grouped by ObjectEnum with counts.
/// Units whose base category exceeds the tunnel's tier are visible but disabled.
pub(super) fn build_tunnel_eject_grid(
    grid: &mut ChildSpawnerCommands,
    target_entity: Option<Entity>,
    tunnel_query: &Query<(&TunnelState, &Owner)>,
    network_units: &Query<(&ObjectInstance, &crate::game::units::types::state::behavior::InTunnelNetwork)>,
) {
    use std::collections::HashMap;
    use crate::game::entities::types::UnitDefinition;
    use crate::game::entities::factions::syndicate::units::agent::SyndicateAgentDef;
    use crate::game::entities::factions::syndicate::units::guard::SyndicateGuardDef;

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
    let hotkeys = ['Q', 'W', 'E', 'R', 'A', 'S', 'D', 'F'];
    let mut slot = 0;
    for (&unit_type, &count) in &sorted_types {
        if slot >= 8 { break; }

        let row = slot / 4;
        let col = slot % 4;
        let hotkey = hotkeys[slot];

        // Check transit tier eligibility
        let unit_base = match unit_type {
            ObjectEnum::SyndicateAgent => SyndicateAgentDef::unit_stats().unit_base,
            ObjectEnum::SyndicateGuard => SyndicateGuardDef::unit_stats().unit_base,
            _ => crate::types::UnitBaseEnum::HeavyInfantry, // fallback
        };
        let can_transit = tunnel_tier.can_transit(&unit_base);

        let label = format!("[{}] {} ({})", hotkey, unit_type.object_type().name, count);
        let action = CommandButtonAction::TunnelEjectUnit(unit_type);
        spawn_grid_button(grid, &label, action, can_transit, false, false, row as u8, col as u8);
        slot += 1;
    }

    // Fill remaining empty cells up to slot 8
    while slot < 8 {
        spawn_empty_grid_cell(grid);
        slot += 1;
    }

    // Back button at (2, 0) = 'Z'
    spawn_grid_button(grid, "[Z] Back", CommandButtonAction::Back, true, false, false, 2, 0);
    // Fill remaining empty cells (slots 9-11)
    spawn_empty_grid_cell(grid);
    spawn_empty_grid_cell(grid);
    spawn_empty_grid_cell(grid);
}

/// Check if tier_a is >= tier_b
pub(super) fn tier_at_least(tier_a: &TunnelTier, tier_b: &TunnelTier) -> bool {
    match (tier_a, tier_b) {
        (_, TunnelTier::Tier1) => true,
        (TunnelTier::Tier2, TunnelTier::Tier2) => true,
        (TunnelTier::Tier3, TunnelTier::Tier2) => true,
        (TunnelTier::Tier3, TunnelTier::Tier3) => true,
        _ => false,
    }
}

/// Get the cost for upgrading a tunnel, or None if not upgradeable
pub(super) fn get_tunnel_upgrade_cost(
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
pub(super) fn get_syndicate_supplies(
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
