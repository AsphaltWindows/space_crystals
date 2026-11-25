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

/// What a press does to `o`, as `Sim::use_ability` does it but for `Sim::change_kind`'s rebuild (the forms share
/// everything but the weapon, and the held form keeps the weapon): the form it has stays, its type flips, and a
/// conversion that hasn't started and whose form is the type again is dropped.
fn press(o: &mut Object) {
    hold_form(o);
    o.kind = converts_to(o.kind).unwrap();
    retarget(o);
}

fn weapon_of_kind(kind: ObjectKind) -> Option<&'static crate::defs::AttackAttrs> {
    kind.def().unit.as_ref().unwrap().attack.as_ref()
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

/// Design change 0198 (`> A toggle sets a target form`): a press flips the type, and the unit keeps the form it has,
/// and its weapon, until its conversion ends; pressed again before its conversion started, "it just takes the new
/// target; if that is its current weapon again, it doesn't convert at all". The weapon is the form's (`combat::weapon`)
/// whatever the type. A unit without another form holds none.
#[test]
fn a_press_flips_the_type_and_the_form_stays_until_the_conversion() {
    for kind in [SabreLegionnaire, RifleLegionnaire] {
        let mut o = unit(kind, Some(AttackPhase::Cooldown));
        assert_eq!(
            (form(&o), waits_to_convert(&o), is_converting(&o)),
            (kind, false, false)
        );
        press(&mut o);
        let other = converts_to(kind).unwrap();
        assert_eq!(o.kind, other);
        assert_eq!(o.conversion, Some(Conversion { form: kind, left: None }));
        assert_eq!((form(&o), waits_to_convert(&o), is_converting(&o)), (kind, true, false));
        assert_eq!(combat::weapon(&o), weapon_of_kind(kind), "the weapon in hand");
        assert_ne!(combat::weapon(&o), weapon_of_kind(other));
        press(&mut o);
        assert_eq!((o.kind, o.conversion), (kind, None), "back to its form: nothing to do");
        assert_eq!(combat::weapon(&o), weapon_of_kind(kind));
    }
    let mut pk = unit(ObjectKind::Peacekeeper, None);
    hold_form(&mut pk);
    assert_eq!(pk.conversion, None, "no other form");
}

/// `> Eligibility`: "interrupting that phase immediately": the aim or the reload ends, the target stays engaged (`>
/// Orders are kept`, design change 0150), the new weapon will start ready (design change 0152); and the unit can't move
/// in its mode, so it makes way for no one. Its form is the old one until the conversion completes.
#[test]
fn a_conversion_interrupts_an_aim_or_a_reload_and_holds_the_unit() {
    for phase in [AttackPhase::Aiming { target: ObjectId(9) }, AttackPhase::Reloading] {
        let mut o = unit(SabreLegionnaire, Some(phase));
        press(&mut o);
        start(&mut o);
        assert_eq!(
            o.conversion,
            Some(Conversion {
                form: SabreLegionnaire,
                left: Some(CONVERT_TICKS - 1)
            })
        );
        assert_eq!(o.conversion.unwrap().frames_done(), Some(1));
        assert!(is_converting(&o) && !waits_to_convert(&o));
        assert_eq!(
            o.attack,
            Some(AttackState {
                target: Some(ObjectId(9)),
                phase: None,
                elapsed: 0,
            })
        );
        let mover = o.mover.as_ref().unwrap();
        assert!(!mover.can_move(5.0));
        finish(&mut o);
        assert_eq!(o.conversion, None);
        assert_eq!(o.mover, Some(Mover::default()), "at full speed again");
    }
}

