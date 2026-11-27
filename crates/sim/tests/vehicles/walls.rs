//! Acceptance, **Walls** and **Terrain** (slice M5-V2): "pressed against a wall at an angle, it creeps along and scrapes
//! free (0035)"; "Rugged tiles are impassable (`NavClass::Ground`)". `systems/units.md > UnitCollision > Collision
//! Response (Un-sticking)`: "a unit pressed against an obstacle restarts from 0 every frame, so it moves at
//! single-frame-acceleration speed. It creeps, but it keeps making its turn while the blocked part of each step is
//! discarded. It gradually scrapes around the obstacle until it is free, then accelerates normally. This matters most
//! for units that can't turn in place (WheeledVehicle: turning needs forward motion), which would otherwise jam
//! against walls." The invariants (I2: never into a wall) hold every tick.

use sc_sim::DebugCommand;

use super::*;

/// The fastest a unit pressed against a wall moves in a tick: from rest, one tick's acceleration, and what it backs
/// off by as it turns, its corners' sweep (`movement/collide/turn.rs`, `World::back_off`).
const CREEP: f32 = 1.0 + 1.0 / RADIUS * 35.0;

/// Design change 0035's Verification: "Drive a Raptor nose-first into a wall, then order it to a point behind it to
/// the right: it creeps, turns while scraping along the wall, then drives off at normal acceleration." The Seeker
/// stands with its nose on a wall to its east (any turn about its centre would sweep a front corner into it), and is
/// sent to a point behind it to its right: it creeps and turns right, its corner on the wall, then drives off at
/// full speed and arrives.
#[test]
fn nose_first_against_a_wall_a_seeker_scrapes_round_and_drives_off() {
    // A wall of Mountain at column 8: x from 512 su.
    let row = format!("{}M{}", ".".repeat(8), ".".repeat(3));
    let map = vec![row; 12].join("\n");
    let nose_on_wall = Vec2::new(512.0 - 28.0, centre(0, 4).y);
    let mut t = ScenarioBuilder::new(&map)
        .spawn(SEEKER, Some(P1), SpawnAt::Point(nose_on_wall))
        .start();
    let goal = nose_on_wall + Vec2::new(-150.0, 300.0);
    move_to(&mut t, 1, &[1], goal);
    let mut moves = Vec::new();
    let mut turned = 0.0;
    while !done(&t, 1) {
        let (d, turn) = step(&mut t, 1);
        assert!(turn >= -1e-4, "tick {}: it turns right, {turn}", t.sim().tick());
        turned += turn;
        moves.push((d.length(), turned));
        assert!(moves.len() < 400, "{:?}\n{}", t.mover(1), t.ascii(None));
    }
    assert_eq!(t.status(1), MoveStatus::Arrived, "{}", t.ascii(None));
    // It creeps as it scrapes: until it has turned away from the wall, no tick's move is more than a creep.
    let free_from = moves.iter().position(|&(_, turned)| turned > 0.7).expect("it turned");
    assert!(free_from > 10, "it scraped for {free_from} ticks");
    assert!(moves[..free_from].iter().all(|&(d, _)| d <= CREEP), "{moves:?}");
    // Then it drives off at normal acceleration.
    let fastest = moves.iter().map(|&(d, _)| d).fold(0.0, f32::max);
    assert!(fastest >= 6.0, "{moves:?}");
}

