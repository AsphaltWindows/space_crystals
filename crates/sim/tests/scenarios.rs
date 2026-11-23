//! Built-in scenarios and `Scenario::validate`.

use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    ObjectId, ObjectKind, PlayerId, Scenario, ScenarioProblem, Sim, Spawn, SpawnAt, SpawnProblem, TilePos, Vec2,
    scenarios,
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
    assert_eq!(starts, [Some(TilePos::new(10, 10)), Some(TilePos::new(53, 37))]);
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
    Spawn {
        kind,
        owner,
        at: SpawnAt::Tile(TilePos::new(x, y)),
    }
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
        Spawn {
            kind: patch,
            owner: None,
            at: SpawnAt::Point(Vec2::new(32.0, 32.0)),
        },
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
        ]
    );
}
