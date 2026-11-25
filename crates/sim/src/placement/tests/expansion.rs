//! The expansion method (slice M4-F, `Method::Expansion`; `factions/syndicate/tunnels.md > AwaitingPlacement
//! (Expansion)`): wholly inside the Tunnel's area, on Drillable ground (design change 0239), with no flatness and no
//! sight, and every structure (0240), resource (0241), ground unit and lingering ghost (Placeholder: Q0080) in its
//! way. `crates/sim/tests/expansions/` checks it in games, the truth against what the owner knows included.

use super::*;

use ObjectKind::{Hideout, Tunnel};
use WholeProblem::OutsideTunnelArea;

/// The Tier 1 Tunnel Area of a Tunnel at (4, 4): (2, 2) to (9, 9).
fn area() -> TileRect {
    TileRect::new(t(2, 2), 8, 8)
}

/// [`check_reserving`] of a `kind` placed by the expansion method in [`area`] at `(x, y)`, unturned, in `mode`.
fn expand_in(
    map: &Map,
    mode: Mode<'_>,
    occupants: &[Occupant],
    reserved: &[Occupant],
    kind: ObjectKind,
    (x, y): (i32, i32),
) -> PlacementCheck {
    let build_area = BuildArea::default();
    let site = Site {
        map,
        area: &build_area,
        mode,
        method: Method::Expansion { area: area() },
    };
    let occupants = occupants.iter().copied();
    check_reserving(
        &site,
        occupants,
        reserved,
        kind,
        t(x, y),
        BuildingOrientation::default(),
    )
}

/// A Hideout by the expansion method in Layout mode, among the Tunnel at (4, 4) and `others`.
fn hideout(map: &Map, others: &[Occupant], x: i32, y: i32) -> PlacementCheck {
    let occupants: Vec<Occupant> = [on(Tunnel, 4, 4)].into_iter().chain(others.iter().copied()).collect();
    expand_in(map, Mode::Layout, &occupants, &[], Hideout, (x, y))
}

/// `> AwaitingPlacement (Expansion)`: "The expansion must fit entirely within the Tunnel Area": inside it, at its
/// corners, it is valid; half outside or wholly outside, it is refused as a whole, its tiles being fine. On the
/// Tunnel's own footprint it is under a structure ("so does the Tunnel's own footprint").
#[test]
fn an_expansion_fits_entirely_within_the_area() {
    let map = Map::new(16, 16);
    for (x, y) in [(2, 2), (8, 2), (2, 8), (8, 8), (6, 2)] {
        assert!(hideout(&map, &[], x, y).is_valid(), "({x}, {y})");
    }
    for (x, y) in [(9, 4), (1, 4), (4, 9), (4, 1), (10, 10), (12, 3)] {
        let check = hideout(&map, &[], x, y);
        assert_eq!(
            (check.tiles, check.whole),
            (vec![], vec![OutsideTunnelArea]),
            "({x}, {y})"
        );
    }
    let check = hideout(&map, &[], 3, 3);
    assert_eq!(check.tiles, [(t(4, 4), Structure)]);
    assert!(check.whole.is_empty());
    let on_it = hideout(&map, &[], 5, 5);
    assert_eq!(on_it.tiles.len(), 4);
    assert!(on_it.tiles.iter().all(|&(_, problem)| problem == Structure));
}

/// Design change 0239: "Every tile under it must be Drillable (... Plane, Rugged and Cliff yes; Mountain and Water
/// no ...)", in place of Buildable: on Rugged ground and on a Cliff it is valid; over Water or a Mountain inside the
/// area, those tiles are red (0239's Verification: "a ghost over Water inside the area is red"). It is the preset's
/// flag: a Plane-looking preset that isn't Drillable refuses it too.
#[test]
fn an_expansion_needs_drillable_ground() {
    let mut map = Map::from_ascii(
        "
        ................
        ................
        ..rc............
        ..rc............
        ........~M......
        ........~M......
        ................
        ................
        ................
        ................",
    )
    .unwrap();
    assert!(hideout(&map, &[], 2, 2).is_valid(), "Rugged and Cliff");
    let check = hideout(&map, &[], 8, 4);
    let red = [t(8, 4), t(9, 4), t(8, 5), t(9, 5)].map(|tile| (tile, NotDrillable));
    assert_eq!((check.tiles, check.whole), (red.to_vec(), vec![]));
    assert_eq!(
        hideout(&map, &[], 8, 5).tiles,
        [(t(8, 5), NotDrillable), (t(9, 5), NotDrillable)]
    );
    let plane = map.preset(crate::map::TilePresetId::PLANE).clone();
    let hard = map.add_preset(TilePreset {
        name: "Hard plane".into(),
        drillable: false,
        ..plane
    });
    map.set_tile(t(9, 9), Tile::new(hard));
    assert_eq!(hideout(&map, &[], 8, 8).tiles, [(t(9, 9), NotDrillable)]);
}

