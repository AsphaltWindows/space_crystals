//! The Supply Chopper's panel (`factions/gdo/units/supply_chopper.md > ObjectInterfaceState[SupplyChopper]`). The
//! design gives no letters but F's:
//! **Placeholder (the letters, not in the design)**, from `systems/controls/interface.md > CommandPanel > Standard Slot
//! Assignments` and the other units' panels: Q Move, E Hold Position and X Stop, their BasicCombatUnitInterfaceState
//! slots; R Pick Up Supplies, the slot the units' own commands take; S Drop Off Supplies, the Agent's Drop Off's; D
//! Attach to Tower. F, Unload All, is left free, and the chopper has no Load or Enter: it carries Supplies alone
//! (`sc_sim::supply_run::Cargo`). The chopper is unarmed: its A is the selection's ([`super::super::attack_slot`]),
//! shown only beside a unit that can attack.
//! - **R, Pick Up Supplies** ([`PanelCommand::PickUpSupplies`]) awaits a Supply Delivery Station.
//! - **S, Drop Off Supplies** ([`PanelCommand::DropOffSupplies`]) awaits an own Supply Tower, greyed while none of its
//!   choppers carries Supplies (`sc_sim::Reject::NothingToDeliver`).
//! - **D, Attach to Tower** ([`PanelCommand::AttachToTower`]) awaits an own Supply Tower.
//!
//! A click on a Supply Tower another chopper is attached to is an invalid target (`factions/gdo/units/supply_chopper.md
//! > AwaitingTarget[AttachToTower] resolution`; the pointer shows Inactive over it,
//! `crate::command_ui::resolution::pointer_display`, and the gate refuses it, `sc_sim::Reject::TowerTaken`). The
//! chopper is Groupable: each command goes to every chopper of the ActiveGroup. Its right-click is the sim's one rule,
//! as every unit's (`crate::command_ui::resolution::unit_right_click`; `sc_sim::orders::resolution::Rule::SupplyRun`).
//! The Supplies it carries show as pips under its HP bar (`crate::combat_effects::HpBar::supplies`).

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Supply Chopper's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::SupplyChopper);

/// Its DefaultState commands, slot by slot (module docs): Q Move, E Hold Position, R Pick Up Supplies; S Drop Off
/// Supplies, D Attach to Tower; X Stop.
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{AttachToTower, DropOffSupplies, HoldPosition, Move, PickUpSupplies, Stop};
    [
        Some(Move), None,                  Some(HoldPosition),  Some(PickUpSupplies),
        None,       Some(DropOffSupplies), Some(AttachToTower), None,
        None,       Some(Stop),            None,                None,
    ]
};
