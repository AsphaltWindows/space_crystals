//! The toggles set a target state (module docs of `sim/modes.rs`, design change 0198), and what waits for an attack
//! (design changes 0153 and 0172).

use glam::Vec2;

use super::*;
use crate::combat::{self, AttackPhase};
use crate::conversion::{CONVERT_TICKS, Conversion};
use crate::events::SimEvent;
use crate::ids::PlayerId;
use crate::object::Object;
use crate::riot_shield::{self, RiotShield, ShieldMode};
use crate::scenarios;

const RAISE: Ability = Ability::RiotShield { raise: true };
const T: u32 = crate::defs::RIOT_SHIELD_TOGGLE_TICKS;
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

fn weapon_of_kind(kind: ObjectKind) -> Option<&'static crate::defs::AttackAttrs> {
    kind.def().unit.as_ref().unwrap().attack.as_ref()
}

/// Design change 0198: "The unit's type, and so its SelectionGroup, switches at the press to the target form" (and
/// the Riot Shield's: "including when the raise is deferred behind a shot"), with a `SimEvent::KindChanged`; the
/// unit's attack goes on as it was, the shot under way with the weapon in hand. Design change 0153 (`> Eligibility`:
/// "the unit finishes Firing and Cooldown, then converts immediately (instead of starting its Reloading phase)") and
/// design change 0172 (`> During the toggle`: "the shot finishes, then the shield starts rising immediately"): the
/// conversion and the raise wait through the Modes steps of the Firing and the Cooldown, the Legionnaire fighting with
/// its old weapon and the Peacekeeper's shield down; in the first one after (its combat phase has put it into a
/// Reloading that hasn't run a tick), the conversion takes its first frame, and the shield its first tick up, as for a
/// press then. The conversion ends the reload; the shield doesn't, the weapon being the same.
#[test]
fn a_toggle_mid_attack_switches_the_type_at_once_and_waits_for_the_cooldown() {
    let firing = AttackPhase::Firing {
        target: Some(ObjectId(9)),
    };
    let (mut sim, rifle, peacekeeper) = fighters(firing);
    let first_new = ObjectId(sim.next_id);
    sim.events.clear();
    sim.use_ability(&[rifle], Ability::Convert);
    sim.use_ability(&[peacekeeper], RAISE);
    assert_eq!(
        sim.events,
        [
            SimEvent::KindChanged {
                id: rifle,
                from: ObjectKind::RifleLegionnaire,
                to: ObjectKind::SabreLegionnaire,
            },
            SimEvent::KindChanged {
                id: peacekeeper,
                from: ObjectKind::Peacekeeper,
                to: ObjectKind::PeacekeeperShielded,
            },
        ],
        "at the press"
    );

    for phase in [firing, AttackPhase::Cooldown] {
        for id in [rifle, peacekeeper] {
            set_phase(&mut sim, id, phase);
        }
        sim.step_modes(first_new);
        let o = object(&sim, rifle);
        assert_eq!(o.kind, ObjectKind::SabreLegionnaire, "{phase:?}");
        assert!(
            conversion::waits_to_convert(o) && !conversion::is_converting(o),
            "{phase:?}"
        );
        assert_eq!(conversion::form(o), ObjectKind::RifleLegionnaire, "{phase:?}");
        assert_eq!(
            combat::weapon(o),
            weapon_of_kind(ObjectKind::RifleLegionnaire),
            "{phase:?}"
        );
        assert_eq!(o.attack.as_ref().unwrap().phase, Some(phase), "{phase:?}");
        let pk = object(&sim, peacekeeper);
        assert_eq!(
            (pk.kind, riot_shield::mode(pk), pk.riot_shield),
            (
                ObjectKind::PeacekeeperShielded,
                Some(ShieldMode::Deferred),
                Some(RiotShield { up: 0 })
            ),
            "{phase:?}"
        );
        assert_eq!(riot_shield::damage_factor(pk), 1.0, "{phase:?}: no reduction yet");
    }
    for id in [rifle, peacekeeper] {
        set_phase(&mut sim, id, AttackPhase::Reloading);
    }
    sim.step_modes(first_new);
    let o = object(&sim, rifle);
    let attack = o.attack.as_ref().unwrap();
    assert_eq!((attack.phase, attack.elapsed), (None, 0), "no reload");
    assert_eq!(attack.target, Some(ObjectId(9)), "the target stays engaged");
    assert_eq!(
        o.conversion,
        Some(Conversion {
            form: ObjectKind::RifleLegionnaire,
            left: Some(CONVERT_TICKS - 1)
        }),
        "its first frame"
    );
    assert!(!o.mover.as_ref().unwrap().can_move(5.0));

    let pk = object(&sim, peacekeeper);
    let attack = pk.attack.as_ref().unwrap();
    assert_eq!(
        (attack.phase, attack.target),
        (Some(AttackPhase::Reloading), Some(ObjectId(9))),
        "its reload goes on"
    );
    assert_eq!(pk.kind, ObjectKind::PeacekeeperShielded);
    assert_eq!(pk.riot_shield, Some(RiotShield { up: 1 }), "its first tick up");
}

