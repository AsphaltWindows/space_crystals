//! The Peacekeeper's two types (`factions/gdo/units/peacekeeper.md`): the Peacekeeper and the Peacekeeper
//! (Shielded), its Riot Shield up or on its way up (design change 0173). They share their round body, the one that
//! turns with its unit's facing, and carry their labels on it (slice M3-J3: M3-K2's walkthrough found the
//! Peacekeepers unlabelled on the map), two letters each, short enough for the Peacekeeper's 24-su body. Their panel
//! is the BasicCombatUnit template, with the Riot Shield's toggle in R, and no Attack on the Peacekeeper
//! (Shielded)'s while no selected unit can attack (`crate::command_ui::interface::templates::grid`, `attack_slot`).

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

/// The Peacekeeper.
pub const PEACEKEEPER: ClientKind = ClientKind {
    label: "PK",
    look: Look::Unit {
        shape: UnitShape::Round,
        tagged: true,
    },
    shot: Some(ShotStyle::Tracer),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};

/// The Peacekeeper (Shielded): the Peacekeeper's, but for its label.
pub const SHIELDED: ClientKind = ClientKind {
    label: "PS",
    ..PEACEKEEPER
};
