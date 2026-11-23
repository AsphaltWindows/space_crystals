//! Generated symmetry (`core/map_editor.md > Symmetry > Editing a symmetric map`): only the editable part of a
//! symmetric map ([`EditableRegion`]) is edited, and [`MapDef::regenerate`] makes the rest the images of it.
//! Everything is mirrored: tiles, corner heights, pre-placed objects with their values, and the slots with their
//! start positions, camera starts and every faction's Starting Layout. Building orientations turn and flip with the
//! map.
//!
//! An object belongs to the editable part when its center does. One that is its own image (a 2×2 station at the
//! center of an even map, a 1×1 patch on the middle tile of a mirror axis) stays one object. One whose footprint
//! partly overlaps an image of itself can't be placed: the editor refuses it, and validation flags the overlap
//! (placeholder rule until the design settles objects on the axis, `core/map_editor.md > Symmetry`'s TBD).
//!
//! Slots come in whole sets of images (placeholder rule for the same TBD): on a map with `n` images, slot `k·n + 1`
//! is edited and slot `k·n + 1 + j` is its image `j`. A slot's objects follow it: the image of an object owned by
//! slot `s` is owned by the image of `s`.

use glam::Vec2;

use super::mapdef::{BuildingOrientation, LayoutObject, MapDef, Slot, Spawn, SpawnAt};
use crate::defs::{Category, Footprint, ObjectKind};
use crate::ids::SlotId;
use crate::object::Placement;
use crate::symmetry::{EditableRegion, MapTransform, Orientation, Symmetry};
use crate::units::{CornerPos, TilePos};

/// The first thing that differs from what the editable part generates.
#[derive(Clone, Debug, PartialEq)]
pub enum Asymmetry {
    Tile(TilePos),
    Corner(CornerPos),
    Slot(SlotId),
    /// The pre-placed objects.
    Spawns,
}

impl MapDef {
    /// The symmetry if it fits the map, else [`Symmetry::None`] (90° rotation of a map that isn't square, which
    /// validation flags).
    fn fitting_symmetry(&self) -> Symmetry {
        if self.symmetry.fits(self.terrain.width(), self.terrain.height()) {
            self.symmetry
        } else {
            Symmetry::None
        }
    }

    /// The transform of each image, the identity first ([`Symmetry::transforms`]).
    pub fn transforms(&self) -> Vec<MapTransform> {
        let (w, h) = (self.terrain.width(), self.terrain.height());
        self.fitting_symmetry()
            .transforms(w, h)
            .unwrap_or_else(|| vec![MapTransform::new(Orientation::IDENTITY, w, h)])
    }

    /// How many images the map consists of: 1, 2 or 4.
    pub fn image_count(&self) -> usize {
        self.fitting_symmetry().images().len()
    }

    /// The part the map maker edits.
    pub fn region(&self) -> EditableRegion {
        EditableRegion::new(self.symmetry, self.terrain.width(), self.terrain.height())
    }

    /// Whether `slot` is edited, rather than generated as the image of another slot.
    pub fn is_slot_editable(&self, slot: SlotId) -> bool {
        slot.index().is_some_and(|i| i % self.image_count() == 0)
    }

    /// The editable slot that `slot` is an image of (itself if it is editable).
    pub fn source_slot(&self, slot: SlotId) -> SlotId {
        let n = self.image_count();
        slot.index().and_then(|i| SlotId::from_index(i - i % n)).unwrap_or(slot)
    }

    /// The image `image` (an index of [`MapDef::transforms`]) of slot `slot`: the slot whose start is that image of
    /// `slot`'s start. Slots outside whole sets of images map to themselves.
    pub fn image_slot(&self, slot: SlotId, image: usize) -> SlotId {
        let orientations = self.fitting_symmetry().images();
        let n = orientations.len();
        let (Some(i), Some(turn)) = (slot.index(), orientations.get(image)) else {
            return slot;
        };
        if !self.slots.len().is_multiple_of(n) || i >= self.slots.len() {
            return slot;
        }
        let (set, a) = (i - i % n, i % n);
        let target = turn.after(orientations[a]);
        let b = orientations.iter().position(|o| *o == target).unwrap_or(a);
        SlotId::from_index(set + b).unwrap_or(slot)
    }

