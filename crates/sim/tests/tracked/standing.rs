//! Design change 0440's Verification (`systems/units.md > UnitCollision > Ground Collision`, "Vehicles"): "the 'idle
//! vehicle doesn't yield' rule covers WheeledVehicle, TrackedVehicle and the DrillUnit above ground (treads and wheels
//! can't step sideways). The HoverVehicle still yields". Own Militants walk east along a lane, through the spot where
//! an idle unit of their owner stands.

use sc_sim::{MoveStatus, ObjectKind, SpawnAt, Vec2};

use super::{HAND, P1, done, facing, flat, move_to};

const MILITANT: ObjectKind = ObjectKind::Militant;
const DRONE: ObjectKind = ObjectKind::Drone;
/// Where the idle unit stands, on the Militants' lane.
const STANDS: Vec2 = Vec2::new(640.0, 320.0);

/// Three Militants of player 1 abreast on the lane west of [`STANDS`] (ids 2 to 4; the idle unit is 1), sent east past
/// it as a group: each to its own spot, the magic box keeping their row. Steps until all three are done, and returns
/// whether the idle unit ever moved or turned.
fn walk_through(kind: ObjectKind) -> (sc_sim::testing::TestSim, bool) {
    let mut builder = flat(20, 10).spawn(kind, Some(P1), SpawnAt::Point(STANDS));
    for dy in [-30.0, 0.0, 30.0] {
        builder = builder.spawn(MILITANT, Some(P1), SpawnAt::Point(Vec2::new(200.0, STANDS.y + dy)));
    }
    let mut t = builder.start();
    let facing_was = facing(&t, 1);
    move_to(&mut t, 1, &[2, 3, 4], Vec2::new(1100.0, STANDS.y));
    let mut budged = false;
    while (2..=4).any(|id| !done(&t, id)) {
        t.step();
        budged |= t.pos(1) != STANDS || facing(&t, 1) != facing_was;
        assert!(t.sim().tick() < 400, "{}", t.ascii(Some(STANDS)));
    }
    for id in 2..=4 {
        assert_eq!(
            t.status(id),
            MoveStatus::Arrived,
            "Militant {id}\n{}",
            t.ascii(Some(STANDS))
        );
        assert!(t.pos(id).x > 1000.0, "Militant {id} got past it: {}", t.pos(id));
    }
    (t, budged)
}

/// "An idle Hand of Heaven in a lane: own infantry moving through go round it; it doesn't budge." The Militants walk
/// into it and ask it to make way, as they would any idle unit of theirs; a TrackedVehicle doesn't yield, so they go
/// round it, and it never moves nor turns.
#[test]
fn an_idle_hand_of_heaven_in_a_lane_stands_and_own_infantry_go_round_it() {
    let (t, budged) = walk_through(HAND);
    assert!(!budged, "it stood: at {}", t.pos(1));
}

/// "An idle Colonists Drone in a lane: it steps aside for own units." The same walk past an idle Drone, a
/// HoverVehicle, which yields: it steps aside. (`gathering_pass_through.rs` has a gathering Drone of its owner bump
/// into one too.)
#[test]
fn an_idle_drone_in_a_lane_steps_aside_for_own_units() {
    let (t, budged) = walk_through(DRONE);
    assert!(budged, "it stepped aside: at {}", t.pos(1));
}
