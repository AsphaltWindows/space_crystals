//! The placement ghost and the build area overlay: the ghost green or red tile by tile by the placement rules
//! (design change 0093), R and F turning it and its side letters (0079), and the overlay following the build
//! area as Generators come and go, and as the player explores it.

use sc_sim::VisibilityState;
use space_crystals::fog::{FogOfWar, Sight};
use space_crystals::placement::overlay::area_mesh;

use super::*;

/// How many times the overlay's mesh was built.
fn overlay_builds(app: &App) -> u32 {
    app.world().resource::<BuildAreaOverlay>().builds
}

/// Design change 0093: the overlay shows the build area only, and the ghost shows each of its tiles green or red as
/// the placement check has it now: water (not Buildable), a slope, a unit, a structure, a resource, a tile nobody
/// sees, each red where it is and the other tiles green; a footprint with no tile in the build area is red
/// throughout. A click on a red ghost places nothing and says why; placement goes on.
#[test]
fn the_ghost_is_green_or_red_tile_by_tile_by_the_placement_rules() {
    use TileProblem::{NotBuildable, NotFlat, NotVisible, Resource, Structure, Unit};
    let mut app = construction_game(1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyQ, GENERATOR);
    let at = TilePos::new;
    let tiles = |tiles: Vec<(TilePos, TileProblem)>| PlacementCheck { tiles, whole: vec![] };
    let every = |origin: TilePos, problem| {
        let all = [(0, 0), (1, 0), (0, 1), (1, 1)];
        tiles(all.map(|(dx, dy)| (at(origin.x + dx, origin.y + dy), problem)).to_vec())
    };
    let outside = PlacementCheck {
        tiles: vec![],
        whole: vec![WholeProblem::OutsideBuildArea],
    };
    let cases = [
        (OPEN, PlacementCheck::default()),
        (at(18, 10), tiles(vec![(WATER, NotBuildable)])),
        (at(24, 10), every(at(24, 10), NotFlat)),
        (at(25, 12), tiles(vec![(at(26, 13), Unit)])),
        (at(23, 14), tiles(vec![(at(23, 14), Structure)])),
        (FREE_PATCH, tiles(vec![(FREE_PATCH, Resource)])),
        (at(30, 8), every(at(30, 8), NotVisible)),
        (at(36, 14), outside),
    ];
    for (origin, check) in cases {
        hover(&mut app, middle(origin, (2, 2)));
        let shown = ghost(&app).expect("the ghost");
        assert_eq!((shown.origin, &shown.check), (origin, &check));
        for t in shown.tiles() {
            let red = !check.whole.is_empty() || check.tiles.iter().any(|(bad, _)| *bad == t);
            assert_eq!(shown.tile_is_green(t), !red, "{t:?} of {origin:?}");
        }
        assert_eq!(shown.is_valid(), check == PlacementCheck::default());
    }
    // The overlay is the area, whatever the ghost is over.
    assert_eq!(overlay(&mut app), (Some(build_area(&app)), true));

    // A click on the red ghost (still outside the area) sends nothing, and the player hears why.
    click(&mut app, middle(at(36, 14), (2, 2)), &[]);
    assert_eq!(constructions(&app).len(), 1, "the Construct only");
    assert_eq!(placing(&app), Some(Placing::new(ObjectId(DC), GENERATOR)));
    assert_eq!(feedback(&app), Some("Outside the build area"));
    click(&mut app, middle(at(25, 12), (2, 2)), &[]);
    assert_eq!(feedback(&app), Some("Blocked by a unit"));
    assert_eq!(constructions(&app).len(), 1);
    assert_eq!(selected(&app), [DC], "the click on the Peacekeeper selected nothing");
}

