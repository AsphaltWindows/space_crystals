//! The operator's drone's panel (slice M6-GD; plan §GD: "the drone's panel: Move, Attack, Stop";
//! `factions/gdo/units/drone_operator.md > Launching and piloting`: "The drone is a separate flying unit **controlled
//! directly by the player**"; "It takes normal Attack orders and attacks like any melee unit; using its attack destroys
//! it"): Q Move, A Attack and X Stop, in their BasicCombatUnitInterfaceState slots. Its Attack is the selection's, as
//! every unit's ([`super::super::attack_slot`]), its explosion on the ground unit or structure attacked ("**Explosion
//! targets ground units only**": the gate leaves an air target to nothing, and its right-click on one follows it). The
//! drone is Groupable ("several drones can be flown as one group"). Its right-click is the sim's one rule, as every
//! unit's (`crate::command_ui::resolution::unit_right_click`). Its leash shows as a ring round its operator while it or
//! its operator is selected (`crate::command_ui::indicators::leash`).

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The drone's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::OperatorDrone);

/// Its DefaultState commands, slot by slot: Q Move; A Attack; X Stop.
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{Attack, Move, Stop};
    [
        Some(Move),   None,       None, None,
        Some(Attack), None,       None, None,
        None,         Some(Stop), None, None,
    ]
};
