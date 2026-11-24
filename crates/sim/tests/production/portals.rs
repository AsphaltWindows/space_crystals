//! The Colonists' portals (slice M3-D, plan decision T19): `factions/colonists/structures.md > Robo Portal`, `> Bio
//! Portal`. A unit leaves by the exit side closest to the rally point, else by a random unblocked one, drawn from
//! the sim's Rng (`> Bio Portal > Exit Behavior`, which the Robo Portal follows over its four sides: question Q0041
//! point 3); a door a unit still stands in is blocked, and the portal pauses only when every door is
//! (`core/entities.md > Structure Type > Exit Doorway`).

use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    ArmyCap, ArmyResource, BuildingOrientation, DebugCommand, Doorway, Faction, MoveStatus, ObjectId, ObjectKind,
    OrderKind, ProductionCommand, Rally, Reject, Rotation, SimEvent, Target, Tick, Vec2, math,
};

use super::doors::{exit_of, step_checked, waiting};
use super::queue::{bank, crystals, queue, rally_of, validate};
use super::{P1, P2, PK, at, flat, production, tile};

const DRONE: ObjectKind = ObjectKind::Drone;
const SABRE: ObjectKind = ObjectKind::SabreLegionnaire;
const NODE: ObjectKind = ObjectKind::EgressNode;

/// The portal of [`robo_portal`] and [`bio_portal`].
const PORTAL: ObjectId = ObjectId(1);

/// Which way a door faces: north, east, south, west.
const N: Vec2 = Vec2::NEG_Y;
const E: Vec2 = Vec2::X;
const S: Vec2 = Vec2::Y;
const W: Vec2 = Vec2::NEG_X;

/// Player 1, Colonists, with a Robo Portal (1) on tiles (12, 6) to (15, 9) of a 30 × 20 map: x 768..1024, y
/// 384..640, its doors in the middle of each side: north (896, 384), east (1024, 512), south (896, 640) and west
/// (768, 512). Two Egress Nodes (2, 3) in the far corner: Beacon Capacity 15.
fn robo_portal() -> ScenarioBuilder {
    ScenarioBuilder::new(&flat(30, 20))
        .faction(P1, Faction::Colonists)
        .spawn(ObjectKind::RoboPortal, Some(P1), tile(12, 6))
        .spawn(NODE, Some(P1), tile(1, 1))
        .spawn(NODE, Some(P1), tile(1, 4))
}

/// Player 1, Colonists, with a Bio Portal (1) turned as given on tiles (12, 6) to (14, 8) of a 30 × 20 map: x
/// 768..960, y 384..576, its centre (864, 480). Two Egress Nodes (2, 3) in the far corner, and `more`.
fn bio_portal(rotation: Rotation, flip_h: bool, more: impl FnOnce(ScenarioBuilder) -> ScenarioBuilder) -> TestSim {
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
fn train(t: &mut TestSim, tick: Tick, kind: ObjectKind, n: usize) {
    for _ in 0..n {
        t.at(tick, P1, production(&[PORTAL], ProductionCommand::Train(kind)));
    }
}

fn set_rally(t: &mut TestSim, tick: Tick, target: Target) {
    t.at(tick, P1, production(&[PORTAL], ProductionCommand::SetRally(target)));
}

fn beacon_capacity(used: u32, available: u32) -> Option<ArmyCap> {
    Some(ArmyCap {
        resource: ArmyResource::BeaconCapacity,
        used,
        available,
    })
}

/// A unit that [`PORTAL`] released: when, which, where it appeared, and which way the door it appeared in faces.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Exit {
    tick: Tick,
    unit: ObjectId,
    pos: Vec2,
    side: Vec2,
}

