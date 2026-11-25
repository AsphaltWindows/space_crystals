//! The records drawn on Explored tiles (`core/vision.md > Explored`): as last seen, never the live objects there, an
//! enemy structure's as placed, with its tag, a Tunnel's at the tier last seen, and a site's as a site at the progress
//! last seen.

use sc_sim::{BuildingOrientation, OrderKind, Rotation, SlotId};
use space_crystals::input::InjectedInput;
use space_crystals::object_render::{SiteProgress, StructureTag};

use super::*;

fn tap(app: &mut App, key: KeyCode) {
    app.world_mut().resource_mut::<InjectedInput>().keys.press(key);
    app.update();
    app.world_mut().resource_mut::<InjectedInput>().keys.release(key);
    app.update();
}

/// `core/vision.md > Explored`: out of sight, the patch and the station show as remembered, where they were, and
/// their live objects stay hidden. A patch removed out of sight is still drawn; one placed out of sight isn't. The
/// fog's debug toggle shows the live objects instead, the player switch the other player's memory, whose Unexplored
/// tiles stay black (design change 0159). Back in sight, the removed patch is gone and the live objects show.
#[test]
fn explored_tiles_draw_records_never_live_objects() {
    let mut app = world(Faction::Gdo);
    assert!(drawn(&app, PATCH) && drawn(&app, STATION));
    assert_eq!(remembered(&app), [0_u32; 0]);

    scout_to(&mut app, 20, 7);
    assert!(!drawn(&app, PATCH) && !drawn(&app, STATION));
    assert_eq!(remembered(&app), [PATCH, STATION]);
    let world = app.world();
    let (live, memory) = (
        &world.resource::<ObjectVisuals>().by_id,
        &world.resource::<RememberedVisuals>().by_id,
    );
    for id in [PATCH, STATION].map(ObjectId) {
        assert_eq!(
            translation(&app, memory[&id]),
            translation(&app, live[&id]),
            "where it was"
        );
    }

    debug(&mut app, DebugCommand::Remove { id: ObjectId(PATCH) });
    let at = SpawnAt::Tile(TilePos::new(6, 4));
    let kind = ObjectKind::CrystalPatch;
    debug(
        &mut app,
        DebugCommand::Spawn {
            kind,
            owner: None,
            at,
            orientation: Default::default(),
        },
    );
    app.update();
    let placed = 4;
    assert!(app.world().resource::<SimRes>().object(ObjectId(PATCH)).is_none());
    assert_eq!(remembered(&app), [PATCH, STATION], "the removal wasn't seen");
    assert!(!drawn(&app, placed), "placed in the fog: unknown");

    tap(&mut app, KeyCode::F3);
    assert_eq!(remembered(&app), [0_u32; 0], "the fog is off: everything live");
    assert!(drawn(&app, placed) && drawn(&app, STATION));
    tap(&mut app, KeyCode::F3);
    assert_eq!(remembered(&app), [PATCH, STATION]);
    tap(&mut app, KeyCode::F2);
    assert_eq!(app.world().resource::<LocalPlayer>().0, P2);
    assert_eq!(remembered(&app), [0_u32; 0], "player 2 never explored their tiles");
    tap(&mut app, KeyCode::F2);
    assert_eq!(remembered(&app), [PATCH, STATION]);

    scout_to(&mut app, 5, 7);
    assert_eq!(remembered(&app), [0_u32; 0]);
    assert!(drawn(&app, placed) && drawn(&app, STATION));
    let sim = app.world().resource::<SimRes>();
    assert!(
        sim.vision().player(P1).unwrap().record(ObjectId(PATCH)).is_none(),
        "seen gone"
    );
}

/// The tag among `entity`'s children ([`StructureTag`]): whose it is, what it says, and its colour.
fn tag(app: &App, entity: Entity) -> Option<(ObjectId, String, Color)> {
    let world = app.world();
    world.get::<Children>(entity)?.iter().find_map(|child| {
        let tag = world.get::<StructureTag>(child)?;
        Some((
            tag.0,
            world.get::<Text2d>(child)?.0.clone(),
            world.get::<TextColor>(child)?.0,
        ))
    })
}

