//! Placement validation in games (slice M2-C; `core/entities.md > Placement Validation`): `Sim::placement_check` and
//! `Sim::build_area` on scenarios whose objects debug commands add, move and remove, and Starting Layouts checked by
//! `placement::check_layout`. One test per design change and its verification points (0004, 0091 to 0097); the
//! rules one by one, on the pure check, are in `src/placement/tests/`. `TestSim` checks the invariants after every
//! tick.

use sc_sim::placement::{Occupant, check_layout};
use sc_sim::scenario::LayoutObject;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{
    BuildArea, BuildingOrientation, DebugCommand, Faction, Map, ObjectId, ObjectKind, Placement, PlacementCheck,
    PlayerId, Rotation, ScenarioProblem, Sim, SlotId, SpawnAt, SpawnProblem, TilePos, Vec2,
};

use ObjectKind::{
    Barracks, CrystalPatch, DeploymentCenter, ExtractionFacility, ExtractionPlate, Generator, Guard, Peacekeeper,
    SupplyDeliveryStation,
};
use sc_sim::TileProblem::{NotFlat, NotVisible, Resource, Unit};
use sc_sim::WholeProblem::{NotOnPatch, OutsideBuildArea};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn flat(w: usize, h: usize) -> String {
    vec![".".repeat(w); h].join("\n")
}

fn at(x: i32, y: i32) -> TilePos {
    TilePos::new(x, y)
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(at(x, y))
}

fn centre(x: i32, y: i32) -> Vec2 {
    at(x, y).center()
}

/// Player 1's Deployment Center (object 1) at (20, 20) on a flat 64 × 64 map: its build area is x and y 8 to 35, it
/// sees 6 tiles around its centre, (22, 22).
fn base() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(64, 64)).spawn(DeploymentCenter, Some(P1), tile(20, 20))
}

/// `player`'s check of a `kind` with its top-left tile at `(x, y)`, unturned.
fn check(sim: &Sim, player: PlayerId, kind: ObjectKind, x: i32, y: i32) -> PlacementCheck {
    sim.placement_check(player, kind, at(x, y), BuildingOrientation::default())
}

fn spawn(kind: ObjectKind, owner: PlayerId, x: i32, y: i32) -> DebugCommand {
    DebugCommand::Spawn {
        kind,
        owner: Some(owner),
        at: tile(x, y),
        orientation: BuildingOrientation::default(),
    }
}

fn remove(id: u32) -> DebugCommand {
    DebugCommand::Remove { id: ObjectId(id) }
}

/// The area that `buildings` give on `sim`'s map, each a kind with its top-left tile at `(x, y)`, unturned.
fn area_of(sim: &Sim, buildings: &[(ObjectKind, i32, i32)]) -> BuildArea {
    let placed: Vec<(ObjectKind, Placement)> = buildings
        .iter()
        .map(|&(kind, x, y)| (kind, Placement::new(at(x, y), kind.def().footprint)))
        .collect();
    BuildArea::of(sim.map(), placed.iter().map(|(kind, p)| (*kind, p)))
}

/// Design change 0091: the area is square, so the starting Deployment Center's is exactly 28 × 28 tiles centred on
/// it, its corners included.
#[test]
fn the_starting_deployment_center_s_area_is_28_by_28_tiles_around_it() {
    let t = base().start();
    let sim = t.sim();
    let area = sim.build_area(P1);
    let tiles: Vec<TilePos> = area.tiles().collect();
    assert_eq!(tiles.len(), 28 * 28);
    assert!(tiles.iter().all(|p| (8..36).contains(&p.x) && (8..36).contains(&p.y)));
    assert!(sim.build_area(P2).is_empty(), "player 2 owns no building");
    // A Generator reaching the corner tile (35, 35) is in it; one more tile diagonally, it isn't.
    assert!(!check(sim, P1, Generator, 35, 35).whole.contains(&OutsideBuildArea));
    assert_eq!(check(sim, P1, Generator, 36, 36).whole, [OutsideBuildArea]);
    assert!(
        !check(sim, P1, Generator, 7, 7).whole.contains(&OutsideBuildArea),
        "the corner tile (8, 8)"
    );
    assert_eq!(check(sim, P1, Generator, 6, 6).whole, [OutsideBuildArea]);
    // Player 2 has no area at all.
    assert_eq!(check(sim, P2, Generator, 24, 24).whole, [OutsideBuildArea]);
}

