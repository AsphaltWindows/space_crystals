//! The Sparrow (`factions/cults/units/sparrow.md`): the plain unit box, a square, until aircraft are drawn as their
//! slice draws them, with its first and last letters, "SW", on it. Its machine gun's shots are tracers. Its panel is
//! the BasicCombatUnit template ("ObjectInterfaceState: BasicCombatUnitInterfaceState"). Its numbers are in the game
//! since slice M5-G2, and nothing makes one yet: the Airstrip builds it with its slice.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "SW",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::Tracer),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
