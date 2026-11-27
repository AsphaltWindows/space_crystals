//! Design change 0451 at the Factory (`core/entities.md > Entrance and exit > Exit Doorway` point 7: "a newly
//! released vehicle whose rally point is already taken by parked vehicles **parks at the nearest free spot around
//! the point**"; `sim/producers/park.rs`), its Verification's "Also five Raptors and five Twisters at the Factory,
//! with the default point", and "With the point cleared, the stall stays the player's" (design change 0447). The
//! Chop Shop's cases are `tests/chop_shop/parking.rs`'.

use super::*;

/// Runs `t` until the Factory has released `count` units in all, checking every tick that it never waits for its
/// doorway; then for `settle` more ticks. Returns the units, in the order they came out.
fn release_all(t: &mut TestSim, count: usize, settle: Tick) -> Vec<ObjectId> {
    let limit = t.sim().tick() + count as Tick * TWISTER_FRAMES + 50;
    while produced(t, WORKS).len() < count {
        assert!(
            t.sim().tick() < limit,
            "{} out\n{}",
            produced(t, WORKS).len(),
            t.ascii(None)
        );
        t.step();
        let stalled = t.sim().production_status(WORKS).is_some_and(|status| status.waiting);
        assert!(!stalled, "tick {}: stalled\n{}", t.sim().tick(), t.ascii(None));
    }
    run_for(t, settle);
    produced(t, WORKS).into_iter().map(|(_, unit)| unit).collect()
}

/// "Also five Raptors and five Twisters at the Factory, with the default point": queued at once, the five Raptors
/// first, they come out one after the other, the Factory never waiting for its doorway, and park round the rally point
/// it starts with: the first on it, each of the others at the free spot nearest it where it fits as it comes out,
/// none farther from it than the parking radius (3 grid units, `Production::parking_radius_gu`) and its own reach, the
/// half diagonal of its Silhouette. None stands in the doorway's way out, nor is there in a line back toward the door:
/// as many of them stand beyond the point, farther out of side B, as on the Factory's side of it. All ten are out of
/// the doorway, stopped, idle, no two boxes overlapping.
#[test]
fn five_raptors_and_five_twisters_park_round_the_default_rally_point() {
    let mut t = works(5);
    for kind in [RAPTOR; 5].into_iter().chain([TWISTER; 5]) {
        t.at(2, P1, train(kind));
    }
    t.run_until(2);
    let door = door(&t, WORKS);
    let Some(Rally::Point(point)) = rally_of(&t, WORKS) else {
        panic!("a default rally point");
    };
    let units = release_all(&mut t, 10, 200);
    assert_eq!(units.len(), 10);
    let radius = 3.0 * 64.0;
    let mut beyond = 0;
    for (i, &unit) in units.iter().enumerate() {
        let o = object(&t, unit);
        assert_eq!(o.kind, if i < 5 { RAPTOR } else { TWISTER }, "{i}");
        assert!(parked(&t, unit), "{i}: parked: {:?}", t.mover(unit.0));
        let reach = o.def().half_extents().unwrap().length();
        assert!(
            o.pos.distance(point) <= radius + reach,
            "{i} round the point: {}, {} su off\n{}",
            o.pos,
            o.pos.distance(point),
            t.ascii(None)
        );
        let out = (o.pos - door.middle()).dot(door.outward());
        assert!(out > 96.0, "{i}: clear of the doorway's way out: {} su out", out);
        if out > RALLY_OUT {
            beyond += 1;
        }
        for &other in &units[i + 1..] {
            assert!(!shape(&t, unit).overlaps(&shape(&t, other)), "{unit:?} and {other:?}");
        }
    }
    assert!(object(&t, units[0]).pos.distance(point) < 1.0, "the first on the point");
    assert!(
        beyond >= 3,
        "beyond the point too, not in a line toward the door: {beyond}\n{}",
        t.ascii(None)
    );
}

/// "With the point cleared, the stall stays the player's" (design change 0447: "A cleared vehicle building leaves the
/// stall to the player"): the new Factory's rally point cleared by Set Rally Point on itself, as a right-click on it
/// does, and three Raptors queued, the first drives straight out of side B and parks just clear of it; the second
/// comes out behind it, fully inside the doorway, touching it, and stands there; the third waits inside, the Factory
/// waiting, for as long as the player leaves them. Moved off by the player, the first makes way, and the third comes
/// out.
#[test]
fn with_the_rally_point_cleared_the_stall_stays_the_players() {
    let mut t = works(0);
    t.at(
        2,
        P1,
        production(WORKS, ProductionCommand::SetRally(Target::Object(WORKS))),
    );
    t.run_until(2);
    assert_eq!(rally_of(&t, WORKS), None, "cleared");
    let door = door(&t, WORKS);
    for _ in 0..3 {
        t.at(3, P1, train(RAPTOR));
    }
    let half = 24.0;
    step_until(&mut t, 3 * RAPTOR_FRAMES, "two Raptors out", |t| {
        produced(t, WORKS).len() == 2
    });
    run_for(&mut t, 2 * RAPTOR_FRAMES);
    let [(_, first), (_, second)] = produced(&t, WORKS)[..] else {
        panic!("two Raptors: {:?}", produced(&t, WORKS));
    };
    let first_at = door.middle() + door.outward() * half;
    assert!(
        object(&t, first).pos.distance(first_at) < 1e-3,
        "the first just clear of side B: {}",
        object(&t, first).pos
    );
    let second_at = door.middle() - door.outward() * half;
    assert!(
        object(&t, second).pos.distance(second_at) <= 1.0 / 64.0,
        "the second fully inside: {}",
        object(&t, second).pos
    );
    let (a, b) = (shape(&t, first), shape(&t, second));
    assert!(!a.overlaps(&b) && a.gap(&b) < 0.1, "touching the first");
    assert_eq!(t.mover(second.0).exit, Some(WORKS), "the second still in the doorway");
    assert_eq!(produced(&t, WORKS).len(), 2, "the third waits inside");
    let status = t.sim().production_status(WORKS).expect("a producer");
    assert!(status.waiting, "{status:?}");
    let away = door.middle() + door.outward() * 400.0;
    let next = t.sim().tick() + 1;
    t.at(next, P1, order(&[first], OrderKind::Move(at(away))));
    step_until(&mut t, 200, "the third out", |t| produced(t, WORKS).len() == 3);
}
