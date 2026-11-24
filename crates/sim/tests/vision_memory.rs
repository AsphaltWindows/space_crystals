//! Last-known records (`core/vision.md > Explored`: structures and resources show in their last-known state, which
//! may be out of date): made and refreshed while the player sees the object, kept while it doesn't, dropped once it
//! sees the place empty; made from the start of the map's neutral objects (design change 0159: "known from the start
//! (pathing and memory), like terrain"), and never of an object placed out of sight later; each player's own;
//! hashed, saved and replayed.
//! No owned structure exists yet, so the patches and stations of the map stand in for structures. `TestSim` checks
//! V1 after every tick, which compares the records with those a fresh build keeps.

use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::vision::LastKnown;
use sc_sim::{
    DebugCommand, ObjectId, ObjectKind, PlayerId, ResourceState, Sim, SpawnAt, SpawnSettings, TilePos, Vec2,
    VisibilityState,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// The objects of [`world`], by id.
const PATCH: u32 = 1;
const STATION: u32 = 2;
const FAR_PATCH: u32 = 3;
const SCOUT: u32 = 4;

fn centre(x: i32, y: i32) -> Vec2 {
    TilePos::new(x, y).center()
}

/// A flat 40 × 12 map with a patch of 777 crystals on tile (14, 5), a station on (13, 8)–(14, 9) that delivers 5
/// supplies every 20 ticks, starting empty, and a patch on (27, 8). Player 1's scout (a Peacekeeper, SightRange 5)
/// on (10, 5) sees the first two; player 2's on (25, 5) sees the third.
fn world() -> TestSim {
    let station = SpawnSettings::SupplyDeliveryStation {
        starting_supplies: 0,
        delivery_size: 5,
        delivery_interval: 20,
    };
    let mut t = ScenarioBuilder::new(&vec![".".repeat(40); 12].join("\n"))
        .spawn_with(
            SpawnSettings::CrystalPatch { starting_amount: 777 },
            SpawnAt::Tile(TilePos::new(14, 5)),
        )
        .spawn_with(station, SpawnAt::Tile(TilePos::new(13, 8)))
        .spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(27, 8)))
        .start();
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, centre(10, 5))
        .spawn_unit_at(1, ObjectKind::Peacekeeper, P2, centre(25, 5));
    t
}

/// Moves player 1's scout to tile `(x, y)` at `tick`: (34, 5) sees none of the objects, (10, 5) the first two.
fn scout_to(t: &mut TestSim, tick: u32, x: i32, y: i32) {
    t.debug_at(
        tick,
        DebugCommand::SetPosition {
            id: ObjectId(SCOUT),
            pos: centre(x, y),
        },
    );
}

fn record(t: &TestSim, player: PlayerId, id: u32) -> Option<LastKnown> {
    t.sim().vision().player(player).unwrap().record(ObjectId(id)).cloned()
}

fn ids(sim: &Sim, player: PlayerId) -> Vec<u32> {
    sim.vision().player(player).unwrap().records().map(|r| r.id.0).collect()
}

fn resource(t: &TestSim, id: u32) -> ResourceState {
    t.sim().object(ObjectId(id)).unwrap().resource.clone().unwrap()
}

/// A record holds the object as the player last saw it: refreshed every tick in sight, the station's countdown
/// included, and kept out of sight while the station takes a delivery the player doesn't see.
#[test]
fn records_keep_the_last_state_seen() {
    let mut t = world();
    t.run_until(5);
    let station = record(&t, P1, STATION).unwrap();
    assert_eq!(
        (station.seen, station.resource.clone()),
        (5, Some(resource(&t, STATION)))
    );
    assert_eq!(station.pos(), t.sim().object(ObjectId(STATION)).unwrap().pos);
    let patch = record(&t, P1, PATCH).unwrap();
    assert_eq!(patch.resource, Some(ResourceState::CrystalPatch { remaining: 777 }));
    let at_5 = station.resource;

    scout_to(&mut t, 6, 34, 5);
    t.run_until(30);
    assert!(matches!(
        resource(&t, STATION),
        ResourceState::SupplyStation {
            current_supplies: 5,
            ..
        }
    ));
    let station = record(&t, P1, STATION).unwrap();
    assert_eq!((station.seen, station.resource), (5, at_5), "not seen since tick 5");
    let remembered = |t: &TestSim| -> Vec<u32> {
        let player = t.sim().vision().player(P1).unwrap();
        player.remembered().map(|r| r.id.0).collect()
    };
    assert_eq!(
        remembered(&t),
        [PATCH, STATION, FAR_PATCH],
        "the far one known from the start"
    );

    scout_to(&mut t, 31, 10, 5);
    t.run_until(31);
    let station = record(&t, P1, STATION).unwrap();
    assert_eq!((station.seen, station.resource), (31, Some(resource(&t, STATION))));
    assert_eq!(remembered(&t), [FAR_PATCH]);
    assert_eq!(record(&t, P1, FAR_PATCH).map(|r| r.seen), Some(0), "never seen");
}

