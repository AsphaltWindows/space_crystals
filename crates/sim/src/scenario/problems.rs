//! What makes a map or a game setup unplayable, as listed by validation, with a message for each.

use std::fmt;

use glam::Vec2;

use super::create::MAX_SLOTS;
use super::generate::Asymmetry;
use super::settings::SettingsProblem;
use super::starting::StartingConditions;
use crate::defs::ObjectKind;
use crate::ids::{PlayerId, SlotId};
use crate::placement::{PlacementCheck, TileProblem, WholeProblem};
use crate::player::Faction;
use crate::units::TilePos;

/// Something that makes a map or a game setup unplayable. [`MapDef::validate`] and [`Scenario::validate`] list them
/// all. `Display` gives a message for the editor or a log.
#[derive(Clone, Debug, PartialEq)]
pub enum ScenarioProblem {
    /// A side of the map is longer than [`crate::map::MAX_MAP_SIDE`] tiles.
    MapTooBig { width: u32, height: u32 },
    /// `core/map.md > Elevation > Maximum slope`: this Traversible tile's corners differ by `slope` levels, more
    /// than [`crate::map::MAX_TRAVERSIBLE_SLOPE`].
    TooSteep { tile: TilePos, slope: u8 },
    /// `core/map_editor.md > Map Types > Skirmish Default`: a symmetry is required.
    SkirmishWithoutSymmetry,
    /// `core/map_editor.md > Symmetry`: 90° rotation needs a square map.
    SymmetryNeedsSquare { width: u32, height: u32 },
    /// `core/map_editor.md > Symmetry > Editing a symmetric map`: the locked part isn't generated from the editable
    /// part (a file edited by hand). The next edit in the editor regenerates it.
    NotSymmetric(Asymmetry),
    /// A map needs at least one player slot.
    NoSlots,
    /// More than [`MAX_SLOTS`] slots.
    TooManySlots(usize),
    /// The slots don't come in whole sets of images (placeholder rule, `scenario::generate`).
    SlotsNotInImageSets { slots: usize, images: usize },
    /// The start position lies off the map.
    StartOffMap { slot: SlotId, tile: TilePos },
    /// The camera start is not a finite point on the map.
    CameraStartOffMap { slot: SlotId, pos: Vec2 },
    /// The spawn at this index of `MapDef::spawns` can't be created.
    InvalidSpawn {
        spawn: usize,
        kind: ObjectKind,
        problem: SpawnProblem,
    },
    /// The tile footprints of the spawns at these indices share `tile`, and only a plate and its patch may
    /// (`ObjectKind::may_overlap`).
    OverlappingFootprints { first: usize, second: usize, tile: TilePos },
    /// Object `index` of a slot's Starting Layout for `faction` can't be created.
    InvalidLayoutObject {
        slot: SlotId,
        faction: Faction,
        index: usize,
        kind: ObjectKind,
        problem: SpawnProblem,
    },
    /// Object `index` of the Starting Layout a player gets overlaps a pre-placed object or another starting object
    /// (only a plate may overlap, the patch it is placed onto: `ObjectKind::may_overlap`).
    LayoutOverlap {
        slot: SlotId,
        faction: Faction,
        index: usize,
        tile: TilePos,
    },
    /// `core/map_editor.md > Starting Layouts > Layout validation` (design change 0097): structure `index` of a slot's
    /// Starting Layout for `faction` breaks the faction's placement rules, minus sight, among the map's pre-placed
    /// objects and the rest of its layout ([`crate::placement::check_layout`]); `check` says how.
    LayoutPlacement {
        slot: SlotId,
        faction: Faction,
        index: usize,
        kind: ObjectKind,
        check: PlacementCheck,
    },
    /// `> Skirmish Default restriction` (design changes 0009, 0012): on a Skirmish Default map, object `index` of a
    /// slot's layout for `faction` is beyond the faction's Starting Conditions, which hold `allowed` objects of its
    /// kind: 0 for a kind they don't hold, and for a faction whose Starting Conditions aren't implemented yet (plan
    /// decision T11). Of more objects of a kind than allowed, the later ones are reported.
    LayoutExtra {
        slot: SlotId,
        faction: Faction,
        index: usize,
        kind: ObjectKind,
        allowed: u32,
    },
    /// `> Skirmish Default restriction`: on a Skirmish Default map, a slot's layout for `faction` lacks `missing`
    /// objects of `kind` of the faction's Starting Conditions, and the map can't be played until it is complete.
    /// Only implemented factions' layouts are checked (plan decision T11).
    LayoutMissing {
        slot: SlotId,
        faction: Faction,
        kind: ObjectKind,
        missing: u32,
    },
    /// A game needs at least one player.
    NoPlayers,
    /// The player takes a slot the map doesn't have.
    UnknownSlot { player: PlayerId, slot: SlotId },
    /// The player takes a slot that an earlier player took.
    SlotTaken { player: PlayerId, slot: SlotId },
}

