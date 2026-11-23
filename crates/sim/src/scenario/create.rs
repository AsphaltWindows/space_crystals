//! New maps (`core/map_editor.md > Map Creation`): the creation settings, their limits, and the map they make.

use std::fmt;

use serde::{Deserialize, Serialize};

use super::mapdef::{MapDef, MapType, Slot};
use crate::map::{MAX_MAP_SIDE, Map};
use crate::symmetry::{EditableRegion, Symmetry};
use crate::units::TilePos;

/// The shortest side of a map the editor creates or resizes to, in tiles (placeholder: `core/map_editor.md > Map
/// Creation` leaves the editor's minimum TBD). The sim itself accepts any size from 1 up to [`MAX_MAP_SIDE`], so
/// tests can use tiny maps.
pub const MIN_MAP_SIDE: u32 = 16;

/// The most player slots a map has (placeholder: `core/map_editor.md > Map Creation` leaves the maximum TBD).
pub const MAX_SLOTS: usize = 8;

/// What the map maker chooses for a new map (`core/map_editor.md > Map Creation`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapSettings {
    pub map_type: MapType,
    pub width: u32,
    pub height: u32,
    /// The number of player slots (start positions).
    pub slots: usize,
    pub symmetry: Symmetry,
    pub name: String,
    pub description: String,
}

impl Default for MapSettings {
    /// A 64×48 Skirmish Default map for two, with 180° symmetry.
    fn default() -> Self {
        Self {
            map_type: MapType::SkirmishDefault,
            width: 64,
            height: 48,
            slots: 2,
            symmetry: Symmetry::Rotate180,
            name: "untitled".into(),
            description: String::new(),
        }
    }
}

/// Why settings can't make a map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CreationProblem {
    /// A side is outside `MIN_MAP_SIDE..=MAX_MAP_SIDE`.
    Size { width: u32, height: u32 },
    /// `core/map_editor.md > Map Types > Skirmish Default`: a symmetry is required.
    SkirmishWithoutSymmetry,
    /// `core/map_editor.md > Symmetry`: 90° rotation needs a square map.
    SymmetryNeedsSquare,
    /// The slot count isn't one of [`MapSettings::slot_counts`].
    SlotCount { slots: usize, allowed: Vec<usize> },
    /// A map needs a name.
    NoName,
}

impl fmt::Display for CreationProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CreationProblem::Size { width, height } => write!(
                f,
                "{width} x {height} tiles: sides go from {MIN_MAP_SIDE} to {MAX_MAP_SIDE}"
            ),
            CreationProblem::SkirmishWithoutSymmetry => write!(f, "a Skirmish Default map needs a symmetry"),
            CreationProblem::SymmetryNeedsSquare => write!(f, "90 degree rotation needs a square map"),
            CreationProblem::SlotCount { slots, allowed } => {
                let allowed: Vec<String> = allowed.iter().map(ToString::to_string).collect();
                write!(
                    f,
                    "{slots} player slots: with this symmetry a map has {}",
                    allowed.join(", ")
                )
            }
            CreationProblem::NoName => write!(f, "a map needs a name"),
        }
    }
}

/// Whether `slots` slots suit a map of `images` symmetric images: whole sets of images, from one set up to
/// [`MAX_SLOTS`] (placeholder rule until the design relates slot counts to symmetry, `core/map_editor.md >
/// Symmetry`'s TBD).
pub(crate) fn slot_count_fits(images: usize, slots: usize) -> bool {
    slots >= images.max(1) && slots <= MAX_SLOTS && slots.is_multiple_of(images.max(1))
}

impl MapSettings {
    /// The slot counts a map with `symmetry` may have ([`slot_count_fits`]).
    pub fn slot_counts(symmetry: Symmetry) -> Vec<usize> {
        let images = symmetry.images().len();
        (1..=MAX_SLOTS).filter(|&n| slot_count_fits(images, n)).collect()
    }

    /// Whether these settings make a map; the first problem if not.
    pub fn check(&self) -> Result<(), CreationProblem> {
        let side = MIN_MAP_SIDE..=MAX_MAP_SIDE;
        if !side.contains(&self.width) || !side.contains(&self.height) {
            return Err(CreationProblem::Size {
                width: self.width,
                height: self.height,
            });
        }
        if self.map_type == MapType::SkirmishDefault && self.symmetry == Symmetry::None {
            return Err(CreationProblem::SkirmishWithoutSymmetry);
        }
        if !self.symmetry.fits(self.width, self.height) {
            return Err(CreationProblem::SymmetryNeedsSquare);
        }
        if !slot_count_fits(self.symmetry.images().len(), self.slots) {
            return Err(CreationProblem::SlotCount {
                slots: self.slots,
                allowed: Self::slot_counts(self.symmetry),
            });
        }
        if self.name.trim().is_empty() {
            return Err(CreationProblem::NoName);
        }
        Ok(())
    }

