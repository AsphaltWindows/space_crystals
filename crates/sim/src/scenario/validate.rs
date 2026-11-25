//! Validation of maps and game setups: problems are listed, never panicked on, so a bad map file can't crash the
//! editor or the game.

mod layouts;
mod occupied;

use std::collections::BTreeSet;

use glam::Vec2;

use super::Scenario;
use super::create::{MAX_SLOTS, slot_count_fits};
use super::mapdef::{BuildingOrientation, MapDef, MapType, Spawn, SpawnAt};
use super::problems::{ScenarioProblem, SpawnProblem};
use super::settings::SpawnSettings;
use crate::defs::{Category, Footprint, ObjectKind};
use crate::ids::{PlayerId, SlotId};
use crate::map::Map;
use crate::nav::TileRect;
use crate::object::Placement;
use crate::paths::misplaced;
use crate::placement::{Occupant, Onto};
use crate::symmetry::Symmetry;
use occupied::Occupied;

/// Where a valid object goes: its center and, for tile-based types, its placement.
pub(crate) type SpawnSpot = (Vec2, Option<Placement>);

/// An object a game starts with, checked and placed.
pub(crate) struct StartObject {
    pub kind: ObjectKind,
    pub owner: Option<PlayerId>,
    pub pos: Vec2,
    pub placement: Option<Placement>,
    pub settings: Option<SpawnSettings>,
}

impl MapDef {
    /// Everything that makes this map unplayable whoever plays it: the terrain (size, then the too-steep tiles row by
    /// row), the symmetry, the slots, the pre-placed objects in order, then the plates among them off a patch
    /// (`placement::Onto`, the placement check's rule), then the units among them on blocked ground or on each other
    /// and the footprints on units (`docs/movement.md > 3`), then the Starting Layouts: every layout on its own, with
    /// its faction's placement rules, and on a Skirmish Default map its Starting Conditions, complete
    /// (`validate/layouts.rs`). Never panics.
    pub fn validate(&self) -> Vec<ScenarioProblem> {
        let mut problems = Vec::new();
        self.check(&mut problems);
        problems
    }

