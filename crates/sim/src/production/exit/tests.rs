use super::*;
use crate::defs::ObjectKind;
use crate::object::Rotation;
use crate::scenario::{BuildingOrientation, SpawnAt};
use crate::sim::Sim;
use crate::testing::ScenarioBuilder;

const HALF: Vec2 = Vec2::splat(12.0);

/// A `kind` placed at tile (2, 1) of an empty map, as turned.
pub(super) fn building(kind: ObjectKind, rotation: Rotation, flip_h: bool) -> Object {
    let mut scenario = ScenarioBuilder::new("........\n........\n........\n........\n........\n........").build();
    let mut spawn = crate::scenario::Spawn::new(kind, None, SpawnAt::Tile(TilePos::new(2, 1)));
    spawn.orientation = BuildingOrientation {
        rotation,
        flip_h,
        flip_v: false,
    };
    scenario.map.spawns.push(spawn);
    Sim::new(&scenario).objects().next().unwrap().clone()
}

fn barracks(rotation: Rotation, flip_h: bool) -> Doorway {
    Doorway::of(&building(ObjectKind::Barracks, rotation, flip_h)).unwrap()
}

/// The one door of a Barracks.
fn side_b(rotation: Rotation, flip_h: bool) -> Door {
    let doors: Vec<Door> = barracks(rotation, flip_h).doors().collect();
    assert_eq!(doors.len(), 1, "{rotation:?} {flip_h}");
    doors[0]
}

/// Every orientation: the four rotations, unflipped and flipped.
pub(super) fn orientations() -> impl Iterator<Item = (Rotation, bool)> {
    [false, true]
        .into_iter()
        .flat_map(|flip_h| [Rotation::R0, Rotation::R90, Rotation::R180, Rotation::R270].map(|r| (r, flip_h)))
}

/// Design change 0079 and `> Exit Doorway`: the doorway is in the middle of side B, wherever that side faces.
#[test]
fn the_doorway_is_the_middle_of_side_b_in_all_eight_orientations() {
    // Unturned: 3 wide and 2 tall at tile (2, 1), so x 128..320 and y 64..192; B faces east.
    let cases = [
        (Rotation::R0, false, Vec2::new(320.0, 128.0), Vec2::X),
        (Rotation::R180, false, Vec2::new(128.0, 128.0), -Vec2::X),
        (Rotation::R0, true, Vec2::new(128.0, 128.0), -Vec2::X),
        (Rotation::R180, true, Vec2::new(320.0, 128.0), Vec2::X),
        // Turned on its side: 2 wide and 3 tall, so x 128..256 and y 64..256.
        (Rotation::R90, false, Vec2::new(192.0, 256.0), Vec2::Y),
        (Rotation::R270, false, Vec2::new(192.0, 64.0), -Vec2::Y),
        (Rotation::R90, true, Vec2::new(192.0, 64.0), -Vec2::Y),
        (Rotation::R270, true, Vec2::new(192.0, 256.0), Vec2::Y),
    ];
    for (rotation, flip_h, middle, outward) in cases {
        let doorway = barracks(rotation, flip_h);
        let door = side_b(rotation, flip_h);
        assert_eq!(
            (door.middle(), door.outward()),
            (middle, outward),
            "{rotation:?} {flip_h}"
        );
        assert_eq!(door.spot(0.0), middle);
        assert!(door.holds(door.spot(0.0), HALF), "half inside");
        assert!(door.holds(door.spot(-12.0), HALF), "fully inside");
        assert!(!door.holds(door.spot(12.0), HALF), "touching it from outside");
        assert_eq!(door.outside(door.spot(-5.0), HALF), door.spot(12.0));
        assert_eq!(doorway.outside(door.spot(-5.0), HALF), door.spot(12.0));
        assert_eq!((door.out_of(door.spot(-5.0)), door.out_of(door.spot(7.0))), (-5.0, 7.0));
        // Wherever a unit is, it is in the one door.
        for pos in [door.spot(0.0), Vec2::new(200.0, 100.0), Vec2::new(-50.0, 900.0)] {
            assert_eq!(doorway.door_at(pos), door);
        }
    }
}

