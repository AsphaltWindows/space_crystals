//! Design change 0416 (`core/vision.md > Starting exploration`, slice M5-S7): with the game option
//! `SimConfig::start_explored` on, the map starts explored, the fog of war still on. Every tile starts Explored for
//! every player, and each remembers the terrain and the neutral objects as they are at the start (the patches with
//! their starting amounts, the Supply Delivery Stations: their records, design change 0159), never a player's objects,
//! so bases still have to be scouted; from there memory works as ever. With it off the map starts Unexplored, as
//! before, and the replays recorded before the option existed replay as recorded. The sim's default is off, so that the
//! goldens and the tests don't move; the client turns it on for the games it launches (`crates/game/src/cli.rs`).
//!
//! Every `TestSim` checks the invariants after each tick, V1 among them.

use std::path::Path;

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::tuning::vision::FOG_DELAY_TICKS;
use sc_sim::vision::LastKnown;
use sc_sim::{
    DebugCommand, ObjectId, ObjectKind, PlayerId, Replay, ResourceState, Scenario, Sim, SimConfig, SpawnAt,
    SpawnSettings, TargetView, TilePos, Vec2, VisibilityState,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// The objects of [`world`], by id.
const PATCH: u32 = 1;
const STATION: u32 = 7;
const GENERATOR: u32 = 8;
const SCOUT: u32 = 9;

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

/// The game options of these tests: debug commands on, as `TestSim::new` has them, and the map explored at the start
/// or not.
fn config(start_explored: bool) -> SimConfig {
    SimConfig {
        start_explored,
        ..SimConfig::DEBUG
    }
}

/// A flat 40 × 10 map (400 tiles: the last word of the Explored bits is a partial one). Player 1's base in the west,
/// as `economy_mining.rs` has it: patches 1 and 2 (1500 crystals each, on (1, 7) and (3, 7)), the Deployment Center 3,
/// the Extraction Facility 4, and plates 5 and 6 on the patches, which mine them. A Supply Delivery Station 7 on
/// (20, 1), in nobody's sight; player 2's Generator 8 on (36, 1) and their Peacekeeper 9 on (36, 7), in the east, far
/// out of player 1's sight, and player 1's far out of theirs.
fn world() -> Scenario {
    let station = SpawnSettings::SupplyDeliveryStation {
        starting_supplies: 0,
        delivery_size: 5,
        delivery_interval: 20,
    };
    ScenarioBuilder::new(&vec![".".repeat(40); 10].join("\n"))
        .spawn(ObjectKind::CrystalPatch, None, tile(1, 7))
        .spawn(ObjectKind::CrystalPatch, None, tile(3, 7))
        .spawn(ObjectKind::DeploymentCenter, Some(P1), tile(1, 1))
        .spawn(ObjectKind::ExtractionFacility, Some(P1), tile(6, 1))
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(1, 7))
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(3, 7))
        .spawn_with(station, tile(20, 1))
        .spawn(ObjectKind::Generator, Some(P2), tile(36, 1))
        .spawn(ObjectKind::Peacekeeper, Some(P2), SpawnAt::Point(centre(36, 7)))
        .build()
}

fn game(start_explored: bool) -> TestSim {
    let mut t = TestSim::with_config(world(), config(start_explored));
    t.without_auto_targeting();
    t
}

/// Every tile of the map.
fn tiles(sim: &Sim) -> impl Iterator<Item = TilePos> + use<> {
    let (w, h) = (sim.map().width() as i32, sim.map().height() as i32);
    (0..h).flat_map(move |y| (0..w).map(move |x| TilePos::new(x, y)))
}

fn records(sim: &Sim, player: PlayerId) -> Vec<LastKnown> {
    sim.vision().player(player).unwrap().records().cloned().collect()
}

/// Each neutral object of `sim` as a record taken at the end of tick 0: the map's, as they stand at the start.
fn neutral_at_start(sim: &Sim) -> Vec<LastKnown> {
    (sim.objects())
        .filter(|o| o.owner.is_none())
        .filter_map(|o| LastKnown::of(o, 0))
        .collect()
}

