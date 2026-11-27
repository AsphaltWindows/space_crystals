//! The GliderMovement model alone (`movement::glider`): a lone Glider's flight, tick by tick, without the rest of the
//! sim. The scenario tests in `crates/sim/tests/air/` cover Sparrows in a game: orders, strafing, the map's edge.

use super::*;
use crate::defs::ObjectKind;
use crate::ids::ObjectId;
use crate::math::PI;
use crate::mover::Takeoff;

/// The Sparrow's half extents: 40 × 40.
const HALF: Vec2 = Vec2::splat(20.0);

/// The Sparrow's GliderMovement (`factions/cults/units/sparrow.md`, design change 0355): IdleSpeed 4, MaxSpeed 10,
/// Acceleration and Deceleration 0.5, MaxCentripetalAcceleration 0.5.
fn sparrow() -> Glider {
    let unit = ObjectKind::Sparrow.def().unit.as_ref().expect("a unit");
    Glider::of(&unit.movement).expect("a Glider")
}

/// A test glider: the Sparrow with the MaxCentripetalAcceleration of 1 su a frame² it had before design change 0355.
fn test_glider() -> Glider {
    Glider {
        max_centripetal_acceleration: 1.0,
        ..sparrow()
    }
}

/// 64 × 64 open tiles: 4096 su a side.
fn map() -> Map {
    Map::new(64, 64)
}

/// A lone Glider in flight, and the object it may head for.
struct Flight {
    glider: Glider,
    pos: Vec2,
    facing: f32,
    mover: Mover,
    /// What `MoveGoal::Object` names, where it stands.
    target: Option<Chased>,
    map: Map,
}

impl Flight {
    /// `glider` at `pos`, facing `facing`, that hasn't flown yet.
    fn new(glider: Glider, pos: Vec2, facing: f32) -> Self {
        Self {
            glider,
            pos,
            facing,
            mover: Mover::default(),
            target: None,
            map: map(),
        }
    }

    /// The same, already flying along its facing at `speed`.
    fn flying(glider: Glider, pos: Vec2, facing: f32, speed: f32) -> Self {
        let mut flight = Self::new(glider, pos, facing);
        flight.mover.momentum = math::facing_vector(facing) * speed;
        flight
    }

    /// With the goal `goal`, from tick 1.
    fn going(mut self, goal: MoveGoal) -> Self {
        self.mover.go_to(goal, None, 1);
        self
    }

    /// Heading for a 40 × 40 object at `at` (its id 9), on a strafing run out to `range`.
    fn strafing(self, at: Vec2, range: f32) -> Self {
        self.strafing_at(at, HALF, range)
    }

    /// Heading for an object at `at` with half extents `half` (its id 9), on a strafing run out to `range`.
    fn strafing_at(mut self, at: Vec2, half: Vec2, range: f32) -> Self {
        self.target = Some(Chased {
            id: ObjectId(9),
            pos: at,
            shape: Shape::rect(at, half),
        });
        self.mover.go_to(
            MoveGoal::Object {
                id: ObjectId(9),
                reach: 0.0,
            },
            None,
            1,
        );
        self.mover.strafe(range);
        self
    }

    /// One tick. Returns the angle it turned.
    fn tick(&mut self) -> f32 {
        let mut turned = 0.0;
        let target = self.target.as_ref();
        (self.glider).fly(
            &mut self.pos,
            &mut self.facing,
            &mut turned,
            &mut self.mover,
            target,
            HALF,
            &self.map,
        );
        turned
    }

    fn speed(&self) -> f32 {
        self.mover.momentum.length()
    }
}

/// Whether `a` and `b` differ by at most `tolerance`.
fn near(a: f32, b: f32, tolerance: f32) -> bool {
    (a - b).abs() <= tolerance
}

/// `systems/units.md > GliderMovement`: "When given orders, the unit accelerates to max speed". A Sparrow that hasn't
/// flown yet starts at IdleSpeed (the module's Placeholder), gains its Acceleration of 0.5 su a tick, every tick, and
/// flies at its MaxSpeed of 10 from the 12th tick on; it flies along its facing, its own move, every tick.
#[test]
fn it_speeds_up_by_its_acceleration_to_max_speed_and_keeps_it() {
    let mut flight =
        Flight::new(sparrow(), Vec2::new(100.0, 2048.0), 0.0).going(MoveGoal::Point(Vec2::new(4000.0, 2048.0)));
    let mut speeds = Vec::new();
    for _ in 0..20 {
        let before = flight.pos;
        flight.tick();
        speeds.push(flight.speed());
        assert_eq!(flight.pos - before, flight.mover.momentum);
        assert_eq!(flight.mover.velocity, flight.mover.momentum);
        assert_eq!(flight.mover.locomotion, Locomotion::Moving);
    }
    for (tick, speed) in speeds.iter().enumerate() {
        let want = (4.0 + 0.5 * (tick + 1) as f32).min(10.0);
        assert!(near(*speed, want, 1e-4), "tick {}: {speed}, not {want}", tick + 1);
    }
    assert!(near(speeds[11], 10.0, 1e-4) && speeds[10] < 10.0, "{speeds:?}");
}

