//! Built-in maps, and how a game starts from a map and a game setup: slots and their owners, the default
//! restrictions, units on open ground. The problems `Scenario::validate` finds are in `scenario_problems.rs`.

use sc_sim::scenario::MAP_FORMAT_VERSION;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    Faction, GameSetup, LayoutObject, MapType, ObjectId, ObjectKind, PlayerId, PlayerSetup, Scenario, ScenarioProblem,
    Sim, SimConfig, SlotId, Spawn, SpawnAt, SpawnProblem, Symmetry, TilePos, Vec2, scenarios,
};

#[test]
fn built_in_scenarios_load() {
    for &name in scenarios::NAMES {
        let scenario = scenarios::by_name(name).unwrap();
        assert_eq!(scenario.name(), name);
        assert_eq!(scenario.validate(), [], "{name}");
        let sim = Sim::try_new(&scenario).unwrap();
        assert_eq!(sim.tick(), 0);
        assert!(sim.events().is_empty());
        assert_eq!(sim.players().len(), 2);
        assert_eq!(scenario.setup, GameSetup::default_for(&scenario.map));
    }
    assert!(scenarios::by_name("no_such_map").is_none());
    assert_eq!(MAP_FORMAT_VERSION, 3);
}

#[test]
fn dev_skirmish_layout() {
    let scenario = scenarios::dev_skirmish();
    let map = &scenario.map;
    assert_eq!((map.terrain.width(), map.terrain.height()), (64, 48));
    assert_eq!(
        (map.map_type, map.symmetry),
        (MapType::SkirmishDefault, Symmetry::Rotate180)
    );
    assert_eq!(map.first_asymmetry(), None, "symmetric by construction");
    let starts: Vec<_> = map.slots.iter().map(|s| s.start).collect();
    // In opposite corners, images of each other.
    assert_eq!(starts, [TilePos::new(8, 8), TilePos::new(55, 39)]);
    assert_eq!(map.slots[1].camera_start, Some(TilePos::new(55, 39).center()));
    let sim = Sim::new(&scenario);
    // Spawns get ascending ids in list order: the west half's patches and the central station, then the patches'
    // images (the station is its own image).
    let kinds: Vec<_> = sim.objects().map(|o| (o.id, o.kind)).collect();
    let expected: Vec<_> = (1..=17)
        .map(|i| {
            let kind = if i == 9 {
                ObjectKind::SupplyDeliveryStation
            } else {
                ObjectKind::CrystalPatch
            };
            (ObjectId(i), kind)
        })
        .collect();
    assert_eq!(kinds, expected);
}

#[test]
fn empty_maps_of_any_size_are_valid() {
    for (w, h) in [(1, 1), (2, 3), (32, 24), (5, 200)] {
        let scenario = scenarios::empty(w, h);
        assert_eq!(scenario.validate(), [], "{w}×{h}");
    }
    let starts: Vec<_> = scenarios::empty(32, 24).map.slots.iter().map(|s| s.start).collect();
    assert_eq!(starts, [TilePos::new(8, 6), TilePos::new(23, 17)]);
}

/// 6 × 4 tiles, 384 × 256 su, two slots and the default setup (players 1 and 2).
fn scenario() -> Scenario {
    ScenarioBuilder::new("......\n......\n......\n......").build()
}

fn tile_spawn(kind: ObjectKind, owner: Option<SlotId>, x: i32, y: i32) -> Spawn {
    Spawn::new(kind, owner, SpawnAt::Tile(TilePos::new(x, y)))
}

