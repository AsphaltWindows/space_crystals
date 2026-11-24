//! Design change 0038's verification (`systems/units.md > UnitCollision > Ground Collision`): crystal patches and
//! Supply Delivery Stations are solid for ground units. Units path around a crystal field, can't walk across a
//! station, and walk through the tile of a patch once it is gone (a debug removal stands in for depletion until
//! mining exists).

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, SU_PER_GU, SpawnAt, Target, TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const PK: ObjectKind = ObjectKind::Peacekeeper;
/// The Peacekeeper's half silhouette (24 × 24 su).
const HALF: f32 = 12.0;

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// The centre of tile `(x, y)`.
fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn move_to(t: &mut TestSim, tick: u32, unit: u32, to: Vec2) {
    t.order_at(tick, P1, &[ObjectId(unit)], OrderKind::Move(Target::Point(to)), false);
}

/// Whether the unit's box at `pos` overlaps the footprint `(x, y, w, h)` in tiles. Touching is free.
fn overlaps(pos: Vec2, (x, y, w, h): (i32, i32, i32, i32)) -> bool {
    let lo = Vec2::new(x as f32, y as f32) * SU_PER_GU;
    let hi = lo + Vec2::new(w as f32, h as f32) * SU_PER_GU;
    pos.x - HALF < hi.x && pos.x + HALF > lo.x && pos.y - HALF < hi.y && pos.y + HALF > lo.y
}

/// Runs `ticks` ticks and returns where unit `id` was at the end of each.
fn track(t: &mut TestSim, id: u32, ticks: u32) -> Vec<Vec2> {
    (0..ticks).map(|_| t.step().pos(id)).collect()
}

/// A crystal field, a wall of nine patches across the middle of the map with open ground only at its south end, lies
/// between a unit and its destination: the unit walks around the field's end, never over a patch, and arrives.
#[test]
fn units_path_around_a_crystal_field() {
    let mut builder = ScenarioBuilder::new(&"....................\n".repeat(12));
    for y in 0..9 {
        builder = builder.spawn(ObjectKind::CrystalPatch, None, tile(10, y)); // 1–9
    }
    let mut t = builder.spawn(PK, Some(P1), SpawnAt::Point(centre(5, 3))).start(); // 10
    move_to(&mut t, 1, 10, centre(15, 3));
    let path = track(&mut t, 10, 400);
    assert_eq!(t.status(10), MoveStatus::Arrived);
    assert!(t.pos(10).distance(centre(15, 3)) < 1.0, "{}", t.pos(10));
    for (tick, &pos) in path.iter().enumerate() {
        assert!(
            !overlaps(pos, (10, 0, 1, 9)),
            "over the field in tick {}: {pos}",
            tick + 1
        );
    }
    // Around the field's south end, which lies 6 tiles south of the straight line.
    assert!(path.iter().any(|p| p.y > 9.0 * SU_PER_GU), "it went around the end");
    let arrival = path.iter().position(|p| p.distance(centre(15, 3)) < 1.0).unwrap() + 1;
    assert!(arrival > 10 * 64 / 4, "slower than the straight 160 ticks: {arrival}");
}