#[test]
fn only_structures_that_produce_have_a_doorway() {
    let sim = ScenarioBuilder::new("....\n....\n....")
        .spawn(ObjectKind::Generator, None, SpawnAt::Tile(TilePos::new(0, 0)))
        .spawn(ObjectKind::Peacekeeper, None, SpawnAt::Point(Vec2::new(200.0, 160.0)))
        .start();
    assert!(sim.sim().objects().all(|o| Doorway::of(o).is_none()));
}

/// The exception's rule: out, never deeper in, and inside only within the exit side's span.
#[test]
fn a_unit_in_the_doorway_may_only_come_out() {
    let (doorway, door) = (barracks(Rotation::R0, false), side_b(Rotation::R0, false));
    let at = door.spot(-4.0);
    // With one door, the doorway's rule is the door's.
    let by_door = |from, to, half| door.allows(from, to, half);
    let by_doorway = |from, to, half| doorway.allows(from, to, half);
    let rules: [&dyn Fn(Vec2, Vec2, Vec2) -> bool; 2] = [&by_door, &by_doorway];
    for allows in rules {
        assert!(allows(at, at + Vec2::new(3.0, 0.0), HALF), "out");
        assert!(allows(at, at + Vec2::new(40.0, 30.0), HALF), "clear of it");
        assert!(!allows(at, at + Vec2::new(-0.5, 0.0), HALF), "deeper in");
        assert!(allows(at, at + Vec2::new(0.0, 5.0), HALF), "along the side, within it");
        assert!(allows(at, at + Vec2::new(2.0, -5.0), HALF));
        // The side spans y 64..192: a box 12 high may not reach past its ends while inside.
        let corner = Vec2::new(at.x, 64.0 + 12.0);
        assert!(
            !allows(corner, corner + Vec2::new(0.0, -1.0), HALF),
            "out through side A"
        );
        assert!(allows(corner, corner + Vec2::new(1.0, 0.0), HALF));
        // A box wider than the side may still go straight out.
        let wide = Vec2::new(12.0, 80.0);
        assert!(allows(at, at + Vec2::new(3.0, 0.0), wide) && !allows(at, at + Vec2::new(3.0, 1.0), wide));
    }
}

/// The ways out: straight ahead first, then in front of each tile of the exit side.
#[test]
fn the_ways_out_touch_the_exit_side_from_outside() {
    let door = side_b(Rotation::R0, false);
    let ways = door.ways_out(door.spot(-12.0) + Vec2::new(0.0, 7.0), HALF);
    assert_eq!(
        ways,
        [Vec2::new(332.0, 135.0), Vec2::new(332.0, 96.0), Vec2::new(332.0, 160.0)]
    );
    assert_eq!(
        barracks(Rotation::R0, false).ways_out(door.spot(-12.0) + Vec2::new(0.0, 7.0), HALF),
        ways
    );
    assert!(ways.iter().all(|&way| !door.holds(way, HALF)));
    let south = side_b(Rotation::R90, false);
    assert_eq!(
        south.ways_out(south.spot(0.0), HALF),
        [
            Vec2::new(192.0, 268.0),
            Vec2::new(160.0, 268.0),
            Vec2::new(224.0, 268.0)
        ]
    );
}

/// What stands in front of a doorway leaves a unit appearing there less room; what stands behind it, as deep as the
/// unit fully inside or beyond the building's far side, doesn't.
#[test]
fn only_what_stands_in_front_of_a_door_leaves_less_room() {
    // The Barracks unturned: x 128..320, y 64..192, its door the middle of the east side, (320, 128).
    let door = side_b(Rotation::R0, false);
    assert_eq!(
        door.room_before(HALF, Vec2::new(332.0, 128.0), HALF),
        Some(-12.0),
        "touching the side"
    );
    assert_eq!(
        door.room_before(HALF, Vec2::new(345.0, 128.0), HALF),
        None,
        "room to stand half inside"
    );
    assert_eq!(
        door.room_before(HALF, Vec2::new(332.0, 152.0), HALF),
        None,
        "beside the doorway"
    );
    assert_eq!(
        door.room_before(HALF, Vec2::new(100.0, 128.0), HALF),
        None,
        "behind the building"
    );
    assert_eq!(
        door.room_before(HALF, Vec2::new(284.0, 128.0), HALF),
        None,
        "as deep as the unit fully inside"
    );
    assert_eq!(
        door.room_before(HALF, Vec2::new(285.0, 128.0), HALF),
        Some(-59.0),
        "less deep"
    );
}
