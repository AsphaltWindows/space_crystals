//! What the selection looks like: the InfoPanel and the SelectionPanel (`systems/controls/hud.md > HUD
//! Bar`, `systems/controls/hud.md > SelectionPanel`), clicks on portraits
//! (`systems/controls/hud.md > SelectionPanel > Interactions`), and the markers under selected objects.

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use sc_sim::{Command, ConstructionCommand, DebugCommand, PlayerCommand};
use space_crystals::hud::info_panel::InfoText;
use space_crystals::hud::layout::PortraitGrid;
use space_crystals::hud::selection_panel::{PortraitCell, SelectionPanelView};
use space_crystals::hud::{HudSection, SectionLabel};
use space_crystals::selection::Relation;
use space_crystals::selection::markers::SelectionMarker;
use space_crystals::sim_driver::PendingCommands;

use super::*;
use KeyCode::{AltLeft, ControlLeft, ShiftLeft, Tab};

/// The InfoPanel's title and lines, while it shows something. (An own structure's activity area is tested in
/// `hud.rs`.)
fn info(app: &mut App) -> Option<(String, String)> {
    let mut texts = app.world_mut().query::<(&InfoText, &Text, &Node)>();
    let mut title = None;
    let mut body = None;
    for (which, text, node) in texts.iter(app.world()) {
        if node.display == Display::None {
            continue;
        }
        match which {
            InfoText::Title => title = Some(text.0.clone()),
            InfoText::Body => body = Some(text.0.clone()),
            InfoText::Activity => {}
        }
    }
    title.zip(body)
}

/// Whether `section` shows its name (it has nothing else to show).
fn label_shown(app: &mut App, section: HudSection) -> bool {
    let mut labels = app.world_mut().query::<(&SectionLabel, &Node)>();
    labels
        .iter(app.world())
        .any(|(label, node)| label.0 == section && node.display != Display::None)
}

/// The portraits shown, as (object, active) in their order, from the panel's entities.
fn portraits(app: &mut App) -> Vec<(u32, bool)> {
    let cells = app.world().resource::<SelectionPanelView>().cells.clone();
    let mut query = app.world_mut().query::<&PortraitCell>();
    assert_eq!(query.iter(app.world()).count(), cells.len(), "one entity per cell");
    cells
        .iter()
        .map(|cell| match *cell {
            PortraitCell::Object { id, active, .. } => (id.0, active),
            PortraitCell::More { .. } => panic!("no overflow here"),
        })
        .collect()
}

/// A click on portrait `index` with the modifier keys `modifiers` held.
fn click_portrait(app: &mut App, index: usize, modifiers: &[KeyCode]) {
    let surface = app.world().resource::<SurfaceSize>().logical();
    let point = PortraitGrid::on(surface).cell_rect(index).center();
    click_screen(app, point, modifiers, 1);
}

fn markers(app: &mut App) -> Vec<(u32, Relation, bool)> {
    let mut query = app.world_mut().query::<&SelectionMarker>();
    let mut markers: Vec<(u32, Relation, bool)> = query
        .iter(app.world())
        .map(|m| (m.id.0, m.relation, m.active))
        .collect();
    markers.sort_by_key(|m| m.0);
    markers
}

/// The InfoPanel's activity area, while it shows something.
fn activity(app: &mut App) -> Option<String> {
    let mut texts = app.world_mut().query::<(&InfoText, &Text, &Node)>();
    texts
        .iter(app.world())
        .find(|(which, _, node)| **which == InfoText::Activity && node.display != Display::None)
        .map(|(_, text, _)| text.0.clone())
}

/// How far the mesh of object `id`'s marker reaches across and down.
fn marker_size(app: &mut App, id: u32) -> Vec2 {
    let mut markers = app.world_mut().query::<(&SelectionMarker, &Mesh2d)>();
    let handle = markers
        .iter(app.world())
        .find(|(marker, _)| marker.id == ObjectId(id))
        .map(|(_, mesh)| mesh.0.clone())
        .expect("a marker");
    let mesh = app.world().resource::<Assets<Mesh>>().get(&handle).unwrap();
    let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
        panic!("a 2D mesh has positions");
    };
    let (lo, hi) = positions
        .iter()
        .fold((Vec2::INFINITY, Vec2::NEG_INFINITY), |(lo, hi), p| {
            let p = Vec2::new(p[0], p[1]);
            (lo.min(p), hi.max(p))
        });
    hi - lo
}

