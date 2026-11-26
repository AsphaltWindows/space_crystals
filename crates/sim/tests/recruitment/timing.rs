//! How fast a Hall recruits, and up to what (`factions/cults/economy.md > RecruitmentHall > Effectiveness`,
//! `factions/cults/economy.md > RecruitmentHall > BaseUnitControlCapacity - 20`, `factions/cults/economy.md >
//! RecruitmentHall > BaseRecruitProductionTime - 12 seconds (192 frames) per Recruit`).

use sc_sim::ObjectId;

use super::*;

/// Plan acceptance, 100%: "6 → 20 Recruits in 2688 ticks, then none" (`factions/cults/economy.md > RecruitmentHall >
/// BaseRecruitProductionTime - 12 seconds (192 frames) per Recruit`: "a starting hall (6 Recruits → 20) fills in 14 ×
/// 12 s"). A Hall on open Plane claims its 100 tiles and makes a Recruit every 192 ticks from tick 1, the k-th in tick
/// 192 k, each of its lineage; at 20 it stops, nothing in production, and the HUD shows 20/20. A Recruit counts from
/// the tick it exists (**Placeholder (Q0100 point 3)**).
#[test]
fn a_full_hall_fills_from_6_to_20_in_2688_ticks_then_stops() {
    let hall = ObjectId(1);
    let mut t = flat(40, 40).spawn(HALL, Some(P1), tile(18, 18)).start();
    t.run_until(FRAMES - 1);
    assert_eq!(unit_control(&t, P1), (0, 20), "the Recruit in production isn't counted");
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

/// The 6 → 20 of the starting Hall (`factions/cults/economy.md > Starting Conditions`), counted from its 6th Recruit:
/// the HUD's used figure climbs by one each 192 ticks.
#[test]
fn the_hud_counts_each_recruit_as_it_comes_out() {
    let hall = ObjectId(1);
    let mut t = flat(40, 40).spawn(HALL, Some(P1), tile(18, 18)).start();
    for k in 1..=20 {
        t.run_until(FRAMES * k - 1);
        assert_eq!(unit_control(&t, P1), (k - 1, 20), "before Recruit {k}");
        t.run(1);
        assert_eq!(unit_control(&t, P1), (k, 20), "Recruit {k}");
        assert_eq!(hall_state(&t, hall).usage, k);
    }
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

/// Plan acceptance, 50 tiles: "capacity 10, 384 ticks a Recruit" (`factions/cults/economy.md > RecruitmentHall >
/// BaseUnitControlCapacity - 20`: "at 50% effectiveness, capacity = 10";
/// `factions/cults/economy.md > RecruitmentHall > RecruitmentHallInstanceState`: "CurrentProductionTime: number
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

/// `factions/cults/economy.md > RecruitmentHall > TileClaiming`: "Buildings do not affect recruitability". Storages
/// of its own and another player's Hall's footprint in its area leave it 100 tiles, and it recruits at 192 ticks;
/// the other Hall, completed later, claims what is left of its own area (**Placeholder (Q0099 point 3)**).
#[test]
fn buildings_in_the_area_take_no_tile() {
    let (hall, theirs) = (ObjectId(1), ObjectId(4));
    let mut t = flat(40, 30)
        .spawn(HALL, Some(P1), tile(8, 8))
        .spawn(ObjectKind::Storage, Some(P1), tile(5, 5))
        .spawn(ObjectKind::Storage, Some(P1), tile(12, 12))
        // Its area is tiles (11, 5) to (20, 14): 4 columns of 10 tiles are the first Hall's.
        .spawn(HALL, Some(P2), tile(14, 8))
        .start();
    t.run_until(FRAMES);
    assert_eq!(hall_state(&t, hall).claimed, 100);
    assert_eq!(hall_state(&t, theirs).claimed, 60);
    assert_eq!(ticks(&t, hall), [FRAMES]);
    assert_eq!(unit_control(&t, P2), (0, 12));
}