    /// Whether `spawn` lies in the editable part: its center does.
    pub fn is_spawn_editable(&self, spawn: &Spawn) -> bool {
        is_editable(&self.region(), spawn.kind, spawn.at, spawn.orientation)
    }

    /// The images of `spawn` other than itself, each once, in transform order: where the regenerated map has them.
    pub fn spawn_images(&self, spawn: &Spawn) -> Vec<Spawn> {
        self.images_by_transform(spawn).into_iter().map(|(_, s)| s).collect()
    }

    /// [`MapDef::spawn_images`], each with the index of the first transform that makes it.
    fn images_by_transform(&self, spawn: &Spawn) -> Vec<(usize, Spawn)> {
        let mut images: Vec<(usize, Spawn)> = Vec::new();
        for (j, m) in self.transforms().iter().enumerate().skip(1) {
            let (at, orientation) = image_location(spawn.kind, spawn.at, spawn.orientation, m);
            let image = Spawn {
                owner: spawn.owner.map(|o| self.image_slot(o, j)),
                at,
                orientation,
                ..spawn.clone()
            };
            let same_place = |s: &Spawn| s.at == image.at && s.orientation == image.orientation;
            if !same_place(spawn) && !images.iter().any(|(_, s)| same_place(s)) {
                images.push((j, image));
            }
        }
        images
    }

    /// Generates the locked part from the editable part: every tile and corner height, pre-placed object and slot
    /// there becomes the image of the one it comes from. The pre-placed objects of the editable part come first, in
    /// their order, then their images, transform by transform. Slots outside whole sets of images stay as they are.
    /// Does nothing without a symmetry.
    pub fn regenerate(&mut self) {
        let transforms = self.transforms();
        if transforms.len() < 2 {
            return;
        }
        let region = self.region();
        let (w, h) = (self.terrain.width() as i32, self.terrain.height() as i32);
        for y in 0..h {
            for x in 0..w {
                let t = TilePos::new(x, y);
                if let (true, Some(tile)) = (region.tile(t), self.terrain.tile(t)) {
                    for m in &transforms[1..] {
                        self.terrain.set_tile(m.tile(t), tile);
                    }
                }
            }
        }
        for y in 0..=h {
            for x in 0..=w {
                let c = CornerPos::new(x, y);
                if let (true, Some(height)) = (region.corner(c), self.terrain.corner(c)) {
                    for m in &transforms[1..] {
                        self.terrain.set_corner(m.corner(c), height);
                    }
                }
            }
        }

        let editable: Vec<Spawn> = self
            .spawns
            .iter()
            .filter(|s| self.is_spawn_editable(s))
            .cloned()
            .collect();
        let images: Vec<Vec<(usize, Spawn)>> = editable.iter().map(|s| self.images_by_transform(s)).collect();
        let mut spawns = editable;
        for j in 1..transforms.len() {
            for of_spawn in &images {
                spawns.extend(of_spawn.iter().filter(|(k, _)| *k == j).map(|(_, s)| s.clone()));
            }
        }
        self.spawns = spawns;

        let n = transforms.len();
        if self.slots.len().is_multiple_of(n) {
            for set in (0..self.slots.len()).step_by(n) {
                let source = self.slots[set].clone();
                for (j, m) in transforms.iter().enumerate().skip(1) {
                    self.slots[set + j] = slot_image(&source, m);
                }
            }
        }
    }

    /// The first thing that isn't what the editable part generates, or `None` if the map is symmetric.
    pub fn first_asymmetry(&self) -> Option<Asymmetry> {
        if self.image_count() < 2 {
            return None;
        }
        let mut generated = self.clone();
        generated.regenerate();
        if generated == *self {
            return None;
        }
        let (w, h) = (self.terrain.width() as i32, self.terrain.height() as i32);
        let tiles = (0..h).flat_map(|y| (0..w).map(move |x| TilePos::new(x, y)));
        if let Some(t) = tiles
            .into_iter()
            .find(|t| generated.terrain.tile(*t) != self.terrain.tile(*t))
        {
            return Some(Asymmetry::Tile(t));
        }
        let corners = (0..=h).flat_map(|y| (0..=w).map(move |x| CornerPos::new(x, y)));
        if let Some(c) = corners
            .into_iter()
            .find(|c| generated.terrain.corner(*c) != self.terrain.corner(*c))
        {
            return Some(Asymmetry::Corner(c));
        }
        let slot = (0..self.slots.len()).find(|&i| generated.slots.get(i) != self.slots.get(i));
        if let Some(slot) = slot.and_then(SlotId::from_index) {
            return Some(Asymmetry::Slot(slot));
        }
        Some(Asymmetry::Spawns)
    }
}