/// `systems/units.md > GliderMovement`: "Turn radius is governed by centripetal acceleration (r = v²/a)". At its
/// MaxSpeed of 10 su a tick, turning round for a point behind it, the test glider (a = 1) makes its 180° turn on a
/// radius of 100 su, and the Sparrow (a = 0.5, design change 0355) on one of 200: it comes about 2 r to the side. A
/// tick's turn is a / v radians, on both.
#[test]
fn its_full_speed_turn_has_a_radius_of_its_speed_squared_over_its_centripetal_acceleration() {
    for (glider, radius) in [(test_glider(), 100.0), (sparrow(), 200.0)] {
        // Behind it and a hair to the south, so that it turns south (clockwise on screen) to come about.
        let goal = Vec2::new(-20_000.0, 2048.5);
        let mut flight = Flight::flying(glider, Vec2::new(3000.0, 2048.0), 0.0, 10.0).going(MoveGoal::Point(goal));
        let mut widest: f32 = 0.0;
        for _ in 0..200 {
            let turned = flight.tick();
            assert!(turned <= glider.max_centripetal_acceleration / 10.0 + 1e-6, "{turned}");
            assert!(near(flight.speed(), 10.0, 1e-4), "at full speed: {}", flight.speed());
            widest = widest.max(flight.pos.y - 2048.0);
        }
        assert!(
            near(widest / 2.0, radius, radius * 0.01),
            "a = {}: {widest}",
            glider.max_centripetal_acceleration
        );
        assert!(flight.facing.abs() > PI * 0.99, "it came about: {}", flight.facing);
    }
}

/// `systems/units.md > GliderMovement`: "When idle, the unit circles at a low speed in tight loops"; design change
/// 0354 point 5: on a fixed radius of 1 grid unit. A Sparrow on its orbit circles it clockwise at IdleSpeed, 64 su
/// from it, lap after lap, and never stops; its flight never turns it more than a / v.
#[test]
fn idle_it_circles_its_orbit_at_idle_speed_on_a_radius_of_one_grid_unit() {
    let centre = Vec2::new(1000.0, 1000.0);
    let mut flight = Flight::new(sparrow(), centre + Vec2::new(64.0, 0.0), PI / 2.0);
    flight.mover.orbit = Some(centre);
    let (mut angle, mut laps) = (0.0, 0.0);
    let mut last = flight.pos - centre;
    for tick in 0..1000 {
        let turned = flight.tick();
        assert!(turned <= 0.5 / 4.0 + 1e-6);
        assert!(near(flight.speed(), 4.0, 1e-4), "tick {tick}: at {}", flight.speed());
        let off = flight.pos - centre;
        assert!(
            near(off.length(), 64.0, 0.5),
            "tick {tick}: {} from its orbit",
            off.length()
        );
        // Clockwise on screen: the angle round the orbit grows.
        let step = math::wrap_angle(math::vector_angle(off) - math::vector_angle(last));
        assert!(step > 0.0, "tick {tick}: {step}");
        angle += step;
        last = off;
        if angle >= 2.0 * PI {
            laps += 1.0;
            angle -= 2.0 * PI;
        }
    }
    // A lap of 2π × 64 su at 4 su a tick: 100.5 ticks.
    assert_eq!(laps, 9.0);
    assert_eq!(flight.mover.goal, MoveGoal::None);
}

/// `factions/cults/units/sparrow.md > Flight rules`: "Stop: it circles where it is". Stopped, its orbit unset, a
/// Sparrow flying at full speed takes the point of the map where it is for its orbit, slows by its Deceleration to
/// IdleSpeed, and is on its idle circle round that point within 20 s.
#[test]
fn stopped_it_slows_down_and_circles_where_it_was() {
    let start = Vec2::new(2000.0, 2000.0);
    let mut flight = Flight::flying(sparrow(), start, 0.0, 10.0);
    flight.mover.stop();
    flight.tick();
    assert_eq!(flight.mover.orbit, Some(start));
    assert!(near(flight.speed(), 9.5, 1e-4), "{}", flight.speed());
    for _ in 1..320 {
        flight.tick();
    }
    for _ in 0..200 {
        flight.tick();
        assert!(near(flight.speed(), 4.0, 1e-4));
        assert!(
            near(flight.pos.distance(start), 64.0, 1.0),
            "{}",
            flight.pos.distance(start)
        );
    }
}

