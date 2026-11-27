//! The Minimap's clicks (slice M5-MM, design change 0415, `systems/controls/hud.md > Minimap > Using it`), through the
//! virtual mouse at the points the minimap shows ([`MinimapFrame`]): a left-click or a drag centres the camera, kept on
//! the map as ever; a right-click is the map's right-click on that ground point, and while a target is awaited a
//! left-click is the map's targeting click there, Shift too; in AwaitingPlacement a left-click centres the camera and
//! places nothing (the Director's reading); and no press on it ever changes the selection. Also the panel itself, with
//! the fog turned on in a paused game: the image drawn at once and the camera's outline following the camera.

use bevy::prelude::*;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Faction, Map, ObjectKind, OrderKind, PlayerId, SpawnAt, TilePos, VisibilityState};
use space_crystals::camera::{
    CameraView, ViewportInsets, clamp_center, map_extent_su, viewport_rect, visible_extent_su,
};
use space_crystals::command_ui::{InterfaceState, PanelCommand};
use space_crystals::coords::{drawn_bounds, drawn_position};
use space_crystals::fog::{FogOfWar, Sight};
use space_crystals::hud::minimap::raster::UNEXPLORED;
use space_crystals::hud::minimap::{MinimapFrame, MinimapImage, MinimapView};
use space_crystals::hud::{HudSection, SectionLabel};
use space_crystals::platform::SurfaceSize;
use space_crystals::sim_driver::{SimControl, SimRes};

use super::*;
use KeyCode::{ArrowRight, ControlLeft, KeyA, KeyW, ShiftLeft};

const AWAIT_ATTACK: InterfaceState = InterfaceState::awaiting(PanelCommand::Attack);
/// Bare ground in the world view of [`game`], where neither the map nor the minimap has anything to click but the
/// ground.
const GROUND: Vec2 = Vec2::new(2000.0, 1000.0);

/// The minimap's frame in `app`.
fn frame(app: &App) -> MinimapFrame {
    let size = *app.world().resource::<SurfaceSize>();
    MinimapFrame::on(size, map_extent_su(app.world().resource::<SimRes>().map()))
}

/// Where the camera looks.
fn camera(app: &App) -> Vec2 {
    app.world().resource::<CameraView>().center
}

/// Where the camera looks once centred on the ground point `ground`: at its drawn position, kept on the map
/// (`clamp_center`) as the camera keeps every view.
fn centred_on(app: &App, ground: Vec2) -> Vec2 {
    let size = *app.world().resource::<SurfaceSize>();
    let insets = *app.world().resource::<ViewportInsets>();
    let map: &Map = app.world().resource::<SimRes>().map();
    let (_, viewport) = viewport_rect(size, insets);
    let visible = visible_extent_su(viewport, size.physical.x, 1.0);
    clamp_center(drawn_position(map, ground), visible, drawn_bounds(map))
}

/// The orders a fresh [`game`] gives after `act`, with player 1's Peacekeepers 1 and 2 and Guards 3 and 4 selected.
fn orders_after(act: impl FnOnce(&mut App)) -> Vec<Given> {
    let mut app = game();
    select_mixed(&mut app);
    act(&mut app);
    updates(&mut app, 2);
    orders(&app)
}

/// Whether `a` and `b` are the same orders to the same units, queued alike, at the same points but for the cursor's
/// rounding.
fn same_orders(a: &[Given], b: &[Given]) -> bool {
    let same_kind = |a: OrderKind, b: OrderKind| match (a, b) {
        (OrderKind::Move(sc_sim::Target::Point(p)), OrderKind::Move(sc_sim::Target::Point(q)))
        | (OrderKind::AttackMove(p), OrderKind::AttackMove(q)) => near(p, q),
        _ => a == b,
    };
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(a, b)| (a.player, &a.units, a.queue) == (b.player, &b.units, b.queue) && same_kind(a.kind, b.kind))
}