/// What patch `id` has left.
fn remaining(sim: &Sim, id: u32) -> u32 {
    match sim.object(ObjectId(id)).map(|o| &o.resource) {
        Some(Some(ResourceState::CrystalPatch { remaining })) => *remaining,
        other => panic!("#{id} is no patch: {other:?}"),
    }
}

/// 0416's first Verification, the sim's half: "Start a game with no flag: the whole map shows terrain and crystal
/// patches, dimmed; enemy bases are not shown until scouted; units only where you see." At 0:00 every tile is Explored
/// or, around each player's own objects, Visible, for both players; each remembers every neutral object as it stands
/// at the start (the patches at 1500, the station empty) and none of a player's: player 1 remembers patches 1 and 2,
/// though its plates stand on them, and the station, and knows nothing of player 2's Generator and Peacekeeper, which
/// it neither sees nor can target, while it can target the station it remembers.
#[test]
fn at_the_start_every_tile_is_explored_and_only_the_neutral_objects_are_remembered() {
    let t = game(true);
    let sim = t.sim();
    assert_eq!(sim.tick(), 0);
    for player in [P1, P2] {
        let vision = sim.vision().player(player).unwrap();
        let states: Vec<VisibilityState> = tiles(sim).map(|tile| vision.state(tile)).collect();
        assert!(
            states.iter().all(|&s| s != VisibilityState::Unexplored),
            "player {}: every tile explored",
            player.0
        );
        let visible = states.iter().filter(|&&s| s == VisibilityState::Visible).count();
        assert!(
            visible > 0 && visible < states.len() / 2,
            "player {}: {visible} Visible",
            player.0
        );
        assert_eq!(records(sim, player), neutral_at_start(sim), "player {}", player.0);
    }
    let ids: Vec<u32> = records(sim, P2).iter().map(|r| r.id.0).collect();
    assert_eq!(ids, [PATCH, 2, STATION]);
    assert!(
        (records(sim, P2).iter()).all(|r| r.owner.is_none() && r.seen == 0),
        "no player's objects"
    );
    assert_eq!(
        records(sim, P2)[0].resource,
        Some(ResourceState::CrystalPatch { remaining: 1500 })
    );
    let tile_of = |id: u32| TilePos::containing(sim.object(ObjectId(id)).unwrap().pos);
    for id in [STATION, GENERATOR, SCOUT] {
        assert_eq!(
            sim.visibility(P1, tile_of(id)),
            VisibilityState::Explored,
            "#{id}'s tile"
        );
    }
    for id in [GENERATOR, SCOUT] {
        let object = sim.object(ObjectId(id)).unwrap();
        assert!(!sim.vision().sees(P1, object), "player 1 doesn't see #{id}");
        assert_eq!(sim.target_view(P1, ObjectId(id)), TargetView::Gone, "#{id} unknown");
    }
    assert!(matches!(
        sim.target_view(P1, ObjectId(STATION)),
        TargetView::Remembered { .. }
    ));
}

