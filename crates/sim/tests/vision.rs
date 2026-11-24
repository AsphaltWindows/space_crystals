//! Vision (`core/vision.md`, phase 5f): the exact edge of a unit's sight, the ElevationModifier on sight, Explored as
//! history, players kept apart, objects that cover several tiles, and saves. `TestSim` checks the vision invariant
//! V1 after every tick (the vision kept up tick by tick is the one a fresh build gives).

use VisibilityState::{Explored, Unexplored, Visible};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::vision::FOG_DELAY_TICKS;
use sc_sim::{
    DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, Sim, SpawnAt, Target, TickInputs, TilePos,
    Vec2, VisibilityState,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// A flat map of `w` × `h` open tiles.
fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

fn state(t: &TestSim, player: PlayerId, x: i32, y: i32) -> VisibilityState {
    t.sim().visibility(player, TilePos::new(x, y))
}

fn count(sim: &Sim, player: PlayerId, state: VisibilityState) -> usize {
    sim.vision()
        .player(player)
        .unwrap()
        .states()
        .filter(|&s| s == state)
        .count()
}

fn is_visible(t: &TestSim, player: PlayerId, id: u32) -> bool {
    t.sim().is_visible(player, t.sim().object(ObjectId(id)).unwrap())
}

/// Whether tile `(x, y)` is Visible to `player` only for the fog delay.
fn lingers(t: &TestSim, player: PlayerId, x: i32, y: i32) -> bool {
    t.sim().vision().player(player).unwrap().lingers(TilePos::new(x, y))
}

/// The sight is a disc around the unit's centre; a tile is seen when its centre lies within the SightRange, the
/// boundary included. A Peacekeeper (5 grid units) on a tile's centre sees the 81 tiles whose centres lie within 5
/// tiles, those exactly 5 away along an axis or 3-4-5 diagonally included; half a space unit east, the tile exactly
/// 5 west drops out of sight (Visible for the fog delay, then Explored) and the one 5 east stays in.
#[test]
fn the_edge_of_sight_is_exact() {
    let mut t = ScenarioBuilder::new(&flat(24, 20)).start();
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, centre(10, 10))
        .run_until(1);
    assert_eq!(
        count(t.sim(), P1, Visible),
        81,
        "seen at the end of the tick it spawned"
    );
    for (x, y, want) in [
        (15, 10, Visible),
        (5, 10, Visible),
        (10, 5, Visible),
        (10, 15, Visible),
        (13, 14, Visible),
        (7, 6, Visible),
        (14, 13, Visible),
        (16, 10, Unexplored),
        (15, 11, Unexplored),
        (14, 14, Unexplored),
    ] {
        assert_eq!(state(&t, P1, x, y), want, "tile ({x}, {y})");
    }
    t.debug_at(
        2,
        DebugCommand::SetPosition {
            id: ObjectId(1),
            pos: centre(10, 10) + Vec2::new(0.5, 0.0),
        },
    )
    .run_until(2);
    assert_eq!(state(&t, P1, 15, 10), Visible, "319.5 su away");
    assert!(!lingers(&t, P1, 15, 10));
    assert!(lingers(&t, P1, 5, 10), "320.5 su away: out of sight");
    assert_eq!(state(&t, P1, 16, 10), Unexplored);
    assert!(is_visible(&t, P1, 1), "a unit's owner sees it");
    t.run_until(2 + FOG_DELAY_TICKS);
    assert_eq!(state(&t, P1, 5, 10), Explored, "320.5 su away");
    assert_eq!(state(&t, P1, 15, 10), Visible);
}

/// 24 × 5 tiles: level 0 in the west, a ramp on column 12 (its centre at height 0.5, level 0), level 1 from column
/// 13 on.
const STEP: &str = "
    ........................
    ........................
    ........................
    ........................
    ........................

    0000000000000111111111111
    0000000000000111111111111
    0000000000000111111111111
    0000000000000111111111111
    0000000000000111111111111
    0000000000000111111111111
";