    /// Validates the map into `problems`; the spot of each spawn, in order, where it can be placed.
    fn check(&self, problems: &mut Vec<ScenarioProblem>) -> Vec<Option<SpawnSpot>> {
        let (width, height) = (self.terrain.width(), self.terrain.height());
        let max = crate::map::MAX_MAP_SIDE;
        if width > max || height > max {
            problems.push(ScenarioProblem::MapTooBig { width, height });
        }
        problems.extend(
            self.terrain
                .too_steep_tiles()
                .into_iter()
                .map(|(tile, slope)| ScenarioProblem::TooSteep { tile, slope }),
        );
        if self.map_type == MapType::SkirmishDefault && self.symmetry == Symmetry::None {
            problems.push(ScenarioProblem::SkirmishWithoutSymmetry);
        }
        if !self.symmetry.fits(width, height) {
            problems.push(ScenarioProblem::SymmetryNeedsSquare { width, height });
        } else if let Some(what) = self.first_asymmetry() {
            problems.push(ScenarioProblem::NotSymmetric(what));
        }

        let slots = self.slots.len();
        if slots == 0 {
            problems.push(ScenarioProblem::NoSlots);
        } else if slots > MAX_SLOTS {
            problems.push(ScenarioProblem::TooManySlots(slots));
        } else if !slot_count_fits(self.image_count(), slots) {
            problems.push(ScenarioProblem::SlotsNotInImageSets {
                slots,
                images: self.image_count(),
            });
        }
        for (slot, s) in self.slot_ids().zip(&self.slots) {
            if !self.terrain.in_bounds(s.start) {
                problems.push(ScenarioProblem::StartOffMap { slot, tile: s.start });
            }
            if let Some(pos) = s.camera_start
                && !self.terrain.contains_point(pos)
            {
                problems.push(ScenarioProblem::CameraStartOffMap { slot, pos });
            }
        }

        let mut spots = Vec::with_capacity(self.spawns.len());
        let mut occupied = Occupied::default();
        for (i, s) in self.spawns.iter().enumerate() {
            let spot = self
                .check_owner(s.kind, s.owner)
                .and_then(|()| place_object(&self.terrain, s.kind, s.at, s.orientation))
                .and_then(|spot| check_settings(s.kind, s.settings.as_ref()).map(|()| spot));
            match spot {
                Ok(spot) => {
                    // One problem per spawn: the first tile it shares with an earlier spawn, unless it is a plate on
                    // its patch.
                    if let Some(placement) = &spot.1
                        && let Some((first, tile)) = occupied.take(s.kind, placement, i)
                    {
                        problems.push(ScenarioProblem::OverlappingFootprints { first, second: i, tile });
                    }
                    spots.push(Some(spot));
                }
                Err(problem) => {
                    problems.push(ScenarioProblem::InvalidSpawn {
                        spawn: i,
                        kind: s.kind,
                        problem,
                    });
                    spots.push(None);
                }
            }
        }
        for i in self.off_patch(&spots) {
            let kind = self.spawns[i].kind;
            let problem = SpawnProblem::NotOnPatch;
            problems.push(ScenarioProblem::InvalidSpawn {
                spawn: i,
                kind,
                problem,
            });
        }
        // Units: every spawn that can be placed, whoever owns it, so a game with any players passes.
        let placed: Vec<(usize, ObjectKind, Vec2, Option<&Placement>)> = self
            .spawns
            .iter()
            .zip(&spots)
            .enumerate()
            .filter_map(|(i, (s, spot))| {
                let (pos, placement) = spot.as_ref()?;
                Some((i, s.kind, *pos, placement.as_ref()))
            })
            .collect();
        let objects = placed.iter().map(|&(_, kind, pos, placement)| (kind, pos, placement));
        for (n, problem) in misplaced(&self.terrain, objects) {
            let (spawn, kind, ..) = placed[n];
            problems.push(ScenarioProblem::InvalidSpawn { spawn, kind, problem });
        }

        self.check_layouts(&spots, problems);
        spots
    }

    /// The pre-placed objects, by index, that don't stand on what they go onto: a plate that isn't exactly on a
    /// patch among the others that can be placed (`factions/gdo/economy.md > ExtractionPlate`, "Placed onto a Space
    /// Crystal Patch"). The placement check's rule ([`Onto`]), so that a map, a Starting Layout and a game's
    /// placement can't disagree; a game would start such a plate on nothing, and it would never mine.
    fn off_patch(&self, spots: &[Option<SpawnSpot>]) -> Vec<usize> {
        let occupant = |(s, spot): (&Spawn, &Option<SpawnSpot>)| {
            let (pos, placement) = spot.as_ref()?;
            Some(Occupant {
                kind: s.kind,
                pos: *pos,
                placement: *placement,
                tier: None,
            })
        };
        let all: Vec<Option<Occupant>> = self.spawns.iter().zip(spots).map(occupant).collect();
        let mut off = Vec::new();
        for (i, o) in all.iter().enumerate() {
            let Some(Occupant {
                kind,
                placement: Some(p),
                ..
            }) = o
            else {
                continue;
            };
            let mut onto = Onto::new(*kind, TileRect::new(p.origin, p.w, p.h));
            if onto.is_kept() {
                continue;
            }
            for other in all
                .iter()
                .enumerate()
                .filter(|&(j, _)| j != i)
                .filter_map(|(_, o)| o.as_ref())
            {
                onto.see(other);
            }
            if !onto.is_kept() {
                off.push(i);
            }
        }
        off
    }

    /// Whether a pre-placed object of `kind` may have `owner`.
    fn check_owner(&self, kind: ObjectKind, owner: Option<SlotId>) -> Result<(), SpawnProblem> {
        let Some(slot) = owner else {
            return Ok(());
        };
        if kind.def().category == Category::Resource {
            return Err(SpawnProblem::OwnedResource);
        }
        if self.slot(slot).is_none() {
            return Err(SpawnProblem::UnknownSlot(slot));
        }
        if self.map_type == MapType::SkirmishDefault {
            return Err(SpawnProblem::OwnedOnSkirmishMap);
        }
        Ok(())
    }
}

