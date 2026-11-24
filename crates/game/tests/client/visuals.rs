//! Visuals (`ARCHITECTURE.md > Client architecture > Visuals`): objects on the terrain and their picking, one
//! visual per object, draw order, facing, the grid overlay, loading another map, and interpolation between
//! ticks.

use std::collections::BTreeSet;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Sim, SpawnAt, Target, TilePos,
    scenarios,
};
use space_crystals::camera::{CameraView, ViewportInsets, cursor_to_ground, viewport_rect};
use space_crystals::coords::{drawn_position, lift};
use space_crystals::input::InputAction;
use space_crystals::map_render::{GridOverlay, MapRenderAssets, MapView};
use space_crystals::object_render::{FacingTick, ObjectVisuals};
use space_crystals::platform::SurfaceSize;
use space_crystals::sim_driver::{PendingCommands, RecordedInputs, RenderAlpha, SimRes, load_scenario};

use super::{
    camera_center, debug, headless_app, patch_at, plateau_scenario, press, sim_ids, small_scenario, tick, updates,
    visual_ids,
};

/// Objects stand on the terrain (drawn lifted by its height), and every pick resolves to the ground drawn under
/// the cursor.
#[test]
fn objects_and_picking_follow_the_terrain_height() {
    let mut app = headless_app(plateau_scenario());
    updates(&mut app, 2);
    let patch = app.world().resource::<SimRes>().object(ObjectId(1)).unwrap().pos;
    let entity = app.world().resource::<ObjectVisuals>().by_id[&ObjectId(1)];
    let translation = app.world().get::<Transform>(entity).unwrap().translation;
    assert_eq!(translation.truncate(), Vec2::new(patch.x, -(patch.y - lift(3.0))));

    // Point the camera at the patch: the cursor at the viewport's center picks the patch's ground point.
    let map = app.world().resource::<SimRes>().map().clone();
    app.world_mut().resource_mut::<CameraView>().center = drawn_position(&map, patch);
    app.update();
    assert_eq!(camera_center(&app), drawn_position(&map, patch), "not clamped");
    let size = *app.world().resource::<SurfaceSize>();
    let insets = *app.world().resource::<ViewportInsets>();
    let (position, viewport) = viewport_rect(size, insets);
    let middle = (position.as_vec2() + viewport.as_vec2() / 2.0) / size.scale_factor;
    let view = *app.world().resource::<CameraView>();
    let ground = cursor_to_ground(middle, size, insets, view, 1.0, &map).unwrap();
    assert!((ground - patch).length() < 0.01, "{ground} should be {patch}");
}

#[test]
fn every_object_has_a_visual_until_it_is_removed() {
    let mut app = headless_app(small_scenario());
    app.update();
    let ids = sim_ids(&app);
    assert_eq!(ids.len(), 4);
    assert_eq!(visual_ids(&mut app), ids);

    // A visual sits at its object's position, flipped into Bevy's +y-up space.
    let patch = app.world().resource::<SimRes>().object(ObjectId(1)).unwrap().pos;
    let entity = app.world().resource::<ObjectVisuals>().by_id[&ObjectId(1)];
    let translation = app.world().get::<Transform>(entity).unwrap().translation;
    assert_eq!(translation.truncate(), Vec2::new(patch.x, -patch.y));

    let removed = ObjectId(2);
    app.world_mut()
        .resource_mut::<PendingCommands>()
        .0
        .push(debug(DebugCommand::Remove { id: removed }));
    app.update();
    let remaining: BTreeSet<ObjectId> = ids.iter().copied().filter(|id| *id != removed).collect();
    assert_eq!(sim_ids(&app), remaining);
    assert_eq!(visual_ids(&mut app), remaining);
    assert!(!app.world().resource::<ObjectVisuals>().by_id.contains_key(&removed));

    // Spawning one later creates its visual too.
    app.world_mut()
        .resource_mut::<PendingCommands>()
        .0
        .push(debug(DebugCommand::Spawn {
            kind: ObjectKind::CrystalPatch,
            owner: None,
            at: SpawnAt::Tile(TilePos::new(12, 3)),
        }));
    app.update();
    assert_eq!(visual_ids(&mut app), sim_ids(&app));
    assert_eq!(visual_ids(&mut app).len(), 4);
}