/// Change 0415's Verification: "clicking the minimap moves the camera there". A left-click centres the camera on the
/// ground point the minimap shows there (`core/camera.md > Window Mode > Centering`: it snaps), in the map's middle
/// exactly, and near its corners as near as the camera goes, kept on the map as ever; the camera's outline goes
/// with it. The selection stays.
#[test]
fn a_left_click_on_the_minimap_centres_the_camera_there() {
    let mut app = game();
    select_mixed(&mut app);
    let frame = frame(&app);
    let middle = frame.screen_at(Vec2::new(2600.0, 1900.0));
    click_with(&mut app, MouseButton::Left, middle, &[]);
    assert_eq!(
        camera(&app),
        frame.ground_at(middle),
        "the flat map is drawn where it lies"
    );
    let outline = app.world().resource::<MinimapView>().camera_rect.expect("on the map");
    assert!(outline.center().distance(middle) <= 1.0, "{outline:?} around {middle}");
    for corner in [Vec2::new(40.0, 3000.0), Vec2::new(4000.0, 20.0)] {
        let screen = frame.screen_at(corner);
        click_with(&mut app, MouseButton::Left, screen, &[]);
        assert_eq!(
            camera(&app),
            centred_on(&app, frame.ground_at(screen)),
            "kept on the map"
        );
        assert_ne!(camera(&app), frame.ground_at(screen));
    }
    assert_eq!(selected(&app), [1, 2, 3, 4]);
}

/// "dragging pans continuously": the camera follows the cursor frame by frame while the button is held, over the
/// world view too, where the point is the map's edge nearest the cursor; no selection box is drawn; once the button
/// is up the cursor moves nothing.
#[test]
fn a_left_drag_on_the_minimap_pans_continuously() {
    let mut app = game();
    select_mixed(&mut app);
    let frame = frame(&app);
    let path = [
        Vec2::new(1000.0, 1000.0),
        Vec2::new(1800.0, 1400.0),
        Vec2::new(3000.0, 2000.0),
    ]
    .map(|ground| frame.screen_at(ground));
    input(&mut app).cursor = Some(path[0]);
    input(&mut app).mouse.press(MouseButton::Left);
    app.update();
    assert_eq!(camera(&app), centred_on(&app, frame.ground_at(path[0])));
    let above = Vec2::new(path[2].x, frame.area.min.y - 30.0);
    for screen in [path[1], path[2], above] {
        input(&mut app).cursor = Some(screen);
        app.update();
        assert_eq!(
            camera(&app),
            centred_on(&app, frame.ground_at(screen)),
            "follows the drag to {screen}"
        );
        assert!(!box_drawn(&mut app), "no selection box");
    }
    assert_eq!(
        frame.ground_at(above).y,
        0.0,
        "over the world view: the map's north edge"
    );
    input(&mut app).mouse.release(MouseButton::Left);
    app.update();
    let released = camera(&app);
    input(&mut app).cursor = Some(path[1]);
    updates(&mut app, 2);
    assert_eq!(camera(&app), released);
    assert_eq!(selected(&app), [1, 2, 3, 4]);
}

/// "Select units, right-click the minimap: they move to that spot": a right-click on the minimap gives the same
/// commands as a right-click on that ground point in the world view, a Move of the four units there, queued with
/// Shift; it moves no camera.
#[test]
fn a_right_click_on_the_minimap_is_the_map_s_right_click_on_that_ground() {
    for modifiers in [&[][..], &[ShiftLeft]] {
        let mut point = Vec2::ZERO;
        let on_minimap = orders_after(|app| {
            let screen = frame(app).screen_at(GROUND);
            point = frame(app).ground_at(screen);
            click_with(app, MouseButton::Right, screen, modifiers);
            assert_eq!(camera(app), VIEW_CENTER, "a right-click moves no camera");
        });
        let on_map = orders_after(|app| right_click(app, point, modifiers));
        assert!(same_orders(&on_minimap, &on_map), "{on_minimap:?} against {on_map:?}");
        let queue = !modifiers.is_empty();
        assert!(matches!(&on_minimap[..], [order] if order.units == [1, 2, 3, 4] && order.queue == queue));
        assert!(near(point_of(on_minimap[0].kind), GROUND), "{on_minimap:?}");
    }
}

