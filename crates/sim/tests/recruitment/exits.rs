//! The side a Recruit leaves its Hall by (design change 0342, answering Q0100: "The Hall has one exit side. It becomes
//! AAAB with the exit on B [...] Recruits come out of it one at a time through the Exit Doorway"; `core/entities.md >
//! Structure Type > Exit Doorway`). The Hall at tile (8, 8), unturned, covers x and y 512 to 768: its side B is the
//! west side (`sc_sim::sides`: the letters run clockwise from the top), whose middle is (512, 640), where a Recruit
//! appears half inside, facing west.

use sc_sim::{ObjectId, Target, Vec2};

use super::*;

const HALL_ID: ObjectId = ObjectId(1);
/// The middle of the Hall's side B.
const WEST: Vec2 = Vec2::new(512.0, 640.0);

/// Steps until the Hall's next Recruit comes out, and returns where it appeared (half inside, its centre on the
/// side's middle) and which way it faced.
fn next_spot(t: &mut TestSim) -> (Vec2, f32) {
    let before = made(t, HALL_ID).len();
    for _ in 0..2000 {
        t.step();
        if let Some(&(_, unit)) = made(t, HALL_ID).get(before) {
            let o = t.sim().object(unit).expect("alive");
            return (o.pos, o.facing);
        }
    }
    panic!("no Recruit\n{}", t.ascii(None));
}

/// Whether `facing` is west, out of side B.
fn faces_west(facing: f32) -> bool {
    Vec2::from_angle(facing).distance(-Vec2::X) < 1e-5
}

/// Whether `spot` is in side B's doorway: on the line from its middle to the centre, from half out to fully inside,
/// the clearance apart (`Sim::doorway_spot`: "as far out as there is room").
fn in_doorway(spot: Vec2) -> bool {
    let inward = (Vec2::new(640.0, 640.0) - WEST).normalize();
    let depth = (spot - WEST).dot(inward);
    (spot - WEST).perp_dot(inward) == 0.0 && (0.0..=12.5).contains(&depth)
}

/// Design change 0342's Verification, "Recruits always appear on side B": without a rally point the first leaves by
/// it facing west, in tick 192, walks straight out and stops there (`core/entities.md > Structure Type > Exit
/// Doorway`, point 2), so the next one appears in the same doorway as far out as there is room.
#[test]
fn without_a_rally_point_a_recruit_leaves_by_side_b() {
    let mut t = flat(30, 20).spawn(HALL, Some(P1), tile(8, 8)).start();
    let (spot, facing) = next_spot(&mut t);
    assert_eq!(t.sim().tick(), FRAMES);
    assert_eq!(spot, WEST);
    assert!(faces_west(facing), "{facing}");
    let (second, _) = next_spot(&mut t);
    assert!(in_doorway(second), "{second}");
}

/// Design change 0342's Verification, "Recruits always appear on side B", whatever side the rally point is on: east,
/// north, south or west of the Hall in turn, each Recruit comes out of side B's doorway facing west, and walks round
/// to it.
#[test]
fn recruits_come_out_of_side_b_wherever_the_rally_point_is() {
    let mut t = flat(30, 20).spawn(HALL, Some(P1), tile(8, 8)).start();
    let rally = |point: Vec2| production(&[HALL_ID], ProductionCommand::SetRally(Target::Point(point)));
    let points = [
        Vec2::new(1200.0, 700.0),
        Vec2::new(600.0, 100.0),
        Vec2::new(500.0, 1100.0),
        Vec2::new(100.0, 600.0),
    ];
    for point in points {
        let tick = t.sim().tick() + 1;
        t.at(tick, P1, rally(point));
        let (spot, facing) = next_spot(&mut t);
        assert!(in_doorway(spot), "rally point {point}: {spot}");
        assert!(faces_west(facing), "rally point {point}: {facing}");
    }
}

/// The Hall pauses while its one doorway is occupied (`core/entities.md > Structure Type > Exit Doorway`, point 3:
/// "the next unit can only appear once the doorway is free"; design change 0342: "one at a time through the Exit
/// Doorway"). Water right against side B keeps its first Recruit in the doorway for good, fully inside (point 5:
/// "terrain right against the exit side leaves the first unit stuck in the doorway"), and the second waits inside,
/// finished: no other side takes it.
#[test]
fn the_hall_pauses_while_its_doorway_is_occupied() {
    let map = with_water(30, 20, |x, y| x == 7 && (8..=11).contains(&y));
    let mut t = cults(&map).spawn(HALL, Some(P1), tile(8, 8)).start();
    t.run(1);
    // 96 tiles of its area are Recruitable: 200 ticks a Recruit.
    assert_eq!(hall_state(&t, HALL_ID).claimed, 96);
    let (first, _) = next_spot(&mut t);
    assert_eq!((first.y, first.x - WEST.x), (WEST.y, 12.0), "fully inside: {first}");
    t.run(3 * 200);
    assert_eq!(made(&t, HALL_ID).len(), 1, "one at a time");
    let status = t.sim().production_status(HALL_ID).expect("in production");
    assert!(status.waiting && status.progress == status.total, "{status:?}");
    let stuck = made(&t, HALL_ID)[0].1;
    assert_eq!(t.pos(stuck.0), first, "still in the doorway");
}