#[test]
fn southern_objects_draw_in_front() {
    let mut app = headless_app(small_scenario());
    app.update();
    let visuals = app.world().resource::<ObjectVisuals>().by_id.clone();
    let z = |app: &App, id: u32| {
        app.world()
            .get::<Transform>(visuals[&ObjectId(id)])
            .unwrap()
            .translation
            .z
    };
    // Patch 3 (row 9) lies south of patch 1 (row 2); patches 1 and 2 share a row.
    assert!(z(&app, 3) > z(&app, 1));
    assert_eq!(z(&app, 1), z(&app, 2));
}

/// A Peacekeeper's visual turns with its facing. A Guard's square stays axis-aligned like its collision box, and only
/// its facing tick turns.
#[test]
fn a_guards_square_stays_axis_aligned_and_only_its_tick_turns() {
    let mut app = headless_app(small_scenario());
    app.update();
    let spawn = |kind, at| {
        debug(DebugCommand::Spawn {
            kind,
            owner: Some(PlayerId(1)),
            at: SpawnAt::Point(at),
        })
    };
    let move_to = |id, to| PlayerCommand {
        player: PlayerId(1),
        command: Command::Order {
            units: vec![ObjectId(id)],
            kind: OrderKind::Move(Target::Point(to)),
            queue: false,
        },
    };
    let pending = |app: &mut App, command| app.world_mut().resource_mut::<PendingCommands>().0.push(command);
    pending(&mut app, spawn(ObjectKind::Guard, Vec2::new(100.0, 650.0)));
    pending(&mut app, spawn(ObjectKind::Peacekeeper, Vec2::new(100.0, 560.0)));
    app.update();
    let (guard, pk) = (5, 6);
    pending(&mut app, move_to(guard, Vec2::new(500.0, 740.0)));
    pending(&mut app, move_to(pk, Vec2::new(500.0, 660.0)));
    updates(&mut app, 12);

    // Where local +x points after a visual's rotation, against the sim facing it should show.
    let facing = |app: &App, id: u32| {
        let o = app.world().resource::<SimRes>().object(ObjectId(id)).unwrap();
        assert!((o.prev_facing - o.facing).abs() < 1e-4, "unit {id} goes straight on");
        assert!(o.facing.abs() > 0.1, "unit {id} faces down-right: {}", o.facing);
        Vec3::new(o.facing.cos(), -o.facing.sin(), 0.0)
    };
    let front = |transform: &Transform| transform.rotation * Vec3::X;
    let visuals = app.world().resource::<ObjectVisuals>().by_id.clone();
    let body = |app: &App, id: u32| *app.world().get::<Transform>(visuals[&ObjectId(id)]).unwrap();
    assert!(
        front(&body(&app, pk)).distance(facing(&app, pk)) < 1e-3,
        "the Peacekeeper turns"
    );
    assert_eq!(body(&app, guard).rotation, Quat::IDENTITY, "the Guard's square doesn't");
    let mut ticks = app.world_mut().query::<(&FacingTick, &Transform, &ChildOf)>();
    let tick: Vec<_> = ticks
        .iter(app.world())
        .map(|(t, transform, parent)| (t.0, *transform, parent.0))
        .collect();
    assert_eq!(tick.len(), 1, "only the Guard has a separate tick");
    let (of, transform, parent) = tick[0];
    assert_eq!((of, parent), (ObjectId(guard), visuals[&ObjectId(guard)]));
    assert!(front(&transform).distance(facing(&app, guard)) < 1e-3, "its tick turns");
}

#[test]
fn the_grid_toggles_with_its_action() {
    let mut app = headless_app(small_scenario());
    app.update();
    let grid_visibility = |app: &mut App| {
        let mut query = app.world_mut().query_filtered::<&Visibility, With<GridOverlay>>();
        *query.single(app.world()).unwrap()
    };
    assert_eq!(grid_visibility(&mut app), Visibility::Hidden);
    press(&mut app, InputAction::ToggleGrid);
    app.update();
    assert_eq!(grid_visibility(&mut app), Visibility::Inherited);
    // A press fires once.
    updates(&mut app, 2);
    assert_eq!(grid_visibility(&mut app), Visibility::Inherited);
    press(&mut app, InputAction::ToggleGrid);
    app.update();
    assert_eq!(grid_visibility(&mut app), Visibility::Hidden);
}

