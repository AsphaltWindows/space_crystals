//! Built-in maps, `MapDef::validate` and `Scenario::validate`, and how a game starts from a map and a game setup.

use sc_sim::map::MAX_MAP_SIDE;
use sc_sim::scenario::MAP_FORMAT_VERSION;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    CornerPos, Faction, GameSetup, Map, MapType, ObjectId, ObjectKind, PlayerId, PlayerSetup, Scenario,
    ScenarioProblem, SettingsProblem, Sim, SlotId, Spawn, SpawnAt, SpawnProblem, SpawnSettings, Symmetry, TilePos,
    Vec2, scenarios,
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

#[test]
fn slot_problems() {
    let mut s = scenario();
    s.map.slots[0].start = TilePos::new(6, 0);
    s.map.slots[1].camera_start = Some(Vec2::new(f32::NAN, 10.0));
    let problems = s.validate();
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert_eq!(
        problems[0],
        ScenarioProblem::StartOffMap {
            slot: SlotId(1),
            tile: TilePos::new(6, 0)
        }
    );
    assert!(matches!(
        problems[1],
        ScenarioProblem::CameraStartOffMap { slot: SlotId(2), pos } if pos.x.is_nan()
    ));

    let mut s = scenario();
    s.map.slots[0].camera_start = Some(Vec2::new(10.0, 256.0));
    assert_eq!(s.validate().len(), 1);

    // Slots come in whole sets of images, up to the maximum.
    let mut s = scenario();
    s.map.slots.clear();
    s.setup.players.clear();
    assert_eq!(s.map.validate(), [ScenarioProblem::NoSlots]);
    let mut s = scenarios::dev_skirmish();
    s.map.slots.pop();
    assert_eq!(
        s.map.validate(),
        [ScenarioProblem::SlotsNotInImageSets { slots: 1, images: 2 }]
    );
    let mut s = scenario();
    s.map.slots = (0..9).map(|x| sc_sim::Slot::new(TilePos::new(x % 6, 0))).collect();
    assert_eq!(s.map.validate(), [ScenarioProblem::TooManySlots(9)]);
}

/// The game setup: players numbered from 1, each in a slot of the map, no slot taken twice.
#[test]
fn setup_problems() {
    let mut s = scenario();
    s.setup.players.clear();
    assert_eq!(s.validate(), [ScenarioProblem::NoPlayers]);
    assert_eq!(s.map.validate(), [], "the map is fine on its own");

    let mut s = scenario();
    s.setup.players[1].slot = SlotId(1);
    s.setup.players.push(PlayerSetup {
        name: "Third".into(),
        faction: Faction::Cults,
        slot: SlotId(3),
    });
    assert_eq!(
        s.validate(),
        [
            ScenarioProblem::SlotTaken {
                player: PlayerId(2),
                slot: SlotId(1)
            },
            ScenarioProblem::UnknownSlot {
                player: PlayerId(3),
                slot: SlotId(3)
            },
        ]
    );
}

/// A player's slot decides where they start; a slot's objects go to whoever takes it, and stay out of the game
/// when nobody does. Factions, names and seeds are the setup's.
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
    };
    let sim = Sim::try_new(&s).unwrap();
    let players: Vec<_> = sim
        .players()
        .iter()
        .map(|p| (p.id, p.name.as_str(), p.faction))
        .collect();
    assert_eq!(players, [(PlayerId(1), "Ann", Faction::Syndicate)]);
    assert_eq!(s.setup.player_at(SlotId(2)), Some(PlayerId(1)));
    assert_eq!(s.setup.player_at(SlotId(1)), None);
    let mut reseeded = s.clone();
    reseeded.setup.seed = 43;
    assert_ne!(Sim::new(&reseeded).state_hash(), sim.state_hash());

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

#[test]
fn spawn_problems() {
    let mut s = scenario();
    let patch = ObjectKind::CrystalPatch;
    let station = ObjectKind::SupplyDeliveryStation;
    s.map.spawns = vec![
        tile_spawn(patch, None, 0, 0),
        tile_spawn(patch, Some(SlotId(1)), 1, 0),
        tile_spawn(station, None, 5, 0),
        tile_spawn(patch, None, -1, 3),
        Spawn::new(patch, None, SpawnAt::Point(Vec2::new(32.0, 32.0))),
        tile_spawn(station, None, i32::MAX, i32::MIN),
    ];
    let invalid = |spawn, kind, problem| ScenarioProblem::InvalidSpawn { spawn, kind, problem };
    assert_eq!(
        s.validate(),
        [
            invalid(1, patch, SpawnProblem::OwnedResource),
            invalid(2, station, SpawnProblem::FootprintOffMap),
            invalid(3, patch, SpawnProblem::FootprintOffMap),
            invalid(4, patch, SpawnProblem::WrongLocationType),
            invalid(5, station, SpawnProblem::FootprintOffMap),
        ]
    );
}

