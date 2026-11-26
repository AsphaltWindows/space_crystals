//! Moves (`super`): `systems/controls/command_behaviors.md > MovingToLocation`, GliderMovement exception, as design
//! change 0404 (answering Q0148 part 2) rewrote it: "Final step is Locomotion = Moving(idle circle over
//! TargetLocation), Orientation = Turning(circle path). **The behavior completes when the glider first reaches
//! TargetLocation** (user, 2026-11-27; Q0148; replaces "does not complete"): the next queued order then starts; with
//! nothing queued it keeps circling there, as when idle". A Sparrow reaches its point as it arrives within the idle
//! circle's radius of it, in its flight (phase 5d, `MoveStatus::Arrived`), and its Move completes in the next tick's
//! phase 5a, where the next queued order starts.

use sc_sim::MoveStatus;

use super::*;

/// Steps `t` until unit 1's flight arrives, its current order `order` all the while; returns the closest it came to
/// `point` on the way.
fn until_arrived(t: &mut TestSim, order: OrderKind, point: Vec2) -> f32 {
    let mut closest = f32::MAX;
    loop {
        t.step();
        closest = closest.min(t.pos(1).distance(point));
        assert_eq!(current(t, 1), Some(order), "tick {}", t.sim().tick());
        if t.mover(1).status == MoveStatus::Arrived {
            return closest;
        }
        assert!(t.sim().tick() < 16 * 30, "still on its way at {}", t.pos(1));
    }
}

/// Unit 1, idle, circles `point` at IdleSpeed on the idle circle's radius of 1 grid unit (design change 0354), from 20
/// s on, for 10 s, and never stops.
fn circles_idle(t: &mut TestSim, point: Vec2) {
    assert_eq!((current(t, 1), t.mover(1).orbit), (None, Some(point)));
    t.run(16 * 20);
    let mut last = t.pos(1);
    for _ in 0..16 * 10 {
        t.step();
        let off = t.pos(1).distance(point);
        assert!(near(off, IDLE_RADIUS, 1.0), "{off} su off");
        assert!(near(speed(t, 1), IDLE_SPEED, 1e-3), "{}", speed(t, 1));
        assert!(t.pos(1) != last, "it never stops");
        last = t.pos(1);
    }
    assert_eq!((current(t, 1), t.mover(1).orbit), (None, Some(point)));
}

/// A Move alone ends as it always did: a Sparrow sent to a point arrives within the idle circle's radius of it and
/// circles it, clockwise, at IdleSpeed, on the radius of 1 grid unit, lap after lap. Its Move is its order until it
/// arrives, and complete in the next tick: it is idle from then on, and circles there "as when idle".
#[test]
fn alone_its_move_completes_as_it_arrives_and_it_circles_the_point() {
    let goal = Vec2::new(1600.0, 640.0);
    let mut t = quiet(flat(40, 20).spawn(SPARROW, Some(P1), point(300.0, 640.0)).start());
    order(&mut t, 1, P1, &[1], move_to(goal.x, goal.y));
    until_arrived(&mut t, move_to(goal.x, goal.y), goal);
    // It arrived at the start of this tick's flight, within the radius, and flew on a tick.
    let off = t.pos(1).distance(goal);
    assert!(off <= IDLE_RADIUS + MAX_SPEED, "arrived {off} su off");
    t.step();
    circles_idle(&mut t, goal);
}

