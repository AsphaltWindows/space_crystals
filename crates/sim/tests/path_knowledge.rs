//! Path knowledge (`systems/controls/behaviors.md > PathKnowledge`; the verification points of design changes 0040
//! and 0042): paths are planned only from what the unit's owner knows, units collide with what is really there, and
//! a unit re-routes when its owner discovers an obstacle. Design change 0038's own points are in
//! `resources_block.rs`.
//!
//! The maps here are 40 × 20 with a mountain wall along column 25, a gap in it at rows 6 and 7 and open ground at
//! its south end, rows 18 and 19: the gap is the short way east, the south end the long one.

use sc_sim::nav::NavClass;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, SU_PER_GU, Sim, SimEvent, SpawnAt, Target,
    TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper;
/// The Peacekeeper's half silhouette (24 × 24 su).
const HALF: f32 = 12.0;
/// The column of the wall, and the two rows of its gap.
const WALL: i32 = 25;
const GAP: i32 = 6;
/// The first row of the open ground at the wall's south end.
const SOUTH: i32 = 18;

/// The 40 × 20 map: with the wall, or all open ground.
fn map(wall: bool) -> String {
    (0..20)
        .map(|y| {
            (0..40)
                .map(|x| {
                    let solid = wall && x == WALL && y < SOUTH && y != GAP && y != GAP + 1;
                    if solid { 'M' } else { '.' }
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

/// A point given in tiles: `(25.5, 7.0)` is the middle of the gap.
fn at(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y) * SU_PER_GU
}

/// West of the wall, out of sight of the gap; on the gap's line east of the wall, out of sight of it too; and
/// beside the gap, in sight of it.
const WEST: (f32, f32) = (8.5, 7.0);
const EAST: (f32, f32) = (36.5, 7.0);
const BY_THE_GAP: (f32, f32) = (29.5, 7.0);

fn point((x, y): (f32, f32)) -> Vec2 {
    at(x, y)
}

fn move_to(t: &mut TestSim, unit: u32, to: Vec2) {
    let tick = t.sim().tick() + 1;
    t.order_at(tick, P1, &[ObjectId(unit)], OrderKind::Move(Target::Point(to)), false);
}

/// Whether the unit's box at `pos` overlaps the footprint `(x, y, w, h)` in tiles. Touching is free.
fn overlaps(pos: Vec2, (x, y, w, h): (i32, i32, i32, i32)) -> bool {
    let lo = Vec2::new(x as f32, y as f32) * SU_PER_GU;
    let hi = lo + Vec2::new(w as f32, h as f32) * SU_PER_GU;
    pos.x - HALF < hi.x && pos.x + HALF > lo.x && pos.y - HALF < hi.y && pos.y + HALF > lo.y
}

/// Steps until unit `id` no longer moves, and returns where it was at the end of each tick.
fn walk(t: &mut TestSim, id: u32) -> Vec<Vec2> {
    let mut track = vec![t.step().pos(id)];
    while t.status(id) == MoveStatus::Moving {
        assert!(track.len() < 2000, "still moving after 2000 ticks");
        track.push(t.step().pos(id));
    }
    track
}

fn went_south(track: &[Vec2]) -> bool {
    track.iter().any(|p| p.y > SOUTH as f32 * SU_PER_GU)
}

fn went_through_the_gap(track: &[Vec2]) -> bool {
    track
        .iter()
        .any(|&p| TilePos::containing(p).x == WALL && (GAP..GAP + 2).contains(&TilePos::containing(p).y))
}

fn version(sim: &Sim, player: PlayerId) -> u32 {
    sim.path_knowledge().version(player).unwrap()
}

fn failed_moves(t: &TestSim) -> usize {
    t.events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::MoveFailed { .. }))
        .count()
}

/// Whether `player` plans around tile `(x, y)`: its known grid blocks it.
fn known_blocked(sim: &Sim, player: PlayerId, x: i32, y: i32) -> bool {
    sim.known_nav(player)
        .unwrap()
        .grid(NavClass::GroundRugged, 1)
        .unwrap()
        .is_blocked(TilePos::new(x, y))
}

/// Design change 0040, first verification: "Build a hidden wall of enemy buildings across a gap in fog. Order a unit
/// through the gap: it heads for the gap, sees the wall, and re-routes. It never avoids the gap before seeing the
/// wall." No move fails on the way.
#[test]
fn a_unit_heads_for_a_hidden_wall_sees_it_and_re_routes() {
    let mut t = ScenarioBuilder::new(&map(true))
        .spawn(ObjectKind::Generator, Some(P2), tile(WALL, GAP)) // 1: seals the gap, tiles (25, 6) to (26, 7)
        .spawn(PK, Some(P1), SpawnAt::Point(at(5.5, 7.0))) // 2
        .start();
    t.without_auto_targeting();
    let generator = (WALL, GAP, 2, 2);
    assert!(
        t.sim()
            .nav()
            .grid(NavClass::GroundRugged, 1)
            .unwrap()
            .is_blocked(TilePos::new(WALL, GAP))
    );
    assert!(
        !known_blocked(t.sim(), P1, WALL, GAP),
        "never seen: open ground for player 1"
    );

    let goal = point(EAST);
    move_to(&mut t, 2, goal);
    let mut track = Vec::new();
    // Until its owner sees the Generator, the unit walks straight at the gap, on the one straight leg it planned.
    while version(t.sim(), P1) == 0 {
        assert!(t.sim().tick() < 400);
        track.push(t.step().pos(2));
        assert_eq!(t.mover(2).path, [at(5.5, 7.0), goal], "tick {}", t.sim().tick());
        assert_eq!(t.pos(2).y, at(0.0, 7.0).y, "straight at the gap");
    }
    let seen_from = t.pos(2);
    let record = t.sim().vision().player(P1).unwrap().record(ObjectId(1));
    assert!(record.is_some(), "seen: remembered");
    // The SightRange is 5 GU to a tile's centre: the west column of the Generator, 5.5 tiles east of the unit.
    let reach = at(25.5, 6.5).distance(seen_from) / SU_PER_GU;
    assert!((4.9..=5.0).contains(&reach), "seen from {seen_from}, {reach} GU away");
    assert!(known_blocked(t.sim(), P1, WALL, GAP));
    // Within its next staggered check it plans again, around the wall's south end, and gets there.
    t.run(4);
    assert!(t.mover(2).path.len() > 2, "{:?}", t.mover(2).path);
    track.extend(walk(&mut t, 2));
    assert_eq!((t.status(2), t.pos(2)), (MoveStatus::Arrived, goal));
    assert!(went_south(&track) && !went_through_the_gap(&track));
    assert!(track.iter().all(|&p| !overlaps(p, generator)));
    assert_eq!(failed_moves(&t), 0);
    assert_eq!(
        t.mover(2).repaths_left,
        sc_sim::mover::MAX_REPATHS,
        "the re-route isn't a counted repath"
    );
}

/// `> PathKnowledge`: "Own structures are always known." The same wall of the player's own: its unit goes around
/// from the first step.
#[test]
fn a_unit_plans_around_its_owners_own_structures_from_the_start() {
    let mut t = ScenarioBuilder::new(&map(true))
        .spawn(ObjectKind::Generator, Some(P1), tile(WALL, GAP)) // 1
        .spawn(PK, Some(P1), SpawnAt::Point(at(5.5, 7.0))) // 2
        .start();
    assert!(known_blocked(t.sim(), P1, WALL, GAP));
    move_to(&mut t, 2, point(EAST));
    t.step();
    assert!(t.mover(2).path.len() > 2, "{:?}", t.mover(2).path);
    let track = walk(&mut t, 2);
    assert_eq!((t.status(2), t.pos(2)), (MoveStatus::Arrived, point(EAST)));
    assert!(went_south(&track) && !went_through_the_gap(&track));
}

/// What stands in the gap of [`remembered_until_seen_gone`].
enum InTheGap {
    /// An enemy Generator the unit scouts first (design change 0040).
    ScoutedGenerator,
    /// Two crystal patches of the map, which the player never sees (design change 0042).
    MapPatches,
}

/// The second verification of design changes 0040 and 0042: something the player knows of in the gap goes away out
/// of its sight. Its unit, sent east, still routes around the wall's south end; once it has seen the gap empty,
/// the way back leads through it.
fn remembered_until_seen_gone(in_the_gap: InTheGap) {
    let builder = ScenarioBuilder::new(&map(true));
    let (builder, blockers, start) = match in_the_gap {
        InTheGap::ScoutedGenerator => (
            builder.spawn(ObjectKind::Generator, Some(P2), tile(WALL, GAP)), // 1
            vec![1],
            // In sight of the Generator: it is scouted from the start.
            at(21.5, 7.0),
        ),
        InTheGap::MapPatches => (
            builder
                .spawn(ObjectKind::CrystalPatch, None, tile(WALL, GAP)) // 1
                .spawn(ObjectKind::CrystalPatch, None, tile(WALL, GAP + 1)), // 2
            vec![1, 2],
            point(WEST),
        ),
    };
    let unit = blockers.len() as u32 + 1;
    let builder = builder.spawn(PK, Some(P1), SpawnAt::Point(start));
    let builder = match in_the_gap {
        // Player 2 keeps a building far from the wall, never in player 1's sight: losing the one in the gap doesn't
        // defeat them (plan decision T8), and the game goes on.
        InTheGap::ScoutedGenerator => builder.spawn(ObjectKind::Generator, Some(P2), tile(0, 0)),
        InTheGap::MapPatches => builder,
    };
    let mut t = builder.start();
    t.without_auto_targeting();
    let sight = |t: &TestSim, id: u32| t.sim().vision().player(P1).unwrap().record(ObjectId(id)).is_some();
    let assumed = |t: &TestSim| t.sim().path_knowledge().player(P1).unwrap().assumed().len();
    match in_the_gap {
        InTheGap::ScoutedGenerator => assert!(sight(&t, 1) && assumed(&t) == 0),
        // Known from the start like terrain, without a record: the fog shows nothing there.
        InTheGap::MapPatches => assert!(!sight(&t, 1) && !sight(&t, 2) && assumed(&t) == 2),
    }
    assert!(known_blocked(t.sim(), P1, WALL, GAP) && known_blocked(t.sim(), P1, WALL, GAP + 1));

    // Away to the west, out of sight; then the gap is cleared behind the player's back.
    move_to(&mut t, unit, point(WEST));
    walk(&mut t, unit);
    assert_eq!(t.pos(unit), point(WEST));
    let tick = t.sim().tick() + 1;
    for &id in &blockers {
        t.debug_at(tick, DebugCommand::Remove { id: ObjectId(id) });
    }
    t.step();
    assert!(blockers.iter().all(|&id| t.sim().object(ObjectId(id)).is_none()));
    let truth = t.sim().nav().grid(NavClass::GroundRugged, 1).unwrap();
    assert!(!truth.is_blocked(TilePos::new(WALL, GAP)) && !truth.is_blocked(TilePos::new(WALL, GAP + 1)));
    assert!(known_blocked(t.sim(), P1, WALL, GAP) && known_blocked(t.sim(), P1, WALL, GAP + 1));
    let before = version(t.sim(), P1);

    // East: the long way around, though the gap is open.
    move_to(&mut t, unit, point(EAST));
    let track = walk(&mut t, unit);
    assert_eq!((t.status(unit), t.pos(unit)), (MoveStatus::Arrived, point(EAST)));
    assert!(went_south(&track) && !went_through_the_gap(&track));
    assert_eq!(version(t.sim(), P1), before, "it never saw the gap");
    assert!(known_blocked(t.sim(), P1, WALL, GAP));
    match in_the_gap {
        InTheGap::ScoutedGenerator => assert!(sight(&t, 1), "still remembered"),
        InTheGap::MapPatches => assert_eq!(assumed(&t), 2, "still assumed"),
    }

    // A look at the gap: it is empty.
    move_to(&mut t, unit, point(BY_THE_GAP));
    walk(&mut t, unit);
    assert!(version(t.sim(), P1) > before);
    assert!(!known_blocked(t.sim(), P1, WALL, GAP) && !known_blocked(t.sim(), P1, WALL, GAP + 1));
    assert!(blockers.iter().all(|&id| !sight(&t, id)) && assumed(&t) == 0);

    // And back west, through it.
    move_to(&mut t, unit, point(WEST));
    let track = walk(&mut t, unit);
    assert_eq!((t.status(unit), t.pos(unit)), (MoveStatus::Arrived, point(WEST)));
    assert!(went_through_the_gap(&track) && !went_south(&track));
    assert_eq!(failed_moves(&t), 0);
}

/// Design change 0040, second verification: "Scout an enemy building, leave, have it destroyed out of sight, then
/// path through its spot: the unit routes around the remembered building until it sees the spot is empty."
#[test]
fn a_scouted_building_destroyed_out_of_sight_is_routed_around_until_seen_gone() {
    remembered_until_seen_gone(InTheGap::ScoutedGenerator);
}

/// Design change 0042, second verification: "A patch mined out in the fog is still routed around until a unit sees
/// its tile." A debug removal stands in for the mining (slice M2-B: a depleted patch goes with its plate).
#[test]
fn a_patch_gone_in_the_fog_is_routed_around_until_its_tile_is_seen() {
    remembered_until_seen_gone(InTheGap::MapPatches);
}

/// Design change 0042, first verification: "A unit ordered across unexplored ground routes around crystal fields it
/// has never seen." A field of 18 patches down column 25 of open ground, 17 tiles from the unit: its first plan
/// already leads around the field's south end.
#[test]
fn a_unit_routes_around_a_crystal_field_it_has_never_seen() {
    let mut builder = ScenarioBuilder::new(&map(false));
    for y in 0..SOUTH {
        builder = builder.spawn(ObjectKind::CrystalPatch, None, tile(WALL, y)); // 1–18
    }
    let mut t = builder.spawn(PK, Some(P1), SpawnAt::Point(point(WEST))).start(); // 19
    let knowledge = |t: &TestSim| {
        let assumed = t.sim().path_knowledge().player(P1).unwrap().assumed().len();
        (assumed, t.sim().vision().player(P1).unwrap().records().count())
    };
    assert_eq!(knowledge(&t), (18, 0), "all assumed, none seen");
    assert!(
        t.sim().path_knowledge().player(P2).unwrap().assumed().len() == 18,
        "a player without units knows the map too"
    );
    assert!(
        t.sim().known_nav(P2).is_none(),
        "but has no grids while nothing of its plans"
    );
    for y in 0..SOUTH {
        assert!(known_blocked(t.sim(), P1, WALL, y));
    }
    move_to(&mut t, 19, point(EAST));
    t.step();
    let path = t.mover(19).path.clone();
    assert!(
        path.len() > 2 && path.iter().any(|p| p.y > SOUTH as f32 * SU_PER_GU),
        "{path:?}"
    );
    let track = walk(&mut t, 19);
    assert_eq!((t.status(19), t.pos(19)), (MoveStatus::Arrived, point(EAST)));
    assert!(went_south(&track));
    assert!(track.iter().all(|&p| !overlaps(p, (WALL, 0, 1, SOUTH))));
    // Each patch is assumed until seen, then remembered: the unit passed the south end, and saw the patches there.
    let (assumed, seen) = knowledge(&t);
    assert_eq!(assumed + seen, 18);
    assert!(seen > 0 && assumed > 0, "{assumed} assumed, {seen} seen");
    assert_eq!(failed_moves(&t), 0);
}

/// The plan's collision point: "An enemy structure never seen is open ground for planning, but no unit ever walks
/// into it, because collision uses the true grid." On open ground the unit plans straight through the Generator,
/// re-routes when it sees it, and passes beside it. A goal on the Generator itself ends at its wall.
#[test]
fn an_unseen_structure_is_open_ground_for_plans_but_solid() {
    let generator = (15, 5, 2, 2);
    let start = at(4.5, 6.0);
    let scene = || {
        let mut t = ScenarioBuilder::new(&map(false))
            .spawn(ObjectKind::Generator, Some(P2), tile(15, 5)) // 1: tiles (15, 5) to (16, 6)
            .spawn(PK, Some(P1), SpawnAt::Point(start)) // 2
            .start();
        t.without_auto_targeting();
        t
    };
    let mut t = scene();
    let beyond = at(26.5, 6.0);
    move_to(&mut t, 2, beyond);
    t.step();
    assert_eq!(
        t.mover(2).path,
        [start, beyond],
        "straight through the footprint it doesn't know"
    );
    let track = walk(&mut t, 2);
    assert_eq!((t.status(2), t.pos(2)), (MoveStatus::Arrived, beyond));
    assert!(track.iter().all(|&p| !overlaps(p, generator)));
    assert!(
        track.iter().any(|p| (p.y - start.y).abs() >= SU_PER_GU + HALF - 1.0),
        "around it"
    );
    assert_eq!(failed_moves(&t), 0);

    // A goal inside the footprint: the unit walks up to the wall and has arrived there, the nearest point it can
    // reach (design change 0025).
    let mut t = scene();
    let inside = at(16.0, 6.0);
    move_to(&mut t, 2, inside);
    let track = walk(&mut t, 2);
    assert_eq!(t.status(2), MoveStatus::Arrived);
    assert!(track.iter().all(|&p| !overlaps(p, generator)));
    assert!(t.pos(2).distance(inside) < 1.6 * SU_PER_GU, "at the wall: {}", t.pos(2));
    assert_eq!(failed_moves(&t), 0);
}

/// Collision never waits for knowledge: a Generator put right in front of a walking unit (8 su ahead) stops it at
/// once, before its owner's knowledge and its next staggered check catch up; then it walks around. `TestSim` checks
/// invariant I2 (no box on a blocked tile of the true grid) after every tick of every test here.
#[test]
fn a_structure_appearing_in_front_of_a_unit_stops_it_at_once() {
    let mut t = ScenarioBuilder::new(&map(false))
        .spawn(PK, Some(P1), SpawnAt::Point(at(4.5, 6.0))) // 1
        .start();
    t.without_auto_targeting();
    let goal = at(20.5, 6.0);
    t.order_at(1, P1, &[ObjectId(1)], OrderKind::Move(Target::Point(goal)), false);
    t.run(19);
    assert_eq!(t.pos(1), at(4.5, 6.0) + Vec2::new(76.0, 0.0), "4 su per tick");
    t.debug_at(
        20,
        DebugCommand::Spawn {
            kind: ObjectKind::Generator,
            owner: Some(P2),
            at: tile(6, 5), // tiles (6, 5) to (7, 6): its west wall is 8 su from the unit's box
            orientation: Default::default(),
        },
    );
    let track = walk(&mut t, 1);
    assert!(track.iter().all(|&p| !overlaps(p, (6, 5, 2, 2))));
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Arrived, goal));
    assert_eq!(failed_moves(&t), 0);
}

