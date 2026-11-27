//! The Sparrow (`factions/cults/units/sparrow.md`): the plain unit box, a square, with its first and last letters,
//! "SW", on it, drawn in flight as every air unit is (slice M5-H1, `crate::object_render::flight`: raised, its shadow
//! on the ground under it). Its machine gun's shots are tracers. Its panel is the BasicCombatUnit template
//! ("ObjectInterfaceState: BasicCombatUnitInterfaceState"). It flies since slice M5-H1; since slice M5-N7 the Airstrip
//! builds it, and it takes off along the Airstrip's runway.

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
