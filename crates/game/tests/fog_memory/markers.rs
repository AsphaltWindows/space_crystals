//! The markers on a target the player remembers (`core/vision.md > Explored > Targeting remembered objects`, design
//! change 0139: "Nothing about the real state leaks: the cursor, the command and any rally marker treat the remembered
//! object as present until the player sees otherwise"; slice M3-K2a): around the record, where it is drawn, never
//! where the object really is.

use sc_sim::{ConstructionCommand, OrderKind, ProductionCommand, SU_PER_GU, SlotId, Target, deploy};
use space_crystals::command_ui::indicators::{
    ATTACK_COLOR, Indicator, MarkerRing, marker_ring, rally_indicators, selection_indicators,
};
use space_crystals::hud::info_panel::activity;
use space_crystals::selection::{ControlStates, Selection};

use super::*;

/// The objects of [`beacon_world`], by id: player 1's Barracks and Peacekeeper, and player 2's Drone, which deploys
/// into a Beacon on [`SITE`] (player 2's Egress Node is 3).
const BARRACKS: u32 = 1;
const DRONE: u32 = 2;
const SENTRY: u32 = 4;
/// The Beacon's site: tiles (14, 6) to (15, 7).
const SITE: TilePos = TilePos::new(14, 6);

/// A flat 40 × 16 map. Player 1 (GDO) has a Barracks on (2, 6) to (4, 7) and a Peacekeeper on (10, 6), which sees
/// [`SITE`]; player 2 (Colonists) a Drone on the site's centre, and an Egress Node on (36, 12) out of player 1's
/// sight, which keeps player 2 in the game once the Drone's deployment is cancelled (`core/game_rules.md > Defeat
/// Condition`). Auto-targeting is off: nobody shoots.
fn beacon_world() -> App {
    let mut scenario = scenarios::empty(40, 16);
    let spawn = |kind, slot, at| Spawn::new(kind, Some(SlotId(slot)), at);
    scenario.map.spawns = vec![
        spawn(ObjectKind::Barracks, 1, SpawnAt::Tile(TilePos::new(2, 6))),
        spawn(ObjectKind::Drone, 2, SpawnAt::Point(deploy::site(SITE).center())),
        spawn(ObjectKind::EgressNode, 2, SpawnAt::Tile(TilePos::new(36, 12))),
        spawn(ObjectKind::Peacekeeper, 1, SpawnAt::Point(centre(10, 6))),
    ];
    scenario.setup.players[1].faction = Faction::Colonists;
    let mut app = headless(AppConfig::new(scenario));
    debug(&mut app, DebugCommand::SetAutoTargeting { on: false });
    app.update();
    app
}

fn order(app: &mut App, player: PlayerId, unit: u32, kind: OrderKind) {
    let units = vec![ObjectId(unit)];
    submit(
        app,
        player,
        Command::Order {
            units,
            kind,
            queue: false,
        },
    );
}

fn object(app: &App, id: u32) -> Option<&sc_sim::Object> {
    app.world().resource::<SimRes>().object(ObjectId(id))
}

/// The markers player 1 sees, as they are drawn: its selected units' orders', then its selected producers' rally
/// points', each with its indicator.
fn markers(app: &App) -> Vec<(Indicator, Option<MarkerRing>)> {
    let world = app.world();
    let (sim, states) = (world.resource::<SimRes>(), world.resource::<ControlStates>());
    let sight = Sight::new(sim, P1, true);
    let rallies = rally_indicators(sim, &sight, states)
        .into_iter()
        .map(|rally| rally.marker);
    let indicators = selection_indicators(sim, &sight, states, false)
        .into_iter()
        .chain(rallies);
    indicators
        .map(|indicator| (indicator, marker_ring(sim, &sight, indicator, 1.0)))
        .collect()
}

