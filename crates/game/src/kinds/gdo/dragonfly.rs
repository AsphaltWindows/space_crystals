//! The Dragonfly (`factions/gdo/units/dragonfly.md`, design change 0366, slice M6-GK): the plain unit box, a square,
//! with its first and last letters, "DY", on it, drawn in flight as every air unit is (`crate::object_render::flight`).
//! Its missile is drawn as a missile ([`ShotStyle::Missile`], the Rocketman's). Its panel is the BasicCombatUnit
//! template, and (M6-GL) the design's "**F: Return to Base**", which the grid adds for an aircraft, one with ammo
//! (`crate::command_ui::interface::templates::grid`), and its missile left shows as a pip under its HP bar
//! (`crate::combat_effects`), as the Hornet's do. Its vertical landing on an Airfield's slot (design change 0428 point
//! 5) is drawn as the Hornet's: stopped over its slot, sinking to the ground over its 16 frames, and rising straight up
//! off it (`crate::object_render::flight`).

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "DY",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::Missile),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