/// `> AwaitingPlacement`: "R rotates the ghost 90 degrees clockwise, Shift+R rotates 90 degrees counter-clockwise.
/// Non-square buildings swap footprint dimensions when rotated to 90 or 270. F flips the ghost horizontally, Shift+F
/// flips vertically. Side labels (A/B/C/D per SymmetryType) are displayed on the ghost and update with
/// rotation/flipping" (design change 0079): the Barracks, ABAC (top A, right B, bottom A, left C). R and F are the
/// placement keys here, not command slots, and Shift with them turns the other way: it doesn't queue anything (risk
/// R11). The Barracks goes on the map as the ghost was turned.
#[test]
fn r_and_f_turn_the_barracks_ghost_and_its_side_letters() {
    use SideLetter::{A, B, C};
    let mut app = construction_game(1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyW, BARRACKS_KIND);
    hover(&mut app, middle(OPEN, (3, 2)));
    let shown = ghost(&app).expect("the ghost");
    assert_eq!(
        (shown.origin, shown.size, shown.sides),
        (OPEN, (3, 2), Some([A, B, A, C]))
    );
    assert_eq!(ghost_labels(&mut app), labels(&["A", "B", "A", "C", "BR"]));

    let turned = |app: &App| placing(app).map(|p| p.orientation);
    let size = |app: &App| ghost(app).map(|g| g.size);
    tap(&mut app, &[], KeyR);
    assert_eq!(turned(&app).map(|o| o.rotation), Some(Rotation::R90));
    assert_eq!((size(&app), letters(&app)), (Some((2, 3)), Some([C, A, B, A])));
    assert_eq!(ghost_labels(&mut app), labels(&["C", "A", "B", "A", "BR"]));
    tap(&mut app, &[ShiftLeft], KeyR);
    assert_eq!(turned(&app), Some(BuildingOrientation::default()));
    assert_eq!((size(&app), letters(&app)), (Some((3, 2)), Some([A, B, A, C])));
    tap(&mut app, &[], KeyF);
    assert_eq!((size(&app), letters(&app)), (Some((3, 2)), Some([A, C, A, B])));
    tap(&mut app, &[], KeyF);
    assert_eq!(turned(&app), Some(BuildingOrientation::default()));
    tap(&mut app, &[], KeyR);
    tap(&mut app, &[ShiftLeft], KeyF);
    assert_eq!((size(&app), letters(&app)), (Some((2, 3)), Some([B, A, C, A])));
    tap(&mut app, &[ShiftLeft], KeyF);
    assert_eq!((size(&app), letters(&app)), (Some((2, 3)), Some([C, A, B, A])));
    assert_eq!(
        constructions(&app).len(),
        1,
        "the Construct only: nothing queued, no slot pressed"
    );
    assert_eq!(placing(&app).map(|p| p.kind), Some(BARRACKS_KIND));

    // Placed as turned: its exit side, B, faces south.
    let shown = ghost(&app).expect("the ghost");
    let orientation = turned(&app).unwrap();
    click(&mut app, middle(OPEN, (3, 2)), &[]);
    assert_eq!(
        constructions(&app).last(),
        Some(&(P1, vec![DC], place(shown.origin, orientation)))
    );
    assert_eq!(
        structure_at(&app, BARRACKS_KIND, shown.origin).map(|(_, o)| o),
        Some(orientation)
    );
    assert!(!placement_keys(&app));
}

