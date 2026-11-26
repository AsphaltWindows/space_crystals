//! The side a Recruit leaves its Hall by (`core/entities.md > Structure Type > Exit Doorway`: "Buildings with several
//! exit-eligible sides choose the side by their own rules; a side whose doorway is occupied counts as blocked, and the
//! building pauses only when every eligible doorway is occupied"; **Placeholder (Q0100 point 1)**: "the side facing
//! the rally point (the bottom when there is none), then the next free side clockwise"). The Hall at tile (8, 8)
//! covers x and y 512 to 768: the middles of its sides are (640, 512) north, (768, 640) east, (640, 768) south and
//! (512, 640) west, where a Recruit appears half inside.

use sc_sim::{ObjectId, Target, Vec2};

use super::*;

const HALL_ID: ObjectId = ObjectId(1);
const NORTH: Vec2 = Vec2::new(640.0, 512.0);
const EAST: Vec2 = Vec2::new(768.0, 640.0);
const SOUTH: Vec2 = Vec2::new(640.0, 768.0);
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

/// Whether `spot` is in the doorway of the side whose middle is `side`: on the line from the middle to the centre,
/// from half out to fully inside, the clearance apart (`Sim::doorway_spot`: "as far out as there is room").
fn in_doorway(spot: Vec2, side: Vec2) -> bool {
    let inward = (Vec2::new(640.0, 640.0) - side).normalize();
    let depth = (spot - side).dot(inward);
    (spot - side).perp_dot(inward) == 0.0 && (0.0..=12.5).contains(&depth)
}

/// Without a rally point a Recruit leaves by the bottom side, facing south (**Placeholder (Q0100 point 1)**). It walks
/// straight out and stops there (`core/entities.md > Structure Type > Exit Doorway`, point 2), so the next one appears
/// in the same doorway as far out as there is room, and leaves it as the first steps aside (point 4).
#[test]
fn without_a_rally_point_a_recruit_leaves_by_the_bottom() {
    let mut t = flat(30, 20).spawn(HALL, Some(P1), tile(8, 8)).start();
    let (spot, facing) = next_spot(&mut t);
    assert_eq!(t.sim().tick(), FRAMES);
    assert_eq!(spot, SOUTH);
    assert_eq!(facing, sc_sim::math::vector_angle(Vec2::Y));
    let (second, _) = next_spot(&mut t);
    assert!(in_doorway(second, SOUTH), "{second}");
    assert_eq!(next_spot(&mut t).0.x, SOUTH.x);
}

/// The side facing the rally point comes first: east, north and west in turn as the rally point moves, and of two
/// sides that face it alike, a rally point on the north-east diagonal, the one clockwise of it, east.
#[test]
fn the_side_facing_the_rally_point_comes_first() {
    let mut t = flat(30, 20).spawn(HALL, Some(P1), tile(8, 8)).start();
    let rally = |point: Vec2| production(&[HALL_ID], ProductionCommand::SetRally(Target::Point(point)));
    let cases = [
        (Vec2::new(1200.0, 700.0), EAST),
        (Vec2::new(600.0, 100.0), NORTH),
        (Vec2::new(100.0, 600.0), WEST),
        (Vec2::new(1040.0, 240.0), EAST),
        (Vec2::new(500.0, 1100.0), SOUTH),
    ];
    for (point, side) in cases {
        let tick = t.sim().tick() + 1;
        t.at(tick, P1, rally(point));
        assert_eq!(next_spot(&mut t).0, side, "rally point {point}");
    }
}

/// A side whose doorway is occupied is passed over for the next free one clockwise. Water right under the Hall keeps
/// its first Recruit in the south doorway for good (`core/entities.md > Structure Type > Exit Doorway`, point 5:
/// "terrain right against the exit side leaves the first unit stuck in the doorway"); the next ones leave by the
/// west side, the next clockwise from the bottom (the third as far out as the second, standing outside, leaves room).
#[test]
fn an_occupied_side_sends_the_next_recruit_clockwise() {
    let map = with_water(30, 20, |x, y| y == 12 && (8..=11).contains(&x));
    let mut t = cults(&map).spawn(HALL, Some(P1), tile(8, 8)).start();
    let (first, _) = next_spot(&mut t);
    assert!(
        first.y < SOUTH.y && (first.x, SOUTH.y - first.y) == (SOUTH.x, 12.0),
        "fully inside: {first}"
    );
    assert_eq!(next_spot(&mut t).0, WEST);
    let (third, _) = next_spot(&mut t);
    assert!(in_doorway(third, WEST), "{third}");
    let stuck = made(&t, HALL_ID)[0].1;
    assert_eq!(t.pos(stuck.0), first, "still in the south doorway");
}

/// The Hall pauses only when every side's doorway is occupied (`core/entities.md > Structure Type > Exit Doorway`):
/// Water all round it keeps its first four Recruits in its four doorways, taken south, west, north, east, and the
/// fifth waits inside, finished.
#[test]
fn the_hall_pauses_only_when_every_side_is_occupied() {
    let ring =
        |x: i32, y: i32| (7..=12).contains(&x) && (7..=12).contains(&y) && [7, 12].iter().any(|&e| x == e || y == e);
    let mut t = cults(&with_water(30, 20, ring))
        .spawn(HALL, Some(P1), tile(8, 8))
        .start();
    // 80 tiles of its area are Recruitable: 240 ticks a Recruit.
    t.run(1);
    assert_eq!(hall_state(&t, HALL_ID).claimed, 80);
    let sides: Vec<Vec2> = (0..4).map(|_| next_spot(&mut t).0).collect();
    let inside = |side: Vec2| side + (Vec2::new(640.0, 640.0) - side).normalize() * 12.0;
    assert_eq!(sides, [SOUTH, WEST, NORTH, EAST].map(inside));
    t.run_until(5 * 240 + 100);
    assert_eq!(made(&t, HALL_ID).len(), 4);
    let status = t.sim().production_status(HALL_ID).expect("in production");
    assert!(status.waiting && status.progress == status.total, "{status:?}");
}
