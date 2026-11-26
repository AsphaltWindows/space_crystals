use core::f32::consts::TAU;

use super::super::tests::{building, orientations};
use super::*;
use crate::defs::ObjectKind;
use crate::math;
use crate::nav::{self, NavClass};
use crate::object::Rotation;
use crate::scenario::SpawnAt;
use crate::sides::{DIRECTIONS, SideLetter};
use crate::testing::ScenarioBuilder;

/// A Drone's half extents.
const HALF: Vec2 = Vec2::splat(18.0);

fn robo_portal() -> EnergyDoorway {
    EnergyDoorway::of(&building(ObjectKind::RoboPortal, Rotation::R0, false)).unwrap()
}

fn bio_portal() -> EnergyDoorway {
    EnergyDoorway::of(&building(ObjectKind::BioPortal, Rotation::R0, false)).unwrap()
}

/// The resolution of the random direction (module docs, 3): 32 headings 11.25° apart, the axes and the diagonals
/// exact, and the same set however the square is turned or flipped.
#[test]
fn the_headings_are_32_directions_that_the_square_turns_and_flips_into_one_another() {
    for (k, &h) in HEADINGS.iter().enumerate() {
        let exact = math::facing_vector(k as f32 * TAU / HEADING_COUNT as f32);
        assert!(h.distance(exact) < 1e-6, "{k}: {h} and {exact}");
    }
    assert_eq!(
        [HEADINGS[0], HEADINGS[8], HEADINGS[16], HEADINGS[24]],
        [Vec2::X, Vec2::Y, Vec2::NEG_X, Vec2::NEG_Y]
    );
    assert_eq!(HEADINGS[4], Vec2::splat(FRAC_1_SQRT_2));
    for h in HEADINGS {
        for image in [Vec2::new(-h.y, h.x), Vec2::new(-h.x, h.y), Vec2::new(h.y, h.x)] {
            assert!(HEADINGS.contains(&image), "{h} gives {image}");
        }
    }
}

/// `factions/colonists/structures/robo_portal.md`: "Since the Robo Portal is fully symmetric (AAAA), units may leave in
/// any direction", however it is turned.
#[test]
fn the_robo_portal_is_open_all_round_in_every_orientation() {
    for (rotation, flip_h) in orientations() {
        let portal = EnergyDoorway::of(&building(ObjectKind::RoboPortal, rotation, flip_h)).unwrap();
        // 4 × 4 at tile (2, 1): x 128..384, y 64..320.
        assert_eq!(portal.centre(), Vec2::new(256.0, 192.0));
        assert_eq!(portal.open_sides(), [true; 4], "{rotation:?} {flip_h}");
        assert!(HEADINGS.iter().all(|&h| portal.margin(h) == f32::INFINITY));
        assert_eq!(portal.arc_ends().count(), 0);
    }
}

/// `factions/colonists/structures/bio_portal.md > Exit Behavior` (design change 0145): open through its two A sides,
/// top and right unturned (design change 0079), wherever they face once it is placed: "a half-circle from pointing at
/// the top-left corner round to pointing at the bottom-right corner", turned and flipped with it, its ends open (the
/// Director's reading). 17 of the 32 headings.
#[test]
fn the_bio_portal_is_open_through_its_two_a_sides_in_all_eight_orientations() {
    let (n, e, s, w) = (Vec2::NEG_Y, Vec2::X, Vec2::Y, Vec2::NEG_X);
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
    for (rotation, flip_h, open) in cases {
        let what = format!("{rotation:?}, flipped {flip_h}");
        let object = building(ObjectKind::BioPortal, rotation, flip_h);
        let portal = EnergyDoorway::of(&object).unwrap();
        // Its open sides are the A sides, as the lettering of the placed building has them.
        let letters = sides(ObjectKind::BioPortal, object.placement.as_ref().unwrap().orientation()).unwrap();
        assert_eq!(portal.open_sides(), letters.map(|l| l == SideLetter::A), "{what}");
        let outwards: Vec<Vec2> = (0..4)
            .filter(|&i| portal.open_sides()[i])
            .map(|i| Vec2::new(DIRECTIONS[i].0 as f32, DIRECTIONS[i].1 as f32))
            .collect();
        assert_eq!(outwards, open, "{what}");
        // A half-circle: the directions that point no less towards the two open sides than away from them.
        let towards = open[0] + open[1];
        for h in HEADINGS {
            assert_eq!(portal.is_open(h), towards.dot(h) >= 0.0, "{what}: {h}");
        }
        assert_eq!(HEADINGS.iter().filter(|&&h| portal.is_open(h)).count(), 17, "{what}");
        // Its ends: the two corners between an open side and a closed one, open; a hair round past each, closed.
        let ends: Vec<Vec2> = portal.arc_ends().collect();
        assert_eq!(ends.len(), 2, "{what}");
        for end in ends {
            assert!(towards.dot(end).abs() < 1e-6, "{what}: {end}");
            assert_eq!(portal.margin(end), 0.0, "{what}: {end}");
            let angle = math::vector_angle(end);
            let (clockwise, back) = (math::facing_vector(angle + 0.01), math::facing_vector(angle - 0.01));
            assert_ne!(portal.is_open(clockwise), portal.is_open(back), "{what}: {end}");
        }
    }
}

