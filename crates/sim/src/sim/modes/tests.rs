//! The abilities that wait for an attack (module docs of `sim/modes.rs`, design changes 0153 and 0172).

use glam::Vec2;

use super::*;
use crate::combat::AttackPhase;
use crate::conversion::{self, CONVERT_TICKS, Conversion};
use crate::defs::ObjectKind;
use crate::ids::PlayerId;
use crate::object::Object;
use crate::riot_shield::{RiotShield, ShieldMode};
use crate::scenarios;

const RAISE: Ability = Ability::RiotShield { raise: true };
const LOWER: Ability = Ability::RiotShield { raise: false };

/// A game with a Rifle Legionnaire (id 1) and a Peacekeeper (id 2) of player 1, both engaging object 9 in `phase`.
fn fighters(phase: AttackPhase) -> (Sim, ObjectId, ObjectId) {
    let mut sim = Sim::new(&scenarios::empty(8, 6));
    let rifle = sim.insert_object(
        ObjectKind::RifleLegionnaire,
        Some(PlayerId(1)),
        Vec2::splat(100.0),
        None,
        None,
    );
    let peacekeeper = sim.insert_object(
        ObjectKind::Peacekeeper,
        Some(PlayerId(1)),
        Vec2::splat(200.0),
        None,
        None,
    );
    for id in [rifle, peacekeeper] {
        set_phase(&mut sim, id, phase);
    }
    (sim, rifle, peacekeeper)
}

fn set_phase(sim: &mut Sim, id: ObjectId, phase: AttackPhase) {
    let attack = sim.objects.get_mut(&id).unwrap().attack.as_mut().unwrap();
    (attack.target, attack.phase, attack.elapsed) = (Some(ObjectId(9)), Some(phase), 0);
}

fn object(sim: &Sim, id: ObjectId) -> &Object {
    &sim.objects[&id]
}

fn pending(sim: &Sim, id: ObjectId) -> Option<Ability> {
    object(sim, id).attack.as_ref().unwrap().pending
}

/// Design change 0153 (`factions/colonists/units.md > Weapon Conversion > Eligibility`): "Pressed during those
/// phases, it is deferred, not rejected: the unit finishes Firing and Cooldown, then converts immediately (instead
/// of starting its Reloading phase)". Design change 0172 (`factions/gdo/units.md > Peacekeeper > Riot Shield > During
/// the toggle`) does the same to a raise of the Riot Shield: "the shot finishes, then the shield starts rising
/// immediately". Each waits as the unit's pending ability through the Modes steps of its Firing and its Cooldown, the
/// Peacekeeper still a Peacekeeper with its shield down; in the first one after (its combat phase has put it into a
/// Reloading that hasn't run a tick), it is used, and that same step takes the conversion's first frame, and the
/// shield's first tick up, as for one given then. The conversion ends the reload; the shield doesn't, the weapon
/// being the same.
#[test]
fn a_convert_or_a_raise_given_mid_attack_waits_for_the_cooldown_then_applies_at_once() {
    let firing = AttackPhase::Firing {
        target: Some(ObjectId(9)),
    };
    let (mut sim, rifle, peacekeeper) = fighters(firing);
    let first_new = ObjectId(sim.next_id);
    sim.use_ability(&[rifle], Ability::Convert);
    sim.use_ability(&[peacekeeper], RAISE);
    assert_eq!(pending(&sim, rifle), Some(Ability::Convert));
    assert!(conversion::waits_to_convert(object(&sim, rifle)));
    assert!(!conversion::is_converting(object(&sim, rifle)) && !conversion::may_convert(object(&sim, rifle)));
    assert_eq!(pending(&sim, peacekeeper), Some(RAISE));

    for phase in [firing, AttackPhase::Cooldown] {
        for id in [rifle, peacekeeper] {
            set_phase(&mut sim, id, phase);
        }
        sim.step_modes(first_new);
        let o = object(&sim, rifle);
        assert_eq!(pending(&sim, rifle), Some(Ability::Convert), "{phase:?}");
        assert_eq!(
            (o.conversion, o.attack.as_ref().unwrap().phase),
            (None, Some(phase)),
            "{phase:?}"
        );
        let pk = object(&sim, peacekeeper);
        assert_eq!(pending(&sim, peacekeeper), Some(RAISE), "{phase:?}");
        assert_eq!(
            (pk.kind, crate::riot_shield::mode(pk)),
            (ObjectKind::Peacekeeper, Some(ShieldMode::Lowered)),
            "{phase:?}"
        );
    }
    for id in [rifle, peacekeeper] {
        set_phase(&mut sim, id, AttackPhase::Reloading);
    }
    sim.step_modes(first_new);
    let o = object(&sim, rifle);
    let attack = o.attack.as_ref().unwrap();
    assert_eq!(
        (attack.pending, attack.phase, attack.elapsed),
        (None, None, 0),
        "no reload"
    );
    assert_eq!(attack.target, Some(ObjectId(9)), "the target stays engaged");
    assert_eq!(
        o.conversion,
        Some(Conversion {
            left: CONVERT_TICKS - 1
        }),
        "its first frame"
    );
    assert!(!o.mover.as_ref().unwrap().can_move(5.0));

    let pk = object(&sim, peacekeeper);
    let attack = pk.attack.as_ref().unwrap();
    assert_eq!(
        (attack.pending, attack.phase, attack.target),
        (None, Some(AttackPhase::Reloading), Some(ObjectId(9))),
        "its reload goes on"
    );
    assert_eq!(pk.kind, ObjectKind::PeacekeeperShielded);
    assert_eq!(pk.riot_shield, Some(RiotShield { up: 1 }), "its first tick up");
}

