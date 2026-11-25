//! Collision uses the true grid: a structure never seen is open ground for plans but solid, and one that
//! appears in front of a walking unit stops it at once.

use super::*;

/// The plan's collision point: "An enemy structure never seen is open ground for planning, but no unit ever walks
/// into it, because collision uses the true grid." On open ground the unit plans straight through the Generator,
/// re-routes when it sees it, and passes beside it. A goal on the Generator itself ends at its wall.
#[test]
fn an_unseen_structure_is_open_ground_for_plans_but_solid() {
    let generator = (15, 5, 2, 2);
    let start = at(4.5, 6.0);
    let scene = || {
        let mut t = ScenarioBuilder::new(&map(false))
            .spawn(ObjectKind::Generator, Some(P2), tile(15, 5)) // 1: tiles (15, 5) to (16, 6)
            .spawn(PK, Some(P1), SpawnAt::Point(start)) // 2
            .start();
        t.without_auto_targeting();
        t
    };
    let mut t = scene();
    let beyond = at(26.5, 6.0);
    move_to(&mut t, 2, beyond);
    t.step();
    assert_eq!(
        t.mover(2).path,
        [start, beyond],
        "straight through the footprint it doesn't know"
    );
    let track = walk(&mut t, 2);
    assert_eq!((t.status(2), t.pos(2)), (MoveStatus::Arrived, beyond));
    assert!(track.iter().all(|&p| !overlaps(p, generator)));
    assert!(
        track.iter().any(|p| (p.y - start.y).abs() >= SU_PER_GU + HALF - 1.0),
        "around it"
    );
    assert_eq!(failed_moves(&t), 0);

    // A goal inside the footprint: the unit walks up to the wall and has arrived there, the nearest point it can
    // reach (design change 0025).
    let mut t = scene();
    let inside = at(16.0, 6.0);
    move_to(&mut t, 2, inside);
    let track = walk(&mut t, 2);
    assert_eq!(t.status(2), MoveStatus::Arrived);
    assert!(track.iter().all(|&p| !overlaps(p, generator)));
    assert!(t.pos(2).distance(inside) < 1.6 * SU_PER_GU, "at the wall: {}", t.pos(2));
    assert_eq!(failed_moves(&t), 0);
}

/// Collision never waits for knowledge: a Generator put right in front of a walking unit (8 su ahead) stops it at
/// once, before its owner's knowledge and its next staggered check catch up; then it walks around. `TestSim` checks
/// invariant I2 (no box on a blocked tile of the true grid) after every tick of every test here.
#[test]
fn a_structure_appearing_in_front_of_a_unit_stops_it_at_once() {
    let mut t = ScenarioBuilder::new(&map(false))
        .spawn(PK, Some(P1), SpawnAt::Point(at(4.5, 6.0))) // 1
        .start();
    t.without_auto_targeting();
    let goal = at(20.5, 6.0);
    t.order_at(1, P1, &[ObjectId(1)], OrderKind::Move(Target::Point(goal)), false);
    t.run(19);
    assert_eq!(t.pos(1), at(4.5, 6.0) + Vec2::new(76.0, 0.0), "4 su per tick");
    t.debug_at(
        20,
        DebugCommand::Spawn {
            kind: ObjectKind::Generator,
            owner: Some(P2),
            at: tile(6, 5), // tiles (6, 5) to (7, 6): its west wall is 8 su from the unit's box
            orientation: Default::default(),
        },
    );
    let track = walk(&mut t, 1);
    assert!(track.iter().all(|&p| !overlaps(p, (6, 5, 2, 2))));
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, goal));
    assert_eq!(failed_moves(&t), 0);
}
