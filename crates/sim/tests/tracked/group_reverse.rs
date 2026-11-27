//! Design change 0439 (the answer to Q0172 part 2): "a Reverse given to several units is placed by the magic box,
//! as a Move is" (`systems/controls/commands.md > Reverse`, `systems/controls/commands.md > Group Destinations (Magic
//! Box)`; `orders/destinations.rs`). Its Verification, with three Hands of Heaven abreast, facing east, 128 su apart:
//! their box is x 768 to 832, y 204 to 500.

use sc_sim::testing::TestSim;
use sc_sim::{MoveStatus, ObjectId, SimEvent, SpawnAt, Vec2, math};

use super::{BACK, HAND, P1, centre, done, facing, flat, reverse_to};

/// Where Hand of Heaven `id` (1 to 3) stands at first.
fn abreast(id: u32) -> Vec2 {
    centre(12, 1 + 2 * id as i32)
}

/// The three Hands of Heaven, reversed as a group to `click`; the sim is at tick 1, when the order started.
fn reversed_to(click: Vec2) -> TestSim {
    let mut builder = flat(24, 14);
    for id in 1..=3 {
        builder = builder.spawn(HAND, Some(P1), SpawnAt::Point(abreast(id)));
    }
    let mut t = builder.start();
    reverse_to(&mut t, 1, &[1, 2, 3], click);
    t.step();
    t
}

/// Unit `id`'s own destination for its current order (`Order::slot`): `None` is the click itself.
fn slot(t: &TestSim, id: u32) -> Option<Vec2> {
    let orders = t
        .sim()
        .object(ObjectId(id))
        .expect("the unit")
        .orders
        .as_ref()
        .expect("orders");
    orders.current.as_ref().expect("an order").order.slot
}

/// Steps until none of the three moves, checking in every tick that each moves 2.5 su at most, and, if `straight`,
/// backwards whenever it moves (a turn that a neighbour cuts while it backs up is made by backing off the way it came,
/// forwards: `movement/tracked.rs`), and calling `each_tick`; then that none of them failed.
fn back_up(t: &mut TestSim, straight: bool, mut each_tick: impl FnMut(&TestSim)) {
    while (1..=3).any(|id| !done(t, id)) {
        let was: Vec<(Vec2, Vec2)> = (1..=3)
            .map(|id| (t.pos(id), math::facing_vector(facing(t, id))))
            .collect();
        t.step();
        for (id, &(pos, ahead)) in (1..=3).zip(&was) {
            let moved = t.pos(id) - pos;
            assert!(
                moved.length() <= BACK + 1e-3,
                "#{id} backs up at {BACK} at most: {moved}"
            );
            assert!(
                !straight || moved.dot(ahead) <= 1e-3,
                "#{id} backs up: {moved} facing {ahead}"
            );
        }
        each_tick(t);
        assert!(t.sim().tick() < 600, "{}", t.ascii(None));
    }
    let failed: Vec<_> = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::MoveFailed { .. }))
        .collect();
    assert!(failed.is_empty(), "{failed:?}\n{}", t.ascii(None));
}

/// "Three units in a row, reversed to a point behind them, back up keeping their row." The click, 400 su behind the
/// middle one, is outside their box: each backs up straight west by the same 400 su to its own spot, the three abreast
/// in every tick, and faces east as it did.
#[test]
fn three_abreast_reversed_to_a_point_behind_them_back_up_keeping_their_row() {
    let shift = Vec2::new(-400.0, 0.0);
    let mut t = reversed_to(abreast(2) + shift);
    for id in 1..=3 {
        assert_eq!(slot(&t, id), Some(abreast(id) + shift), "#{id}'s own spot");
    }
    back_up(&mut t, true, |t| {
        let row: Vec<f32> = (1..=3).map(|id| t.pos(id).x).collect();
        assert!(
            row.iter().all(|&x| x == row[0]),
            "tick {}: out of their row: {row:?}",
            t.sim().tick()
        );
    });
    for id in 1..=3 {
        assert_eq!(
            (t.status(id), t.pos(id)),
            (MoveStatus::Arrived, abreast(id) + shift),
            "#{id}"
        );
        assert_eq!(facing(&t, id), 0.0, "#{id} faces east as it did");
    }
}

/// "Reversed to a point inside their spread, they converge." The click, between the first two, is inside their box:
/// every one of them backs up towards the point itself, turning by the tread law to bring its back round to it. They
/// can't all stand on it: the crowd stops them near it by the occupied target rule (`docs/movement.md > 6.5`, rule 1),
/// all three arrived, and the farthest is far nearer to it than the farthest stood.
#[test]
fn three_abreast_reversed_to_a_point_inside_their_spread_converge() {
    let click = Vec2::new(790.0, 300.0);
    let mut t = reversed_to(click);
    for id in 1..=3 {
        assert_eq!(slot(&t, id), None, "#{id} goes to the click itself");
    }
    back_up(&mut t, false, |_| {});
    let distances: Vec<f32> = (1..=3).map(|id| t.pos(id).distance(click)).collect();
    let before = (1..=3).map(|id| abreast(id).distance(click)).fold(0.0_f32, f32::max);
    println!("converged: {distances:?} su from the click, the farthest {before} before");
    for id in 1..=3 {
        assert_eq!(t.status(id), MoveStatus::Arrived, "#{id}");
    }
    assert!(
        distances.iter().all(|&d| d < before - 60.0),
        "all of them near it: {distances:?}, the farthest {before} before"
    );
}