/// A player's slot decides where they start; a slot's objects go to whoever takes it, and stay out of the game
/// when nobody does. Factions, names, seeds and game options are the setup's.
#[test]
fn slot_ownership_is_resolved_at_start() {
    let mut s = scenario();
    s.setup = GameSetup {
        players: vec![PlayerSetup {
            name: "Ann".into(),
            faction: Faction::Syndicate,
            slot: SlotId(2),
        }],
        seed: 42,
        config: SimConfig::DEBUG,
    };
    let sim = Sim::try_new(&s).unwrap();
    let players: Vec<_> = sim
        .players()
        .iter()
        .map(|p| (p.id, p.name.as_str(), p.faction))
        .collect();
    assert_eq!(players, [(PlayerId(1), "Ann", Faction::Syndicate)]);
    assert_eq!(sim.config(), SimConfig::DEBUG);
    assert_eq!(s.setup.player_at(SlotId(2)), Some(PlayerId(1)));
    assert_eq!(s.setup.player_at(SlotId(1)), None);
    let mut reseeded = s.clone();
    reseeded.setup.seed = 43;
    assert_ne!(Sim::new(&reseeded).state_hash(), sim.state_hash());
    let mut without_debug = s.clone();
    without_debug.setup.config = SimConfig::default();
    assert_ne!(Sim::new(&without_debug).state_hash(), sim.state_hash());

    // Owned objects: no object type can be owned by a map yet (resources never are), so the editor's lenient
    // preview shows how ownership resolves.
    s.map.spawns = vec![
        tile_spawn(ObjectKind::CrystalPatch, Some(SlotId(2)), 0, 0),
        tile_spawn(ObjectKind::CrystalPatch, Some(SlotId(1)), 1, 0),
        tile_spawn(ObjectKind::CrystalPatch, None, 2, 0),
    ];
    let (preview, ids) = Sim::preview(&s);
    assert_eq!(ids, [Some(ObjectId(1)), None, Some(ObjectId(2))]);
    assert_eq!(preview.object(ObjectId(1)).unwrap().owner, Some(PlayerId(1)));
    assert_eq!(preview.object(ObjectId(2)).unwrap().owner, None);
}

/// `core/map_editor.md > Map Types > Skirmish Default`: symmetry is required, and objects are neutral outside the
/// Starting Layouts. Custom maps have neither restriction.
#[test]
fn skirmish_default_restrictions() {
    let mut s = scenarios::dev_skirmish();
    s.map.symmetry = Symmetry::None;
    assert_eq!(s.map.validate(), [ScenarioProblem::SkirmishWithoutSymmetry]);
    s.map.map_type = MapType::Custom;
    assert_eq!(s.map.validate(), []);

    let mut s = scenarios::dev_skirmish();
    // The station on the center is its own image, so giving it an owner keeps the map symmetric.
    let station = s
        .map
        .spawns
        .iter()
        .position(|sp| sp.kind == ObjectKind::SupplyDeliveryStation)
        .unwrap();
    s.map.spawns[station].owner = Some(SlotId(1));
    assert_eq!(
        s.map.validate(),
        [ScenarioProblem::InvalidSpawn {
            spawn: station,
            kind: ObjectKind::SupplyDeliveryStation,
            problem: SpawnProblem::OwnedResource
        }]
    );
    s.map.spawns[station].owner = Some(SlotId(5));
    assert!(matches!(
        s.map.validate()[0],
        ScenarioProblem::InvalidSpawn {
            problem: SpawnProblem::OwnedResource,
            ..
        }
    ));
}

