//! The Prayer-mobile (`factions/cults/units/prayer_mobile.md`, design changes 0364 and 0365): the plain unit box, a
//! square, until vehicles are drawn by their silhouettes (plan slice V1), with its initials, "PM", on it; unarmed. Its
//! numbers are in the game since slice M5-G2, and nothing makes one yet: the Chop Shop builds it with its slice, which
//! brings its panel too ("ObjectInterfaceState: BasicCombatUnitInterfaceState without attack commands, plus **R:
//! Inspire** and **F: Suppress**"); until then it has none.

use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "PM",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: None,
    template: PanelTemplate::NONE,
};
