use super::*;
use crate::hash::StateHasher;

const PRISM: ObjectKind = ObjectKind::Prism;

fn hash_of(thrust: &Thrust) -> u64 {
    let mut h = StateHasher::new();
    thrust.hash(&mut h);
    h.finish()
}

/// The Prism's thrusts start with none spent; a type without a `ThrustDef` has none.
#[test]
fn a_type_with_thrusts_starts_with_every_charge() {
    assert_eq!(fresh(PRISM), Some(Thrust::default()));
    assert_eq!(fresh(ObjectKind::Guard), None);
    assert_eq!(fresh(ObjectKind::Phantom), None);
}

/// A thrust spends a charge as it starts and moves the full distance toward its point, 3 grid units over 16 ticks
/// (12 su a tick), whether the point is 10 grid units away or 1; its order ends once the burst is over.
#[test]
fn a_thrust_moves_its_full_distance_toward_the_point_whatever_its_distance() {
    let from = Vec2::new(640.0, 640.0);
    for far in [10.0, 1.0] {
        let mut thrust = Thrust::default();
        let mut fired = false;
        let at = from + Vec2::new(gu(far), 0.0);
        assert!(!run(Some(&mut thrust), PRISM, from, 0.0, at, &mut fired));
        assert!(fired);
        let burst = Burst {
            step: Vec2::new(12.0, 0.0),
            left: 16,
        };
        assert_eq!(
            thrust,
            Thrust {
                used: 1,
                burst: Some(burst)
            },
            "{far} GU away"
        );
        let mut slot = Some(thrust);
        let mut moved = Vec2::ZERO;
        for tick in 0..16 {
            assert!(
                !run(slot.as_mut(), PRISM, from + moved, 0.0, at, &mut fired),
                "tick {tick}"
            );
            moved += burst_step(&mut slot).expect("under way");
        }
        assert_eq!(moved, Vec2::new(192.0, 0.0));
        assert_eq!(slot, Some(Thrust { used: 1, burst: None }));
        assert_eq!(burst_step(&mut slot), None, "over");
        assert!(run(slot.as_mut(), PRISM, from + moved, 0.0, at, &mut fired), "complete");
    }
}

/// Placeholder (Q0198): a thrust that starts with every charge spent ends at once, without a burst; one that
/// starts during another's burst waits for it, its charge unspent; at its own position it thrusts along its facing.
#[test]
fn a_thrust_without_a_charge_ends_at_once_and_one_during_a_burst_waits() {
    let from = Vec2::new(640.0, 640.0);
    let at = from + Vec2::new(0.0, 640.0);
    let mut spent = Thrust { used: 3, burst: None };
    let mut fired = false;
    assert!(run(Some(&mut spent), PRISM, from, 0.0, at, &mut fired));
    assert_eq!((spent, fired), (Thrust { used: 3, burst: None }, false));
    let under_way = Burst {
        step: Vec2::new(12.0, 0.0),
        left: 5,
    };
    let mut busy = Thrust {
        used: 1,
        burst: Some(under_way),
    };
    assert!(!run(Some(&mut busy), PRISM, from, 0.0, at, &mut fired));
    assert_eq!(
        (busy, fired),
        (
            Thrust {
                used: 1,
                burst: Some(under_way)
            },
            false
        )
    );
    let mut here = Thrust::default();
    assert!(!run(
        Some(&mut here),
        PRISM,
        from,
        std::f32::consts::FRAC_PI_2,
        from,
        &mut fired
    ));
    let step = here.burst.expect("fired").step;
    assert!(step.distance(Vec2::new(0.0, 12.0)) < 1e-4, "south, as it faces: {step}");
    let mut fired = false;
    assert!(run(None, ObjectKind::Guard, from, 0.0, at, &mut fired), "no thrusts");
}

/// The invariants' allowance: a tick with a burst at either end, or a charge spent in between, moved the unit by up to
/// a step; any other moved it by none.
#[test]
fn a_burst_moved_the_unit_only_in_its_ticks() {
    let burst = Burst {
        step: Vec2::new(12.0, 0.0),
        left: 3,
    };
    let idle = Thrust { used: 1, burst: None };
    let bursting = Thrust {
        used: 1,
        burst: Some(burst),
    };
    assert_eq!(burst_moved(PRISM, Some(&idle), Some(&idle)), None);
    assert_eq!(
        burst_moved(PRISM, Some(&bursting), Some(&idle)),
        Some(12.0),
        "its last tick"
    );
    assert_eq!(
        burst_moved(PRISM, Some(&Thrust::default()), Some(&idle)),
        Some(12.0),
        "a tick's burst"
    );
    assert_eq!(burst_moved(ObjectKind::Guard, None, None), None);
}

/// The hash follows the charges spent and the burst under way.
#[test]
fn the_hash_follows_the_charges_and_the_burst() {
    let fresh = Thrust::default();
    let spent = Thrust { used: 1, burst: None };
    let burst = Burst {
        step: Vec2::new(12.0, 0.0),
        left: 16,
    };
    let bursting = Thrust {
        used: 1,
        burst: Some(burst),
    };
    assert_ne!(hash_of(&fresh), hash_of(&spent));
    assert_ne!(hash_of(&spent), hash_of(&bursting));
    let later = Thrust {
        burst: Some(Burst { left: 15, ..burst }),
        ..bursting
    };
    assert_ne!(hash_of(&bursting), hash_of(&later));
}
