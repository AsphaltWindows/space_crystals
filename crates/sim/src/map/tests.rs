use super::*;
use crate::scenarios;

fn hash(map: &Map) -> u64 {
    let mut h = StateHasher::new();
    map.hash(&mut h);
    h.finish()
}

fn tile(map: &Map, x: i32, y: i32) -> Tile {
    map.tile(TilePos::new(x, y)).unwrap()
}

fn custom(name: &str) -> TilePreset {
    TilePreset {
        name: name.into(),
        buildable: true,
        traversible: false,
        rugged: true,
        drillable: false,
        recruitable: true,
        look: TileLook::Rugged,
    }
}

fn err(ascii: &str) -> String {
    Map::from_ascii(ascii).expect_err("the map should be rejected")
}

#[test]
fn ascii_reads_presets_and_corner_heights() {
    let map = Map::from_ascii(
        "
        .rcM~
        ~Mcr.

        012345
        9abcgg
        000000
        ",
    )
    .unwrap();
    assert_eq!((map.width(), map.height()), (5, 2));
    let presets: Vec<TilePresetId> = (0..5).map(|x| tile(&map, x, 0).preset).collect();
    use TilePresetId as P;
    assert_eq!(presets, [P::PLANE, P::RUGGED, P::CLIFF, P::MOUNTAIN, P::WATER]);
    assert_eq!(tile(&map, 0, 1).preset, P::WATER);
    let heights: Vec<u8> = (0..6).map(|x| map.corner(CornerPos::new(x, 1)).unwrap()).collect();
    assert_eq!(heights, [9, 10, 11, 12, 16, 16]);
    assert_eq!(map.corner(CornerPos::new(5, 0)), Some(5));
    assert_eq!(map.corner(CornerPos::new(6, 0)), None, "corners go up to x = width");
    assert_eq!(map.corner(CornerPos::new(0, 3)), None, "and to y = height");
    assert_eq!(map.tile_corner_heights(TilePos::new(4, 0)), Some([4, 5, 16, 16]));
}

#[test]
fn ascii_without_height_block_is_flat() {
    let map = Map::from_ascii("..\r\n~~\r\n").unwrap();
    assert_eq!((map.width(), map.height()), (2, 2));
    assert!(map.heights.iter().all(|&h| h == 0));
    assert_eq!(map.heights.len(), 9);
    assert_eq!(tile(&map, 1, 1).preset, TilePresetId::WATER);
}

#[test]
fn ascii_round_trips() {
    let map = Map::from_ascii(".r\ncM\n~.\n\n0g0\n5a0\n000\n123").unwrap();
    assert_eq!(map.to_ascii(), ".r\ncM\n~.\n\n0g0\n5a0\n000\n123\n");
    assert_eq!(Map::from_ascii(&map.to_ascii()), Ok(map));
    let skirmish = scenarios::dev_skirmish().map.terrain;
    assert_eq!(Map::from_ascii(&skirmish.to_ascii()), Ok(skirmish));
}

#[test]
fn ascii_errors_say_what_and_where() {
    assert_eq!(err(""), "the map has no tile rows");
    assert_eq!(err(" \n\n "), "the map has no tile rows");
    assert_eq!(err("...\n..\n..."), "ragged tile rows: row 1 has 2 tiles, row 0 has 3");
    assert_eq!(err("...\n.x."), "unknown tile symbol 'x' at tile (1, 1)");
    assert_eq!(err(". ."), "unknown tile symbol ' ' at tile (1, 0)");
    assert_eq!(
        err("..\n..\n\n00\n00\n00"),
        "height row 0 has 2 digits, expected 3 (one per corner: one more than the 2 tiles of a row)"
    );
    assert_eq!(
        err("..\n..\n\n000\n000"),
        "the height block has 2 rows, expected 3 (one per corner row: one more than the 2 tile rows)"
    );
    assert_eq!(
        err("..\n..\n\n000\n0000\n000"),
        "height row 1 has 4 digits, expected 3 (one per corner: one more than the 2 tiles of a row)"
    );
    assert_eq!(
        err("..\n\n00h\n000"),
        "invalid height digit 'h' at corner (2, 0): use 0-9 and a-g for 0 to 16"
    );
    assert!(err("..\n\n0A0\n000").starts_with("invalid height digit 'A'"));
    assert_eq!(
        err("..\n\n000\n000\n\n000"),
        "expected a tile block and an optional height block, found 3 blocks separated by blank lines"
    );
}

#[test]
fn non_finite_points_are_off_the_map() {
    let map = Map::new(4, 3);
    assert!(map.contains_point(Vec2::new(0.0, 0.0)));
    assert!(map.contains_point(Vec2::new(255.9, 191.9)));
    assert!(!map.contains_point(Vec2::new(256.0, 10.0)));
    assert!(!map.contains_point(Vec2::new(-0.1, 10.0)));
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(!map.contains_point(Vec2::new(bad, 10.0)), "{bad}");
        assert!(!map.contains_point(Vec2::new(10.0, bad)), "{bad}");
        assert!(!map.contains_point(Vec2::splat(bad)), "{bad}");
    }
}