/// `> Convert Command`: "The unit converts in place over 4 frames": the Modes step of the press's tick takes the
/// first, that of the fourth tick the last, and the next one completes it: the unit has its type's form, and its
/// conversion is over. Objects from `first_new` on wait.
#[test]
fn a_conversion_takes_four_frames_then_completes() {
    let mut objects = BTreeMap::new();
    for (id, kind) in [(1, SabreLegionnaire), (2, RifleLegionnaire), (3, SabreLegionnaire)] {
        let mut o = unit(kind, None);
        o.id = ObjectId(id);
        press(&mut o);
        objects.insert(o.id, o);
    }
    let conversion = |objects: &BTreeMap<ObjectId, Object>, id| objects[&ObjectId(id)].conversion;
    let first_new = ObjectId(3);
    for frame in 1..=CONVERT_TICKS {
        step(&mut objects, first_new);
        let one = conversion(&objects, 1).unwrap();
        assert_eq!(one.form, SabreLegionnaire, "frame {frame}: the old form until the end");
        assert_eq!(one.frames_done(), Some(frame), "frame {frame}");
        assert_eq!(one.left, Some(CONVERT_TICKS - frame), "frame {frame}");
        let waiting = Conversion {
            form: SabreLegionnaire,
            left: None,
        };
        assert_eq!(conversion(&objects, 3), Some(waiting), "created this tick: waits");
    }
    step(&mut objects, first_new);
    assert_eq!([conversion(&objects, 1), conversion(&objects, 2)], [None, None]);
    let forms = [1, 2].map(|id| form(&objects[&ObjectId(id)]));
    assert_eq!(forms, [RifleLegionnaire, SabreLegionnaire]);
    assert_eq!(objects[&ObjectId(1)].mover, Some(Mover::default()), "at full speed");
}

/// `> Completion`: "Once the 4-frame conversion has actually started, it runs to completion and cannot be
/// interrupted"; "If the target form changed meanwhile, the unit converts back right after" (design change 0198): the
/// Modes step that completes it starts the conversion back, 8 frames in all, and the unit ends in the form it started
/// in.
#[test]
fn a_conversion_whose_target_flipped_back_runs_on_then_converts_back() {
    let mut objects = BTreeMap::from([(ObjectId(1), unit(SabreLegionnaire, None))]);
    let o = |objects: &BTreeMap<ObjectId, Object>| objects[&ObjectId(1)].clone();
    press(objects.get_mut(&ObjectId(1)).unwrap());
    step(&mut objects, ObjectId(2));
    press(objects.get_mut(&ObjectId(1)).unwrap());
    assert_eq!(o(&objects).kind, SabreLegionnaire, "its type flipped back");
    assert_eq!(
        o(&objects).conversion.unwrap().frames_done(),
        Some(1),
        "its conversion runs on"
    );
    let mut seen = Vec::new();
    for _ in 1..2 * CONVERT_TICKS {
        step(&mut objects, ObjectId(2));
        seen.push(o(&objects).conversion.map(|c| (c.form, c.frames_done())));
    }
    assert_eq!(
        seen,
        [
            Some((SabreLegionnaire, Some(2))),
            Some((SabreLegionnaire, Some(3))),
            Some((SabreLegionnaire, Some(4))),
            Some((RifleLegionnaire, Some(1))),
            Some((RifleLegionnaire, Some(2))),
            Some((RifleLegionnaire, Some(3))),
            Some((RifleLegionnaire, Some(4))),
        ],
        "the Modes step that completes it starts it back"
    );
    step(&mut objects, ObjectId(2));
    assert_eq!((o(&objects).conversion, form(&o(&objects))), (None, SabreLegionnaire));
}

/// Design change 0153 (`> Eligibility`: "the unit finishes Firing and Cooldown, then converts immediately (instead of
/// starting its Reloading phase)"): a unit Firing or in Cooldown waits, its attack untouched, and starts in the first
/// Modes step after its Cooldown, the reload that followed ending.
#[test]
fn a_conversion_waits_for_the_firing_and_the_cooldown() {
    let mut objects = BTreeMap::new();
    let mut o = unit(
        RifleLegionnaire,
        Some(AttackPhase::Firing {
            target: Some(ObjectId(9)),
        }),
    );
    press(&mut o);
    objects.insert(o.id, o);
    for phase in [
        AttackPhase::Firing {
            target: Some(ObjectId(9)),
        },
        AttackPhase::Cooldown,
    ] {
        objects.get_mut(&ObjectId(1)).unwrap().attack.as_mut().unwrap().phase = Some(phase);
        step(&mut objects, ObjectId(2));
        let o = &objects[&ObjectId(1)];
        assert!(waits_to_convert(o) && !is_converting(o), "{phase:?}");
        assert_eq!(o.attack.as_ref().unwrap().phase, Some(phase), "{phase:?}");
        assert_eq!(combat::weapon(o), weapon_of_kind(RifleLegionnaire), "{phase:?}");
    }
    objects.get_mut(&ObjectId(1)).unwrap().attack.as_mut().unwrap().phase = Some(AttackPhase::Reloading);
    step(&mut objects, ObjectId(2));
    let o = &objects[&ObjectId(1)];
    assert_eq!(o.conversion.unwrap().frames_done(), Some(1));
    assert_eq!(o.attack.as_ref().unwrap().phase, None, "no reload");
}