/// A station seals a corridor two tiles wide: a unit ordered to the far side walks up to it and stops there, the
/// nearest point it can reach (design change 0025), never crossing. On open ground a unit walks around one.
#[test]
fn units_cant_walk_across_a_supply_station() {
    let wall = "MMMMMMMMMMMMMMMMMMMM\n";
    let corridor = "....................\n";
    let map = [wall, wall, corridor, corridor, wall, wall].concat();
    let mut t = ScenarioBuilder::new(&map)
        .spawn(ObjectKind::SupplyDeliveryStation, None, tile(10, 2)) // 1: tiles (10, 2) to (11, 3)
        .spawn(
            PK,
            Some(P1),
            SpawnAt::Point(Vec2::new(4.0 * SU_PER_GU, 3.0 * SU_PER_GU)),
        ) // 2
        .start();
    let far_side = Vec2::new(16.0 * SU_PER_GU, 3.0 * SU_PER_GU);
    move_to(&mut t, 1, 2, far_side);
    let path = track(&mut t, 2, 300);
    for (tick, &pos) in path.iter().enumerate() {
        assert!(
            !overlaps(pos, (10, 2, 2, 2)),
            "on the station in tick {}: {pos}",
            tick + 1
        );
        assert!(pos.x < 10.0 * SU_PER_GU, "past the station in tick {}: {pos}", tick + 1);
    }
    assert_eq!(t.status(2), MoveStatus::Arrived);
    assert_eq!(
        TilePos::containing(t.pos(2)).x,
        9,
        "it walked up to the tile next to the station: {}",
        t.pos(2)
    );

    // In the open, the straight line through the station bends around it.
    let mut t = ScenarioBuilder::new(&"....................\n".repeat(10))
        .spawn(ObjectKind::SupplyDeliveryStation, None, tile(9, 4)) // 1: tiles (9, 4) to (10, 5)
        .spawn(
            PK,
            Some(P1),
            SpawnAt::Point(Vec2::new(4.0 * SU_PER_GU, 5.0 * SU_PER_GU)),
        ) // 2
        .start();
    let beyond = Vec2::new(16.0 * SU_PER_GU, 5.0 * SU_PER_GU);
    move_to(&mut t, 1, 2, beyond);
    let path = track(&mut t, 2, 300);
    assert_eq!(t.status(2), MoveStatus::Arrived);
    assert!(t.pos(2).distance(beyond) < 1.0, "{}", t.pos(2));
    for (tick, &pos) in path.iter().enumerate() {
        assert!(
            !overlaps(pos, (9, 4, 2, 2)),
            "on the station in tick {}: {pos}",
            tick + 1
        );
    }
    assert!(
        path.iter()
            .any(|p| (p.y - 5.0 * SU_PER_GU).abs() >= SU_PER_GU + HALF - 1.0)
    );
}

/// A mountain wall with a one-tile gap, and a patch in the gap: a unit can't get through and stops at the patch.
/// Once a debug removal takes the patch away (as depletion will), its tile is open ground: the grids are rebuilt
/// and the unit, ordered again, walks through the gap.
#[test]
fn a_removed_patch_opens_its_tile() {
    let wall = ".........M..........\n";
    let gap = "....................\n";
    let map = [wall, wall, wall, wall, wall, gap, wall, wall, wall, wall, wall].concat();
    let mut t = ScenarioBuilder::new(&map)
        .spawn(ObjectKind::CrystalPatch, None, tile(9, 5)) // 1, in the gap
        .spawn(PK, Some(P1), SpawnAt::Point(centre(4, 5))) // 2
        .start();
    let beyond = centre(15, 5);
    move_to(&mut t, 1, 2, beyond);
    let path = track(&mut t, 2, 200);
    assert!(path.iter().all(|&p| !overlaps(p, (9, 5, 1, 1))), "never on the patch");
    assert_eq!(t.status(2), MoveStatus::Arrived, "at the nearest point it can reach");
    assert!(t.pos(2).x < 9.0 * SU_PER_GU, "{}", t.pos(2));

    let version = t.sim().nav_version();
    let tick = t.sim().tick() + 1;
    t.debug_at(tick, DebugCommand::Remove { id: ObjectId(1) });
    move_to(&mut t, tick + 1, 2, beyond);
    let path = track(&mut t, 2, 200);
    assert!(t.sim().object(ObjectId(1)).is_none());
    assert!(
        t.sim().nav_version() > version,
        "the grids were rebuilt without the patch"
    );
    assert_eq!(t.status(2), MoveStatus::Arrived);
    assert!(t.pos(2).distance(beyond) < 1.0, "{}", t.pos(2));
    assert!(
        path.iter().any(|&p| TilePos::containing(p) == TilePos::new(9, 5)),
        "through the patch's tile"
    );
}
