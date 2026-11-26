//! How each kind is drawn: a structure as placed with a tag, a Peacekeeper turning with its facing and a
//! Guard's square axis-aligned with only its tick turning, the Colonists' kinds with their tags, the
//! Peacekeepers' tags, upright and above the command indicators, and a Laser Turret's barrel turning to its target.

use space_crystals::command_ui::indicators::INDICATORS_Z;
use space_crystals::coords::DrawLayer;
use space_crystals::object_render::TurretBarrel;

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

/// Slice M4-S1: a Laser Turret, a structure that attacks (plan decision T49), is drawn as a structure is, its 1 × 1
/// footprint unturned with its tag ("LT"), with a head on its roof and a barrel of its own (`TurretBarrel`,
/// `object_render::turret`; placeholders until Q0001) that turns with the turret's facing, which the sim turns at once
/// to what it fires at (design change 0298 point 8: it picks its own target). Player 1's turret stands in its Tunnel's
/// area, where alone it is active (design change 0360); player 2's Peacekeeper, holding south-east of it within its
/// Range, draws its fire: the turret rises, and the barrel points at it.
#[test]
fn a_laser_turrets_barrel_turns_to_its_target() {
    let mut app = headless_app(scenarios::empty(16, 12));
    app.update();
    let spawn = |kind, owner, at| {
        debug(DebugCommand::Spawn {
            kind,
            owner: Some(PlayerId(owner)),
            at,
            orientation: Default::default(),
        })
    };
    let pending = |app: &mut App, command| app.world_mut().resource_mut::<PendingCommands>().0.push(command);
    // The Tunnel's Tier 1 area, (0, 4) to (7, 11), takes in the turret's tile.
    pending(
        &mut app,
        spawn(ObjectKind::Tunnel, 1, SpawnAt::Tile(TilePos::new(2, 6))),
    );
    let turret_at = SpawnAt::Tile(TilePos::new(5, 5)); // centred on (352, 352)
    pending(&mut app, spawn(ObjectKind::LaserTurret, 1, turret_at));
    pending(
        &mut app,
        spawn(ObjectKind::Peacekeeper, 2, SpawnAt::Point(Vec2::new(544.0, 480.0))),
    );
    app.update();
    let id_of = |app: &App, kind| {
        let sim = app.world().resource::<SimRes>();
        sim.objects().find(|o| o.kind == kind).expect("spawned").id
    };
    let (turret, target) = (
        id_of(&app, ObjectKind::LaserTurret),
        id_of(&app, ObjectKind::Peacekeeper),
    );
    let hold = PlayerCommand {
        player: PlayerId(2),
        command: Command::Order {
            units: vec![target],
            kind: OrderKind::Hold,
            queue: false,
        },
    };
    pending(&mut app, hold);
    updates(&mut app, 30);

    let sim = app.world().resource::<SimRes>();
    let (o, aimed_at) = (sim.object(turret).unwrap(), sim.object(target).unwrap());
    let aim = o.attack.as_ref().and_then(|attack| attack.target);
    assert_eq!(aim, Some(target), "it fires at the Peacekeeper");
    let to = aimed_at.pos - o.pos;
    assert!((o.facing - to.y.atan2(to.x)).abs() < 1e-3, "it faces it: {}", o.facing);
    assert_eq!(o.prev_facing, o.facing, "and holds still");
    let facing = Vec3::new(o.facing.cos(), -o.facing.sin(), 0.0);
    let visuals = app.world().resource::<ObjectVisuals>().by_id.clone();
    let entity = visuals[&turret];
    assert_eq!(tags(&app, entity), [(turret, "LT".to_string())]);
    assert_eq!(app.world().get::<Transform>(entity).unwrap().rotation, Quat::IDENTITY);
    let mut barrels = app.world_mut().query::<(&TurretBarrel, &Transform, &ChildOf)>();
    let barrels: Vec<_> = (barrels.iter(app.world()))
        .map(|(barrel, transform, parent)| (barrel.0, *transform, parent.0))
        .collect();
    let [(of, transform, parent)] = barrels.as_slice() else {
        panic!("one barrel: {barrels:?}");
    };
    assert_eq!((*of, *parent), (turret, entity));
    let front = transform.rotation * Vec3::X;
    assert!(front.distance(facing) < 1e-3, "the barrel points at it: {front}");
}