/// Design change 0092, verification point 1: a building at the edge of the area extends it; once it is lost, the
/// area shrinks back, and the buildings in the part that was lost stay.
#[test]
fn a_building_at_the_edge_extends_the_area_until_it_is_lost() {
    let mut t = base().start();
    let dc_area = t.sim().build_area(P1);
    // A Barracks at (36, 28) (x 36 to 38, y 28 and 29) is one column beyond the Deployment Center's area.
    assert_eq!(check(t.sim(), P1, Barracks, 36, 28).whole, [OutsideBuildArea]);
    // Generator 2 at (34, 30), at the edge: its part (x 33 to 36, y 29 to 32) adds 4 tiles of column 36.
    t.debug_at(1, spawn(Generator, P1, 34, 30)).run_until(1);
    let area = t.sim().build_area(P1);
    assert_eq!(area.tiles().count(), 28 * 28 + 4);
    assert!(area.contains(at(36, 29)) && !dc_area.contains(at(36, 29)));
    assert!(!check(t.sim(), P1, Barracks, 36, 28).whole.contains(&OutsideBuildArea));
    // Generator 3 at (36, 31), in the new part; then Generator 2 is lost.
    t.debug_at(2, spawn(Generator, P1, 36, 31)).run_until(2);
    t.debug_at(3, remove(2)).run_until(3);
    let sim = t.sim();
    assert!(sim.object(ObjectId(3)).is_some(), "the building in the lost part stays");
    let area = sim.build_area(P1);
    assert_eq!(area, area_of(sim, &[(DeploymentCenter, 20, 20), (Generator, 36, 31)]));
    assert!(!area.contains(at(36, 29)), "Generator 2 alone covered it");
    assert_eq!(check(sim, P1, Barracks, 36, 28).whole, [OutsideBuildArea]);
    // Without Generator 3 too, the area is the Deployment Center's again.
    t.debug_at(4, remove(3)).run_until(4);
    assert_eq!(t.sim().build_area(P1), dc_area);
}

/// Design change 0092, verification point 2: once the Deployment Center is lost, the other buildings' areas still
/// count. The Extraction Facility can place plates inside its own area and inside another building's.
#[test]
fn without_a_deployment_center_the_other_buildings_areas_still_count() {
    let mut t = base()
        // Object 2: its part is x and y 32 to 38. Object 3: x 9 to 12, y 29 to 32.
        .spawn(ExtractionFacility, Some(P1), tile(34, 34))
        .spawn(Generator, Some(P1), tile(10, 30))
        // Outside the Deployment Center's area, inside the Extraction Facility's, 2.8 tiles from its centre.
        .spawn(CrystalPatch, None, tile(37, 37))
        // Inside the Deployment Center's area and the Generator's, 2.1 tiles from its centre.
        .spawn(CrystalPatch, None, tile(12, 32))
        // Inside the Deployment Center's area only.
        .spawn(CrystalPatch, None, tile(26, 12))
        .start();
    assert!(check(t.sim(), P1, ExtractionPlate, 37, 37).is_valid());
    assert!(check(t.sim(), P1, ExtractionPlate, 12, 32).is_valid());
    assert!(check(t.sim(), P1, ExtractionPlate, 26, 12).whole.is_empty());
    t.debug_at(1, remove(1)).run_until(1);
    let sim = t.sim();
    assert_eq!(
        sim.build_area(P1),
        area_of(sim, &[(ExtractionFacility, 34, 34), (Generator, 10, 30)])
    );
    assert!(check(sim, P1, ExtractionPlate, 37, 37).is_valid(), "in its own area");
    assert!(check(sim, P1, ExtractionPlate, 12, 32).is_valid(), "in the Generator's");
    assert_eq!(check(sim, P1, ExtractionPlate, 26, 12).whole, [OutsideBuildArea]);
}

/// Design change 0094: a ground unit of any player on a footprint tile makes that tile invalid; units are never
/// pushed aside, so the placement waits until they move away. (Air units never block, but none exists yet.)
#[test]
fn ground_units_of_any_player_block_until_they_move_away() {
    let mut t = base().start();
    t.without_auto_targeting();
    // Object 2, player 1's own Peacekeeper, and object 3, player 2's Guard, both in the Deployment Center's sight.
    t.spawn_unit_at(1, Peacekeeper, P1, centre(24, 24));
    t.spawn_unit_at(1, Guard, P2, centre(17, 21)).run_until(1);
    assert_eq!(check(t.sim(), P1, Generator, 24, 24).tiles, [(at(24, 24), Unit)]);
    assert!(check(t.sim(), P1, Generator, 24, 24).whole.is_empty());
    assert_eq!(check(t.sim(), P1, Generator, 16, 20).tiles, [(at(17, 21), Unit)]);
    let away = |id, pos| DebugCommand::SetPosition { id: ObjectId(id), pos };
    t.debug_at(2, away(2, centre(40, 40)));
    t.debug_at(2, away(3, centre(50, 10))).run_until(2);
    assert!(check(t.sim(), P1, Generator, 24, 24).is_valid());
    assert!(check(t.sim(), P1, Generator, 16, 20).is_valid());
}

