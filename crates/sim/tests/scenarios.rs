//! Built-in scenarios and `Scenario::validate`.

use sc_sim::map::MAX_MAP_SIDE;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    CornerPos, Map, ObjectId, ObjectKind, PlayerId, Scenario, ScenarioProblem, SettingsProblem, Sim, Spawn, SpawnAt,
    SpawnProblem, SpawnSettings, TilePos, Vec2, scenarios,
};

#[test]
fn built_in_scenarios_load() {
    for &name in scenarios::NAMES {
        let scenario = scenarios::by_name(name).unwrap();
        assert_eq!(scenario.name, name);
        assert_eq!(scenario.validate(), [], "{name}");
        let sim = Sim::try_new(&scenario).unwrap();
        assert_eq!(sim.tick(), 0);
        assert!(sim.events().is_empty());
        assert_eq!(sim.players().len(), 2);
        for p in &scenario.players {
            let start = p.start_location.unwrap_or_else(|| panic!("{name}: no start location"));
            assert!(scenario.map.in_bounds(start), "{name}: {start:?}");
        }
    }
    assert!(scenarios::by_name("no_such_map").is_none());
}

#[test]
fn dev_skirmish_layout() {
    let scenario = scenarios::dev_skirmish();
    assert_eq!((scenario.map.width(), scenario.map.height()), (64, 48));
    let starts: Vec<_> = scenario.players.iter().map(|p| p.start_location).collect();
    // In opposite corners, mirrored through the center.
    assert_eq!(starts, [Some(TilePos::new(8, 8)), Some(TilePos::new(55, 39))]);
    let sim = Sim::new(&scenario);
    // Spawns get ascending ids in list order.
    let kinds: Vec<_> = sim.objects().map(|o| (o.id, o.kind)).collect();
    let expected: Vec<_> = (1..=17)
        .map(|i| {
            let kind = if i <= 16 {
                ObjectKind::CrystalPatch
            } else {
                ObjectKind::SupplyDeliveryStation
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
    let starts: Vec<_> = scenarios::empty(32, 24)
        .players
        .iter()
        .map(|p| p.start_location)
        .collect();
    assert_eq!(starts, [Some(TilePos::new(8, 6)), Some(TilePos::new(23, 17))]);
}

/// 6 × 4 tiles, 384 × 256 su, two players (ids 1 and 2).
fn scenario() -> Scenario {
    ScenarioBuilder::new("......\n......\n......\n......").build()
}

fn tile_spawn(kind: ObjectKind, owner: Option<PlayerId>, x: i32, y: i32) -> Spawn {
    Spawn::new(kind, owner, SpawnAt::Tile(TilePos::new(x, y)))
}

#[test]
fn player_problems() {
    let mut s = scenario();
    s.players[1].id = PlayerId(1);
    assert_eq!(s.validate(), [ScenarioProblem::DuplicatePlayerId(PlayerId(1))]);

    let mut s = scenario();
    s.players[0].id = PlayerId(0);
    s.players[1].id = PlayerId(3);
    let out_of_range = |id| ScenarioProblem::PlayerIdOutOfRange {
        player: PlayerId(id),
        players: 2,
    };
    assert_eq!(s.validate(), [out_of_range(0), out_of_range(3)]);

    let mut s = scenario();
    s.players[0].start_location = Some(TilePos::new(6, 0));
    s.players[1].camera_start = Some(Vec2::new(f32::NAN, 10.0));
    let problems = s.validate();
    assert_eq!(problems.len(), 2);
    assert_eq!(
        problems[0],
        ScenarioProblem::StartLocationOffMap {
            player: PlayerId(1),
            tile: TilePos::new(6, 0)
        }
    );
    assert!(matches!(
        problems[1],
        ScenarioProblem::CameraStartOffMap { player: PlayerId(2), pos } if pos.x.is_nan()
    ));

    let mut s = scenario();
    s.players[0].camera_start = Some(Vec2::new(10.0, 256.0));
    assert_eq!(s.validate().len(), 1);
}

#[test]
fn spawn_problems() {
    let mut s = scenario();
    let patch = ObjectKind::CrystalPatch;
    let station = ObjectKind::SupplyDeliveryStation;
    s.spawns = vec![
        tile_spawn(patch, None, 0, 0),
        tile_spawn(patch, Some(PlayerId(1)), 1, 0),
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
    s.spawns = vec![
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
    s.spawns.drain(..5);
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
    s.map = map;
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
        s.map.set_corner(CornerPos::new(x, y), height);
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
    s.spawns = vec![
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
    s.spawns = vec![tile_spawn(patch, None, 0, 0)];
    assert_eq!(s.validate(), []);
    s.map.set_corner(CornerPos::new(1, 1), 1);
    assert_eq!(s.validate(), [not_flat(0, patch, 0, 0)]);
}

#[test]
fn maps_over_the_largest_side_are_invalid() {
    let mut s = scenario();
    s.map = Map::new(MAX_MAP_SIDE + 1, 4);
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
    s.spawns = vec![
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
    s.players[1].id = PlayerId(7);
    s.spawns
        .push(tile_spawn(ObjectKind::CrystalPatch, Some(PlayerId(7)), 0, 0));
    let problems = s.validate();
    assert_eq!(problems.len(), 2);
    assert_eq!(Sim::try_new(&s), Err(problems));
}

#[test]
fn problems_read_as_messages() {
    let messages: Vec<String> = [
        ScenarioProblem::DuplicatePlayerId(PlayerId(2)),
        ScenarioProblem::PlayerIdOutOfRange {
            player: PlayerId(0),
            players: 2,
        },
        ScenarioProblem::StartLocationOffMap {
            player: PlayerId(1),
            tile: TilePos::new(-1, 4),
        },
        ScenarioProblem::InvalidSpawn {
            spawn: 3,
            kind: ObjectKind::CrystalPatch,
            problem: SpawnProblem::UnknownOwner(PlayerId(5)),
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
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    assert_eq!(
        messages,
        [
            "player id 2 is used more than once",
            "player id 0 is outside 1..=2 (players are numbered 1 to n)",
            "player 1: start location (-1, 4) is off the map",
            "spawn 3 (CrystalPatch): owner 5 is not a player",
            "spawns 0 and 4 overlap at tile (2, 3)",
            "tile (4, 7) is Traversible but 3 levels steep (at most 1): flatten it or make it a cliff",
            "spawn 1 (SupplyDeliveryStation): the delivery interval must be at least 1 tick",
            "spawn 2 (CrystalPatch): its per-object values are missing",
        ]
    );
}
