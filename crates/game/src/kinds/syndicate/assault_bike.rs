//! The Assault Bike (`factions/syndicate/units/assault_bike.md`, design change 0367; slices M6-SK and M6-SV): the plain
//! unit box, its Silhouette's 56 × 32 rectangle, turned with its facing as its Silhouette is (slice M5-V1), with its
//! initials, "AB", on it, and its turret's barrel, which turns within its 40° (slice M5-U1). Its mortar is "like the
//! Grenadier's grenade (slow, dodgeable explosive projectile with area damage)": drawn in flight as the grenade is
//! ([`ShotStyle::Grenade`], `crate::combat_effects::projectiles`), a placeholder (Q0001), with its blast where it
//! lands. Its panel is the BasicCombatUnit template ("ObjectInterfaceState: BasicCombatUnitInterfaceState"), which
//! gives it Attack Ground, as every attack that can miss, and W: Reverse, as it CanReverse. The Vehicle Assembly trains
//! it into the Tunnel Network (slice M6-SV), and a Tier 2+ Tunnel's EjectMenu lets it out ("exits only through Tier 2+
//! Tunnels"): a Tier 1 Tunnel's shows its tile greyed, "This Tunnel's tier is too low".

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
