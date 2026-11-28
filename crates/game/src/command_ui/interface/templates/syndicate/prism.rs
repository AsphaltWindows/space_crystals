//! The Prism's panel (`factions/syndicate/units/prism.md > Panel`, designer placeholder, Q0168: "Q Move, R Thrust, X
//! Stop"): its own grid ([`GRID`]), without an Attack, which comes with the Prism's weapon (the design: "A Attack only
//! while armed"). Its right-click is the sim's one rule, as every unit's
//! (`crate::command_ui::resolution::unit_right_click`): a Prism's on an own Tunnel moves to it, being one way only
//! (`sc_sim::network::one_way`).
//! - **R, Thrust** (`super::super::PanelCommand::Thrust`): targeted, a left-click on a point, and queueable. Greyed
//!   while every Prism of the ActiveGroup has spent its charges (`sc_sim::Reject::NoCharges`,
//!   `crate::command_ui::interface::Conditions::refusal`).

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate};

/// The Prism's panel template.
pub const TEMPLATE: PanelTemplate = PanelTemplate::of(InterfaceTemplate::Prism);

/// Its DefaultState commands, slot by slot: Q Move, R Thrust, X Stop.
#[rustfmt::skip]
pub const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{Move, Stop, Thrust};
    [
        Some(Move), None, None, Some(Thrust),
        None,       None, None, None,
        None,       Some(Stop), None, None,
    ]
};
