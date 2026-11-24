//! The abilities that wait for an attack (module docs of `sim/modes.rs`, design change 0153).

use glam::Vec2;

use super::*;
use crate::combat::AttackPhase;
use crate::conversion::{self, CONVERT_TICKS, Conversion};
use crate::defs::ObjectKind;
use crate::ids::PlayerId;
use crate::riot_shield::RiotShield;
use crate::scenarios;

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

/// Design change 0153 (`factions/colonists/units.md > Weapon Conversion > Eligibility`): "Pressed during those
/// phases, it is deferred, not rejected: the unit finishes Firing and Cooldown, then converts immediately (instead
/// of starting its Reloading phase)". The Convert waits as the unit's pending ability through the Modes steps of its
/// Firing and its Cooldown; in the first one after (its combat phase has put it into a Reloading that hasn't run a
/// tick), it is used, and that same step takes the conversion's first frame, as for a Convert given then. The other
/// abilities don't wait (design change 0172 will hold a raised Riot Shield so, slice M3-S).
#[test]
fn a_convert_given_mid_attack_waits_for_the_cooldown_then_converts_at_once() {
    let firing = AttackPhase::Firing {
        target: Some(ObjectId(9)),
    };
    let (mut sim, rifle, peacekeeper) = fighters(firing);
    let first_new = ObjectId(sim.next_id);
    sim.use_ability(&[rifle], Ability::Convert);
    sim.use_ability(&[peacekeeper], Ability::RiotShield { raise: true });
    assert_eq!(
        object(&sim, rifle).attack.as_ref().unwrap().pending,
        Some(Ability::Convert)
    );
    assert!(conversion::waits_to_convert(object(&sim, rifle)));
    assert!(!conversion::is_converting(object(&sim, rifle)) && !conversion::may_convert(object(&sim, rifle)));
    let shield = object(&sim, peacekeeper).riot_shield;
    assert!(shield.is_some_and(RiotShield::is_going_up), "at once: {shield:?}");
    assert_eq!(object(&sim, peacekeeper).attack.as_ref().unwrap().pending, None);

    for phase in [firing, AttackPhase::Cooldown] {
        set_phase(&mut sim, rifle, phase);
        sim.step_modes(first_new);
        let o = object(&sim, rifle);
        assert_eq!(o.attack.as_ref().unwrap().pending, Some(Ability::Convert), "{phase:?}");
        assert_eq!(
            (o.conversion, o.attack.as_ref().unwrap().phase),
            (None, Some(phase)),
            "{phase:?}"
        );
    }
    set_phase(&mut sim, rifle, AttackPhase::Reloading);
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
}

/// A unit created this tick waits for the next Modes step (`Sim::step_modes`), its pending ability too.
#[test]
fn a_pending_ability_of_a_unit_created_this_tick_waits_a_step() {
    let (mut sim, rifle, _) = fighters(AttackPhase::Cooldown);
    sim.use_ability(&[rifle], Ability::Convert);
    set_phase(&mut sim, rifle, AttackPhase::Reloading);
    sim.step_modes(rifle);
    assert_eq!(
        object(&sim, rifle).attack.as_ref().unwrap().pending,
        Some(Ability::Convert)
    );
    sim.step_modes(ObjectId(sim.next_id));
    assert!(conversion::is_converting(object(&sim, rifle)));
}
