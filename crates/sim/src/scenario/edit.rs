//! Map editing helpers for tools such as the map editor: player slots and resizing. They keep the invariants a map
//! needs where they can (slots in whole sets of images, a symmetric map) and report what they had to drop.

use super::create::{MAX_SLOTS, default_start};
use super::mapdef::{MapDef, Slot, Spawn};
use super::validate::locate_oriented;
use crate::ids::SlotId;
use crate::map::Map;
use crate::units::TilePos;

/// What [`MapDef::resize`] changed because it fell outside the new map.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ResizeReport {
    /// The removed spawns, each with its index in `MapDef::spawns` before the resize.
    pub removed_spawns: Vec<(usize, Spawn)>,
    /// Slots whose start was moved onto the new map.
    pub moved_starts: Vec<SlotId>,
    /// Slots whose camera start was cleared.
    pub cleared_camera_starts: Vec<SlotId>,
}

impl ResizeReport {
    /// Nothing was dropped or moved.
    pub fn is_empty(&self) -> bool {
        self.removed_spawns.is_empty() && self.moved_starts.is_empty() && self.cleared_camera_starts.is_empty()
    }
}

impl MapDef {
    /// Resizes the map, anchored at the top-left corner ([`Map::resize`]), drops what falls outside it (spawns that
    /// fitted the old map but not the new one, camera starts that were on the old map), moves starts onto the
    /// nearest tile of the new map, and regenerates the locked part ([`MapDef::regenerate`]). Things that were
    /// already off the map stay, with their validation problem. Panics if a side is 0.
    pub fn resize(&mut self, width: u32, height: u32) -> ResizeReport {
        let fits = |map: &Map, s: &Spawn| locate_oriented(map, s.kind.def().footprint, s.at, s.orientation).is_ok();
        let fitted: Vec<bool> = self.spawns.iter().map(|s| fits(&self.terrain, s)).collect();
        let cameras_on: Vec<bool> = self
            .slots
            .iter()
            .map(|s| s.camera_start.is_some_and(|c| self.terrain.contains_point(c)))
            .collect();
        let starts_on: Vec<bool> = self.slots.iter().map(|s| self.terrain.in_bounds(s.start)).collect();
        self.terrain.resize(width, height);

        let mut report = ResizeReport::default();
        for (i, (spawn, fitted)) in std::mem::take(&mut self.spawns).into_iter().zip(fitted).enumerate() {
            if fitted && !fits(&self.terrain, &spawn) {
                report.removed_spawns.push((i, spawn));
            } else {
                self.spawns.push(spawn);
            }
        }
        let ids: Vec<SlotId> = self.slot_ids().collect();
        for (((slot, id), camera_on), start_on) in self.slots.iter_mut().zip(ids).zip(cameras_on).zip(starts_on) {
            if camera_on && slot.camera_start.is_some_and(|c| !self.terrain.contains_point(c)) {
                slot.camera_start = None;
                report.cleared_camera_starts.push(id);
            }
            if start_on && !self.terrain.in_bounds(slot.start) {
                let keep = |v: i32, size: u32| v.clamp(0, size as i32 - 1);
                slot.start = TilePos::new(keep(slot.start.x, width), keep(slot.start.y, height));
                report.moved_starts.push(id);
            }
        }
        self.regenerate();
        report
    }

    /// Adds a set of slots: an editable one, at a free spot of the editable part, and its images. Returns the new
    /// editable slot, or `None`, changing nothing, if that would make more than [`MAX_SLOTS`].
    pub fn add_slots(&mut self) -> Option<SlotId> {
        let n = self.image_count();
        if !self.slots.len().is_multiple_of(n) || self.slots.len() + n > MAX_SLOTS {
            return None;
        }
        let region = self.region();
        let count = self.slots.len() / n + 1;
        let start = (0..count)
            .map(|i| default_start(&region, i, count))
            .find(|t| self.slots.iter().all(|s| s.start != *t))
            .unwrap_or_else(|| default_start(&region, 0, 1));
        let id = SlotId::from_index(self.slots.len())?;
        self.slots.push(Slot::new(start));
        for _ in 1..n {
            self.slots.push(Slot::new(start));
        }
        self.regenerate();
        Some(id)
    }

    /// Removes the set of slots that `slot` belongs to (one slot without a symmetry). The slots after them move down
    /// to keep the numbers `1..=n`, and so do their objects' owners; the removed slots' objects become neutral.
    /// Returns how many became neutral, or `None`, changing nothing, if there is no such slot or it would leave
    /// none.
    pub fn remove_slots(&mut self, slot: SlotId) -> Option<usize> {
        let n = self.image_count();
        let index = slot.index().filter(|&i| i < self.slots.len())?;
        let first = index - index % n;
        let last = (first + n).min(self.slots.len());
        if last - first >= self.slots.len() {
            return None;
        }
        self.slots.drain(first..last);
        let removed = last - first;
        let mut neutralized = 0;
        for s in &mut self.spawns {
            let Some(owner) = s.owner.and_then(SlotId::index) else {
                continue;
            };
            if (first..last).contains(&owner) {
                s.owner = None;
                neutralized += 1;
            } else if owner >= last {
                s.owner = SlotId::from_index(owner - removed);
            }
        }
        Some(neutralized)
    }
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use super::*;
    use crate::defs::ObjectKind;
    use crate::scenario::{MapSettings, MapType, ScenarioProblem, SpawnAt};
    use crate::scenarios;
    use crate::symmetry::Symmetry;

