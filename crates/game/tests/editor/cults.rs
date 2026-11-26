//! The Cults' Starting Layouts in the map editor (slice M5-B, plan decision T63; `core/map_editor.md > Starting
//! Layouts`, `factions/cults/economy.md > Starting Conditions`): on a Skirmish Default map the Layout tool's palette is
//! their Recruitment Hall, Storage and six Recruits, the Recruits go where they are clicked, as units; an incomplete
//! layout lists what it lacks and the map can't be play-tested (design change 0012: "Leave the Cults layout
//! incomplete: the editor lists the missing Recruits, and the map can't be started"); a Hall flush against the map's
//! edge is valid (design change 0387, slice M5-S4); and the map plays once the layout is complete.

use bevy::prelude::App;
use sc_sim::{
    ArmyCap, ArmyResource, BuildingOrientation, Faction, LayoutObject, MapDef, ObjectKind, PlayerId, Rotation,
    SU_PER_GU, SlotId, SpawnAt, Symmetry, TilePos, Vec2,
};
use space_crystals::ClientMode;
use space_crystals::editor::actions::EditorAction as A;
use space_crystals::editor::layouts::check::LayoutPreview;
use space_crystals::editor::layouts::{PaletteEntry, palette};
use space_crystals::editor::state::StatusLevel;
use space_crystals::editor::ui::lists::ProblemList;
use space_crystals::editor::ui::starting::LayoutPanel;

use super::colonists::with_colonists;
use super::layouts::{click, laid, skirmish, spot, texts_under, with_patches};
use super::syndicate::with_syndicate;
use super::{act, doc, editor, sim, status, temp_maps, tools};

const S1: SlotId = SlotId(1);
const S2: SlotId = SlotId(2);
const HALL: ObjectKind = ObjectKind::RecruitmentHall; // 4 × 4 tiles
const STORAGE: ObjectKind = ObjectKind::Storage; // 3 × 2 (design change 0380)
const RECRUIT: ObjectKind = ObjectKind::Recruit;
const UNTURNED: BuildingOrientation = BuildingOrientation {
    rotation: Rotation::R0,
    flip_h: false,
    flip_v: false,
};

/// Where [`cults_base`] puts its Recruits, in grid units: in a row south of the Storage, 2.5 grid units and more from
/// the Hall's east door, 1 grid unit apart.
const RECRUITS: [(f32, f32); 6] = [
    (9.5, 12.5),
    (10.5, 12.5),
    (11.5, 12.5),
    (12.5, 12.5),
    (13.5, 12.5),
    (14.5, 12.5),
];

/// A point in grid units.
fn point(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y) * SU_PER_GU
}

fn recruit(x: f32, y: f32) -> LayoutObject {
    LayoutObject {
        kind: RECRUIT,
        at: SpawnAt::Point(point(x, y)),
        orientation: UNTURNED,
    }
}

/// A Cults Starting Layout for slot 1 of the 32 × 24 maps of the layout tests, clear of their other layouts: the
/// Recruitment Hall on (3, 10) to (6, 13), its 10 × 10 area, (0, 7) to (9, 16), wholly on the map (a Hall short of
/// tiles provides less Unit Control); the Storage on (8, 10) to (10, 11); and the six [`RECRUITS`]. All of it is
/// clear of the Water of the Syndicate's layout test (x = 2).
fn cults_base() -> Vec<LayoutObject> {
    let mut base = vec![laid(HALL, 3, 10, UNTURNED), laid(STORAGE, 8, 10, UNTURNED)];
    base.extend(RECRUITS.map(|(x, y)| recruit(x, y)));
    base
}

/// `map` with [`cults_base`] at slot 1 and its images: for the tests of other layouts on a Skirmish Default map,
/// which can't be played without it (slice M5-B, plan decision T63).
pub(super) fn with_cults(mut map: MapDef) -> MapDef {
    *map.slot_mut(S1).unwrap().layouts.get_mut(Faction::Cults) = cults_base();
    map.regenerate();
    map
}

fn cults(app: &App, slot: SlotId) -> Vec<LayoutObject> {
    doc(app).slot(slot).unwrap().layouts.get(Faction::Cults).to_vec()
}

