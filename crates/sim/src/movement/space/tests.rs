//! Unit tests of the box geometry, the spatial grid and the collision exceptions of [`blocks`].

use super::*;
use crate::ids::ObjectId;

const REACH: Vec2 = Vec2::splat(24.0);

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

#[test]
fn a_sweep_finds_the_first_contact_and_its_axis() {
    // 30 su apart in x: 6 su of free travel.
    let hit = sweep(v(0.0, 0.0), v(8.0, 0.0), v(30.0, 0.0), REACH).unwrap();
    assert_eq!((hit.t, hit.axis), (0.75, 0));
    assert_eq!(
        sweep(v(0.0, 0.0), v(6.0, 0.0), v(30.0, 0.0), REACH),
        None,
        "it ends touching"
    );
    // From above, moving down and a little east.
    let hit = sweep(v(0.0, -30.0), v(1.0, 12.0), v(0.0, 0.0), REACH).unwrap();
    assert_eq!((hit.t, hit.axis), (0.5, 1));
}

#[test]
fn touching_boxes_slide_and_part_freely() {
    let (a, b) = (v(0.0, 0.0), v(24.0, 0.0));
    assert_eq!(sweep(a, v(0.0, 5.0), b, REACH), None, "sliding along");
    assert_eq!(sweep(a, v(-5.0, 3.0), b, REACH), None, "leaving");
    assert_eq!(sweep(a, v(5.0, 0.0), b, REACH).map(|c| c.t), Some(0.0), "pushing in");
    assert_eq!(sweep(a, v(0.0, 0.0), b, REACH), None, "standing");
    // Past each other on the other axis.
    assert_eq!(sweep(a, v(50.0, 0.0), v(24.0, 24.0), REACH), None);
}

/// A box touching another's corner and moving in on both axes rests on the axis it moves less along.
#[test]
fn a_corner_contact_keeps_the_larger_motion() {
    let hit = sweep(v(-24.0, 24.0), v(4.0, -0.5), v(0.0, 0.0), REACH).unwrap();
    assert_eq!((hit.t, hit.axis), (0.0, 1));
    let hit = sweep(v(-24.0, 24.0), v(0.5, -4.0), v(0.0, 0.0), REACH).unwrap();
    assert_eq!((hit.t, hit.axis), (0.0, 0));
}

/// The assigned contact never overlaps by the predicate, whatever the rounding.
#[test]
fn resting_positions_never_overlap() {
    let mut rng = crate::rng::Rng::from_seed(5);
    for _ in 0..20_000 {
        let other = rng.next_f32() * 16_384.0;
        for reach in [24.0, 30.0, 36.0, 29.7] {
            for low_side in [true, false] {
                let at = resting(other, reach, low_side);
                assert!((at - other).abs() >= reach, "{at} rests inside {other} ± {reach}");
                assert!((at - other).abs() < reach + 0.01);
            }
        }
    }
}

/// (G) The cell marks never miss a body that lane seeking would count as oncoming: for every pair of
/// directions, a heading against `ahead` by more than the cosine is in one of the sectors of `kinds_against`.
#[test]
fn the_sectors_against_a_direction_hold_every_heading_against_it() {
    let mut rng = crate::rng::Rng::from_seed(11);
    let mut random_direction = || loop {
        let d = Vec2::new(rng.next_f32() * 2.0 - 1.0, rng.next_f32() * 2.0 - 1.0);
        if d.length() > 0.1 && d.length() <= 1.0 {
            break d.normalize();
        }
    };
    let body = |heading: Vec2, yields: bool| Body {
        id: ObjectId(1),
        owner: None,
        pos: Vec2::ZERO,
        half: Vec2::splat(12.0),
        oriented: None,
        velocity: Vec2::ZERO,
        heading,
        stationary: false,
        progress: None,
        arrived_with: None,
        yields,
        gathers: false,
        leads: true,
        moved: 0,
        crushes: false,
        crushable: false,
    };
    let (mut against, mut others) = (0, 0);
    for _ in 0..20_000 {
        // Headings come in any length.
        let (ahead, heading) = (random_direction(), random_direction() * 37.0);
        let kinds = kinds_against(ahead, -0.5);
        assert_eq!(
            body(heading, false).heads_against(ahead, -0.5),
            heading.normalize().dot(ahead) < -0.5
        );
        if body(heading, false).heads_against(ahead, -0.5) {
            assert_ne!(body(heading, false).kinds() & kinds, 0, "{heading} against {ahead}");
            against += 1;
        } else if body(heading, false).kinds() & kinds == 0 {
            others += 1;
        }
    }
    assert!(
        against > 1000 && others > 1000,
        "the marks also rule most others out: {others}"
    );
    for axis in [Vec2::X, Vec2::Y, -Vec2::X, -Vec2::Y] {
        assert_ne!(body(-axis, false).kinds() & kinds_against(axis, -0.5), 0);
        assert_eq!(body(axis, false).kinds() & kinds_against(axis, -0.5), 0);
    }
    assert_eq!(body(Vec2::ZERO, false).kinds(), 0);
    assert_eq!(body(Vec2::ZERO, true).kinds(), KIND_YIELDS);

    let mut layer = Layer::default();
    layer.reset(4, 4);
    layer.insert(0, v(100.0, 100.0), Vec2::splat(12.0), 0);
    let mut seen = Vec::new();
    assert!(!layer.near_any(v(64.0, 64.0), v(70.0, 70.0), KIND_YIELDS, |i| seen.push(i)));
    layer.mark(v(100.0, 100.0), Vec2::splat(12.0), KIND_YIELDS);
    assert!(layer.near_any(v(64.0, 64.0), v(70.0, 70.0), KIND_YIELDS, |i| seen.push(i)));
    assert_eq!(seen, [0]);
    assert!(!layer.near_any(v(64.0, 64.0), v(70.0, 70.0), 0xff, |i| seen.push(i)));
    assert!(!layer.near_any(v(130.0, 64.0), v(140.0, 70.0), KIND_YIELDS, |i| seen.push(i)));
    layer.reset(4, 4);
    assert!(!layer.near_any(v(0.0, 0.0), v(255.0, 255.0), KIND_YIELDS, |i| seen.push(i)));
    assert_eq!(seen, [0]);
}

