use bevy::prelude::*;
use crate::types::*;
use crate::game::types::{
    ObjectInstance, StructureInstance,
    DeploymentCenterState, BarracksState, ExtractionFacilityState,
    SupplyTowerState, TunnelState, HeadquartersState,
};
use crate::game::units::types::commands::CommandType;
use crate::game::world::types::{SpaceCrystalPatch, SupplyDeliveryStation};
use super::super::types::*;

/// Describes what target_info the resolution function needs to know about a
/// cursor-hovered entity, extracted from ECS queries.
#[derive(Debug, Clone, Default)]
pub(super) struct CursorEntityInfo {
    pub(super) has_crystal_patch: bool,
    pub(super) has_supply_station: bool,
    pub(super) has_tunnel: bool,
    pub(super) is_own: bool,
    pub(super) has_supply_tower: bool,
}

/// Pure function that resolves the pointer display type from the current
/// interface state, cursor target, and selection capabilities.
///
/// Separated from the ECS system for testability.
pub(super) fn resolve_pointer_display(
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
pub(super) fn resolve_awaiting_target(
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
pub(super) fn resolve_default_state(
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