/// Plan decision T15 (`SimEvent::KindChanged`, slice M3-E): a selected object that changes its kind in place stays
/// selected, with its id, as its new kind, and the panels show its new type. Player 1's Beacon 12 (2 × 2 tiles, a
/// debug spawn), selected, warps in a Robo Portal (4 × 4): the InfoPanel names the Robo Portal and shows the Warp In
/// in its activity area, and the marker's frame takes the new footprint (inset 4 space units all round). Cancelled,
/// it is a selected Beacon again.
#[test]
fn a_selected_beacon_stays_selected_through_its_warp_in() {
    let mut app = game();
    let player = PlayerId(1);
    let push = |app: &mut App, command| {
        app.world_mut()
            .resource_mut::<PendingCommands>()
            .0
            .push(PlayerCommand { player, command })
    };
    let spawn = DebugCommand::Spawn {
        kind: ObjectKind::Beacon,
        owner: Some(player),
        at: SpawnAt::Tile(TilePos::new(18, 15)),
        orientation: Default::default(),
    };
    let funds = DebugCommand::SetResources {
        player,
        crystals: 1000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    push(&mut app, Command::Debug(spawn));
    push(&mut app, Command::Debug(funds));
    app.update();
    assert!(object_exists(&app, 12));
    click(&mut app, Vec2::new(19.0, 16.0) * 64.0, &[]);
    assert_eq!(selected(&app), [12]);
    let title = |app: &mut App| info(app).map(|(title, _)| title);
    assert_eq!(title(&mut app).as_deref(), Some("Beacon"));
    assert_eq!(marker_size(&mut app, 12), Vec2::splat(2.0 * 64.0 - 8.0));
    let construction = |command| Command::Construction {
        objects: vec![ObjectId(12)],
        command,
    };
    let warp_in = ConstructionCommand::WarpIn {
        kind: ObjectKind::RoboPortal,
        orientation: Default::default(),
    };
    push(&mut app, construction(warp_in));
    app.update();
    let kind = |app: &App| {
        let states = app.world().resource::<ControlStates>();
        states.selection(player).kind_of(ObjectId(12))
    };
    assert_eq!(kind(&app), Some(ObjectKind::RoboPortal));
    assert_eq!((selected(&app), active_group(&app)), (vec![12], vec![12]));
    assert_eq!(title(&mut app).as_deref(), Some("Robo Portal"));
    assert_eq!(activity(&mut app).as_deref(), Some("Warping in: Robo Portal 0%"));
    assert_eq!(marker_size(&mut app, 12), Vec2::splat(4.0 * 64.0 - 8.0));
    push(&mut app, construction(ConstructionCommand::CancelTransform));
    app.update();
    assert_eq!((kind(&app), selected(&app)), (Some(ObjectKind::Beacon), vec![12]));
    assert_eq!(title(&mut app).as_deref(), Some("Beacon"));
    assert_eq!(activity(&mut app), None);
    assert_eq!(marker_size(&mut app, 12), Vec2::splat(2.0 * 64.0 - 8.0));
}

/// `systems/controls/hud.md > HUD Bar > Sections`: the InfoPanel shows from one selected object, the
/// SelectionPanel from two.
#[test]
fn the_info_panel_shows_the_selected_object() {
    let mut app = game();
    assert_eq!(info(&mut app), None);
    assert!(label_shown(&mut app, HudSection::InfoPanel));
    click(&mut app, at(1), &[]);
    assert_eq!(
        info(&mut app),
        Some(("Peacekeeper".into(), "Player 1\nHP 50 / 50".into()))
    );
    assert!(!label_shown(&mut app, HudSection::InfoPanel));
    assert!(
        label_shown(&mut app, HudSection::SelectionPanel),
        "one object: no portraits"
    );
    click(&mut app, at(ENEMY_GUARD), &[]);
    assert_eq!(info(&mut app), Some(("Guard".into(), "Player 2\nHP 80 / 80".into())));
    // `core/map.md > SpaceCrystalsPatch > InfoPanel` and `core/map.md > SupplyDeliveryStation > InfoPanel`.
    click(&mut app, at(PATCH), &[]);
    assert_eq!(
        info(&mut app),
        Some(("Space Crystals".into(), "Neutral\nSpace Crystals: 1500".into()))
    );
    click(&mut app, at(STATION), &[]);
    let (title, lines) = info(&mut app).unwrap();
    assert_eq!(title, "Supply Delivery Station");
    assert!(
        lines.starts_with("Neutral\nSupplies: 0\nDelivery size: 4\nDelivery interval: 60 s\nNext delivery in "),
        "{lines}"
    );
    // Two or more: the ActiveGroup's type (the placeholder kept by design change 0054).
    click(&mut app, at(1), &[ControlLeft]);
    assert_eq!(
        info(&mut app),
        Some(("Peacekeeper".into(), "3 in group\n3 selected".into()))
    );
    click(&mut app, Vec2::new(1500.0, 1200.0), &[]);
    assert_eq!(info(&mut app), None);
    assert!(label_shown(&mut app, HudSection::InfoPanel));
}

#[test]
fn the_selection_panel_shows_two_or_more_and_highlights_the_active_group() {
    let mut app = game();
    click(&mut app, at(1), &[]);
    assert!(portraits(&mut app).is_empty());
    drag(&mut app, Vec2::new(850.0, 750.0), Vec2::new(1400.0, 1150.0));
    // By group (type order), each by id; the active group's highlighted.
    let pk = |id| (id, true);
    assert_eq!(portraits(&mut app), [pk(1), pk(2), pk(5), (3, false), (4, false)]);
    assert!(!label_shown(&mut app, HudSection::SelectionPanel));
    tap(&mut app, &[], Tab);
    assert_eq!(
        portraits(&mut app),
        [(1, false), (2, false), (5, false), (3, true), (4, true)]
    );
    // Down to one object, the panel empties and the InfoPanel shows it (`systems/controls/hud.md >
    // SelectionPanel > Edge cases`).
    click(&mut app, at(1), &[]);
    assert!(portraits(&mut app).is_empty());
    assert!(label_shown(&mut app, HudSection::SelectionPanel));
}

/// `systems/controls/hud.md > SelectionPanel` (design change 0055): the portraits come group by group, in
/// one order whatever order the objects were selected in, and Tab and Shift-Tab move the ActiveGroup's highlight along
/// it, wrapping around.
#[test]
fn tab_and_shift_tab_follow_the_order_of_the_portraits() {
    let mut app = game();
    let order = |app: &mut App| portraits(app).into_iter().map(|(id, _)| id).collect::<Vec<_>>();
    let highlighted = |app: &mut App| {
        let cells = portraits(app).into_iter();
        cells
            .filter(|&(_, active)| active)
            .map(|(id, _)| id)
            .collect::<Vec<_>>()
    };
    // The Guards first, then the Peacekeepers added: the panel still shows the Peacekeepers first.
    click(&mut app, at(3), &[ControlLeft]);
    click(&mut app, at(1), &[ControlLeft, ShiftLeft]);
    assert_eq!(order(&mut app), [1, 2, 5, 3, 4]);
    assert_eq!(
        highlighted(&mut app),
        OWN_GUARDS,
        "the Guards are still the ActiveGroup"
    );
    tap(&mut app, &[], Tab);
    assert_eq!(highlighted(&mut app), OWN_PKS, "past the last group, the first");
    tap(&mut app, &[], Tab);
    assert_eq!(highlighted(&mut app), OWN_GUARDS);
    tap(&mut app, &[ShiftLeft], Tab);
    assert_eq!(highlighted(&mut app), OWN_PKS);
    tap(&mut app, &[ShiftLeft], Tab);
    assert_eq!(highlighted(&mut app), OWN_GUARDS, "before the first group, the last");
    assert_eq!(order(&mut app), [1, 2, 5, 3, 4], "cycling keeps the order");
}

/// `systems/controls/hud.md > SelectionPanel > Interactions`, with portraits 0–4 showing units 1, 2, 5
/// (Peacekeepers), 3 and 4 (Guards).
#[test]
fn portrait_clicks_follow_the_panels_rules() {
    let mut app = game();
    let select_all = |app: &mut App| {
        drag(app, Vec2::new(850.0, 750.0), Vec2::new(1400.0, 1150.0));
        assert_eq!(selected(app), ids([1, 2, 3, 4, 5]));
    };
    select_all(&mut app);
    // Alt: the camera centers on that unit, the selection stays.
    click_portrait(&mut app, 3, &[AltLeft]);
    assert_eq!(camera_center(&app), at(3));
    assert_eq!(selected(&app), ids([1, 2, 3, 4, 5]));
    click_portrait(&mut app, 1, &[ShiftLeft]);
    assert_eq!(selected(&app), ids([1, 5, 3, 4]), "Shift removes that unit");
    click_portrait(&mut app, 3, &[ControlLeft]);
    assert_eq!(selected(&app), OWN_GUARDS, "Ctrl keeps that unit's type");
    select_all(&mut app);
    click_portrait(&mut app, 0, &[ControlLeft, ShiftLeft]);
    assert_eq!(selected(&app), OWN_GUARDS, "Ctrl + Shift removes that unit's type");
    select_all(&mut app);
    click_portrait(&mut app, 4, &[]);
    assert_eq!(selected(&app), [4], "a click keeps only that unit");
    assert!(portraits(&mut app).is_empty());
    // A click where no portrait is changes nothing.
    click_portrait(&mut app, 0, &[]);
    assert_eq!(selected(&app), [4]);
}

/// The markers under selected objects: by relation to the local player, the ActiveGroup's apart.
#[test]
fn markers_follow_the_selection() {
    let mut app = game();
    assert!(markers(&mut app).is_empty());
    drag(&mut app, Vec2::new(850.0, 750.0), Vec2::new(1400.0, 1150.0));
    let own = |id, active| (id, Relation::Own, active);
    assert_eq!(
        markers(&mut app),
        [own(1, true), own(2, true), own(3, false), own(4, false), own(5, true)]
    );
    tap(&mut app, &[], Tab);
    assert_eq!(
        markers(&mut app),
        [own(1, false), own(2, false), own(3, true), own(4, true), own(5, false)]
    );
    click(&mut app, at(ENEMY_PK), &[]);
    assert_eq!(markers(&mut app), [(ENEMY_PK, Relation::Enemy, true)]);
    click(&mut app, at(PATCH), &[]);
    assert_eq!(markers(&mut app), [(PATCH, Relation::Neutral, true)]);
    click(&mut app, Vec2::new(1500.0, 1200.0), &[]);
    assert!(markers(&mut app).is_empty());
}
