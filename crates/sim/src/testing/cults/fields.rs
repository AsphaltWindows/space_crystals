//! The reference layouts of the Cults' economy check (`factions/cults/economy.md > Economy check`, design change 0346,
//! answering Q0103: "Two scenarios, on an 8-patch field"; "**Distance** is measured **edge to edge**, patch to
//! Storage, in whole tiles"; "**one Storage per face** for the ideal comparison"). "The reference layout is only a
//! benchmark" (design change 0108): nothing in the game assumes them.
//!
//! **The field** (slice M5-S2a, for design change 0380's 3x2 Storage): 8 patches in a row in three faces, A to C from
//! the west, a free column between faces. A and B are 3 patches each and C 2. Each face has its own Storage, unturned,
//! its long north side a 1-tile gap south of the face (`factions/cults/structures/storage.md`: "two 3-tile long faces
//! and two 2-tile short faces, so it serves more Recruits per face and an efficient field needs fewer Storages"; "The
//! best layout leaves a 1-tile gap between patch and Storage"). C's Storage reaches a tile past the east end of its
//! two patches. The free columns keep the faces apart and leave the Recruits a way between the Storages to the lane
//! in front of the patches. Slice M5-L's field, for the 2x2 Storage, was four pairs a free column apart, each pair a
//! 2-tile face with its own 2x2 Storage. [`adjacent_field`] is the field without the free columns
//! (`docs/economy_oracle.md > The Cults` has both measured): since design change 0375 a Recruit takes its load to the
//! Storage nearest to itself (`orders::gather::deliver`), which keeps a Recruit that mines in front of its own face's
//! Storage with it.

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
/// [`reference_field`]'s and [`crowd_field`]'s patches, objects 1 to 8, by face from the west: A on tiles (10, 8) to
/// (12, 8), B on (14, 8) to (16, 8), C on (18, 8) and (19, 8).
pub const FIELD_FACES: [&[ObjectId]; 3] = [
    &[ObjectId(1), ObjectId(2), ObjectId(3)],
    &[ObjectId(4), ObjectId(5), ObjectId(6)],
    &[ObjectId(7), ObjectId(8)],
];
/// Each face's Storage, the top-left tile of its 3×2 footprint, unturned: right under the face's west end, a 1-tile
/// gap south of it (tiles (x, 10) to (x + 2, 11)).
pub const FIELD_STORAGES: [TilePos; 3] = [TilePos::new(10, 10), TilePos::new(14, 10), TilePos::new(18, 10)];
/// The face the starting Storage faces: B, the middle one, as `dev_skirmish`'s Storage faces the middle of its row.
pub const START_FACE: usize = 1;
/// The starting Recruitment Hall's top-left tile: (15, 14) to (18, 17), turned 90° so that its exit side, B, faces
/// north, toward the field; its door, the middle of that side at (17, 14) grid units, lines up with the free column
/// between faces B and C. Its Recruitment Area, (12, 11) to (21, 20), is all on the map.
const HALL: TilePos = TilePos::new(15, 14);
const HALL_TURNED: BuildingOrientation = BuildingOrientation {
    rotation: Rotation::R90,
    flip_h: false,
    flip_v: false,
};
/// Scenario 2's second Hall (design change 0346: "Where the second Hall goes [...] is your choice"): on tiles (25, 7)
/// to (28, 10), unturned, east of face C's Storage across four free columns, its exit side, B, facing west toward the
/// field. Its Recruitment Area, (22, 4) to (31, 13), is all on the map and clear of the starting Hall's, so it claims
/// all 100 tiles: the nearest site east of the field with the whole area its own, by the far face C that some of its
/// builders gather.
pub const SECOND_HALL: TilePos = TilePos::new(25, 7);
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

/// The field's patches, ids 1 to 8 ([`FIELD_FACES`]).
fn field() -> ScenarioBuilder {
    let mut builder = flat(WIDTH, HEIGHT);
    for x in [10, 11, 12, 14, 15, 16, 18, 19] {
        builder = builder.spawn_with(patch(), tile(TilePos::new(x, 8)));
    }
    builder
}

