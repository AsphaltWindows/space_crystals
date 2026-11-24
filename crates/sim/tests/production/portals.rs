//! The Colonists' portals (slice M3-D; slice M3-D2, design change 0145): `factions/colonists/structures.md > Portals
//! > Portal Exits (energy doorway)`, `> Robo Portal`, `> Bio Portal > Exit Behavior`. A finished unit materializes at
//! the portal's centre and walks straight out along its heading: towards the rally point if that direction is open,
//! else the nearest open one whose way out is free; with no rally point, a random open free one, drawn from the
//! sim's Rng. The Robo Portal is open all round, the Bio Portal through its two A sides. A portal whose way out is
//! blocked: `portal_stalls`.

use sc_sim::production::exit::HEADINGS;
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    ArmyCap, ArmyResource, BuildingOrientation, DebugCommand, EnergyDoorway, Faction, MoveStatus, ObjectId, ObjectKind,
    OrderKind, ProductionCommand, Rally, Rotation, Sim, SimEvent, Target, Tick, Vec2, math,
};

use super::doors::{exit_of, step_checked};
use super::queue::{bank, crystals, rally_of};
use super::{P1, flat, production, tile};

pub(super) const DRONE: ObjectKind = ObjectKind::Drone;
pub(super) const SABRE: ObjectKind = ObjectKind::SabreLegionnaire;
const NODE: ObjectKind = ObjectKind::EgressNode;

/// The portal of [`robo_portal`] and [`bio_portal`].
pub(super) const PORTAL: ObjectId = ObjectId(1);

/// The centre of [`robo_portal`]'s Robo Portal, and of [`bio_portal`]'s Bio Portal.
pub(super) const ROBO: Vec2 = Vec2::new(896.0, 512.0);
const BIO: Vec2 = Vec2::new(864.0, 480.0);

/// The half extents of a Drone.
const HALF: Vec2 = Vec2::splat(18.0);

/// Player 1, Colonists, with a Robo Portal (1) on tiles (12, 6) to (15, 9) of a 30 × 20 map: x 768..1024, y
/// 384..640, its centre [`ROBO`]. Two Egress Nodes (2, 3) in the far corner: Beacon Capacity 15.
pub(super) fn robo_portal() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(30, 20))
        .faction(P1, Faction::Colonists)
        .spawn(ObjectKind::RoboPortal, Some(P1), tile(12, 6))
        .spawn(NODE, Some(P1), tile(1, 1))
        .spawn(NODE, Some(P1), tile(1, 4))
}