/// An enemy structure seen and then out of sight shows as last seen, like a map resource: where it stands, as placed
/// (a Barracks turned a quarter), in its owner's colour, with its tag; dimmed, the tag too.
#[test]
fn a_remembered_enemy_structure_shows_as_placed_with_its_tag() {
    let mut scenario = scenarios::empty(24, 16);
    let mut barracks = Spawn::new(ObjectKind::Barracks, Some(SlotId(2)), SpawnAt::Tile(TilePos::new(8, 4)));
    barracks.orientation.rotation = Rotation::R90; // (8, 4) to (9, 6)
    scenario.map.spawns = vec![barracks];
    let mut app = headless(AppConfig::new(scenario));
    // Looking only: units attack enemy structures on their own (`World::pick`).
    debug(&mut app, DebugCommand::SetAutoTargeting { on: false });
    spawn_unit(&mut app, P1, 5, 7); // 2: sees the Barracks
    app.update();
    let barracks = 1;
    assert!(drawn(&app, barracks));
    assert_eq!(remembered(&app), [0_u32; 0]);
    let live = app.world().resource::<ObjectVisuals>().by_id[&ObjectId(barracks)];
    let (id, label, bright) = tag(&app, live).expect("a live structure has its tag");
    assert_eq!((id, label.as_str()), (ObjectId(barracks), "BR"));

    move_to(&mut app, ObjectId(2), 20, 7);
    assert!(!drawn(&app, barracks));
    assert_eq!(remembered(&app), [barracks]);
    let memory = app.world().resource::<RememberedVisuals>().by_id[&ObjectId(barracks)];
    assert_eq!(translation(&app, memory), translation(&app, live), "where it stands");
    let mesh = |entity| app.world().get::<Mesh2d>(entity).unwrap().0.clone();
    assert_eq!(
        mesh(memory),
        mesh(live),
        "the same shape, as placed, in its owner's colour"
    );
    let material = |entity| {
        app.world()
            .get::<MeshMaterial2d<ColorMaterial>>(entity)
            .unwrap()
            .0
            .clone()
    };
    assert_ne!(material(memory), material(live), "dimmed");
    let (id, label, dim) = tag(&app, memory).expect("a remembered structure has its tag");
    assert_eq!((id, label.as_str()), (ObjectId(barracks), "BR"));
    let (bright, dim) = (bright.to_srgba(), dim.to_srgba());
    assert!(
        dim.red < bright.red && dim.green < bright.green && dim.blue < bright.blue,
        "{dim:?}"
    );
}

/// Design changes 0258 ("Enemies see a Tunnel's tier") and 0237 (the tier last seen): an enemy Tunnel at Tier 2 is
/// tagged "T2" and drawn as one; raised to Tier 3 out of sight (a debug command), its record still shows Tier 2, its
/// look and its tag, until the player sees it again.
#[test]
fn a_remembered_tunnel_keeps_the_tier_it_was_seen_at() {
    let mut scenario = scenarios::empty(24, 16);
    scenario.setup.players[1].faction = Faction::Syndicate;
    let tunnel = Spawn::new(ObjectKind::Tunnel, Some(SlotId(2)), SpawnAt::Tile(TilePos::new(8, 4)));
    scenario.map.spawns = vec![tunnel]; // 1: (8, 4) to (11, 7)
    let mut app = headless(AppConfig::new(scenario));
    // Looking only: units attack enemy structures on their own (`World::pick`).
    debug(&mut app, DebugCommand::SetAutoTargeting { on: false });
    spawn_unit(&mut app, P1, 5, 7); // 2: sees the Tunnel
    let (tunnel, scout) = (1, ObjectId(2));
    let set_tier = |tier| DebugCommand::SetTier {
        id: ObjectId(tunnel),
        tier,
    };
    debug(&mut app, set_tier(2));
    app.update();
    let live = |app: &App| app.world().resource::<ObjectVisuals>().by_id[&ObjectId(tunnel)];
    let label = |app: &App, entity| tag(app, entity).map(|(_, label, _)| label);
    let mesh = |app: &App, entity| app.world().get::<Mesh2d>(entity).unwrap().0.clone();
    assert!(drawn(&app, tunnel));
    assert_eq!(label(&app, live(&app)).as_deref(), Some("T2"));
    let tier_2 = mesh(&app, live(&app));

    move_to(&mut app, scout, 20, 7);
    debug(&mut app, set_tier(3));
    app.update();
    let sim = app.world().resource::<SimRes>();
    let now = sim
        .object(ObjectId(tunnel))
        .and_then(|o| o.tunnel.as_ref())
        .map(|t| t.tier);
    assert_eq!(now, Some(3), "raised out of sight");
    assert!(!drawn(&app, tunnel));
    assert_eq!(remembered(&app), [tunnel]);
    let memory = app.world().resource::<RememberedVisuals>().by_id[&ObjectId(tunnel)];
    assert_eq!(label(&app, memory).as_deref(), Some("T2"), "the tier last seen");
    assert_eq!(mesh(&app, memory), tier_2, "drawn as last seen, at Tier 2");

    move_to(&mut app, scout, 5, 7);
    assert!(drawn(&app, tunnel));
    assert_eq!(remembered(&app), [0_u32; 0]);
    assert_eq!(label(&app, live(&app)).as_deref(), Some("T3"), "seen again");
}

