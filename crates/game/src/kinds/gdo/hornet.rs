//! The Hornet (`factions/gdo/units/hornet.md`, design change 0366, slice M6-GK): the plain unit box, a square, with its
//! first and last letters, "HT", on it, drawn in flight as every air unit is (`crate::object_render::flight`). Its
//! missiles are drawn as missiles ([`ShotStyle::Missile`], the Rocketman's). Its panel is the BasicCombatUnit
//! template, without yet the design's "**F: Return to Base**", which is slice M6-GH1's, with its Airfield entry.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "HT",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::Missile),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
