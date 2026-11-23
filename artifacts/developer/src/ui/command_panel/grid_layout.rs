use bevy::prelude::*;
use crate::types::*;
use super::super::types::*;
use crate::game::entities::types::{EntityRegistry, SlotConditionContext};

/// Hotkey letters for the 3x4 grid, indexed [row][col]
pub(super) const GRID_HOTKEYS: [[char; 4]; 3] = [
    ['Q', 'W', 'E', 'R'],
    ['A', 'S', 'D', 'F'],
    ['Z', 'X', 'C', 'V'],
];

/// Map a KeyCode to a (row, col) grid slot, if it's a grid hotkey
pub(super) fn keycode_to_grid_slot(key: KeyCode) -> Option<(u8, u8)> {
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
        KeyCode::KeyR => Some((0, 3)),
        KeyCode::KeyF => Some((1, 3)),
        KeyCode::KeyV => Some((2, 3)),
        _ => None,
    }
}

/// Map a StructureMenuState to the ObjectEnum whose registry entry defines its command slots.
/// Returns None for Inert (no commands) and awaiting-placement states (no grid buttons).
fn structure_menu_to_object(sm: &StructureMenuState) -> Option<ObjectEnum> {
    match sm {
        StructureMenuState::DcIdle
        | StructureMenuState::DcBuildMenu
        | StructureMenuState::DcConstructing
        | StructureMenuState::DcReadyToPlace => Some(ObjectEnum::DeploymentCenter),
        StructureMenuState::BarracksMenu => Some(ObjectEnum::Barracks),
        StructureMenuState::EfIdle => Some(ObjectEnum::ExtractionFacility),
        StructureMenuState::SupplyTowerMenu => Some(ObjectEnum::SupplyTower),
        StructureMenuState::HeadquartersMenu => Some(ObjectEnum::Headquarters),
        StructureMenuState::TunnelIdle
        | StructureMenuState::TunnelExpandMenu
        | StructureMenuState::TunnelEjectMenu => Some(ObjectEnum::Tunnel),
        StructureMenuState::RecruitmentCenterMenu => Some(ObjectEnum::RecruitmentCenter),
        StructureMenuState::ArmoryMenu => Some(ObjectEnum::CultsArmory),
        // Awaiting placement — no grid buttons
        StructureMenuState::DcAwaitingPlacement
        | StructureMenuState::EfAwaitingPlacement
        | StructureMenuState::TunnelAwaitingPlacement => None,
        // Inert — no commands (Power Plant, Extraction Plate)
        StructureMenuState::Inert => None,
    }
}

/// Determine which ObjectEnum to look up in the registry for the given interface state.
/// Falls back to `active_type` (from Selection) for Default state and Inert structures.
fn resolve_object_for_state(
    state: &ObjectInterfaceState,
    active_type: Option<ObjectEnum>,
) -> Option<ObjectEnum> {
    match state {
        ObjectInterfaceState::StructureMenu(sm) => structure_menu_to_object(sm),
        ObjectInterfaceState::Default => active_type,
        ObjectInterfaceState::AgentMenu(_) => Some(ObjectEnum::SyndicateAgent),
        ObjectInterfaceState::CultsRecruitMenu(_) => Some(ObjectEnum::CultsRecruit),
        ObjectInterfaceState::AwaitingTarget(_) => None, // handled specially
    }
}

/// Get the action for a given grid slot in the current interface state.
///
/// Uses the EntityRegistry to look up command slots for the active entity type,
/// evaluating slot conditions against the provided runtime context.
pub(super) fn get_grid_slot_action(
    state: &ObjectInterfaceState,
    row: u8,
    col: u8,
    active_type: Option<ObjectEnum>,
    registry: &EntityRegistry,
    ctx: &SlotConditionContext,
) -> Option<CommandButtonAction> {
    // AwaitingTarget is universal: always show Back at (2,0)
    if matches!(state, ObjectInterfaceState::AwaitingTarget(_)) {
        return if row == 2 && col == 0 {
            Some(CommandButtonAction::Back)
        } else {
            None
        };
    }

    let object = resolve_object_for_state(state, active_type)?;
    let entry = registry.get(&object)?;

    // Search command_slots for a matching (state, row, col) whose condition passes.
    // First match wins — conditional slots should be ordered before unconditional fallbacks.
    for slot in entry.command_slots.iter() {
        if slot.state == *state && slot.row == row && slot.col == col {
            // Evaluate condition (None = always visible)
            let condition_met = match &slot.condition {
                None => true,
                Some(cond) => ctx.evaluate(cond),
            };
            if condition_met {
                return Some(slot.action.clone());
            }
        }
    }

    None
}
