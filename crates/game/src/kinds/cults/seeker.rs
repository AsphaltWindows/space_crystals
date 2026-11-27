//! The Seeker (`factions/cults/units/seeker.md`, design change 0364: the Jackal renamed): the plain unit box, its
//! Silhouette's rectangle, turned with its facing as its Silhouette is (slice M5-V1), with its label on it, "SKR": its
//! first and last letters, "SR", are the Shrine's, "SE" the Storage's and "SK" the Skylance's. Its machine gun's shots
//! are tracers. Its panel is the BasicCombatUnit template ("ObjectInterfaceState: BasicCombatUnitInterfaceState"),
//! with W: Reverse available, its CanReverse (slice M5-V2). Its numbers are in the game since slice M5-G2, and the Chop
//! Shop trains it of two Recruits since slice M5-N6.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "SKR",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::Tracer),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