/// Steps `t` until tick `until` under the doorway checks (`doors::step_checked`), and notes each unit [`PORTAL`]
/// releases as it appears: in the middle of one of its doors, at most half outside, facing out (`> Exit Doorway`).
fn run_exits(t: &mut TestSim, until: Tick) -> Vec<Exit> {
    let mut out = Vec::new();
    let mut exits = Vec::new();
    while t.sim().tick() < until {
        step_checked(t, &mut out);
        let (sim, tick) = (t.sim(), t.sim().tick());
        for (_, event) in t.events().iter().filter(|(made, _)| *made == tick) {
            let SimEvent::UnitProduced { producer: PORTAL, unit } = *event else {
                continue;
            };
            let o = sim.object(unit).unwrap();
            let door = Doorway::of(sim.object(PORTAL).unwrap()).unwrap().door_at(o.pos);
            let side = door.outward();
            let what = format!("#{} at {} in the door facing {side}", unit.0, o.pos);
            assert_eq!((o.pos - door.middle()).dot(side.perp()), 0.0, "{what}: on the middle");
            assert!((-18.04..=0.0).contains(&door.out_of(o.pos)), "{what}: at most half out");
            assert_eq!(math::facing_vector(o.facing).round(), side, "{what}: facing out");
            assert_eq!(exit_of(sim, unit), Some(PORTAL), "{what}");
            exits.push(Exit {
                tick,
                unit,
                pos: o.pos,
                side,
            });
        }
    }
    exits
}

fn sides(exits: &[Exit]) -> Vec<Vec2> {
    exits.iter().map(|exit| exit.side).collect()
}

/// The first order of `unit`.
fn order_of(t: &TestSim, unit: ObjectId) -> Option<OrderKind> {
    let orders = t.sim().object(unit).unwrap().orders.as_ref().unwrap();
    orders.queue.front().map(|order| order.kind)
}

/// The sim's Rng, as a save holds it.
fn rng_of(t: &TestSim) -> serde_json::Value {
    serde_json::to_value(t.sim()).unwrap()["rng"].clone()
}

/// Plan acceptance: "Drone 50 SC paid at queue; ordered tick t → appears at door closest to rally at end of t+159,
/// acts from t+160" (`> Robo Portal > Produces`: "Drone: 50 Space Crystals, 160 frames (10 seconds)"); the rally
/// point lies east, so the east door. Choosing it draws nothing from the Rng.
#[test]
fn a_drone_takes_160_frames_and_leaves_by_the_door_closest_to_the_rally_point() {
    let mut t = robo_portal().start();
    let point = Vec2::new(1700.0, 500.0);
    t.debug_at(1, crystals(P1, 120));
    set_rally(&mut t, 1, Target::Point(point));
    train(&mut t, 5, DRONE, 1);
    t.run_until(5);
    let rng = rng_of(&t);
    assert_eq!(
        (bank(t.sim(), P1), t.sim().army_cap(P1)),
        (70, beacon_capacity(1, 15)),
        "50 paid and its Beacon Capacity reserved when queued"
    );
    assert_eq!(run_exits(&mut t, 163), []);
    let exits = run_exits(&mut t, 164);
    assert_eq!(sides(&exits), [E]);
    let exit = exits[0];
    assert_eq!(
        (exit.tick, exit.pos),
        (164, Vec2::new(1024.0, 512.0)),
        "half inside the east side"
    );
    assert_eq!(t.sim().object(exit.unit).unwrap().prev_pos, exit.pos);
    assert_eq!(order_of(&t, exit.unit), Some(OrderKind::Move(Target::Point(point))));
    assert_eq!(rng_of(&t), rng, "one door was closest: no draw");
    t.run(1);
    assert_ne!(t.pos(exit.unit.0), exit.pos, "it acts from the next tick");
    t.run_until(400);
    assert_eq!(t.pos(exit.unit.0), point);
}

