//! The reference layouts of the Cults' economy check (`factions/cults/economy.md > Economy check`, design changes 0346
//! and 0385; "**Distance** is measured **edge to edge**, patch to Storage, in whole tiles"). "The reference layout is
//! only a benchmark" (design change 0108): nothing in the game assumes them.
//!
//! **The reference field** (slice M5-S3, design change 0385, answering Q0144 in part): "**8 patches in a contiguous
//! line**, mined from **both long sides**. It should seat about **32 Recruits** at full efficiency (8 tiles × 2 sides ×
//! about 2 per tile). Storages stand along both long sides only: **none at the ends**". The patches are on tiles
//! (16, 12) to (23, 12), objects 1 to 8 from the west ([`LINE`]). Each long side has three Storage sites ([`SITES`]),
//! side by side, each a 1-tile gap from the line (`factions/cults/structures/storage.md`: "The best layout leaves a
//! 1-tile gap between patch and Storage"): W, unturned, its long side facing patches 1 to 3; M facing 4 to 6; and E,
//! turned a quarter, its short side facing 7 and 8 (design change 0380: "two 3-tile long faces and two 2-tile short
//! faces"). The three cover the line's 8 tiles exactly, so that each tile of each side has a Storage across its gap,
//! and nothing stands past the line's ends: the literal reading of "8 tiles × 2 sides" and "none at the ends" (the
//! layout's distances are not in the design: slice M5-S3's choice, reported). The **near side** is the south, where the
//! Cults start ([`line_field`]): the starting Hall, turned to face the line, and the starting Storage on near M. The
//! **far side** is the north, which the Recruits reach round the line's ends.
//!
//! [`face_field`] is one face of a Storage, for the check's per-Recruit and per-face figures; [`full_line`] the field
//! with every Storage standing, for the seats it holds.
//!
//! Before design change 0385 (slices M5-L and M5-S2a) the field was mined from one side only, its patches in faces a
//! free column apart, one Storage per face (`docs/economy_oracle.md > The Cults > Before design change 0385`).

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
/// The reference field's patches, objects 1 to 8 from the west, on tiles (16, 12) to (23, 12).
pub const LINE: [ObjectId; 8] = [
    ObjectId(1),
    ObjectId(2),
    ObjectId(3),
    ObjectId(4),
    ObjectId(5),
    ObjectId(6),
    ObjectId(7),
    ObjectId(8),
];
/// The tile row of [`LINE`], and its westernmost column.
const ROW: i32 = 12;
const WEST: i32 = 16;

/// A long side of the line (module docs).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LineSide {
    /// The south, the Cults' start's.
    Near,
    /// The north.
    Far,
}

impl LineSide {
    /// From a patch's centre to the middle of the 1-tile gap between it and this side's Storages: the lane a Recruit
    /// mines and delivers in.
    pub fn lane(self) -> Vec2 {
        match self {
            LineSide::Near => Vec2::new(0.0, SU_PER_GU),
            LineSide::Far => Vec2::new(0.0, -SU_PER_GU),
        }
    }
}

/// A Storage site of the reference field ([`SITES`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StorageSite {
    /// The side of the line it stands on.
    pub side: LineSide,
    /// The top-left tile of its footprint.
    pub origin: TilePos,
    /// Turned a quarter, its short side facing the line.
    pub turned: bool,
    /// The patches across the gap from it, those nearest the starting Storage first.
    pub patches: &'static [ObjectId],
}

impl StorageSite {
    /// How it is turned.
    pub fn orientation(&self) -> BuildingOrientation {
        BuildingOrientation {
            rotation: if self.turned { Rotation::R90 } else { Rotation::R0 },
            flip_h: false,
            flip_v: false,
        }
    }
}

/// The reference field's Storage sites (module docs), in the order the Cults' lines build them, the starting Storage's
/// first: the near side (M, W, E), then the far side (M, W, E). On the near side unturned W covers tiles (16, 14) to
/// (18, 15), M (19, 14) to (21, 15), and E, turned, (22, 14) to (23, 16); on the far side W (16, 9) to (18, 10), M
/// (19, 9) to (21, 10), and E (22, 8) to (23, 10). Each site's patches are listed nearest the starting Storage first.
pub const SITES: [StorageSite; 6] = {
    const W: &[ObjectId] = &[ObjectId(3), ObjectId(2), ObjectId(1)];
    const M: &[ObjectId] = &[ObjectId(4), ObjectId(5), ObjectId(6)];
    const E: &[ObjectId] = &[ObjectId(7), ObjectId(8)];
    [
        site(LineSide::Near, WEST + 3, ROW + 2, false, M),
        site(LineSide::Near, WEST, ROW + 2, false, W),
        site(LineSide::Near, WEST + 6, ROW + 2, true, E),
        site(LineSide::Far, WEST + 3, ROW - 3, false, M),
        site(LineSide::Far, WEST, ROW - 3, false, W),
        site(LineSide::Far, WEST + 6, ROW - 4, true, E),
    ]
};

