//! The Layout tool (T): it edits the layout of the chosen faction at the chosen slot; places, turns (R, Shift+R,
//! F, Shift+F), moves and removes its objects; and a symmetric map's images follow, turned to match.

use super::*;

/// 0008 (a layout per start position and faction) and T11 (only the GDO is implemented): the picker chooses the
/// slot and the faction, and lists the editable slots with the generated ones that follow them.
#[test]
fn the_layout_tool_edits_the_chosen_slot_and_faction() {
    let mut app = editor(skirmish(48, 32, 4, Symmetry::Rotate180), &temp_maps("layout_pick"));
    act(&mut app, [A::SelectSlot(SlotId(3)), A::SetLayoutFaction(Faction::Gdo)]);
    assert_eq!(tools(&app).tool, Tool::Layout);
    assert_eq!((tools(&app).slot, tools(&app).faction), (SlotId(3), Faction::Gdo));
    assert_eq!(
        status_message(&app),
        (
            StatusLevel::Info,
            "Editing the S3 GDO layout: missing Deployment Center, Extraction Facility, Extraction Plate x2".into()
        )
    );
    let panel = texts_under::<LayoutPanel>(&mut app);
    for expected in [
        "S1",
        "S3",
        "Generated: S2 follows S1; S4 follows S3",
        "S3 GDO layout: missing Deployment Center, Extraction Facility, Extraction Plate x2",
        "Deployment Center: 1 left",
        "Extraction Plate: 2 left",
    ] {
        assert!(panel.iter().any(|t| t == expected), "{expected:?} not in {panel:?}");
    }
    assert!(
        !panel.iter().any(|t| t == "S2"),
        "generated slots aren't edited: {panel:?}"
    );

    // The Cults can be chosen, but have nothing to place yet (the Syndicate has had its kinds since slice M4-B).
    act(&mut app, [A::SetLayoutFaction(Faction::Cults)]);
    assert_eq!(tools(&app).faction, Faction::Cults);
    assert!(
        status(&app).message.contains("not implemented yet"),
        "{}",
        status(&app).message
    );
    assert_eq!(palette(doc(&app), SlotId(3), Faction::Cults), []);
    let before = doc(&app).clone();
    click(&mut app, at(5, 20));
    assert_warned(&app, "Cults: not implemented yet, its layouts stay empty");
    assert_eq!(*doc(&app), before);
    let panel = texts_under::<LayoutPanel>(&mut app);
    assert!(
        panel
            .iter()
            .any(|t| t == "Cults: not implemented yet, its layouts stay empty"),
        "{panel:?}"
    );
}

/// 0009 #3: a Custom map's layouts take any player object, a Barracks too. A structure goes in turned as R turned
/// it; the Layout tool drags an object of the layout, and Delete removes the selected one.
#[test]
fn layout_objects_are_placed_turned_moved_and_removed() {
    let mut app = editor(scenarios::empty_map(32, 24), &temp_maps("layout_edit"));
    act(&mut app, [A::SetLayoutFaction(Faction::Gdo), A::SetLayoutKind(DC)]);
    click(&mut app, spot(2, 6, 4, 4));
    assert_eq!(gdo(&app, S1), [laid(DC, 2, 6, UNTURNED)]);
    assert_eq!(
        selection(&app),
        Some(Selection::LayoutObject {
            slot: S1,
            faction: Faction::Gdo,
            index: 0
        })
    );

    // A Barracks, turned a quarter clockwise: 2 × 3 tiles.
    act(&mut app, [A::SetLayoutKind(BARRACKS), A::Turn(Turn::Clockwise)]);
    assert_eq!(tools(&app).orientation, turned(Rotation::R90, false));
    assert_eq!(status(&app).message, "Barracks: turned 90");
    click(&mut app, spot(10, 6, 2, 3));
    let barracks = laid(BARRACKS, 10, 6, turned(Rotation::R90, false));
    assert_eq!(gdo(&app, S1), [laid(DC, 2, 6, UNTURNED), barracks.clone()]);
    assert_eq!(problems(&app), []);
    assert!(
        status(&app)
            .message
            .starts_with("Placed the Barracks in the S1 GDO layout"),
        "{}",
        status(&app).message
    );
    // The sim shows it, turned.
    let shown = sim(&app)
        .objects()
        .find(|o| o.kind == BARRACKS)
        .and_then(|o| o.placement)
        .unwrap();
    assert_eq!(
        (shown.origin, shown.w, shown.h, shown.rotation),
        (TilePos::new(10, 6), 2, 3, Rotation::R90)
    );

    // Dragged 2 tiles south: a click on it grabs it instead of placing another.
    let from = spot(10, 6, 2, 3);
    let to = from + Vec2::new(0.0, 2.0 * SU_PER_GU);
    act(
        &mut app,
        [A::Hover(Some(from)), A::Press(from), A::Drag(to), A::Release],
    );
    let moved = laid(BARRACKS, 10, 8, turned(Rotation::R90, false));
    assert_eq!(gdo(&app, S1), [laid(DC, 2, 6, UNTURNED), moved]);
    assert_eq!(
        selection(&app),
        Some(Selection::LayoutObject {
            slot: S1,
            faction: Faction::Gdo,
            index: 1
        })
    );

    act(&mut app, [A::DeleteSelected]);
    assert_eq!(gdo(&app, S1), [laid(DC, 2, 6, UNTURNED)]);
    assert_eq!(selection(&app), None);
    act(&mut app, [A::Undo]);
    assert_eq!(gdo(&app, S1).len(), 2);
    act(&mut app, [A::Undo, A::Undo]);
    assert_eq!(
        gdo(&app, S1),
        [laid(DC, 2, 6, UNTURNED)],
        "the move and the placement undo one by one"
    );
}