/// `docs/movement.md > 3`: a game starts with its units on open ground inside the map and apart, and no blocking
/// footprint on a ground unit. Touching is allowed. Of two objects in conflict, the later one is reported: pre-placed
/// objects by the map, the Starting Layouts the players get by the game.
#[test]
fn units_start_on_open_ground_and_apart() {
    // 8 × 6 tiles (512 × 384 su), water on tiles (5..=6, 1..=2): x 320..448, y 64..192.
    let mut s = ScenarioBuilder::new(
        "
        ........
        .....~~.
        .....~~.
        ........
        ........
        ........
        ",
    )
    .build();
    let (pk, guard, station) = (
        ObjectKind::Peacekeeper,
        ObjectKind::Guard,
        ObjectKind::SupplyDeliveryStation,
    );
    let unit = |kind, owner, x, y| Spawn::new(kind, owner, SpawnAt::Point(Vec2::new(x, y)));
    // Peacekeepers are 24 × 24 su, Guards 36 × 36.
    s.map.spawns = vec![
        unit(pk, Some(SlotId(1)), 100.0, 100.0),
        unit(guard, None, 100.0, 70.0), // touches 0
        unit(pk, Some(SlotId(2)), 110.0, 120.0),
        unit(pk, None, 384.0, 128.0),    // in the water
        unit(pk, None, 308.0, 128.0),    // touches the water
        unit(guard, None, 10.0, 300.0),  // across the map's edge
        tile_spawn(station, None, 1, 1), // x, y 64..192: on 0, 1 and 2
        unit(pk, None, 160.0, 160.0),    // on the station
        unit(pk, None, 204.0, 160.0),    // touches the station
    ];
    let invalid = |spawn, kind, problem| ScenarioProblem::InvalidSpawn { spawn, kind, problem };
    let expected = [
        invalid(2, pk, SpawnProblem::OverlapsUnit),
        invalid(3, pk, SpawnProblem::Blocked),
        invalid(5, guard, SpawnProblem::Blocked),
        invalid(6, station, SpawnProblem::CoversUnit),
        invalid(7, pk, SpawnProblem::Blocked),
    ];
    assert_eq!(s.map.validate(), expected);
    assert_eq!(s.validate(), expected);
    assert_eq!(
        expected[0].to_string(),
        "spawn 2 (Peacekeeper): its box overlaps another unit's"
    );

    for i in [7, 6, 5, 3, 2] {
        s.map.spawns.remove(i);
    }
    assert_eq!(s.validate(), []);
    let sim = Sim::try_new(&s).unwrap();
    let owners: Vec<_> = sim.objects().map(|o| (o.kind, o.owner)).collect();
    assert_eq!(owners, [(pk, Some(PlayerId(1))), (guard, None), (pk, None), (pk, None)]);
    assert!(sim.objects().all(|o| o.orders.is_some() && o.mover.is_some()));

    // Player 1 plays GDO in slot 1. Its layout objects come after all the pre-placed ones. (Layouts hold no
    // blocking footprints yet: stations are map resources.)
    s.map.spawns.push(tile_spawn(station, None, 0, 4)); // x 0..128, y 256..384
    let layout = |kind, x, y| LayoutObject {
        kind,
        at: SpawnAt::Point(Vec2::new(x, y)),
        orientation: Default::default(),
    };
    *s.map.slots[0].layouts.get_mut(Faction::Gdo) = vec![
        layout(pk, 300.0, 300.0),
        layout(pk, 100.0, 100.0),    // on pre-placed spawn 0
        layout(guard, 320.0, 300.0), // on layout object 0
        layout(pk, 384.0, 128.0),    // in the water
        layout(pk, 64.0, 320.0),     // on the station
    ];
    // Nobody plays the Syndicate in slot 1.
    *s.map.slots[0].layouts.get_mut(Faction::Syndicate) = vec![layout(guard, 384.0, 128.0)];
    assert_eq!(s.map.validate(), [], "on the map alone, each layout object fits");
    let invalid = |index, kind, problem| ScenarioProblem::InvalidLayoutObject {
        slot: SlotId(1),
        faction: Faction::Gdo,
        index,
        kind,
        problem,
    };
    let expected = vec![
        invalid(1, pk, SpawnProblem::OverlapsUnit),
        invalid(2, guard, SpawnProblem::OverlapsUnit),
        invalid(3, pk, SpawnProblem::Blocked),
        invalid(4, pk, SpawnProblem::Blocked),
    ];
    assert_eq!(s.validate(), expected);
    assert_eq!(Sim::try_new(&s), Err(expected));
}
