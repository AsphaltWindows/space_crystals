//! How fast a Hall recruits, and up to what (`factions/cults/structures/recruitment_hall.md > Effectiveness`,
//! `factions/cults/structures/recruitment_hall.md > BaseUnitControlCapacity - 20`,
//! `factions/cults/structures/recruitment_hall.md > BaseRecruitProductionTime - 12 seconds (192 frames) per Recruit`).

use sc_sim::{DebugCommand, ObjectId, Target, Vec2};

use super::*;

/// Plan acceptance, 100%: "6 → 20 Recruits in 2688 ticks, then none" (`factions/cults/structures/recruitment_hall.md >
/// BaseRecruitProductionTime - 12 seconds (192 frames) per Recruit`: "a starting hall (6 Recruits → 20) fills in 14 ×
/// 12 s"). A Hall on open Plane claims its 100 tiles and makes a Recruit every 192 ticks from tick 1, the k-th in tick
/// 192 k, each of its lineage; at 20 it stops, nothing in production, and the HUD shows 20/20. A Recruit counts from
/// the start of its production (design change 0342: "The Recruit in production counts toward LocalUnitControlUsage from
/// the start of its production").
#[test]
fn a_full_hall_fills_from_6_to_20_in_2688_ticks_then_stops() {
    let hall = ObjectId(1);
    let mut t = flat(40, 40).spawn(HALL, Some(P1), tile(18, 18)).start();
    assert_eq!(
        unit_control(&t, P1),
        (0, 20),
        "nothing in production before the Hall's first step"
    );
    t.run_until(FRAMES - 1);
    assert_eq!(unit_control(&t, P1), (1, 20), "the Recruit in production counts");
    assert_eq!(progress(&t, hall), Some(100 * (FRAMES - 1)), "100 tiles a frame");
    t.run_until(5000);
    let ticks = ticks(&t, hall);
    assert_eq!(ticks, (1..=20).map(|k| FRAMES * k).collect::<Vec<_>>());
    assert_eq!(ticks[19] - ticks[5], 2688, "6 → 20: 14 × 12 s");
    for (_, unit) in made(&t, hall) {
        let o = t.sim().object(unit).expect("alive");
        assert_eq!(
            (o.kind, o.owner, o.lineage.as_slice()),
            (RECRUIT, Some(P1), [hall].as_slice())
        );
    }
    let state = hall_state(&t, hall);
    assert_eq!(
        (state.claimed, state.usage, state.effectiveness_percent()),
        (100, 20, 100)
    );
    assert_eq!(progress(&t, hall), None, "at capacity nothing is in production");
    assert_eq!(unit_control(&t, P1), (20, 20));
}

/// The HUD's used figure climbs by one each 192 ticks, as each Recruit starts (design change 0342: "The Recruit in
/// production counts toward LocalUnitControlUsage from the start of its production"): the k-th in production from
/// tick 192 (k − 1), counted from then; out in tick 192 k, when the next starts, and none after the 20th.
#[test]
fn the_hud_counts_each_recruit_from_the_start_of_its_production() {
    let hall = ObjectId(1);
    let mut t = flat(40, 40).spawn(HALL, Some(P1), tile(18, 18)).start();
    for k in 1..=20 {
        t.run_until(FRAMES * k - 1);
        assert_eq!(unit_control(&t, P1), (k, 20), "Recruit {k} in production");
        assert_eq!(made(&t, hall).len(), k as usize - 1);
        t.run(1);
        let used = (k + 1).min(20);
        assert_eq!(unit_control(&t, P1), (used, 20), "Recruit {k} out");
        assert_eq!(hall_state(&t, hall).usage, used);
    }
}

/// Design change 0342's Verification: "At 19/20 with a Recruit in production, the HUD shows 20/20". The Hall's 19th
/// Recruit is out in tick 3648 and its 20th starts: 20/20 until it is out, and after.
#[test]
fn at_19_with_a_recruit_in_production_the_hud_shows_20_of_20() {
    let hall = ObjectId(1);
    let mut t = flat(40, 40).spawn(HALL, Some(P1), tile(18, 18)).start();
    t.run_until(19 * FRAMES + 1);
    assert_eq!(made(&t, hall).len(), 19);
    assert_eq!(progress(&t, hall), Some(100), "the 20th in production");
    assert_eq!(unit_control(&t, P1), (20, 20));
    assert_eq!(hall_state(&t, hall).usage, 20);
    t.run_until(20 * FRAMES + 1);
    assert_eq!((made(&t, hall).len(), progress(&t, hall)), (20, None));
    assert_eq!(unit_control(&t, P1), (20, 20));
}

/// The Recruitment Area of the Hall at tile (8, 8) is tiles (5, 5) to (14, 14). Water on 50 of them, none right
/// against the footprint (rows 5, 6, 13 and 14, column 5 of rows 7 to 12, and column 14 of rows 7 to 10), so that every
/// side is open.
fn half_water(x: i32, y: i32) -> bool {
    let rows = [5, 6, 13, 14].contains(&y) && (5..=14).contains(&x);
    let west = x == 5 && (7..=12).contains(&y);
    let east = x == 14 && (7..=10).contains(&y);
    rows || west || east
}