/// 0416's second Verification, the sim's half: "Start with the flag set to unexplored: the map is black outside your
/// sight." With the option off the map starts Unexplored but where each player sees, with the same records of the
/// neutral objects (design change 0159). The option is off by default, left out of files and of the hash while off,
/// and hashed while on.
#[test]
fn with_the_option_off_the_map_starts_unexplored_as_before() {
    let t = game(false);
    let sim = t.sim();
    for player in [P1, P2] {
        let vision = sim.vision().player(player).unwrap();
        let unexplored = tiles(sim).filter(|&tile| vision.state(tile) == VisibilityState::Unexplored);
        assert!(
            unexplored.count() > 200,
            "player {}: black outside their sight",
            player.0
        );
        assert_eq!(records(sim, player), neutral_at_start(sim), "player {}", player.0);
    }
    assert_eq!(sim.visibility(P1, TilePos::new(20, 1)), VisibilityState::Unexplored);

    assert!(!SimConfig::default().start_explored && !SimConfig::DEBUG.start_explored);
    let json = |config: SimConfig| serde_json::to_string(&config).unwrap();
    assert_eq!(json(config(false)), r#"{"debug":true}"#, "as files had it");
    assert_eq!(json(config(true)), r#"{"debug":true,"start_explored":true}"#);
    let parsed: SimConfig = serde_json::from_str(r#"{"debug":true}"#).unwrap();
    assert_eq!(parsed, config(false), "a file without it: off");
    assert_ne!(sim.state_hash(), game(true).sim().state_hash(), "hashed while on");
}

/// 0416's third Verification: "Mine a remote patch an enemy can't see: the enemy still remembers its starting amount
/// until it looks again." Player 1's plates mine patch 1 out of player 2's sight; player 2's record keeps 1500, as of
/// tick 0, until their Peacekeeper comes to look (moved by a debug command), when it takes what is left.
#[test]
fn a_patch_mined_out_of_an_enemy_s_sight_keeps_its_starting_amount_in_their_memory() {
    let mut t = game(true);
    t.run_until(120);
    let left = remaining(t.sim(), PATCH);
    assert!(left < 1500, "mined: {left} left");
    let record = |t: &TestSim| {
        let record = t
            .sim()
            .vision()
            .player(P2)
            .unwrap()
            .record(ObjectId(PATCH))
            .cloned()
            .unwrap();
        (record.seen, record.resource)
    };
    let start = Some(ResourceState::CrystalPatch { remaining: 1500 });
    assert_eq!(record(&t), (0, start.clone()), "as at the start");
    let mine = t
        .sim()
        .vision()
        .player(P1)
        .unwrap()
        .record(ObjectId(PATCH))
        .cloned()
        .unwrap();
    assert_eq!(
        (mine.seen, mine.resource),
        (120, Some(ResourceState::CrystalPatch { remaining: left })),
        "player 1 sees it mined"
    );

    t.debug_at(
        121,
        DebugCommand::SetPosition {
            id: ObjectId(SCOUT),
            pos: centre(4, 5),
        },
    );
    t.run_until(121);
    let now = Some(ResourceState::CrystalPatch {
        remaining: remaining(t.sim(), PATCH),
    });
    assert_eq!(record(&t), (121, now), "looked again");
    // Gone again, the record keeps what it last saw: memory as ever.
    t.debug_at(
        122,
        DebugCommand::SetPosition {
            id: ObjectId(SCOUT),
            pos: centre(36, 7),
        },
    );
    t.run_until(122 + FOG_DELAY_TICKS + 100);
    let (seen, resource) = record(&t);
    assert_eq!(seen, 122 + FOG_DELAY_TICKS - 1, "seen last in the fog delay");
    assert_ne!(resource, start);
    assert!(remaining(t.sim(), PATCH) < left, "mined since, unseen");
}

/// The option is part of the game: a save of a game that started explored loads with every tile still Explored and
/// the same state hash, and its replay, which records the option in its scenario, plays back to the same game.
#[test]
fn a_game_started_explored_saves_and_replays_explored() {
    let mut t = game(true);
    t.run_until(40);
    let sim = t.sim();
    let saved = serde_json::to_string(sim).unwrap();
    let loaded: Sim = serde_json::from_str(&saved).unwrap();
    assert_eq!(loaded.state_hash(), sim.state_hash());
    assert!(tiles(&loaded).all(|tile| loaded.visibility(P2, tile) != VisibilityState::Unexplored));

    let text = serde_json::to_string(t.replay()).unwrap();
    assert!(text.contains(r#""start_explored":true"#), "recorded");
    let back: Replay = serde_json::from_str(&text).unwrap();
    assert_eq!(back.run().state_hash(), sim.state_hash());
}

/// The replays recorded before the option existed have no such field: they load with the map unexplored at the start
/// and replay to their recorded hashes. One of the goldens stands in for them (`tests/golden/fog_chase.json`, a game
/// whose fog is the point).
#[test]
fn an_old_replay_without_the_option_replays_to_its_recorded_hash() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/fog_chase.json");
    let text = std::fs::read_to_string(path).unwrap();
    assert!(!text.contains("start_explored"));
    let replay: Replay = serde_json::from_str(&text).unwrap();
    assert!(!replay.scenario.setup.config.start_explored);
    assert!(!replay.checkpoints.is_empty());
    assert!(replay.run_checked().is_ok(), "replays as recorded");
}
