//! The Drone and the ground: Rugged terrain at half speed (design change 0039), a wall met at speed that takes
//! all its velocity (0035), and a wall it only touches, which is no obstacle to a move along it.

use super::*;

/// Acceptance, **Rugged**, and 0039's Verification: "A Drone crosses a Rugged patch at roughly half speed and
/// speeds back up on leaving it." Its path leads straight through the strip (Rugged is no obstacle for a hover
/// unit); the drag is doubled while the tile under its centre is Rugged, so it settles at 4 there.
#[test]
fn a_drone_crosses_rugged_terrain_at_half_speed() {
    // Tiles 10 to 21 of the row are Rugged: 768 su.
    let row = format!("{}{}{}", ".".repeat(10), "r".repeat(12), ".".repeat(12));
    let mut t = ScenarioBuilder::new(&vec![row; 5].join("\n"))
        .spawn(DRONE, Some(P1), SpawnAt::Point(centre(1, 2)))
        .start();
    let goal = centre(32, 2);
    move_to(&mut t, 1, goal);
    let (mut before, mut inside, mut after) = (Vec::new(), Vec::new(), Vec::new());
    while t.status(1) != MoveStatus::Arrived {
        let d = step(&mut t);
        assert_eq!(d.y, 0.0, "straight through");
        let tile = TilePos::containing(t.pos(1)).x;
        match tile {
            ..10 => before.push(d.x),
            10..22 => inside.push(d.x),
            _ => after.push(d.x),
        }
        assert!(t.sim().tick() < 500, "{}", t.ascii(None));
    }
    assert_eq!(t.pos(1), goal);
    let fastest = |speeds: &[f32]| speeds.iter().copied().fold(0.0, f32::max);
    assert!(
        fastest(&before) > 7.9,
        "up to speed before the strip: {}",
        fastest(&before)
    );
    // It comes in at 8 and is braked by the drag: within 1 % of 4 from a third of the strip on.
    let settled = &inside[inside.len() / 3..];
    assert!(settled.iter().all(|&s| (3.96..=4.04).contains(&s)), "{inside:?}");
    assert!(
        inside.windows(2).all(|pair| pair[1] <= pair[0] + 1e-3),
        "it only slows in there"
    );
    // And back up to speed beyond it.
    assert!(fastest(&after) > 7.9, "{}", fastest(&after));
    assert!(after[..20].windows(2).all(|pair| pair[1] > pair[0]), "{after:?}");
}

/// Acceptance, **Collision** (design change 0035: "If the movement hits an obstacle, drop the blocked portion (no
/// overlap) and set velocity to 0"): a Drone flying east at speed is sent round a corner to the north. Its
/// momentum carries it into the wall beyond the corner. It goes as far as the wall and never into it (I2, every
/// tick), and **all its velocity is gone**, what it had along the wall too: it starts north again from rest, at
/// single-tick acceleration, and accelerates normally from there.
#[test]
fn a_drone_that_meets_a_wall_at_speed_loses_all_its_velocity() {
    // An L-shaped passage: east along row 6, then north along column 14. The wall east of column 14 is Mountain.
    let map = [
        "MMMMMMMMMMMMMM.M",
        "MMMMMMMMMMMMMM.M",
        "MMMMMMMMMMMMMM.M",
        "MMMMMMMMMMMMMM.M",
        "MMMMMMMMMMMMMM.M",
        "MMMMMMMMMMMMMM.M",
        "...............M",
        "MMMMMMMMMMMMMMMM",
    ];
    let mut t = ScenarioBuilder::new(&map.join("\n"))
        .spawn(DRONE, Some(P1), SpawnAt::Point(centre(1, 6)))
        .start();
    let goal = centre(14, 1);
    move_to(&mut t, 1, goal);
    // The east wall begins at x = 15 × 64: the Drone's centre rests 18 su before it.
    let against = 15.0 * 64.0 - HALF;
    let (mut came_at, mut after) = (Vec2::ZERO, Vec::new());
    while t.status(1) != MoveStatus::Arrived {
        let before = t.mover(1).momentum;
        let d = step(&mut t);
        assert!(t.pos(1).x <= against, "tick {}: in the wall", t.sim().tick());
        if came_at == Vec2::ZERO && t.pos(1).x == against {
            // The tick it met the wall: it went as far as the wall, and has no velocity left.
            came_at = before;
            assert!(
                d.x > 0.0 && d.x < before.x * 0.9,
                "the blocked part is dropped: {d} of {before}"
            );
            assert_eq!(t.mover(1).momentum, Vec2::ZERO, "tick {}", t.sim().tick());
        } else if came_at != Vec2::ZERO {
            after.push(d);
        }
        assert!(t.sim().tick() < 400, "{}", t.ascii(None));
    }
    assert!(
        came_at.x > 6.0 && came_at.y < 0.0,
        "it came at speed, turning north: {came_at}"
    );
    // From rest: 0.8 su in the first tick, however fast it was going north already, and faster every tick.
    assert!(after[0].length() <= 0.8 + 1e-3, "{:?}", &after[..4]);
    assert!(
        after[..20].windows(2).all(|pair| pair[1].y < pair[0].y),
        "{:?}",
        &after[..20]
    );
    assert_eq!(t.pos(1), goal);
}

/// A wall the Drone only touches is no obstacle to a move along it (touching is free): at rest against a wall
/// and ordered along it, it accelerates as in the open. Every step is at most what one tick's thrust adds to what
/// the drag left of the last. (The creep of a Drone whose way leads into the wall is
/// `movement/follow/tests.rs`.)
#[test]
fn a_drone_pressed_against_a_wall_creeps_away_along_it() {
    let map = ["......", "......", "MMMMMM", "MMMMMM"];
    // Its box rests on the wall to the south.
    let start = Vec2::new(96.0, 2.0 * 64.0 - HALF);
    let mut t = ScenarioBuilder::new(&map.join("\n"))
        .spawn(DRONE, Some(P1), SpawnAt::Point(start))
        .start();
    let goal = Vec2::new(320.0, start.y);
    move_to(&mut t, 1, goal);
    let mut last = 0.0_f32;
    while t.status(1) != MoveStatus::Arrived {
        let d = step(&mut t);
        assert_eq!(d.y, 0.0, "along the wall");
        assert!(
            d.x <= last * 0.9 + 0.8 + 1e-3,
            "tick {}: {d} after {last}",
            t.sim().tick()
        );
        last = d.x;
        assert!(t.sim().tick() < 200, "{}", t.ascii(None));
    }
    assert_eq!(t.pos(1), goal);
}