/// A raise waits only while the unit is Firing or in Cooldown: Aiming, Reloading or idle, the shield starts up in the
/// press's Modes step. Design change 0198 (answering Q0053 point 2): "Lower pressed while a raise is still deferred:
/// the target flips back to shield down, so the waiting raise is dropped; the shield never starts rising"; and a
/// Convert pressed again while the conversion waits: "it just takes the new target; if that is its current weapon
/// again, it doesn't convert at all", its attack going on (its reload not ended).
#[test]
fn a_second_press_before_the_change_began_drops_it() {
    for phase in [AttackPhase::Aiming { target: ObjectId(9) }, AttackPhase::Reloading] {
        let (mut sim, _, peacekeeper) = fighters(phase);
        sim.use_ability(&[peacekeeper], RAISE);
        let pk = object(&sim, peacekeeper);
        assert_eq!(pk.kind, ObjectKind::PeacekeeperShielded);
        assert_eq!(
            riot_shield::mode(pk),
            Some(ShieldMode::Raising { left: T }),
            "{phase:?}"
        );
        sim.step_modes(ObjectId(sim.next_id));
        assert_eq!(
            object(&sim, peacekeeper).riot_shield,
            Some(RiotShield { up: 1 }),
            "{phase:?}"
        );
    }
    let (mut sim, rifle, peacekeeper) = fighters(AttackPhase::Cooldown);
    for ability in [Ability::Convert, Ability::Convert] {
        sim.use_ability(&[rifle], ability);
    }
    sim.use_ability(&[peacekeeper], RAISE);
    assert_eq!(riot_shield::mode(object(&sim, peacekeeper)), Some(ShieldMode::Deferred));
    sim.use_ability(&[peacekeeper], LOWER);
    let o = object(&sim, rifle);
    assert_eq!((o.kind, o.conversion), (ObjectKind::RifleLegionnaire, None), "dropped");
    let pk = object(&sim, peacekeeper);
    assert_eq!(
        (pk.kind, riot_shield::mode(pk)),
        (ObjectKind::Peacekeeper, Some(ShieldMode::Lowered)),
        "dropped"
    );
    for id in [rifle, peacekeeper] {
        set_phase(&mut sim, id, AttackPhase::Reloading);
    }
    sim.step_modes(ObjectId(sim.next_id));
    for id in [rifle, peacekeeper] {
        let o = object(&sim, id);
        assert_eq!(o.attack.as_ref().unwrap().phase, Some(AttackPhase::Reloading), "{id:?}");
        assert!(!conversion::is_converting(o), "{id:?}");
    }
    assert_eq!(object(&sim, peacekeeper).riot_shield, Some(RiotShield { up: 0 }));
}

/// A unit created this tick waits for the next Modes step (`Sim::step_modes`), its waiting conversion too.
#[test]
fn a_waiting_conversion_of_a_unit_created_this_tick_waits_a_step() {
    let (mut sim, rifle, _) = fighters(AttackPhase::Cooldown);
    sim.use_ability(&[rifle], Ability::Convert);
    set_phase(&mut sim, rifle, AttackPhase::Reloading);
    sim.step_modes(rifle);
    assert!(conversion::waits_to_convert(object(&sim, rifle)));
    sim.step_modes(ObjectId(sim.next_id));
    assert!(conversion::is_converting(object(&sim, rifle)));
}

/// `Ability::switched_kind`, the one rule of the type a press switches to, the sim's and the client's: a Convert
/// flips a Legionnaire, a raise or a lowering makes a Peacekeeper the type it asks for, and nothing else switches.
#[test]
fn a_press_switches_to_the_kind_it_asks_for() {
    let cases = [
        (
            Ability::Convert,
            ObjectKind::SabreLegionnaire,
            Some(ObjectKind::RifleLegionnaire),
        ),
        (
            Ability::Convert,
            ObjectKind::RifleLegionnaire,
            Some(ObjectKind::SabreLegionnaire),
        ),
        (Ability::Convert, ObjectKind::Peacekeeper, None),
        (RAISE, ObjectKind::Peacekeeper, Some(ObjectKind::PeacekeeperShielded)),
        (RAISE, ObjectKind::PeacekeeperShielded, None),
        (LOWER, ObjectKind::PeacekeeperShielded, Some(ObjectKind::Peacekeeper)),
        (LOWER, ObjectKind::Peacekeeper, None),
        (Ability::SetDropOff(ObjectId(3)), ObjectKind::Drone, None),
    ];
    for (ability, kind, switched) in cases {
        assert_eq!(ability.switched_kind(kind), switched, "{ability:?} on {kind:?}");
    }
}