impl Scenario {
    /// Everything that makes this game unplayable: the map's problems ([`MapDef::validate`], which checks every
    /// Starting Layout on its own), then the setup's, then the overlaps of the Starting Layouts the players get, then
    /// their units on blocked ground or on other units and their footprints on units: the layouts of different
    /// players in each other's way. Empty means [`crate::Sim::try_new`] accepts it. Never panics.
    pub fn validate(&self) -> Vec<ScenarioProblem> {
        self.start_objects().err().unwrap_or_default()
    }

    /// Validates the scenario. If it is valid, returns the objects the game starts with, in order: the pre-placed
    /// objects (those of slots that no player takes are left out), then each player's Starting Layout, player by
    /// player, the objects owned by their players.
    pub(crate) fn start_objects(&self) -> Result<Vec<StartObject>, Vec<ScenarioProblem>> {
        let mut problems = Vec::new();
        let spots = self.map.check(&mut problems);
        if self.setup.players.is_empty() {
            problems.push(ScenarioProblem::NoPlayers);
        }
        let mut taken = BTreeSet::new();
        for (player, p) in self.setup.player_ids().zip(&self.setup.players) {
            if self.map.slot(p.slot).is_none() {
                problems.push(ScenarioProblem::UnknownSlot { player, slot: p.slot });
            } else if !taken.insert(p.slot) {
                problems.push(ScenarioProblem::SlotTaken { player, slot: p.slot });
            }
        }
        if !problems.is_empty() {
            return Err(problems);
        }

        let mut objects = Vec::new();
        // The Starting Layout object each start object comes from (`None` for a pre-placed one).
        let mut from_layout = Vec::new();
        let mut occupied: Occupied<()> = Occupied::default();
        for (s, spot) in self.map.spawns.iter().zip(spots) {
            let Some((pos, placement)) = spot else {
                continue;
            };
            if let Some(placement) = &placement {
                occupied.take(s.kind, placement, ());
            }
            let owner = match s.owner {
                None => None,
                // No player takes the slot: its objects stay out of the game.
                Some(slot) => match self.setup.player_at(slot) {
                    Some(player) => Some(player),
                    None => continue,
                },
            };
            objects.push(StartObject {
                kind: s.kind,
                owner,
                pos,
                placement,
                settings: s.settings,
            });
            from_layout.push(None);
        }
        for (player, p) in self.setup.player_ids().zip(&self.setup.players) {
            let Some(slot) = self.map.slot(p.slot) else {
                continue;
            };
            for (index, o) in slot.layouts.get(p.faction).iter().enumerate() {
                let Ok((pos, placement)) = place_layout_object(&self.map.terrain, o.kind, o.at, o.orientation) else {
                    continue;
                };
                if let Some(placement) = &placement
                    && let Some(((), tile)) = occupied.take(o.kind, placement, ())
                {
                    problems.push(ScenarioProblem::LayoutOverlap {
                        slot: p.slot,
                        faction: p.faction,
                        index,
                        tile,
                    });
                }
                objects.push(StartObject {
                    kind: o.kind,
                    owner: Some(player),
                    pos,
                    placement,
                    settings: None,
                });
                from_layout.push(Some((p.slot, p.faction, index)));
            }
        }
        // Units. The pre-placed objects passed the same check against all the map's spawns, and each layout against
        // them (`MapDef::check`), so only objects of two players' layouts in each other's way can fail it here.
        let placed = objects.iter().map(|o| (o.kind, o.pos, o.placement.as_ref()));
        for (n, problem) in misplaced(&self.map.terrain, placed) {
            if let (Some(&Some((slot, faction, index))), Some(o)) = (from_layout.get(n), objects.get(n)) {
                problems.push(ScenarioProblem::InvalidLayoutObject {
                    slot,
                    faction,
                    index,
                    kind: o.kind,
                    problem,
                });
            }
        }
        if problems.is_empty() {
            Ok(objects)
        } else {
            Err(problems)
        }
    }
}

