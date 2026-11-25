//! How each kind is drawn: a structure as placed with a tag, a Peacekeeper turning with its facing and a
//! Guard's square axis-aligned with only its tick turning, and the Colonists' kinds with their tags.

use super::*;

/// A structure is drawn as placed: its footprint, turned, a little inside its edge (8 su, clear of the selection
/// frame), with a tag naming its type. In the QA map, slot 2's Barracks is turned a quarter (2 × 3 tiles, not 3 ×
/// 2). Map resources have no tag; a plate draws over the patch it covers.
#[test]
fn structures_are_drawn_as_placed_with_a_tag() {
    let mut app = headless_app(scenarios::dev_structures());
    updates(&mut app, 2);
    let objects: Vec<_> = app
        .world()
        .resource::<SimRes>()
        .objects()
        .map(|o| (o.id, o.kind, o.placement))
        .collect();
    let visuals = app.world().resource::<ObjectVisuals>().by_id.clone();
    let mut sizes = BTreeSet::new();
    for (id, kind, placement) in objects {
        let entity = visuals[&id];
        if kind.def().structure.is_none() {
            assert_eq!(tags(&app, entity), [], "{kind:?}");
            continue;
        }
        let placement = placement.unwrap();
        let half = Vec2::new(placement.w as f32, placement.h as f32) * 32.0 - 8.0;
        let (lo, hi) = mesh_bounds(&app, entity);
        assert_eq!((lo.truncate(), hi.truncate()), (-half, half), "{kind:?}");
        assert_eq!(tags(&app, entity), [(id, short_label(kind).to_string())]);
        if kind == ObjectKind::Barracks {
            sizes.insert((placement.w, placement.h));
        }
    }
    assert_eq!(sizes, BTreeSet::from([(3, 2), (2, 3)]));
    // Where the mesh of object `id` starts and ends in depth.
    let depth = |id| {
        let anchor = app.world().get::<Transform>(visuals[&id]).unwrap().translation.z;
        let (lo, hi) = mesh_bounds(&app, visuals[&id]);
        (anchor + lo.z, anchor + hi.z)
    };
    let sim = app.world().resource::<SimRes>();
    let plates: Vec<_> = sim
        .objects()
        .filter(|o| o.kind == ObjectKind::ExtractionPlate)
        .collect();
    assert_eq!(plates.len(), 2);
    for plate in plates {
        let covered = |o: &&sc_sim::Object| o.kind == ObjectKind::CrystalPatch && o.placement == plate.placement;
        let patch = sim.objects().find(covered).unwrap();
        assert!(depth(plate.id).0 > depth(patch.id).1, "the plate over its patch");
    }
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
            orientation: Default::default(),
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

/// The unit tags among `entity`'s children: whose they are, and what they say.
fn unit_tags(app: &App, entity: Entity) -> Vec<(ObjectId, String)> {
    let world = app.world();
    let Some(children) = world.get::<Children>(entity) else {
        return Vec::new();
    };
    let tag = |child| Some((world.get::<UnitTag>(child)?.0, world.get::<Text2d>(child)?.0.clone()));
    children.iter().filter_map(tag).collect()
}

/// The Colonists' kinds (slice M3-A), placeholders until the art direction (Q0001): a structure is its footprint with
/// a tag naming its type, as the GDO's are. A unit is its silhouette's size (36 su), a square for a Legionnaire and
/// an octagon for the Drone, with a tag naming its type; moving off at an angle, its body stays axis-aligned and only
/// its own facing tick turns.
#[test]
fn the_colonists_kinds_are_drawn_with_tags() {
    let mut app = headless_app(scenarios::empty(40, 12));
    app.update();
    let kinds = [
        ObjectKind::Drone,
        ObjectKind::Beacon,
        ObjectKind::EgressNode,
        ObjectKind::RoboPortal,
        ObjectKind::BioPortal,
        ObjectKind::SabreLegionnaire,
        ObjectKind::RifleLegionnaire,
    ];
    for (i, kind) in kinds.into_iter().enumerate() {
        let tile = TilePos::new(1 + 5 * i as i32, 2);
        let at = if kind.def().unit.is_some() {
            SpawnAt::Point(tile.center())
        } else {
            SpawnAt::Tile(tile)
        };
        let spawn = debug(DebugCommand::Spawn {
            kind,
            owner: Some(PlayerId(1)),
            at,
            orientation: Default::default(),
        });
        app.world_mut().resource_mut::<PendingCommands>().0.push(spawn);
    }
    app.update();
    let units = [1, 6, 7].map(ObjectId);
    let go = PlayerCommand {
        player: PlayerId(1),
        command: Command::Order {
            units: units.to_vec(),
            kind: OrderKind::Move(Target::Point(Vec2::new(1800.0, 700.0))),
            queue: false,
        },
    };
    app.world_mut().resource_mut::<PendingCommands>().0.push(go);
    updates(&mut app, 8);
    let visuals = app.world().resource::<ObjectVisuals>().by_id.clone();
    let objects: Vec<_> = app
        .world()
        .resource::<SimRes>()
        .objects()
        .map(|o| (o.id, o.kind, o.placement, o.facing))
        .collect();
    assert_eq!(objects.len(), kinds.len());
    for (id, kind, placement, facing) in objects {
        let entity = visuals[&id];
        let label = short_label(kind).to_string();
        let (lo, hi) = mesh_bounds(&app, entity);
        if let Some(placement) = placement {
            let half = Vec2::new(placement.w as f32, placement.h as f32) * 32.0 - 8.0;
            assert_eq!((lo.truncate(), hi.truncate()), (-half, half), "{kind:?}");
            assert_eq!(
                (tags(&app, entity), unit_tags(&app, entity)),
                (vec![(id, label)], vec![])
            );
            continue;
        }
        assert!(
            units.contains(&id) && facing.abs() > 0.1,
            "{kind:?} faces down-right: {facing}"
        );
        assert_eq!(
            (lo.truncate(), hi.truncate()),
            (Vec2::splat(-18.0), Vec2::splat(18.0)),
            "{kind:?}"
        );
        assert_eq!(
            (tags(&app, entity), unit_tags(&app, entity)),
            (vec![], vec![(id, label)])
        );
        let body = app.world().get::<Transform>(entity).unwrap();
        assert_eq!(body.rotation, Quat::IDENTITY, "{kind:?}");
    }
    let mut ticks = app.world_mut().query::<(&FacingTick, &ChildOf)>();
    let ticks: BTreeSet<_> = ticks.iter(app.world()).map(|(t, parent)| (t.0, parent.0)).collect();
    assert_eq!(ticks, BTreeSet::from(units.map(|id| (id, visuals[&id]))));
}