/// Why an object can't be created as requested, by a map, a Starting Layout or a debug `Spawn` command.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpawnProblem {
    /// `core/map.md > Resource`: resources are never owned (a Starting Layout's objects all are).
    OwnedResource,
    /// The owner is not a player of the game.
    UnknownOwner(PlayerId),
    /// The owner is not a slot of the map.
    UnknownSlot(SlotId),
    /// `core/map_editor.md > Map Types > Skirmish Default`: no player-owned objects outside the Starting Layouts.
    OwnedOnSkirmishMap,
    /// Tile-based types spawn at a [`SpawnAt::Tile`], units at a [`SpawnAt::Point`].
    WrongLocationType,
    /// Part of the tile footprint lies off the map.
    FootprintOffMap,
    /// The point is not finite or lies off the map.
    PointOffMap,
    /// `core/map.md > Elevation > Structures only on flat terrain`, `> Map resources only on flat terrain`: this
    /// footprint tile is sloped.
    NotFlat(TilePos),
    /// `factions/gdo/economy.md > ExtractionPlate`, "Placed onto a Space Crystal Patch": the pre-placed Extraction
    /// Plate isn't exactly on one, by the placement check's rule ([`crate::placement::Onto`]).
    NotOnPatch,
    /// `core/map_editor.md > Per-object Resource Values`: the spawn has no values, but its kind needs them.
    MissingSettings,
    /// The spawn's values belong to another kind of object.
    WrongSettings,
    /// One of the spawn's values makes no sense.
    InvalidSettings(SettingsProblem),
    /// `docs/movement.md > 3`: the unit's box overlaps ground it can't stand on: impassable terrain, the map's edge
    /// or the blocking footprint of an earlier object.
    Blocked,
    /// `docs/movement.md > 3`: the unit's box overlaps the box of an earlier unit of its layer (ground or
    /// underground; touching is allowed).
    OverlapsUnit,
    /// `docs/movement.md > 3`: the blocking footprint covers part of an earlier ground unit's box.
    CoversUnit,
}

impl fmt::Display for SpawnProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpawnProblem::OwnedResource => write!(f, "resources can't have an owner"),
            SpawnProblem::UnknownOwner(p) => write!(f, "owner {} is not a player", p.0),
            SpawnProblem::UnknownSlot(s) => write!(f, "owner slot {} is not a slot of the map", s.0),
            SpawnProblem::OwnedOnSkirmishMap => {
                write!(
                    f,
                    "Skirmish Default maps have no owned objects outside the Starting Layouts"
                )
            }
            SpawnProblem::WrongLocationType => {
                write!(f, "tile-based objects need a tile location and units a point")
            }
            SpawnProblem::FootprintOffMap => write!(f, "the footprint is not entirely on the map"),
            SpawnProblem::PointOffMap => write!(f, "the point is not finite or off the map"),
            SpawnProblem::NotFlat(t) => write!(f, "it needs flat ground, but its tile ({}, {}) is sloped", t.x, t.y),
            SpawnProblem::NotOnPatch => write!(f, "{}", WholeProblem::NotOnPatch),
            SpawnProblem::MissingSettings => write!(f, "its per-object values are missing"),
            SpawnProblem::WrongSettings => write!(f, "its per-object values belong to another kind of object"),
            SpawnProblem::InvalidSettings(problem) => write!(f, "{problem}"),
            SpawnProblem::Blocked => write!(
                f,
                "its box overlaps impassable ground, the map's edge or a structure or resource"
            ),
            SpawnProblem::OverlapsUnit => write!(f, "its box overlaps another unit's"),
            SpawnProblem::CoversUnit => write!(f, "its footprint covers a unit"),
        }
    }
}

/// What a failed placement check says, short: the footprint's problems, then each tile problem at its first tile
/// with how many more tiles have it ("not buildable ground at tile (3, 4) and 8 more tiles").
fn placement_text(check: &PlacementCheck) -> String {
    let mut parts: Vec<String> = check.whole.iter().map(ToString::to_string).collect();
    for problem in TileProblem::ALL {
        let mut tiles = check.tiles.iter().filter(|(_, p)| *p == problem).map(|(t, _)| t);
        let Some(first) = tiles.next() else {
            continue;
        };
        let more = match tiles.count() {
            0 => String::new(),
            1 => " and 1 more tile".into(),
            n => format!(" and {n} more tiles"),
        };
        parts.push(format!("{problem} at tile ({}, {}){more}", first.x, first.y));
    }
    parts.join("; ")
}