/// The players' grids are built lazily: a player gets them with its first unit that walks on a grid, and an order
/// in the very tick of that unit's spawn already plans on what the player knows.
#[test]
fn a_players_grids_are_built_with_its_first_walking_unit() {
    let mut t = ScenarioBuilder::new(&map(true))
        .spawn(ObjectKind::Generator, Some(P2), tile(WALL, GAP)) // 1
        .spawn(PK, Some(P1), SpawnAt::Point(point(WEST))) // 2
        .start();
    t.without_auto_targeting();
    assert!(t.sim().known_nav(P1).is_some() && t.sim().known_nav(P2).is_none());
    t.run(3);
    assert_eq!(t.sim().nav_rebuilds(), 0);
    // Player 2's first unit, ordered east of its own Generator in the tick it appears.
    t.spawn_unit_at(4, PK, P2, at(20.5, 7.0));
    t.order_at(
        4,
        P2,
        &[ObjectId(3)],
        OrderKind::Move(Target::Point(point(EAST))),
        false,
    );
    t.step();
    assert!(known_blocked(t.sim(), P2, WALL, GAP));
    assert_eq!(t.sim().nav_rebuilds(), 0, "built for the order, not rebuilt");
    // A unit acts from the tick after its spawn: its first plan.
    t.step();
    assert!(
        t.mover(3).path.len() > 2,
        "around its own Generator from the start: {:?}",
        t.mover(3).path
    );
    let track = walk(&mut t, 3);
    assert_eq!((t.status(3), t.pos(3)), (MoveStatus::Arrived, point(EAST)));
    assert!(went_south(&track) && !went_through_the_gap(&track));
}