/// `core/vision.md > ElevationModifier`: from higher ground a unit sees one grid unit further over lower tiles, from
/// lower ground one less up to higher tiles; equal levels keep the SightRange. The target side is the ground at the
/// tile's centre.
#[test]
fn high_ground_sees_one_further_and_low_ground_one_less() {
    let mut t = ScenarioBuilder::new(STEP).start();
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, centre(17, 2))
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, centre(8, 2))
        .run_until(1);
    assert_eq!(t.sim().map().elevation_level(centre(17, 2)), 1);
    assert_eq!(t.sim().map().elevation_level(centre(12, 2)), 0, "the ramp's centre");
    // Player 1 on level 1: 6 grid units down to level 0, 5 along level 1.
    assert_eq!(state(&t, P1, 11, 2), Visible, "6 away, lower");
    assert_eq!(state(&t, P1, 10, 2), Unexplored, "7 away");
    assert_eq!(state(&t, P1, 12, 2), Visible);
    assert_eq!(state(&t, P1, 22, 2), Visible, "5 away, same level");
    assert_eq!(state(&t, P1, 23, 2), Unexplored, "6 away, same level");
    // Player 2 on level 0: 4 grid units up to level 1, 5 along level 0.
    assert_eq!(state(&t, P2, 12, 2), Visible, "4 away, same level");
    assert_eq!(state(&t, P2, 13, 2), Unexplored, "5 away, higher");
    assert_eq!(state(&t, P2, 3, 2), Visible, "5 away, same level");
    assert_eq!(state(&t, P2, 2, 2), Unexplored);
    // Neither sees the other.
    assert!(!is_visible(&t, P1, 2) && !is_visible(&t, P2, 1));
}

/// `core/vision.md > Explored`: tiles a unit saw stay Explored after it leaves, and after it is gone.
#[test]
fn explored_tiles_stay_explored() {
    let mut t = ScenarioBuilder::new(&flat(40, 12)).start();
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, centre(5, 5))
        .order_at(
            2,
            P1,
            &[ObjectId(1)],
            OrderKind::Move(Target::Point(centre(30, 5))),
            false,
        )
        .run_until(2);
    while t.status(1) == MoveStatus::Moving {
        assert!(t.sim().tick() < 1000, "the scout never arrived");
        t.step();
    }
    assert_eq!(t.pos(1), centre(30, 5));
    for (x, y, want) in [
        (5, 5, Explored),
        (0, 5, Explored),
        (17, 1, Explored),
        (17, 9, Explored),
        (17, 11, Unexplored),
        (30, 5, Visible),
        (35, 5, Visible),
        (36, 5, Unexplored),
    ] {
        assert_eq!(state(&t, P1, x, y), want, "tile ({x}, {y})");
    }
    let explored = count(t.sim(), P1, Explored) + count(t.sim(), P1, Visible);
    let tick = t.sim().tick();
    t.debug_at(tick + 1, DebugCommand::Remove { id: ObjectId(1) }).step();
    assert!(lingers(&t, P1, 30, 5), "Visible for the fog delay after it is gone");
    t.run(FOG_DELAY_TICKS);
    assert_eq!(count(t.sim(), P1, Visible), 0);
    assert_eq!(count(t.sim(), P1, Explored), explored, "nothing is forgotten");
}

/// Each player sees through its own objects only, and sees the other's units only on tiles it sees.
#[test]
fn players_are_kept_apart() {
    let mut t = ScenarioBuilder::new(&flat(30, 12)).start();
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, centre(5, 5))
        .spawn_unit_at(1, ObjectKind::Guard, P2, centre(20, 5))
        .run_until(1);
    assert_eq!((state(&t, P1, 5, 5), state(&t, P1, 20, 5)), (Visible, Unexplored));
    assert_eq!((state(&t, P2, 5, 5), state(&t, P2, 20, 5)), (Unexplored, Visible));
    assert!(is_visible(&t, P1, 1) && !is_visible(&t, P1, 2));
    assert!(is_visible(&t, P2, 2) && !is_visible(&t, P2, 1));
    // A player who isn't in the game sees nothing.
    assert_eq!(t.sim().visibility(PlayerId(3), TilePos::new(5, 5)), Unexplored);
    assert!(!is_visible(&t, PlayerId(3), 1));
    // The Guard walks up to the Peacekeeper: each now sees the other.
    t.debug_at(
        2,
        DebugCommand::SetPosition {
            id: ObjectId(2),
            pos: centre(9, 5),
        },
    )
    .run_until(2 + FOG_DELAY_TICKS);
    assert!(is_visible(&t, P1, 2) && is_visible(&t, P2, 1));
    assert_eq!(state(&t, P2, 20, 5), Explored);
    assert_eq!(state(&t, P1, 20, 5), Unexplored);
}

