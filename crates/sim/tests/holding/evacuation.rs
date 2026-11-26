//! Evacuation on destruction (`factions/cults/economy.md > Cults Building Mechanics > Evacuation on Destruction`:
//! "Everyone inside is evacuated alive"; "Units come out unchanged"; design change 0349, slice M5-F3: "evacuated units
//! **all appear at the same time inside its former footprint**, unharmed, spread over it. Units that overlap ignore
//! each other until separated, and idle ones push apart"; "Afterwards evacuated units idle"). A debug removal stands
//! for the destruction: both are the one way objects leave the game (`Sim::remove_objects`), and the removal happens
//! before the movement of its tick, so that the evacuees may take their first step apart in it.

use sc_sim::{Map, Tile, TileLook, TilePreset};

use super::*;

/// The centres of [`base`]'s Armory's tiles (x 768..960, y 640..768), in the order the evacuees take them: row by row
/// from the north-west tile, west to east.
const TILES: [Vec2; 6] = [
    Vec2::new(800.0, 672.0),
    Vec2::new(864.0, 672.0),
    Vec2::new(928.0, 672.0),
    Vec2::new(800.0, 736.0),
    Vec2::new(864.0, 736.0),
    Vec2::new(928.0, 736.0),
];

/// How far an idle unit steps off another in a tick (`tuning::movement::SEPARATE_SPEED_SU`).
const SEPARATE_SPEED: f32 = 2.0;

/// Player 1's Recruits ordered into [`A1`] at tick 2, run until they all are in; they are in the order they went in,
/// longest inside first.
fn fill(t: &mut TestSim, recruits: &[ObjectId]) -> Vec<ObjectId> {
    t.at(2, P1, enter(recruits, A1));
    run_checked(t, 2);
    while inside(t, A1).len() < recruits.len() {
        assert!(t.sim().tick() < 200, "{} are in", inside(t, A1).len());
        step_checked(t);
    }
    inside(t, A1)
}

/// The events of tick `tick`, in the order the sim emitted them.
fn events_of(t: &TestSim, tick: Tick) -> Vec<SimEvent> {
    (t.events().iter())
        .filter(|&&(at, _)| at == tick)
        .map(|(_, e)| e.clone())
        .collect()
}

/// The units evacuated out of `building` so far, in order.
fn evacuated(t: &TestSim, building: ObjectId) -> Vec<ObjectId> {
    (t.events().iter())
        .filter_map(|(_, e)| match e {
            SimEvent::Evacuated { unit, building: from } if *from == building => Some(*unit),
            _ => None,
        })
        .collect()
}

/// Whether a Recruit at `pos` stands inside [`base`]'s Armory's former footprint, its whole box.
fn on_footprint(pos: Vec2) -> bool {
    pos.x - HALF.x >= 768.0 && pos.x + HALF.x <= 960.0 && pos.y - HALF.y >= 640.0 && pos.y + HALF.y <= 768.0
}

/// The pairs of `units`, Recruits all, whose boxes overlap: touching is no overlap.
fn overlapping(t: &TestSim, units: &[ObjectId]) -> Vec<(ObjectId, ObjectId)> {
    let pos = |unit: ObjectId| t.sim().object(unit).expect("on the map").pos;
    let mut pairs = Vec::new();
    for (i, &a) in units.iter().enumerate() {
        for &b in &units[i + 1..] {
            let d = (pos(a) - pos(b)).abs();
            if d.x < 2.0 * HALF.x && d.y < 2.0 * HALF.y {
                pairs.push((a, b));
            }
        }
    }
    pairs
}

/// Runs until no two of `units` overlap, at most `ticks` ticks.
fn until_apart(t: &mut TestSim, units: &[ObjectId], ticks: Tick) {
    let start = t.sim().tick();
    while !overlapping(t, units).is_empty() {
        assert!(
            t.sim().tick() < start + ticks,
            "still overlapping after {ticks} ticks: {:?}\n{}",
            overlapping(t, units),
            t.ascii(Some(Vec2::new(864.0, 704.0)))
        );
        step_checked(t);
    }
}

/// Each of `units` idle, on the map at `at` (in order), for `ticks` more ticks.
fn idle_where_they_are(t: &mut TestSim, units: &[ObjectId], ticks: Tick) {
    let at: Vec<Vec2> = units.iter().map(|&unit| t.sim().object(unit).unwrap().pos).collect();
    run_for(t, ticks);
    for (&unit, at) in units.iter().zip(at) {
        let o = t.sim().object(unit).expect("on the map");
        assert!(o.orders.as_ref().unwrap().is_idle(), "{unit:?} idle");
        assert_eq!(o.pos, at, "{unit:?} where it was");
    }
}

