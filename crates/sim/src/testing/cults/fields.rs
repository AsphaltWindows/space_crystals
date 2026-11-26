//! The reference layouts of the Cults' economy check (`factions/cults/economy.md > Economy check`, design change 0346,
//! answering Q0103: "Two scenarios, on an 8-patch field"; "**Distance** is measured **edge to edge**, patch to
//! Storage, in whole tiles"; "**one Storage per face** for the ideal comparison"). "The reference layout is only a
//! benchmark" (design change 0108): nothing in the game assumes them.
//!
//! **The field**: 8 patches in four pairs, A to D from the west, a free column between pairs, each pair a 2-tile face
//! with its own Storage a 1-tile gap south of it (`factions/cults/structures/storage.md`: "The best layout leaves a
//! 1-tile gap between patch and Storage; each 2-tile face supports roughly 4–5 Recruits"; "3–4 for an 8-patch
//! field"). The free columns keep the faces apart: on 8 adjacent patches a patch at the end of one face lies a tile
//! from the next face's Storage too, edge to edge, as near as its own, and the Recruit takes the Storage of the lower
//! id among equals (`orders::gather::deliver`), so that faces share their loads and crowd one another
//! (`docs/economy_oracle.md > The Cults` has the measurement).

use glam::Vec2;

use crate::defs::ObjectKind;
use crate::ids::{ObjectId, PlayerId};
use crate::object::Rotation;
use crate::player::Faction;
use crate::scenario::{BuildingOrientation, LayoutObject, Scenario, SpawnAt, SpawnSettings};
use crate::testing::ScenarioBuilder;
use crate::units::{SU_PER_GU, TilePos};

/// The player of the reference layouts.
pub const FIELD_PLAYER: PlayerId = PlayerId(1);
/// The patches hold so much that none runs out.
pub const FIELD_RESERVE: u32 = 1_000_000;
/// [`reference_field`]'s and [`crowd_field`]'s patches, objects 1 to 8, by pair from the west: A on tiles (10, 8) and
/// (11, 8), B on (13, 8) and (14, 8), C on (16, 8) and (17, 8), D on (19, 8) and (20, 8).
pub const FIELD_PAIRS: [[ObjectId; 2]; 4] = [
    [ObjectId(1), ObjectId(2)],
    [ObjectId(3), ObjectId(4)],
    [ObjectId(5), ObjectId(6)],
    [ObjectId(7), ObjectId(8)],
];
/// Each pair's Storage, the top-left tile of its 2×2 footprint: right under the pair, a 1-tile gap south of it
/// (tiles (x, 10) to (x + 1, 11)).
pub const FIELD_STORAGES: [TilePos; 4] = [
    TilePos::new(10, 10),
    TilePos::new(13, 10),
    TilePos::new(16, 10),
    TilePos::new(19, 10),
];
/// The pair the starting Storage faces: B, west of the field's middle, as `dev_skirmish`'s Storage stands west of
/// its row's middle.
pub const START_PAIR: usize = 1;
/// The starting Recruitment Hall's top-left tile: (13, 14) to (16, 17), turned 90° so that its exit side, B, faces
/// north, toward the field; its door, the middle of that side at (15, 14) grid units, lines up with the free column
/// between pairs B and C. Its Recruitment Area, (10, 11) to (19, 20), is all on the map.
const HALL: TilePos = TilePos::new(13, 14);
const HALL_TURNED: BuildingOrientation = BuildingOrientation {
    rotation: Rotation::R90,
    flip_h: false,
    flip_v: false,
};
/// Scenario 2's second Hall (design change 0346: "Where the second Hall goes [...] is your choice"): on tiles (24, 7)
/// to (27, 10), unturned, east of pair D across three free columns, its exit side, B, facing west toward the field.
/// Its Recruitment Area, (21, 4) to (30, 13), is all on the map and clear of the starting Hall's, so it claims all
/// 100 tiles: the nearest site to the field with the whole area its own, by the far pairs C and D that its builders
/// gather.
pub const SECOND_HALL: TilePos = TilePos::new(24, 7);
/// The map: 34 × 24 tiles of flat Plane.
const WIDTH: usize = 34;
const HEIGHT: usize = 24;

fn flat(w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&vec![".".repeat(w); h].join("\n")).faction(FIELD_PLAYER, Faction::Cults)
}

fn tile(at: TilePos) -> SpawnAt {
    SpawnAt::Tile(at)
}

