//! The Prism (`factions/syndicate/units/prism.md`, design change 0367; slice M6-SK): the plain unit box, a square, with
//! "PRM" on it, its first and last letters with its R (its "PM" is the Prayer-mobile's), drawn in flight as every air
//! unit is (`crate::object_render::flight`); unarmed. No panel yet: its own ("**Q** Move, **R** Thrust, **X** Stop")
//! comes with slice M6-SP, which has the Aerospace Assembly build it (plan decision T84): until then it is the debug
//! spawn's alone, and the Tunnel's EjectMenu shows its tile, greyed with a count of 0.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "PRM",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: None,
    template: PanelTemplate::NONE,
};
