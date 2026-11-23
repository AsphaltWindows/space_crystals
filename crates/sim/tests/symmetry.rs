//! Generated symmetry (`core/map_editor.md > Symmetry > Editing a symmetric map`): the locked part of a symmetric
//! map is the image of its editable part, for every mode: tiles, corner heights, pre-placed objects with their
//! values, and slots with their start positions, camera starts and Starting Layouts.

use sc_sim::{
    BuildingOrientation, CornerPos, LayoutObject, MapDef, MapSettings, MapType, ObjectKind, SlotId, Spawn, SpawnAt,
    SpawnSettings, Symmetry, Tile, TilePos, TilePresetId, Vec2,
};

/// A map of `symmetry` with edits all over its editable part: varied tiles and heights, patches with their own
/// amounts, a station with its own values, every slot's camera start and a layout object per faction.
fn edited(symmetry: Symmetry, width: u32, height: u32) -> MapDef {
    let slots = MapSettings::slot_counts(symmetry)[1];
    let mut map = MapSettings {
        map_type: MapType::Custom,
        width,
        height,
        slots,
        symmetry,
        name: format!("{symmetry:?}"),
        description: String::new(),
    }
    .create()
    .unwrap();
    let region = map.region();
    let presets = [TilePresetId::RUGGED, TilePresetId::WATER, TilePresetId::CLIFF];
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let t = TilePos::new(x, y);
            if region.tile(t) && (x * 7 + y * 3) % 5 == 0 {
                map.terrain.set_tile(t, Tile::new(presets[(x + y) as usize % 3]));
            }
        }
    }
    for y in 0..=height as i32 {
        for x in 0..=width as i32 {
            let c = CornerPos::new(x, y);
            if region.corner(c) && (x + 2 * y) % 3 == 0 {
                map.terrain.set_corner(c, ((x * y) % 5) as u8);
            }
        }
    }
    let (min, max) = region.inner_rect();
    let mut amount = 100;
    for y in (min.y..max.y).step_by(3) {
        for x in (min.x..max.x).step_by(4) {
            let mut patch = Spawn::new(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(x, y)));
            patch.settings = Some(SpawnSettings::CrystalPatch {
                starting_amount: amount,
            });
            amount += 1;
            map.spawns.push(patch);
        }
    }
    let mut station = Spawn::new(
        ObjectKind::SupplyDeliveryStation,
        None,
        SpawnAt::Tile(TilePos::new(1, max.y - 3)),
    );
    station.settings = Some(SpawnSettings::SupplyDeliveryStation {
        starting_supplies: 5,
        delivery_size: 9,
        delivery_interval: 77,
    });
    map.spawns.push(station);
    for id in map.slot_ids().collect::<Vec<_>>() {
        if map.is_slot_editable(id) {
            let slot = map.slot_mut(id).unwrap();
            slot.camera_start = Some(slot.start.center() + Vec2::new(10.0, 20.0));
            for (i, faction) in sc_sim::Faction::ALL.iter().enumerate() {
                slot.layouts.get_mut(*faction).push(LayoutObject {
                    kind: ObjectKind::SupplyDeliveryStation,
                    at: SpawnAt::Tile(TilePos::new(i as i32 + 1, 2)),
                    orientation: BuildingOrientation::default(),
                });
            }
        }
    }
    map.regenerate();
    map
}

/// Every mode, on maps with even and odd sides.
fn cases() -> Vec<(Symmetry, u32, u32)> {
    let mut cases = Vec::new();
    for s in Symmetry::ALL {
        for (w, h) in [(24, 18), (23, 17), (20, 20), (21, 21)] {
            if s.fits(w, h) {
                cases.push((s, w, h));
            }
        }
    }
    cases
}

