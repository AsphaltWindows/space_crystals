//! The Glider's flight (`super`): `systems/units.md > GliderMovement`: "Unit must always maintain movement to stay
//! airborne. When idle, the unit circles at a low speed in tight loops. When given orders, the unit accelerates to max
//! speed. Turn radius is governed by centripetal acceleration (r = v²/a)".

use super::*;

/// "When given orders, the unit accelerates to max speed": a Sparrow just in the game circles at its IdleSpeed of 4
/// su a tick; sent east, it gains its Acceleration of 0.5 su a tick each tick, from the tick of the order, and flies at
/// its MaxSpeed of 10 from the 12th on, all the way. It never stops.
#[test]
fn on_orders_it_speeds_up_by_its_acceleration_to_max_speed() {
    let mut t = quiet(flat(64, 20).spawn(SPARROW, Some(P1), point(300.0, 640.0)).start());
    t.run(10);
    assert!(near(speed(&t, 1), IDLE_SPEED, 1e-4), "{}", speed(&t, 1));
    order(&mut t, 11, P1, &[1], move_to(3800.0, 640.0));
    let mut speeds = Vec::new();
    for _ in 0..30 {
        t.step();
        speeds.push(speed(&t, 1));
    }
    for (n, speed) in speeds.iter().enumerate() {
        let want = (IDLE_SPEED + 0.5 * (n + 1) as f32).min(MAX_SPEED);
        assert!(near(*speed, want, 1e-3), "tick {}: {speed}, not {want}", 11 + n);
    }
}

/// "Turn radius is governed by centripetal acceleration (r = v²/a)": flying east at its MaxSpeed of 10 su a tick, a
/// Sparrow sent to a point far behind it comes about on a radius of 10² / 0.5 = 200 su (design change 0355): 400 su to
/// the side, at full speed throughout.
#[test]
fn its_full_speed_turn_has_a_radius_of_200_su() {
    let mut t = quiet(flat(80, 40).spawn(SPARROW, Some(P1), point(300.0, 1000.0)).start());
    order(&mut t, 1, P1, &[1], move_to(5000.0, 1000.0));
    t.run(60);
    assert!(near(speed(&t, 1), MAX_SPEED, 1e-3));
    let y = t.pos(1).y;
    // Far behind it, and a hair to the south, so that it turns clockwise on screen.
    let tick = t.sim().tick() + 1;
    order(&mut t, tick, P1, &[1], move_to(20.0, y + 1.0));
    let mut widest: f32 = 0.0;
    for _ in 0..80 {
        t.step();
        assert!(near(speed(&t, 1), MAX_SPEED, 1e-3), "{}", speed(&t, 1));
        widest = widest.max(t.pos(1).y - y);
    }
    assert!(near(widest / 2.0, 200.0, 2.0), "{widest}");
}

/// `systems/controls/command_behaviors.md > MovingToLocation`, GliderMovement exception: "Final step is Locomotion =
/// Moving(idle circle over TargetLocation), Orientation = Turning(circle path). Behavior does not complete — continues
/// circling until a new command is issued". A Sparrow sent to a point circles it from its arrival, clockwise, at
/// IdleSpeed, on the idle circle's radius of 1 grid unit (design change 0354), lap after lap; its Move is still its
/// order a minute later, and a Move queued behind it never starts.
#[test]
fn at_its_goal_it_circles_the_point_for_ever() {
    let goal = Vec2::new(1600.0, 640.0);
    let mut t = quiet(flat(40, 20).spawn(SPARROW, Some(P1), point(300.0, 640.0)).start());
    order(&mut t, 1, P1, &[1], move_to(goal.x, goal.y));
    t.order_at(2, P1, &[ObjectId(1)], move_to(300.0, 300.0), true);
    t.run(16 * 20);
    let mut last = t.pos(1);
    for _ in 0..16 * 40 {
        t.step();
        assert!(
            near(t.pos(1).distance(goal), IDLE_RADIUS, 1.0),
            "{}",
            t.pos(1).distance(goal)
        );
        assert!(near(speed(&t, 1), IDLE_SPEED, 1e-3));
        assert!(t.pos(1) != last, "it never stops");
        last = t.pos(1);
    }
    assert_eq!(current(&t, 1), Some(move_to(goal.x, goal.y)));
    let queued = &t.sim().object(ObjectId(1)).unwrap().orders.as_ref().unwrap().queue;
    assert_eq!(queued.len(), 1, "the queued Move waits");
    assert_eq!(t.mover(1).orbit, Some(goal));
}

/// "When idle, the unit circles at a low speed in tight loops": a Sparrow that has had no order circles the point
/// where it came into the game, at IdleSpeed, 64 su from it, and never stops.
#[test]
fn idle_it_circles_where_it_came_into_the_game() {
    let at = Vec2::new(640.0, 640.0);
    let mut t = quiet(flat(20, 20).spawn(SPARROW, Some(P1), SpawnAt::Point(at)).start());
    t.run(16 * 20);
    for _ in 0..16 * 10 {
        t.step();
        assert!(
            near(t.pos(1).distance(at), IDLE_RADIUS, 1.0),
            "{}",
            t.pos(1).distance(at)
        );
        assert!(near(speed(&t, 1), IDLE_SPEED, 1e-3));
    }
    assert_eq!(t.mover(1).orbit, Some(at));
}
