//! The Grenadier (`factions/syndicate/units/grenadier.md`, design change 0260; slice M4-U): a HeavyInfantry's square
//! with its label, as the Agent's. Its grenade is no laser (design change 0207 makes only the Syndicate's lasers red):
//! a projectile, drawn in flight as a lobbed grenade ([`ShotStyle::Grenade`], `crate::combat_effects::projectiles`),
//! and its blast where it lands. Its panel is the BasicCombatUnit template (`> ObjectInterfaceState:
//! BasicCombatUnitInterfaceState`), which gives it Attack Ground, as every attack that can miss.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "GR",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::Grenade),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