/// Checks one object owned by a player against the map, turned by `orientation`. The debug `Spawn` command uses it,
/// so that it accepts the objects scenario validation does (the gate checks the other objects in the way itself).
pub(crate) fn place_spawn(
    map: &Map,
    kind: ObjectKind,
    owner: Option<PlayerId>,
    at: SpawnAt,
    orientation: BuildingOrientation,
    is_player: impl Fn(PlayerId) -> bool,
) -> Result<SpawnSpot, SpawnProblem> {
    if let Some(p) = owner {
        // `core/map.md > Resource`: resources are never owned.
        if kind.def().category == Category::Resource {
            return Err(SpawnProblem::OwnedResource);
        }
        if !is_player(p) {
            return Err(SpawnProblem::UnknownOwner(p));
        }
    }
    place_object(map, kind, at, orientation)
}

/// Checks a Starting Layout object, which its player owns, against the map.
fn place_layout_object(
    map: &Map,
    kind: ObjectKind,
    at: SpawnAt,
    orientation: BuildingOrientation,
) -> Result<SpawnSpot, SpawnProblem> {
    if kind.def().category == Category::Resource {
        return Err(SpawnProblem::OwnedResource);
    }
    place_object(map, kind, at, orientation)
}

/// Where an object of `kind` goes at `at`, turned by `orientation`, if that fits the map and its ground.
fn place_object(
    map: &Map,
    kind: ObjectKind,
    at: SpawnAt,
    orientation: BuildingOrientation,
) -> Result<SpawnSpot, SpawnProblem> {
    let def = kind.def();
    let spot = locate_oriented(map, def.footprint, at, orientation)?;
    // `core/map.md > Elevation > Structures only on flat terrain`, `core/map.md > Map resources only on flat terrain`.
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

/// Where an object with `footprint` goes when spawned `at` and turned by `orientation`, if that fits the map.
pub(crate) fn locate_oriented(
    map: &Map,
    footprint: Footprint,
    at: SpawnAt,
    orientation: BuildingOrientation,
) -> Result<SpawnSpot, SpawnProblem> {
    match (footprint, at) {
        (Footprint::Tiles { w, h }, SpawnAt::Tile(origin)) => {
            let (w, h) = if orientation.swaps_axes() { (h, w) } else { (w, h) };
            // 64-bit sums: no overflow, whatever the origin.
            let fits =
                |start: i32, len: u32, size: u32| start >= 0 && i64::from(start) + i64::from(len) <= i64::from(size);
            if !(fits(origin.x, w, map.width()) && fits(origin.y, h, map.height())) {
                return Err(SpawnProblem::FootprintOffMap);
            }
            let placement = Placement {
                origin,
                w,
                h,
                rotation: orientation.rotation,
                flip_h: orientation.flip_h,
                flip_v: orientation.flip_v,
            };
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
    use crate::object::Rotation;
    use crate::units::TilePos;

    const UNIT: Footprint = Footprint::Silhouette { w: 20.0, h: 20.0 };
    const BLOCK: Footprint = Footprint::Tiles { w: 2, h: 3 };

    fn locate(map: &Map, footprint: Footprint, at: SpawnAt) -> Result<SpawnSpot, SpawnProblem> {
        locate_oriented(map, footprint, at, BuildingOrientation::default())
    }

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

    #[test]
    fn a_quarter_turn_puts_the_footprint_on_its_side() {
        let map = Map::new(4, 4);
        let turned = BuildingOrientation {
            rotation: Rotation::R90,
            ..Default::default()
        };
        let (_, placement) = locate_oriented(&map, BLOCK, SpawnAt::Tile(TilePos::new(1, 2)), turned).unwrap();
        let placement = placement.unwrap();
        assert_eq!((placement.w, placement.h, placement.rotation), (3, 2, Rotation::R90));
        assert_eq!(
            locate_oriented(&map, BLOCK, SpawnAt::Tile(TilePos::new(2, 2)), turned),
            Err(SpawnProblem::FootprintOffMap)
        );
    }
}
