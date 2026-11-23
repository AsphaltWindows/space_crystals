//! What makes a map or a game setup unplayable, as listed by validation, with a message for each.

use std::fmt;

use glam::Vec2;

use super::create::MAX_SLOTS;
use super::generate::Asymmetry;
use super::settings::SettingsProblem;
use crate::defs::ObjectKind;
use crate::ids::{PlayerId, SlotId};
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
    /// The tile footprints of the spawns at these indices share `tile`.
    OverlappingFootprints { first: usize, second: usize, tile: TilePos },
    /// Object `index` of a slot's Starting Layout for `faction` can't be created.
    InvalidLayoutObject {
        slot: SlotId,
        faction: Faction,
        index: usize,
        kind: ObjectKind,
        problem: SpawnProblem,
    },
    /// Object `index` of the Starting Layout a player gets overlaps a pre-placed object or another starting object.
    LayoutOverlap {
        slot: SlotId,
        faction: Faction,
        index: usize,
        tile: TilePos,
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
    /// `core/map.md > Elevation > Map resources only on flat terrain`: this footprint tile is sloped.
    NotFlat(TilePos),
    /// `core/map_editor.md > Per-object Resource Values`: the spawn has no values, but its kind needs them.
    MissingSettings,
    /// The spawn's values belong to another kind of object.
    WrongSettings,
    /// One of the spawn's values makes no sense.
    InvalidSettings(SettingsProblem),
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
            SpawnProblem::MissingSettings => write!(f, "its per-object values are missing"),
            SpawnProblem::WrongSettings => write!(f, "its per-object values belong to another kind of object"),
            SpawnProblem::InvalidSettings(problem) => write!(f, "{problem}"),
        }
    }
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