/// The arc comes from the sides, whatever the footprint's shape (design change 0145: the Heavy and Star Portals'
/// sizes are TBD): a 4 × 2 doorway open north and east is open from its top-left corner round to its bottom-right
/// one, which aren't diagonals.
#[test]
fn a_wide_doorway_open_on_two_sides_is_open_from_corner_to_corner() {
    let portal = EnergyDoorway {
        frame: Frame {
            origin: TilePos::new(0, 0),
            size: (4, 2),
            lo: Vec2::ZERO,
            hi: Vec2::new(256.0, 128.0),
        },
        open: [true, true, false, false],
    };
    let ends: Vec<Vec2> = portal.arc_ends().collect();
    assert_eq!(ends.len(), 2);
    assert!(ends[0].distance(Vec2::new(2.0, 1.0).normalize()) < 1e-6, "{ends:?}");
    assert!(ends[1].distance(Vec2::new(-2.0, -1.0).normalize()) < 1e-6, "{ends:?}");
    assert!(portal.is_open(Vec2::X) && portal.is_open(Vec2::NEG_Y));
    assert!(!portal.is_open(Vec2::Y) && !portal.is_open(Vec2::NEG_X));
    // Either side of each corner.
    assert!(portal.is_open(Vec2::new(2.0, 0.9)) && !portal.is_open(Vec2::new(2.0, 1.1)));
    assert!(portal.is_open(Vec2::new(-2.0, -1.1)) && !portal.is_open(Vec2::new(-2.0, -0.9)));
    // Rallied due south, it goes round by the nearer end.
    let south = portal.centre() + Vec2::new(0.0, 500.0);
    let heading = portal.heading(Some(south), |_| true, |_| unreachable!()).unwrap();
    assert!(heading.distance(ends[0]) < 1e-6, "{heading}");
}

/// Points 1 and 3 of the module docs: from the centre a unit walks along its heading until its box is clear of the
/// footprint, touching the side it clears, that coordinate exact; and while it is in the doorway it never comes
/// closer to the centre, whatever the rounding of its steps.
#[test]
fn a_unit_walks_out_along_its_heading_and_never_back_in() {
    let portal = robo_portal();
    let c = portal.centre();
    // x 128..384, y 64..320.
    assert_eq!(portal.outside(c, Vec2::X, HALF), Vec2::new(402.0, 192.0));
    assert_eq!(portal.outside(c, Vec2::NEG_Y, HALF), Vec2::new(256.0, 46.0));
    for h in HEADINGS {
        let out = portal.outside(c, h, HALF);
        assert!(
            !portal.holds(out, HALF) && portal.holds(out - h * 0.01, HALF),
            "{h}: {out}"
        );
        assert!((out - c).perp_dot(h).abs() < 1e-3, "{h}: {out} is on the ray");
        assert_eq!(portal.outside(out, h, HALF), out, "{h}: clear already");
        // Steps of any length along the heading: each allowed.
        let mut pos = c;
        for step in [0.8, 1.52, 7.99, 0.031_25, 5.0, 8.0, 0.000_1, 3.3]
            .into_iter()
            .cycle()
            .take(60)
        {
            let to = pos + h * step;
            assert!(portal.allows(pos, to, HALF), "{h}: {pos} to {to}");
            pos = to;
        }
    }
    let at = c + Vec2::new(30.0, -10.0);
    assert!(portal.allows(at, at + Vec2::new(3.0, -1.0), HALF), "out");
    assert!(
        portal.allows(at, at + Vec2::new(0.0, -5.0), HALF),
        "sideways, no closer"
    );
    assert!(
        !portal.allows(at, at - Vec2::new(3.0, -1.0), HALF),
        "back towards the centre"
    );
    assert!(portal.allows(at, Vec2::new(500.0, 192.0), HALF), "clear of it");
}

