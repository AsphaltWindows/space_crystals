//! The Sabre Legionnaire (`factions/colonists/units/sabre_legionnaire.md`): a HeavyInfantry's square, the Rifle
//! Legionnaire's too, as the Legionnaire's two forms share one silhouette
//! (`factions/colonists/units/weapon_conversion.md`), which their labels tell apart. Its panel is
//! "BasicCombatUnitInterfaceState, plus: R: Convert to Rifle Legionnaire"
//! (`crate::command_ui::interface::templates::grid`).

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "SL",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::Tracer),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