/// `core/vision.md > Explored`: an object removed out of sight stays remembered (saved and replayed with the game)
/// until the player sees its place again.
#[test]
fn a_removed_object_is_remembered_until_its_place_is_seen() {
    let mut t = world();
    scout_to(&mut t, 6, 34, 5);
    t.debug_at(10, DebugCommand::Remove { id: ObjectId(PATCH) })
        .run_until(12);
    assert!(t.sim().object(ObjectId(PATCH)).is_none());
    let patch = record(&t, P1, PATCH).unwrap();
    assert_eq!((patch.seen, patch.pos()), (5, centre(14, 5)));
    let player = t.sim().vision().player(P1).unwrap();
    assert!(
        player.is_record_tile(TilePos::new(14, 5)),
        "still in the way, as far as player 1 knows"
    );

    let json = serde_json::to_string(t.sim()).unwrap();
    assert!(json.contains("\"last_known\":[{\"id\":1,"), "{json}");
    let mut loaded: Sim = serde_json::from_str(&json).unwrap();
    assert_eq!(&loaded, t.sim());
    assert_eq!(loaded.state_hash(), t.sim().state_hash());

    scout_to(&mut t, 15, 10, 5);
    for _ in 0..5 {
        t.step();
        assert!(t.replay().step(&mut loaded));
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert_eq!(record(&t, P1, PATCH), None, "seen gone at tick 15");
    assert_eq!(ids(t.sim(), P1), [STATION, FAR_PATCH]);
    assert_eq!(ids(&loaded, P1), [STATION, FAR_PATCH]);
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
}

/// An object placed on Explored tiles, out of sight, stays unknown until the player sees it.
#[test]
fn an_object_placed_in_the_fog_stays_unknown() {
    let mut t = world();
    scout_to(&mut t, 6, 34, 5);
    let at = SpawnAt::Tile(TilePos::new(12, 3));
    t.debug_at(
        8,
        DebugCommand::Spawn {
            kind: ObjectKind::CrystalPatch,
            owner: None,
            at,
            orientation: Default::default(),
        },
    )
    .run_until(10);
    let placed = 6;
    assert!(t.sim().object(ObjectId(placed)).is_some());
    assert_eq!(t.sim().visibility(P1, TilePos::new(12, 3)), VisibilityState::Explored);
    assert_eq!(record(&t, P1, placed), None);
    scout_to(&mut t, 15, 10, 5);
    t.run_until(15);
    assert_eq!(record(&t, P1, placed).map(|r| r.seen), Some(15));
}

/// Each player remembers what it saw itself: the map's objects, which both know from the start (design change 0159),
/// each as it last saw them, and an object placed later only the player who saw it (player 2, patch 6 on (27, 3)).
#[test]
fn players_remember_apart() {
    let mut t = world();
    t.debug_at(
        1,
        DebugCommand::Spawn {
            kind: ObjectKind::CrystalPatch,
            owner: None,
            at: SpawnAt::Tile(TilePos::new(27, 3)),
            orientation: Default::default(),
        },
    )
    .run_until(3);
    assert_eq!(ids(t.sim(), P1), [PATCH, STATION, FAR_PATCH]);
    assert_eq!(ids(t.sim(), P2), [PATCH, STATION, FAR_PATCH, 6]);
    let seen = |player, id| record(&t, player, id).unwrap().seen;
    assert_eq!((seen(P1, PATCH), seen(P1, FAR_PATCH)), (3, 0));
    assert_eq!((seen(P2, PATCH), seen(P2, FAR_PATCH)), (0, 3));
}

/// Two games that end with the same objects and the same Explored tiles, but in one of them player 1 saw the
/// patch before it went, hash differently.
#[test]
fn the_records_are_hashed() {
    let game = |removed_at: u32| {
        let mut t = world();
        scout_to(&mut t, 6, 34, 5);
        t.debug_at(removed_at, DebugCommand::Remove { id: ObjectId(PATCH) })
            .run_until(12);
        t
    };
    let (seen, unseen) = (game(10), game(1));
    let objects = |t: &TestSim| t.sim().objects().cloned().collect::<Vec<_>>();
    assert_eq!(objects(&seen), objects(&unseen));
    let tiles = |t: &TestSim| t.sim().vision().player(P1).unwrap().states().collect::<Vec<_>>();
    assert_eq!(tiles(&seen), tiles(&unseen));
    assert_eq!(
        (ids(seen.sim(), P1), ids(unseen.sim(), P1)),
        (vec![PATCH, STATION, FAR_PATCH], vec![STATION, FAR_PATCH])
    );
    assert_ne!(seen.sim().state_hash(), unseen.sim().state_hash());
}

/// A save's records must be of objects the player doesn't own, each once, on the map, seen by the save's tick.
#[test]
fn loading_rejects_broken_records() {
    let mut t = world();
    t.run_until(5);
    let json = serde_json::to_value(t.sim()).unwrap();
    let broken = |change: &dyn Fn(&mut serde_json::Value)| {
        let mut json = json.clone();
        change(&mut json["vision"][0]["last_known"]);
        serde_json::from_value::<Sim>(json).unwrap_err().to_string()
    };
    let duplicate = broken(&|records| {
        let first = records[0].clone();
        records.as_array_mut().unwrap().push(first);
    });
    assert_eq!(duplicate, "player 1's record of #1 appears more than once");
    let own = broken(&|records| records[1]["owner"] = 1.into());
    assert_eq!(own, "player 1's record of #2 is of its own object");
    let off = broken(&|records| records[1]["placement"]["origin"]["x"] = 39.into());
    assert_eq!(off, "player 1's record of #2 lies off the map");
    let future = broken(&|records| records[0]["seen"] = 6.into());
    assert_eq!(future, "player 1's record of #1 was seen at tick 6, after tick 5");
    assert_eq!(serde_json::from_value::<Sim>(json).unwrap(), *t.sim());
}
