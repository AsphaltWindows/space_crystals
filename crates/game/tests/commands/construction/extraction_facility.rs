//! The Extraction Facility's plate (`> ObjectInterfaceState[ExtractionFacility]`): built, cancelled, and placed
//! on a free patch in the build area.

use super::*;

/// `> ObjectInterfaceState[ExtractionFacility]`: Q builds the plate (75 Space Crystals); while it is built only X
/// (Cancel Construction) shows; once it is ready Q places it and X cancels it. Its ghost is green only on a patch in
/// the build area without a plate; Esc and a right-click return to DefaultState, the plate still ready; a click on the
/// free patch places it there.
#[test]
fn the_extraction_facility_places_its_plate_on_a_free_patch() {
    use ButtonLook::Common;
    use PanelCommand::{CancelConstruction, CancelReady, Construct, PlaceReady};
    let mut app = construction_game(1000);
    select(&mut app, EF);
    let c = PanelButton::Command;
    assert_eq!(buttons(&mut app), [(q(), c(Construct(PLATE_KIND)), Common, "EP")]);
    tap(&mut app, &[], KeyQ);
    let construct = ConstructionCommand::Construct(PLATE_KIND);
    assert_eq!(constructions(&app), [(P1, vec![EF], construct)]);
    assert_eq!(bank(&app), 925);
    assert_eq!(buttons(&mut app), [(X, c(CancelConstruction), Common, "Cancel")]);
    plate_on(&mut app, PLATED_PATCH);
    run_until_ready(&mut app, EF);
    assert_eq!(
        buttons(&mut app),
        [
            (q(), c(PlaceReady(PLATE_KIND)), Common, "Place"),
            (X, c(CancelReady), Common, "Cancel"),
        ]
    );
    tap(&mut app, &[], KeyQ);
    assert_eq!(placing(&app), Some(Placing::new(ObjectId(EF), PLATE_KIND)));
    assert_eq!(buttons(&mut app)[0].2, ButtonLook::Pending);

    let bare = TilePos::new(17, 15);
    let cases = [
        (bare, vec![], vec![WholeProblem::NotOnPatch]),
        (PLATED_PATCH, vec![(PLATED_PATCH, TileProblem::Structure)], vec![]),
        (FAR_PATCH, vec![], vec![WholeProblem::OutsideBuildArea]),
        (FREE_PATCH, vec![], vec![]),
    ];
    for (origin, tiles, whole) in cases {
        hover(&mut app, middle(origin, (1, 1)));
        let shown = ghost(&app).expect("the ghost");
        assert_eq!((shown.origin, shown.size), (origin, (1, 1)));
        assert_eq!(
            (&shown.check.tiles, &shown.check.whole),
            (&tiles, &whole),
            "at {origin:?}"
        );
        assert_eq!(shown.tile_is_green(origin), tiles.is_empty() && whole.is_empty());
    }
    assert_eq!(
        ghost_labels(&mut app),
        [(4, "EP".to_string())],
        "no side letters on one tile"
    );
    assert_eq!(overlay(&mut app), (Some(build_area(&app)), true));

    let ready = Constructor::Ready { kind: PLATE_KIND };
    tap(&mut app, &[], Escape);
    assert_eq!(
        (interface(&app), constructor(&app, EF)),
        (InterfaceState::DefaultState, ready)
    );
    tap(&mut app, &[], KeyQ);
    right_click(&mut app, middle(FREE_PATCH, (1, 1)), &[]);
    assert_eq!(
        (interface(&app), constructor(&app, EF)),
        (InterfaceState::DefaultState, ready)
    );
    tap(&mut app, &[], KeyQ);
    hover(&mut app, middle(FREE_PATCH, (1, 1)));
    click(&mut app, middle(FREE_PATCH, (1, 1)), &[]);
    let placed = place(FREE_PATCH, BuildingOrientation::default());
    assert_eq!(constructions(&app), [(P1, vec![EF], construct), (P1, vec![EF], placed)]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert!(structure_at(&app, PLATE_KIND, FREE_PATCH).is_some());
    assert_eq!(constructor(&app, EF), Constructor::Idle);
    assert_eq!(buttons(&mut app), [(q(), c(Construct(PLATE_KIND)), Common, "EP")]);
}
