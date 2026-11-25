//! The Recruit (`factions/cults/units/recruit.md`): a LightInfantry's round body, the Peacekeeper's, the one that turns
//! with its unit's facing, with its label, "RT", on it. Its melee blow is drawn as a tracer, as the Sabre
//! Legionnaire's is (placeholder, Q0001). No panel yet: its Construct and Assist Construction (`> Recruit
//! ObjectInterfaceState`) come with slice M5-J1.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::PanelTemplate;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "RT",
    look: Look::Unit {
        shape: UnitShape::Round,
        tagged: true,
    },
    shot: Some(ShotStyle::Tracer),
    template: PanelTemplate::NONE,
};