/// Point 3 of the module docs: towards the rally point if that is open, else the nearest open direction whose way
/// out is free, the clockwise one first of two as near; with none, a draw among the open free headings.
#[test]
fn the_heading_is_towards_the_rally_point_or_the_nearest_open_free_direction() {
    let (robo, bio) = (robo_portal(), bio_portal());
    let free = |_: Vec2| true;
    let no_draw = |_: u32| -> u32 { panic!("no draw with a rally point") };
    // Any way at all out of the Robo Portal.
    for offset in [Vec2::new(300.0, -300.0), Vec2::new(123.0, 457.0), Vec2::new(-1.0, 0.0)] {
        let rally = robo.centre() + offset;
        assert_eq!(robo.heading(Some(rally), free, no_draw), offset.try_normalize());
    }
    // The Bio Portal unturned is open north and east. South-west of it is closed, and the two ends of its arc,
    // north-west and south-east, are as near to it: the clockwise one, north-west, goes first.
    let c = bio.centre();
    let (nw, se) = (Vec2::new(-1.0, -1.0).normalize(), Vec2::new(1.0, 1.0).normalize());
    let south_west = Some(c + Vec2::new(-300.0, 300.0));
    let near = |heading: Option<Vec2>, to: Vec2| heading.is_some_and(|h| h.distance(to) < 1e-6);
    assert!(near(bio.heading(south_west, free, no_draw), nw));
    // Its way blocked: the other end; blocked every way: the nearest all the same.
    assert!(near(bio.heading(south_west, |d| d.distance(nw) > 1e-3, no_draw), se));
    assert!(near(bio.heading(south_west, |_| false, no_draw), nw));
    // Due south: the south-east end is 45° off, the north-west one 135°.
    assert!(near(bio.heading(Some(c + Vec2::new(0.0, 400.0)), free, no_draw), se));
    // East, open but blocked: the headings 11.25° either side of it, the clockwise (southern) one first.
    let east = Some(c + Vec2::new(400.0, 0.0));
    assert_eq!(bio.heading(east, |d| d.x < 0.99, no_draw), Some(HEADINGS[1]));
    assert!(near(bio.heading(east, free, no_draw), Vec2::X));
    assert!(
        near(bio.heading(east, |_| false, no_draw), Vec2::X),
        "no way out: its own"
    );

    // No rally point: a draw among the open free headings, in their order (east round to the south-east end, then
    // from the north-west end round to east).
    let mut drawn = Vec::new();
    assert_eq!(bio.heading(None, free, pick(&mut drawn, 0)), Some(HEADINGS[0]));
    assert_eq!(bio.heading(None, free, pick(&mut drawn, 4)), Some(HEADINGS[4]));
    assert_eq!(bio.heading(None, free, pick(&mut drawn, 5)), Some(HEADINGS[20]));
    assert_eq!(bio.heading(None, free, pick(&mut drawn, 16)), Some(HEADINGS[31]));
    // One free: that one, without a draw; none: a draw among every open one.
    assert_eq!(bio.heading(None, |d| d == Vec2::NEG_Y, no_draw), Some(Vec2::NEG_Y));
    assert_eq!(bio.heading(None, |_| false, pick(&mut drawn, 1)), Some(HEADINGS[1]));
    assert_eq!(robo.heading(None, free, pick(&mut drawn, 9)), Some(HEADINGS[9]));
    // A rally point at the very centre gives no direction: as with none.
    assert_eq!(
        robo.heading(Some(robo.centre()), free, pick(&mut drawn, 3)),
        Some(HEADINGS[3])
    );
    assert_eq!(drawn, [17, 17, 17, 17, 17, 32, 32]);
}

/// A draw that notes how many it chose among, in `drawn`, and picks the `k`th.
fn pick(drawn: &mut Vec<u32>, k: u32) -> impl FnOnce(u32) -> u32 + '_ {
    move |n| {
        drawn.push(n);
        k
    }
}

/// A unit without a heading (a state saved before the field existed) leaves straight away from the centre if that
/// is open, else along the first open heading.
#[test]
fn a_unit_without_a_heading_leaves_straight_away_from_the_centre_if_it_can() {
    let bio = bio_portal();
    let c = bio.centre();
    assert_eq!(bio.heading_of(c + Vec2::new(0.0, -10.0), None), Vec2::NEG_Y);
    assert_eq!(
        bio.heading_of(c + Vec2::new(-10.0, 0.0), None),
        HEADINGS[0],
        "west is closed"
    );
    assert_eq!(bio.heading_of(c, None), HEADINGS[0]);
    assert_eq!(bio.heading_of(c, Some(Vec2::NEG_Y)), Vec2::NEG_Y);
}

