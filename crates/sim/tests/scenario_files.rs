//! Map files: the versioned, human-readable JSON form of a `MapDef` that the map editor saves and the game loads,
//! and the JSON form of a whole `Scenario` (map and game setup) that replays use. A bad file must never crash
//! either.

use sc_sim::scenario::MAP_FORMAT_VERSION;
use sc_sim::{
    Faction, GameSetup, Map, MapDef, MapType, ObjectKind, PlayerSetup, Rng, Scenario, Sim, SimConfig, Slot, SlotId,
    Spawn, SpawnAt, SpawnSettings, Symmetry, TickInputs, TileLook, TilePos, TilePreset, Vec2, scenarios,
};

fn tiny_map() -> MapDef {
    let mut terrain = Map::from_ascii("..~\n.c.\n\n0000\n0011\n0111").unwrap();
    let ash = TilePreset {
        name: "Ash".into(),
        buildable: true,
        traversible: true,
        rugged: false,
        drillable: false,
        recruitable: true,
        look: TileLook::Rugged,
    };
    let ash = terrain.add_preset_with_symbol(ash, 'a').unwrap();
    terrain.set_tile(TilePos::new(0, 0), sc_sim::Tile::new(ash));
    let mut slot = Slot::new(TilePos::new(0, 1));
    slot.camera_start = Some(Vec2::new(100.0, 70.0));
    MapDef {
        map_type: MapType::Custom,
        name: "tiny".into(),
        description: "A test map.".into(),
        terrain,
        symmetry: Symmetry::None,
        slots: vec![slot],
        spawns: vec![Spawn {
            kind: ObjectKind::CrystalPatch,
            owner: None,
            at: SpawnAt::Tile(TilePos::new(2, 1)),
            orientation: Default::default(),
            settings: Some(SpawnSettings::CrystalPatch { starting_amount: 900 }),
        }],
    }
}

const TINY_FILE: &str = r#"{
  "version": 3,
  "map_type": "Custom",
  "name": "tiny",
  "description": "A test map.",
  "symmetry": "None",
  "terrain": {
    "custom_presets": [
      {
        "symbol": "a",
        "name": "Ash",
        "buildable": true,
        "traversible": true,
        "rugged": false,
        "drillable": false,
        "recruitable": true,
        "look": "Rugged"
      }
    ],
    "tiles": [
      "a.~",
      ".c."
    ],
    "heights": [
      "0000",
      "0011",
      "0111"
    ]
  },
  "slots": [
    {
      "start": {
        "x": 0,
        "y": 1
      },
      "camera_start": [
        100.0,
        70.0
      ],
      "layouts": {
        "Gdo": [],
        "Syndicate": [],
        "Cults": [],
        "Colonists": []
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
      },
      "settings": {
        "CrystalPatch": {
          "starting_amount": 900
        }
      }
    }
  ]
}"#;

#[test]
fn file_format() {
    assert_eq!(MAP_FORMAT_VERSION, 3);
    assert_eq!(serde_json::to_string_pretty(&tiny_map()).unwrap(), TINY_FILE);
    assert_eq!(serde_json::from_str::<MapDef>(TINY_FILE).unwrap(), tiny_map());
    assert_eq!(tiny_map().validate(), []);
}

/// A scenario is the map file plus the game setup, which the map file never holds.
#[test]
fn scenarios_are_a_map_and_a_setup() {
    let scenario = Scenario {
        map: tiny_map(),
        setup: GameSetup {
            players: vec![PlayerSetup {
                name: "Ann".into(),
                faction: Faction::Cults,
                slot: SlotId(1),
            }],
            seed: 7,
            config: SimConfig::DEBUG,
        },
    };
    let json = serde_json::to_value(&scenario).unwrap();
    assert_eq!(
        json["setup"],
        serde_json::json!({
            "players": [{ "name": "Ann", "faction": "Cults", "slot": 1 }],
            "seed": 7,
            "config": { "debug": true }
        })
    );
    assert_eq!(json["map"], serde_json::to_value(tiny_map()).unwrap());
    let back: Scenario = serde_json::from_value(json).unwrap();
    assert_eq!(back, scenario);
    assert!(!TINY_FILE.contains("seed") && !TINY_FILE.contains("faction"));
}

#[test]
fn dev_skirmish_round_trips_as_a_short_readable_file() {
    let map = scenarios::dev_skirmish().map;
    let json = serde_json::to_string_pretty(&map).unwrap();
    // One line per map row: 48 tile rows and 49 corner height rows, not one object per tile. The two slots' layouts
    // of the four factions take about 450 lines.
    let lines = json.lines().count();
    assert!(lines < 900, "{lines} lines");
    let ascii = map.terrain.to_ascii();
    let first_heights = ascii.split("\n\n").nth(1).unwrap().lines().next().unwrap().to_string();
    assert_eq!(first_heights.len(), 65);
    assert!(json.contains(&format!("      \"{first_heights}\",\n")));
    let first_row = ascii.lines().next().unwrap().to_string();
    assert!(json.contains(&format!("      \"{first_row}\",\n")));
    let back: MapDef = serde_json::from_str(&json).unwrap();
    assert_eq!(back, map);
    assert_eq!(serde_json::to_string_pretty(&back).unwrap(), json);
    assert_eq!(
        Sim::new(&Scenario::new(back)).state_hash(),
        Sim::new(&Scenario::new(map)).state_hash()
    );
}