/// `systems/controls/command_behaviors.md > MovingToLocation`, GliderMovement exception: "Final step is Locomotion =
/// Moving(idle circle over TargetLocation)". A Sparrow sent to a point arrives once within the idle circle's radius
/// of it, has no goal from then on, and circles the point: the point is its orbit.
#[test]
fn at_its_goal_it_arrives_and_circles_the_point() {
    let goal = Vec2::new(1500.0, 1000.0);
    let mut flight = Flight::new(sparrow(), Vec2::new(500.0, 1000.0), 0.0).going(MoveGoal::Point(goal));
    let mut arrived = None;
    for tick in 1..=400 {
        flight.tick();
        if arrived.is_none() && flight.mover.status == MoveStatus::Arrived {
            arrived = Some(tick);
            assert!(
                flight.pos.distance(goal) <= 64.0 + 10.0,
                "{}",
                flight.pos.distance(goal)
            );
        }
    }
    assert!(arrived.is_some_and(|tick| tick < 120), "{arrived:?}");
    assert_eq!((flight.mover.goal, flight.mover.orbit), (MoveGoal::None, Some(goal)));
    for _ in 0..100 {
        flight.tick();
        assert!(
            near(flight.pos.distance(goal), 64.0, 1.0),
            "{}",
            flight.pos.distance(goal)
        );
    }
}

/// The module's Placeholder: a point inside its turn at the speed it has it slows down for. At full speed a Sparrow's
/// tightest turn has a radius of 200 su, so a point 150 su to its side lies inside it, and it would circle that point
/// for ever 150 su off it. It slows to IdleSpeed, whose turn is 32 su, and arrives.
#[test]
fn a_point_inside_its_turn_it_slows_down_for() {
    let start = Vec2::new(1000.0, 1000.0);
    let goal = start + Vec2::new(0.0, 150.0);
    let mut flight = Flight::flying(sparrow(), start, 0.0, 10.0).going(MoveGoal::Point(goal));
    let mut slowest: f32 = 10.0;
    for _ in 0..160 {
        flight.tick();
        slowest = slowest.min(flight.speed());
    }
    assert_eq!(flight.mover.status, MoveStatus::Arrived);
    assert!(near(slowest, 4.0, 1e-4), "{slowest}");
}

/// `systems/controls/command_behaviors.md > AttackingObject`, GliderMovement exception, and
/// `factions/cults/units/sparrow.md > Flight rules`: "Strafing: after a pass it flies on until the target's **centre**
/// is out of range, then loops back at full speed" (design change 0408). On a run at an object 1000 su east of it, out
/// to its Range of 192 su, a Sparrow passes over it again and again, each pass a fly-past within a few su of it; past
/// it, it holds its course until the target's centre is more than 192 su from its edge; it never slows below MaxSpeed
/// once at it, and never arrives.
#[test]
fn on_a_strafing_run_it_passes_over_its_target_flies_on_out_of_range_and_loops_back() {
    let target = Vec2::new(2000.0, 2000.0);
    let mut flight = Flight::new(sparrow(), Vec2::new(1000.0, 2000.0), 0.0).strafing(target, 192.0);
    let gap = |flight: &Flight| Shape::rect(flight.pos, HALF).distance_to(target);
    let (mut passes, mut closest, mut past) = (0, f32::INFINITY, Vec::new());
    let mut pass = Pass::Toward;
    for tick in 1..=1600 {
        let heading = math::facing_vector(flight.facing);
        flight.tick();
        if tick > 12 {
            assert!(near(flight.speed(), 10.0, 1e-4), "tick {tick}: {}", flight.speed());
        }
        assert_eq!(flight.mover.status, MoveStatus::Moving);
        let now = flight.mover.strafe.expect("on its run").pass;
        if now == Pass::On {
            // Past the target it holds its course.
            assert_eq!(math::facing_vector(flight.facing), heading, "tick {tick}");
        }
        match (pass, now) {
            (Pass::Run, Pass::On) => {
                passes += 1;
                past.push(closest);
                closest = f32::INFINITY;
            }
            // It turns back once out of range.
            (Pass::On, Pass::Toward) => assert!(gap(&flight) > 192.0 - 10.0, "tick {tick}: {}", gap(&flight)),
            _ => {}
        }
        closest = closest.min(flight.pos.distance(target));
        pass = now;
    }
    // About 16 s a pass at 10 su a tick: out 192 su and more, the turn back (2 × 200 su to the side), and in again.
    assert!(passes >= 5, "{passes} passes");
    assert!(past.iter().all(|&closest| closest < 40.0), "{past:?}");
}