/// Question Q0041 point 4 (placeholder): "closest" is the straight line from the middle of the door to the rally
/// point, and ties go top, right, bottom, left. A rally point beyond any side takes the Drone out of that side;
/// one as far from two doors takes it out of the first of them; an object's (an own Egress Node: Move to it), out
/// of the door closest to where it stands.
#[test]
fn each_drone_leaves_by_the_side_closest_to_its_rally_point() {
    let cases = [
        (Target::Point(Vec2::new(900.0, 60.0)), N),
        (Target::Point(Vec2::new(1800.0, 700.0)), E),
        (Target::Point(Vec2::new(700.0, 1200.0)), S),
        (Target::Point(Vec2::new(60.0, 300.0)), W),
        // 300 su across and 300 up or down from the centre, (896, 512): 300 and 172 from the middles of two doors.
        (Target::Point(Vec2::new(1196.0, 212.0)), N),
        (Target::Point(Vec2::new(1196.0, 812.0)), E),
        (Target::Point(Vec2::new(596.0, 812.0)), S),
        (Target::Point(Vec2::new(596.0, 212.0)), N),
        // The Egress Node on tiles (1, 1) to (2, 2), its centre (128, 128): closer to the west door.
        (Target::Object(ObjectId(2)), W),
    ];
    for (target, side) in cases {
        let mut t = robo_portal().start();
        t.debug_at(1, crystals(P1, 50));
        set_rally(&mut t, 1, target);
        train(&mut t, 1, DRONE, 1);
        let exits = run_exits(&mut t, 160);
        assert_eq!(sides(&exits), [side], "{target:?}");
        let order = match target {
            Target::Point(_) => OrderKind::Move(target),
            Target::Object(id) => OrderKind::Move(Target::Object(id)),
        };
        assert_eq!(order_of(&t, exits[0].unit), Some(order));
    }
}

/// Plan acceptance: with no rally point the door is random, from the sim's Rng (`> Bio Portal > Exit Behavior`:
/// "chosen at random among unblocked candidates"): the same seed gives the same doors, run after run, and another
/// seed others. The Drones leave by several sides, and with every door free when a Drone is done, production never
/// pauses.
#[test]
fn with_no_rally_point_the_door_is_random_by_the_seed() {
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
        sides(&exits)
    };
    let first = run(1);
    assert_eq!(run(1), first, "the same seed");
    let mut used: Vec<[i32; 2]> = first.iter().map(|side| side.as_ivec2().to_array()).collect();
    used.sort();
    used.dedup();
    assert!(used.len() >= 2, "several sides: {first:?}");
    assert_ne!(run(2), first, "another seed");
}

/// Plan acceptance: "chosen door blocked → another free door". Two Egress Nodes flush against the east side close
/// it off ("obstructed, not currently traversable"): with the rally point east, the Drones leave by the other
/// doors, at random, and walk round to it.
#[test]
fn a_drone_leaves_by_another_door_when_the_closest_is_walled_off() {
    let mut t = robo_portal()
        .spawn(NODE, Some(P1), tile(16, 6))
        .spawn(NODE, Some(P1), tile(16, 8))
        .start();
    let point = Vec2::new(1500.0, 512.0);
    t.debug_at(1, crystals(P1, 1000));
    set_rally(&mut t, 1, Target::Point(point));
    train(&mut t, 1, DRONE, 4);
    let rng = rng_of(&t);
    let exits = run_exits(&mut t, 650);
    assert_eq!(exits.len(), 4);
    assert!(sides(&exits).iter().all(|&side| side != E), "{:?}", sides(&exits));
    assert_ne!(rng_of(&t), rng, "drawn among the three others");
    // Round the wall (x 1024..1152) to the point, where the four of them crowd.
    t.run_until(1400);
    for exit in exits {
        let pos = t.pos(exit.unit.0);
        assert!(pos.x > 1152.0 && pos.distance(point) < 200.0, "{pos}");
        assert_eq!(t.status(exit.unit.0), MoveStatus::Arrived);
    }
}

