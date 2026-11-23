//! Scenario validation: problems are listed, never panicked on, so a bad map file can't
//! crash the editor or the game.

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use glam::Vec2;

use super::settings::{SettingsProblem, SpawnSettings};
use super::{Scenario, SpawnAt};
use crate::defs::{Category, Footprint, ObjectKind};
use crate::ids::PlayerId;
use crate::map::Map;
use crate::object::Placement;
use crate::units::TilePos;

/// Something that makes a scenario unplayable. [`Scenario::validate`] lists them all.
/// `Display` gives a message for the editor or a log.
#[derive(Clone, Debug, PartialEq)]
pub enum ScenarioProblem {
    /// A side of the map is longer than [`crate::map::MAX_MAP_SIDE`] tiles.
    MapTooBig { width: u32, height: u32 },
    /// `core/map.md > Elevation > Maximum slope`: this Traversible tile's corners differ by `slope` levels, more
    /// than [`crate::map::MAX_TRAVERSIBLE_SLOPE`].
    TooSteep { tile: TilePos, slope: u8 },
    /// More than one player has this id.
    DuplicatePlayerId(PlayerId),
    /// `core/entities.md > Player`: players are numbered from 1 to n, the number of players.
    PlayerIdOutOfRange { player: PlayerId, players: usize },
    /// The start location (placeholder model, Q0004) lies off the map.
    StartLocationOffMap { player: PlayerId, tile: TilePos },
    /// The camera start is not a finite point on the map.
    CameraStartOffMap { player: PlayerId, pos: Vec2 },
    /// The spawn at this index of `Scenario::spawns` can't be created.
    InvalidSpawn {
        spawn: usize,
        kind: ObjectKind,
        problem: SpawnProblem,
    },
    /// The tile footprints of the spawns at these indices share `tile`.
    OverlappingFootprints { first: usize, second: usize, tile: TilePos },
}

/// Why an object can't be created as requested, by a scenario spawn or a debug `Spawn` command.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpawnProblem {
    /// `core/map.md > Resource`: resources are never owned.
    OwnedResource,
    /// The owner is not a player of the game.
    UnknownOwner(PlayerId),
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
            ScenarioProblem::DuplicatePlayerId(p) => write!(f, "player id {} is used more than once", p.0),
            ScenarioProblem::PlayerIdOutOfRange { player, players } => write!(
                f,
                "player id {} is outside 1..={players} (players are numbered 1 to n)",
                player.0
            ),
            ScenarioProblem::StartLocationOffMap { player, tile } => write!(
                f,
                "player {}: start location ({}, {}) is off the map",
                player.0, tile.x, tile.y
            ),
            ScenarioProblem::CameraStartOffMap { player, pos } => write!(
                f,
                "player {}: camera start ({}, {}) is not a finite point on the map",
                player.0, pos.x, pos.y
            ),
            ScenarioProblem::InvalidSpawn { spawn, kind, problem } => write!(f, "spawn {spawn} ({kind:?}): {problem}"),
            ScenarioProblem::OverlappingFootprints { first, second, tile } => write!(
                f,
                "spawns {first} and {second} overlap at tile ({}, {})",
                tile.x, tile.y
            ),
        }
    }
}

/// Where a valid spawn goes: its center and, for tile-based types, its placement.
pub(crate) type SpawnSpot = (Vec2, Option<Placement>);

impl Scenario {
    /// Everything that makes this scenario unplayable: the map's too-steep tiles row by row, then the players,
    /// then the spawns in order. Empty means [`crate::Sim::try_new`] accepts it. Never panics.
    pub fn validate(&self) -> Vec<ScenarioProblem> {
        self.spawn_spots().err().unwrap_or_default()
    }

    /// Validates the scenario. If it is valid, returns the spot of each spawn, in order.
    pub(crate) fn spawn_spots(&self) -> Result<Vec<SpawnSpot>, Vec<ScenarioProblem>> {
        let (width, height) = (self.map.width(), self.map.height());
        let max = crate::map::MAX_MAP_SIDE;
        let mut problems = Vec::new();
        if width > max || height > max {
            problems.push(ScenarioProblem::MapTooBig { width, height });
        }
        problems.extend(
            self.map
                .too_steep_tiles()
                .into_iter()
                .map(|(tile, slope)| ScenarioProblem::TooSteep { tile, slope }),
        );
        let players = self.players.len();
        let mut ids = BTreeSet::new();
        for p in &self.players {
            if !ids.insert(p.id) {
                problems.push(ScenarioProblem::DuplicatePlayerId(p.id));
            }
            if p.id.0 == 0 || usize::from(p.id.0) > players {
                problems.push(ScenarioProblem::PlayerIdOutOfRange { player: p.id, players });
            }
            if let Some(tile) = p.start_location
                && !self.map.in_bounds(tile)
            {
                problems.push(ScenarioProblem::StartLocationOffMap { player: p.id, tile });
            }
            if let Some(pos) = p.camera_start
                && !self.map.contains_point(pos)
            {
                problems.push(ScenarioProblem::CameraStartOffMap { player: p.id, pos });
            }
        }

        let mut spots = Vec::with_capacity(self.spawns.len());
        let mut occupied: BTreeMap<TilePos, usize> = BTreeMap::new();
        for (i, s) in self.spawns.iter().enumerate() {
            let spot = place_spawn(&self.map, s.kind, s.owner, s.at, |id| ids.contains(&id))
                .and_then(|spot| check_settings(s.kind, s.settings.as_ref()).map(|()| spot));
            match spot {
                Ok(spot) => {
                    if let Some(placement) = &spot.1 {
                        // One problem per spawn: the first tile it shares with an earlier spawn.
                        let mut overlap = None;
                        for tile in placement.tiles() {
                            match occupied.entry(tile) {
                                Entry::Vacant(e) => {
                                    e.insert(i);
                                }
                                Entry::Occupied(e) => {
                                    overlap.get_or_insert((*e.get(), tile));
                                }
                            }
                        }
                        if let Some((first, tile)) = overlap {
                            problems.push(ScenarioProblem::OverlappingFootprints { first, second: i, tile });
                        }
                    }
                    spots.push(spot);
                }
                Err(problem) => problems.push(ScenarioProblem::InvalidSpawn {
                    spawn: i,
                    kind: s.kind,
                    problem,
                }),
            }
        }
        if problems.is_empty() { Ok(spots) } else { Err(problems) }
    }
}