/// Design change 0408's point: "The loop is the same size whatever the target". The same run as above at a Sparrow's
/// 40 × 40 and at a building's 3 × 2 tiles (192 × 128 su, its long side across the run): from its second pass on, each
/// loop takes the Sparrow as far from the target's centre, within a few su; edge to edge, as before the change, the
/// building's loop would reach some 44 su farther, its half width along the run less the Sparrow-sized target's.
#[test]
fn a_strafing_run_s_loop_is_the_same_size_whatever_the_target() {
    let target = Vec2::new(2000.0, 2000.0);
    let farthest = |half: Vec2| {
        let start = Vec2::new(1000.0, 2000.0);
        let mut flight = Flight::new(sparrow(), start, 0.0).strafing_at(target, half, 192.0);
        let (mut passes, mut farthest, mut pass) = (0, 0.0_f32, Pass::Toward);
        for _ in 0..1600 {
            flight.tick();
            let now = flight.mover.strafe.expect("on its run").pass;
            if (pass, now) == (Pass::Run, Pass::On) {
                passes += 1;
            }
            if passes >= 2 {
                farthest = farthest.max(flight.pos.distance(target));
            }
            pass = now;
        }
        assert!(passes >= 5, "{passes} passes");
        farthest
    };
    let small = farthest(HALF);
    let building = farthest(Vec2::new(64.0, 96.0));
    assert!(near(small, building, 4.0), "{small} and {building}");
    assert!((400.0..600.0).contains(&small), "{small}");
}

/// (M5-N7) `factions/cults/units/sparrow.md > Flight rules`: "**Takeoff**: it appears over the runway's B end facing C
/// at IdleSpeed and speeds up along the runway". A Sparrow on its takeoff run west along a runway whose end is 236 su
/// ahead, ordered to a point behind it, flies straight on, speeding up by its Acceleration from IdleSpeed, until its
/// centre is past the end, and only then turns for its goal; its run is over there.
#[test]
fn on_its_takeoff_run_it_flies_straight_along_the_runway_then_turns_for_its_goal() {
    let start = Vec2::new(2000.0, 2000.0);
    let end = start - Vec2::new(236.0, 0.0);
    let goal = MoveGoal::Point(Vec2::new(3000.0, 2000.0));
    let mut flight = Flight::flying(sparrow(), start, PI, 4.0).going(goal);
    flight.mover.takeoff = Some(Takeoff { end, along: -Vec2::X });
    let mut ticks = 0;
    loop {
        let before = flight.pos;
        let turned = flight.tick();
        ticks += 1;
        if flight.mover.takeoff.is_none() {
            break;
        }
        assert_eq!(turned, 0.0, "tick {ticks}: it turned on the runway");
        assert!(near(flight.pos.y, start.y, 1e-3), "tick {ticks}: {}", flight.pos);
        let expected = (4.0 + 0.5 * ticks as f32).min(10.0);
        let flown = before.x - flight.pos.x;
        assert!(near(flown, expected, 1e-3), "tick {ticks}: {flown}");
        assert!(ticks < 40, "still on the runway at {}", flight.pos);
    }
    // 4.5 + 5 + ... + 10 su in the first 12 ticks (87 su), 10 a tick after: past the end in tick 27, over in the next.
    assert_eq!(ticks, 28);
    assert!(flight.pos.x < end.x, "{}", flight.pos);
    for _ in 0..20 {
        assert!(flight.tick() > 0.0, "it turns for its goal");
    }
}

/// `systems/units.md > Air Collision`: "Gliders may unavoidably travel a little outside the map edge while turning or
/// circling"; `factions/cults/units/sparrow.md > Flight rules`: "Map edge: it may overshoot the edge as far as its
/// turn needs". A Sparrow circling an orbit 32 su from the map's corner flies past both edges, as far as its idle
/// circle takes it, and back.
#[test]
fn circling_by_the_edge_it_flies_past_it_and_back() {
    let centre = Vec2::new(32.0, 32.0);
    let mut flight = Flight::new(sparrow(), centre + Vec2::new(64.0, 0.0), PI / 2.0);
    flight.mover.orbit = Some(centre);
    let (mut west, mut north): (f32, f32) = (0.0, 0.0);
    for _ in 0..400 {
        flight.tick();
        west = west.min(flight.pos.x);
        north = north.min(flight.pos.y);
        assert!(flight.pos.distance(centre) < 64.5, "{}", flight.pos);
    }
    assert!(west < -30.0 && north < -30.0, "{west}, {north}");
}

/// Phase 5e's aim of a Glider's base is nothing (`movement::turn_rate`): its flight alone turns it.
#[test]
fn nothing_is_left_of_a_glider_s_turn_for_its_base_s_aim() {
    let unit = ObjectKind::Sparrow.def().unit.as_ref().expect("a unit");
    assert_eq!(super::super::turn_rate(&unit.movement), 0.0);
    assert!(near(sparrow().turn_rate(10.0), 0.05, 1e-6));
    assert!(near(sparrow().turn_rate(4.0), 0.125, 1e-6));
}