#[test]
fn custom_presets_get_free_symbols() {
    let mut map = Map::new(2, 1);
    let a = map.add_preset(custom("a"));
    assert_eq!((a, map.preset_symbol(a)), (TilePresetId(5), 'A'));
    let z = map.add_preset_with_symbol(custom("z"), 'B').unwrap();
    assert_eq!(map.preset_symbol(z), 'B');
    let c = map.add_preset(custom("c"));
    assert_eq!(map.preset_symbol(c), 'C');
    assert_eq!(map.preset_symbol(TilePresetId::MOUNTAIN), 'M');
    // Taken symbols, including the defaults, are skipped.
    for _ in 0..9 {
        map.add_preset(custom("filler"));
    }
    assert_eq!(map.preset_symbol(TilePresetId(16)), 'L');
    let after_l = map.add_preset(custom("after L"));
    assert_eq!(map.preset_symbol(after_l), 'N');
}

#[test]
fn invalid_or_taken_symbols_are_rejected() {
    let mut map = Map::new(2, 1);
    map.add_preset_with_symbol(custom("x"), 'x').unwrap();
    for bad in [' ', '"', '\\', '\n', 'é', '\u{7f}'] {
        let e = map.add_preset_with_symbol(custom("bad"), bad).unwrap_err();
        assert!(e.starts_with("invalid preset symbol"), "{bad:?}: {e}");
    }
    assert_eq!(
        map.add_preset_with_symbol(custom("dup"), '~').unwrap_err(),
        "preset symbol '~' is already used by preset \"Water\""
    );
    assert_eq!(
        map.add_preset_with_symbol(custom("dup"), 'x').unwrap_err(),
        "preset symbol 'x' is already used by preset \"x\""
    );
    assert_eq!(map.presets().len(), 6);
}

#[test]
fn symbols_run_out_after_87_custom_presets() {
    let mut map = Map::new(1, 1);
    for _ in 0..87 {
        map.add_preset(custom("p"));
    }
    let mut symbols = map.symbols.clone();
    symbols.sort_unstable();
    symbols.dedup();
    assert_eq!(symbols.len(), 92);
    assert!(map.add_preset_with_symbol(custom("one more"), '!').is_err());
}

#[test]
#[should_panic(expected = "every preset symbol is taken")]
fn add_preset_panics_when_symbols_run_out() {
    let mut map = Map::new(1, 1);
    for _ in 0..88 {
        map.add_preset(custom("p"));
    }
}

/// A named change to one field.
type Mutation = (&'static str, fn(&mut Map));

#[test]
fn hash_covers_every_field() {
    let mut base = Map::from_ascii("..r\nc~M\n\n0120\n0000\n0000").unwrap();
    base.add_preset(custom("custom"));
    let mutations: Vec<Mutation> = vec![
        ("width", |m| m.width += 1),
        ("height", |m| m.height += 1),
        ("preset name", |m| m.presets[1].name.push('!')),
        ("buildable", |m| m.presets[5].buildable ^= true),
        ("traversible", |m| m.presets[5].traversible ^= true),
        ("rugged", |m| m.presets[5].rugged ^= true),
        ("drillable", |m| m.presets[5].drillable ^= true),
        ("recruitable", |m| m.presets[5].recruitable ^= true),
        ("look", |m| m.presets[5].look = TileLook::Water),
        ("symbol", |m| m.symbols[5] = 'Z'),
        ("added preset", |m| {
            m.add_preset(custom("another"));
        }),
        ("tile preset", |m| {
            m.set_tile(TilePos::new(0, 0), Tile::new(TilePresetId(5)))
        }),
        ("corner height", |m| m.set_corner(CornerPos::new(3, 2), 1)),
    ];
    for (name, mutate) in mutations {
        let mut m = base.clone();
        mutate(&mut m);
        assert_ne!(m, base, "{name}: the mutation changed nothing");
        assert_ne!(hash(&m), hash(&base), "{name}");
    }
    assert_eq!(hash(&base.clone()), hash(&base));
}

fn custom_preset_map() -> Map {
    let mut map = Map::from_ascii("..r.\n~~..\n\n00011\n00000\n00000").unwrap();
    let ash = map
        .add_preset_with_symbol(
            TilePreset {
                name: "Ash Field".into(),
                buildable: true,
                traversible: true,
                rugged: false,
                drillable: false,
                recruitable: true,
                look: TileLook::Plane,
            },
            '#',
        )
        .unwrap();
    let ice = map.add_preset(custom("Ice"));
    map.set_tile(TilePos::new(2, 0), Tile::new(ash));
    map.set_tile(TilePos::new(3, 1), Tile::new(ice));
    map.set_corner(CornerPos::new(4, 2), 16);
    map
}

#[test]
fn serde_form_is_rows_and_custom_presets() {
    let json = serde_json::to_value(custom_preset_map()).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "custom_presets": [
                {
                    "symbol": "#", "name": "Ash Field", "buildable": true, "traversible": true,
                    "rugged": false, "drillable": false, "recruitable": true, "look": "Plane"
                },
                {
                    "symbol": "A", "name": "Ice", "buildable": true, "traversible": false,
                    "rugged": true, "drillable": false, "recruitable": true, "look": "Rugged"
                }
            ],
            "tiles": ["..#.", "~~.A"],
            "heights": ["00011", "00000", "0000g"]
        })
    );
}