/// The Cults' start on the field (`factions/cults/economy.md > Starting Conditions`: "One pre-built **Recruitment
/// Hall**", "One pre-built **Storage**", "Six **Recruits**, all belonging to the starting Recruitment Hall", "**0
/// Space Crystals**"), as player 1's Starting Layout, which gives the Recruits their Hall (`Scenario::start_objects`):
/// the Hall ([`HALL`]), face B's Storage, and six Recruits in two rows of three a grid unit apart, north-east of the
/// Hall between it and face C's Storage, at (17.5, 12.5) to (19.5, 13.5) grid units. Besides it, the Storages of the
/// faces `faces` names (indices into [`FIELD_STORAGES`]; B's is the start's), pre-placed: "one Storage per face for
/// the ideal comparison" (design change 0346). The patches are ids 1 to 8, the pre-placed Storages next, then the
/// Hall, its Storage and the Recruits.
pub fn reference_field(faces: &[usize]) -> Scenario {
    let mut builder = field();
    for &face in faces.iter().filter(|&&face| face != START_FACE) {
        builder = builder.spawn(ObjectKind::Storage, Some(FIELD_PLAYER), tile(FIELD_STORAGES[face]));
    }
    let mut scenario = builder.build();
    let laid = |kind, at, orientation| LayoutObject { kind, at, orientation };
    let mut layout = vec![
        laid(ObjectKind::RecruitmentHall, tile(HALL), HALL_TURNED),
        laid(
            ObjectKind::Storage,
            tile(FIELD_STORAGES[START_FACE]),
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

/// The field with every face's Storage (ids 9 to 11, from the west) and `recruits` Recruits that no Hall made (ids 12
/// on), in rows of ten south of the Storages from (10.5, 13.5) grid units, half a grid unit apart: the faces at a
/// number of Recruits, with no Hall to add more.
pub fn crowd_field(recruits: u32) -> Scenario {
    let mut builder = field();
    for at in FIELD_STORAGES {
        builder = builder.spawn(ObjectKind::Storage, Some(FIELD_PLAYER), tile(at));
    }
    crowd(builder, recruits).build()
}

/// The layout the free columns are measured against (module docs; `docs/economy_oracle.md > The Cults`): the 8
/// patches side by side on tiles (10, 8) to (17, 8), ids 1 to 8, in faces A to C from the west as [`FIELD_FACES`]
/// numbers them; each face's Storage (ids 9 to 11) unturned a 1-tile gap south of it, on tiles (x, 10) to (x + 2, 11)
/// for x = 10, 13 and 16 (C's reaching a tile past the row), or with `alternate` B's a 1-tile gap north of its face,
/// on (13, 5) to (15, 6); and `recruits` Recruits that no Hall made (ids 12 on), where [`crowd_field`] has them.
pub fn adjacent_field(recruits: u32, alternate: bool) -> Scenario {
    let mut builder = flat(WIDTH, HEIGHT);
    for x in 10..18 {
        builder = builder.spawn_with(patch(), tile(TilePos::new(x, 8)));
    }
    for (face, x) in [10, 13, 16].into_iter().enumerate() {
        let y = if alternate && face == 1 { 5 } else { 10 };
        builder = builder.spawn(ObjectKind::Storage, Some(FIELD_PLAYER), tile(TilePos::new(x, y)));
    }
    crowd(builder, recruits).build()
}

/// `recruits` Recruits that no Hall made, in rows of ten south of the field's Storages from (10.5, 13.5) grid units,
/// half a grid unit apart.
fn crowd(mut builder: ScenarioBuilder, recruits: u32) -> ScenarioBuilder {
    for i in 0..recruits {
        let at = Vec2::new(10.5 + (i % 10) as f32, 13.5 + 0.5 * (i / 10) as f32) * SU_PER_GU;
        builder = builder.spawn(ObjectKind::Recruit, Some(FIELD_PLAYER), SpawnAt::Point(at));
    }
    builder
}

/// Which side of a 3x2 Storage [`face_field`]'s patches face (design change 0380: "two 3-tile long faces and two
/// 2-tile short faces").
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Side {
    /// A long side: three patches.
    Long,
    /// A short side: two patches, the Storage turned a quarter.
    Short,
}

impl Side {
    /// How many patches face it: as many as it is tiles long.
    pub const fn patches(self) -> u32 {
        match self {
            Side::Long => 3,
            Side::Short => 2,
        }
    }
}

/// One face: a patch per tile of one of a Storage's sides (ids 1 on) in a row on tiles (8, 4) eastwards; their Storage
/// (the id after) `gap` tiles south of them, unturned on (8, 5 + `gap`) to (10, 6 + `gap`) for a long side, turned a
/// quarter on (8, 5 + `gap`) to (9, 7 + `gap`) for a short one; and `recruits` Recruits that no Hall made (the ids
/// after) in a row farther south, from (6.5, 12.5) grid units, a grid unit apart, on a flat 20 × 16 map.
pub fn face_field(side: Side, gap: i32, recruits: u32) -> Scenario {
    let mut builder = flat(20, 16);
    for x in 0..side.patches() as i32 {
        builder = builder.spawn_with(patch(), tile(TilePos::new(8 + x, 4)));
    }
    builder = builder.spawn(ObjectKind::Storage, Some(FIELD_PLAYER), tile(TilePos::new(8, 5 + gap)));
    for i in 0..recruits {
        let at = Vec2::new(6.5 + i as f32, 12.5) * SU_PER_GU;
        builder = builder.spawn(ObjectKind::Recruit, Some(FIELD_PLAYER), SpawnAt::Point(at));
    }
    let mut scenario = builder.build();
    if side == Side::Short {
        let storage = &mut scenario.map.spawns[side.patches() as usize];
        storage.orientation.rotation = Rotation::R90;
    }
    scenario
}