    /// A flat map of Plane tiles with these settings: the editable slots' starts spread over the editable part
    /// ([`default_start`]), their images generated, no objects.
    pub fn create(&self) -> Result<MapDef, CreationProblem> {
        self.check()?;
        let mut map = MapDef {
            map_type: self.map_type,
            name: self.name.trim().to_string(),
            description: self.description.trim().to_string(),
            terrain: Map::new(self.width, self.height),
            symmetry: self.symmetry,
            slots: Vec::new(),
            spawns: Vec::new(),
        };
        let images = self.symmetry.images().len();
        let editable = self.slots / images;
        let region = map.region();
        for i in 0..editable {
            map.slots.push(Slot::new(default_start(&region, i, editable)));
            for _ in 1..images {
                map.slots.push(Slot::new(TilePos::new(0, 0)));
            }
        }
        map.regenerate();
        Ok(map)
    }
}

/// Where editable slot `i` of `count` starts on a new map: spread evenly along the diagonal of the part strictly
/// inside the editable region (so that no start lies on an axis), from its north-west.
pub(crate) fn default_start(region: &EditableRegion, i: usize, count: usize) -> TilePos {
    let (min, max) = region.inner_rect();
    let along = |lo: i32, hi: i32| {
        let span = i64::from(hi - lo);
        let at = span * (i as i64 + 1) / (count as i64 + 1);
        lo + at.clamp(0, (span - 1).max(0)) as i32
    };
    TilePos::new(along(min.x, max.x), along(min.y, max.y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_counts_come_in_whole_sets_of_images() {
        assert_eq!(MapSettings::slot_counts(Symmetry::None), [1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(MapSettings::slot_counts(Symmetry::Rotate180), [2, 4, 6, 8]);
        assert_eq!(MapSettings::slot_counts(Symmetry::MirrorVerticalAxis), [2, 4, 6, 8]);
        assert_eq!(MapSettings::slot_counts(Symmetry::Rotate90), [4, 8]);
        assert_eq!(MapSettings::slot_counts(Symmetry::MirrorBothAxes), [4, 8]);
    }

    #[test]
    fn settings_are_checked_against_the_limits() {
        let ok = MapSettings::default();
        assert_eq!(ok.check(), Ok(()));
        let with = |change: fn(&mut MapSettings)| {
            let mut s = MapSettings::default();
            change(&mut s);
            s.check()
        };
        assert!(matches!(
            with(|s| s.width = MIN_MAP_SIDE - 1),
            Err(CreationProblem::Size { .. })
        ));
        assert!(matches!(
            with(|s| s.height = MAX_MAP_SIDE + 1),
            Err(CreationProblem::Size { .. })
        ));
        assert_eq!(
            with(|s| s.symmetry = Symmetry::None),
            Err(CreationProblem::SkirmishWithoutSymmetry)
        );
        assert_eq!(
            with(|s| s.symmetry = Symmetry::Rotate90),
            Err(CreationProblem::SymmetryNeedsSquare)
        );
        assert!(matches!(with(|s| s.slots = 3), Err(CreationProblem::SlotCount { .. })));
        assert!(matches!(with(|s| s.slots = 10), Err(CreationProblem::SlotCount { .. })));
        assert_eq!(with(|s| s.name = "  ".into()), Err(CreationProblem::NoName));
        // Custom maps may do without symmetry, with any number of slots up to the maximum.
        let custom = |s: &mut MapSettings| {
            s.map_type = MapType::Custom;
            s.symmetry = Symmetry::None;
            s.slots = 3;
        };
        assert_eq!(with(custom), Ok(()));
    }

    #[test]
    fn new_maps_are_flat_and_symmetric_with_their_slots_spread() {
        for symmetry in Symmetry::ALL {
            for &slots in &MapSettings::slot_counts(symmetry) {
                let settings = MapSettings {
                    map_type: MapType::Custom,
                    width: 32,
                    height: 32,
                    slots,
                    symmetry,
                    name: " New ".into(),
                    description: "d".into(),
                };
                let map = settings.create().unwrap();
                assert_eq!(map.name, "New");
                assert_eq!(map.slots.len(), slots);
                assert!(map.validate().is_empty(), "{symmetry:?} {slots}: {:?}", map.validate());
                for id in map.slot_ids().filter(|s| map.is_slot_editable(*s)) {
                    assert!(map.region().tile(map.slot(id).unwrap().start), "{symmetry:?} {slots}");
                }
            }
        }
    }
}
