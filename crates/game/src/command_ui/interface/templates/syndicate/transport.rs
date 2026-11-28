//! The Subterranean Transport's panel (`factions/syndicate/units/subterranean_transport.md > Placeholder stats`:
//! "Panel: Q Move, X Stop, **R: Burrow / Surface**, **F: Unload All**"; slice M6-ST1), its two types' ([`SURFACED`],
//! [`BURROWED`]): its own grid ([`grid`]), its A the selection's, as every unit's (`super::super::attack_slot`: the
//! Transport is unarmed), and its right-click the sim's one rule, as every unit's
//! (`crate::command_ui::resolution::unit_right_click`): the Transport follows what it is sent to, and the infantry
//! selected with it board it ("**Loading**: right-click the transport with infantry selected (Enter)").
//! - **F, Unload All** (`super::super::PanelCommand::UnloadAll`): an Unload All order to every Transport of the
//!   ActiveGroup, which comes to rest and puts its passengers round it (`sc_sim::carry`). Greyed while the gate would
//!   refuse it: no one aboard, or underground ("above ground only").
//! - **R, Burrow / Surface**: held, not available ([`Planned::Burrow`] on the surfaced type, [`Planned::Surface`] on
//!   the burrowed one), until slice M6-ST2 makes it the types' vehicle deploy, which the panel's grid then puts in R as
//!   it does every deploy's (`super::super::grid`, `super::super::gdo::artillery_rig::DEPLOY_SLOT`).
//!
//! Its passengers show as pips over its HP bar while its owner sees it (`crate::combat_effects`).

use sc_sim::ObjectKind;

use super::super::{InterfaceTemplate, PanelCommand, PanelTemplate, Planned};
use crate::kinds;

/// The surfaced Transport's panel template: R holds Burrow until slice M6-ST2.
pub const SURFACED: PanelTemplate = PanelTemplate {
    planned: &[(R, Planned::Burrow)],
    ..PanelTemplate::of(InterfaceTemplate::Transport)
};

/// The burrowed Transport's panel template: R holds Surface until slice M6-ST2.
pub const BURROWED: PanelTemplate = PanelTemplate {
    planned: &[(R, Planned::Surface)],
    ..PanelTemplate::of(InterfaceTemplate::Transport)
};

/// R's slot: Burrow / Surface.
const R: usize = 3;

/// Its DefaultState commands, slot by slot: Q Move, R (planned), F Unload All, X Stop.
#[rustfmt::skip]
const GRID: [Option<PanelCommand>; 12] = {
    use PanelCommand::{Move, Stop, UnloadAll};
    [
        Some(Move), None,       None, None,
        None,       None,       None, Some(UnloadAll),
        None,       Some(Stop), None, None,
    ]
};

/// A `kind` Transport's DefaultState commands ([`GRID`]), its commands not in the game yet in their slots
/// ([`PanelTemplate::planned`]).
pub fn grid(kind: ObjectKind) -> [Option<PanelCommand>; 12] {
    let mut grid = GRID;
    for &(slot, planned) in kinds::of(kind).template.planned {
        grid[slot] = grid[slot].or(Some(PanelCommand::Planned(planned)));
    }
    grid
}
