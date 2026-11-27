//! Corners (the brief's corner choice: "choose the speed to take it at, from full speed down to a stop and spin.
//! Take the fastest whose arc fits the free space on the nav grid, counting the time to brake to it";
//! `movement/tracked/corner.rs`) and Rugged terrain (`systems/units.md > TrackedVehicle`: RuggedTerrain false, so
//! Rugged tiles are impassable for it).

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{MoveStatus, SpawnAt, TilePos, Vec2};

use super::{HAND, P1, TOP, about, centre, done, gap_to_walls, move_to, step, turns};

/// The tiles of an ASCII map that are `M` (a mountain), or off it.
fn mountains(map: &str) -> impl Fn(TilePos) -> bool + '_ {
    let rows: Vec<&[u8]> = map.lines().map(str::as_bytes).collect();
    move |tile: TilePos| {
        let row = usize::try_from(tile.y).ok().and_then(|y| rows.get(y));
        row.and_then(|row| usize::try_from(tile.x).ok().and_then(|x| row.get(x)))
            .is_none_or(|&c| c == b'M')
    }
}

/// The brief's acceptance: "at full speed into a 90° bend of a corridor two tiles wide, it [slows for it] and never
/// touches the walls". Up a corridor two tiles (128 su) wide at full speed, it brakes before the bend to the speed of
/// its corner choice, turns by the law from its first tick below MaxSpeed, and drives on up the corridor to its goal.
/// Its Silhouette never touches a wall: it keeps a gap to every mountain tile in every tick.
#[test]
fn it_takes_a_right_angled_bend_of_a_corridor_without_touching_the_walls() {
    let map = [
        "MMMMMMMMMMMMMM",
        "MMMMMMMMMMM..M",
        "MMMMMMMMMMM..M",
        "MMMMMMMMMMM..M",
        "MMMMMMMMMMM..M",
        "MMMMMMMMMMM..M",
        "MMMMMMMMMMM..M",
        "MMMMMMMMMMM..M",
        "MMMMMMMMMMM..M",
        "M............M",
        "M............M",
        "MMMMMMMMMMMMMM",
    ]
    .join("\n");
    let solid = mountains(&map);
    let mut t = ScenarioBuilder::new(&map)
        .spawn(HAND, Some(P1), SpawnAt::Point(Vec2::new(128.0, 640.0)))
        .start();
    let goal = Vec2::new(768.0, 160.0);
    move_to(&mut t, 1, &[1], goal);
    let (mut fastest, mut slowest_after, mut nearest) = (0.0_f32, f32::INFINITY, f32::INFINITY);
    let mut turned_at_top = false;
    while !done(&t, 1) {
        let (moved, turned) = step(&mut t, 1);
        let speed = moved.length();
        fastest = fastest.max(speed);
        if about(fastest, TOP) && t.pos(1).y > 400.0 {
            slowest_after = slowest_after.min(speed);
        }
        turned_at_top |= about(speed, TOP) && turns(turned);
        nearest = nearest.min(gap_to_walls(&t, 1, &solid));
        assert!(t.sim().tick() < 500, "{}", t.ascii(Some(t.pos(1))));
    }
    println!("a bend of a corridor two tiles wide: full speed {fastest}, slowest at the bend {slowest_after}");
    println!("nearest a wall: {nearest} su");
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, goal));
    assert!(about(fastest, TOP), "it was at full speed: {fastest}");
    assert!(slowest_after < 4.0, "it slowed for the bend: {slowest_after}");
    assert!(!turned_at_top, "it never turned at MaxSpeed");
    assert!(nearest > 0.0, "it never touched a wall: {nearest}");
}

/// The brief's acceptance: "A gentle curve keeps it within 10% of MaxSpeed". A wall across the way, three tiles long,
/// bends its path by about 10° round its end. From the moment it first reaches full speed until it brakes for its goal
/// it never goes slower than 4.5 su a tick.
#[test]
fn a_gentle_curve_keeps_it_within_a_tenth_of_its_top_speed() {
    let mut rows = vec![".".repeat(48); 14];
    for row in &mut rows[6..=8] {
        row.replace_range(24..25, "M");
    }
    let map = rows.join("\n");
    let mut t = ScenarioBuilder::new(&map)
        .spawn(HAND, Some(P1), SpawnAt::Point(centre(1, 7)))
        .start();
    let goal = centre(46, 7);
    move_to(&mut t, 1, &[1], goal);
    let mut speeds = Vec::new();
    let mut curve = 0.0_f32;
    while !done(&t, 1) {
        let (moved, turned) = step(&mut t, 1);
        speeds.push(moved.length());
        curve += turned.abs();
        assert!(t.sim().tick() < 800);
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, goal));
    let first_full = speeds.iter().position(|&speed| about(speed, TOP)).expect("full speed");
    let last_full = speeds.iter().rposition(|&speed| about(speed, TOP)).expect("full speed");
    let slowest = speeds[first_full..=last_full]
        .iter()
        .copied()
        .fold(f32::INFINITY, f32::min);
    println!("a gentle curve: turned {curve} rad in all, slowest {slowest} su a tick at speed");
    assert!(curve > 0.1, "its path curved: {curve}");
    assert!(slowest >= 0.9 * TOP, "{slowest}: {speeds:?}");
}

/// Rugged terrain is impassable for a TrackedVehicle (RuggedTerrain false, design change 0039): a band of Rugged tiles
/// across the map with a gap at one end, and its path goes round through the gap. Its Silhouette never overlaps a
/// Rugged tile (invariant I2 on its nav class's grid).
#[test]
fn its_path_goes_round_rugged_terrain() {
    let mut rows = vec![".".repeat(20); 10];
    for row in &mut rows[..8] {
        row.replace_range(10..11, "r");
    }
    let map = rows.join("\n");
    let mut t = ScenarioBuilder::new(&map)
        .spawn(HAND, Some(P1), SpawnAt::Point(centre(4, 3)))
        .start();
    let goal = centre(16, 3);
    move_to(&mut t, 1, &[1], goal);
    let mut lowest = 0.0_f32;
    while !done(&t, 1) {
        t.step();
        lowest = lowest.max(t.pos(1).y);
        assert!(t.sim().tick() < 800);
    }
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, goal));
    assert!(lowest > 8.0 * 64.0, "it went round through the gap: {lowest}");
}
