//! Starting Layouts in the map editor (slice M2-I; `core/map_editor.md > Starting Layouts`, design changes 0008,
//! 0009, 0012 and 0097): the Layout tool (T) edits the layout of the chosen faction at the chosen slot, turned as R,
//! Shift+R, F and Shift+F turn it; a Skirmish Default layout takes only its faction's Starting Conditions and says
//! what it lacks; every layout follows its faction's placement rules minus sight; symmetric images follow, turned
//! to match; a completed map saves, opens and play-tests.

use bevy::prelude::*;
use sc_sim::{
    BuildingOrientation, Faction, LayoutObject, MapDef, MapSettings, MapType, ObjectKind, PlayerId, Rotation,
    SU_PER_GU, ScenarioProblem, SlotId, Spawn, SpawnAt, Symmetry, TilePos, TilePresetId, scenarios,
};
use space_crystals::ClientMode;
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::doc::EditorDoc;
use space_crystals::editor::files;
use space_crystals::editor::layouts::{PaletteEntry, palette};
use space_crystals::editor::orientation::Turn;
use space_crystals::editor::state::{Selection, StatusLevel};
use space_crystals::editor::tools::Tool;
use space_crystals::editor::ui::lists::ProblemList;
use space_crystals::editor::ui::starting::LayoutPanel;

use super::{
    act, assert_warned, at, created, doc, editor, patch, problems, selection, sim, status, tap, temp_maps, tools,
};

const DC: ObjectKind = ObjectKind::DeploymentCenter; // 4 × 4 tiles
const EF: ObjectKind = ObjectKind::ExtractionFacility; // 3 × 3
const PLATE: ObjectKind = ObjectKind::ExtractionPlate; // 1 × 1
const BARRACKS: ObjectKind = ObjectKind::Barracks; // 3 × 2
const PK: ObjectKind = ObjectKind::Peacekeeper;
const S1: SlotId = SlotId(1);
const S2: SlotId = SlotId(2);
const UNTURNED: BuildingOrientation = BuildingOrientation {
    rotation: Rotation::R0,
    flip_h: false,
    flip_v: false,
};

fn turned(rotation: Rotation, flip_h: bool) -> BuildingOrientation {
    BuildingOrientation {
        rotation,
        flip_h,
        flip_v: false,
    }
}

/// A flat Skirmish Default map, as the creation dialog makes it.
fn skirmish(width: u32, height: u32, slots: usize, symmetry: Symmetry) -> MapDef {
    MapSettings {
        map_type: MapType::SkirmishDefault,
        width,
        height,
        slots,
        symmetry,
        name: "test".into(),
        description: String::new(),
    }
    .create()
    .unwrap()
}

/// `map` with crystal patches on `tiles` and their images.
fn with_patches(mut map: MapDef, tiles: &[(i32, i32)]) -> MapDef {
    map.spawns.extend(tiles.iter().map(|&(x, y)| patch(x, y)));
    map.regenerate();
    map
}

/// The point where a press puts a `w` × `h` footprint's top-left tile at (`x`, `y`): the footprint's center.
fn spot(x: i32, y: i32, w: u32, h: u32) -> Vec2 {
    TilePos::new(x, y).corner() + Vec2::new(w as f32, h as f32) * SU_PER_GU / 2.0
}

fn laid(kind: ObjectKind, x: i32, y: i32, orientation: BuildingOrientation) -> LayoutObject {
    LayoutObject {
        kind,
        at: SpawnAt::Tile(TilePos::new(x, y)),
        orientation,
    }
}

fn gdo(app: &App, slot: SlotId) -> Vec<LayoutObject> {
    doc(app).slot(slot).unwrap().layouts.get(Faction::Gdo).to_vec()
}

/// A click of the Layout tool at `pos`.
fn click(app: &mut App, pos: Vec2) {
    act(app, [A::Hover(Some(pos)), A::Press(pos), A::Release]);
}

/// The texts of the UI under the node marked `M`, in order.
fn texts_under<M: Component>(app: &mut App) -> Vec<String> {
    app.update(); // The lists rebuild after the actions, their texts appear a frame later.
    let mut roots = app.world_mut().query_filtered::<Entity, With<M>>();
    let root = roots.single(app.world()).unwrap();
    let mut texts = Vec::new();
    let mut stack = vec![root];
    while let Some(entity) = stack.pop() {
        if let Some(text) = app.world().get::<Text>(entity) {
            texts.push(text.0.clone());
        }
        if let Some(children) = app.world().get::<Children>(entity) {
            stack.extend(children.iter().rev());
        }
    }
    texts
}

fn status_message(app: &App) -> (StatusLevel, String) {
    (status(app).level, status(app).message.clone())
}

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

    // The other factions can be chosen, but have nothing to place yet.
    act(&mut app, [A::SetLayoutFaction(Faction::Syndicate)]);
    assert_eq!(tools(&app).faction, Faction::Syndicate);
    assert!(
        status(&app).message.contains("not implemented yet"),
        "{}",
        status(&app).message
    );
    assert_eq!(palette(doc(&app), SlotId(3), Faction::Syndicate), []);
    let before = doc(&app).clone();
    click(&mut app, at(5, 20));
    assert_warned(&app, "Syndicate: not implemented yet, its layouts stay empty");
    assert_eq!(*doc(&app), before);
    let panel = texts_under::<LayoutPanel>(&mut app);
    assert!(
        panel
            .iter()
            .any(|t| t == "Syndicate: not implemented yet, its layouts stay empty"),
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
/// The images of slot 1's layout in slot 2 follow it, turned half around.
#[test]
fn a_new_skirmish_map_lists_what_its_layouts_lack_until_it_plays() {
    let dir = temp_maps("layout_complete");
    let map = with_patches(skirmish(32, 24, 2, Symmetry::Rotate180), &[(3, 3), (4, 3)]);
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
}

/// `> Outdated layouts`: a layout that doesn't hold the faction's current Starting Conditions is flagged until the
/// map maker fixes it; nothing is fixed automatically. Here an older layout with a starting unit the GDO no longer
/// has.
#[test]
fn outdated_layouts_are_flagged_until_fixed() {
    let mut map = with_patches(skirmish(32, 24, 2, Symmetry::Rotate180), &[(3, 3), (4, 3)]);
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