/// `core/map.md > Elevation` (design change 0005: "Syndicate underground expansions (inside a Tunnel Area) may be
/// placed on sloped tiles") and `core/entities.md > Direct Placement` ("Surface visibility is not relevant"): no tile
/// of an expansion is ever sloped or out of sight. The method has neither rule, whatever the kind and the mode: a
/// Generator placed so would need neither. Player 1 sees nothing of this map, which shows in a Direct Placement.
#[test]
fn an_expansion_needs_no_flat_ground_and_no_sight() {
    let ascii = "
        ................
        ................
        ................
        ................
        ................
        ................

        11111222222222222
        11111222222222222
        11111222222222222
        11111222222222222
        11111222222222222
        11111222222222222
        11111222222222222";
    let sim = Sim::new(&ScenarioBuilder::new(ascii).build());
    let map = sim.map();
    assert!(!map.is_flat(t(4, 3)), "a slope at x = 4");
    assert!(hideout(map, &[], 4, 2).is_valid(), "on the slope");
    assert!(expand_in(map, Mode::Layout, &[], &[], Generator, (4, 2)).is_valid());
    let unseen = Mode::Command {
        vision: sim.vision(),
        player: PlayerId(1),
    };
    assert_eq!(sim.visibility(PlayerId(1), t(6, 6)), VisibilityState::Unexplored);
    assert!(expand_in(map, unseen, &[on(Tunnel, 4, 4)], &[], Hideout, (6, 2)).is_valid());
    assert!(expand_in(map, unseen, &[], &[], Generator, (4, 2)).is_valid());
}

/// Every structure blocks an expansion, whichever its layer and whoever owns it (design change 0240: "no expansion can
/// be placed under any surface building, any player's"; "Own and neutral underground structures block it"), and so
/// do the resources (0241: "a Space Crystals patch or Supply Delivery Station above blocks an expansion") and, by
/// `core/entities.md > Direct Placement` read literally, a ground unit (Placeholder: Q0080 point 1).
#[test]
fn structures_resources_and_ground_units_block_an_expansion() {
    let map = Map::new(16, 16);
    let others = [
        on(Hideout, 2, 2),
        on(Barracks, 7, 2),
        on(CrystalPatch, 2, 8),
        on(SupplyDeliveryStation, 8, 8),
        unit(Peacekeeper, 6.5 * crate::SU_PER_GU, 8.5 * crate::SU_PER_GU),
    ];
    let problems = |x, y| hideout(&map, &others, x, y).tiles;
    assert_eq!(
        problems(3, 2),
        [(t(3, 2), Structure), (t(3, 3), Structure)],
        "an expansion"
    );
    assert_eq!(
        problems(6, 2),
        [(t(7, 2), Structure), (t(7, 3), Structure)],
        "a surface building"
    );
    assert_eq!(problems(2, 7), [(t(2, 8), Resource)], "a patch");
    assert_eq!(problems(8, 7), [(t(8, 8), Resource), (t(9, 8), Resource)], "a station");
    assert_eq!(problems(5, 8), [(t(6, 8), Unit)], "a ground unit");
    assert!(hideout(&map, &others, 4, 8).is_valid());
}

/// `core/entities.md > Lingering build ghosts`: the owner's pending worker builds reserve their sites against an
/// expansion too (Placeholder: Q0080 point 2).
#[test]
fn the_owners_lingering_ghosts_reserve_against_an_expansion() {
    let map = Map::new(16, 16);
    let ghost = [on(Tunnel, 8, 8)];
    let check = expand_in(&map, Mode::Layout, &[on(Tunnel, 4, 4)], &ghost, Hideout, (8, 8));
    let reserved = [t(8, 8), t(9, 8), t(8, 9), t(9, 9)].map(|tile| (tile, Reserved));
    assert_eq!(check.tiles, reserved);
}
