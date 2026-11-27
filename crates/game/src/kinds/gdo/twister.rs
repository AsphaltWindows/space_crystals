//! The Twister (`factions/gdo/units/twister.md`, design change 0366, slice M6-GK): the plain unit box, its
//! Silhouette's rectangle turned with its facing, as every vehicle's, with its first and last letters, "TR", on it.
//! Its machine gun's shots are tracers, from the barrel every unit turret has (`crate::object_render::turret`). Its
//! panel is the BasicCombatUnit template plus the design's "**R: Lay Mine** (targeted) and **F: Buy Mine**"
//! (`factions/gdo/units/twister.md > Placeholder stats`), which are slice M6-GM's: (M6-GF) each holds its slot,
//! greyed, "Not available yet" ([`Planned::LayMine`], [`Planned::BuyMine`]). The Factory trains it, its W.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate, Planned};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "TR",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::Tracer),
    template: PanelTemplate {
        planned: &[(3, Planned::LayMine), (7, Planned::BuyMine)],
        ..PanelTemplate::of(InterfaceTemplate::BasicCombatUnit)
    },
};