#[test]
fn the_generated_part_is_the_image_of_the_editable_part() {
    for (symmetry, w, h) in cases() {
        let map = edited(symmetry, w, h);
        let label = format!("{symmetry:?} {w}x{h}");
        let region = map.region();
        let transforms = map.transforms();
        assert_eq!(transforms.len(), symmetry.images().len(), "{label}");
        assert_eq!(map.first_asymmetry(), None, "{label}");
        // Tiles and corners: the image of every editable one equals it.
        for y in 0..=h as i32 {
            for x in 0..=w as i32 {
                let (t, c) = (TilePos::new(x, y), CornerPos::new(x, y));
                for m in &transforms {
                    if region.tile(t) && map.terrain.in_bounds(t) {
                        assert_eq!(map.terrain.tile(m.tile(t)), map.terrain.tile(t), "{label} tile {t:?}");
                    }
                    if region.corner(c) {
                        assert_eq!(
                            map.terrain.corner(m.corner(c)),
                            map.terrain.corner(c),
                            "{label} corner {c:?}"
                        );
                    }
                }
            }
        }
        // Objects: the editable ones first, then their images, each with the same values.
        let editable: Vec<&Spawn> = map.spawns.iter().filter(|s| map.is_spawn_editable(s)).collect();
        assert_eq!(map.spawns.len(), editable.len() * transforms.len(), "{label}");
        for s in &editable {
            for m in &transforms[1..] {
                let SpawnAt::Tile(o) = s.at else { unreachable!() };
                let (w2, h2) = match s.kind {
                    ObjectKind::CrystalPatch => (1, 1),
                    ObjectKind::SupplyDeliveryStation => (2, 2),
                };
                let (image, _, _) = m.footprint(o, w2, h2);
                let found = map
                    .spawns
                    .iter()
                    .find(|i| i.at == SpawnAt::Tile(image) && i.kind == s.kind);
                assert_eq!(found.map(|i| i.settings), Some(s.settings), "{label} image of {s:?}");
            }
        }
        // Slots: each editable slot's images, in order, with every faction's layout.
        let n = transforms.len();
        for id in map.slot_ids() {
            let index = usize::from(id.0) - 1;
            let source = &map.slots[index - index % n];
            let m = transforms[index % n];
            let slot = map.slot(id).unwrap();
            assert_eq!(slot.start, m.tile(source.start), "{label} slot {id:?}");
            assert_eq!(slot.camera_start, source.camera_start.map(|c| m.point(c)), "{label}");
            for (faction, objects) in slot.layouts.iter() {
                let expected: Vec<SpawnAt> = source
                    .layouts
                    .get(faction)
                    .iter()
                    .map(|o| {
                        let SpawnAt::Tile(t) = o.at else { unreachable!() };
                        SpawnAt::Tile(m.footprint(t, 2, 2).0)
                    })
                    .collect();
                let got: Vec<SpawnAt> = objects.iter().map(|o| o.at).collect();
                assert_eq!(got, expected, "{label} {id:?} {faction:?}");
            }
            assert_eq!(map.image_slot(map.source_slot(id), index % n), id, "{label}");
            assert_eq!(map.is_slot_editable(id), index % n == 0);
        }
        // Validation finds the map symmetric, its slots in whole sets, and no object overlapping its images. (The
        // varied heights make steep tiles and slope patches, and the layouts' resources would be owned.)
        let problems: Vec<_> = map
            .validate()
            .into_iter()
            .filter(|p| {
                use sc_sim::ScenarioProblem as P;
                matches!(
                    p,
                    P::NotSymmetric(_)
                        | P::OverlappingFootprints { .. }
                        | P::SlotsNotInImageSets { .. }
                        | P::StartOffMap { .. }
                )
            })
            .collect();
        assert_eq!(problems, [], "{label}");
    }
}

/// A hand edit in the locked part is flagged, and regenerating undoes it from the editable part.
#[test]
fn a_locked_edit_is_flagged_and_regenerated() {
    for (symmetry, w, h) in cases() {
        if symmetry == Symmetry::None {
            continue;
        }
        let map = edited(symmetry, w, h);
        let region = map.region();
        let locked = (0..h as i32)
            .flat_map(|y| (0..w as i32).map(move |x| TilePos::new(x, y)))
            .find(|t| !region.tile(*t))
            .unwrap();
        let mut changed = map.clone();
        let current = changed.terrain.tile(locked).unwrap().preset;
        let other = if current == TilePresetId::MOUNTAIN {
            TilePresetId::PLANE
        } else {
            TilePresetId::MOUNTAIN
        };
        changed.terrain.set_tile(locked, Tile::new(other));
        assert_eq!(
            changed.first_asymmetry(),
            Some(sc_sim::scenario::Asymmetry::Tile(locked)),
            "{symmetry:?}"
        );
        assert!(
            changed
                .validate()
                .iter()
                .any(|p| matches!(p, sc_sim::ScenarioProblem::NotSymmetric(_)))
        );
        changed.regenerate();
        assert_eq!(changed, map, "{symmetry:?}");

        // A locked object is dropped; the editable part's images come back.
        let mut extra = map.clone();
        let mut patch = Spawn::new(ObjectKind::CrystalPatch, None, SpawnAt::Tile(locked));
        patch.settings = Some(SpawnSettings::CrystalPatch { starting_amount: 5 });
        extra.spawns.push(patch);
        assert_eq!(extra.first_asymmetry(), Some(sc_sim::scenario::Asymmetry::Spawns));
        extra.regenerate();
        assert_eq!(extra, map);
    }
}