/// A site of [`SITES`].
const fn site(side: LineSide, x: i32, y: i32, turned: bool, patches: &'static [ObjectId]) -> StorageSite {
    StorageSite {
        side,
        origin: TilePos::new(x, y),
        turned,
        patches,
    }
}

/// The starting Storage's site in [`SITES`]: near M, under the middle of the line, as `dev_skirmish`'s Storage faces
/// the middle of its row.
pub const START_SITE: usize = 0;
/// The starting Recruitment Hall's top-left tile: (18, 18) to (21, 21), centred under the line two free rows south of
/// the near Storages, turned a quarter so that its exit side, B, faces north, toward the line. Its Recruitment Area,
/// (15, 15) to (24, 24), is all on the map.
const HALL: TilePos = TilePos::new(18, 18);
const HALL_TURNED: BuildingOrientation = BuildingOrientation {
    rotation: Rotation::R90,
    flip_h: false,
    flip_v: false,
};
/// The Two Halls scenario's second Hall (design change 0385: built "by the Recruits on the patches farthest from the
/// starting Storage"; its site is not in the design: slice M5-S3's choice, reported): on tiles (8, 10) to (11, 13),
/// across four free columns from the line's west end, by its patches farthest from the starting Storage, turned
/// half round so that its exit side, B, faces east, toward the line, between its two sides. Its Recruitment Area, (5, 7)
/// to (14, 16), is all on the map and clear of the starting Hall's, so it claims all 100 tiles: the nearest site west of
/// the line with the whole area its own.
pub const SECOND_HALL: TilePos = TilePos::new(8, 10);
/// How [`SECOND_HALL`] is turned.
pub const SECOND_HALL_TURNED: BuildingOrientation = BuildingOrientation {
    rotation: Rotation::R180,
    flip_h: false,
    flip_v: false,
};
/// The map: 32 × 28 tiles of flat Plane.
const WIDTH: usize = 32;
const HEIGHT: usize = 28;

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

/// The line's patches, ids 1 to 8 ([`LINE`]).
fn field() -> ScenarioBuilder {
    let mut builder = flat(WIDTH, HEIGHT);
    for x in 0..LINE.len() as i32 {
        builder = builder.spawn_with(patch(), tile(TilePos::new(WEST + x, ROW)));
    }
    builder
}

/// The Cults' start on the reference field (`factions/cults/economy.md > Starting Conditions`: "One pre-built
/// **Recruitment Hall**", "One pre-built **Storage**", "Six **Recruits**, all belonging to the starting Recruitment
/// Hall", "**0 Space Crystals**"), as player 1's Starting Layout, which gives the Recruits their Hall
/// (`Scenario::start_objects`): the Hall ([`HALL`]), the Storage on near M ([`START_SITE`]), and six Recruits in two
/// rows of three a grid unit apart, west of the Hall's door between it and near W, at (16.5, 16.5) to (18.5, 17.5) grid
/// units. The patches are ids 1 to 8, then the Hall (9), its Storage (10) and the Recruits (11 to 16).
pub fn line_field() -> Scenario {
    let mut scenario = field().build();
    let laid = |kind, at, orientation| LayoutObject { kind, at, orientation };
    let start = SITES[START_SITE];
    let mut layout = vec![
        laid(ObjectKind::RecruitmentHall, tile(HALL), HALL_TURNED),
        laid(ObjectKind::Storage, tile(start.origin), start.orientation()),
    ];
    for (x, y) in [(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (0.0, 1.0), (1.0, 1.0), (2.0, 1.0)] {
        let at = SpawnAt::Point(Vec2::new(16.5 + x, 16.5 + y) * SU_PER_GU);
        layout.push(laid(ObjectKind::Recruit, at, BuildingOrientation::default()));
    }
    *scenario.map.slots[0].layouts.get_mut(Faction::Cults) = layout;
    scenario
}

/// The reference field with every Storage of [`SITES`] standing (ids 9 to 14, in its order) and `near` + `far`
/// Recruits that no Hall made, `near` (ids 15 on) south of the near Storages from (15.5, 18.5) grid units and `far`
/// (the ids after) north of the far Storages from (15.5, 6.5), in rows of ten half a grid unit apart: the field at a
/// number of Recruits a side, with no Hall to add more.
pub fn full_line(near: u32, far: u32) -> Scenario {
    let mut builder = field();
    for site in SITES {
        builder = builder.spawn(ObjectKind::Storage, Some(FIELD_PLAYER), tile(site.origin));
    }
    for (first, n, rows) in [(18.5, near, 0.5), (6.5, far, -0.5)] {
        for i in 0..n {
            let at = Vec2::new(15.5 + (i % 10) as f32, first + rows * (i / 10) as f32) * SU_PER_GU;
            builder = builder.spawn(ObjectKind::Recruit, Some(FIELD_PLAYER), SpawnAt::Point(at));
        }
    }
    let mut scenario = builder.build();
    for (i, site) in SITES.iter().enumerate() {
        scenario.map.spawns[LINE.len() + i].orientation = site.orientation();
    }
    scenario
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