/// Design change 0095, verification point 2: a patch inside the build area but out of sight can't take a plate:
/// its tile isn't Visible. (Point 1, the test map's patches in sight at the start, belongs to its Starting Layout.)
#[test]
fn a_plate_needs_its_patch_in_sight() {
    let t = base()
        .spawn(CrystalPatch, None, tile(32, 22))
        .spawn(CrystalPatch, None, tile(26, 22))
        .start();
    let sim = t.sim();
    // (32, 22) is inside the area (x < 36), 10.5 tiles from the Deployment Center's centre, beyond its sight of 6.
    let far = check(sim, P1, ExtractionPlate, 32, 22);
    assert_eq!((far.tiles, far.whole), (vec![(at(32, 22), NotVisible)], vec![]));
    assert!(check(sim, P1, ExtractionPlate, 26, 22).is_valid(), "4.5 tiles away");
}

/// Design change 0096: no building on a resource, except a plate on a patch.
#[test]
fn no_building_goes_on_a_resource_but_a_plate_on_a_patch() {
    let t = base()
        .spawn(CrystalPatch, None, tile(25, 21))
        .spawn(SupplyDeliveryStation, None, tile(17, 21))
        .start();
    let sim = t.sim();
    // 1. A Generator over the patch.
    assert_eq!(check(sim, P1, Generator, 25, 21).tiles, [(at(25, 21), Resource)]);
    // 2. A Generator over the station (x and y 17 and 18 from (17, 21)).
    let tiles = [at(17, 21), at(17, 22)].map(|tile| (tile, Resource));
    assert_eq!(check(sim, P1, Generator, 16, 21).tiles, tiles);
    let plate = check(sim, P1, ExtractionPlate, 17, 21);
    assert_eq!(
        (plate.tiles, plate.whole),
        (vec![(at(17, 21), Resource)], vec![NotOnPatch])
    );
    // 3. A plate on the patch.
    assert!(check(sim, P1, ExtractionPlate, 25, 21).is_valid());
}

/// Design change 0004: a building with one column of its footprint on a slope is refused there; on a flat plateau
/// it isn't. A 40 × 40 map: a plateau at level 1 up to x = 24, a plain at level 0 from x = 26, the column x = 25
/// sloped between them; player 1's Deployment Center on the plateau at (18, 14), seeing 6 tiles around (20, 16).
#[test]
fn structures_need_flat_ground() {
    let tiles = flat(40, 40);
    let row: String = (0..=40).map(|x| if x <= 25 { '1' } else { '0' }).collect();
    let heights = vec![row; 41].join("\n");
    let t = ScenarioBuilder::new(&format!("{tiles}\n\n{heights}"))
        .spawn(DeploymentCenter, Some(P1), tile(18, 14))
        .start();
    let sim = t.sim();
    let sloped: Vec<_> = check(sim, P1, Generator, 24, 15)
        .tiles
        .into_iter()
        .filter(|&(_, problem)| problem == NotFlat)
        .collect();
    assert_eq!(sloped, [(at(25, 15), NotFlat), (at(25, 16), NotFlat)]);
    assert!(check(sim, P1, Generator, 22, 15).is_valid(), "on the plateau");
    let plain = check(sim, P1, Generator, 26, 15);
    assert!(plain.tiles.iter().all(|&(_, problem)| problem != NotFlat), "{plain:?}");
}

