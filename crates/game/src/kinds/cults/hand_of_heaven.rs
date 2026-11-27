//! The Hand of Heaven (`factions/cults/units/hand_of_heaven.md`, design changes 0361 and 0363): a tracked hull, its
//! Silhouette's 64 × 40 rectangle turned with its facing as its Silhouette is (slice M5-V1), with a dark tread down
//! each long side ([`UnitShape::Tracked`], slice M5-N10) and its initials, "HOH", on it, as the Supply Delivery
//! Station's "SDS" are its three words'. Its launcher is the barrel every unit turret has (`crate::object_render::
//! turret`, slice M5-U1), drawn at the turret's facing, which the sim turns 4° a tick within its 20° (`sc_sim::combat::
//! turret`): to aim at most targets the hull turns, and the barrel barely moves on it. Its missile is drawn in flight
//! by the existing projectile effects (`crate::combat_effects::projectiles`), as every projectile but a grenade and
//! the Rocketman's missile is: a plain ball, slow (8 su a tick), over the straight path to where it lands, lifted by
//! their cosmetic lob with its shadow under it (the design's "semi-ballistic"), a placeholder (Q0001), its style a
//! tracer's ([`ShotStyle::Tracer`]: the Rocketman's [`ShotStyle::Missile`] flies flat to its target's body); its blast
//! is the ring of every blast, of its AoERadius, 96 su, in its shooter's colour (`crate::combat_effects::blasts`). Its panel is the
//! BasicCombatUnit template ("ObjectInterfaceState: BasicCombatUnitInterfaceState, plus **CanTargetGround** (Attack
//! Ground), as every attack that can miss"), whose Attack Ground its CanTargetGround lights, with W: Reverse, its
//! CanReverse (slice M5-V2). The Chop Shop builds it of three Recruits (slice M5-N10, its "**W: Build Hand of
//! Heaven**").

use crate::combat_effects::ShotStyle;
use crate::command_ui::interface::templates::{InterfaceTemplate, PanelTemplate};
use crate::kinds::ClientKind;
use crate::object_render::{Look, UnitShape};

pub const KIND: ClientKind = ClientKind {
    label: "HOH",
    look: Look::Unit {
        shape: UnitShape::Tracked,
        tagged: true,
    },
    shot: Some(ShotStyle::Tracer),
    template: PanelTemplate::of(InterfaceTemplate::BasicCombatUnit),
};