/// Slice M5-B's acceptance: "An incomplete Cults layout lists what it lacks, in the editor and in validation". On a new
/// Skirmish Default map (here with the GDO's, the Syndicate's and the Colonists' layouts done) the Problems list says
/// what the Cults' layout lacks; the palette offers the Recruitment Hall, the Storage and six Recruits, and moves on to
/// the next kind as each is done; a Hall's ghost flush against the map's edge is valid (design change 0387's
/// Verification: "Place a Hall flush against the map edge: it is accepted"; slice M5-J1 refused it, design change 0341
/// point 1); a Recruit goes exactly where it is clicked, a point. Two Recruits short, the Layout panel and the Problems
/// list name them, and the map can't be play-tested (design change 0012). Complete, slot 2's layout is the image, and
/// the map plays: slot 2's Cults player starts with it, each Recruit of its Hall's lineage, at Unit Control 6/20.
#[test]
fn the_cults_layout_lists_its_missing_recruits_until_complete() {
    let dir = temp_maps("cults_layout");
    let mut map = with_colonists(with_syndicate(with_patches(
        skirmish(32, 24, 2, Symmetry::Rotate180),
        &[(3, 3), (4, 3)],
    )));
    *map.slot_mut(S1).unwrap().layouts.get_mut(Faction::Gdo) = vec![
        laid(ObjectKind::DeploymentCenter, 2, 6, UNTURNED),
        laid(ObjectKind::ExtractionFacility, 7, 6, UNTURNED),
        laid(ObjectKind::ExtractionPlate, 3, 3, UNTURNED),
        laid(ObjectKind::ExtractionPlate, 4, 3, UNTURNED),
    ];
    map.regenerate();
    let mut app = editor(map, &dir);
    assert_eq!(
        texts_under::<ProblemList>(&mut app),
        ["S1 Cults layout: add Recruitment Hall, Storage, Recruit x6 (Layout tool, T); S2 follows"]
    );

    act(&mut app, [A::SetLayoutFaction(Faction::Cults)]);
    assert_eq!(
        status(&app).message,
        "Editing the S1 Cults layout: missing Recruitment Hall, Storage, Recruit x6"
    );
    let entry = |kind, left| PaletteEntry { kind, left: Some(left) };
    assert_eq!(
        palette(doc(&app), S1, Faction::Cults),
        [entry(HALL, 1), entry(STORAGE, 1), entry(RECRUIT, 6)]
    );
    let panel = texts_under::<LayoutPanel>(&mut app);
    for expected in ["Recruitment Hall: 1 left", "Storage: 1 left", "Recruit: 6 left"] {
        assert!(panel.iter().any(|t| t == expected), "{expected:?} not in {panel:?}");
    }
    assert_eq!(tools(&app).layout_kind, HALL);
    // Flush against the west edge, 3 columns of its Recruitment Area off the map (its image flush against the east).
    act(&mut app, [A::Hover(Some(spot(0, 8, 4, 4)))]);
    app.update();
    let ghost = app.world().resource::<LayoutPreview>().0.clone().expect("a ghost");
    assert_eq!((ghost.at, ghost.verdict), (SpawnAt::Tile(TilePos::new(0, 8)), Ok(())));
    click(&mut app, spot(3, 10, 4, 4));
    assert_eq!(tools(&app).layout_kind, STORAGE, "on to the next kind the layout lacks");
    click(&mut app, spot(8, 10, 3, 2));
    assert_eq!(tools(&app).layout_kind, RECRUIT);
    for (x, y) in &RECRUITS[..4] {
        click(&mut app, point(*x, *y));
    }

    // Two Recruits short: listed, and no play-test.
    let missing = "S1 Cults layout: add Recruit x2 (Layout tool, T); S2 follows";
    assert_eq!(texts_under::<ProblemList>(&mut app), [missing]);
    let panel = texts_under::<LayoutPanel>(&mut app);
    assert!(
        panel.iter().any(|t| t == "S1 Cults layout: missing Recruit x2"),
        "{panel:?}"
    );
    assert!(panel.iter().any(|t| t == "Recruit: 2 left"), "{panel:?}");
    act(&mut app, [A::PlayTest]);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::Editor);
    assert_eq!(
        (status(&app).level, status(&app).message.clone()),
        (StatusLevel::Error, format!("Can't play-test: {missing}"))
    );

    for (x, y) in &RECRUITS[4..] {
        click(&mut app, point(*x, *y));
    }
    assert_eq!(cults(&app, S1), cults_base(), "the Recruits at the points clicked");
    // Slot 2's is the image: the footprints through the map's center, turned half around, and the Recruits' points.
    let half = BuildingOrientation {
        rotation: Rotation::R180,
        ..UNTURNED
    };
    let mut image = vec![laid(HALL, 25, 10, half), laid(STORAGE, 21, 12, half)];
    image.extend(RECRUITS.map(|(x, y)| recruit(32.0 - x, 24.0 - y)));
    assert_eq!(cults(&app, S2), image);
    let panel = texts_under::<LayoutPanel>(&mut app);
    assert!(panel.iter().any(|t| t == "S1 Cults layout: complete"), "{panel:?}");
    assert_eq!(texts_under::<ProblemList>(&mut app), ["None: the map is playable"]);
    assert_eq!(doc(&app).outdated_layouts(), []);

    act(&mut app, [A::SetPlayTestFaction(S2, Faction::Cults), A::PlayTest]);
    assert_eq!(*app.world().resource::<ClientMode>(), ClientMode::PlayTest);
    let p2 = PlayerId(2);
    let base: Vec<(ObjectKind, usize)> = sim(&app)
        .objects()
        .filter(|o| o.owner == Some(p2))
        .map(|o| (o.kind, o.lineage.len()))
        .collect();
    let mut expected = vec![(HALL, 0), (STORAGE, 0)];
    expected.extend([(RECRUIT, 1); 6]);
    assert_eq!(base, expected);
    let cap = ArmyCap {
        resource: ArmyResource::CultsUnitControl,
        used: 6,
        available: 20,
    };
    assert_eq!(sim(&app).army_cap(p2), Some(cap));
    let _ = std::fs::remove_dir_all(&dir);
}