/// A raise waits only while the unit is Firing or in Cooldown: Aiming, Reloading or idle, the Peacekeeper becomes a
/// Peacekeeper (Shielded) at once. A lowering never waits, and cancels a raise that does (a placeholder reading:
/// only a command made by hand sends it, a Peacekeeper's panel having no Lower Shield).
#[test]
fn only_a_raise_waits_and_a_lowering_cancels_it() {
    for phase in [AttackPhase::Aiming { target: ObjectId(9) }, AttackPhase::Reloading] {
        let (mut sim, _, peacekeeper) = fighters(phase);
        sim.use_ability(&[peacekeeper], RAISE);
        assert_eq!(pending(&sim, peacekeeper), None, "{phase:?}");
        assert_eq!(object(&sim, peacekeeper).kind, ObjectKind::PeacekeeperShielded);
    }
    let (mut sim, _, peacekeeper) = fighters(AttackPhase::Cooldown);
    sim.use_ability(&[peacekeeper], RAISE);
    assert_eq!(pending(&sim, peacekeeper), Some(RAISE));
    sim.use_ability(&[peacekeeper], LOWER);
    assert_eq!(pending(&sim, peacekeeper), None, "cancelled");
    set_phase(&mut sim, peacekeeper, AttackPhase::Reloading);
    sim.step_modes(ObjectId(sim.next_id));
    let pk = object(&sim, peacekeeper);
    assert_eq!(
        (pk.kind, crate::riot_shield::mode(pk)),
        (ObjectKind::Peacekeeper, Some(ShieldMode::Lowered))
    );
}

/// A unit created this tick waits for the next Modes step (`Sim::step_modes`), its pending ability too.
#[test]
fn a_pending_ability_of_a_unit_created_this_tick_waits_a_step() {
    let (mut sim, rifle, _) = fighters(AttackPhase::Cooldown);
    sim.use_ability(&[rifle], Ability::Convert);
    set_phase(&mut sim, rifle, AttackPhase::Reloading);
    sim.step_modes(rifle);
    assert_eq!(pending(&sim, rifle), Some(Ability::Convert));
    sim.step_modes(ObjectId(sim.next_id));
    assert!(conversion::is_converting(object(&sim, rifle)));
}
