//! The Recruits' placement (slice M5-J1, `space_crystals::command_ui::Builder::Recruits`): a Construct menu's entry
//! places its building for the whole group, paid once (design changes 0072 and 0327); a Shift chain of two, one
//! lingering ghost each (design change 0072, `systems/controls/interface.md > AwaitingTarget > Shift while
//! placing`); a Storage's ghost, 3x2, turning between two orientations (design change 0380); and a Recruitment Hall
//! flush against the map's edge, its Recruitment Area reaching off the map, placed (design change 0387).

use space_crystals::command_ui::feedback::placement_text;
use space_crystals::command_ui::{Builder, Placing};

use super::*;
use KeyCode::{KeyQ, KeyR, ShiftLeft};

/// Where the Recruits build in these tests, in the north of the world view, a few seconds' walk away: an Armory's
/// site (3 × 2), and two Storages' (3 × 2 since design change 0380), side by side.
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
    click(&mut app, middle(STORAGE_SITES[0], (3, 2)), &[]);
    assert_eq!(interface(&app), InterfaceState::AwaitingPlacement(chained));
    let over_first = middle(STORAGE_SITES[0], (3, 2));
    pointer_over(&mut app, over_first);
    let shown = ghost(&app).expect("a ghost");
    assert!(!shown.is_valid(), "the first site is reserved");
    click(&mut app, over_first, &[]);
    assert_eq!(said(&app), Some(placement_text(shown.check.first_problem())));
    assert_eq!(builds(&app).len(), 1);
    click(&mut app, middle(STORAGE_SITES[1], (3, 2)), &[]);
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
    assert_eq!(lingering(&app), STORAGE_SITES.map(|site| (site, 3, 2)));
}

/// Design change 0380's Verification: "A Storage ghost is 3x2 and rotates between two orientations"
/// (`factions/cults/structures/storage.md`: "Size - 3x2 grid units", "SymmetryType - ABAB (long sides alike, short
/// sides alike)"). The Storage's ghost is 3 tiles across and 2 down; R turns it a quarter, 2 across and 3 down, its
/// sides turned with it; a second R shows it as it was at first, footprint and sides, a half turn of ABAB being the
/// same, and a third the turned one again.
#[test]
fn a_storage_ghost_is_3x2_and_turns_between_two_orientations() {
    let mut app = recruit_game(100);
    select_units(&mut app, &RECRUITS);
    tap(&mut app, &[], KeyA);
    tap(&mut app, &[], KeyW);
    assert_eq!(
        interface(&app),
        InterfaceState::AwaitingPlacement(Placing::recruits(STORAGE_KIND))
    );
    pointer_over(&mut app, middle(STORAGE_SITES[0], (3, 2)));
    let shown = |app: &App| {
        let ghost = ghost(app).expect("a ghost");
        assert_eq!(ghost.kind, STORAGE_KIND);
        (ghost.size, ghost.sides)
    };
    let first = shown(&app);
    assert_eq!(first.0, (3, 2), "3 across, 2 down");
    tap(&mut app, &[], KeyR);
    let turned = shown(&app);
    assert_eq!(turned.0, (2, 3), "a quarter turn");
    assert_ne!(turned.1, first.1, "its sides turned with it");
    tap(&mut app, &[], KeyR);
    assert_eq!(shown(&app), first, "a half turn: as at first");
    tap(&mut app, &[], KeyR);
    assert_eq!(shown(&app), turned);
}

/// Design change 0387's Verification: "Place a Hall flush against the map edge: it is accepted"
/// (`factions/cults/structures/recruitment_hall.md > RecruitmentArea`: "a Hall can be placed anywhere its 4x4 footprint
/// fits on the map, including right at the edge"), where slice M5-J1 refused one 2 tiles from it (design change 0341
/// point 1). Flush against the north edge, 3 rows of its Recruitment Area off the map, the Hall's ghost is green, and a
/// click sends the group the Build and ends placement.
#[test]
fn a_hall_flush_against_the_map_edge_is_placed() {
    let mut app = recruit_game(300);
    select_units(&mut app, &RECRUITS);
    tap(&mut app, &[], KeyA);
    tap(&mut app, &[], KeyQ);
    assert_eq!(
        interface(&app),
        InterfaceState::AwaitingPlacement(Placing::recruits(HALL_KIND))
    );
    let flush = TilePos::new(20, 0);
    look_at(&mut app, middle(flush, (4, 4)));
    pointer_over(&mut app, middle(flush, (4, 4)));
    let shown = ghost(&app).expect("a ghost");
    assert_eq!((shown.kind, shown.origin), (HALL_KIND, flush));
    assert!(shown.is_valid(), "{:?}", shown.check);
    click(&mut app, middle(flush, (4, 4)), &[]);
    assert_eq!(builds(&app), [given(&RECRUITS, build_at(HALL_KIND, flush), false)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
}