/// `> Exit Doorway`: "a side whose doorway is occupied counts as blocked". Enemy units (holding their fire) stand
/// along the east side: the first Drone, rallied east, appears in the east doorway, fully inside, and stays there;
/// the next ones leave by the other doors.
#[test]
fn a_drone_leaves_by_another_door_while_the_closest_is_occupied() {
    let mut builder = robo_portal();
    // Eleven Peacekeepers (24 su) touching one another from y 384 to 648, 1 su off the east side.
    for i in 0..11 {
        builder = builder.spawn(PK, Some(P2), at(1037.0, 396.0 + 24.0 * i as f32));
    }
    let mut t = builder.start();
    t.without_auto_targeting();
    t.debug_at(1, crystals(P1, 1000));
    set_rally(&mut t, 1, Target::Point(Vec2::new(1700.0, 512.0)));
    train(&mut t, 1, DRONE, 3);
    let exits = run_exits(&mut t, 500);
    assert_eq!(sides(&exits)[0], E);
    let first = exits[0].unit;
    assert_eq!(
        exits[0].pos,
        Vec2::new(1024.0 - 17.0 - 1.0 / 64.0, 512.0),
        "as far out as there is room"
    );
    assert_eq!(exits.len(), 3);
    assert!(sides(&exits)[1..].iter().all(|&side| side != E), "{:?}", sides(&exits));
    let door = Doorway::of(t.sim().object(PORTAL).unwrap())
        .unwrap()
        .door_at(t.pos(first.0));
    assert_eq!(
        (exit_of(t.sim(), first), door.outward()),
        (Some(PORTAL), E),
        "still in the east doorway"
    );
}

/// Plan acceptance: "all blocked → pause at 100%". The Robo Portal walled in on every side by Egress Nodes: each
/// Drone appears fully inside a free door and stays there, and the fifth waits, finished, behind the four ("the
/// building pauses only when every eligible doorway is occupied"). Take away the east wall: the Drone there walks
/// out, and the fifth comes out of the east door.
#[test]
fn a_portal_pauses_while_every_door_is_occupied() {
    let walls = [(12, 4), (14, 4), (16, 6), (16, 8), (12, 10), (14, 10), (10, 6), (10, 8)];
    let mut builder = robo_portal();
    for (x, y) in walls {
        builder = builder.spawn(NODE, Some(P1), tile(x, y));
    }
    let mut t = builder.start();
    let east_wall = [ObjectId(6), ObjectId(7)];
    t.debug_at(1, crystals(P1, 1000));
    train(&mut t, 1, DRONE, 5);
    let exits = run_exits(&mut t, 1000);
    assert_eq!(
        exits.iter().map(|exit| exit.tick).collect::<Vec<_>>(),
        [160, 320, 480, 640]
    );
    let mut used: Vec<[i32; 2]> = sides(&exits).iter().map(|side| side.as_ivec2().to_array()).collect();
    used.sort();
    assert_eq!(used, [[-1, 0], [0, -1], [0, 1], [1, 0]], "one Drone in each door");
    for exit in &exits {
        assert_eq!(t.pos(exit.unit.0), exit.pos, "still in its doorway");
        assert_eq!(exit_of(t.sim(), exit.unit), Some(PORTAL));
    }
    let status = t.sim().production_status(PORTAL).unwrap();
    assert!(status.waiting && status.progress == status.total, "held at 100%");
    assert!(waiting(t.sim(), PORTAL));
    assert_eq!(queue(t.sim(), PORTAL), 1);
    for wall in east_wall {
        t.debug_at(1001, DebugCommand::Remove { id: wall });
    }
    let after = run_exits(&mut t, 1020);
    let east = exits.iter().find(|exit| exit.side == E).unwrap();
    assert_eq!(exit_of(t.sim(), east.unit), None, "it walked out");
    assert_eq!(sides(&after), [E], "the fifth, through the door that came free");
    assert!(
        after[0].tick > 1001 && after[0].tick <= 1008,
        "at once: {}",
        after[0].tick
    );
}

/// Plan acceptance: the Bio Portal's Sabre like the Drone: "Sabre Legionnaire: 100 Space Crystals, 160 frames",
/// out of the A side closest to the rally point.
#[test]
fn a_sabre_takes_160_frames_at_the_bio_portal() {
    let mut t = bio_portal(Rotation::R0, false, |b| b);
    let point = Vec2::new(1500.0, 480.0);
    t.debug_at(1, crystals(P1, 250));
    set_rally(&mut t, 1, Target::Point(point));
    train(&mut t, 3, SABRE, 1);
    t.run_until(3);
    assert_eq!((bank(t.sim(), P1), t.sim().army_cap(P1)), (150, beacon_capacity(2, 15)));
    assert_eq!(run_exits(&mut t, 161), []);
    let exits = run_exits(&mut t, 162);
    assert_eq!(
        (exits[0].tick, exits[0].pos, exits[0].side),
        (162, Vec2::new(960.0, 480.0), E)
    );
    assert_eq!(order_of(&t, exits[0].unit), Some(OrderKind::Move(Target::Point(point))));
    t.run(1);
    assert_ne!(t.pos(exits[0].unit.0), exits[0].pos, "it acts from the next tick");
}

