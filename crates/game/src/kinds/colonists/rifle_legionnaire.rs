//! The Rifle Legionnaire (`factions/colonists/units/rifle_legionnaire.md`): the Sabre Legionnaire's silhouette,
//! with its own label (`factions/colonists/units/weapon_conversion.md`). Its panel is the BasicCombatUnit template,
//! plus "R: Convert to Sabre Legionnaire" (`crate::command_ui::interface::templates::grid`).

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "RL",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::Tracer),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