/// Design change 0404's Verification: "Shift-queue Move A then Move B for a Sparrow: it passes A and flies on to B,
/// then circles at B." Sent east to A, with a Move to B, 1000 su beyond A and 260 su to the south, Shift-queued behind
/// it: its Move to A is its order until it arrives within the idle circle's radius of A, and the Move to B from the
/// next tick. It flies past A, within that radius, and on to B, never turning back west and never slowing by more than
/// its Deceleration of a tick (in its arrival's tick at A, where it begins to circle until the Move to B starts); at B
/// it arrives, its Move completes, and it circles B, idle.
#[test]
fn shift_queued_moves_pass_a_and_fly_on_to_b_then_circle_at_b() {
    let (a, b) = (Vec2::new(1300.0, 640.0), Vec2::new(2300.0, 900.0));
    let mut t = quiet(flat(40, 20).spawn(SPARROW, Some(P1), point(300.0, 640.0)).start());
    order(&mut t, 1, P1, &[1], move_to(a.x, a.y));
    t.order_at(2, P1, &[ObjectId(1)], move_to(b.x, b.y), true);
    let mut closest = until_arrived(&mut t, move_to(a.x, a.y), a);
    assert!(speed(&t, 1) >= MAX_SPEED - 0.5 - 1e-3, "{}", speed(&t, 1));
    let queue = |t: &TestSim| {
        t.sim()
            .object(ObjectId(1))
            .unwrap()
            .orders
            .as_ref()
            .unwrap()
            .queue
            .len()
    };
    assert_eq!(queue(&t), 1, "the Move to B waits until A is reached");
    let mut x = t.pos(1).x;
    t.step();
    assert_eq!((current(&t, 1), queue(&t)), (Some(move_to(b.x, b.y)), 0));
    loop {
        closest = closest.min(t.pos(1).distance(a));
        assert!(
            t.pos(1).x > x,
            "tick {}: it turned back at {}",
            t.sim().tick(),
            t.pos(1)
        );
        x = t.pos(1).x;
        let speed = speed(&t, 1);
        assert!(speed >= MAX_SPEED - 0.5 - 1e-3, "tick {}: {speed}", t.sim().tick());
        if t.mover(1).status == MoveStatus::Arrived {
            break;
        }
        assert_eq!(current(&t, 1), Some(move_to(b.x, b.y)));
        assert!(t.sim().tick() < 16 * 30, "still on its way at {}", t.pos(1));
        t.step();
    }
    assert!(closest <= IDLE_RADIUS, "it passed A {closest} su off");
    let off = t.pos(1).distance(b);
    assert!(off <= IDLE_RADIUS + MAX_SPEED, "arrived {off} su off B");
    t.step();
    circles_idle(&mut t, b);
}

/// A Move with an AttackMove Shift-queued behind it, which waited for ever before design change 0404: the AttackMove
/// starts the tick after the Sparrow arrives at the Move's point, and takes it on to its own destination, where it is
/// over (`systems/controls/command_behaviors.md > AttackMovingToLocation`, GliderMovement exception: "On arrival at
/// TargetLocation, transitions to idle circling"), and the Sparrow circles there, idle.
#[test]
fn a_queued_attack_move_starts_as_the_move_before_it_arrives() {
    let (a, c) = (Vec2::new(1300.0, 640.0), Vec2::new(2300.0, 640.0));
    let mut t = quiet(flat(40, 20).spawn(SPARROW, Some(P1), point(300.0, 640.0)).start());
    order(&mut t, 1, P1, &[1], move_to(a.x, a.y));
    t.order_at(2, P1, &[ObjectId(1)], OrderKind::AttackMove(c), true);
    until_arrived(&mut t, move_to(a.x, a.y), a);
    t.step();
    assert_eq!(current(&t, 1), Some(OrderKind::AttackMove(c)));
    until_arrived(&mut t, OrderKind::AttackMove(c), c);
    let off = t.pos(1).distance(c);
    assert!(off <= IDLE_RADIUS + MAX_SPEED, "arrived {off} su off");
    t.step();
    circles_idle(&mut t, c);
}

/// Stop and Hold are as they were (`factions/cults/units/sparrow.md > Flight rules`: "**Stop**: it circles where it
/// is. **Hold Position**: it circles there"; `super::orders`), Shift-queued behind a Move too: each starts the tick
/// after the Sparrow arrives within the idle circle's radius of the Move's point, and it circles where it is as it
/// starts, short of the point, not the point itself. The Stop is done at once; the Hold stays its order.
#[test]
fn a_queued_stop_or_hold_circles_where_the_sparrow_reaches_the_point() {
    for (kind, then) in [(OrderKind::Stop, None), (OrderKind::Hold, Some(OrderKind::Hold))] {
        let a = Vec2::new(1300.0, 640.0);
        let mut t = quiet(flat(40, 20).spawn(SPARROW, Some(P1), point(300.0, 640.0)).start());
        order(&mut t, 1, P1, &[1], move_to(a.x, a.y));
        t.order_at(2, P1, &[ObjectId(1)], kind, true);
        until_arrived(&mut t, move_to(a.x, a.y), a);
        let at = t.pos(1);
        t.step();
        assert_eq!((current(&t, 1), t.mover(1).orbit), (then, Some(at)), "{kind:?}");
        let short = at.distance(a);
        assert!(
            short > 0.0 && short <= IDLE_RADIUS,
            "{kind:?}: {short} su short of the point"
        );
        t.run(16 * 20);
        let off = t.pos(1).distance(at);
        assert!(near(off, IDLE_RADIUS, 1.0), "{kind:?}: {off} su off");
        assert!(near(speed(&t, 1), IDLE_SPEED, 1e-3), "{kind:?}: {}", speed(&t, 1));
        assert_eq!(current(&t, 1), then, "{kind:?}");
    }
}
