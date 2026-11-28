//! What is drawn in front where objects overlap, and so what a click there picks (`docs/architecture/client-architecture-visuals.md > Client
//! architecture > Visuals > Draw order`): a unit is drawn above every structure, also while it crosses one's
//! footprint, as a unit walking out of a portal's energy doorway does (slice M3-D2, design change 0145), and its
//! selection ring between the two; a click on it picks the unit, not the portal under it, also once it is stuck there
//! half out (design change 0145, point 4: "a normal unit there"; slice M3-K2a).

use bevy::prelude::*;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Command, DebugCommand, Faction, Object, PlayerCommand, ProductionCommand, Target};
use space_crystals::coords::DrawLayer;
use space_crystals::object_render::{ObjectVisuals, draw_layer};
use space_crystals::selection::markers::SelectionMarkers;
use space_crystals::sim_driver::PendingCommands;

use super::*;

const P1: PlayerId = PlayerId(1);
/// Player 1's Robo Portal, open all round, on tiles (23, 12) to (26, 15): x 1472 to 1728, y 768 to 1024, in the
/// world view.
const PORTAL: u32 = 1;
const PORTAL_AT: Vec2 = Vec2::new(1600.0, 896.0);
/// The northern edge of its footprint.
const PORTAL_NORTH_Y: f32 = 768.0;
/// Half a Drone's side: its silhouette is 36 × 36.
const DRONE_HALF: f32 = 18.0;
/// The portal's rally point, due north of it in the world view: a Drone it trains walks straight out of its centre
/// towards it, across the footprint's northern half, where it lies north of the portal's anchor all the way.
const RALLY: Vec2 = Vec2::new(1600.0, 600.0);

/// A flat 64 × 48 map with player 1's [`PORTAL`] and Egress Node (2), and player 2's far off screen (3), both
/// Colonists.
fn portal_scenario() -> Scenario {
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    ScenarioBuilder::new(&vec![".".repeat(64); 48].join("\n"))
        .faction(P1, Faction::Colonists)
        .faction(PlayerId(2), Faction::Colonists)
        .spawn(ObjectKind::RoboPortal, Some(P1), tile(23, 12))
        .spawn(ObjectKind::EgressNode, Some(P1), tile(12, 9))
        .spawn(ObjectKind::EgressNode, Some(PlayerId(2)), tile(56, 40))
        .build()
}

/// Submits `command` in player 1's name, for the next tick.
fn submit(app: &mut App, command: Command) {
    let command = PlayerCommand { player: P1, command };
    app.world_mut().resource_mut::<PendingCommands>().0.push(command);
}

/// The Drone [`PORTAL`] trained, once it is out of production.
fn drone(app: &App) -> Option<&Object> {
    let sim = app.world().resource::<SimRes>();
    sim.objects().find(|o| o.kind == ObjectKind::Drone)
}

/// The z of what is drawn for object `id`: its visual's, and its selection marker's if it is selected.
fn drawn_z(app: &App, id: ObjectId) -> (f32, Option<f32>) {
    let z = |entity: Entity| app.world().get::<Transform>(entity).unwrap().translation.z;
    let visual = app.world().resource::<ObjectVisuals>().by_id[&id];
    let marker = app.world().resource::<SelectionMarkers>().by_id.get(&id).copied();
    (z(visual), marker.map(z))
}

