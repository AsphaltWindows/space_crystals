//! The rules a layout follows: its faction's placement rules minus sight (design change 0097), plates onto
//! uncovered patches only (the Place tool's too), and the layouts of two slots in each other's way, listed
//! under Problems at once.

use super::*;

/// 0097: each layout follows its faction's placement rules, minus sight. The Extraction Facility and the plates go
/// within the build area of the layout's own Deployment Center, a plate onto an uncovered patch, a starting unit
/// not onto impassable ground; a Deployment Center that sees no patch is fine, and nothing warns about sight.
#[test]
fn layouts_follow_the_placement_rules_minus_sight() {
    // The patches lie far from where the Deployment Center goes, 10 tiles and more.
    let map = with_patches(skirmish(48, 32, 2, Symmetry::MirrorVerticalAxis), &[(20, 3), (21, 3)]);
    let mut app = editor(map, &temp_maps("layout_rules"));
    act(&mut app, [A::SetLayoutFaction(Faction::Gdo)]);
    click(&mut app, spot(2, 18, 4, 4));
    assert_eq!(status(&app).level, StatusLevel::Info, "{}", status(&app).message);
    assert!(
        problems(&app)
            .iter()
            .all(|p| matches!(p, ScenarioProblem::LayoutMissing { .. })),
        "{:?}",
        problems(&app)
    );

    // The Deployment Center's build area reaches 12 tiles past its edge: x 17 at the most.
    click(&mut app, spot(18, 18, 3, 3));
    assert_warned(
        &app,
        "Can't place the Extraction Facility there: no tile is in the build area of the layout's Deployment Center",
    );
    click(&mut app, spot(17, 18, 3, 3));
    assert_eq!(gdo(&app, S1).last(), Some(&laid(EF, 17, 18, UNTURNED)));
    assert_eq!(status(&app).level, StatusLevel::Info, "{}", status(&app).message);

    // A plate goes onto a patch only, and there into the build area.
    act(&mut app, [A::SetLayoutKind(PLATE)]);
    click(&mut app, at(10, 10));
    assert_warned(&app, "Can't place the Extraction Plate there");
    click(&mut app, at(20, 3));
    assert_warned(&app, "no tile is in the build area");
    assert_eq!(gdo(&app, S1).len(), 2);

    // Starting units: not onto a cliff, nor onto a structure (on a Custom map, which takes them).
    let mut app = editor(scenarios::empty_map(32, 24), &temp_maps("layout_units"));
    act(
        &mut app,
        [A::SetPreset(TilePresetId::CLIFF), A::Press(at(12, 12)), A::Release],
    );
    act(&mut app, [A::SetLayoutFaction(Faction::Gdo), A::SetLayoutKind(PK)]);
    click(&mut app, at(12, 12));
    assert_warned(
        &app,
        "Can't place the Peacekeeper there: its box overlaps impassable ground",
    );
    assert_eq!(gdo(&app, S1), []);
    click(&mut app, at(14, 12));
    assert_eq!(
        gdo(&app, S1),
        [LayoutObject {
            kind: PK,
            at: SpawnAt::Point(at(14, 12)),
            orientation: UNTURNED
        }]
    );
    assert_eq!(problems(&app), []);
}

/// The Place tool puts a plate onto a patch that no other plate covers (`ObjectKind::may_overlap`, the one overlap
/// the sim allows); nothing else goes onto a patch.
#[test]
fn the_place_tool_puts_plates_onto_uncovered_patches() {
    let mut app = editor(scenarios::empty_map(16, 12), &temp_maps("plate_patch"));
    act(
        &mut app,
        [A::SetKind(ObjectKind::CrystalPatch), A::Press(at(5, 5)), A::Release],
    );
    act(
        &mut app,
        [A::SetKind(PLATE), A::SetOwner(Some(S1)), A::Press(at(5, 5)), A::Release],
    );
    let plate = Spawn::new(PLATE, Some(S1), SpawnAt::Tile(TilePos::new(5, 5)));
    assert_eq!(doc(&app).spawns, [patch(5, 5), plate.clone()]);
    assert_eq!(problems(&app), []);
    act(&mut app, [A::Press(at(5, 5)), A::Release]);
    assert_eq!(doc(&app).spawns.len(), 2, "one plate per patch");
    // A Generator (2 x 2) over the patch.
    act(
        &mut app,
        [
            A::SetKind(ObjectKind::Generator),
            A::Press(TilePos::new(5, 5).corner()),
            A::Release,
        ],
    );
    assert_eq!(doc(&app).spawns, [patch(5, 5), plate], "no other building on a patch");
    // A plate on bare ground goes in, but the map lists it in the sim's words (`factions/gdo/economy.md >
    // ExtractionPlate`, slice M2-J2: the rule of a layout's plate and a game's placement), and doesn't play.
    act(&mut app, [A::SetKind(PLATE), A::Press(at(9, 5)), A::Release]);
    assert_eq!(doc(&app).spawns.len(), 3);
    let line = "spawn 2 (ExtractionPlate): an Extraction Plate goes onto a Space Crystals patch";
    assert_eq!(texts_under::<ProblemList>(&mut app), [line]);
    act(&mut app, [A::PlayTest]);
    assert_eq!(
        status_message(&app),
        (StatusLevel::Error, format!("Can't play-test: {line}"))
    );
}

/// Slice M2-J2 (M2-I's finding): the layouts of two slots in each other's way are listed under Problems as soon as
/// they are, like every other problem, not only when F5 is refused: the list says what keeps the play-test's game
/// from starting (`Scenario::validate`, with the play-test's setup). Each layout is valid on its own
/// (`core/map_editor.md > Overlay editing`), so the Layout tool takes the object. With slot 2 playing a faction whose
/// layout there is empty, nothing is in the way any more.
#[test]
fn layouts_in_each_others_way_are_listed_at_once() {
    let mut app = editor(scenarios::empty_map(32, 24), &temp_maps("layout_overlap"));
    act(&mut app, [A::SetLayoutFaction(Faction::Gdo), A::SetLayoutKind(DC)]);
    click(&mut app, spot(2, 2, 4, 4));
    assert_eq!(problems(&app), []);
    act(
        &mut app,
        [
            A::SelectSlot(S2),
            A::SetLayoutFaction(Faction::Gdo),
            A::SetLayoutKind(DC),
        ],
    );
    click(&mut app, spot(4, 3, 4, 4)); // (4, 3) to (7, 6), over slot 1's from (2, 2) to (5, 5)
    assert_eq!(
        (gdo(&app, S1), gdo(&app, S2)),
        (vec![laid(DC, 2, 2, UNTURNED)], vec![laid(DC, 4, 3, UNTURNED)])
    );
    let overlap = ScenarioProblem::LayoutOverlap {
        slot: S2,
        faction: Faction::Gdo,
        index: 0,
        tile: TilePos::new(4, 3),
    };
    assert_eq!(problems(&app), [overlap]);
    let line = "S2 GDO layout, Deployment Center: overlaps another object at tile (4, 3)";
    assert_eq!(texts_under::<ProblemList>(&mut app), [line]);
    act(&mut app, [A::PlayTest]);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::Editor);
    assert_eq!(
        status_message(&app),
        (StatusLevel::Error, format!("Can't play-test: {line}"))
    );

    act(&mut app, [A::SetPlayTestFaction(S2, Faction::Syndicate)]);
    assert_eq!(problems(&app), []);
    assert_eq!(texts_under::<ProblemList>(&mut app), ["None: the map is playable"]);
    act(&mut app, [A::PlayTest]);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::PlayTest);
}