/// `core/map_editor.md > Per-object Resource Values`: every patch and station carries its own values, of its own
/// kind, and they make sense.
#[test]
fn settings_problems() {
    let mut s = scenario();
    let patch = ObjectKind::CrystalPatch;
    let station = ObjectKind::SupplyDeliveryStation;
    let station_settings = |starting_supplies, delivery_size, delivery_interval| {
        Some(SpawnSettings::SupplyDeliveryStation {
            starting_supplies,
            delivery_size,
            delivery_interval,
        })
    };
    let with = |mut spawn: Spawn, settings| {
        spawn.settings = settings;
        spawn
    };
    s.map.spawns = vec![
        with(tile_spawn(patch, None, 0, 0), None),
        with(tile_spawn(patch, None, 1, 0), station_settings(0, 4, 960)),
        with(
            tile_spawn(patch, None, 2, 0),
            Some(SpawnSettings::CrystalPatch { starting_amount: 0 }),
        ),
        with(tile_spawn(station, None, 0, 2), station_settings(0, 0, 960)),
        with(tile_spawn(station, None, 2, 2), station_settings(5, 4, 0)),
        with(tile_spawn(station, None, 4, 2), station_settings(u32::MAX, 1, u32::MAX)),
        with(
            tile_spawn(patch, None, 5, 0),
            Some(SpawnSettings::CrystalPatch { starting_amount: 1 }),
        ),
    ];
    let invalid = |spawn, kind, problem| ScenarioProblem::InvalidSpawn { spawn, kind, problem };
    assert_eq!(
        s.validate(),
        [
            invalid(0, patch, SpawnProblem::MissingSettings),
            invalid(1, patch, SpawnProblem::WrongSettings),
            invalid(2, patch, SpawnProblem::InvalidSettings(SettingsProblem::EmptyPatch)),
            invalid(
                3,
                station,
                SpawnProblem::InvalidSettings(SettingsProblem::EmptyDelivery)
            ),
            invalid(4, station, SpawnProblem::InvalidSettings(SettingsProblem::ZeroInterval)),
        ]
    );
    assert!(Sim::try_new(&s).is_err());
    s.map.spawns.drain(..5);
    assert_eq!(s.validate(), []);
    Sim::try_new(&s).unwrap().step(&Default::default());
}

/// `core/map.md > Elevation > Maximum slope`: each Traversible tile whose corners differ by more than 1 is reported
/// with its position; non-Traversible tiles may be as steep as they like.
#[test]
fn too_steep_tiles() {
    let map = Map::from_ascii(
        "
        .r.c
        ~..c

        00022
        00248
        00000
        ",
    )
    .unwrap();
    let mut s = ScenarioBuilder::new("....\n....").build();
    s.map.terrain = map;
    let steep = |x, y, slope| ScenarioProblem::TooSteep {
        tile: TilePos::new(x, y),
        slope,
    };
    // The Rugged (1, 0) rises 0 → 2, the Plane (2, 0) 0 → 4, (1, 1) 0 → 2 and (2, 1) 0 → 4. The cliffs in the east
    // column and the water at (0, 1) are fine.
    let problems = s.validate();
    assert_eq!(
        problems,
        [steep(1, 0, 2), steep(2, 0, 4), steep(1, 1, 2), steep(2, 1, 4)]
    );
    assert_eq!(Sim::try_new(&s), Err(problems));
    // Flattening the rise makes the map valid.
    for (x, y, height) in [(2, 1, 0), (3, 1, 0), (3, 0, 1)] {
        s.map.terrain.set_corner(CornerPos::new(x, y), height);
    }
    assert_eq!(s.validate(), []);
}

/// `core/map.md > Elevation > Map resources only on flat terrain`: every footprint tile of a patch or station is
/// flat; validation names the first sloped one.
#[test]
fn resources_need_flat_ground() {
    let mut s = ScenarioBuilder::new(
        "
        ......
        ......
        ......
        ......

        0000000
        0000000
        0000111
        0000111
        0000111
        ",
    )
    .build();
    let patch = ObjectKind::CrystalPatch;
    let station = ObjectKind::SupplyDeliveryStation;
    s.map.spawns = vec![
        tile_spawn(patch, None, 0, 0),
        tile_spawn(patch, None, 3, 2),
        tile_spawn(station, None, 1, 0),
        tile_spawn(station, None, 3, 1),
        // On top of the rise: flat at level 1.
        tile_spawn(patch, None, 4, 2),
        tile_spawn(station, None, 4, 2),
    ];
    let not_flat = |spawn, kind, x, y| ScenarioProblem::InvalidSpawn {
        spawn,
        kind,
        problem: SpawnProblem::NotFlat(TilePos::new(x, y)),
    };
    let problems = s.validate();
    assert_eq!(
        problems,
        [
            not_flat(1, patch, 3, 2),
            not_flat(3, station, 3, 1),
            ScenarioProblem::OverlappingFootprints {
                first: 4,
                second: 5,
                tile: TilePos::new(4, 2)
            },
        ]
    );
    assert_eq!(
        problems[1].to_string(),
        "spawn 3 (SupplyDeliveryStation): it needs flat ground, but its tile (3, 1) is sloped"
    );
    // Reshaping the ground under a patch makes it invalid.
    s.map.spawns = vec![tile_spawn(patch, None, 0, 0)];
    assert_eq!(s.validate(), []);
    s.map.terrain.set_corner(CornerPos::new(1, 1), 1);
    assert_eq!(s.validate(), [not_flat(0, patch, 0, 0)]);
}