impl fmt::Display for Asymmetry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Asymmetry::Tile(t) => write!(f, "tile ({}, {})", t.x, t.y),
            Asymmetry::Corner(c) => write!(f, "the corner height at ({}, {})", c.x, c.y),
            Asymmetry::Slot(s) => write!(f, "slot {}", s.0),
            Asymmetry::Spawns => write!(f, "the objects"),
        }
    }
}

impl fmt::Display for ScenarioProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScenarioProblem::MapTooBig { width, height } => write!(
                f,
                "the map is {width} x {height} tiles, more than {max} x {max}",
                max = crate::map::MAX_MAP_SIDE
            ),
            ScenarioProblem::TooSteep { tile, slope } => write!(
                f,
                "tile ({}, {}) is Traversible but {slope} levels steep (at most {}): flatten it or make it a cliff",
                tile.x,
                tile.y,
                crate::map::MAX_TRAVERSIBLE_SLOPE
            ),
            ScenarioProblem::SkirmishWithoutSymmetry => write!(f, "a Skirmish Default map needs a symmetry"),
            ScenarioProblem::SymmetryNeedsSquare { width, height } => {
                write!(
                    f,
                    "90 degree rotation needs a square map; this one is {width} x {height}"
                )
            }
            ScenarioProblem::NotSymmetric(what) => write!(
                f,
                "{what} is not generated from the editable part: the next edit regenerates it"
            ),
            ScenarioProblem::NoSlots => write!(f, "the map has no player slots"),
            ScenarioProblem::TooManySlots(n) => write!(f, "{n} player slots, more than {MAX_SLOTS}"),
            ScenarioProblem::SlotsNotInImageSets { slots, images } => write!(
                f,
                "{slots} player slots don't come in whole sets of {images} symmetric images"
            ),
            ScenarioProblem::StartOffMap { slot, tile } => {
                write!(f, "slot {}: start ({}, {}) is off the map", slot.0, tile.x, tile.y)
            }
            ScenarioProblem::CameraStartOffMap { slot, pos } => write!(
                f,
                "slot {}: camera start ({}, {}) is not a finite point on the map",
                slot.0, pos.x, pos.y
            ),
            ScenarioProblem::InvalidSpawn { spawn, kind, problem } => write!(f, "spawn {spawn} ({kind:?}): {problem}"),
            ScenarioProblem::OverlappingFootprints { first, second, tile } => write!(
                f,
                "spawns {first} and {second} overlap at tile ({}, {})",
                tile.x, tile.y
            ),
            ScenarioProblem::InvalidLayoutObject {
                slot,
                faction,
                index,
                kind,
                problem,
            } => write!(
                f,
                "slot {} {faction:?} layout object {index} ({kind:?}): {problem}",
                slot.0
            ),
            ScenarioProblem::LayoutOverlap {
                slot,
                faction,
                index,
                tile,
            } => write!(
                f,
                "slot {} {faction:?} layout object {index} overlaps another object at tile ({}, {})",
                slot.0, tile.x, tile.y
            ),
            ScenarioProblem::LayoutPlacement {
                slot,
                faction,
                index,
                kind,
                check,
            } => write!(
                f,
                "slot {} {faction:?} layout object {index} ({kind:?}): {}",
                slot.0,
                placement_text(check)
            ),
            ScenarioProblem::LayoutExtra {
                slot,
                faction,
                index,
                kind,
                allowed,
            } => {
                write!(f, "slot {} {faction:?} layout object {index} ({kind:?}): ", slot.0)?;
                match (StartingConditions::of(*faction), allowed) {
                    (None, _) => write!(
                        f,
                        "the {faction:?} Starting Conditions aren't implemented yet: a Skirmish Default layout of \
                         the faction stays empty"
                    ),
                    (Some(_), 0) => write!(
                        f,
                        "not in the {faction:?} Starting Conditions: a Skirmish Default layout holds only those"
                    ),
                    (Some(_), n) => write!(
                        f,
                        "beyond the {n} of the {faction:?} Starting Conditions: a Skirmish Default layout holds only \
                         those"
                    ),
                }
            }
            ScenarioProblem::LayoutMissing {
                slot,
                faction,
                kind,
                missing,
            } => write!(
                f,
                "slot {} {faction:?} layout lacks {missing} {kind:?} of the Starting Conditions: a Skirmish Default \
                 map is played with every layout complete",
                slot.0
            ),
            ScenarioProblem::NoPlayers => write!(f, "the game has no players"),
            ScenarioProblem::UnknownSlot { player, slot } => {
                write!(
                    f,
                    "player {} takes slot {}, which the map doesn't have",
                    player.0, slot.0
                )
            }
            ScenarioProblem::SlotTaken { player, slot } => {
                write!(
                    f,
                    "player {} takes slot {}, which another player took",
                    player.0, slot.0
                )
            }
        }
    }
}