/// Checks one spawn against the map and the players. Scenario validation and the debug
/// `Spawn` command both use it, so they accept the same spawns (only scenarios also reject
/// overlapping footprints).
pub(crate) fn place_spawn(
    map: &Map,
    kind: ObjectKind,
    owner: Option<PlayerId>,
    at: SpawnAt,
    is_player: impl Fn(PlayerId) -> bool,
) -> Result<SpawnSpot, SpawnProblem> {
    let def = kind.def();
    if let Some(p) = owner {
        // `core/map.md > Resource`: resources are never owned.
        if def.category == Category::Resource {
            return Err(SpawnProblem::OwnedResource);
        }
        if !is_player(p) {
            return Err(SpawnProblem::UnknownOwner(p));
        }
    }
    let spot = locate(map, def.footprint, at)?;
    // `core/map.md > Elevation > Map resources only on flat terrain`.
    if def.needs_flat_ground
        && let (_, Some(placement)) = &spot
        && let Some(sloped) = placement.tiles().find(|t| !map.is_flat(*t))
    {
        return Err(SpawnProblem::NotFlat(sloped));
    }
    Ok(spot)
}

/// `core/map_editor.md > Per-object Resource Values`: a spawn of `kind` needs exactly the values of its kind, and
/// they must make sense.
fn check_settings(kind: ObjectKind, settings: Option<&SpawnSettings>) -> Result<(), SpawnProblem> {
    match (SpawnSettings::defaults(kind), settings) {
        (None, None) => Ok(()),
        (Some(_), None) => Err(SpawnProblem::MissingSettings),
        (_, Some(s)) if s.kind() != kind => Err(SpawnProblem::WrongSettings),
        (_, Some(s)) => s.check().map_err(SpawnProblem::InvalidSettings),
    }
}

/// Where an object with `footprint` goes when spawned `at`, if that fits the map.
pub(crate) fn locate(map: &Map, footprint: Footprint, at: SpawnAt) -> Result<SpawnSpot, SpawnProblem> {
    match (footprint, at) {
        (Footprint::Tiles { w, h }, SpawnAt::Tile(origin)) => {
            // 64-bit sums: no overflow, whatever the origin.
            let fits =
                |start: i32, len: u32, size: u32| start >= 0 && i64::from(start) + i64::from(len) <= i64::from(size);
            if !(fits(origin.x, w, map.width()) && fits(origin.y, h, map.height())) {
                return Err(SpawnProblem::FootprintOffMap);
            }
            let placement = Placement::new(origin, footprint);
            Ok((placement.center(), Some(placement)))
        }
        (Footprint::Silhouette { .. }, SpawnAt::Point(pos)) => {
            if map.contains_point(pos) {
                Ok((pos, None))
            } else {
                Err(SpawnProblem::PointOffMap)
            }
        }
        _ => Err(SpawnProblem::WrongLocationType),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNIT: Footprint = Footprint::Silhouette { w: 20.0, h: 20.0 };
    const BLOCK: Footprint = Footprint::Tiles { w: 2, h: 3 };

    #[test]
    fn units_need_a_finite_point_on_the_map() {
        let map = Map::new(4, 4);
        let at = |x, y| SpawnAt::Point(Vec2::new(x, y));
        assert_eq!(locate(&map, UNIT, at(10.0, 200.0)), Ok((Vec2::new(10.0, 200.0), None)));
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.0, 256.0, 1e30] {
            assert_eq!(
                locate(&map, UNIT, at(bad, 10.0)),
                Err(SpawnProblem::PointOffMap),
                "{bad}"
            );
            assert_eq!(
                locate(&map, UNIT, at(10.0, bad)),
                Err(SpawnProblem::PointOffMap),
                "{bad}"
            );
        }
        let tile = SpawnAt::Tile(TilePos::new(0, 0));
        assert_eq!(locate(&map, UNIT, tile), Err(SpawnProblem::WrongLocationType));
    }

    #[test]
    fn tile_footprints_must_fit_entirely() {
        let map = Map::new(4, 4);
        let at = |x, y| SpawnAt::Tile(TilePos::new(x, y));
        let (center, placement) = locate(&map, BLOCK, at(2, 1)).unwrap();
        assert_eq!(center, Vec2::new(192.0, 160.0));
        assert_eq!(
            placement.map(|p| (p.origin, p.w, p.h)),
            Some((TilePos::new(2, 1), 2, 3))
        );
        for (x, y) in [
            (3, 0),
            (0, 2),
            (-1, 0),
            (0, -1),
            (i32::MAX, 0),
            (0, i32::MAX),
            (i32::MIN, i32::MIN),
        ] {
            assert_eq!(
                locate(&map, BLOCK, at(x, y)),
                Err(SpawnProblem::FootprintOffMap),
                "({x}, {y})"
            );
        }
        let point = SpawnAt::Point(Vec2::new(10.0, 10.0));
        assert_eq!(locate(&map, BLOCK, point), Err(SpawnProblem::WrongLocationType));
    }
}
