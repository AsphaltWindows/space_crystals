//! Scenario files: the versioned, human-readable JSON form of a `Scenario` that the map
//! editor saves and the game loads. A bad file must never crash either.

use sc_sim::scenario::SCENARIO_FORMAT_VERSION;
use sc_sim::{
    Faction, Map, ObjectKind, PlayerId, PlayerSetup, Rng, Scenario, Sim, Spawn, SpawnAt, TickInputs, TilePos,
    TilePreset, scenarios,
};

fn tiny_scenario() -> Scenario {
    let mut map = Map::from_ascii("..~\n.c.\n\n001\n012").unwrap();
    let ash = TilePreset {
        name: "Ash".into(),
        buildable: true,
        traversible: true,
        rugged: false,
        drillable: false,
        recruitable: true,
    };
    let ash = map.add_preset_with_symbol(ash, 'a').unwrap();
    map.set_tile(TilePos::new(0, 0), sc_sim::Tile::new(ash, 0));
    Scenario {
        name: "tiny".into(),
        map,
        players: vec![PlayerSetup {
            id: PlayerId(1),
            name: "P1".into(),
            faction: Faction::Gdo,
            camera_start: None,
            start_location: Some(TilePos::new(0, 1)),
        }],
        spawns: vec![Spawn {
            kind: ObjectKind::CrystalPatch,
            owner: None,
            at: SpawnAt::Tile(TilePos::new(2, 1)),
        }],
        seed: 7,
    }
}

const TINY_FILE: &str = r#"{
  "version": 1,
  "name": "tiny",
  "map": {
    "custom_presets": [
      {
        "symbol": "a",
        "name": "Ash",
        "buildable": true,
        "traversible": true,
        "rugged": false,
        "drillable": false,
        "recruitable": true
      }
    ],
    "tiles": [
      "a.~",
      ".c."
    ],
    "elevation": [
      "001",
      "012"
    ]
  },
  "players": [
    {
      "id": 1,
      "name": "P1",
      "faction": "Gdo",
      "camera_start": null,
      "start_location": {
        "x": 0,
        "y": 1
      }
    }
  ],
  "spawns": [
    {
      "kind": "CrystalPatch",
      "owner": null,
      "at": {
        "Tile": {
          "x": 2,
          "y": 1
        }
      }
    }
  ],
  "seed": 7
}"#;

#[test]
fn file_format() {
    assert_eq!(SCENARIO_FORMAT_VERSION, 1);
    assert_eq!(serde_json::to_string_pretty(&tiny_scenario()).unwrap(), TINY_FILE);
    assert_eq!(serde_json::from_str::<Scenario>(TINY_FILE).unwrap(), tiny_scenario());
}

#[test]
fn dev_skirmish_round_trips_as_a_short_readable_file() {
    let scenario = scenarios::dev_skirmish();
    let json = serde_json::to_string_pretty(&scenario).unwrap();
    // One line per map row: 48 tile rows and 48 elevation rows, not one object per tile.
    let lines = json.lines().count();
    assert!(lines < 400, "{lines} lines");
    let first_row = scenario.map.to_ascii().lines().next().unwrap().to_string();
    assert!(json.contains(&format!("      \"{first_row}\",\n")));
    let back: Scenario = serde_json::from_str(&json).unwrap();
    assert_eq!(back, scenario);
    assert_eq!(serde_json::to_string_pretty(&back).unwrap(), json);
    assert_eq!(Sim::new(&back).state_hash(), Sim::new(&scenario).state_hash());
}

#[test]
fn hand_written_files_may_omit_optional_fields() {
    let json = r#"{
        "version": 1, "name": "hand-made", "seed": 3,
        "map": { "tiles": ["....", ".~~."] },
        "players": [ { "id": 1, "name": "Ann", "faction": "Cults" } ],
        "spawns": []
    }"#;
    let scenario: Scenario = serde_json::from_str(json).unwrap();
    assert_eq!(scenario.map, Map::from_ascii("....\n.~~.").unwrap());
    assert_eq!(scenario.players[0].camera_start, None);
    assert_eq!(scenario.players[0].start_location, None);
    assert_eq!(scenario.validate(), []);
}

#[test]
fn reading_rejects_other_versions_unknown_fields_and_typos() {
    let json = serde_json::to_string(&tiny_scenario()).unwrap();
    let error = |json: &str| serde_json::from_str::<Scenario>(json).unwrap_err().to_string();
    let v2 = json.replacen("\"version\":1", "\"version\":2", 1);
    assert!(
        error(&v2).contains("unsupported scenario format version 2 (this build reads version 1)"),
        "{}",
        error(&v2)
    );
    let unversioned = json.replacen("\"version\":1,", "", 1);
    assert!(error(&unversioned).contains("missing field `version`"));
    let extra = json.replacen("\"seed\":7", "\"seed\":7,\"extra\":0", 1);
    assert!(error(&extra).contains("unknown field `extra`"));
    let typo = json.replacen("\"start_location\"", "\"start_locaton\"", 1);
    assert!(error(&typo).contains("unknown field `start_locaton`"));
    let bad_tile = json.replacen("\"a.~\"", "\"a.?\"", 1);
    assert!(error(&bad_tile).contains("unknown tile symbol '?' at tile (2, 0)"));
}

/// Damaged files: every truncation and many random single-character edits of a valid
/// file either fail to parse or give a scenario that validation handles, never a panic.
#[test]
fn damaged_files_never_panic() {
    let mut scenario = scenarios::dev_skirmish();
    scenario.map = Map::from_ascii("..r~\n.cM.\n....\n\n0123\n4567\n89ag").unwrap();
    let tile_spawn = |kind, x, y| Spawn {
        kind,
        owner: None,
        at: SpawnAt::Tile(TilePos::new(x, y)),
    };
    scenario.spawns = vec![
        tile_spawn(ObjectKind::CrystalPatch, 0, 0),
        tile_spawn(ObjectKind::CrystalPatch, 3, 2),
        tile_spawn(ObjectKind::SupplyDeliveryStation, 1, 1),
    ];
    for p in &mut scenario.players {
        p.camera_start = Some(sc_sim::Vec2::new(100.0, 100.0));
        p.start_location = Some(TilePos::new(1, 1));
    }
    assert_eq!(scenario.validate(), []);
    let json = serde_json::to_string(&scenario).unwrap();
    let mut loaded = 0;
    let mut try_load = |text: &str| {
        let Ok(s) = serde_json::from_str::<Scenario>(text) else {
            return;
        };
        let problems = s.validate();
        match Sim::try_new(&s) {
            Ok(mut sim) => {
                assert!(problems.is_empty());
                sim.step(&TickInputs::default());
                loaded += 1;
            }
            Err(p) => assert_eq!(p, problems),
        }
    };
    for end in 0..json.len() {
        try_load(&json[..end]);
    }
    let edits = b"0123456789-.,:{}[]\" eEaxM~rcg";
    let mut rng = Rng::from_seed(99);
    for _ in 0..5000 {
        let mut bytes = json.clone().into_bytes();
        let i = rng.below(bytes.len() as u32) as usize;
        bytes[i] = edits[rng.below(edits.len() as u32) as usize];
        if let Ok(text) = String::from_utf8(bytes) {
            try_load(&text);
        }
    }
    try_load(&json);
    assert!(loaded > 100, "only {loaded} edited files loaded");
}
