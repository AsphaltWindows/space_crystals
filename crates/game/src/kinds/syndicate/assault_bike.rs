//! The Assault Bike (`factions/syndicate/units/assault_bike.md`, design change 0367; slice M6-SK): the plain unit box,
//! a square, with its initials, "AB", on it. Its mortar is "like the Grenadier's grenade (slow, dodgeable explosive
//! projectile with area damage)": drawn in flight as the grenade is ([`ShotStyle::Grenade`],
//! `crate::combat_effects::projectiles`), a placeholder (Q0001), with its blast where it lands. Its panel is the
//! BasicCombatUnit template ("ObjectInterfaceState: BasicCombatUnitInterfaceState"), which gives it Attack Ground, as
//! every attack that can miss, and W: Reverse, as it CanReverse. No building makes one until slice M6-SV (plan
//! decision T84): it is the debug spawn's alone, and the Tunnel's EjectMenu shows its tile, greyed with a count of 0.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "AB",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::Grenade),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