/// Slice M4-S2: a pop-up Laser Turret (`factions/syndicate/structures/laser_turret.md > Pop-up turret`,
/// `object_render::turret`; placeholders until Q0001), always one (design change 0360: "There is no upright form any
/// more"). Inside its owner's Tunnel Area with no target it is underground from its first tick: its visual is an
/// outline with its tag and no barrel. When player 2's Peacekeeper appears in its Range it rises: a visual made anew,
/// the raised body, from its first rising frame, with a barrel that already points at the target and grows to its
/// whole length over the rise.
#[test]
fn a_popup_turret_is_an_outline_while_lowered_and_its_barrel_comes_out_as_it_rises() {
    let mut app = headless_app(scenarios::empty(24, 16));
    app.update();
    let spawn = |kind, owner, at| {
        debug(DebugCommand::Spawn {
            kind,
            owner: Some(PlayerId(owner)),
            at,
            orientation: Default::default(),
        })
    };
    let pending = |app: &mut App, command| app.world_mut().resource_mut::<PendingCommands>().0.push(command);
    // The Tunnel's Tier 1 area is (4, 4) to (11, 11); the turret on its east edge, centred on (736, 480).
    pending(
        &mut app,
        spawn(ObjectKind::Tunnel, 1, SpawnAt::Tile(TilePos::new(6, 6))),
    );
    pending(
        &mut app,
        spawn(ObjectKind::LaserTurret, 1, SpawnAt::Tile(TilePos::new(11, 7))),
    );
    app.update();
    let turret = {
        let sim = app.world().resource::<SimRes>();
        let found = sim.objects().find(|o| o.kind == ObjectKind::LaserTurret);
        found.expect("spawned").id
    };
    let barrels = |app: &mut App| -> Vec<(ObjectId, Transform)> {
        let mut barrels = app.world_mut().query::<(&TurretBarrel, &Transform)>();
        (barrels.iter(app.world())).map(|(barrel, t)| (barrel.0, *t)).collect()
    };
    let state = |app: &App| app.world().resource::<SimRes>().object(turret).unwrap().popup;
    assert_eq!(state(&app), Some(sc_sim::popup::PopUp::LOWERED), "as the game finds it");
    assert_eq!(barrels(&mut app), [], "no barrel underground");
    updates(&mut app, 45);
    assert_eq!(state(&app), Some(sc_sim::popup::PopUp::LOWERED));
    let lowered = app.world().resource::<ObjectVisuals>().by_id[&turret];
    assert_eq!(barrels(&mut app), [], "still none");
    assert_eq!(tags(&app, lowered), [(turret, "LT".to_string())]);
    // An enemy 3 tiles east of it.
    pending(
        &mut app,
        spawn(
            ObjectKind::Peacekeeper,
            2,
            SpawnAt::Point(Vec2::new(736.0 + 224.0, 480.0)),
        ),
    );
    updates(&mut app, 4);
    assert!(
        matches!(state(&app), Some(sc_sim::popup::PopUp::Rising(_))),
        "{:?}",
        state(&app)
    );
    let rising = app.world().resource::<ObjectVisuals>().by_id[&turret];
    assert_ne!(rising, lowered, "the raised body, a new visual");
    let [(of, transform)] = barrels(&mut app)[..] else {
        panic!("one barrel");
    };
    assert_eq!(of, turret);
    assert!(
        (transform.rotation * Vec3::X).distance(Vec3::X) < 1e-3,
        "it points east, at its target"
    );
    assert!(
        transform.scale.x > 0.25 && transform.scale.x < 1.0,
        "part of the way out: {}",
        transform.scale.x
    );
    updates(&mut app, 8);
    assert!(matches!(state(&app), Some(sc_sim::popup::PopUp::Raised { .. })));
    assert_eq!(barrels(&mut app)[0].1.scale, Vec3::ONE, "all the way out");
    assert_eq!(
        app.world().resource::<ObjectVisuals>().by_id[&turret],
        rising,
        "the same visual"
    );
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

/// Slice M3-J3 (M3-K2's walkthrough): the Peacekeeper's two types carry tags of their own, "PK" and "PS" (M3-T draws
/// a unit by its form, and the two share their round body, design change 0173), which stay upright while the body
/// turns with its facing. And a unit's tag draws above the command indicators, so that no marker on the unit hides
/// it (in the walkthrough, the Bio Portal's rally marker on a Rifle Legionnaire), and in its unit's y order.
#[test]
fn the_peacekeepers_tags_stay_upright_above_the_indicators() {
    let mut app = headless_app(small_scenario());
    app.update();
    let kinds = [ObjectKind::Peacekeeper, ObjectKind::PeacekeeperShielded];
    for (kind, y) in kinds.into_iter().zip([560.0, 650.0]) {
        let spawn = debug(DebugCommand::Spawn {
            kind,
            owner: Some(PlayerId(1)),
            at: SpawnAt::Point(Vec2::new(100.0, y)),
            orientation: Default::default(),
        });
        app.world_mut().resource_mut::<PendingCommands>().0.push(spawn);
    }
    app.update();
    let ids = [5, 6].map(ObjectId);
    let go = PlayerCommand {
        player: PlayerId(1),
        command: Command::Order {
            units: ids.to_vec(),
            kind: OrderKind::Move(Target::Point(Vec2::new(500.0, 740.0))),
            queue: false,
        },
    };
    app.world_mut().resource_mut::<PendingCommands>().0.push(go);
    updates(&mut app, 12);

    let visuals = app.world().resource::<ObjectVisuals>().by_id.clone();
    let mut tags = app.world_mut().query::<(&UnitTag, &GlobalTransform)>();
    let tags: Vec<(ObjectId, GlobalTransform)> = tags.iter(app.world()).map(|(tag, at)| (tag.0, *at)).collect();
    let mut zs = Vec::new();
    for (id, label) in ids.into_iter().zip(["PK", "PS"]) {
        let entity = visuals[&id];
        assert_eq!(unit_tags(&app, entity), [(id, label.to_string())]);
        let body = app.world().get::<GlobalTransform>(entity).unwrap();
        let (_, tag) = tags.iter().find(|(of, _)| *of == id).unwrap();
        assert!(
            body.rotation().angle_between(Quat::IDENTITY) > 0.1,
            "{label}'s body turns"
        );
        assert!(
            tag.rotation().angle_between(Quat::IDENTITY) < 1e-4,
            "{label}'s tag doesn't"
        );
        assert!(
            tag.translation().z > DrawLayer::Overlays.base_z() + INDICATORS_Z,
            "{label}"
        );
        zs.push(tag.translation().z);
    }
    assert!(
        zs[1] > zs[0],
        "y-sorted: the southern unit's tag over the northern one's"
    );
}