#[test]
fn hand_written_files_may_omit_optional_fields() {
    let json = r#"{
        "version": 3, "map_type": "Custom", "name": "hand-made", "symmetry": "None",
        "terrain": { "tiles": ["....", ".~~."] },
        "slots": [ { "start": { "x": 0, "y": 0 } } ],
        "spawns": []
    }"#;
    let map: MapDef = serde_json::from_str(json).unwrap();
    assert_eq!(map.terrain, Map::from_ascii("....\n.~~.").unwrap());
    assert_eq!(map.description, "");
    assert_eq!(map.slots[0].camera_start, None);
    assert!(map.slots[0].layouts.is_empty());
    assert_eq!(map.validate(), []);
}

#[test]
fn reading_rejects_other_versions_unknown_fields_and_typos() {
    let json = serde_json::to_string(&tiny_map()).unwrap();
    let error = |json: &str| serde_json::from_str::<MapDef>(json).unwrap_err().to_string();
    // Version 2 (players and seed in the map) is not read: there is no loader for old versions.
    let v2 = json.replacen("\"version\":3", "\"version\":2", 1);
    assert!(
        error(&v2).contains("unsupported map format version 2 (this build reads version 3)"),
        "{}",
        error(&v2)
    );
    // A real version 2 file, with fields this version doesn't know, gets the same error.
    let old = r#"{ "version": 2, "name": "old", "map": { "tiles": [".."] }, "players": [], "spawns": [], "seed": 1 }"#;
    assert!(
        error(old).contains("unsupported map format version 2"),
        "{}",
        error(old)
    );
    let v4 = json.replacen("\"version\":3", "\"version\":4", 1);
    assert!(error(&v4).contains("unsupported map format version 4"));
    let unversioned = json.replacen("\"version\":3,", "", 1);
    assert!(error(&unversioned).contains("missing field `version`"));
    let extra = json.replacen("\"name\":\"tiny\"", "\"name\":\"tiny\",\"seed\":0", 1);
    assert!(error(&extra).contains("unknown field `seed`"), "{}", error(&extra));
    let typo = json.replacen("\"camera_start\"", "\"camera_strat\"", 1);
    assert!(
        error(&typo).contains("unknown field `camera_strat`"),
        "{}",
        error(&typo)
    );
    let missing = json.replacen("\"symmetry\":\"None\",", "", 1);
    assert!(
        error(&missing).contains("missing field `symmetry`"),
        "{}",
        error(&missing)
    );
    let bad_tile = json.replacen("\"a.~\"", "\"a.?\"", 1);
    assert!(error(&bad_tile).contains("unknown tile symbol '?' at tile (2, 0)"));
    let bad_look = json.replacen("\"Rugged\"", "\"Lava\"", 1);
    assert!(
        error(&bad_look).contains("unknown variant `Lava`"),
        "{}",
        error(&bad_look)
    );
}

/// Damaged files: every truncation and many random single-character edits of a valid
/// file either fail to parse or give a map that validation handles, never a panic.
#[test]
fn damaged_files_never_panic() {
    let mut map = scenarios::empty_map(4, 3);
    // The patches and the station stand on flat ground (`core/map.md > Elevation > Map resources only on flat
    // terrain`).
    map.terrain = Map::from_ascii("..r~\n.cM.\n....\n\n11122\n11112\n01111\n01111").unwrap();
    let tile_spawn = |kind, x, y| Spawn::new(kind, None, SpawnAt::Tile(TilePos::new(x, y)));
    map.spawns = vec![
        tile_spawn(ObjectKind::CrystalPatch, 0, 0),
        tile_spawn(ObjectKind::CrystalPatch, 3, 2),
        tile_spawn(ObjectKind::SupplyDeliveryStation, 1, 1),
    ];
    for s in &mut map.slots {
        s.camera_start = Some(Vec2::new(100.0, 100.0));
    }
    assert_eq!(map.validate(), []);
    let json = serde_json::to_string(&map).unwrap();
    let mut loaded = 0;
    let mut try_load = |text: &str| {
        let Ok(map) = serde_json::from_str::<MapDef>(text) else {
            return;
        };
        let scenario = Scenario::new(map);
        let problems = scenario.validate();
        match Sim::try_new(&scenario) {
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