/// Design change 0349's Verification: "Destroy an Armory holding 10 Recruits: all 10 appear on its tiles in the same
/// frame, then spread." In the tick the Armory goes, after its `Removed`, every one of the ten comes out on the centre
/// of a tile of its footprint, the next tile in the footprint's order for each, longest inside first, round again
/// after the sixth: two on each of the first four tiles. Unharmed, idle, every box inside the former footprint. The
/// pairs on a tile overlap, and step apart: within half a second none does (each is 12 su off the middle at 2 su a
/// tick), and they idle where they parted, still on the footprint.
#[test]
fn ten_recruits_appear_on_the_armorys_tiles_at_once_then_spread() {
    // Player 1 keeps a Storage, and so isn't defeated with the Armory.
    let mut t = base().spawn(ObjectKind::Storage, Some(P1), tile(2, 2)).start(); // 2
    t.without_auto_targeting();
    let recruits = block(&mut t, 1, 10, 5, Vec2::new(1150.0, 704.0));
    let order = fill(&mut t, &recruits);
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, A1);
    run_checked(&mut t, gone);
    let events = events_of(&t, gone);
    let removed = events.iter().position(|e| *e == SimEvent::Removed(A1));
    let first = events.iter().position(|e| matches!(e, SimEvent::Evacuated { .. }));
    assert!(removed.is_some_and(|r| first.is_some_and(|f| r < f)), "{events:?}");
    assert_eq!(
        evacuated(&t, A1),
        order,
        "all ten in the tick it went, longest inside first"
    );
    assert!(t.sim().held().next().is_none());
    for (i, &unit) in order.iter().enumerate() {
        let o = t.sim().object(unit).expect("alive, on the map");
        let tile = TILES[i % TILES.len()];
        // The removal is the command phase's: the movement after it may have taken a first step apart.
        assert!(
            o.pos.distance(tile) <= SEPARATE_SPEED + 1e-3,
            "{unit:?} at {}, not on {tile}",
            o.pos
        );
        assert!(on_footprint(o.pos), "{unit:?} at {}", o.pos);
        assert_eq!((o.kind, o.owner, o.hp), (RECRUIT, Some(P1), Some(40.0)), "unchanged");
        assert!(o.orders.as_ref().unwrap().is_idle());
    }
    let stacked = overlapping(&t, &order);
    assert_eq!(stacked.len(), 4, "two on each of the first four tiles: {stacked:?}");
    until_apart(&mut t, &order, 8);
    for &unit in &order {
        let pos = t.sim().object(unit).unwrap().pos;
        assert!(on_footprint(pos), "{unit:?} at {pos}");
    }
    idle_where_they_are(&mut t, &order, 32);
    assert!(t.sim().player(P1).is_some_and(|p| !p.is_defeated()));
}

/// Inside the footprint, whatever stands round it: a Storage against side A (x 768..896), a unit of the enemy's at the
/// middle of side B, and the Recruit Eject All had just let into the doorway (half inside the footprint, at the middle
/// of side C), which stays where it is, an ordinary unit out of a doorway that is gone. The nine still inside come out
/// on the tiles in their order, and part; nothing round the footprint moves, and none of the nine overlaps it
/// (`TestSim`'s invariants hold throughout).
#[test]
fn evacuees_appear_inside_the_footprint_whatever_stands_round_it() {
    let mut t = base()
        .spawn(ObjectKind::Storage, Some(P1), tile(12, 8)) // 2: x 768..896, y 512..640
        .start();
    t.without_auto_targeting();
    let recruits = block(&mut t, 1, 10, 5, Vec2::new(1150.0, 704.0)); // 3..=12
    let order = fill(&mut t, &recruits);
    let eject = t.sim().tick() + 1;
    let enemy = id(next_id(&t));
    t.spawn_unit_at(eject, RECRUIT, P2, Vec2::new(984.0, 704.0));
    t.at(eject, P1, eject_all(&[A1]));
    run_checked(&mut t, eject);
    let leaving = order[0];
    assert_eq!(exits(&t), [(eject, leaving)]);
    let at = t.sim().object(leaving).unwrap().pos;
    assert!(at.distance(EXIT) < 0.5, "in the doorway: {at}");
    let gone = eject + 1;
    remove_at(&mut t, gone, A1);
    run_checked(&mut t, gone);
    let nine = &order[1..];
    assert_eq!(evacuated(&t, A1), nine, "the nine still inside, in order");
    for (i, &unit) in nine.iter().enumerate() {
        let pos = t.sim().object(unit).expect("alive, on the map").pos;
        assert!(
            pos.distance(TILES[i % TILES.len()]) <= SEPARATE_SPEED + 1e-3,
            "{unit:?} at {pos}"
        );
    }
    let mut round: Vec<ObjectId> = vec![leaving, enemy];
    let o = t.sim().object(leaving).unwrap();
    assert_eq!(o.mover.as_ref().unwrap().exit, None, "out of a doorway that is gone");
    let enemy_at = t.sim().object(enemy).unwrap().pos;
    until_apart(&mut t, nine, 8);
    round.extend_from_slice(nine);
    assert_eq!(overlapping(&t, &round), [], "nothing overlaps");
    for &unit in nine {
        assert!(on_footprint(t.sim().object(unit).unwrap().pos));
    }
    assert_eq!(t.sim().object(enemy).unwrap().pos, enemy_at);
    run_for(&mut t, 32);
}

