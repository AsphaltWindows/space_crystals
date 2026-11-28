//! The Prism (`factions/syndicate/units/prism.md`, design change 0367; slice M6-SK): the plain unit box, a square, with
//! "PRM" on it, its first and last letters with its R (its "PM" is the Prayer-mobile's), drawn in flight as every air
//! unit; its own panel ("**Q** Move, **R** Thrust, **X** Stop", `factions/syndicate/units/prism.md > Panel`) is
//! `crate::command_ui::interface::templates::syndicate::prism`. Its unfold is drawn as a growing body
//! (`crate::object_render`).

use crate::command_ui::interface::templates::syndicate::prism;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "PRM",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: None,
    template: prism::TEMPLATE,
};
