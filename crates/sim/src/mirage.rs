//! (M6-CM) The Disciple's Mirage and its illusions: `factions/cults/units/disciple.md > Spells > Mirage` (design change
//! 0448, with design change 0450's correction and 0452's answer to Q0185).
//!
//! - **The cast** (`Ability::Mirage`, `Sim::cast_mirage`): "An **instant** spell: the Disciple creates **4 illusions**
//!   of itself", in the commands phase, for its cost ("**50 energy**", `MirageDef::cost`), which the gate asks it to
//!   have (`Reject::NotEnoughEnergy`). "**Casting it cancels Shroud**: casting Mirage ends the Disciple's channel (the
//!   cloud then lingers as usual)": its current order, if it is a Shroud, ends there, and the orders queued behind it
//!   go on from the tick's phase 5a, as after any channel that ends ("a Shift-queued order waits and runs when the
//!   channel ends"). Its other orders stay, as any ability leaves them.
//! - **Where** ([`compass`], `Sim::cast_mirage`): "one on each side of the Disciple, **north, east, south and west**; the
//!   Disciple stays where it is", "**1 grid unit** from the Disciple's centre (centre to centre)" (`MirageDef::offset`).
//!   "A blocked spot puts that illusion at the **nearest free spot**": where the illusion fits by the rule of every
//!   unit put on the map (`Sim::check_unit_fits_facing`: on the map, its box clear of blocked tiles, of blocking
//!   footprints and of the units of its layer, its parent and the illusions placed before it among them), the first
//!   free point of the rings round the spot, from the nearest out, each ring's points from the one straight on from
//!   the Disciple round both ways in turn. **Placeholder (not in the design: how finely and how far it looks, which of
//!   two spots as near it takes, and what becomes of an illusion with no free spot near)**: the rings
//!   `Spells::mirage_search_step_su` apart out to `Spells::mirage_search_radius_gu`, and an illusion with no free spot
//!   within them isn't made.
//! - **An illusion** is an object of its parent's kind with a link to it, [`Object::illusion_of`] ("Seen by enemies:
//!   exactly like a Disciple, on the map and when selected; enemies can't tell them apart"), so that whatever shows a
//!   Disciple to an enemy (the map, the InfoPanel, the computer players' `View`, every event but its owner's
//!   `SimEvent::Mirage`) shows a Disciple of an illusion. It "starts with the Disciple's **current HP**, and has its
//!   MaxHP and speed (augmentations included)": its HP and its Augmentations are its parent's as it is made, which
//!   its MaxHP and its speed follow (`Object::max_hp_with`, `Object::max_speed`), and it faces its parent's way. It
//!   starts idle. **Placeholder (not in the design: what else of its parent's it has)**: nothing more, no buff its
//!   parent holds.
//! - **What the link makes of it**, each read where its rule is:
//!   - "illusions take **double damage from all sources** (Shroud releases included)" ([`damage_taken`],
//!     `combat::damage::apply`: the HP a hit takes, after every step of `systems/combat.md > DamageModifierOrder`);
//!   - "They use **no Unit Control**" (`recruitment::Lineages::of`);
//!   - "Illusions have no energy of their own", and "an illusion's energy bar and InfoPanel always show its **parent's
//!     current energy**, live" (design change 0452): no `Object::energy`, and [`shown_energy`];
//!   - "**No spells**: illusions can't cast" (the gate's `Reject::IllusionsCantCast`);
//!   - "Illusions can't be augmented at the Altar themselves": a building that augments the units inside it
//!     (`ProductionDef::augment`: the Altar) doesn't take one in ([`admitted`]); "illusions may enter a Garrison, but
//!     **don't fire** from it, as Disciples don't" (design change 0450): a Disciple has no weapon, and no stand-in's
//!     in a Garrison (`OccupantFire::stand_in`), so neither has an illusion;
//!   - "**Shroud mirroring**: when the parent Disciple **starts channelling Shroud**, its illusions **play the same
//!     channelling animation**, but make **no cloud**" (`orders::run::update`): each illusion takes a Shroud order of
//!     its own, in place of whatever it did ("An illusion that is moving or busy when the parent starts channelling
//!     stops and plays the animation too (designer's reading)"), which drains nothing and casts no cloud
//!     (`shroud::member`), and ends as the parent's channel does ("all of them stop when the parent's channel ends"),
//!     or by an order of its own ("Each illusion's animation can be **cancelled individually** (as the channel: Stop
//!     or another order)").
//!   - "Other effects: the Shrine's health regeneration works on them (they are own Cults infantry)": a buff affects
//!     an illusion as it does a Disciple (`AreaAbility::affects`), the Shrine's Energy regeneration too, which does
//!     nothing for it.
//! - **How illusions end**: "an illusion lasts until it **dies**, its **parent dies**, or the parent **casts Mirage
//!   again** (the old set vanishes and a new set of 4 appears; a Disciple never has more than one set). An illusion
//!   inside a Garrison when it ends simply vanishes": an illusion dies as any unit does; its parent's removal takes its
//!   illusions along, on the map and held, with no death (`Sim::remove_objects`, `Sim::remove_held`), and so does a
//!   recast, before its new set comes.