/// Design change 0139: the markers on a Beacon player 1 remembers stay on the Beacon's record while the object walks
/// away in the fog. Player 2's Drone deploys on the site, in player 1's sight; player 1 rallies its Barracks on the
/// Beacon, its Peacekeeper leaves, and the Beacon is remembered; the Peacekeeper is ordered to attack it. Then player
/// 2 cancels the deployment, which turns the Beacon back into the Drone, the same object (`sc_sim::deploy`), and walks
/// it away unseen: the attack's ring and the rally point's (and so the rally line, which ends at it) stay around the
/// record's footprint, and the Barracks' InfoPanel still names the Beacon, while the Drone moves away.
#[test]
fn markers_on_a_remembered_enemy_stay_put_while_it_moves_in_the_fog() {
    let mut app = beacon_world();
    let beacon = ObjectId(DRONE);
    order(&mut app, P2, DRONE, OrderKind::Deploy { origin: SITE });
    let start = tick(&app);
    while object(&app, DRONE).unwrap().kind != ObjectKind::Beacon {
        assert!(tick(&app) < start + 20, "the deployment never started");
        app.update();
    }
    let rally = ProductionCommand::SetRally(Target::Object(beacon));
    let objects = vec![ObjectId(BARRACKS)];
    submit(
        &mut app,
        P1,
        Command::Production {
            objects,
            command: rally,
        },
    );
    let selection = Selection::new([
        (ObjectId(BARRACKS), ObjectKind::Barracks),
        (ObjectId(SENTRY), ObjectKind::Peacekeeper),
    ]);
    app.world_mut().resource_mut::<ControlStates>().of_mut(P1).selection = selection;
    move_to(&mut app, ObjectId(SENTRY), 38, 2);
    assert_eq!(remembered(&app), [DRONE], "the Beacon is remembered, out of sight");
    order(&mut app, P1, SENTRY, OrderKind::Attack(beacon));
    app.update();

    let footprint = deploy::site(SITE);
    let expected = markers(&app);
    let ring = expected[0].1.expect("a ring");
    let marked = Indicator::Object {
        id: beacon,
        color: ATTACK_COLOR,
    };
    assert_eq!(
        expected,
        [(marked, Some(ring)); 2],
        "the attack's and the rally point's"
    );
    assert_eq!(
        (ring.center, ring.height),
        (footprint.center(), 0.0),
        "on the record's footprint"
    );
    assert!(ring.radius > Vec2::splat(SU_PER_GU).length(), "around it: {ring:?}");
    let named = |app: &App| {
        let sim = app.world().resource::<SimRes>();
        let barracks = sim.object(ObjectId(BARRACKS)).unwrap();
        activity::lines(sim, &Sight::new(sim, P1, true), barracks)
    };
    assert_eq!(named(&app), ["Rally point: Beacon (enemy)"]);

    let cancel = ConstructionCommand::CancelTransform;
    submit(
        &mut app,
        P2,
        Command::Construction {
            objects: vec![beacon],
            command: cancel,
        },
    );
    app.update();
    let drone = object(&app, DRONE).unwrap();
    assert_eq!((drone.kind, drone.pos), (ObjectKind::Drone, footprint.center()));
    order(&mut app, P2, DRONE, OrderKind::Move(Target::Point(centre(15, 14))));
    let start = tick(&app);
    while object(&app, DRONE).unwrap().pos.distance(footprint.center()) < 2.0 * ring.radius {
        assert!(tick(&app) < start + 300, "the Drone never walked away");
        app.update();
        assert_eq!(markers(&app), expected, "tick {}", tick(&app));
    }
    assert!(!drawn(&app, DRONE), "the Drone walked in the fog");
    assert_eq!(remembered(&app), [DRONE], "still remembered as a Beacon");
    assert_eq!(named(&app), ["Rally point: Beacon (enemy)"]);
    let sim = app.world().resource::<SimRes>();
    assert!(
        !sim.player(P2).unwrap().is_defeated(),
        "its Egress Node keeps player 2 in the game"
    );
}