/// Plan acceptance: the Bio Portal's units leave "only from A sides in all 8 orientations" (`> Bio Portal > Exit
/// Behavior`: "two exit-eligible sides (the "A" sides, per its AABB symmetry)"; unturned, A is the top and the
/// right side, design change 0079): by the A side closest to the rally point wherever that lies, and by one of the
/// two with none.
#[test]
fn the_bio_portal_releases_its_units_only_from_its_a_sides_in_all_eight_orientations() {
    let cases = [
        (Rotation::R0, false, [N, E]),
        (Rotation::R90, false, [E, S]),
        (Rotation::R180, false, [S, W]),
        (Rotation::R270, false, [N, W]),
        (Rotation::R0, true, [N, W]),
        (Rotation::R90, true, [N, E]),
        (Rotation::R180, true, [E, S]),
        (Rotation::R270, true, [S, W]),
    ];
    let centre = Vec2::new(864.0, 480.0);
    for (rotation, flip_h, a_sides) in cases {
        for rally in [None, Some(N), Some(E), Some(S), Some(W)] {
            let mut t = bio_portal(rotation, flip_h, |b| b);
            let what = format!("{rotation:?}, flipped {flip_h}, rally {rally:?}");
            t.debug_at(1, crystals(P1, 200));
            if let Some(direction) = rally {
                set_rally(&mut t, 1, Target::Point(centre + direction * 400.0));
            }
            train(&mut t, 1, SABRE, 2);
            let exits = run_exits(&mut t, 330);
            assert_eq!(exits.len(), 2, "{what}");
            assert!(
                sides(&exits).iter().all(|side| a_sides.contains(side)),
                "{what}: {:?}",
                sides(&exits)
            );
            if let Some(direction) = rally {
                // The A side whose middle (96 su out from the centre) is nearer to the point.
                let distance = |side: Vec2| (centre + side * 96.0).distance(centre + direction * 400.0);
                let nearest = if distance(a_sides[1]) < distance(a_sides[0]) {
                    a_sides[1]
                } else {
                    a_sides[0]
                };
                assert_eq!(sides(&exits), [nearest; 2], "{what}");
            }
        }
    }
}

