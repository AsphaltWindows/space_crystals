//! The rules of the footprint as a whole: a valid one on open ground, the build area to reach, a plate onto an
//! uncovered patch, only structures placed; and the fixed order of the problems.

use super::*;

#[test]
fn a_footprint_on_open_ground_is_valid() {
    let (map, area) = open(16, 16);
    for kind in [DeploymentCenter, Generator, Barracks, ExtractionFacility] {
        assert_eq!(
            place(&map, &area, &[], kind, 4, 4),
            PlacementCheck::default(),
            "{kind:?}"
        );
    }
    assert!(place(&map, &area, &[], Generator, 0, 14).is_valid(), "in the corner");
}

/// The build area (`factions/gdo/structures/README.md > GDOBuildArea`): a GDO building needs one tile in it, the
/// Deployment Center, which seeds it, none.
#[test]
fn a_footprint_must_reach_the_build_area() {
    let map = Map::new(64, 64);
    let area = dc_area(&map, 20, 20);
    assert_eq!(area.parts(), [TileRect::new(t(8, 8), 28, 28)]);
    for (x, y, inside) in [
        (35, 20, true),
        (36, 20, false),
        (35, 35, true),
        (36, 35, false),
        (6, 6, false),
        (7, 7, true),
    ] {
        let check = place(&map, &area, &[], Generator, x, y);
        assert_eq!(check.whole.is_empty(), inside, "({x}, {y})");
        assert!(check.tiles.is_empty());
    }
    assert_eq!(place(&map, &area, &[], Barracks, 40, 40).whole, [OutsideBuildArea]);
    assert!(place(&map, &BuildArea::default(), &[], DeploymentCenter, 40, 40).is_valid());
}

/// An Extraction Plate goes exactly onto a patch that has no plate yet, inside the area.
#[test]
fn plates_go_onto_uncovered_patches() {
    let map = Map::new(64, 64);
    let area = dc_area(&map, 20, 20);
    let patches = [on(CrystalPatch, 30, 30), on(CrystalPatch, 40, 40)];
    assert!(place(&map, &area, &patches, ExtractionPlate, 30, 30).is_valid());
    assert_eq!(
        place(&map, &area, &patches, ExtractionPlate, 31, 30).whole,
        [NotOnPatch]
    );
    assert_eq!(
        place(&map, &area, &patches, ExtractionPlate, 40, 40).whole,
        [OutsideBuildArea]
    );
    assert_eq!(
        place(&map, &area, &patches, ExtractionPlate, 41, 41).whole,
        [OutsideBuildArea, NotOnPatch]
    );
    // Covered: the plate on it is in the way.
    let covered = [on(CrystalPatch, 30, 30), on(ExtractionPlate, 30, 30)];
    let check = place(&map, &area, &covered, ExtractionPlate, 30, 30);
    assert_eq!((check.tiles, check.whole), (vec![(t(30, 30), Structure)], vec![]));
}

/// Units and map resources aren't placed by this check.
#[test]
fn only_structures_are_placed() {
    let (map, area) = open(16, 16);
    for kind in [Peacekeeper, Guard, CrystalPatch, SupplyDeliveryStation] {
        let check = place(&map, &area, &[], kind, 2, 2);
        assert_eq!((check.tiles, check.whole), (vec![], vec![NotAStructure]), "{kind:?}");
    }
}

/// Problems come tile by tile, row by row, each tile's in rule order, whatever the occupants' order.
#[test]
fn problems_come_in_a_fixed_order() {
    let map = Map::from_ascii(
        "
        ....
        .r..
        ....
        ....

        00000
        00000
        00000
        00010
        00000",
    )
    .unwrap();
    let area = dc_area(&map, 0, 0);
    let occupants = vec![
        unit(Peacekeeper, 100.0, 100.0),
        on(CrystalPatch, 2, 1),
        on(Generator, 2, 2),
        unit(Guard, 64.0, 140.0),
    ];
    let check = place(&map, &area, &occupants, DeploymentCenter, 0, 0);
    let expected = vec![
        (t(0, 1), Unit),
        (t(1, 1), NotBuildable),
        (t(1, 1), Unit),
        (t(2, 1), Resource),
        (t(0, 2), Unit),
        (t(1, 2), Unit),
        (t(2, 2), NotFlat),
        (t(2, 2), Structure),
        (t(3, 2), NotFlat),
        (t(3, 2), Structure),
        (t(2, 3), NotFlat),
        (t(2, 3), Structure),
        (t(3, 3), NotFlat),
        (t(3, 3), Structure),
    ];
    assert_eq!(check.tiles, expected);
    let reversed: Vec<Occupant> = occupants.into_iter().rev().collect();
    assert_eq!(place(&map, &area, &reversed, DeploymentCenter, 0, 0), check);
}