/// Pressed against a wall at an angle: a Seeker that turned about 30° to its right as it set off, and stopped, is put
/// (`DebugCommand::SetPosition`) with its front right corner 0.5 su above a wall that runs east, and sent east along
/// the wall. It drives into the wall at its angle; pressed against it, it creeps along it at single-tick
/// acceleration, turning left as it scrapes, never into it (I2), until it runs along it free and accelerates
/// normally to its goal.
#[test]
fn pressed_against_a_wall_at_an_angle_a_seeker_scrapes_along_it_and_gets_free() {
    // A wall of Mountain along row 9: from y = 576 su.
    let map: Vec<String> = (0..12)
        .map(|y| if y == 9 { "M".repeat(30) } else { ".".repeat(30) })
        .collect();
    let mut t = ScenarioBuilder::new(&map.join("\n"))
        .spawn(SEEKER, Some(P1), SpawnAt::Point(Vec2::new(100.0, 50.0)))
        .start();
    move_to(&mut t, 1, &[1], Vec2::new(800.0, 530.0));
    while facing(&t, 1) < 0.5 {
        t.step();
        assert!(t.sim().tick() < 100, "{:?}", t.mover(1));
    }
    let tick = t.sim().tick() + 1;
    t.order_at(tick, P1, &[ObjectId(1)], OrderKind::Stop, false).run(12);
    assert_eq!(t.mover(1).momentum, Vec2::ZERO);
    let angle = facing(&t, 1);
    // Its lowest point, its front right corner.
    let below = 28.0 * math::sin(angle) + 20.0 * math::cos(angle);
    let at = Vec2::new(400.0, 576.0 - below - 0.5);
    let tick = t.sim().tick() + 1;
    t.debug_at(
        tick,
        DebugCommand::SetPosition {
            id: ObjectId(1),
            pos: at,
        },
    )
    .step();
    assert_eq!(t.pos(1), at);
    let goal = Vec2::new(1500.0, 576.0 - 30.0);
    move_to(&mut t, tick + 1, &[1], goal);
    let mut pressed = Vec::new();
    let mut fastest: f32 = 0.0;
    while t.sim().tick() <= tick || !done(&t, 1) {
        let carried = t.mover(1).momentum;
        let (d, turn) = step(&mut t, 1);
        let tick = t.sim().tick();
        assert!(turn <= 1e-4, "tick {tick}: it turns left, along the wall, {turn}");
        if t.mover(1).momentum == Vec2::ZERO && d != Vec2::ZERO && t.status(1) == MoveStatus::Moving {
            // Pressed against the wall: the tick it meets it, at the speed it had; from the next on, from rest.
            assert!(
                carried != Vec2::ZERO || d.length() <= CREEP,
                "tick {tick}: it creeps, {d}"
            );
            assert!(d.x > 0.0, "tick {tick}: along the wall, {d}");
            pressed.push(facing(&t, 1));
        } else if !pressed.is_empty() {
            fastest = fastest.max(d.length());
        }
        assert!(tick < 600, "{:?}\n{}", t.mover(1), t.ascii(None));
    }
    assert_eq!(t.status(1), MoveStatus::Arrived, "{}", t.ascii(None));
    assert!(pressed.len() > 10, "pressed for {} ticks", pressed.len());
    assert!(pressed[0] > 0.3, "at an angle: {pressed:?}");
    assert!(fastest >= TOP - 0.01, "then free at full speed: {fastest}");
}

/// "Rugged tiles are impassable": the WheeledVehicle's RuggedTerrain is false (`systems/units.md > WheeledVehicle`),
/// so its grid is `NavClass::Ground`, whose Rugged tiles are blocked. A band of Rugged terrain between the Seeker and
/// its goal, open at its south end: the Seeker drives round through the opening, never onto the band (I2).
#[test]
fn a_seeker_goes_round_rugged_terrain() {
    let map: Vec<String> = (0..12)
        .map(|y| {
            (0..20)
                .map(|x| if (9..11).contains(&x) && y < 9 { 'r' } else { '.' })
                .collect()
        })
        .collect();
    let mut t = ScenarioBuilder::new(&map.join("\n"))
        .spawn(SEEKER, Some(P1), SpawnAt::Point(centre(3, 3)))
        .start();
    let goal = centre(16, 3);
    move_to(&mut t, 1, &[1], goal);
    let mut deepest: f32 = 0.0;
    while !done(&t, 1) {
        t.step();
        if (9.0 * 64.0..11.0 * 64.0).contains(&t.pos(1).x) {
            deepest = deepest.max(t.pos(1).y);
        }
        assert!(t.sim().tick() < 800, "{:?}\n{}", t.mover(1), t.ascii(None));
    }
    assert_eq!(t.status(1), MoveStatus::Arrived);
    assert!(deepest > 9.0 * 64.0, "it crossed through the opening, at y {deepest}");
}
