//! The Recruit (`factions/cults/units/recruit.md`): a LightInfantry's round body, the Peacekeeper's, the one that turns
//! with its unit's facing, with its label, "RT", on it. Its melee blow is drawn as a tracer, as the Sabre
//! Legionnaire's is (placeholder, Q0001). Its panel (`> Recruit ObjectInterfaceState`, slice M5-J1:
//! `crate::command_ui::interface::templates::cults::recruit`): Move, the two Construct menus, Assist Construction,
//! Repair (slice M5-N8) and Stop.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::cults;
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "RT",
    look: Look::Unit {
        shape: UnitShape::Round,
        tagged: true,
    },
    shot: Some(ShotStyle::Tracer),
    template: cults::recruit::TEMPLATE,
};