/// The unit visibility the M3-U brief asks for: a Drone walking out of a Robo Portal is drawn above it all the way
/// across its footprint, though it lies north of the portal's anchor, where sorting by depth alone would put it
/// under the portal; a click on it there selects it, and its ring is drawn under it and over the portal. The portal's
/// parts lie within 0.01 of its visual's z (`object_render::structures`: its tag the highest, at 0.005).
#[test]
fn a_unit_walking_out_of_a_portal_is_drawn_and_picked_above_it() {
    let mut app = game_of(portal_scenario(), true);
    let crystals = DebugCommand::SetResources {
        player: P1,
        crystals: 50,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    submit(&mut app, Command::Debug(crystals));
    app.update();
    let portal = vec![ObjectId(PORTAL)];
    let rally = ProductionCommand::SetRally(Target::Point(RALLY));
    let train = ProductionCommand::Train(ObjectKind::Drone);
    for command in [rally, train] {
        let objects = portal.clone();
        submit(&mut app, Command::Production { objects, command });
    }
    // Trained in 160 frames, it materializes at the portal's centre and heads north.
    for _ in 0..200 {
        if drone(&app).is_some_and(|d| d.pos.y < PORTAL_AT.y - 40.0) {
            break;
        }
        app.update();
    }
    let walking = drone(&app).expect("trained");
    let (id, pos) = (walking.id, walking.pos);
    assert!(pos.y < PORTAL_AT.y - 40.0, "{pos}: walking north");
    assert!(
        pos.y + DRONE_HALF > PORTAL_NORTH_Y + 60.0,
        "{pos}: well inside the footprint"
    );
    let portal_z = drawn_z(&app, ObjectId(PORTAL)).0;
    assert!(drawn_z(&app, id).0 > portal_z + 0.01, "drawn above the portal");
    assert_eq!(
        DrawLayer::GroundObjects.z_at(PORTAL_AT.y),
        portal_z,
        "the portal: in its layer, at its depth"
    );

    click(&mut app, pos, &[]);
    assert_eq!(
        selected(&app),
        [id.0],
        "the click picks the Drone drawn over the portal"
    );
    let mut inside = 0;
    while let Some(d) = drone(&app).filter(|d| d.pos.y + DRONE_HALF > PORTAL_NORTH_Y) {
        assert!(d.pos.y < PORTAL_AT.y, "north of the portal's anchor");
        let (unit, ring) = drawn_z(&app, d.id);
        let ring = ring.expect("selected");
        assert!(portal_z + 0.01 < ring && ring < unit, "{portal_z} < {ring} < {unit}");
        inside += 1;
        assert!(inside < 100, "it gets out");
        app.update();
    }
    assert!(inside > 0, "seen crossing the footprint");
    // Out of it, the Drone keeps its place above the structures, and the portal its own among them.
    assert_eq!(draw_layer(ObjectKind::Drone), DrawLayer::GroundUnits);
    for kind in [
        ObjectKind::RoboPortal,
        ObjectKind::ExtractionPlate,
        ObjectKind::CrystalPatch,
    ] {
        assert_eq!(draw_layer(kind), DrawLayer::GroundObjects, "{kind:?}");
    }
}

/// Player 1's Bio Portal, unturned (open north and east), on tiles (23, 12) to (25, 14): x 1472 to 1664, y 768 to 960
/// in the world view, its centre (1568, 864). Mountains on the five tiles along its north side and a column of six
/// Drones of player 2's east of it, 18 su off its east side, leave no open direction free: the Sabre Legionnaire it
/// trains, rallied east, walks out east all the same and stops half out against the Drones, its centre on the
/// footprint's east side ([`HALF_OUT`]), as `sc_sim`'s `production::half_out` tests have it. Player 1's Egress Node
/// (2) far off.
fn walled_in_bio_portal() -> Scenario {
    let tile = |x, y| SpawnAt::Tile(TilePos::new(x, y));
    let mut rows = vec![".".repeat(64); 48];
    rows[11].replace_range(22..27, "MMMMM");
    let mut builder = ScenarioBuilder::new(&rows.join("\n"))
        .faction(P1, Faction::Colonists)
        .faction(PlayerId(2), Faction::Colonists)
        .spawn(ObjectKind::BioPortal, Some(P1), tile(23, 12))
        .spawn(ObjectKind::EgressNode, Some(P1), tile(4, 40));
    for k in 0..6 {
        let at = Vec2::new(1700.0, 786.0 + 36.0 * k as f32);
        builder = builder.spawn(ObjectKind::Drone, Some(PlayerId(2)), SpawnAt::Point(at));
    }
    builder.build()
}

/// Where the Sabre of [`walled_in_bio_portal`] stops, half out.
const HALF_OUT: Vec2 = Vec2::new(1664.0, 864.0);

/// Design change 0145, point 4 (slice M3-K2a): a unit that can't get out of a portal stays "half out at the edge of
/// the footprint. It is a normal unit there (selectable, attackable, able to attack)". A click on the half of the
/// Sabre still over the portal picks it, drawn above the portal, though it stands in its doorway (its attacks:
/// `sc_sim`'s `production::half_out`).
#[test]
fn a_unit_stuck_half_out_of_a_portal_is_picked_by_a_click() {
    let mut app = game_of(walled_in_bio_portal(), true);
    let crystals = DebugCommand::SetResources {
        player: P1,
        crystals: 100,
        supplies: 0,
        alloys: 0,
        essence: 0,
        flux: 0,
    };
    submit(&mut app, Command::Debug(crystals));
    app.update();
    let portal = vec![ObjectId(PORTAL)];
    let rally = ProductionCommand::SetRally(Target::Point(Vec2::new(2200.0, 864.0)));
    let train = ProductionCommand::Train(ObjectKind::SabreLegionnaire);
    for command in [rally, train] {
        let objects = portal.clone();
        submit(&mut app, Command::Production { objects, command });
    }
    let sabre = |app: &App| {
        let sim = app.world().resource::<SimRes>();
        let sabre = sim.objects().find(|o| o.kind == ObjectKind::SabreLegionnaire)?;
        Some((sabre.id, sabre.pos, sabre.mover.as_ref()?.exit))
    };
    for _ in 0..300 {
        if sabre(&app).is_some_and(|(_, pos, _)| pos == HALF_OUT) {
            break;
        }
        app.update();
    }
    updates(&mut app, 10);
    let (id, pos, exit) = sabre(&app).expect("trained");
    assert_eq!((pos, exit), (HALF_OUT, Some(ObjectId(PORTAL))), "stuck half out");
    assert!(
        drawn_z(&app, id).0 > drawn_z(&app, ObjectId(PORTAL)).0 + 0.01,
        "drawn above the portal"
    );

    click(&mut app, HALF_OUT - Vec2::new(9.0, 0.0), &[]);
    assert_eq!(selected(&app), [id.0], "the click picks the Sabre, not the portal");
}