/// "Press A, left-click the minimap: they Attack Move there": the same AttackMove as a click on that ground point in
/// the world view, and back to DefaultState; a Shift-click queues it and keeps the target awaited until Shift is let
/// go, as on the map (design change 0070). The targeting click moves no camera, and a press that moves beyond the
/// click allowance gives nothing, the target still awaited.
#[test]
fn a_targeting_click_on_the_minimap_is_the_map_s_click_on_that_ground() {
    for modifiers in [&[][..], &[ShiftLeft]] {
        let mut point = Vec2::ZERO;
        let on_minimap = orders_after(|app| {
            tap(app, &[], KeyA);
            assert_eq!(interface(app), AWAIT_ATTACK);
            let screen = frame(app).screen_at(GROUND);
            point = frame(app).ground_at(screen);
            click_with(app, MouseButton::Left, screen, modifiers);
            assert_eq!(interface(app), InterfaceState::DefaultState);
            assert_eq!(camera(app), VIEW_CENTER, "a targeting click moves no camera");
        });
        let on_map = orders_after(|app| {
            tap(app, &[], KeyA);
            click(app, point, modifiers);
        });
        assert!(same_orders(&on_minimap, &on_map), "{on_minimap:?} against {on_map:?}");
        let queue = !modifiers.is_empty();
        assert!(matches!(&on_minimap[..], [order] if order.units == [1, 2, 3, 4] && order.queue == queue));
        assert!(matches!(on_minimap[0].kind, OrderKind::AttackMove(p) if near(p, GROUND)));
    }
    let dragged = orders_after(|app| {
        tap(app, &[], KeyA);
        let screen = frame(app).screen_at(GROUND);
        input(app).cursor = Some(screen);
        input(app).mouse.press(MouseButton::Left);
        app.update();
        input(app).cursor = Some(screen + Vec2::new(30.0, 0.0));
        app.update();
        input(app).mouse.release(MouseButton::Left);
        app.update();
        assert_eq!(interface(app), AWAIT_ATTACK, "still awaited");
        assert_eq!(camera(app), VIEW_CENTER);
    });
    assert_eq!(dragged, []);
}

/// "A press on the minimap belongs to the minimap": with nothing selected a click there selects nothing, and with
/// units selected no click there changes the selection, whatever the modifiers (Shift and Ctrl add and select by type
/// on the map), nor a double-click, nor a right-click.
#[test]
fn a_press_on_the_minimap_never_changes_the_selection() {
    let mut app = game();
    let screen = frame(&app).screen_at(GROUND);
    click_with(&mut app, MouseButton::Left, screen, &[]);
    assert_eq!(selected(&app), Vec::<u32>::new());
    app.world_mut().resource_mut::<CameraView>().center = VIEW_CENTER;
    select_mixed(&mut app);
    for modifiers in [&[][..], &[ShiftLeft], &[ControlLeft]] {
        click_with(&mut app, MouseButton::Left, screen, modifiers);
        assert_eq!(selected(&app), [1, 2, 3, 4], "{modifiers:?}");
        click_with(&mut app, MouseButton::Right, screen, modifiers);
        assert_eq!(selected(&app), [1, 2, 3, 4], "{modifiers:?}");
    }
    input(&mut app).cursor = Some(screen);
    for _ in 0..2 {
        input(&mut app).mouse.press(MouseButton::Left);
        app.update();
        input(&mut app).mouse.release(MouseButton::Left);
        app.update();
    }
    assert_eq!(selected(&app), [1, 2, 3, 4], "a double-click");
}

/// Player 1's Drone, 3, at the centre of tile (20, 17), in the world view of a Colonists game set up as [`game`] does,
/// with each player's Egress Node to keep them in it.
fn drone_game() -> App {
    let (p1, p2) = (PlayerId(1), PlayerId(2));
    let map = vec![".".repeat(64); 48].join("\n");
    let scenario = ScenarioBuilder::new(&map)
        .faction(p1, Faction::Colonists)
        .faction(p2, Faction::Colonists)
        .spawn(ObjectKind::EgressNode, Some(p1), SpawnAt::Tile(TilePos::new(12, 9)))
        .spawn(ObjectKind::EgressNode, Some(p2), SpawnAt::Tile(TilePos::new(56, 40)))
        .spawn(
            ObjectKind::Drone,
            Some(p1),
            SpawnAt::Point(TilePos::new(20, 17).center()),
        )
        .build();
    game_of(scenario)
}