#[test]
fn serde_round_trips_custom_presets_and_symbols() {
    let map = custom_preset_map();
    let json = serde_json::to_string_pretty(&map).unwrap();
    let back: Map = serde_json::from_str(&json).unwrap();
    assert_eq!(back, map);
    assert_eq!(back.preset_symbol(TilePresetId(5)), '#');
    assert_eq!(serde_json::to_string_pretty(&back).unwrap(), json);
}

#[test]
fn serde_accepts_omitted_presets_and_heights() {
    let map: Map = serde_json::from_str(r#"{ "tiles": [".~", "M."] }"#).unwrap();
    assert_eq!(map, Map::from_ascii(".~\nM.").unwrap());
}

#[test]
fn serde_rejects_bad_maps_with_a_message() {
    let cases = [
        (r#"{ "tiles": [] }"#, "the map has no tile rows"),
        (
            r#"{ "tiles": ["..", "."] }"#,
            "ragged tile rows: row 1 has 1 tiles, row 0 has 2",
        ),
        (r#"{ "tiles": ["A."] }"#, "unknown tile symbol 'A' at tile (0, 0)"),
        (
            r#"{ "tiles": [".."], "heights": ["000"] }"#,
            "the height block has 1 rows, expected 2",
        ),
        (
            r#"{ "tiles": [".."], "heights": ["00", "00"] }"#,
            "height row 0 has 2 digits, expected 3",
        ),
        (
            r#"{ "tiles": [".."], "heights": ["0z0", "000"] }"#,
            "invalid height digit 'z'",
        ),
        (
            r#"{ "tiles": [".."], "elevation": ["00"] }"#,
            "unknown field `elevation`",
        ),
        (r#"{ "heights": ["000"] }"#, "missing field `tiles`"),
        (
            r#"{ "custom_presets": [{ "symbol": "~", "name": "Lava", "buildable": false, "traversible": false,
                 "rugged": false, "drillable": false, "recruitable": false, "look": "Water" }], "tiles": [".."] }"#,
            "custom preset \"Lava\": preset symbol '~' is already used by preset \"Water\"",
        ),
        (
            r#"{ "custom_presets": [{ "symbol": " ", "name": "Gap", "buildable": false, "traversible": false,
                 "rugged": false, "drillable": false, "recruitable": false, "look": "Cliff" }], "tiles": [".."] }"#,
            "custom preset \"Gap\": invalid preset symbol ' '",
        ),
    ];
    for (json, expected) in cases {
        let e = serde_json::from_str::<Map>(json).unwrap_err().to_string();
        assert!(e.contains(expected), "{json}\n  gave: {e}\n  expected: {expected}");
    }
}

/// A file claiming a huge map is rejected before anything is allocated for it: here 100 000 × 100 000 tiles,
/// which would need 20 GB.
#[test]
fn loading_rejects_maps_over_the_largest_side_before_allocating() {
    let huge = 100_000;
    let wide_row = ".".repeat(huge);
    let rows: Vec<&str> = std::iter::once(wide_row.as_str())
        .chain(std::iter::repeat_n(".", huge - 1))
        .collect();
    let json = serde_json::json!({ "tiles": rows }).to_string();
    let e = serde_json::from_str::<Map>(&json).unwrap_err().to_string();
    assert!(
        e.contains("the map is too big: 100000 x 100000 tiles (at most 256 x 256)"),
        "{e}"
    );
    // Each side on its own.
    let side = MAX_MAP_SIDE as usize;
    assert!(Map::from_ascii(&".".repeat(side + 1)).unwrap_err().contains("too big"));
    assert!(
        Map::from_ascii(&".\n".repeat(side + 1))
            .unwrap_err()
            .contains("too big")
    );
    let largest = Map::from_ascii(&format!("{}\n", ".".repeat(side)).repeat(side)).unwrap();
    assert_eq!((largest.width(), largest.height()), (256, 256));
}