/// `> Robo Portal > RallyPoint behavior`: "For a Drone: ... rally on enemy -> Move to that location (Drone is unarmed
/// and cannot attack)"; the Bio Portal's Sabre, armed, gets the default right-click command on it: Attack (`> Bio
/// Portal > RallyPoint behavior`). The enemy is a GDO Generator east of the portal, which a Drone of player 1 keeps
/// in sight.
#[test]
fn a_drone_rallied_on_an_enemy_moves_to_it_and_a_sabre_attacks_it() {
    // The Generator on tiles (18, 7) to (19, 8): its centre (1216, 512). The Drone south of it.
    let enemy_and_eyes = |b: ScenarioBuilder| {
        b.spawn(ObjectKind::Generator, Some(P2), tile(18, 7))
            .spawn(DRONE, Some(P1), at(1240.0, 700.0))
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
    assert_eq!(
        order_of(&t, exits[0].unit),
        Some(OrderKind::Move(Target::Point(generator)))
    );

    let mut t = bio_portal(Rotation::R0, false, enemy_and_eyes);
    t.debug_at(1, crystals(P1, 100));
    set_rally(&mut t, 2, Target::Object(enemy));
    train(&mut t, 2, SABRE, 1);
    let exits = run_exits(&mut t, 161);
    assert_eq!(sides(&exits), [E]);
    assert_eq!(order_of(&t, exits[0].unit), Some(OrderKind::Attack(enemy)));
}

/// Plan acceptance, after design changes 0084 and 0085: a portal destroyed with Drones queued refunds them in full,
/// and the Beacon Capacity they held is free again (the 5 the portal provided goes with it). A debug removal stands
/// for the destruction, as for the Barracks.
#[test]
fn a_destroyed_portal_refunds_its_queue_and_frees_its_capacity() {
    let mut t = robo_portal().start();
    t.debug_at(1, crystals(P1, 150));
    train(&mut t, 1, DRONE, 3);
    t.run(2);
    assert_eq!((bank(t.sim(), P1), t.sim().army_cap(P1)), (0, beacon_capacity(3, 15)));
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

/// Plan acceptance, design change 0112: a portal over capacity takes no new Drones (the gate refuses them; the ones
/// queued before complete, as slice M3-C has it), and trains again as soon as units are gone and there is room.
#[test]
fn an_over_capacity_portal_trains_again_once_there_is_room() {
    let mut builder = robo_portal();
    for i in 0..12 {
        builder = builder.spawn(DRONE, Some(P1), at(100.0 + 40.0 * i as f32, 1150.0));
    }
    let mut t = builder.start();
    let drones: Vec<ObjectId> = (4..16).map(ObjectId).collect();
    let train_one = production(&[PORTAL], ProductionCommand::Train(DRONE));
    t.debug_at(1, crystals(P1, 1000));
    train(&mut t, 2, DRONE, 2);
    t.debug_at(3, DebugCommand::Remove { id: ObjectId(2) });
    t.run(3);
    assert_eq!(t.sim().army_cap(P1), beacon_capacity(14, 10), "over capacity");
    assert_eq!(validate(t.sim(), P1, train_one.clone()), Err(Reject::UnitControlCap));
    let exits = run_exits(&mut t, 400);
    assert_eq!(exits.len(), 2, "the queue completes");
    assert_eq!(validate(t.sim(), P1, train_one.clone()), Err(Reject::UnitControlCap));
    // Five Drones gone: 9 used of 10.
    for &drone in &drones[..5] {
        t.debug_at(401, DebugCommand::Remove { id: drone });
    }
    t.run(1);
    assert_eq!(t.sim().army_cap(P1), beacon_capacity(9, 10));
    assert_eq!(validate(t.sim(), P1, train_one.clone()), Ok(()));
    t.at(402, P1, train_one.clone());
    let exits = run_exits(&mut t, 600);
    assert_eq!(exits.iter().map(|exit| exit.tick).collect::<Vec<_>>(), [561], "trained");
    assert_eq!(t.sim().army_cap(P1), beacon_capacity(10, 10));
    assert_eq!(
        validate(t.sim(), P1, train_one),
        Err(Reject::UnitControlCap),
        "full again"
    );
}

/// Determinism with the Rng drawn: a game whose portal picks its doors at random replays to the same state, and a
/// save made in the middle of it picks the same doors as the game.
#[test]
fn a_game_with_random_doors_replays_and_saves() {
    let mut t = robo_portal().seed(11).start();
    t.debug_at(1, crystals(P1, 1000));
    train(&mut t, 1, DRONE, 6);
    t.at(700, P1, production(&[PORTAL], ProductionCommand::Train(DRONE)));
    let mut exits = run_exits(&mut t, 400);
    let saved: sc_sim::Sim = serde_json::from_str(&serde_json::to_string(t.sim()).unwrap()).unwrap();
    assert_eq!(&saved, t.sim());
    exits.extend(run_exits(&mut t, 1300));
    assert_eq!(exits.len(), 7);
    let replayed = t.replay().run();
    assert_eq!(replayed.state_hash(), t.sim().state_hash());
    assert_eq!(&replayed, t.sim());
    let mut resumed = saved;
    for inputs in &t.replay().inputs[400..] {
        resumed.step(inputs);
    }
    assert_eq!(resumed.state_hash(), t.sim().state_hash());
}