/// "None may land on terrain units can't stand on" (the brief of slice M5-F3): the Armory's north-west tile is of a
/// map's own preset, buildable but not traversible, so that no Recruit can stand on it. The ten pass it over: two on
/// each of the other five tiles, in the footprint's order, and they part on those.
#[test]
fn no_evacuee_lands_on_ground_it_cant_stand_on() {
    let mut scenario = base().spawn(ObjectKind::Storage, Some(P1), tile(2, 2)).build();
    let terrain: &mut Map = &mut scenario.map.terrain;
    let paving = terrain.add_preset(TilePreset {
        name: "Paving".into(),
        buildable: true,
        traversible: false,
        rugged: false,
        drillable: false,
        recruitable: false,
        look: TileLook::Plane,
    });
    terrain.set_tile(TilePos::new(12, 10), Tile::new(paving));
    let mut t = TestSim::new(scenario);
    t.without_auto_targeting();
    let recruits = block(&mut t, 1, 10, 5, Vec2::new(1150.0, 704.0));
    let order = fill(&mut t, &recruits);
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, A1);
    run_checked(&mut t, gone);
    assert_eq!(evacuated(&t, A1), order);
    for (i, &unit) in order.iter().enumerate() {
        let pos = t.sim().object(unit).expect("alive, on the map").pos;
        let tile = TILES[1 + i % 5];
        assert!(
            pos.distance(tile) <= SEPARATE_SPEED + 1e-3,
            "{unit:?} at {pos}, not on {tile}"
        );
    }
    until_apart(&mut t, &order, 8);
    // Off the paved tile (x 768..832, y 640..704) all the while, and still on the footprint.
    for &unit in &order {
        let pos = t.sim().object(unit).unwrap().pos;
        let off_paving = pos.x - HALF.x >= 832.0 || pos.y - HALF.y >= 704.0;
        assert!(on_footprint(pos) && off_paving, "{unit:?} at {pos}");
    }
    idle_where_they_are(&mut t, &order, 32);
}

/// Before the defeat check (`economy::defeat`, at the end of the economy phase): with the Armory, player 1's last
/// structure, goes the player, but its Recruits come out first, alive, on the footprint, and turn neutral with its
/// other units (`core/game_rules.md > On Defeat`) instead of being lost inside.
#[test]
fn evacuees_come_out_before_their_owner_is_defeated() {
    let mut t = base().spawn(ARMORY, Some(P2), tile(20, 4)).start(); // 2
    t.without_auto_targeting();
    let recruits = block(&mut t, 1, 3, 3, Vec2::new(1150.0, 704.0)); // 3..=5
    let order = fill(&mut t, &recruits);
    let gone = t.sim().tick() + 1;
    remove_at(&mut t, gone, A1);
    run_checked(&mut t, gone);
    assert!(t.sim().player(P1).is_some_and(|p| p.is_defeated()));
    assert_eq!(evacuated(&t, A1), order);
    for (&unit, tile) in order.iter().zip(TILES) {
        let o = t.sim().object(unit).expect("alive, on the map");
        assert_eq!((o.owner, o.pos), (None, tile), "neutral, on its own tile");
    }
    let lost = (t.events().iter()).any(|(_, e)| matches!(e, SimEvent::Removed(unit) if order.contains(unit)));
    assert!(!lost);
    run_for(&mut t, 32);
}