/// `Sim::is_visible`: an object is seen when any tile it covers is Visible. A Supply Delivery Station covers 2 × 2
/// tiles: seen with one of them in sight, Explored once none is (after the fog delay). A unit covers the tiles its
/// silhouette overlaps.
#[test]
fn an_object_is_seen_when_any_tile_it_covers_is() {
    let station = SpawnAt::Tile(TilePos::new(15, 5));
    let mut t = ScenarioBuilder::new(&flat(30, 12))
        .spawn(ObjectKind::SupplyDeliveryStation, None, station)
        .start();
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, centre(10, 5))
        .run_until(1);
    // Only the station's north-west tile (15, 5) is within 5 grid units.
    assert_eq!(state(&t, P1, 15, 5), Visible);
    assert_eq!((state(&t, P1, 16, 5), state(&t, P1, 15, 6)), (Unexplored, Unexplored));
    assert!(is_visible(&t, P1, 1));
    let of = |t: &TestSim, player| {
        let station = t.sim().object(ObjectId(1)).unwrap();
        t.sim().vision().object_state(player, station)
    };
    t.debug_at(
        2,
        DebugCommand::SetPosition {
            id: ObjectId(2),
            pos: centre(9, 5),
        },
    )
    .run_until(2);
    assert!(is_visible(&t, P1, 1), "for the fog delay");
    t.run_until(2 + FOG_DELAY_TICKS);
    assert!(!is_visible(&t, P1, 1));
    assert_eq!((of(&t, P1), of(&t, P2)), (Explored, Unexplored));

    // A Guard (36 su wide) straddling the edge between tiles 14 and 15 is seen through tile 14 alone.
    let mut t = ScenarioBuilder::new(&flat(30, 12)).start();
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, centre(9, 5))
        .spawn_unit_at(1, ObjectKind::Guard, P2, Vec2::new(15.0 * 64.0, centre(9, 5).y))
        .run_until(1);
    assert_eq!((state(&t, P1, 14, 5), state(&t, P1, 15, 5)), (Visible, Unexplored));
    assert!(is_visible(&t, P1, 2));
    t.debug_at(
        2,
        DebugCommand::SetPosition {
            id: ObjectId(2),
            pos: Vec2::new(15.0 * 64.0 + 20.0, centre(9, 5).y),
        },
    )
    .run_until(2);
    assert!(!is_visible(&t, P1, 2), "now on tile 15 only");
}

/// Two scouts of each player criss-crossing a map, ordered in the first ticks only.
fn scouting() -> TestSim {
    let mut t = ScenarioBuilder::new(&flat(48, 32)).start();
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, centre(4, 4))
        .spawn_unit_at(1, ObjectKind::Guard, P1, centre(4, 27))
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, centre(43, 27))
        .spawn_unit_at(1, ObjectKind::Guard, P2, centre(43, 4));
    let to = |x, y| OrderKind::Move(Target::Point(centre(x, y)));
    t.order_at(2, P1, &[ObjectId(1)], to(40, 20), false)
        .order_at(2, P1, &[ObjectId(2)], to(30, 4), false)
        .order_at(2, P2, &[ObjectId(3)], to(6, 10), false)
        .order_at(2, P2, &[ObjectId(4)], to(10, 28), false);
    t
}

/// The Explored tiles are state: saved, loaded and hashed. A game saved mid-way loads equal, sees the same tiles
/// (the Visible ones rebuilt from the objects) and plays on the same, tick for tick; a replay ends the same.
#[test]
fn saves_and_replays_keep_the_vision() {
    let mut t = scouting();
    t.run_until(60);
    let json = serde_json::to_string(t.sim()).unwrap();
    assert!(
        json.contains("\"vision\":[{\"player\":1,\"explored\":["),
        "{}",
        &json[..200]
    );
    let mut loaded: Sim = serde_json::from_str(&json).unwrap();
    assert_eq!(&loaded, t.sim());
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    let tiles = |sim: &Sim| {
        let players = sim.vision().players();
        players.iter().flat_map(|p| p.states()).collect::<Vec<_>>()
    };
    assert_eq!(tiles(&loaded), tiles(t.sim()));
    for _ in 0..400 {
        t.step();
        loaded.step(&TickInputs::default());
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert_eq!(tiles(&loaded), tiles(t.sim()));
    assert!(count(t.sim(), P1, Explored) > 0 && count(t.sim(), P2, Explored) > 0);
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
}

/// Two games that end with the very same objects but explored different tiles hash differently.
#[test]
fn the_explored_tiles_are_hashed() {
    let scout = |detour: bool| {
        let mut t = ScenarioBuilder::new(&flat(40, 12)).start();
        t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, centre(5, 5));
        if detour {
            let away = DebugCommand::SetPosition {
                id: ObjectId(1),
                pos: centre(30, 5),
            };
            let back = DebugCommand::SetPosition {
                id: ObjectId(1),
                pos: centre(5, 5),
            };
            t.debug_at(2, away).debug_at(3, back);
        }
        t.run_until(3 + FOG_DELAY_TICKS);
        t
    };
    let (stayed, roamed) = (scout(false), scout(true));
    let objects = |t: &TestSim| t.sim().objects().cloned().collect::<Vec<_>>();
    assert_eq!(objects(&stayed), objects(&roamed));
    assert_eq!(state(&roamed, P1, 30, 5), Explored);
    assert_ne!(stayed.sim().state_hash(), roamed.sim().state_hash());
}