/// Whether an object of `kind` at `at` lies in the editable part `region`: its center does.
pub(crate) fn is_editable(
    region: &EditableRegion,
    kind: ObjectKind,
    at: SpawnAt,
    orientation: BuildingOrientation,
) -> bool {
    match (at, kind.def().footprint) {
        (SpawnAt::Tile(o), Footprint::Tiles { w, h }) => {
            let (w, h) = if orientation.swaps_axes() { (h, w) } else { (w, h) };
            region.footprint(o, w, h)
        }
        (SpawnAt::Tile(o), Footprint::Silhouette { .. }) => region.tile(o),
        (SpawnAt::Point(p), _) => region.point(p),
    }
}

/// Where the image under `m` of an object of `kind` at `at` with `orientation` lies, and how it is turned. Only
/// structures have an orientation (`core/entities.md > Structure Type`); other tile-based objects keep theirs, and
/// one whose footprint a quarter turn would put on its side keeps its shape around the image of its center.
pub fn image_location(
    kind: ObjectKind,
    at: SpawnAt,
    orientation: BuildingOrientation,
    m: &MapTransform,
) -> (SpawnAt, BuildingOrientation) {
    match (at, kind.def().footprint) {
        (SpawnAt::Tile(o), Footprint::Tiles { w, h }) => {
            let (w, h) = if orientation.swaps_axes() { (h, w) } else { (w, h) };
            if kind.def().category == Category::Structure {
                let placement = Placement {
                    origin: o,
                    w,
                    h,
                    rotation: orientation.rotation,
                    flip_h: orientation.flip_h,
                    flip_v: orientation.flip_v,
                };
                let image = m.placement(&placement);
                let turned = BuildingOrientation {
                    rotation: image.rotation,
                    flip_h: image.flip_h,
                    flip_v: image.flip_v,
                };
                return (SpawnAt::Tile(image.origin), turned);
            }
            let (origin, w2, h2) = m.footprint(o, w, h);
            if (w2, h2) == (w, h) {
                return (SpawnAt::Tile(origin), orientation);
            }
            // The image's center in doubled tile units, and the same shape around it (rounded north-west).
            let center = |start: i32, size: u32| 2 * i64::from(start) + i64::from(size);
            let corner = |c: i64, size: u32| narrow((c - i64::from(size)).div_euclid(2));
            let origin = TilePos::new(corner(center(origin.x, w2), w), corner(center(origin.y, h2), h));
            (SpawnAt::Tile(origin), orientation)
        }
        (SpawnAt::Tile(o), Footprint::Silhouette { .. }) => (SpawnAt::Tile(m.tile(o)), orientation),
        (SpawnAt::Point(p), _) => (SpawnAt::Point(m.point(p)), orientation),
    }
}

/// The image of a layout object under `m`.
fn layout_image(object: &LayoutObject, m: &MapTransform) -> LayoutObject {
    let (at, orientation) = image_location(object.kind, object.at, object.orientation, m);
    LayoutObject {
        kind: object.kind,
        at,
        orientation,
    }
}

/// The image of a slot under `m`: its start position, camera start and every faction's layout.
fn slot_image(slot: &Slot, m: &MapTransform) -> Slot {
    let mut image = Slot {
        start: m.tile(slot.start),
        camera_start: slot.camera_start.map(|c: Vec2| m.point(c)),
        layouts: Default::default(),
    };
    for (faction, objects) in slot.layouts.iter() {
        *image.layouts.get_mut(faction) = objects.iter().map(|o| layout_image(o, m)).collect();
    }
    image
}

/// `v` as an `i32`, saturated.
fn narrow(v: i64) -> i32 {
    v.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}