#[test]
fn loading_another_scenario_rebuilds_the_map_and_the_visuals() {
    let mut app = headless_app(small_scenario());
    updates(&mut app, 3);
    assert_eq!(app.world().resource::<MapView>().builds, 1);

    let mut other = scenarios::empty(20, 10);
    other.map.spawns.push(patch_at(1, 1));
    load_scenario(app.world_mut(), other);
    app.update();

    assert_eq!(app.world().resource::<MapView>().builds, 2);
    let handle = app.world().resource::<MapRenderAssets>().terrain_mesh.clone();
    let vertices = app
        .world()
        .resource::<Assets<Mesh>>()
        .get(&handle)
        .unwrap()
        .count_vertices();
    assert_eq!(vertices, 20 * 10 * 4);
    // Object 1 of the new sim is a different object than object 1 of the old one.
    assert_eq!(visual_ids(&mut app), BTreeSet::from([ObjectId(1)]));
    assert_eq!(
        app.world().resource::<RecordedInputs>().ticks.len(),
        tick(&app) as usize
    );
}

/// Advances real time by `fraction` of a tick in one frame. Returns the frame's interpolation factor and where
/// object 1 is drawn: on flat ground, the ground position it is drawn at.
fn advance(app: &mut App, fraction: f64) -> (f32, Vec2) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(timestep.mul_f64(fraction)));
    app.update();
    let alpha = app.world().resource::<RenderAlpha>().0;
    let entity = app.world().resource::<ObjectVisuals>().by_id[&ObjectId(1)];
    let drawn = app.world().get::<Transform>(entity).unwrap().translation;
    (alpha, Vec2::new(drawn.x, -drawn.y))
}

/// `ARCHITECTURE.md > Client architecture > Visuals > Interpolation`: between ticks, an object is drawn between its
/// `prev_pos` and `pos`, by the part of the next tick that has passed (`Time<Fixed>::overstep_fraction`); a teleport
/// (`DebugCommand::SetPosition`) is drawn at its target at once, never on the way there.
#[test]
fn objects_are_drawn_between_their_tick_positions_except_after_a_teleport() {
    let mut scenario = scenarios::empty(16, 12);
    scenario.map.spawns.push(patch_at(2, 2));
    let mut app = headless_app(scenario);
    updates(&mut app, 2);
    // Nothing in the sim moves yet: make object 1 a free (not tile-based) object that moved in the last tick.
    let (from, to) = (Vec2::new(200.0, 300.0), Vec2::new(520.0, 140.0));
    let mut state = serde_json::to_value(app.world().resource::<SimRes>().sim()).unwrap();
    let object = &mut state["objects"][0];
    object["prev_pos"] = serde_json::to_value(from).unwrap();
    object["pos"] = serde_json::to_value(to).unwrap();
    object["placement"] = serde_json::Value::Null;
    let moved: Sim = serde_json::from_value(state).unwrap();
    app.world_mut().resource_mut::<SimRes>().replace(moved);
    let start = tick(&app);

    let close = |a: Vec2, b: Vec2| (a - b).length() < 1e-3;
    let (alpha, drawn) = advance(&mut app, 0.0);
    assert_eq!(alpha, 0.0);
    assert!(close(drawn, from), "{drawn} should be {from}");
    for (fraction, expected_alpha) in [(0.25, 0.25), (0.5, 0.75)] {
        let (alpha, drawn) = advance(&mut app, fraction);
        assert!((alpha - expected_alpha).abs() < 1e-6, "{alpha}");
        let expected = from.lerp(to, expected_alpha);
        assert!(close(drawn, expected), "{drawn} should be {expected}");
    }
    assert_eq!(tick(&app), start, "no tick ran yet");

    // The next tick teleports the object: drawn at the target in the frame of the tick, wherever the clock is.
    let target = Vec2::new(700.0, 600.0);
    app.world_mut()
        .resource_mut::<PendingCommands>()
        .0
        .push(debug(DebugCommand::SetPosition {
            id: ObjectId(1),
            pos: target,
        }));
    let (alpha, drawn) = advance(&mut app, 0.5);
    assert_eq!(tick(&app), start + 1);
    assert!((alpha - 0.25).abs() < 1e-6, "{alpha}");
    assert_eq!(drawn, target);
    let object = app.world().resource::<SimRes>().object(ObjectId(1)).unwrap().clone();
    assert_eq!((object.prev_pos, object.pos), (target, target));
    let (_, drawn) = advance(&mut app, 0.5);
    assert_eq!(drawn, target);
}