#[test]
fn maps_over_the_largest_side_are_invalid() {
    let mut s = scenario();
    s.map.terrain = Map::new(MAX_MAP_SIDE + 1, 4);
    assert_eq!(s.validate(), [ScenarioProblem::MapTooBig { width: 257, height: 4 }]);
    assert_eq!(
        s.validate()[0].to_string(),
        "the map is 257 x 4 tiles, more than 256 x 256"
    );
}

#[test]
fn overlapping_footprints() {
    let mut s = scenario();
    let patch = ObjectKind::CrystalPatch;
    let station = ObjectKind::SupplyDeliveryStation;
    s.map.spawns = vec![
        tile_spawn(station, None, 1, 1), // (1..=2, 1..=2)
        tile_spawn(patch, None, 2, 2),
        tile_spawn(patch, None, 4, 0),
        tile_spawn(station, None, 2, 2), // (2..=3, 2..=3)
        tile_spawn(patch, None, 4, 0),
    ];
    let overlap = |first, second, x, y| ScenarioProblem::OverlappingFootprints {
        first,
        second,
        tile: TilePos::new(x, y),
    };
    assert_eq!(
        s.validate(),
        [overlap(0, 1, 2, 2), overlap(0, 3, 2, 2), overlap(2, 4, 4, 0)]
    );
}

#[test]
fn try_new_lists_the_problems_instead_of_panicking() {
    let mut s = scenario();
    s.setup.players[1].slot = SlotId(7);
    s.map
        .spawns
        .push(tile_spawn(ObjectKind::CrystalPatch, Some(SlotId(7)), 0, 0));
    let problems = s.validate();
    assert_eq!(problems.len(), 2);
    assert_eq!(Sim::try_new(&s), Err(problems));
}

#[test]
fn problems_read_as_messages() {
    let messages: Vec<String> = [
        ScenarioProblem::StartOffMap {
            slot: SlotId(1),
            tile: TilePos::new(-1, 4),
        },
        ScenarioProblem::InvalidSpawn {
            spawn: 3,
            kind: ObjectKind::CrystalPatch,
            problem: SpawnProblem::UnknownSlot(SlotId(5)),
        },
        ScenarioProblem::OverlappingFootprints {
            first: 0,
            second: 4,
            tile: TilePos::new(2, 3),
        },
        ScenarioProblem::TooSteep {
            tile: TilePos::new(4, 7),
            slope: 3,
        },
        ScenarioProblem::InvalidSpawn {
            spawn: 1,
            kind: ObjectKind::SupplyDeliveryStation,
            problem: SpawnProblem::InvalidSettings(SettingsProblem::ZeroInterval),
        },
        ScenarioProblem::InvalidSpawn {
            spawn: 2,
            kind: ObjectKind::CrystalPatch,
            problem: SpawnProblem::MissingSettings,
        },
        ScenarioProblem::SlotsNotInImageSets { slots: 3, images: 2 },
        ScenarioProblem::SlotTaken {
            player: PlayerId(2),
            slot: SlotId(1),
        },
        ScenarioProblem::NotSymmetric(sc_sim::scenario::Asymmetry::Tile(TilePos::new(40, 3))),
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    assert_eq!(
        messages,
        [
            "slot 1: start (-1, 4) is off the map",
            "spawn 3 (CrystalPatch): owner slot 5 is not a slot of the map",
            "spawns 0 and 4 overlap at tile (2, 3)",
            "tile (4, 7) is Traversible but 3 levels steep (at most 1): flatten it or make it a cliff",
            "spawn 1 (SupplyDeliveryStation): the delivery interval must be at least 1 tick",
            "spawn 2 (CrystalPatch): its per-object values are missing",
            "3 player slots don't come in whole sets of 2 symmetric images",
            "player 2 takes slot 1, which another player took",
            "tile (40, 3) is not generated from the editable part: the next edit regenerates it",
        ]
    );
}