/// Determinism: the grids are derived and rebuilt on load, the assumed resources and the versions are state. A
/// save at any point of a game with stale knowledge loads equal (state, hash and each player's grids) and
/// continues tick for tick like the game it was taken from; and the game replays to the same state.
#[test]
fn a_save_with_stale_knowledge_loads_and_continues_identically() {
    let game = || {
        let mut t = ScenarioBuilder::new(&map(true))
            .spawn(ObjectKind::Generator, Some(P2), tile(WALL, GAP)) // 1
            .spawn(ObjectKind::CrystalPatch, None, tile(30, 12)) // 2
            .spawn(ObjectKind::CrystalPatch, None, tile(3, 15)) // 3
            .spawn(PK, Some(P1), SpawnAt::Point(at(21.5, 7.0))) // 4
            .spawn(PK, Some(P2), SpawnAt::Point(at(33.5, 9.0))) // 5
            // Player 2's other building, never in player 1's sight: losing the Generator in the gap doesn't defeat
            // them (plan decision T8), and the game goes on.
            .spawn(ObjectKind::Generator, Some(P2), tile(0, 0)) // 6
            .start();
        t.without_auto_targeting();
        let order = |unit: u32, to: (f32, f32)| (unit, OrderKind::Move(Target::Point(point(to))));
        // Player 1 scouts the Generator, leaves, comes back the long way and looks at the gap; player 2 crosses
        // the map past the patches. The Generator goes while nobody of player 1 looks.
        for (tick, (unit, kind)) in [
            (1, order(4, WEST)),
            (2, order(5, (5.5, 15.0))),
            (260, order(4, EAST)),
            (900, order(4, BY_THE_GAP)),
            (1000, order(4, WEST)),
        ] {
            let player = if unit == 4 { P1 } else { P2 };
            t.order_at(tick, player, &[ObjectId(unit)], kind, false);
        }
        t.debug_at(250, DebugCommand::Remove { id: ObjectId(1) });
        t.debug_at(255, DebugCommand::Remove { id: ObjectId(3) });
        t
    };
    let mut t = game();
    let mut versions = Vec::new();
    for k in [0, 5, 249, 251, 400, 899, 960, 1100] {
        t.run_until(k);
        let json = serde_json::to_string(t.sim()).unwrap();
        let mut loaded: Sim = serde_json::from_str(&json).unwrap();
        assert_eq!(&loaded, t.sim(), "tick {k}");
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {k}");
        assert_eq!(loaded.path_knowledge(), t.sim().path_knowledge(), "tick {k}");
        for player in [P1, P2] {
            assert_eq!(
                loaded.known_nav(player),
                t.sim().known_nav(player),
                "tick {k}, {player:?}"
            );
        }
        let mut twin = game();
        twin.run_until(k);
        for _ in 0..120 {
            twin.step();
            loaded.step(&twin.replay().inputs[twin.sim().tick() as usize - 1]);
            assert_eq!(
                loaded.state_hash(),
                twin.sim().state_hash(),
                "from tick {k}, at {}",
                twin.sim().tick()
            );
        }
        versions.push((version(t.sim(), P1), version(t.sim(), P2)));
    }
    assert!(
        versions
            .windows(2)
            .all(|pair| pair[0].0 <= pair[1].0 && pair[0].1 <= pair[1].1)
    );
    // Player 1 saw the Generator gone (a patch it passed only moved from assumed to remembered); player 2 saw
    // the Generator gone too, and the patch in the west gone.
    assert_eq!(*versions.last().unwrap(), (1, 2), "{versions:?}");
    t.run_until(1300);
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());

    // The knowledge is part of the save: one that doesn't fit the game is refused, never a panic.
    let mut value: serde_json::Value = serde_json::to_value(t.sim()).unwrap();
    assert!(value["path_knowledge"][0]["version"].is_u64());
    value["path_knowledge"].as_array_mut().unwrap().pop();
    let error = serde_json::from_value::<Sim>(value).unwrap_err().to_string();
    assert!(error.contains("path knowledge"), "{error}");
}