use glam::Vec2;

use crate::defs::{MirageDef, ObjectKind};
use crate::ids::ObjectId;
use crate::math::{self, PI};
use crate::object::Object;
use crate::units::gu;

/// The Mirage of `kind`, if its type casts one (`UnitDef::mirage`): the Disciple's.
pub fn def_of(kind: ObjectKind) -> Option<&'static MirageDef> {
    kind.def().unit.as_ref()?.mirage()
}

/// Whether `o` is an illusion (module docs): its link to its parent is set.
pub fn is_illusion(o: &Object) -> bool {
    o.illusion_of.is_some()
}

/// What the HP `o` loses to a hit is multiplied by (module docs): its type's Mirage's `damage_taken` for an illusion
/// ("double damage from all sources"), 1 for everything else.
pub fn damage_taken(o: &Object) -> f32 {
    match o.illusion_of {
        Some(_) => def_of(o.kind).map_or(1.0, |def| def.damage_taken),
        None => 1.0,
    }
}

/// Where the illusions of a Mirage cast at `at` go, before any is blocked (module docs): `def.illusions` spots
/// `def.offset` grid units away, evenly round it clockwise from the north (+y is south), the first to the north: north,
/// east, south and west for the design's 4, exactly.
pub fn compass(def: &MirageDef, at: Vec2) -> Vec<Vec2> {
    let offset = gu(def.offset);
    (0..def.illusions)
        .map(|i| at + direction(i, def.illusions) * offset)
        .collect()
}

/// The `i`-th of `n` directions evenly round, clockwise from the north: a quarter turn's multiple exactly.
fn direction(i: u32, n: u32) -> Vec2 {
    let quarters = u64::from(i) * 4;
    if quarters % u64::from(n) == 0 {
        return match quarters / u64::from(n) {
            0 => Vec2::NEG_Y,
            1 => Vec2::X,
            2 => Vec2::Y,
            _ => Vec2::NEG_X,
        };
    }
    math::facing_vector(-PI / 2.0 + i as f32 * 2.0 * PI / n as f32)
}

/// The energy `o` shows, and its pool's maximum, in points (design change 0452): an illusion's parent's, found by
/// `parent` (on the map or held), "read live" (`crate::energy::of`); every other object's own. `None` without a pool,
/// and for an illusion whose parent is gone.
pub fn shown_energy<'a>(o: &Object, parent: impl FnOnce(ObjectId) -> Option<&'a Object>) -> Option<(f32, f32)> {
    match o.illusion_of {
        Some(id) => parent(id).and_then(crate::energy::of),
        None => crate::energy::of(o),
    }
}

/// Whether a building of `kind` refuses illusions (module docs): one that augments the units inside it
/// (`ProductionDef::augment`, the Altar's: "Illusions can't be augmented at the Altar themselves").
pub fn refuses_illusions(kind: ObjectKind) -> bool {
    (kind.def().structure.as_ref())
        .and_then(|structure| structure.production)
        .is_some_and(|production| !production.augment.is_empty())
}

/// Whether `building` takes `unit` in, as far as illusions go (module docs): it does, but for an illusion at a building
/// that refuses them ([`refuses_illusions`]). What its type takes in otherwise is `holding::admits`'s.
pub fn admitted(building: &Object, unit: &Object) -> bool {
    !(is_illusion(unit) && refuses_illusions(building.kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The design's 4: north, east, south and west, 1 grid unit (64 space units) from the Disciple's centre, exactly.
    #[test]
    fn the_four_illusions_go_north_east_south_and_west() {
        let def = def_of(ObjectKind::Disciple).expect("the Disciple casts Mirage");
        let at = Vec2::new(640.0, 320.0);
        assert_eq!(
            compass(def, at),
            [
                Vec2::new(640.0, 256.0),
                Vec2::new(704.0, 320.0),
                Vec2::new(640.0, 384.0),
                Vec2::new(576.0, 320.0),
            ]
        );
        let six = MirageDef { illusions: 6, ..*def };
        let spots = compass(&six, at);
        assert_eq!(
            (spots.len(), spots[0], spots[3]),
            (6, Vec2::new(640.0, 256.0), Vec2::new(640.0, 384.0))
        );
        assert!(spots.iter().all(|p| (p.distance(at) - 64.0).abs() < 1e-3), "{spots:?}");
        let none = MirageDef { illusions: 0, ..*def };
        assert!(compass(&none, at).is_empty());
    }

    /// The Altar augments, so it refuses illusions; the Garrison, which only shelters them, and every other building,
    /// doesn't.
    #[test]
    fn only_the_altar_refuses_illusions() {
        let refusing: Vec<ObjectKind> = (ObjectKind::ALL.iter().copied())
            .filter(|&kind| refuses_illusions(kind))
            .collect();
        assert_eq!(refusing, [ObjectKind::Altar]);
    }
}
