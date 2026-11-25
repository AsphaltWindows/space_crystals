//! Object visuals: one for every object until it is removed, southern objects drawn in front, and a new visual
//! for an object that changes its kind in place (plan decision T15).

use super::*;

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
            orientation: Default::default(),
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

/// The vertex colours of `entity`'s mesh.
fn mesh_colors(app: &App, entity: Entity) -> Vec<[f32; 4]> {
    let handle = &app.world().get::<Mesh2d>(entity).expect("a mesh").0;
    let mesh = app.world().resource::<Assets<Mesh>>().get(handle).unwrap();
    let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) else {
        panic!("a mesh of vertex colours");
    };
    colors.clone()
}

/// Plan decision T15 (`SimEvent::KindChanged`, slice M3-E): an object that changes its kind in place gets a new
/// visual, under the same id, with its new kind's mesh, size and tag. While it transforms its colours are tinted
/// (`object_render::transforming_color`, a placeholder); complete, it is drawn as any other of its kind. Player 1's
/// Beacon 1 warps in a Robo Portal (4 × 4), which is cancelled: the Beacon is drawn as before, with the very same
/// mesh. Beacon 2 warps in an Egress Node (2 × 2, the Beacon's own size), which completes after its 320 frames.
/// Player 1's Robo Portal 3 and Egress Node 4, complete from the start, are drawn as their kinds are.
#[test]
fn a_kind_change_rebuilds_the_visual() {
    use ObjectKind::{Beacon, EgressNode, RoboPortal};
    let mut app = headless_app(scenarios::empty(40, 16));
    app.update();
    let p1 = PlayerId(1);
    let spawn = |kind, x, y| {
        debug(DebugCommand::Spawn {
            kind,
            owner: Some(p1),
            at: SpawnAt::Tile(TilePos::new(x, y)),
            orientation: Default::default(),
        })
    };
    let funds = debug(DebugCommand::SetResources {
        player: p1,
        crystals: 1000,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    });
    let spawns = [
        spawn(Beacon, 4, 6),
        spawn(Beacon, 14, 6),
        spawn(RoboPortal, 22, 5),
        spawn(EgressNode, 32, 6),
        funds,
    ];
    app.world_mut().resource_mut::<PendingCommands>().0.extend(spawns);
    app.update();
    let entity = |app: &App, n: u32| app.world().resource::<ObjectVisuals>().by_id[&ObjectId(n)];
    let mesh = |app: &App, n: u32| app.world().get::<Mesh2d>(entity(app, n)).unwrap().0.clone();
    let kind = |app: &App, n: u32| app.world().resource::<SimRes>().object(ObjectId(n)).unwrap().kind;
    let half = |tiles: f32| Vec2::splat(tiles * 32.0 - 8.0);
    let bounds = |app: &App, n: u32| {
        let (lo, hi) = mesh_bounds(app, entity(app, n));
        (lo.truncate(), hi.truncate())
    };
    let (beacon_entity, beacon_mesh) = (entity(&app, 1), mesh(&app, 1));
    let construction = |n: u32, command| PlayerCommand {
        player: p1,
        command: Command::Construction {
            objects: vec![ObjectId(n)],
            command,
        },
    };
    let warp_in = |kind| ConstructionCommand::WarpIn {
        kind,
        orientation: Default::default(),
    };
    let warps = [
        construction(1, warp_in(RoboPortal)),
        construction(2, warp_in(EgressNode)),
    ];
    app.world_mut().resource_mut::<PendingCommands>().0.extend(warps);
    app.update();
    assert_eq!((kind(&app, 1), kind(&app, 2)), (RoboPortal, EgressNode));
    assert_ne!(entity(&app, 1), beacon_entity, "a new visual");
    assert_eq!(bounds(&app, 1), (-half(4.0), half(4.0)), "the Robo Portal's 4 × 4");
    assert_eq!(tags(&app, entity(&app, 1)), [(ObjectId(1), "RP".to_string())]);
    assert_eq!(tags(&app, entity(&app, 2)), [(ObjectId(2), "EN".to_string())]);
    // Tinted: the shape of Robo Portal 3, in other colours.
    assert_eq!(bounds(&app, 1), bounds(&app, 3));
    assert_ne!(mesh_colors(&app, entity(&app, 1)), mesh_colors(&app, entity(&app, 3)));
    assert_ne!(mesh_colors(&app, entity(&app, 2)), mesh_colors(&app, entity(&app, 4)));
    let cancel = construction(1, ConstructionCommand::CancelTransform);
    app.world_mut().resource_mut::<PendingCommands>().0.push(cancel);
    app.update();
    assert_eq!(kind(&app, 1), Beacon);
    assert_eq!(mesh(&app, 1), beacon_mesh, "the Beacon's own mesh again");
    assert_eq!(bounds(&app, 1), (-half(2.0), half(2.0)));
    assert_eq!(tags(&app, entity(&app, 1)), [(ObjectId(1), "BN".to_string())]);
    let transforming = |app: &App| {
        app.world()
            .resource::<SimRes>()
            .object(ObjectId(2))
            .unwrap()
            .is_transforming()
    };
    for _ in 0..320 {
        if !transforming(&app) {
            break;
        }
        app.update();
    }
    assert!(!transforming(&app), "complete");
    assert_eq!(mesh(&app, 2), mesh(&app, 4), "drawn as any Egress Node");
}
