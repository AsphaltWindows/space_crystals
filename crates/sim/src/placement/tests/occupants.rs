//! The tile rules of what stands on a tile (rules 5 to 7): structures, map resources and ground units; the
//! sites the placing player has reserved (design change 0125); and `Mode::Known`, where what stands on a tile
//! the player never explored passes (design change 0120).

use super::*;

/// Rule 5: no tile under another structure, whoever owns it.
#[test]
fn tiles_under_structures() {
    let (map, area) = open(16, 16);
    let check = place(&map, &area, &[on(Generator, 5, 5)], Generator, 6, 6);
    assert_eq!(check.tiles, [(t(6, 6), Structure)]);
    let check = place(&map, &area, &[on(DeploymentCenter, 2, 2)], Barracks, 4, 5);
    assert_eq!(check.tiles, [(t(4, 5), Structure), (t(5, 5), Structure)]);
    assert!(
        place(&map, &area, &[on(Generator, 5, 5)], Generator, 7, 5).is_valid(),
        "side by side"
    );
}

/// Rule 6 (design change 0096): no tile under a map resource, but for a plate on a patch.
#[test]
fn tiles_under_resources() {
    let (map, area) = open(16, 16);
    let resources = [on(CrystalPatch, 3, 3), on(SupplyDeliveryStation, 8, 8)];
    assert_eq!(
        place(&map, &area, &resources, Generator, 2, 2).tiles,
        [(t(3, 3), Resource)]
    );
    assert_eq!(
        place(&map, &area, &resources, Barracks, 6, 9).tiles,
        [(t(8, 9), Resource)],
        "the station's corner"
    );
    let check = place(&map, &area, &resources, ExtractionPlate, 9, 9);
    assert_eq!(
        (check.tiles, check.whole),
        (vec![(t(9, 9), Resource)], vec![NotOnPatch])
    );
    assert!(
        place(&map, &area, &resources, ExtractionPlate, 3, 3).is_valid(),
        "a plate on the patch"
    );
}

/// Rule 7 (design change 0094): no tile a ground unit's box overlaps, whoever owns it; a box only touching the
/// footprint leaves it free.
#[test]
fn tiles_under_ground_units() {
    let (map, area) = open(16, 16);
    let at = |x: i32, y: i32| t(x, y).center();
    let centred = at(5, 5);
    let check = place(&map, &area, &[unit(Peacekeeper, centred.x, centred.y)], Generator, 5, 5);
    assert_eq!(check.tiles, [(t(5, 5), Unit)]);
    // On the corner of four tiles, all four.
    let check = place(&map, &area, &[unit(Guard, 384.0, 384.0)], Generator, 5, 5);
    assert_eq!(
        check.tiles,
        [t(5, 5), t(6, 5), t(5, 6), t(6, 6)].map(|tile| (tile, Unit))
    );
    // A Peacekeeper (24 × 24 su) touching the east edge (x = 448) and the south edge (y = 448).
    let touching = [unit(Peacekeeper, 460.0, 352.0), unit(Peacekeeper, 352.0, 460.0)];
    assert!(place(&map, &area, &touching, Generator, 5, 5).is_valid());
    let overlapping = unit(Peacekeeper, 459.0, 352.0);
    assert_eq!(
        place(&map, &area, &[overlapping], Generator, 5, 5).tiles,
        [(t(6, 5), Unit)]
    );
}

/// A pending Deploy's site, as a game reserves it (`Sim::pending_builds`): the Beacon it will be, on its footprint.
fn reserved_beacon(site: Placement) -> Occupant {
    Occupant {
        kind: ObjectKind::Beacon,
        pos: site.center(),
        placement: Some(site),
        tier: None,
    }
}

/// Design change 0125 (`core/entities.md > Lingering build ghosts`): the tiles of the sites the placing player has
/// reserved are `Reserved`, in their place among the other problems; a site only touching the footprint reserves none
/// of it.
#[test]
fn tiles_under_reserved_sites() {
    let (map, area) = open(16, 16);
    let site = Site {
        map: &map,
        area: &area,
        mode: Mode::Layout,
        method: Method::WorkerBuilt,
    };
    let beacon = |x, y| reserved_beacon(Placement::new(t(x, y), ObjectKind::Beacon.def().footprint));
    let reserved = [beacon(5, 5), beacon(2, 4)];
    let unturned = BuildingOrientation::default();
    let check = check_reserving(
        &site,
        [on(CrystalPatch, 4, 5)],
        &reserved,
        ObjectKind::Beacon,
        t(4, 4),
        unturned,
    );
    assert_eq!(check.tiles, [(t(4, 5), Resource), (t(5, 5), Reserved)]);
    assert!(check_reserving(&site, [], &reserved, ObjectKind::Beacon, t(7, 7), unturned).is_valid());
}

/// `Mode::Known` (plan decision T16; design change 0120: "The ghost judges by what the player knows"): what stands on
/// a tile the player never explored, and the ground, pass; its reservations and the map's edge never do. Player 1's
/// Generator at (4, 4) explored the Mountain at (7, 5), not the one at (20, 4) nor player 2's Barracks at (18, 10).
#[test]
fn never_explored_tiles_pass_but_reservations() {
    let mut rows = vec![".".repeat(24); 16];
    rows[5].replace_range(7..8, "M");
    rows[4].replace_range(20..21, "M");
    let scenario = ScenarioBuilder::new(&rows.join("\n"))
        .spawn(Generator, Some(PlayerId(1)), crate::SpawnAt::Tile(t(4, 4)))
        .spawn(Barracks, Some(PlayerId(2)), crate::SpawnAt::Tile(t(18, 10)))
        .build();
    let sim = Sim::new(&scenario);
    let area = BuildArea::default();
    let known = Site {
        map: sim.map(),
        area: &area,
        mode: Mode::Known {
            vision: sim.vision(),
            player: PlayerId(1),
        },
        method: Method::WorkerBuilt,
    };
    let unturned = BuildingOrientation::default();
    let reserved = [reserved_beacon(Placement::new(
        t(20, 5),
        ObjectKind::Beacon.def().footprint,
    ))];
    let beacon = |x, y| {
        let objects = sim.objects().map(Occupant::of);
        check_reserving(&known, objects, &reserved, ObjectKind::Beacon, t(x, y), unturned)
    };
    assert_eq!(sim.visibility(PlayerId(1), t(7, 5)), VisibilityState::Visible);
    assert_eq!(beacon(6, 5).tiles, [(t(7, 5), NotBuildable)]);
    assert!(beacon(20, 2).is_valid() && beacon(18, 10).is_valid(), "never explored");
    assert_eq!(beacon(20, 4).tiles, [(t(20, 5), Reserved), (t(21, 5), Reserved)]);
    assert_eq!(
        beacon(23, 15).tiles,
        [(t(24, 15), OffMap), (t(23, 16), OffMap), (t(24, 16), OffMap)]
    );
}
