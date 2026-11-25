//! Layouts and their faction's Starting Conditions: on a Skirmish Default map (design changes 0009 and 0012)
//! the palette offers only them and the map says what its layouts lack until it plays; and an outdated layout
//! is flagged until fixed (`> Outdated layouts`).

use super::*;

/// 0009 and 0012: on a Skirmish Default map the palette offers only the Starting Conditions with how many are left,
/// and the layout takes no more than they hold.
#[test]
fn skirmish_layouts_take_only_the_starting_conditions() {
    let map = with_patches(skirmish(32, 24, 2, Symmetry::Rotate180), &[(3, 3), (4, 3), (5, 3)]);
    let mut app = editor(map, &temp_maps("layout_restricted"));
    act(&mut app, [A::SetLayoutFaction(Faction::Gdo)]);
    let left = |app: &App| -> Vec<(ObjectKind, Option<u32>)> {
        palette(doc(app), S1, Faction::Gdo)
            .into_iter()
            .map(|PaletteEntry { kind, left }| (kind, left))
            .collect()
    };
    assert_eq!(left(&app), [(DC, Some(1)), (EF, Some(1)), (PLATE, Some(2))]);
    act(&mut app, [A::SetLayoutKind(BARRACKS)]);
    assert_warned(
        &app,
        "A Skirmish Default GDO layout holds only its Starting Conditions (Deployment Center, Extraction Facility, \
         Extraction Plate x2): no Barracks",
    );
    assert_eq!(tools(&app).layout_kind, DC);

    click(&mut app, spot(2, 6, 4, 4));
    assert_eq!(left(&app), [(DC, Some(0)), (EF, Some(1)), (PLATE, Some(2))]);
    assert_eq!(tools(&app).layout_kind, EF, "on to what the layout lacks");
    // A second Deployment Center is refused.
    act(&mut app, [A::SetLayoutKind(DC)]);
    click(&mut app, spot(9, 12, 4, 4));
    assert_warned(
        &app,
        "The layout already has the Deployment Center of the GDO Starting Conditions",
    );
    assert_eq!(gdo(&app, S1).len(), 1);

    act(&mut app, [A::SetLayoutKind(PLATE)]);
    click(&mut app, at(3, 3));
    click(&mut app, at(4, 3));
    assert_eq!(left(&app), [(DC, Some(0)), (EF, Some(1)), (PLATE, Some(0))]);
    assert_eq!(tools(&app).layout_kind, EF);
    act(&mut app, [A::SetLayoutKind(PLATE)]);
    click(&mut app, at(5, 3));
    assert_warned(
        &app,
        "The layout already has the Extraction Plate x2 of the GDO Starting Conditions",
    );
    assert_eq!(gdo(&app, S1).len(), 3);
    let panel = texts_under::<LayoutPanel>(&mut app);
    for expected in [
        "Deployment Center: done",
        "Extraction Facility: 1 left",
        "Extraction Plate: done",
    ] {
        assert!(panel.iter().any(|t| t == expected), "{expected:?} not in {panel:?}");
    }
}

