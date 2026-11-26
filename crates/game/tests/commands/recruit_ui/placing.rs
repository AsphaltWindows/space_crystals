//! The Recruits' placement (slice M5-J1, `space_crystals::command_ui::Builder::Recruits`): a Construct menu's entry
//! places its building for the whole group, paid once (design changes 0072 and 0327); a Shift chain of two, one
//! lingering ghost each (design change 0072, `systems/controls/interface.md > AwaitingTarget > Shift while
//! placing`); and a Recruitment Hall whose Recruitment Area would leave the map, refused with its reason (design
//! change 0341 point 1).

use sc_sim::{PlacementProblem, WholeProblem};
use space_crystals::command_ui::feedback::placement_text;
use space_crystals::command_ui::{Builder, Placing};

use super::*;
use KeyCode::{KeyQ, ShiftLeft};

/// Where the Recruits build in these tests, in the north of the world view, a few seconds' walk away: an Armory's
/// site (3 × 2), and two Storages' (2 × 2), side by side.
const ARMORY_SITE: TilePos = TilePos::new(20, 9);
const STORAGE_SITES: [TilePos; 2] = [TilePos::new(22, 9), TilePos::new(25, 9)];

/// Design change 0327's Verification: "Recruits: A then A places an Armory", and 0072's Verification 1: "Select 4
/// Recruits, press A (Construct), choose a building and place it. [...] The cost is deducted once." A then A enters
/// the Armory's placement for the four, its ghost green on clear ground; a click sends them one Build, together, and
/// ends placement; its 200 Space Crystals are paid once, and it lingers as one ghost.
#[test]
fn a_then_a_places_an_armory_for_the_whole_group() {
    let mut app = recruit_game(300);
    select_units(&mut app, &RECRUITS);
    tap(&mut app, &[], KeyA);
    tap(&mut app, &[], KeyA);
    assert_eq!(
        interface(&app),
        InterfaceState::AwaitingPlacement(Placing::recruits(ARMORY_KIND))
    );
    let at = middle(ARMORY_SITE, (3, 2));
    pointer_over(&mut app, at);
    let shown = ghost(&app).expect("a ghost");
    assert_eq!(
        (shown.kind, shown.origin, shown.size),
        (ARMORY_KIND, ARMORY_SITE, (3, 2))
    );
    assert!(shown.is_valid(), "{:?}", shown.check);
    click(&mut app, at, &[]);
    assert_eq!(
        builds(&app),
        [given(&RECRUITS, build_at(ARMORY_KIND, ARMORY_SITE), false)]
    );
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    updates(&mut app, 2);
    assert_eq!(crystals(&app), 100, "paid once");
    assert_eq!(lingering(&app), [(ARMORY_SITE, 3, 2)]);
}

/// Design change 0072's Verification 2: "With 4 Recruits selected, hold Shift and place 2 buildings. The group builds
/// the first one together, then moves to the second" (`systems/controls/interface.md > AwaitingTarget > Shift
/// while placing`: "e.g. Cults Recruits (the whole selected group) build one and walk together to the next"). With
/// Shift held, each click queues a Storage's Build for the four and placement stays; the second ghost is red over the
/// first one's site, reserved, and a click there says so and sends nothing. Letting go of Shift ends placement. Two
/// Builds, each paid as it was given, and two lingering ghosts: one per ticket, not per Recruit (slice M5-E).
#[test]
fn shift_chains_the_groups_buildings_one_ghost_each() {
    let mut app = recruit_game(200);
    select_units(&mut app, &RECRUITS);
    tap(&mut app, &[], KeyA);
    tap(&mut app, &[], KeyW);
    press_keys(&mut app, &[ShiftLeft]);
    let chained = Placing {
        builder: Builder::Recruits { queued: true },
        ..Placing::recruits(STORAGE_KIND)
    };
    click(&mut app, middle(STORAGE_SITES[0], (2, 2)), &[]);
    assert_eq!(interface(&app), InterfaceState::AwaitingPlacement(chained));
    let over_first = middle(STORAGE_SITES[0], (2, 2));
    pointer_over(&mut app, over_first);
    let shown = ghost(&app).expect("a ghost");
    assert!(!shown.is_valid(), "the first site is reserved");
    click(&mut app, over_first, &[]);
    assert_eq!(said(&app), Some(placement_text(shown.check.first_problem())));
    assert_eq!(builds(&app).len(), 1);
    click(&mut app, middle(STORAGE_SITES[1], (2, 2)), &[]);
    assert_eq!(interface(&app), InterfaceState::AwaitingPlacement(chained));
    release_keys(&mut app, &[ShiftLeft]);
    app.update();
    assert_eq!(
        interface(&app),
        InterfaceState::DefaultState,
        "letting go of Shift ends placement"
    );
    let chain = STORAGE_SITES.map(|site| given(&RECRUITS, build_at(STORAGE_KIND, site), true));
    assert_eq!(builds(&app), chain);
    updates(&mut app, 2);
    assert_eq!(crystals(&app), 0, "100 each");
    assert_eq!(lingering(&app), STORAGE_SITES.map(|site| (site, 2, 2)));
}

/// Design change 0341 point 1 (`factions/cults/structures/recruitment_hall.md > RecruitmentArea`): "The whole 10x10
/// area must lie on the map. A Hall that would leave it can't be placed" (0341's Verification: "Placing a Hall 3
/// tiles from the edge is refused", the footprint on the map's third tile: its area, 3 tiles beyond it, would reach a
/// tile past the edge). Its ghost is red there, and a click says why and sends nothing, placement going on; a tile
/// further in, its area just on the map, the ghost is green, and the click sends the Build.
#[test]
fn a_hall_whose_area_leaves_the_map_is_refused_with_its_reason() {
    let mut app = recruit_game(300);
    select_units(&mut app, &RECRUITS);
    tap(&mut app, &[], KeyA);
    tap(&mut app, &[], KeyQ);
    assert_eq!(
        interface(&app),
        InterfaceState::AwaitingPlacement(Placing::recruits(HALL_KIND))
    );
    let (refused, valid) = (TilePos::new(2, 20), TilePos::new(3, 20));
    look_at(&mut app, middle(refused, (4, 4)));
    pointer_over(&mut app, middle(refused, (4, 4)));
    let shown = ghost(&app).expect("a ghost");
    assert_eq!((shown.kind, shown.origin), (HALL_KIND, refused));
    assert_eq!(shown.check.whole, [WholeProblem::RecruitmentAreaOffMap]);
    assert!(!shown.is_valid());
    click(&mut app, middle(refused, (4, 4)), &[]);
    let problem = Some(PlacementProblem::Whole(WholeProblem::RecruitmentAreaOffMap));
    assert_eq!(said(&app), Some(placement_text(problem)));
    assert_eq!(said(&app), Some("The Recruitment Area reaches off the map"));
    assert_eq!(builds(&app), []);
    assert!(matches!(interface(&app), InterfaceState::AwaitingPlacement(_)));

    pointer_over(&mut app, middle(valid, (4, 4)));
    let shown = ghost(&app).expect("a ghost");
    assert_eq!(shown.origin, valid);
    assert!(shown.is_valid(), "{:?}", shown.check);
    click(&mut app, middle(valid, (4, 4)), &[]);
    assert_eq!(builds(&app), [given(&RECRUITS, build_at(HALL_KIND, valid), false)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}
