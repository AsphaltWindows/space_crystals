use glam::Vec2;

use super::*;
use crate::combat::{self, AttackPhase, AttackState};
use crate::defs::ObjectKind::{RifleLegionnaire, SabreLegionnaire};

/// A `kind` unit, as a new one starts (`Sim::insert_object`), with its attack in `phase`, engaging object 9.
fn unit(kind: ObjectKind, phase: Option<AttackPhase>) -> Object {
    Object {
        id: ObjectId(1),
        kind,
        owner: None,
        pos: Vec2::ZERO,
        prev_pos: Vec2::ZERO,
        facing: 0.0,
        prev_facing: 0.0,
        hp: kind.def().max_hp,
        shields: kind.def().max_shields,
        placement: None,
        resource: None,
        orders: None,
        mover: Some(Mover::default()),
        attack: Some(AttackState {
            target: Some(ObjectId(9)),
            phase,
            elapsed: 2,
            pending: None,
        }),
        riot_shield: None,
        extractor: None,
        production: None,
        constructor: None,
        worker: None,
        transform: None,
        conversion: None,
    }
}

/// `> Weapon Conversion > Convert Command`: "each converts to its respective other form". The Legionnaires are the
/// only kinds with another form.
#[test]
fn each_legionnaire_converts_to_its_other_form() {
    assert_eq!(converts_to(SabreLegionnaire), Some(RifleLegionnaire));
    assert_eq!(converts_to(RifleLegionnaire), Some(SabreLegionnaire));
    for &kind in ObjectKind::ALL {
        if let Some(other) = converts_to(kind) {
            assert_eq!(converts_to(other), Some(kind), "{kind:?} converts back");
            assert_ne!(other, kind);
        }
    }
    let others = ObjectKind::ALL
        .iter()
        .filter(|&&kind| converts_to(kind).is_some())
        .count();
    assert_eq!(others, 2);
}

/// `> Eligibility`: "it may be issued freely while the unit is idle, moving, or in the Aiming or Reloading phases",
/// and in the Firing or Cooldown phases too (`combat::mid_attack`), where it is "deferred, not rejected" (design change
/// 0153: `Sim::use_ability` holds it as the unit's pending ability); not while it converts already or waits to (a
/// placeholder: `> Completion`, "cannot be cancelled").
#[test]
fn a_legionnaire_may_convert_unless_it_converts_or_waits_to() {
    for kind in [SabreLegionnaire, RifleLegionnaire] {
        let aiming = AttackPhase::Aiming { target: ObjectId(9) };
        for phase in [None, Some(aiming), Some(AttackPhase::Reloading)] {
            assert!(may_convert(&unit(kind, phase)), "{kind:?} in {phase:?}");
            assert!(!combat::mid_attack(&unit(kind, phase)), "{kind:?} in {phase:?}");
        }
        for phase in [AttackPhase::Firing { target: None }, AttackPhase::Cooldown] {
            let mut o = unit(kind, Some(phase));
            assert!(may_convert(&o) && combat::mid_attack(&o), "{kind:?} in {phase:?}");
            o.attack.as_mut().unwrap().pending = Some(Ability::Convert);
            assert!(waits_to_convert(&o), "{kind:?} in {phase:?}");
            assert!(
                !is_converting(&o) && !may_convert(&o),
                "{kind:?} in {phase:?}: it waits"
            );
        }
        let mut converting = unit(kind, None);
        start(&mut converting);
        assert!(is_converting(&converting) && !may_convert(&converting));
        assert!(!waits_to_convert(&converting));
    }
    assert!(!may_convert(&unit(ObjectKind::Peacekeeper, None)), "no other form");
}

/// `> Eligibility`: "interrupting that phase immediately": the aim or the reload ends, the target stays engaged (`>
/// Orders are kept`, design change 0150), the new weapon will start ready (design change 0152); and the unit can't move
/// in its mode, so it makes way for no one.
#[test]
fn a_conversion_interrupts_an_aim_or_a_reload_and_holds_the_unit() {
    for phase in [AttackPhase::Aiming { target: ObjectId(9) }, AttackPhase::Reloading] {
        let mut o = unit(SabreLegionnaire, Some(phase));
        start(&mut o);
        assert_eq!(o.conversion, Some(Conversion { left: CONVERT_TICKS }));
        assert_eq!(
            o.attack,
            Some(AttackState {
                target: Some(ObjectId(9)),
                phase: None,
                elapsed: 0,
                pending: None,
            })
        );
        let mover = o.mover.as_ref().unwrap();
        assert!(!mover.can_move(5.0));
        finish(&mut o);
        assert_eq!(o.conversion, None);
        assert_eq!(o.mover, Some(Mover::default()), "at full speed again");
    }
}

/// `> Convert Command`: "The unit converts in place over 4 frames": the Modes step of the command's tick takes the
/// first, that of the fourth tick the last, and the next one completes it; objects created this tick wait.
#[test]
fn a_conversion_takes_four_frames_then_completes() {
    let mut objects = BTreeMap::new();
    for (id, kind) in [(1, SabreLegionnaire), (2, RifleLegionnaire), (3, SabreLegionnaire)] {
        let mut o = unit(kind, None);
        o.id = ObjectId(id);
        start(&mut o);
        objects.insert(o.id, o);
    }
    let left = |objects: &BTreeMap<ObjectId, Object>, id| objects[&ObjectId(id)].conversion.map(|c| c.left);
    let first_new = ObjectId(3);
    for (frame, expected) in [(1, 3), (2, 2), (3, 1), (4, 0)] {
        assert_eq!(step(&mut objects, first_new), [], "frame {frame}");
        assert_eq!(left(&objects, 1), Some(expected), "frame {frame}");
        assert_eq!(objects[&ObjectId(1)].conversion.unwrap().frames_done(), frame);
        assert_eq!(left(&objects, 3), Some(CONVERT_TICKS), "created this tick: waits");
    }
    assert_eq!(
        step(&mut objects, first_new),
        [(ObjectId(1), RifleLegionnaire), (ObjectId(2), SabreLegionnaire)]
    );
}