fn patch() -> SpawnSettings {
    SpawnSettings::CrystalPatch {
        starting_amount: FIELD_RESERVE,
    }
}

/// The field's patches, ids 1 to 8 (`FIELD_PAIRS`).
fn field() -> ScenarioBuilder {
    let mut builder = flat(WIDTH, HEIGHT);
    for x in [10, 11, 13, 14, 16, 17, 19, 20] {
        builder = builder.spawn_with(patch(), tile(TilePos::new(x, 8)));
    }
    builder
}

/// The Cults' start on the field (`factions/cults/economy.md > Starting Conditions`: "One pre-built **Recruitment
/// Hall**", "One pre-built **Storage**", "Six **Recruits**, all belonging to the starting Recruitment Hall", "**0
/// Space Crystals**"), as player 1's Starting Layout, which gives the Recruits their Hall (`Scenario::start_objects`):
/// the Hall ([`HALL`]), pair B's Storage, and six Recruits in two rows of three a grid unit apart, north-east of the
/// Hall between it and pairs C and D's Storages, at (17.5, 12.5) to (19.5, 13.5) grid units. Besides it, the Storages
/// of the pairs `faces` names (indices into [`FIELD_STORAGES`]; B's is the start's), pre-placed: "one Storage per
/// face for the ideal comparison" (design change 0346). The patches are ids 1 to 8, the pre-placed Storages next,
/// then the Hall, its Storage and the Recruits.
pub fn reference_field(faces: &[usize]) -> Scenario {
    let mut builder = field();
    for &pair in faces.iter().filter(|&&pair| pair != START_PAIR) {
        builder = builder.spawn(ObjectKind::Storage, Some(FIELD_PLAYER), tile(FIELD_STORAGES[pair]));
    }
    let mut scenario = builder.build();
    let laid = |kind, at, orientation| LayoutObject { kind, at, orientation };
    let mut layout = vec![
        laid(ObjectKind::RecruitmentHall, tile(HALL), HALL_TURNED),
        laid(
            ObjectKind::Storage,
            tile(FIELD_STORAGES[START_PAIR]),
            BuildingOrientation::default(),
        ),
    ];
    for (x, y) in [(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (0.0, 1.0), (1.0, 1.0), (2.0, 1.0)] {
        let at = SpawnAt::Point(Vec2::new(17.5 + x, 12.5 + y) * SU_PER_GU);
        layout.push(laid(ObjectKind::Recruit, at, BuildingOrientation::default()));
    }
    *scenario.map.slots[0].layouts.get_mut(Faction::Cults) = layout;
    scenario
}

/// The field with every pair's Storage (ids 9 to 12, from the west) and `recruits` Recruits that no Hall made (ids 13
/// on), in rows of ten south of the Storages from (10.5, 13.5) grid units, half a grid unit apart: the faces at a
/// number of Recruits, with no Hall to add more.
pub fn crowd_field(recruits: u32) -> Scenario {
    let mut builder = field();
    for at in FIELD_STORAGES {
        builder = builder.spawn(ObjectKind::Storage, Some(FIELD_PLAYER), tile(at));
    }
    for i in 0..recruits {
        let at = Vec2::new(10.5 + (i % 10) as f32, 13.5 + 0.5 * (i / 10) as f32) * SU_PER_GU;
        builder = builder.spawn(ObjectKind::Recruit, Some(FIELD_PLAYER), SpawnAt::Point(at));
    }
    builder.build()
}

/// One 2-tile face: two patches (ids 1 and 2) on tiles (8, 4) and (9, 4), their Storage (id 3) `gap` tiles south of
/// them, on (8, 5 + `gap`) to (9, 6 + `gap`), and `recruits` Recruits that no Hall made (ids 4 on) in a row farther
/// south, from (6.5, 12.5) grid units, a grid unit apart, on a flat 20 × 16 map.
pub fn face_field(gap: i32, recruits: u32) -> Scenario {
    let mut builder = flat(20, 16)
        .spawn_with(patch(), tile(TilePos::new(8, 4)))
        .spawn_with(patch(), tile(TilePos::new(9, 4)))
        .spawn(ObjectKind::Storage, Some(FIELD_PLAYER), tile(TilePos::new(8, 5 + gap)));
    for i in 0..recruits {
        let at = Vec2::new(6.5 + i as f32, 12.5) * SU_PER_GU;
        builder = builder.spawn(ObjectKind::Recruit, Some(FIELD_PLAYER), SpawnAt::Point(at));
    }
    builder.build()
}
