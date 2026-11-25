//! The attack state's own rules (`super::AttackState`): what interrupts an aim, forgetting the objects gone, a
//! location engaged in place of a target (slice M4-O), and its hash while no location is set.

use super::*;

fn state(target: Option<u32>, phase: Option<AttackPhase>) -> AttackState {
    AttackState {
        target: target.map(ObjectId),
        phase,
        elapsed: 1,
        location: None,
    }
}

#[test]
fn a_different_target_interrupts_aiming_only() {
    let aiming = Some(AttackPhase::Aiming { target: ObjectId(5) });
    let mut s = state(Some(5), aiming);
    s.engage(Some(ObjectId(5)));
    assert_eq!(s, state(Some(5), aiming), "the same target keeps the aim");
    s.engage(Some(ObjectId(6)));
    assert_eq!((s.target, s.phase, s.elapsed), (Some(ObjectId(6)), None, 0));
    let mut s = state(Some(5), aiming);
    s.engage(None);
    assert_eq!((s.target, s.phase), (None, None), "any other command");
    for phase in [
        AttackPhase::Firing {
            target: Some(ObjectId(5)),
        },
        AttackPhase::Cooldown,
        AttackPhase::Reloading,
    ] {
        let mut s = state(Some(5), Some(phase));
        s.engage(None);
        assert_eq!(
            (s.target, s.phase, s.elapsed),
            (None, Some(phase), 1),
            "{phase:?} runs on"
        );
    }
}

#[test]
fn forgetting_the_gone_clears_every_reference_to_them() {
    let gone = [ObjectId(2), ObjectId(5)];
    let mut aiming = state(Some(5), Some(AttackPhase::Aiming { target: ObjectId(5) }));
    aiming.forget(&gone);
    assert_eq!((aiming.target, aiming.phase, aiming.elapsed), (None, None, 0));
    let mut firing = state(
        Some(5),
        Some(AttackPhase::Firing {
            target: Some(ObjectId(5)),
        }),
    );
    firing.forget(&gone);
    assert_eq!(firing, state(None, Some(AttackPhase::Firing { target: None })));
    // Others are untouched.
    let mut other = state(Some(3), Some(AttackPhase::Aiming { target: ObjectId(3) }));
    other.forget(&gone);
    assert_eq!(other, state(Some(3), Some(AttackPhase::Aiming { target: ObjectId(3) })));
}

#[test]
fn aiming_and_reloading_are_interruptible() {
    assert!(AttackPhase::Aiming { target: ObjectId(1) }.interruptible());
    assert!(!AttackPhase::Firing { target: None }.interruptible());
    assert!(!AttackPhase::Cooldown.interruptible());
    assert!(AttackPhase::Reloading.interruptible());
    assert!(AttackPhase::AimingAt { at: Vec2::ONE }.interruptible());
    let firing_at = AttackPhase::FiringAt {
        at: Vec2::ONE,
        target: None,
    };
    assert!(!firing_at.interruptible());
}

/// (M4-O) At most one of a target and a location is engaged; an aim at the other is interrupted, as an aim at
/// another target is, and a Firing at a locked location runs on whatever the unit engages.
#[test]
fn engaging_a_location_lets_go_of_the_target_and_back() {
    let at = Vec2::new(300.0, 200.0);
    let mut s = state(Some(5), Some(AttackPhase::Aiming { target: ObjectId(5) }));
    s.engage_ground(at);
    assert_eq!((s.target, s.location, s.phase), (None, Some(at), None));
    s.phase = Some(AttackPhase::AimingAt { at });
    s.engage_ground(at);
    assert_eq!(
        s.phase,
        Some(AttackPhase::AimingAt { at }),
        "the same location keeps the aim"
    );
    s.engage_ground(at + Vec2::X);
    assert_eq!((s.location, s.phase), (Some(at + Vec2::X), None));
    s.phase = Some(AttackPhase::AimingAt { at: at + Vec2::X });
    s.engage(Some(ObjectId(5)));
    assert_eq!((s.target, s.location, s.phase), (Some(ObjectId(5)), None, None));
    let firing = Some(AttackPhase::FiringAt {
        at,
        target: Some(ObjectId(5)),
    });
    s.phase = firing;
    s.engage_ground(at + Vec2::Y);
    s.engage(None);
    assert_eq!(s.phase, firing, "Firing runs on");
    s.forget(&[ObjectId(5)]);
    assert_eq!(
        s.phase,
        Some(AttackPhase::FiringAt { at, target: None }),
        "and still launches"
    );
}

/// The goldens' guard (m4-common: "Hash a new field only while it is set"): an attack state without a location
/// hashes as the three fields it had before slice M4-O, and one with a location differs.
#[test]
fn an_attack_without_a_location_hashes_as_before_it_existed() {
    let hash = |v: &dyn Fn(&mut StateHasher)| {
        let mut h = StateHasher::new();
        v(&mut h);
        h.finish()
    };
    let s = state(Some(5), Some(AttackPhase::Firing { target: None }));
    let before = hash(&|h| {
        s.target.hash(h);
        s.phase.hash(h);
        s.elapsed.hash(h);
    });
    assert_eq!(hash(&|h| s.hash(h)), before);
    let located = AttackState {
        location: Some(Vec2::ZERO),
        ..s.clone()
    };
    assert_ne!(hash(&|h| located.hash(h)), before);
}