/// `> Symmetry > Editing a symmetric map`: "start positions with all their faction Starting Layouts" are mirrored,
/// and "building orientations are rotated or flipped to match". The overlay's ghost shows where the images go.
#[test]
fn layout_images_follow_turned_and_flipped_to_match() {
    // 32 × 24 tiles mirrored across x = 16: slot 2 is slot 1's mirror image.
    let mut app = editor(
        created(Symmetry::MirrorVerticalAxis, 32, 24, 2),
        &temp_maps("layout_images"),
    );
    act(&mut app, [A::SetLayoutFaction(Faction::Gdo), A::SetLayoutKind(DC)]);
    click(&mut app, spot(2, 6, 4, 4));
    act(&mut app, [A::SetLayoutKind(BARRACKS), A::Turn(Turn::Clockwise)]);
    click(&mut app, spot(8, 6, 2, 3));
    assert_eq!(
        gdo(&app, S1),
        [
            laid(DC, 2, 6, UNTURNED),
            laid(BARRACKS, 8, 6, turned(Rotation::R90, false))
        ]
    );
    // Mirrored left to right: a quarter turn clockwise, flipped, is three quarters flipped.
    assert_eq!(
        gdo(&app, S2),
        [
            laid(DC, 26, 6, turned(Rotation::R0, true)),
            laid(BARRACKS, 22, 6, turned(Rotation::R270, true))
        ]
    );
    assert_eq!(doc(&app).first_asymmetry(), None);
    // The locked half is generated: the Layout tool doesn't edit slot 2, nor place into the locked half.
    click(&mut app, spot(20, 14, 2, 3));
    assert_warned(&app, "Locked");
    assert_eq!(gdo(&app, S1).len(), 2);
}

/// R, Shift+R, F and Shift+F turn the structure the Layout tool places (as the game's placement ghost does, slice
/// M2-H2); without a structure to place, F is still the Fill tool.
#[test]
fn keys_turn_the_structure_being_placed() {
    let mut app = editor(scenarios::empty_map(32, 24), &temp_maps("layout_keys"));
    tap(&mut app, &[], KeyCode::KeyT);
    assert_eq!(tools(&app).tool, Tool::Layout);
    assert_eq!(tools(&app).layout_kind, DC);
    app.update(); // The input layer learns that a structure is being placed.
    tap(&mut app, &[], KeyCode::KeyR);
    assert_eq!(tools(&app).orientation, turned(Rotation::R90, false));
    tap(&mut app, &[KeyCode::ShiftLeft], KeyCode::KeyR);
    tap(&mut app, &[KeyCode::ShiftLeft], KeyCode::KeyR);
    assert_eq!(tools(&app).orientation, turned(Rotation::R270, false));
    // Flipped left to right as seen: a quarter turn clockwise of the flipped building.
    tap(&mut app, &[], KeyCode::KeyF);
    assert_eq!(tools(&app).orientation, turned(Rotation::R90, true));
    assert_eq!(tools(&app).tool, Tool::Layout);
    tap(&mut app, &[], KeyCode::KeyF);
    assert_eq!(tools(&app).orientation, turned(Rotation::R270, false), "flipped back");
    tap(&mut app, &[KeyCode::ShiftLeft], KeyCode::KeyF);
    assert_eq!(
        tools(&app).orientation,
        BuildingOrientation {
            rotation: Rotation::R90,
            flip_h: false,
            flip_v: true
        },
        "flipped top to bottom"
    );
    tap(&mut app, &[KeyCode::ShiftLeft], KeyCode::KeyF);
    assert_eq!(tools(&app).orientation, turned(Rotation::R270, false));

    // A unit doesn't turn: F chooses the Fill tool.
    act(&mut app, [A::SetLayoutKind(PK)]);
    tap(&mut app, &[], KeyCode::KeyR);
    assert_eq!(tools(&app).orientation, turned(Rotation::R270, false));
    tap(&mut app, &[], KeyCode::KeyF);
    assert_eq!(tools(&app).tool, Tool::Fill);
}