/// The check changes nothing, gives the same answer every time, and the same in a loaded save: the area and the
/// rules come from the state alone.
#[test]
fn the_check_changes_nothing_and_a_loaded_save_answers_alike() {
    let mut t = base()
        .spawn(CrystalPatch, None, tile(25, 21))
        .spawn(ExtractionFacility, Some(P1), tile(26, 26))
        .spawn(Generator, Some(P2), tile(40, 40))
        .start();
    t.without_auto_targeting();
    t.spawn_unit_at(1, Peacekeeper, P1, centre(24, 24));
    t.spawn_unit_at(1, Guard, P2, centre(30, 22)).run_until(5);
    let every_check = |sim: &Sim| -> Vec<PlacementCheck> {
        let mut checks = Vec::new();
        for player in [P1, P2, PlayerId(7)] {
            for kind in [
                DeploymentCenter,
                Generator,
                Barracks,
                ExtractionFacility,
                ExtractionPlate,
                Peacekeeper,
            ] {
                for rotation in [Rotation::R0, Rotation::R90] {
                    let orientation = BuildingOrientation {
                        rotation,
                        ..Default::default()
                    };
                    for y in (14..44).step_by(3) {
                        for x in (14..44).step_by(3) {
                            checks.push(sim.placement_check(player, kind, at(x, y), orientation));
                        }
                    }
                }
            }
        }
        checks
    };
    let hash = t.sim().state_hash();
    let checks = every_check(t.sim());
    assert!(checks.iter().any(PlacementCheck::is_valid) && !checks.iter().all(PlacementCheck::is_valid));
    assert_eq!(every_check(t.sim()), checks);
    assert_eq!(t.sim().state_hash(), hash, "checking changes nothing");
    let json = serde_json::to_string(t.sim()).unwrap();
    let loaded: Sim = serde_json::from_str(&json).unwrap();
    assert_eq!(every_check(&loaded), checks);
    for player in [P1, P2] {
        assert_eq!(loaded.build_area(player), t.sim().build_area(player));
    }
}

/// A tile-based object of a Starting Layout, unturned, with its top-left tile at `(x, y)`.
fn laid(kind: ObjectKind, x: i32, y: i32) -> Occupant {
    let placement = Placement::new(at(x, y), kind.def().footprint);
    Occupant {
        kind,
        pos: placement.center(),
        placement: Some(placement),
        tier: None,
    }
}

/// A unit of a Starting Layout, centred on `pos`.
fn unit_at(kind: ObjectKind, pos: Vec2) -> Occupant {
    Occupant {
        kind,
        pos,
        placement: None,
        tier: None,
    }
}

/// Design change 0097: a Starting Layout follows the normal placement rules, minus sight. Its Extraction Facility
/// and plates must reach the build area of its own Deployment Center; nothing needs to be seen; a starting unit
/// can't stand on impassable tiles.
#[test]
fn starting_layouts_follow_the_rules_without_sight() {
    let map = Map::new(64, 64);
    // The map's pre-placed patches: one inside the area of a Deployment Center at (20, 20), one outside.
    let patches = [laid(CrystalPatch, 30, 20), laid(CrystalPatch, 45, 20)];
    let layout = [
        laid(DeploymentCenter, 20, 20),
        laid(ExtractionFacility, 40, 20),
        laid(ExtractionFacility, 34, 24),
        laid(ExtractionPlate, 30, 20),
        laid(ExtractionPlate, 45, 20),
    ];
    let checked = |index| check_layout(&map, patches, &layout, index).expect("a structure of the layout");
    // 4. The Deployment Center seeds the area and needs no sight: it sees no patch and is accepted.
    assert!(checked(0).is_valid());
    // 1. An Extraction Facility outside the area is refused; 2. inside it (x 34 and 35), it is accepted.
    assert_eq!(checked(1).whole, [OutsideBuildArea]);
    assert!(checked(2).is_valid());
    // Plates: on a patch inside the area, no sight needed; outside it, refused. The other Extraction Facility
    // doesn't extend the layout's area.
    assert!(checked(3).is_valid());
    assert_eq!(checked(4).whole, [OutsideBuildArea]);
    // A footprint over a starting unit is refused, as in a game.
    let with_unit = [
        laid(DeploymentCenter, 20, 20),
        unit_at(Peacekeeper, centre(30, 30)),
        laid(Generator, 30, 30),
    ];
    let check = check_layout(&map, [], &with_unit, 2).unwrap();
    assert_eq!(check.tiles, [(at(30, 30), Unit)]);
    assert_eq!(check_layout(&map, [], &with_unit, 1), None, "units have their own rule");
    assert_eq!(check_layout(&map, [], &with_unit, 3), None);
    // 3. A starting unit on a cliff is refused: the unit-start rule of scenario validation.
    let mut scenario = ScenarioBuilder::new(
        "
        ....
        .c..
        ....",
    )
    .build();
    *scenario.map.slots[0].layouts.get_mut(Faction::Gdo) = vec![LayoutObject {
        kind: Peacekeeper,
        at: SpawnAt::Point(centre(1, 1)),
        orientation: BuildingOrientation::default(),
    }];
    let refused = ScenarioProblem::InvalidLayoutObject {
        slot: SlotId(1),
        faction: Faction::Gdo,
        index: 0,
        kind: Peacekeeper,
        problem: SpawnProblem::Blocked,
    };
    assert_eq!(scenario.validate(), [refused]);
}
