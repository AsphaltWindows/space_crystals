//! The Hand of Heaven (`factions/cults/units/hand_of_heaven.md`, design changes 0361 and 0363): the plain unit box, its
//! Silhouette's rectangle, turned with its facing as its Silhouette is (slice M5-V1), with its initials, "HOH", on it,
//! as the Supply Delivery Station's "SDS" are its three words'. Its missile is drawn in flight as a plain ball, as
//! every projectile but a grenade is (`crate::combat_effects::projectiles`), a placeholder (Q0001), its style a
//! tracer's ([`ShotStyle::Tracer`], every weapon's but a laser's and a grenade's). Its panel is the BasicCombatUnit
//! template ("ObjectInterfaceState: BasicCombatUnitInterfaceState, plus **CanTargetGround** (Attack Ground), as every
//! attack that can miss"). Its numbers are in the game since slice M5-G2, and nothing makes one yet: the Chop Shop,
//! which trains the Seeker since slice M5-N6, builds it with slice M5-N10.

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "HOH",
    look: Look::Unit {
        shape: UnitShape::Square,
        tagged: true,
    },
    shot: Some(ShotStyle::Tracer),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