/// "Building placement (AwaitingPlacement) is not possible from the minimap"; the Director's reading: a left-click on
/// it centres the camera and never places. A Drone's Deploy (W) awaits its Beacon's placement: a click on the minimap
/// moves the camera, gives no Deploy and keeps the placement; a right-click there leaves it, as anywhere.
#[test]
fn in_awaiting_placement_a_minimap_click_centres_the_camera_and_places_nothing() {
    let mut app = drone_game();
    click(&mut app, TilePos::new(20, 17).center(), &[]);
    assert_eq!(selected(&app), [3]);
    tap(&mut app, &[], KeyW);
    let placing = interface(&app).placing();
    assert!(placing.is_some(), "{:?}", interface(&app));
    let frame = frame(&app);
    let screen = frame.screen_at(GROUND);
    click_with(&mut app, MouseButton::Left, screen, &[]);
    assert_eq!(camera(&app), frame.ground_at(screen), "centred there");
    assert_eq!(interface(&app).placing(), placing, "still placing");
    assert_eq!(orders(&app), [], "no Deploy");
    click_with(&mut app, MouseButton::Right, screen, &[]);
    assert_eq!(interface(&app), InterfaceState::DefaultState);
    assert_eq!(orders(&app), []);
}

/// The panel in the game: the image node shows the minimap's image at the frame, in place of the section's name, and
/// with the sim paused, turning the fog on redraws it at once: the far corner, which no unit of player 1's has seen,
/// turns black. The camera's outline follows the camera as it pans ("The rectangle follows the camera as you pan").
#[test]
fn the_minimap_shows_the_fog_and_the_camera_s_outline() {
    let mut app = game();
    let frame = frame(&app);
    let (image, node, label) = {
        let world = app.world_mut();
        let mut nodes = world.query_filtered::<(&ImageNode, &Node), With<MinimapImage>>();
        let (image, node) = nodes.single(world).expect("one minimap");
        let (image, width) = (image.image.clone(), node.width);
        let mut labels = world.query::<(&SectionLabel, &Node)>();
        let label = labels
            .iter(world)
            .find(|(l, _)| l.0 == HudSection::Minimap)
            .map(|(_, n)| n.display);
        (image, width, label)
    };
    assert_eq!(Some(&image), app.world().resource::<MinimapView>().image.as_ref());
    assert_eq!(node, Val::Px(frame.area.width()));
    assert_eq!(label, Some(Display::None), "the section's name gives way to the map");
    let corner = |app: &App| {
        let images = app.world().resource::<Assets<Image>>();
        let data = images.get(&image).and_then(|i| i.data.clone()).expect("drawn");
        let i = ((frame.texels.x - 1) * 4) as usize;
        <[u8; 4]>::try_from(&data[i..i + 4]).unwrap()
    };
    assert_ne!(corner(&app), UNEXPLORED, "the fog is off");
    let sim = app.world().resource::<SimRes>();
    let state = Sight::new(sim, PlayerId(1), true).tile(TilePos::new(63, 0));
    assert_eq!(state, VisibilityState::Unexplored, "the far corner, unseen");
    app.world_mut().resource_mut::<SimControl>().paused = true;
    app.update();
    let redraws = app.world().resource::<MinimapView>().redraws;
    app.world_mut().resource_mut::<FogOfWar>().enabled = true;
    app.update();
    assert_eq!(app.world().resource::<MinimapView>().redraws, redraws + 1, "at once");
    assert_eq!(corner(&app), UNEXPLORED, "unexplored: black");

    let outline = |app: &App| app.world().resource::<MinimapView>().camera_rect.expect("shown");
    assert!(outline(&app).center().distance(frame.screen_at(camera(&app))) <= 1.0);
    press_keys(&mut app, &[ArrowRight]);
    updates(&mut app, 8);
    release_keys(&mut app, &[ArrowRight]);
    app.update();
    assert!(camera(&app).x > VIEW_CENTER.x + 100.0, "panned");
    assert!(
        outline(&app).center().distance(frame.screen_at(camera(&app))) <= 1.0,
        "the outline follows"
    );
}