/// Point 3 of the module docs, the Director's reading: the way out along a direction is free unless the unit's box,
/// swept from the centre to one unit length beyond where it comes clear, enters a blocked tile but the portal's own
/// (or the map's edge), or a unit. Touching is free.
#[test]
fn the_way_out_is_free_unless_a_blocked_tile_or_a_unit_lies_in_its_sweep() {
    // A Robo Portal at tile (2, 2) of a 10 × 9 map, x 128..384 and y 128..384, and a cliff at tile (7, 4), x
    // 448..512 and y 256..320. Straight east, the box comes clear at x 402 and sweeps on to 438: up to x 456.
    let map = [
        "..........",
        "..........",
        "..........",
        "..........",
        ".......c..",
        "..........",
        "..........",
        "..........",
        "..........",
    ]
    .join("\n");
    let t = ScenarioBuilder::new(&map)
        .spawn(ObjectKind::RoboPortal, None, SpawnAt::Tile(TilePos::new(2, 2)))
        .start();
    let portal = EnergyDoorway::of(t.sim().objects().next().unwrap()).unwrap();
    let grid = t.sim().nav().grid(drone_class(), nav::size_class(HALF * 2.0));
    assert!(grid.is_some());
    let free = |dir: Vec2, others: &[(Vec2, Vec2)]| portal.way_out_free(dir, HALF, grid, others);
    assert!(!free(Vec2::X, &[]), "the cliff");
    assert!(!free(HEADINGS[1], &[]), "the cliff, a little south of east");
    assert!(free(HEADINGS[31], &[]), "a little north of east, past it");
    assert!(free(Vec2::NEG_Y, &[]) && free(Vec2::Y, &[]) && free(Vec2::NEG_X, &[]));
    // Its own tiles never block it. Units: one whose box the sweep enters blocks it, one it only touches doesn't.
    assert!(!free(Vec2::NEG_X, &[(Vec2::new(73.0, 256.0), HALF)]));
    assert!(free(Vec2::NEG_X, &[(Vec2::new(38.0, 256.0), HALF)]));
    assert!(free(Vec2::NEG_X, &[(Vec2::new(74.0, 292.0), HALF)]));
    assert!(!free(Vec2::NEG_X, &[(Vec2::new(74.0, 284.0), HALF)]));
    // Without a grid (a unit that crosses every tile), only units count.
    assert!(portal.way_out_free(Vec2::X, HALF, None, &[]));
    // The map's edge: a portal at tile (2, 0) has no room north of it.
    let t = ScenarioBuilder::new(&map)
        .spawn(ObjectKind::RoboPortal, None, SpawnAt::Tile(TilePos::new(2, 0)))
        .start();
    let portal = EnergyDoorway::of(t.sim().objects().next().unwrap()).unwrap();
    let grid = t.sim().nav().grid(drone_class(), nav::size_class(HALF * 2.0));
    assert!(!portal.way_out_free(Vec2::NEG_Y, HALF, grid, &[]));
    assert!(portal.way_out_free(Vec2::NEG_X, HALF, grid, &[]));
}

/// (M5-Z1) Under the portal, ground the unit can't stand on blocks it (`Frame::blocks`): a map's object or a debug
/// spawn may stand on any flat ground, and the doorway lets the unit overlap the portal, not that ground. A Robo
/// Portal over a Cliff has no way out across it, and no spot over it; its open tiles never block.
#[test]
fn ground_it_cant_stand_on_under_the_portal_blocks_its_unit() {
    // The Robo Portal at tile (2, 2), x and y 128..384, over a Cliff on tile (5, 3), x 320..384 and y 192..256:
    // straight east from the centre, (256, 256), the box sweeps y 238..274 across it.
    let map = "........\n........\n........\n.....c..\n........\n........\n........\n........";
    let t = ScenarioBuilder::new(map)
        .spawn(ObjectKind::RoboPortal, None, SpawnAt::Tile(TilePos::new(2, 2)))
        .start();
    let portal = EnergyDoorway::of(t.sim().objects().next().unwrap()).unwrap();
    let grid = t.sim().nav().grid(drone_class(), nav::size_class(HALF * 2.0));
    assert!(!portal.way_out_free(Vec2::X, HALF, grid, &[]), "across the Cliff");
    assert!(portal.way_out_free(Vec2::NEG_X, HALF, grid, &[]));
    assert!(portal.way_out_free(Vec2::NEG_Y, HALF, grid, &[]));
    let (grid, centre) = (grid.unwrap(), portal.centre());
    assert!(portal.box_clear(grid, HALF, centre), "the centre, on its open tiles");
    // At (320, 216) the box covers tiles (4, 3) and (5, 3).
    assert!(!portal.box_clear(grid, HALF, centre + Vec2::new(64.0, -40.0)));
}

/// The Drone's nav class (a hover vehicle's, which crosses Rugged terrain).
fn drone_class() -> NavClass {
    let def = ObjectKind::Drone.def();
    def.unit.as_ref().unwrap().base_def().nav_class().unwrap()
}
