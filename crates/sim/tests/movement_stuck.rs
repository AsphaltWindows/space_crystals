//! Whose progress a waiting unit shares (`docs/movement.md > 6.5`; slice M3-H2, `movement::stuck::leads`): a unit
//! blocked by one that is on its way takes over that unit's progress clock, so a queue isn't stuck while its head
//! moves. A new leg sets a unit's own clock, and **that is no progress**: a blocked unit whose legs begin again and
//! again must not keep the units behind it from ever looking for a way round.

use sc_sim::testing::{ScenarioBuilder, TestSim};
const MAX_REPATHS: u8 = sc_sim::tuning::movement::BUILTIN.max_repaths;
use sc_sim::{MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, SpawnAt, Target, Tick, TilePos, Vec2};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su, 4 su per tick
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su, 5 su per tick

/// Two corridors one tile wide, 12 tiles long, joined at both ends: a ring.
const RING: &str = "
    ................
    ..~~~~~~~~~~~~..
    ................
";

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

/// The south corridor of [`RING`] with a plug in it: an enemy Guard that never moves (3), a Guard of player 1 that
/// walks into it on its way east (1), and another Guard of player 1 behind that one, on its way to the corridor's
/// east end (2). A Guard fills the corridor: nothing passes beside it. The other way round the ring is free.
fn plugged() -> TestSim {
    let mut t = ScenarioBuilder::new(RING)
        .spawn(GUARD, Some(P1), SpawnAt::Point(centre(5, 2)))
        .spawn(GUARD, Some(P1), SpawnAt::Point(centre(2, 2)))
        .spawn(GUARD, Some(P2), SpawnAt::Point(centre(8, 2)))
        .start();
    t.without_auto_targeting();
    t
}

fn move_to(t: &mut TestSim, tick: Tick, unit: u32, to: Vec2) {
    t.order_at(tick, P1, &[ObjectId(unit)], OrderKind::Move(Target::Point(to)), false);
}

/// The plug's own Guard is sent east again every second (a player clicking on a blocked unit; a behaviour that
/// begins its leg again), so it never gets to look for another route itself, and its clock is never older than 16
/// ticks. The Guard behind it doesn't share that clock: it is stuck 24 ticks after it stopped getting on,
/// looks for another route, and takes the way round the ring.
#[test]
fn a_unit_behind_one_whose_legs_begin_again_still_finds_its_way_round() {
    let mut t = plugged();
    let goal = centre(13, 2);
    move_to(&mut t, 1, 2, goal);
    for tick in (1..1000).step_by(16) {
        move_to(&mut t, tick, 1, centre(12, 2));
    }
    let mut north = false;
    for _ in 0..1000 {
        t.step();
        north |= t.pos(2).y < 64.0;
        if t.status(2) != MoveStatus::Moving {
            break;
        }
    }
    assert_eq!(
        (t.status(2), t.pos(2)),
        (MoveStatus::Arrived, goal),
        "\n{}",
        t.ascii(None)
    );
    assert!(north, "by the north corridor");
    assert!(t.mover(2).repaths_left < MAX_REPATHS, "after looking for another route");
    // The Guard is still where it was stopped, still being sent.
    assert_eq!(t.status(1), MoveStatus::Moving);
    assert!(t.pos(1).x < 520.0, "{}", t.pos(1));
}

/// The rule it must not break: behind a unit that **is** getting on, a unit isn't stuck, however slowly they go. A
/// Peacekeeper (4 su per tick) leads a Guard (5 su per tick) down the corridor, and the Guard, sent after it again
/// every second, follows it to the end without ever looking for another route.
#[test]
fn a_unit_behind_one_that_gets_on_shares_its_progress() {
    let mut t = ScenarioBuilder::new(RING)
        .spawn(PK, Some(P1), SpawnAt::Point(centre(3, 2)))
        .spawn(GUARD, Some(P1), SpawnAt::Point(centre(2, 2)))
        .start();
    move_to(&mut t, 1, 1, centre(13, 2));
    move_to(&mut t, 1, 2, centre(12, 2));
    for _ in 0..400 {
        t.step();
        assert_eq!(t.mover(2).repaths_left, MAX_REPATHS, "tick {}", t.sim().tick());
        assert!(t.pos(2).y > 128.0, "it stays in the corridor");
    }
    assert_eq!(t.status(1), MoveStatus::Arrived);
    assert_eq!(t.status(2), MoveStatus::Arrived);
}
