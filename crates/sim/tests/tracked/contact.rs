//! The contact step of a turned Silhouette (design change 0323: "for infantry, hover vehicles and **TrackedVehicles**,
//! the step is at **90°** to its heading (straight sideways), and its length is **the unit's own width** (the size of
//! its Silhouette across its heading)"; `systems/units.md > UnitCollision > Ground Collision`). Slice M5-V3 takes the
//! step straight along the face the turned shape met, by that shape's own extent along it (`mover::StepAside::end`),
//! where a box steps along an axis.

use sc_sim::math::{self, PI};
use sc_sim::{MoveStatus, SpawnAt, Vec2};

use super::{HAND, P1, done, facing, flat, move_to};

/// A Hand of Heaven turned 30° drives at an idle Hand of Heaven of its owner, which stands (design change 0440), whose
/// middle is 4 su to its right of its line. Its front meets the other's corner: the face it met lies across its
/// heading, so it steps straight sideways, to its left, the near side, by its own width of 40 su: not along an axis,
/// and not by the 66.6 su its turned box is high. It keeps its facing all the while, moves nothing forward, and once
/// round, arrives.
#[test]
fn a_hand_of_heaven_turned_thirty_degrees_steps_straight_sideways_by_its_own_width() {
    let heading = Vec2::new(math::cos(PI / 6.0), math::sin(PI / 6.0));
    let right = Vec2::new(-heading.y, heading.x);
    let start = Vec2::new(200.0, 200.0);
    let blocker = start + heading * 200.0 + right * 4.0;
    let mut t = flat(20, 14)
        .spawn(HAND, Some(P1), SpawnAt::Point(start))
        .spawn(HAND, Some(P1), SpawnAt::Point(blocker))
        .start();
    let goal = start + heading * 560.0;
    move_to(&mut t, 1, &[1], goal);
    let mut before = t.pos(1);
    while t.mover(1).step_aside.is_none() {
        before = t.pos(1);
        t.step();
        assert!(t.sim().tick() < 100, "{}", t.ascii(Some(t.pos(1))));
    }
    let turned = facing(&t, 1);
    assert!(
        math::wrap_angle(turned - PI / 6.0).abs() < 1e-4,
        "it is turned 30°: {turned}"
    );
    let end = t
        .mover(1)
        .step_aside
        .and_then(|aside| aside.end)
        .expect("a turned Silhouette's step runs to a point");
    let step = end - before;
    assert!((step.length() - 40.0).abs() < 1e-3, "by its own width: {step}");
    assert!(step.dot(heading).abs() < 1e-3, "straight sideways: {step}");
    assert!(step.dot(right) < 0.0, "to the near side, its left: {step}");
    let mut ticks = 0;
    while t.mover(1).step_aside.is_some() {
        let was = t.pos(1);
        t.step();
        assert_eq!(facing(&t, 1), turned, "it keeps its facing");
        assert!((t.pos(1) - was).dot(heading).abs() < 1e-3, "nothing forward");
        ticks += 1;
        assert!(ticks < 20);
    }
    assert!(
        (t.pos(1) - end).length() < 1e-3,
        "it made the whole step: {} for {end}",
        t.pos(1)
    );
    assert_eq!(t.pos(2), blocker, "the idle Hand of Heaven stood");
    while !done(&t, 1) {
        t.step();
        assert!(t.sim().tick() < 600, "{}", t.ascii(Some(t.pos(1))));
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, goal));
    assert_eq!(t.pos(2), blocker);
}