/// An object that is its own image stays one object; the images of a slot's objects belong to the slot's images.
#[test]
fn objects_on_the_center_stay_single_and_owners_follow_their_slots() {
    let mut map = MapSettings {
        map_type: MapType::Custom,
        width: 20,
        height: 20,
        slots: 4,
        symmetry: Symmetry::Rotate90,
        ..Default::default()
    }
    .create()
    .unwrap();
    let center = Spawn::new(
        ObjectKind::SupplyDeliveryStation,
        None,
        SpawnAt::Tile(TilePos::new(9, 9)),
    );
    map.spawns.push(center.clone());
    map.regenerate();
    assert_eq!(map.spawns, [center]);

    // Owned by slot 1 (invalid for a resource, but the owners still follow the slots): the image under the n-th
    // transform belongs to slot 1 + n.
    let mut owned = Spawn::new(
        ObjectKind::CrystalPatch,
        Some(SlotId(1)),
        SpawnAt::Tile(TilePos::new(2, 3)),
    );
    owned.settings = Some(SpawnSettings::CrystalPatch { starting_amount: 9 });
    map.spawns.push(owned);
    map.regenerate();
    let owners: Vec<Option<u8>> = map.spawns[1..].iter().map(|s| s.owner.map(|o| o.0)).collect();
    assert_eq!(owners, [Some(1), Some(2), Some(3), Some(4)]);
    for (s, slot) in map.spawns[1..].iter().zip(map.slots.iter()) {
        // Each image is as far from its slot's start as the original is from slot 1's.
        let SpawnAt::Tile(t) = s.at else { unreachable!() };
        let d = (t.x - slot.start.x).abs() + (t.y - slot.start.y).abs();
        let SpawnAt::Tile(o) = map.spawns[1].at else {
            unreachable!()
        };
        assert_eq!(
            d,
            (o.x - map.slots[0].start.x).abs() + (o.y - map.slots[0].start.y).abs()
        );
    }
    // Owned by slot 2, an image slot: the images go round from there.
    map.spawns[1].owner = Some(SlotId(2));
    map.regenerate();
    let owners: Vec<Option<u8>> = map.spawns[1..].iter().map(|s| s.owner.map(|o| o.0)).collect();
    assert_eq!(owners, [Some(2), Some(3), Some(4), Some(1)]);
}

/// Building orientations turn and flip with the map (`core/map_editor.md > Symmetry > Editing a symmetric map`).
/// Only structures have one; there are none yet, so this checks the transform on a resource's footprint and the
/// rule that other objects keep theirs.
#[test]
fn only_structures_turn_with_the_map() {
    let m = Symmetry::Rotate90.transforms(20, 20).unwrap()[1];
    let station = ObjectKind::SupplyDeliveryStation;
    let (at, orientation) = sc_sim::scenario::image_location(
        station,
        SpawnAt::Tile(TilePos::new(2, 3)),
        BuildingOrientation::default(),
        &m,
    );
    assert_eq!(orientation, BuildingOrientation::default());
    assert_eq!(at, SpawnAt::Tile(m.footprint(TilePos::new(2, 3), 2, 2).0));
    let point = Vec2::new(100.0, 70.0);
    let (at, _) = sc_sim::scenario::image_location(
        ObjectKind::CrystalPatch,
        SpawnAt::Point(point),
        BuildingOrientation::default(),
        &m,
    );
    assert_eq!(at, SpawnAt::Point(m.point(point)));
}