/// 0012 and the slice's acceptance: a new Skirmish Default map says what each layout lacks, in the Problems list and
/// when a play-test is refused, until it is complete; then it saves, opens and play-tests with every player's base.
/// The images of slot 1's layout in slot 2 follow it, turned half around. (The Colonists' and the Syndicate's layouts
/// are given here; `colonists.rs` and `syndicate.rs` place them.)
#[test]
fn a_new_skirmish_map_lists_what_its_layouts_lack_until_it_plays() {
    let dir = temp_maps("layout_complete");
    let map = with_syndicate(with_colonists(with_patches(
        skirmish(32, 24, 2, Symmetry::Rotate180),
        &[(3, 3), (4, 3)],
    )));
    let mut app = editor(map, &dir);
    let missing = "S1 GDO layout: add Deployment Center, Extraction Facility, Extraction Plate x2 (Layout tool, T); \
                   S2 follows";
    assert_eq!(texts_under::<ProblemList>(&mut app), [missing]);
    act(&mut app, [A::PlayTest]);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::Editor);
    assert_eq!(
        status_message(&app),
        (StatusLevel::Error, format!("Can't play-test: {missing}"))
    );

    // A click on the line sets the Layout tool on what to place first.
    act(&mut app, [A::FocusProblem(0)]);
    assert_eq!(tools(&app).tool, Tool::Layout);
    assert_eq!(
        (tools(&app).slot, tools(&app).faction, tools(&app).layout_kind),
        (S1, Faction::Gdo, DC)
    );
    click(&mut app, spot(2, 6, 4, 4));
    assert_eq!(
        texts_under::<ProblemList>(&mut app),
        ["S1 GDO layout: add Extraction Facility, Extraction Plate x2 (Layout tool, T); S2 follows"]
    );
    let panel = texts_under::<LayoutPanel>(&mut app);
    assert!(
        panel
            .iter()
            .any(|t| t == "S1 GDO layout: missing Extraction Facility, Extraction Plate x2"),
        "{panel:?}"
    );

    // The Extraction Facility goes where the Deployment Center's build area reaches (0097).
    assert_eq!(tools(&app).layout_kind, EF);
    click(&mut app, spot(7, 6, 3, 3));
    click(&mut app, at(3, 3));
    act(&mut app, [A::SetLayoutKind(PLATE)]);
    click(&mut app, at(4, 3));
    assert_eq!(texts_under::<ProblemList>(&mut app), ["None: the map is playable"]);
    let panel = texts_under::<LayoutPanel>(&mut app);
    assert!(panel.iter().any(|t| t == "S1 GDO layout: complete"), "{panel:?}");
    assert!(!panel.iter().any(|t| t.starts_with("Outdated")), "{panel:?}");
    assert_eq!(doc(&app).outdated_layouts(), []);
    let layout = [
        laid(DC, 2, 6, UNTURNED),
        laid(EF, 7, 6, UNTURNED),
        laid(PLATE, 3, 3, UNTURNED),
        laid(PLATE, 4, 3, UNTURNED),
    ];
    assert_eq!(gdo(&app, S1), layout);
    // Slot 2's is the image: each footprint through the map's center, turned half around.
    let half = turned(Rotation::R180, false);
    assert_eq!(
        gdo(&app, S2),
        [
            laid(DC, 26, 14, half),
            laid(EF, 22, 15, half),
            laid(PLATE, 28, 20, half),
            laid(PLATE, 27, 20, half)
        ]
    );

    act(&mut app, [A::Save]);
    let path = app.world().resource::<EditorDoc>().path().unwrap().to_path_buf();
    let saved = doc(&app).clone();
    assert_eq!(files::load(&path), Ok(saved.clone()));
    act(&mut app, [A::NewMap, A::CreateMap, A::Open(path.clone())]);
    assert_eq!(*doc(&app), saved);
    act(&mut app, [A::PlayTest]);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::PlayTest);
    for player in [PlayerId(1), PlayerId(2)] {
        let mut base: Vec<ObjectKind> = sim(&app)
            .objects()
            .filter(|o| o.owner == Some(player))
            .map(|o| o.kind)
            .collect();
        base.sort();
        let mut expected = vec![DC, EF, PLATE, PLATE];
        expected.sort();
        assert_eq!(base, expected, "{player:?}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// `> Outdated layouts`: a layout that doesn't hold the faction's current Starting Conditions is flagged until the
/// map maker fixes it; nothing is fixed automatically. Here an older layout with a starting unit the GDO no longer
/// has.
#[test]
fn outdated_layouts_are_flagged_until_fixed() {
    let mut map = with_syndicate(with_colonists(with_patches(
        skirmish(32, 24, 2, Symmetry::Rotate180),
        &[(3, 3), (4, 3)],
    )));
    *map.slot_mut(S1).unwrap().layouts.get_mut(Faction::Gdo) = vec![
        laid(DC, 2, 6, UNTURNED),
        laid(EF, 7, 6, UNTURNED),
        laid(PLATE, 3, 3, UNTURNED),
        laid(PLATE, 4, 3, UNTURNED),
        LayoutObject {
            kind: PK,
            at: SpawnAt::Point(at(10, 12)),
            orientation: UNTURNED,
        },
    ];
    map.regenerate();
    let mut app = editor(map, &temp_maps("layout_outdated"));
    assert_eq!(doc(&app).outdated_layouts(), [(S1, Faction::Gdo), (S2, Faction::Gdo)]);
    let panel = texts_under::<LayoutPanel>(&mut app);
    assert!(
        panel
            .iter()
            .any(|t| t.starts_with("Outdated or unfinished: S1 GDO, S2 GDO")),
        "{panel:?}"
    );
    assert_eq!(
        texts_under::<ProblemList>(&mut app),
        ["S1 GDO layout, Peacekeeper: not in the GDO Starting Conditions; S2 follows"]
    );
    // Nothing changed by itself; the map maker removes the unit.
    act(&mut app, [A::FocusProblem(0)]);
    assert_eq!(
        selection(&app),
        Some(Selection::LayoutObject {
            slot: S1,
            faction: Faction::Gdo,
            index: 4
        })
    );
    act(&mut app, [A::DeleteSelected]);
    assert_eq!(doc(&app).outdated_layouts(), []);
    assert_eq!(problems(&app), []);
    let panel = texts_under::<LayoutPanel>(&mut app);
    assert!(!panel.iter().any(|t| t.starts_with("Outdated")), "{panel:?}");
}