/// Slice M4-I2 (`core/vision.md > Explored`: "shown in their last-known state"): player 2's Agent builds a Tunnel in
/// sight of player 1's scout, which leaves while the site grows. The site is remembered as a site, drawn as the live one
/// was, its progress bar at the share done when last seen, and stays so while the site grows on and is finished out of
/// sight. Seen again, it is a finished Tunnel; out of sight once more, it is remembered as one, with no bar.
#[test]
fn a_remembered_site_shows_as_a_site_at_the_progress_last_seen() {
    let mut scenario = scenarios::empty(24, 16);
    scenario.setup.players[1].faction = Faction::Syndicate;
    let mut app = headless(AppConfig::new(scenario));
    debug(&mut app, DebugCommand::SetAutoTargeting { on: false });
    spawn_unit(&mut app, P1, 5, 7); // 1: sees the site, tiles (8, 4) to (11, 7)
    let agent = DebugCommand::Spawn {
        kind: ObjectKind::Agent,
        owner: Some(P2),
        at: SpawnAt::Point(centre(13, 9)),
        orientation: Default::default(),
    };
    debug(&mut app, agent); // 2
    app.update();
    let (scout, agent) = (ObjectId(1), ObjectId(2));
    let build = OrderKind::Build {
        kind: ObjectKind::Tunnel,
        origin: TilePos::new(8, 4),
        orientation: BuildingOrientation::default(),
    };
    let order = Command::Order {
        units: vec![agent],
        kind: build,
        queue: false,
    };
    submit(&mut app, P2, order);
    let site_of = |app: &App| {
        let sim = app.world().resource::<SimRes>();
        sim.objects().find(|o| o.is_site()).map(|o| (o.id, tick(app)))
    };
    let mut started = None;
    for _ in 0..200 {
        app.update();
        started = site_of(&app);
        if started.is_some() {
            break;
        }
    }
    let (site, started) = started.expect("a site within 200 ticks");
    run_until(&mut app, started + 120);
    let mesh = |app: &App, entity| app.world().get::<Mesh2d>(entity).unwrap().0.clone();
    let live = |app: &App| app.world().resource::<ObjectVisuals>().by_id[&site];
    let site_mesh = mesh(&app, live(&app));
    // The bars of the site's visuals: whether each is a record's, and the share it shows.
    let bars = |app: &mut App| {
        let mut query = app.world_mut().query::<&SiteProgress>();
        let mut bars: Vec<(bool, f32)> = (query.iter(app.world()))
            .filter(|fill| fill.id == site)
            .map(|fill| (fill.remembered, fill.shown))
            .collect();
        bars.sort_by(|a, b| a.0.cmp(&b.0));
        bars
    };

    move_to(&mut app, scout, 20, 14);
    assert!(!drawn(&app, site.0));
    assert_eq!(remembered(&app), [site.0]);
    let record = (app.world().resource::<SimRes>().vision().player(P1))
        .and_then(|vision| vision.record(site))
        .cloned()
        .expect("remembered");
    let last_seen = record.site.expect("remembered as a site").ratio();
    assert!(last_seen > 0.25 && last_seen < 0.5, "{last_seen}");
    let memory = |app: &App| app.world().resource::<RememberedVisuals>().by_id[&site];
    assert_eq!(mesh(&app, memory(&app)), site_mesh, "drawn as the site was");
    let [(false, now), (true, shown)] = bars(&mut app)[..] else {
        panic!("a bar for the site and one for its record: {:?}", bars(&mut app));
    };
    assert_eq!(shown, last_seen, "the progress last seen");
    assert!(now > last_seen, "the site grows on out of sight");

    run_until(&mut app, started + 500);
    let finished = |app: &App| {
        app.world()
            .resource::<SimRes>()
            .object(site)
            .is_some_and(|o| !o.is_site())
    };
    assert!(finished(&app), "finished out of sight");
    assert_eq!(mesh(&app, memory(&app)), site_mesh, "still a site");
    assert_eq!(bars(&mut app), [(true, last_seen)], "still at the progress last seen");

    move_to(&mut app, scout, 5, 7);
    assert!(drawn(&app, site.0));
    assert_eq!(remembered(&app), [0_u32; 0]);
    let tunnel_mesh = mesh(&app, live(&app));
    assert_ne!(tunnel_mesh, site_mesh, "a finished Tunnel");
    move_to(&mut app, scout, 20, 14);
    assert_eq!(remembered(&app), [site.0]);
    assert_eq!(mesh(&app, memory(&app)), tunnel_mesh, "remembered finished");
    assert_eq!(bars(&mut app), [], "with no bar");
}
