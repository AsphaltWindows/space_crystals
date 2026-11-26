//! The door arrows (`core/entities.md > Entrance and exit (production and research buildings)`, design changes 0390
//! and 0394) in the client: on the Recruits' ghost of an Armory, turning and flipping with it, and on the selected
//! Armory. Which doorways each type has, and their look, are the unit tests' (`door_arrows/tests.rs`).

use space_crystals::door_arrows::{DoorArrow, Way, door_arrows};
use space_crystals::selection::doors::{SelectedDoors, SelectedDoorsMesh};

use super::*;
use KeyCode::{KeyF, KeyR};

/// Each arrow's way and the direction it points out of its side, and the doorway it stands at.
fn ways(arrows: &[DoorArrow]) -> Vec<(Way, Vec2, Vec2)> {
    arrows
        .iter()
        .map(|arrow| (arrow.way, arrow.outward, arrow.at))
        .collect()
}

/// Design change 0390's Verification: "Place a Cults Armory: an inward arrow shows on side B and an outward arrow on
/// side C. Rotating the ghost turns the arrows with it." The Recruits' W then A (design change 0398) puts an Armory's
/// ghost under the cursor, unturned (3 × 2, ABAC: B east, C west): an arrow in at the middle of its east side, its
/// entrance, and one out at the middle of its west side, its exit. F flips it as seen on the map (B west, C east), and
/// R then turns it a quarter clockwise (2 × 3: B north, C south), the arrows with it, each at the middle of its side;
/// the ghost's mesh is built again each time.
#[test]
fn the_armory_ghosts_arrows_turn_and_flip_with_it() {
    let mut app = recruit_game(300);
    select_units(&mut app, &RECRUITS);
    tap(&mut app, &[], KeyW);
    tap(&mut app, &[], KeyA);
    let site = TilePos::new(20, 9);
    pointer_over(&mut app, middle(site, (3, 2)));
    let builds = |app: &App| app.world().resource::<PlacementGhost>().builds;
    // Each arrow's way and direction, and whether it stands at the middle of its side of the ghost.
    let shown = |app: &App| -> Vec<(Way, Vec2)> {
        let ghost = ghost(app).expect("a ghost");
        assert_eq!(ghost.kind, ARMORY_KIND);
        let half = Vec2::new(ghost.size.0 as f32, ghost.size.1 as f32) * SU_PER_GU / 2.0;
        let centre = ghost.origin.corner() + half;
        (ways(&ghost.door_arrows()).into_iter())
            .map(|(way, outward, at)| {
                assert_eq!(at, centre + outward * outward.abs().dot(half), "at its side's middle");
                (way, outward)
            })
            .collect()
    };
    let (north, east, south, west) = (Vec2::NEG_Y, Vec2::X, Vec2::Y, Vec2::NEG_X);
    assert_eq!(ghost(&app).map(|g| (g.origin, g.size)), Some((site, (3, 2))));
    assert_eq!(shown(&app), [(Way::In, east), (Way::Out, west)]);
    // Tiles (20, 9) to (22, 10): x 1280 to 1472, y 576 to 704.
    let arrows = ghost(&app).expect("a ghost").door_arrows();
    assert_eq!(
        ways(&arrows).iter().map(|&(.., at)| at).collect::<Vec<_>>(),
        [Vec2::new(1472.0, 640.0), Vec2::new(1280.0, 640.0)]
    );
    let before = builds(&app);
    tap(&mut app, &[], KeyF);
    assert_eq!(shown(&app), [(Way::Out, east), (Way::In, west)]);
    assert!(builds(&app) > before, "the mesh is built again");
    let before = builds(&app);
    tap(&mut app, &[], KeyR);
    assert_eq!(ghost(&app).expect("a ghost").size, (2, 3));
    assert_eq!(shown(&app), [(Way::In, north), (Way::Out, south)]);
    assert!(builds(&app) > before, "the mesh is built again");
}

/// Design change 0394's Verification: "Select an Armory: an inward arrow shows on its entrance and an outward arrow on
/// its exit." Player 1's Armory, unturned on tiles (30, 9) to (32, 10), is entered at side B, the east, and left at
/// side C, the west: a click selects it and its arrows show there; a click on bare ground clears the selection and
/// hides them. The Recruits selected have no doorway: no arrows.
#[test]
fn a_selected_armory_shows_its_entrance_and_exit_arrows() {
    let mut app = recruit_game(0);
    let shown = |app: &mut App| {
        let doors = app.world().resource::<SelectedDoors>();
        let arrows: Vec<DoorArrow> = (doors.shown.iter())
            .flat_map(|(kind, placement)| door_arrows(*kind, placement))
            .collect();
        let mut mesh = app.world_mut().query_filtered::<&Visibility, With<SelectedDoorsMesh>>();
        let visible = *mesh.single(app.world()).expect("one mesh") != Visibility::Hidden;
        (ways(&arrows), visible)
    };
    assert_eq!(shown(&mut app), (vec![], false));
    click(&mut app, middle(ARMORY_AT, (3, 2)), &[]);
    assert_eq!(selected(&app), [ARMORY]);
    // x 1920 to 2112, y 576 to 704.
    assert_eq!(
        shown(&mut app),
        (
            vec![
                (Way::In, Vec2::X, Vec2::new(2112.0, 640.0)),
                (Way::Out, Vec2::NEG_X, Vec2::new(1920.0, 640.0))
            ],
            true
        )
    );
    pause(&mut app);
    click(&mut app, Vec2::new(1600.0, 800.0), &[]);
    assert_eq!(selected(&app), Vec::<u32>::new());
    assert_eq!(shown(&mut app), (vec![], false));
    select_units(&mut app, &RECRUITS);
    assert_eq!(shown(&mut app), (vec![], false));
}
