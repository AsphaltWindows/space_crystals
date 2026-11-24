use super::*;
use crate::defs::ObjectKind;
use crate::object::Rotation;
use crate::scenario::{BuildingOrientation, SpawnAt};
use crate::sim::Sim;
use crate::testing::ScenarioBuilder;

const HALF: Vec2 = Vec2::splat(12.0);

/// A `kind` placed at tile (2, 1) of an empty map, as turned.
fn building(kind: ObjectKind, rotation: Rotation, flip_h: bool) -> Object {
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
fn orientations() -> impl Iterator<Item = (Rotation, bool)> {
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

/// `factions/colonists/structures.md > Robo Portal`: "a warped-in Drone may emerge from any of the 4 sides". Its
/// doors are in the middle of each side, top, right, bottom, left, however it is turned (AAAA), and a unit in one
/// of them is in that one.
#[test]
fn the_robo_portal_has_a_door_in_the_middle_of_each_side() {
    // 4 × 4 at tile (2, 1): x 128..384, y 64..320.
    let middles = [
        (Vec2::new(256.0, 64.0), -Vec2::Y),
        (Vec2::new(384.0, 192.0), Vec2::X),
        (Vec2::new(256.0, 320.0), Vec2::Y),
        (Vec2::new(128.0, 192.0), -Vec2::X),
    ];
    for (rotation, flip_h) in orientations() {
        let doorway = Doorway::of(&building(ObjectKind::RoboPortal, rotation, flip_h)).unwrap();
        let doors: Vec<Door> = doorway.doors().collect();
        assert_eq!(
            doors.iter().map(|d| (d.middle(), d.outward())).collect::<Vec<_>>(),
            middles,
            "{rotation:?} {flip_h}"
        );
        for door in doors {
            for out in [0.0, -18.0, -18.03125, 5.0] {
                assert_eq!(doorway.door_at(door.spot(out)), door, "{rotation:?} {flip_h} {out}");
            }
        }
    }
}

/// `factions/colonists/structures.md > Bio Portal > Exit Behavior`: "two exit-eligible sides (the "A" sides, per
/// its AABB symmetry)", wherever they face once it is placed (design change 0079: unturned, A is top and right).
#[test]
fn the_bio_portal_has_doors_on_its_two_a_sides_in_all_eight_orientations() {
    use crate::sides::SideLetter;
    let (n, e, s, w) = (-Vec2::Y, Vec2::X, Vec2::Y, -Vec2::X);
    let cases = [
        (Rotation::R0, false, [n, e]),
        (Rotation::R90, false, [e, s]),
        (Rotation::R180, false, [s, w]),
        (Rotation::R270, false, [n, w]),
        (Rotation::R0, true, [n, w]),
        (Rotation::R90, true, [n, e]),
        (Rotation::R180, true, [e, s]),
        (Rotation::R270, true, [s, w]),
    ];
    for (rotation, flip_h, outwards) in cases {
        let portal = building(ObjectKind::BioPortal, rotation, flip_h);
        let doorway = Doorway::of(&portal).unwrap();
        let doors: Vec<Door> = doorway.doors().collect();
        assert_eq!(
            doors.iter().map(Door::outward).collect::<Vec<_>>(),
            outwards,
            "{rotation:?} {flip_h}"
        );
        // The same sides as the lettering says.
        let letters = sides(ObjectKind::BioPortal, portal.placement.as_ref().unwrap().orientation()).unwrap();
        let a_sides: Vec<Vec2> = (0..4)
            .filter(|&i| letters[i] == SideLetter::A)
            .map(|i| Vec2::new(DIRECTIONS[i].0 as f32, DIRECTIONS[i].1 as f32))
            .collect();
        assert_eq!(a_sides, outwards, "{rotation:?} {flip_h}");
    }
}

/// A unit may not leave its door for the next one while it overlaps the footprint: deep in the corner between
/// two exit sides it stops, and every way out of its own door stays open.
#[test]
fn a_unit_stays_in_its_door_in_a_corner_between_two() {
    // The Bio Portal unturned, 3 × 3 at tile (2, 1): x 128..320, y 64..256, doors north and east.
    let doorway = Doorway::of(&building(ObjectKind::BioPortal, Rotation::R0, false)).unwrap();
    let [north, east]: [Door; 2] = doorway.doors().collect::<Vec<_>>().try_into().unwrap();
    let half = Vec2::splat(18.0);
    // In the east door, a little deeper than fully inside, near the north end of the side.
    let at = Vec2::new(320.0 - 18.0 - 1.0 / 32.0, 83.0);
    assert_eq!(doorway.door_at(at), east);
    // Up to the end of the side its centre would be less deep behind the north side than the east one.
    let corner = Vec2::new(at.x, 82.0);
    assert_eq!(doorway.door_at(corner), north);
    assert!(east.allows(at, corner, half) && !doorway.allows(at, corner, half));
    // Out, and along the side away from the corner, as ever.
    for to in [
        at + Vec2::new(0.5, 0.0),
        at + Vec2::new(1.0, -1.0),
        at + Vec2::new(0.0, 20.0),
        at + Vec2::X * 40.0,
    ] {
        assert!(doorway.allows(at, to, half), "{to}");
    }
    // A unit in the north door's middle may not go through the building and out of the east one.
    let top = north.spot(-18.0);
    assert!(!doorway.allows(top, top + Vec2::new(4.0, 4.0), half));
    assert!(doorway.allows(top, top + Vec2::new(4.0, -4.0), half));
}

/// [`choose`]: the door closest to the RallyPoint, else a random unblocked one; occupied doors are blocked, and
/// the building pauses only when every door is.
#[test]
fn a_unit_leaves_by_the_door_closest_to_the_rally_point_or_a_random_unblocked_one() {
    let doorway = Doorway::of(&building(ObjectKind::RoboPortal, Rotation::R0, false)).unwrap();
    let state = |occupied: [bool; 4], obstructed: [bool; 4]| -> Vec<DoorState> {
        doorway
            .doors()
            .enumerate()
            .map(|(i, door)| DoorState {
                door,
                occupied: occupied[i],
                obstructed: obstructed[i],
            })
            .collect()
    };
    let no_draw = |_| -> u32 { panic!("no draw for one door") };
    let open = state([false; 4], [false; 4]);
    // x 128..384, y 64..320: east of it, north-east of it (closer to the north door), and its very middle.
    assert_eq!(choose(&open, Some(Vec2::new(900.0, 200.0)), no_draw), Some(1));
    assert_eq!(choose(&open, Some(Vec2::new(400.0, -300.0)), no_draw), Some(0));
    assert_eq!(
        choose(&open, Some(Vec2::new(256.0, 192.0)), no_draw),
        Some(0),
        "ties go top first"
    );
    assert_eq!(
        choose(&open, Some(Vec2::new(0.0, 450.0)), no_draw),
        Some(2),
        "then right, then bottom"
    );
    // No rally point: a draw among all four.
    let mut drawn = Vec::new();
    assert_eq!(
        choose(&open, None, |n| {
            drawn.push(n);
            3
        }),
        Some(3)
    );
    // The closest door occupied or obstructed: a draw among the unblocked others.
    let east = Some(Vec2::new(900.0, 200.0));
    let busy = state([false, true, false, false], [false; 4]);
    assert_eq!(
        choose(&busy, east, |n| {
            drawn.push(n);
            1
        }),
        Some(2)
    );
    let walled = state([false; 4], [false, true, false, true]);
    assert_eq!(
        choose(&walled, east, |n| {
            drawn.push(n);
            1
        }),
        Some(2)
    );
    assert_eq!(drawn, [4, 3, 2]);
    // One unblocked door: that one, without a draw.
    let one = state([true, false, true, false], [false, false, false, true]);
    assert_eq!(choose(&one, None, no_draw), Some(1));
    // Every door that isn't occupied obstructed: the closest of them, or a draw among them; never a pause.
    let stuck = state([true, false, false, true], [true; 4]);
    assert_eq!(choose(&stuck, east, no_draw), Some(1));
    assert_eq!(choose(&stuck, Some(Vec2::new(0.0, 0.0)), |n| n - 1), Some(2));
    // Every door occupied: the building pauses.
    assert_eq!(choose(&state([true; 4], [false; 4]), east, no_draw), None);
}

/// What stands in front of a doorway leaves a unit appearing there less room; what stands behind it, in another
/// door or beyond the building's far side, doesn't (a portal's units stand in its other doors).
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