/// Plan acceptance, 50 tiles: "capacity 10, 384 ticks a Recruit" (`factions/cults/structures/recruitment_hall.md >
/// BaseUnitControlCapacity - 20`: "at 50% effectiveness, capacity = 10";
/// `factions/cults/structures/recruitment_hall.md > RecruitmentHallInstanceState`: "CurrentProductionTime: number
/// (BaseRecruitProductionTime / Effectiveness)"). Water isn't Recruitable, so it lowers the effectiveness.
#[test]
fn a_hall_at_fifty_tiles_makes_ten_at_384_ticks_each() {
    let hall = ObjectId(1);
    let mut t = cults(&with_water(24, 20, half_water))
        .spawn(HALL, Some(P1), tile(8, 8))
        .start();
    t.run(1);
    let state = hall_state(&t, hall);
    assert_eq!(
        (state.claimed, state.tiles, state.effectiveness_percent()),
        (50, 100, 50)
    );
    assert_eq!(state.capacity.floor(), 10);
    assert_eq!(progress(&t, hall), Some(50), "50 tiles a frame");
    t.run_until(5000);
    assert_eq!(ticks(&t, hall), (1..=10).map(|k| 2 * FRAMES * k).collect::<Vec<_>>());
    assert_eq!(unit_control(&t, P1), (10, 10));
    assert_eq!(progress(&t, hall), None);
}

/// `factions/cults/structures/recruitment_hall.md > TileClaiming`: "Buildings do not affect recruitability". Storages
/// of its own and another player's Hall's footprint in its area leave it 100 tiles, and it recruits at 192 ticks;
/// the other Hall, completed later (a debug spawn's, in tick 1), claims what is left of its own area (design change
/// 0341: "The Hall completed first wins"; "Different players' Halls compete for tiles").
#[test]
fn buildings_in_the_area_take_no_tile() {
    let (hall, theirs) = (ObjectId(1), ObjectId(4));
    let mut t = flat(40, 30)
        .spawn(HALL, Some(P1), tile(8, 8))
        .spawn(ObjectKind::Storage, Some(P1), tile(5, 5))
        .spawn(ObjectKind::Storage, Some(P1), tile(12, 12))
        .start();
    // Its area is tiles (11, 5) to (20, 14): 4 columns of 10 tiles are the first Hall's.
    t.debug_at(
        1,
        DebugCommand::Spawn {
            kind: HALL,
            owner: Some(P2),
            at: tile(14, 8),
            orientation: Default::default(),
        },
    );
    t.run_until(FRAMES);
    assert_eq!(hall_state(&t, hall).claimed, 100);
    assert_eq!(hall_state(&t, theirs).claimed, 60);
    assert_eq!(ticks(&t, hall), [FRAMES]);
    assert_eq!(unit_control(&t, P2), (1, 12), "its Recruit in production");
}

/// The Recruitment Area of the Hall at tile (8, 8) with 33 Recruitable tiles: Water but for its footprint, 16 tiles,
/// and 17 west of it, by side B: columns 5 to 7 of rows 7 to 11, and columns 6 and 7 of row 12.
fn a_third_water(x: i32, y: i32) -> bool {
    let area = (5..=14).contains(&x) && (5..=14).contains(&y);
    let footprint = (8..=11).contains(&x) && (8..=11).contains(&y);
    let west = ((5..=7).contains(&x) && (7..=11).contains(&y)) || ((6..=7).contains(&x) && y == 12);
    area && !footprint && !west
}

/// Design change 0341's Verification: "A Hall with 33 claimed tiles stops at 6 Recruits, and the HUD shows 6"
/// (`factions/cults/structures/recruitment_hall.md > TileClaiming`: "LocalUnitControlCapacity is rounded down
/// everywhere (production and HUD): 33 claimed tiles of 100 gives 6.6, so 6"). Its Recruits take 582 ticks each, ⌈19200
/// / 33⌉, and walk off west to the rally point.
#[test]
fn a_hall_with_33_tiles_stops_at_6_and_the_hud_shows_6() {
    let hall = ObjectId(1);
    let mut t = cults(&with_water(24, 20, a_third_water))
        .spawn(HALL, Some(P1), tile(8, 8))
        .start();
    let west = Target::Point(Vec2::new(64.0, 640.0));
    t.at(1, P1, production(&[hall], ProductionCommand::SetRally(west)));
    t.run(1);
    let state = hall_state(&t, hall);
    assert_eq!((state.claimed, state.capacity.floor()), (33, 6));
    assert_eq!(unit_control(&t, P1), (1, 6));
    t.run_until(10_000);
    assert_eq!(ticks(&t, hall), (1..=6).map(|k| 582 * k).collect::<Vec<_>>());
    assert_eq!(progress(&t, hall), None, "no 7th");
    assert_eq!(hall_state(&t, hall).usage, 6);
    assert_eq!(unit_control(&t, P1), (6, 6));
}