#[test]
fn a_layer_lists_a_body_in_every_cell_it_reaches() {
    let mut layer = Layer::default();
    layer.reset(4, 4);
    let half = Vec2::splat(12.0);
    layer.insert(7, v(64.0, 100.0), half, 0); // straddles tiles (0, 1) and (1, 1)
    let mut seen = Vec::new();
    layer.near(v(0.0, 64.0), v(10.0, 70.0), |i| seen.push(i));
    assert_eq!(seen, [7]);
    seen.clear();
    layer.near(v(130.0, 64.0), v(140.0, 70.0), |i| seen.push(i));
    assert!(seen.is_empty());
    layer.remove(7, v(64.0, 100.0), half);
    layer.insert(7, v(200.0, 200.0), half, 0);
    layer.near(v(0.0, 0.0), v(100.0, 127.0), |i| seen.push(i));
    assert!(seen.is_empty());
    layer.reset(4, 4);
    layer.near(v(0.0, 0.0), v(255.0, 255.0), |i| seen.push(i));
    assert!(seen.is_empty());
}

fn body(id: u32, pos: Vec2, half: f32, gathers: bool) -> Body {
    Body {
        id: ObjectId(id),
        owner: None,
        pos,
        half: Vec2::splat(half),
        oriented: None,
        velocity: Vec2::ZERO,
        heading: Vec2::ZERO,
        stationary: true,
        progress: None,
        arrived_with: None,
        yields: false,
        gathers,
        leads: true,
        moved: 0,
        crushes: false,
        crushable: false,
    }
}

/// Design change 0106, `factions/colonists/units/drone.md > Gathering Drones pass through each other`: two Drones
/// on a Gather job don't block each other; a Drone on one and a unit that isn't (an idle Drone, any other unit) do,
/// both ways.
#[test]
fn only_two_gathering_drones_pass_through_each_other() {
    let (a, b) = (v(100.0, 100.0), v(150.0, 100.0));
    for (first, second, passes) in [
        (true, true, true),
        (true, false, false),
        (false, true, false),
        (false, false, false),
    ] {
        let (x, y) = (body(1, a, 18.0, first), body(2, b, 18.0, second));
        assert_eq!(blocks(&x, &y), !passes, "{first} {second}");
        assert_eq!(blocks(&y, &x), !passes, "{second} {first}");
    }
    let me = body(1, a, 18.0, true);
    assert!(!blocks(&me, &me), "never itself");
}

/// `systems/units.md > UnitCollision > Units that already overlap` (design change 0106): two units whose boxes
/// overlap ignore each other, whatever they are, and only until they have separated: touching is apart.
#[test]
fn units_that_already_overlap_ignore_each_other_until_they_are_apart() {
    // A Peacekeeper (24 wide) and a Guard (36 wide): they touch 30 su apart.
    let guard = body(2, v(100.0, 100.0), 18.0, false);
    for (distance, ignored) in [
        (0.0, true),
        (12.0, true),
        (29.9, true),
        (30.0, false),
        (31.0, false),
        (200.0, false),
    ] {
        for along in [Vec2::X, Vec2::Y, -Vec2::X, -Vec2::Y] {
            let peacekeeper = body(1, guard.pos + along * distance, 12.0, false);
            assert_eq!(blocks(&peacekeeper, &guard), !ignored, "{distance} {along}");
            assert_eq!(blocks(&guard, &peacekeeper), !ignored, "{distance} {along}");
        }
    }
    // Overlapping on one axis only is apart.
    let beside = body(1, v(100.0, 131.0), 12.0, false);
    assert!(blocks(&beside, &guard) && blocks(&guard, &beside));
}