/// Player 1, Colonists, with a Bio Portal (1) turned as given on tiles (12, 6) to (14, 8) of a 30 × 20 map: x
/// 768..960, y 384..576, its centre [`BIO`]. Two Egress Nodes (2, 3) in the far corner, and `more`.
pub(super) fn bio_portal(rotation: Rotation, flip_h: bool, more: fn(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
    let builder = ScenarioBuilder::new(&flat(30, 20))
        .faction(P1, Faction::Colonists)
        .spawn(ObjectKind::BioPortal, Some(P1), tile(12, 6))
        .spawn(NODE, Some(P1), tile(1, 1))
        .spawn(NODE, Some(P1), tile(1, 4));
    let mut scenario = more(builder).build();
    scenario.map.spawns[0].orientation = BuildingOrientation {
        rotation,
        flip_h,
        flip_v: false,
    };
    TestSim::new(scenario)
}

/// Queues `n` units of `kind` at [`PORTAL`] in `tick`.
pub(super) fn train(t: &mut TestSim, tick: Tick, kind: ObjectKind, n: usize) {
    for _ in 0..n {
        t.at(tick, P1, production(&[PORTAL], ProductionCommand::Train(kind)));
    }
}

pub(super) fn set_rally(t: &mut TestSim, tick: Tick, target: Target) {
    t.at(tick, P1, production(&[PORTAL], ProductionCommand::SetRally(target)));
}

pub(super) fn beacon_capacity(used: u32, available: u32) -> Option<ArmyCap> {
    Some(ArmyCap {
        resource: ArmyResource::BeaconCapacity,
        used,
        available,
    })
}

/// A unit that [`PORTAL`] released: when, which, and the heading it leaves along (`Mover::exit_heading`).
#[derive(Copy, Clone, Debug, PartialEq)]
pub(super) struct Exit {
    pub(super) tick: Tick,
    pub(super) unit: ObjectId,
    pub(super) heading: Vec2,
}

/// Steps `t` until tick `until` under the doorway checks (`doors::step_checked`), and notes each unit [`PORTAL`]
/// releases as it appears: at the portal's centre, in its doorway, facing its heading, which is open (design change
/// 0145, points 1 and 2). Every tick, each unit it released is still on its heading's ray from the centre while it
/// is in the doorway (it walks straight out).
pub(super) fn run_exits(t: &mut TestSim, until: Tick) -> Vec<Exit> {
    let mut out = Vec::new();
    let mut exits: Vec<Exit> = Vec::new();
    while t.sim().tick() < until {
        step_checked(t, &mut out);
        let (sim, tick) = (t.sim(), t.sim().tick());
        let Some(portal) = sim.object(PORTAL).and_then(EnergyDoorway::of) else {
            continue;
        };
        for (_, event) in t.events().iter().filter(|(made, _)| *made == tick) {
            let SimEvent::UnitProduced { producer: PORTAL, unit } = *event else {
                continue;
            };
            let o = sim.object(unit).unwrap();
            let mover = o.mover.as_ref().unwrap();
            let heading = mover.exit_heading.expect("a unit in an energy doorway has a heading");
            let what = format!("#{} at {} heading {heading}", unit.0, o.pos);
            assert_eq!(
                (o.pos, o.prev_pos),
                (portal.centre(), portal.centre()),
                "{what}: at the centre"
            );
            assert_eq!(mover.exit, Some(PORTAL), "{what}: in the doorway");
            assert!(portal.is_open(heading), "{what}: an open direction");
            assert!(
                math::facing_vector(o.facing).distance(heading) < 1e-5,
                "{what}: facing it"
            );
            exits.push(Exit { tick, unit, heading });
        }
        for exit in &exits {
            let Some(o) = sim
                .object(exit.unit)
                .filter(|o| o.mover.as_ref().unwrap().exit.is_some())
            else {
                continue;
            };
            let along = o.pos - portal.centre();
            assert!(
                along.perp_dot(exit.heading).abs() < 1e-3 && along.dot(exit.heading) >= 0.0,
                "tick {tick}: #{} at {} is off its ray {}",
                exit.unit.0,
                o.pos,
                exit.heading
            );
        }
    }
    exits
}

fn headings(exits: &[Exit]) -> Vec<Vec2> {
    exits.iter().map(|exit| exit.heading).collect()
}

/// The first order of `unit`.
fn order_of(t: &TestSim, unit: ObjectId) -> Option<OrderKind> {
    let orders = t.sim().object(unit).unwrap().orders.as_ref().unwrap();
    orders.queue.front().map(|order| order.kind)
}

/// The sim's Rng, as a save holds it.
pub(super) fn rng_of(t: &TestSim) -> serde_json::Value {
    serde_json::to_value(t.sim()).unwrap()["rng"].clone()
}

/// 0145's Verification: "Robo Portal rallied northeast: Drones emerge from the centre heading northeast"; and the
/// plan's timing: "Drone 50 SC paid at queue; ordered tick t → appears at end of t+159, acts from t+160" (`> Robo
/// Portal > Produces`: "Drone: 50 Space Crystals, 160 frames (10 seconds)"). With a rally point nothing is drawn.
#[test]
fn a_drone_takes_160_frames_and_emerges_from_the_centre_heading_for_its_rally_point() {
    let mut t = robo_portal().start();
    let point = ROBO + Vec2::new(400.0, -400.0);
    t.debug_at(1, crystals(P1, 120));
    set_rally(&mut t, 1, Target::Point(point));
    train(&mut t, 5, DRONE, 1);
    t.run_until(5);
    let rng = rng_of(&t);
    assert_eq!(
        (bank(t.sim(), P1), t.sim().army_cap(P1)),
        (70, beacon_capacity(1, 15)),
        "50 paid when queued, and its Beacon Capacity taken as its production starts (design change 0143)"
    );
    assert_eq!(run_exits(&mut t, 163), []);
    let exits = run_exits(&mut t, 164);
    assert_eq!(exits.len(), 1);
    let exit = exits[0];
    assert_eq!((exit.tick, exit.heading), (164, (point - ROBO).normalize()));
    assert_eq!(order_of(&t, exit.unit), Some(OrderKind::Move(Target::Point(point))));
    assert_eq!(rng_of(&t), rng, "no draw");
    // It acts from the next tick, along its heading, and keeps to it until it is out.
    run_exits(&mut t, 165);
    let moved = t.pos(exit.unit.0) - ROBO;
    assert!(moved.length() > 0.0, "it acts from the next tick");
    run_exits(&mut t, 220);
    assert_eq!(exit_of(t.sim(), exit.unit), None, "out");
    t.run_until(400);
    assert_eq!(t.pos(exit.unit.0), point);
}

/// Point 3: "straight toward the RallyPoint if that direction is open", every direction for the Robo Portal. An
/// object's rally point is where its owner last saw it (an own Egress Node: Move to it).
#[test]
fn each_drone_heads_straight_for_its_rally_point() {
    let cases = [
        Target::Point(Vec2::new(900.0, 60.0)),
        Target::Point(Vec2::new(1800.0, 700.0)),
        Target::Point(Vec2::new(700.0, 1200.0)),
        Target::Point(Vec2::new(60.0, 300.0)),
        Target::Point(Vec2::new(1196.0, 812.0)),
        // The Egress Node on tiles (1, 1) to (2, 2), its centre (128, 128).
        Target::Object(ObjectId(2)),
    ];
    for target in cases {
        let mut t = robo_portal().start();
        t.debug_at(1, crystals(P1, 50));
        set_rally(&mut t, 1, target);
        train(&mut t, 1, DRONE, 1);
        let exits = run_exits(&mut t, 160);
        let point = match target {
            Target::Point(point) => point,
            Target::Object(_) => Vec2::new(128.0, 128.0),
        };
        assert_eq!(headings(&exits), [(point - ROBO).normalize()], "{target:?}");
        assert_eq!(order_of(&t, exits[0].unit), Some(OrderKind::Move(target)));
    }
}

/// 0145's Verification: "Bio Portal (default orientation) rallied to the southwest (a closed side): units exit
/// through the nearest open direction (toward the arc end) and walk around." Its arc runs from the north-west corner
/// round to the south-east one, both 90° from south-west: the clockwise one, north-west, goes first (the Director's
/// reading). Out there, the Sabre walks round the portal's west side to the point.
#[test]
fn a_bio_portal_rallied_behind_it_sends_its_units_round_by_the_nearer_end_of_its_arc() {
    let mut t = bio_portal(Rotation::R0, false, |b| b);
    let point = BIO + Vec2::new(-400.0, 400.0);
    t.debug_at(1, crystals(P1, 100));
    set_rally(&mut t, 1, Target::Point(point));
    train(&mut t, 1, SABRE, 1);
    let rng = rng_of(&t);
    let exits = run_exits(&mut t, 200);
    assert_eq!(exits.len(), 1);
    let north_west = Vec2::new(-1.0, -1.0).normalize();
    assert!(exits[0].heading.distance(north_west) < 1e-6, "{}", exits[0].heading);
    assert_eq!(rng_of(&t), rng, "no draw");
    let unit = exits[0].unit.0;
    // Out at the north-west corner, x 768 and y 384 less its half: then down the west side.
    assert_eq!(exit_of(t.sim(), exits[0].unit), None);
    let mut beside = false;
    while t.sim().tick() < 600 {
        t.step();
        let pos = t.pos(unit);
        beside |= pos.x <= 750.0 && (384.0..576.0).contains(&pos.y);
    }
    assert!(beside, "round the west side");
    assert_eq!((t.pos(unit), t.status(unit)), (point, MoveStatus::Arrived));
}

/// The end of a Bio Portal's arc nearer to the closed direction `d`, its arc open towards `towards` (the sum of its
/// two open sides' outward directions): of the two diagonals across `towards`, the one at the smaller angle from `d`,
/// the clockwise one (on screen) of two as near.
fn nearer_end(d: Vec2, towards: Vec2) -> Vec2 {
    let ends = [
        Vec2::new(-towards.y, towards.x).normalize(),
        Vec2::new(towards.y, -towards.x).normalize(),
    ];
    let key = |end: Vec2| (d.dot(end), d.perp_dot(end));
    if key(ends[0]) >= key(ends[1]) { ends[0] } else { ends[1] }
}

/// The Bio Portal's arc in all 8 orientations (`> Bio Portal > Exit Behavior`: its two open sides are its A sides,
/// top and right unturned, design change 0079, turning with it): towards a rally point in an open direction, else by
/// the nearer end of the arc (the clockwise one of two as near); with none, an open direction. Each Sabre walks out
/// through an open side (invariant I2 holds it to the arc every tick) at 5 su a tick, and is clear of the portal
/// within 33 ticks: 114 su from its centre straight across, √2 times as far along a diagonal.
#[test]
fn the_bio_portal_releases_its_units_only_through_its_a_sides_in_all_eight_orientations() {
    let (n, e, s, w) = (Vec2::NEG_Y, Vec2::X, Vec2::Y, Vec2::NEG_X);
    let cases = [
        (Rotation::R0, false, [n, e]),
        (Rotation::R90, false, [e, s]),
        (Rotation::R180, false, [s, w]),
        (Rotation::R270, false, [n, w]),
        (Rotation::R0, true, [n, w]),
        (Rotation::R90, true, [n, e]),
        (Rotation::R180, true, [e, s]),
        (Rotation::R270, true, [s, w]),
    ];
    let rallies = [
        None,
        Some(n),
        Some(e),
        Some(s),
        Some(w),
        Some(n + e),
        Some(e + s),
        Some(s + w),
        Some(w + n),
    ];
    for (rotation, flip_h, open) in cases {
        let towards = open[0] + open[1];
        for rally in rallies {
            let what = format!("{rotation:?}, flipped {flip_h}, rally {rally:?}");
            let mut t = bio_portal(rotation, flip_h, |b| b);
            t.debug_at(1, crystals(P1, 100));
            if let Some(direction) = rally {
                set_rally(&mut t, 1, Target::Point(BIO + direction * 400.0));
            }
            train(&mut t, 1, SABRE, 1);
            let exits = run_exits(&mut t, 193);
            assert_eq!(exits.len(), 1, "{what}");
            let heading = exits[0].heading;
            assert!(towards.dot(heading) >= -1e-6, "{what}: {heading}");
            if let Some(direction) = rally {
                let toward = (direction * 400.0).normalize();
                let expected = if towards.dot(toward) >= 0.0 {
                    toward
                } else {
                    nearer_end(toward, towards)
                };
                assert!(heading.distance(expected) < 1e-6, "{what}: {heading}, not {expected}");
            }
            assert_eq!(exit_of(t.sim(), exits[0].unit), None, "{what}: out");
        }
    }
}

/// Point 3: "With no RallyPoint: a random open, unblocked direction (deterministic random)", drawn from the sim's Rng
/// among the 32 headings: the same seed gives the same headings, run after run, and another seed others. Each Drone
/// walks out and stands where it came clear of the portal, so later ones leave by other directions; no Drone waits.
#[test]
fn with_no_rally_point_the_heading_is_random_by_the_seed() {
    let run = |seed| {
        let mut t = robo_portal().seed(seed).start();
        t.debug_at(1, crystals(P1, 1000));
        train(&mut t, 1, DRONE, 8);
        let rng = rng_of(&t);
        let exits = run_exits(&mut t, 1300);
        assert_eq!(
            exits.iter().map(|exit| exit.tick).collect::<Vec<_>>(),
            [160, 320, 480, 640, 800, 960, 1120, 1280]
        );
        assert_ne!(rng_of(&t), rng, "drawn");
        let portal = EnergyDoorway::of(t.sim().object(PORTAL).unwrap()).unwrap();
        for exit in &exits {
            assert!(HEADINGS.contains(&exit.heading), "{}", exit.heading);
        }
        for exit in &exits[..7] {
            let stands = t.pos(exit.unit.0);
            let clear = portal.outside(ROBO, exit.heading, HALF);
            assert!(stands.distance(clear) < 1e-3, "{stands} for {clear}");
            assert_eq!(exit_of(t.sim(), exit.unit), None);
        }
        headings(&exits)
    };
    let first = run(1);
    assert_eq!(run(1), first, "the same seed");
    assert!(first.iter().any(|&heading| heading != first[0]), "{first:?}");
    assert_ne!(run(2), first, "another seed");
}

/// Point 3: "If it isn't [open], or the way out in that direction is blocked, the unit tries to go around: the
/// nearest open direction in which it can get out." Two Egress Nodes flush against the east side (x 1024..1152, y
/// 384..640) wall it off from a rally point due east. A Drone's box swept out of the centre clears them only at
/// 56.25° from east or more (11.25° steps): the southern of those two first (clockwise), and from there it walks round
/// the nodes to the point. With a rally point nothing is drawn.
#[test]
fn a_drone_goes_round_by_the_nearest_free_direction_when_its_way_is_walled_off() {
    let mut t = robo_portal()
        .spawn(NODE, Some(P1), tile(16, 6))
        .spawn(NODE, Some(P1), tile(16, 8))
        .start();
    let point = Vec2::new(1500.0, 512.0);
    t.debug_at(1, crystals(P1, 1000));
    set_rally(&mut t, 1, Target::Point(point));
    train(&mut t, 1, DRONE, 2);
    let rng = rng_of(&t);
    let exits = run_exits(&mut t, 330);
    assert_eq!(headings(&exits), [HEADINGS[5]; 2]);
    assert_eq!(rng_of(&t), rng, "no draw");
    t.run_until(900);
    for exit in exits {
        let pos = t.pos(exit.unit.0);
        assert!(pos.x > 1152.0 && pos.distance(point) < 100.0, "{pos}");
        assert_eq!(t.status(exit.unit.0), MoveStatus::Arrived);
    }
}

/// The Bio Portal's Sabre: "Sabre Legionnaire: 100 Space Crystals, 160 frames", towards the rally point due east. A
/// Sabre stops within the tick (TurnRateMovement, 5 su a tick): it steps 5 su along its heading every tick, and is
/// clear of the portal (x 768..960) once its centre is past x 978, on its 23rd step, which takes it the rest of the
/// way on along its heading.
#[test]
fn a_sabre_takes_160_frames_at_the_bio_portal_and_walks_straight_out() {
    let mut t = bio_portal(Rotation::R0, false, |b| b);
    let point = BIO + Vec2::new(512.0, 0.0);
    t.debug_at(1, crystals(P1, 250));
    set_rally(&mut t, 1, Target::Point(point));
    train(&mut t, 3, SABRE, 1);
    t.run_until(3);
    assert_eq!((bank(t.sim(), P1), t.sim().army_cap(P1)), (150, beacon_capacity(2, 15)));
    assert_eq!(run_exits(&mut t, 161), []);
    let exits = run_exits(&mut t, 162);
    assert_eq!((exits.len(), exits[0].tick, exits[0].heading), (1, 162, Vec2::X));
    assert_eq!(order_of(&t, exits[0].unit), Some(OrderKind::Move(Target::Point(point))));
    let unit = exits[0].unit;
    run_exits(&mut t, 163);
    assert_eq!(t.pos(unit.0), BIO + Vec2::new(5.0, 0.0), "it acts from the next tick");
    run_exits(&mut t, 184);
    assert_eq!(
        (t.pos(unit.0), exit_of(t.sim(), unit)),
        (Vec2::new(974.0, 480.0), Some(PORTAL))
    );
    run_exits(&mut t, 185);
    assert_eq!((t.pos(unit.0), exit_of(t.sim(), unit)), (Vec2::new(979.0, 480.0), None));
    t.run_until(300);
    assert_eq!((t.pos(unit.0), t.status(unit.0)), (point, MoveStatus::Arrived));
}

/// Design change 0136 (`systems/controls/selection_and_hud.md > What a rally point makes a new unit do`): each new
/// unit gets its own default right-click command on the rally target: the Drone, unarmed, a Move to the enemy
/// structure (`> Drone > Right-Click Resolution`), the Bio Portal's Sabre an Attack; both head for where the enemy
/// was seen. The enemy is a GDO Generator east of the portal, which a Drone of player 1 keeps in sight.
#[test]
fn a_drone_rallied_on_an_enemy_moves_to_it_and_a_sabre_attacks_it() {
    // The Generator on tiles (18, 7) to (19, 8): its centre (1216, 512). The Drone south of it.
    let enemy_and_eyes = |b: ScenarioBuilder| {
        b.spawn(ObjectKind::Generator, Some(super::P2), tile(18, 7))
            .spawn(DRONE, Some(P1), super::at(1240.0, 700.0))
    };
    let enemy = ObjectId(4);
    let generator = Vec2::new(1216.0, 512.0);

    let mut t = enemy_and_eyes(robo_portal()).start();
    t.debug_at(1, crystals(P1, 50));
    set_rally(&mut t, 2, Target::Object(enemy));
    train(&mut t, 2, DRONE, 1);
    let exits = run_exits(&mut t, 161);
    assert_eq!(
        rally_of(t.sim(), PORTAL),
        Some(Rally::Object {
            id: enemy,
            at: generator
        })
    );
    assert_eq!(headings(&exits), [(generator - ROBO).normalize()]);
    assert_eq!(
        order_of(&t, exits[0].unit),
        Some(OrderKind::Move(Target::Object(enemy)))
    );

    let mut t = bio_portal(Rotation::R0, false, enemy_and_eyes);
    t.debug_at(1, crystals(P1, 100));
    set_rally(&mut t, 2, Target::Object(enemy));
    train(&mut t, 2, SABRE, 1);
    let exits = run_exits(&mut t, 161);
    assert_eq!(headings(&exits), [(generator - BIO).normalize()]);
    assert_eq!(order_of(&t, exits[0].unit), Some(OrderKind::Attack(enemy)));
}

/// Plan acceptance, after design changes 0084 and 0085: a portal destroyed with Drones queued refunds them in full,
/// and the Beacon Capacity its Drone in production held, the only one that took any (design change 0143), is free
/// again (the 5 the portal provided goes with it). A debug removal stands for the destruction, as for the Barracks.
#[test]
fn a_destroyed_portal_refunds_its_queue_and_frees_its_capacity() {
    let mut t = robo_portal().start();
    t.debug_at(1, crystals(P1, 150));
    train(&mut t, 1, DRONE, 3);
    t.run(2);
    assert_eq!((bank(t.sim(), P1), t.sim().army_cap(P1)), (0, beacon_capacity(1, 15)));
    t.debug_at(100, DebugCommand::Remove { id: PORTAL });
    t.run_until(100);
    assert_eq!((bank(t.sim(), P1), t.sim().army_cap(P1)), (150, beacon_capacity(0, 10)));
    let refunds = t
        .events()
        .iter()
        .filter(|(_, e)| matches!(e, SimEvent::ProductionCancelled { refund: 50, .. }))
        .count();
    assert_eq!(refunds, 3);
    assert_eq!(dangling_references(t.sim()), []);
}

/// Determinism with the Rng drawn: a game whose portal picks its headings at random replays to the same state, and a
/// save made while a Drone walks out of the doorway (its heading saved with it) plays on as the game did.
#[test]
fn a_game_with_random_headings_replays_and_saves() {
    let mut t = robo_portal().seed(11).start();
    t.debug_at(1, crystals(P1, 1000));
    train(&mut t, 1, DRONE, 6);
    t.at(700, P1, production(&[PORTAL], ProductionCommand::Train(DRONE)));
    let mut exits = run_exits(&mut t, 330);
    let walking = t.sim().object(exits[1].unit).unwrap().mover.as_ref().unwrap();
    assert_eq!(
        (walking.exit, walking.exit_heading),
        (Some(PORTAL), Some(exits[1].heading))
    );
    let saved: Sim = serde_json::from_str(&serde_json::to_string(t.sim()).unwrap()).unwrap();
    assert_eq!(&saved, t.sim());
    exits.extend(run_exits(&mut t, 1300));
    assert_eq!(exits.len(), 7);
    let replayed = t.replay().run();
    assert_eq!(replayed.state_hash(), t.sim().state_hash());
    assert_eq!(&replayed, t.sim());
    let mut resumed = saved;
    for inputs in &t.replay().inputs[330..] {
        resumed.step(inputs);
    }
    assert_eq!(resumed.state_hash(), t.sim().state_hash());
}