/// `factions/gdo/structures/README.md > GDOBuildArea`: "After placement, the build area expands outward from the placed
/// building by that building's BuildRadiusExtension", and "it shrinks when a building is destroyed". The overlay shows
/// the area as it is each time a building is placed, and follows it while placement goes on; its mesh is built only
/// when the area changed (design change 0093: "one clean shape, which doesn't flicker").
#[test]
fn the_overlay_follows_the_build_area_as_generators_come_and_go() {
    let mut app = construction_game(1000);
    select(&mut app, DC);
    construct_and_place(&mut app, KeyQ, GENERATOR);
    let first = build_area(&app);
    assert_eq!(overlay(&mut app), (Some(first.clone()), true));
    let built = overlay_builds(&app);
    updates(&mut app, 10);
    assert_eq!(overlay_builds(&app), built, "nothing changed: not built again");
    let east = TilePos::new(36, 12);
    assert!(!first.contains(east));

    hover(&mut app, middle(EDGE, (2, 2)));
    assert!(ghost(&app).is_some_and(|g| g.is_valid()));
    click(&mut app, middle(EDGE, (2, 2)), &[]);
    let (generator, _) = structure_at(&app, GENERATOR, EDGE).expect("placed");
    assert!(!overlay(&mut app).1, "no overlay out of placement");
    let grown = build_area(&app);
    assert!(grown.contains(east));

    construct_and_place(&mut app, KeyQ, GENERATOR);
    assert_eq!(overlay(&mut app), (Some(grown), true));
    assert_eq!(overlay_builds(&app), built + 1);

    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::Remove {
            id: ObjectId(generator),
        }),
    );
    updates(&mut app, 2);
    assert_eq!(build_area(&app), first, "it shrinks back");
    assert_eq!(overlay(&mut app), (Some(first), true));
    assert_eq!(overlay_builds(&app), built + 2);
}

/// How many vertices the overlay's mesh has.
fn overlay_vertices(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<&Mesh2d, With<BuildAreaMesh>>();
    let mesh = query.single(app.world()).unwrap();
    let meshes = app.world().resource::<Assets<Mesh>>();
    meshes.get(&mesh.0).unwrap().count_vertices()
}

/// Slice M3-J3 (M3-K2's walkthrough: "the build-area tint turns unexplored ground lake-blue"): with the fog on (F3:
/// debug commands are on), the overlay leaves out the tiles of the area player 1 hasn't explored, which stay black
/// (`core/vision.md > Unexplored`: "fully black"), tiles (30, 8) to (31, 9) among them, in nobody's sight; it tints
/// them once their Peacekeeper has been there to look, and every tile with the fog off.
#[test]
fn the_overlay_leaves_out_the_tiles_the_player_hasnt_explored() {
    let mut app = construction_game(1000);
    tap(&mut app, &[], KeyCode::F3);
    assert!(app.world().resource::<FogOfWar>().enabled, "the fog on");
    select(&mut app, DC);
    construct_and_place(&mut app, KeyQ, GENERATOR);
    let area = build_area(&app);
    let map = app.world().resource::<SimRes>().map().clone();
    let unexplored = |app: &App| app.world().resource::<BuildAreaOverlay>().unexplored.clone();
    let without = |left_out: &[TilePos]| area_mesh(&map, &area, |t| !left_out.contains(&t)).vertex_count();
    let dark = [(30, 8), (31, 8), (30, 9), (31, 9)].map(|(x, y)| TilePos::new(x, y));

    let left_out = unexplored(&app);
    let sight_says: Vec<TilePos> = {
        let sim = app.world().resource::<SimRes>();
        let sight = Sight::new(sim, P1, true);
        (area.tiles())
            .filter(|&t| sight.tile(t) == VisibilityState::Unexplored)
            .collect()
    };
    assert_eq!(left_out, sight_says);
    assert!(dark.iter().all(|t| left_out.contains(t)), "{left_out:?}");
    assert!(left_out.len() < area.tiles().count(), "the rest is explored");
    assert_eq!(overlay(&mut app), (Some(area.clone()), true));
    assert_eq!(overlay_vertices(&mut app), without(&left_out));
    let all = without(&[]);
    assert!(overlay_vertices(&mut app) < all);

    let pos = TilePos::new(31, 9).corner();
    submit(
        &mut app,
        P1,
        Command::Debug(DebugCommand::SetPosition { id: ObjectId(PK), pos }),
    );
    updates(&mut app, 3);
    let explored = unexplored(&app);
    assert!(dark.iter().all(|t| !explored.contains(t)), "{explored:?}");
    assert!(explored.len() < left_out.len());
    assert_eq!(overlay_vertices(&mut app), without(&explored));

    tap(&mut app, &[], KeyCode::F3);
    assert!(unexplored(&app).is_empty(), "the fog off");
    assert_eq!(overlay_vertices(&mut app), all);
}