    fn spawn(kind: ObjectKind, owner: Option<u8>, x: i32, y: i32) -> Spawn {
        Spawn::new(kind, owner.map(SlotId), SpawnAt::Tile(TilePos::new(x, y)))
    }

    /// Whether the only problems of `map`, a new Skirmish Default map, are its Starting Layouts: they start empty
    /// (design change 0009), and such a map is played with each one complete (design change 0012).
    fn only_layouts_missing(map: &MapDef) -> bool {
        let problems = map.validate();
        !problems.is_empty()
            && problems
                .iter()
                .all(|p| matches!(p, ScenarioProblem::LayoutMissing { .. }))
    }

    #[test]
    fn slots_are_added_and_removed_in_whole_sets_of_images() {
        let mut map = MapSettings {
            width: 24,
            height: 24,
            slots: 4,
            symmetry: Symmetry::Rotate90,
            ..Default::default()
        }
        .create()
        .unwrap();
        assert_eq!(map.add_slots(), Some(SlotId(5)));
        assert_eq!(map.slots.len(), 8);
        assert!(only_layouts_missing(&map), "{:?}", map.validate());
        assert_eq!(map.add_slots(), None, "at most {MAX_SLOTS}");
        // Removing slot 7 removes its set, slots 5 to 8.
        assert_eq!(map.remove_slots(SlotId(7)), Some(0));
        assert_eq!(map.slots.len(), 4);
        assert_eq!(map.remove_slots(SlotId(2)), None, "the last set stays");
        assert_eq!(map.remove_slots(SlotId(9)), None);
    }

    #[test]
    fn removing_slots_renumbers_the_rest_and_their_objects() {
        let mut map = scenarios::empty(8, 8).map;
        map.add_slots();
        assert_eq!(map.slots.len(), 3);
        // Owned resources are invalid, but the owners still follow the renumbering.
        map.spawns = vec![
            spawn(ObjectKind::CrystalPatch, Some(1), 0, 0),
            spawn(ObjectKind::CrystalPatch, Some(2), 1, 0),
            spawn(ObjectKind::CrystalPatch, Some(3), 2, 0),
            spawn(ObjectKind::CrystalPatch, None, 3, 0),
        ];
        assert_eq!(map.remove_slots(SlotId(2)), Some(1));
        let owners: Vec<Option<u8>> = map.spawns.iter().map(|s| s.owner.map(|p| p.0)).collect();
        assert_eq!(owners, [Some(1), None, Some(2), None]);
    }

    #[test]
    fn resizing_drops_what_falls_outside_and_reports_it() {
        let mut map = scenarios::empty(10, 8).map;
        map.slots[0].start = TilePos::new(1, 1);
        map.slots[0].camera_start = Some(Vec2::new(600.0, 100.0)); // tile (9, 1)
        map.slots[1].start = TilePos::new(8, 6);
        map.spawns = vec![
            spawn(ObjectKind::CrystalPatch, None, 2, 2),
            spawn(ObjectKind::SupplyDeliveryStation, None, 5, 1), // covers x 5..=6
            spawn(ObjectKind::CrystalPatch, None, 1, 6),
        ];
        let report = map.resize(6, 7);
        assert_eq!((map.terrain.width(), map.terrain.height()), (6, 7));
        let removed: Vec<usize> = report.removed_spawns.iter().map(|(i, _)| *i).collect();
        assert_eq!(removed, [1]);
        assert_eq!(report.removed_spawns[0].1.kind, ObjectKind::SupplyDeliveryStation);
        assert_eq!(map.spawns.len(), 2);
        assert_eq!(report.moved_starts, [SlotId(2)]);
        assert_eq!(map.slots[1].start, TilePos::new(5, 6));
        assert_eq!(report.cleared_camera_starts, [SlotId(1)]);
        assert!(map.validate().is_empty());
        assert!(map.resize(6, 7).is_empty(), "nothing more to drop");
    }

    #[test]
    fn resizing_keeps_what_was_already_off_the_map_and_regenerates() {
        let mut map = scenarios::empty(4, 4).map;
        map.spawns = vec![spawn(ObjectKind::CrystalPatch, None, 9, 9)];
        map.slots[0].start = TilePos::new(-1, 0);
        let report = map.resize(3, 3);
        assert!(report.is_empty());
        assert_eq!(map.spawns.len(), 1);
        assert!(matches!(map.validate()[0], ScenarioProblem::StartOffMap { .. }));

        let mut symmetric = MapSettings {
            map_type: MapType::SkirmishDefault,
            width: 20,
            height: 16,
            ..Default::default()
        }
        .create()
        .unwrap();
        symmetric.resize(24, 16);
        assert_eq!(symmetric.first_asymmetry(), None);
        assert!(only_layouts_missing(&symmetric), "{:?}", symmetric.validate());
    }
}
