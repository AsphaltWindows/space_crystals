//! The army caps when production starts (design change 0143, for every faction; slice M3-P): queuing needs only the
//! cost and a queue slot, and the unit at the front of a queue starts only while its owner's cap has room for it
//! (`factions/gdo/overview.md > GDOResources > Unit Control`, "Reserved when production starts";
//! `factions/colonists/overview.md > Beacon Capacity > Reserved when production starts`). Without room the producer
//! waits at 0% (`ProductionStatus::blocked`, one `SimEvent::ProductionWaiting`) and starts by itself once there is;
//! a unit in production always finishes (Q0041 point 2). One test per point of 0143's Verification and of 0112's as
//! 0143 amends it, a portal over capacity (slice M3-D's acceptance), the refunds of a waiting and of a producing
//! producer (design changes 0084 and 0085), a cancel, a producer without power, and a game with waits saved and
//! replayed.

use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    ArmyResource, Command, ConstructionCommand, DebugCommand, Faction, ObjectId, ObjectKind, OrderKind,
    ProductionCommand, Sim, SimEvent, SpawnAt, Tick, TilePos, UNIT_CONTROL_CAP,
};

use super::queue::{bank, crystals, produced, queue, train, validate};
use super::{BARRACKS, P1, PK, at, base, flat, production, tile};

const UC: ArmyResource = ArmyResource::UnitControl;
const BC: ArmyResource = ArmyResource::BeaconCapacity;
const DRONE: ObjectKind = ObjectKind::Drone;

/// The Robo Portal of [`colonists`].
const PORTAL: ObjectId = ObjectId(1);

/// Where [`colonists`]' Drone 3 deploys: the Beacon on tiles (8, 9) to (9, 10), in the Drone's sight.
const SITE: TilePos = TilePos::new(8, 9);

/// The waits that began so far (`SimEvent::ProductionWaiting`): tick, producer, the cap it waits for.
fn waits(t: &TestSim) -> Vec<(Tick, ObjectId, ArmyResource)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match *e {
            SimEvent::ProductionWaiting { producer, reason } => Some((*tick, producer, reason)),
            _ => None,
        })
        .collect()
}

/// Producer `id`'s unit at the front of its queue: how many frames of it are done, and the cap it waits for.
fn front(sim: &Sim, id: ObjectId) -> (u32, Option<ArmyResource>) {
    let status = sim.production_status(id).expect("a unit is queued");
    (status.progress >> 16, status.blocked)
}

/// Player 1's army cap: used, available.
fn capacity(t: &TestSim) -> (u32, u32) {
    let cap = t.sim().army_cap(P1).expect("a faction with a cap");
    (cap.used, cap.available)
}

/// The units produced so far: tick, producer.
fn made(t: &TestSim) -> Vec<(Tick, u32)> {
    produced(t).iter().map(|&(tick, by, _)| (tick, by)).collect()
}

/// Player 1 of the GDO near the Unit Control cap: [`base`]'s Barracks (2) and two more, 3 on tiles (8, 8) to (10, 9)
/// and 4 on (8, 12) to (10, 13), with three Generators (5 to 7) so that all three work at full power; `pks`
/// Peacekeepers (from 8) in rows of 20 well east of the doorways; and 1000 Space Crystals from tick 1.
fn three_barracks_and(pks: usize) -> TestSim {
    let mut builder = base()
        .spawn(ObjectKind::Barracks, Some(P1), tile(8, 8))
        .spawn(ObjectKind::Barracks, Some(P1), tile(8, 12))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 8))
        .spawn(ObjectKind::Generator, Some(P1), tile(1, 11))
        .spawn(ObjectKind::Generator, Some(P1), tile(4, 8));
    for i in 0..pks {
        let (col, row) = ((i % 20) as f32, (i / 20) as f32);
        builder = builder.spawn(PK, Some(P1), at(1000.0 + 30.0 * col, 80.0 + 30.0 * row));
    }
    let mut t = builder.start();
    assert!(!t.sim().power(P1).is_negative(), "full power");
    t.debug_at(1, crystals(P1, 1000));
    t
}

/// [`base`]'s Barracks (2) at the cap: 200 Peacekeepers (3 to 202), and 1000 Space Crystals from tick 1.
fn barracks_at_the_cap() -> TestSim {
    let mut builder = base();
    for i in 0..200 {
        let (col, row) = ((i % 20) as f32, (i / 20) as f32);
        builder = builder.spawn(PK, Some(P1), at(1000.0 + 30.0 * col, 80.0 + 30.0 * row));
    }
    let mut t = builder.start();
    assert_eq!(t.sim().unit_control(P1), UNIT_CONTROL_CAP);
    t.debug_at(1, crystals(P1, 1000));
    t
}

/// Player 1 of the Colonists on a 30 × 20 map: the Robo Portal (1) on tiles (12, 2) to (15, 5), `second` (2), a
/// Colonists structure that provides 5 more Beacon Capacity, from tile (2, 2), and `drones` Drones: 3 at the centre
/// of tile (6, 10), in sight of [`SITE`], and the others (from 4) along tile row 18 from column 14. 1000 Space
/// Crystals from tick 1.
fn colonists(second: ObjectKind, drones: i32) -> TestSim {
    let mut builder = ScenarioBuilder::new(&flat(30, 20))
        .faction(P1, Faction::Colonists)
        .spawn(ObjectKind::RoboPortal, Some(P1), tile(12, 2))
        .spawn(second, Some(P1), tile(2, 2))
        .spawn(DRONE, Some(P1), SpawnAt::Point(TilePos::new(6, 10).center()));
    for x in 14..13 + drones {
        builder = builder.spawn(DRONE, Some(P1), SpawnAt::Point(TilePos::new(x, 18).center()));
    }
    let mut t = builder.start();
    assert_eq!(capacity(&t), (drones as u32, 10));
    t.debug_at(1, crystals(P1, 1000));
    t
}

/// Queues `n` Drones at `portal` in `tick`.
fn train_drones(t: &mut TestSim, tick: Tick, portal: ObjectId, n: usize) {
    for _ in 0..n {
        t.at(tick, P1, production(&[portal], ProductionCommand::Train(DRONE)));
    }
}

/// Orders player 1's Drone `drone` in the next tick to deploy at `origin` (the gate passing it), and steps until it
/// began: from its first frame it is a Beacon (`factions/colonists/economy.md > Beacon Deployment`). Returns the tick
/// it began.
fn deploy(t: &mut TestSim, drone: ObjectId, origin: TilePos) -> Tick {
    let command = Command::Order {
        units: vec![drone],
        kind: OrderKind::Deploy { origin },
        queue: false,
    };
    assert_eq!(validate(t.sim(), P1, command.clone()), Ok(()));
    let ordered = t.sim().tick() + 1;
    t.at(ordered, P1, command);
    while t.sim().object(drone).unwrap().kind == DRONE || t.sim().tick() < ordered {
        assert!(t.sim().tick() < ordered + 300, "Drone {} deploys", drone.0);
        t.step();
    }
    t.sim().tick()
}

/// Design change 0143's first Verification: "GDO near the 200 cap: queue more Peacekeepers than fit — all queue
/// (crystals paid); those that fit start, the next one waits at 0% with a notification; kill a unit and it starts."
/// At 198 of 200, two of three Barracks start and the third waits, notified once; the HUD's figure counts the two in
/// production (it "includes in-production units, not merely queued ones"). A unit removed (a debug removal stands for
/// a kill) makes room, and the waiting Barracks starts in that tick. When two wait for one room, the lower id takes
/// it: the step serves the producers in id order.
#[test]
fn peacekeepers_past_the_cap_all_queue_and_the_last_waits_at_the_front() {
    let mut t = three_barracks_and(198);
    let (b2, b3, b4) = (BARRACKS, ObjectId(3), ObjectId(4));
    let all = [b2, b3, b4];
    t.at(2, P1, train(&all)).at(2, P1, train(&all));
    t.run_until(2);
    assert_eq!(
        (all.map(|b| queue(t.sim(), b)), bank(t.sim(), P1)),
        ([2, 2, 2], 700),
        "all queue, paid"
    );
    assert!(
        !t.events()
            .iter()
            .any(|(_, e)| matches!(e, SimEvent::ProductionRefused { .. }))
    );
    assert_eq!(all.map(|b| front(t.sim(), b)), [(1, None), (1, None), (0, Some(UC))]);
    assert_eq!(t.sim().unit_control(P1), UNIT_CONTROL_CAP, "198 alive, 2 in production");
    assert_eq!(waits(&t), [(2, b4, UC)]);
    t.run_until(30);
    assert_eq!(front(t.sim(), b4), (0, Some(UC)), "it waits");
    assert_eq!(waits(&t).len(), 1, "notified once");

    t.debug_at(31, DebugCommand::Remove { id: ObjectId(8) });
    t.run_until(31);
    assert_eq!(front(t.sim(), b4), (1, None), "it starts in the tick there is room");
    assert_eq!(t.sim().unit_control(P1), UNIT_CONTROL_CAP);

    // The first two come out after their 80 frames, and the units behind them wait in the next tick.
    t.run_until(82);
    assert_eq!(made(&t), [(81, 2), (81, 3)]);
    assert_eq!(&waits(&t)[1..], [(82, b2, UC), (82, b3, UC)]);
    t.debug_at(90, DebugCommand::Remove { id: ObjectId(9) });
    t.run_until(90);
    assert_eq!(
        [b2, b3].map(|b| front(t.sim(), b)),
        [(1, None), (0, Some(UC))],
        "one room: the lower id"
    );
    assert_eq!(t.sim().unit_control(P1), UNIT_CONTROL_CAP);
}

/// Design change 0143's third Verification: "Destroy a Beacon while a Drone is at 60%: it finishes (over capacity
/// shown red); the next one waits." At 9 of 10 a Drone starts (10/10); at 96 of its 160 frames the Beacon is removed
/// (a debug removal for the destruction): 10 used of 5, and the Drone comes out all the same (Q0041 point 2); the
/// one behind it waits at 0%.
#[test]
fn a_drone_in_production_finishes_when_the_beacon_is_destroyed() {
    let mut t = colonists(ObjectKind::Beacon, 9);
    train_drones(&mut t, 2, PORTAL, 2);
    t.run_until(2);
    assert_eq!((front(t.sim(), PORTAL), capacity(&t)), ((1, None), (10, 10)));
    t.run_until(97);
    assert_eq!(front(t.sim(), PORTAL), (96, None), "60% of 160 frames");
    t.debug_at(98, DebugCommand::Remove { id: ObjectId(2) });
    t.run_until(98);
    assert_eq!(capacity(&t), (10, 5), "over capacity");
    t.run_until(161);
    assert_eq!(made(&t), [(161, 1)], "it finishes");
    t.run_until(162);
    assert_eq!((front(t.sim(), PORTAL), capacity(&t)), ((0, Some(BC)), (10, 5)));
    assert_eq!(waits(&t), [(162, PORTAL, BC)]);
    t.run_until(400);
    assert_eq!(
        (front(t.sim(), PORTAL), made(&t).len()),
        ((0, Some(BC)), 1),
        "and waits on"
    );
}

/// Design change 0143's second Verification: "Colonists at 10/10: queue Drones freely; the portal waits; deploy a
/// Beacon → production resumes." Two Robo Portals at 10/10 take two Drones each, paid, and both wait. Drone 3
/// deploys: from its first frame it is a Beacon, which costs nothing (9/10, design change 0112), and the first portal
/// starts a Drone in that tick (10/10; one room, the lower id); the second waits on through the deployment, the
/// Beacon providing nothing until it completes (`factions/colonists/economy.md > Beacon Deployment > Beacon Capacity
/// during deployment`), and starts in the tick its 5 comes.
#[test]
fn colonists_at_capacity_queue_drones_freely_and_resume_with_a_beacon() {
    let mut t = colonists(ObjectKind::RoboPortal, 10);
    let (p1, p2) = (PORTAL, ObjectId(2));
    train_drones(&mut t, 2, p1, 2);
    train_drones(&mut t, 2, p2, 2);
    t.run_until(2);
    assert_eq!(([p1, p2].map(|p| queue(t.sim(), p)), bank(t.sim(), P1)), ([2, 2], 800));
    assert_eq!([p1, p2].map(|p| front(t.sim(), p)), [(0, Some(BC)); 2]);
    assert_eq!(waits(&t), [(2, p1, BC), (2, p2, BC)]);
    let began = deploy(&mut t, ObjectId(3), SITE);
    assert_eq!(capacity(&t), (10, 10), "9 and the Drone that started");
    assert_eq!([p1, p2].map(|p| front(t.sim(), p)), [(1, None), (0, Some(BC))]);
    while t.sim().object(ObjectId(3)).unwrap().is_deploying() {
        assert!(t.sim().tick() < began + 100, "the deployment completes");
        assert_eq!((front(t.sim(), p2), capacity(&t).1), ((0, Some(BC)), 10));
        t.step();
    }
    assert_eq!(capacity(&t), (11, 15));
    assert_eq!(
        front(t.sim(), p2),
        (1, None),
        "it starts in the tick the Beacon completed"
    );
    assert_eq!(waits(&t).len(), 2, "no new wait");
}

/// Design change 0112's Verification as 0143 amends it ("0112's example now reads 'a new Drone starts production'"),
/// at 10/10 with Drones queued at the Robo Portal, waiting: Drone 3 deploys (9/10), and a new Drone starts production
/// (10/10); the deployment is cancelled (`factions/colonists/economy.md > Beacon Deployment > Cancel Deployment`) and
/// the Drone takes its 1 back: 11/10, over capacity. The Drone in production finishes, and the next waits until a
/// Beacon completes: through Drone 3's second deployment (10/10, the deploying Beacon providing nothing), and in the
/// tick that completes, it starts.
#[test]
fn a_cancelled_deployment_holds_the_portal_until_a_beacon_completes() {
    let mut t = colonists(ObjectKind::EgressNode, 10);
    train_drones(&mut t, 2, PORTAL, 2);
    t.run_until(2);
    assert_eq!((front(t.sim(), PORTAL), capacity(&t)), ((0, Some(BC)), (10, 10)));
    let began = deploy(&mut t, ObjectId(3), SITE);
    assert_eq!(
        (front(t.sim(), PORTAL), capacity(&t)),
        ((1, None), (10, 10)),
        "deployed: 9/10, and a new Drone starts production: 10/10"
    );
    let cancel = Command::Construction {
        objects: vec![ObjectId(3)],
        command: ConstructionCommand::CancelTransform,
    };
    assert_eq!(validate(t.sim(), P1, cancel.clone()), Ok(()));
    t.at(began + 5, P1, cancel);
    t.run_until(began + 5);
    assert_eq!(t.sim().object(ObjectId(3)).unwrap().kind, DRONE);
    assert_eq!(capacity(&t), (11, 10), "over capacity");
    t.run_until(began + 160);
    assert_eq!(made(&t), [(began + 159, 1)], "the Drone in production finishes");
    assert_eq!((front(t.sim(), PORTAL), capacity(&t)), ((0, Some(BC)), (11, 10)));
    assert_eq!(waits(&t), [(2, PORTAL, BC), (began + 160, PORTAL, BC)]);
    let again = deploy(&mut t, ObjectId(3), SITE);
    assert_eq!(capacity(&t), (10, 10));
    while t.sim().object(ObjectId(3)).unwrap().is_deploying() {
        assert!(t.sim().tick() < again + 100, "the deployment completes");
        assert_eq!(front(t.sim(), PORTAL), (0, Some(BC)), "it waits through the deployment");
        t.step();
    }
    assert_eq!((front(t.sim(), PORTAL), capacity(&t)), ((1, None), (11, 15)));
}

/// Slice M3-D's plan acceptance "over capacity", design change 0112 as 0143 amends it: the Egress Node removed (a
/// debug removal for its destruction), the portal is over capacity, and finishes its Drone in production all the
/// same (Q0041 point 2); the one queued behind it waits at 0% (`> Over capacity`: "no new unit starts production
/// until there is room again"), and a Train is still taken. Once Drones are gone and there is room, the portal starts
/// by itself, in that tick.
#[test]
fn an_over_capacity_portal_finishes_its_drone_and_waits_for_room() {
    let mut t = colonists(ObjectKind::EgressNode, 7);
    train_drones(&mut t, 2, PORTAL, 2);
    t.debug_at(3, DebugCommand::Remove { id: ObjectId(2) });
    t.run_until(3);
    assert_eq!(
        (front(t.sim(), PORTAL), capacity(&t)),
        ((2, None), (8, 5)),
        "over capacity: seven Drones and one in production"
    );
    let train_one = production(&[PORTAL], ProductionCommand::Train(DRONE));
    assert_eq!(validate(t.sim(), P1, train_one), Ok(()), "queuing doesn't check it");
    t.run_until(400);
    assert_eq!(made(&t), [(161, 1)], "the Drone in production finishes");
    assert_eq!(
        (front(t.sim(), PORTAL), capacity(&t)),
        ((0, Some(BC)), (8, 5)),
        "the next waits"
    );
    assert_eq!(waits(&t), [(162, PORTAL, BC)]);
    // Four Drones gone: 4 used of 5, and the portal starts at once.
    for id in 4..8 {
        t.debug_at(401, DebugCommand::Remove { id: ObjectId(id) });
    }
    t.run_until(401);
    assert_eq!((front(t.sim(), PORTAL), capacity(&t)), ((1, None), (5, 5)));
    t.run_until(560);
    assert_eq!(made(&t), [(161, 1), (560, 1)], "trained");
    assert_eq!(capacity(&t), (5, 5));
}

/// Design changes 0084 and 0085 under 0143: a destroyed producer refunds its whole queue in full, whatever it was
/// doing; of the army cap it frees only what its unit in production took ("Unit Control released for the unit in
/// production", `factions/gdo/structures/barracks.md > Destroyed while producing`; the queued ones never took
/// any). The waiting Barracks frees nothing; the producing one its one Peacekeeper. Debug removals for the
/// destructions.
#[test]
fn a_destroyed_producer_refunds_its_queue_and_frees_only_its_unit_in_production() {
    let mut t = three_barracks_and(198);
    let (b2, b3, b4) = (BARRACKS, ObjectId(3), ObjectId(4));
    let all = [b2, b3, b4];
    t.at(2, P1, train(&all)).at(2, P1, train(&all));
    t.debug_at(10, DebugCommand::Remove { id: b4 });
    t.debug_at(20, DebugCommand::Remove { id: b3 });
    t.run_until(9);
    assert_eq!(front(t.sim(), b4), (0, Some(UC)));
    assert_eq!((bank(t.sim(), P1), t.sim().unit_control(P1)), (700, 200));
    t.run_until(10);
    assert_eq!(
        (bank(t.sim(), P1), t.sim().unit_control(P1)),
        (800, 200),
        "the waiting one: its two refunded, nothing freed"
    );
    t.run_until(20);
    assert_eq!(
        (bank(t.sim(), P1), t.sim().unit_control(P1)),
        (900, 199),
        "the producing one: its two refunded, its unit in production freed"
    );
    let refunds: Vec<(Tick, ObjectId)> = t
        .events()
        .iter()
        .filter_map(|(tick, e)| match *e {
            SimEvent::ProductionCancelled {
                producer, refund: 50, ..
            } => Some((*tick, producer)),
            _ => None,
        })
        .collect();
    assert_eq!(refunds, [(10, b4), (10, b4), (20, b3), (20, b3)]);
    assert_eq!(dangling_references(t.sim()), []);
}

/// Design change 0143: "the player can still cancel it for a full refund". A Peacekeeper waiting at the cap is
/// cancelled: refunded in full, and the wait ends with it. Queued again it waits again, a new wait with a new
/// notification; a cancel of the unit queued behind it leaves it waiting.
#[test]
fn a_waiting_unit_can_be_cancelled_for_a_full_refund() {
    let mut t = barracks_at_the_cap();
    let cancel = production(&[BARRACKS], ProductionCommand::CancelProduction);
    t.at(2, P1, train(&[BARRACKS]));
    t.at(5, P1, cancel.clone());
    t.run_until(4);
    assert_eq!((front(t.sim(), BARRACKS), bank(t.sim(), P1)), ((0, Some(UC)), 950));
    t.run_until(5);
    assert_eq!((t.sim().production_status(BARRACKS), bank(t.sim(), P1)), (None, 1000));
    assert!(!t.sim().object(BARRACKS).unwrap().production.as_ref().unwrap().blocked);
    t.at(8, P1, train(&[BARRACKS])).at(8, P1, train(&[BARRACKS]));
    t.at(9, P1, cancel);
    t.run_until(9);
    assert_eq!((queue(t.sim(), BARRACKS), bank(t.sim(), P1)), (1, 950));
    assert_eq!(front(t.sim(), BARRACKS), (0, Some(UC)));
    assert_eq!(waits(&t), [(2, BARRACKS, UC), (8, BARRACKS, UC)]);
    t.debug_at(12, DebugCommand::Remove { id: ObjectId(3) });
    t.run_until(12);
    assert_eq!(front(t.sim(), BARRACKS), (1, None));
}

/// The start check is made when the building would begin progress (design change 0143): a Barracks with nothing
/// generated works at a power ratio of 0 (design change 0089), so its Peacekeeper doesn't start, takes no Unit
/// Control and doesn't wait for any, though there is room for one. With a Generator it starts.
#[test]
fn a_barracks_without_power_starts_nothing_and_takes_nothing() {
    let mut builder = ScenarioBuilder::new(&flat(30, 16)).spawn(ObjectKind::Barracks, Some(P1), tile(8, 4));
    for i in 0..199 {
        let (col, row) = ((i % 20) as f32, (i / 20) as f32);
        builder = builder.spawn(PK, Some(P1), at(1000.0 + 30.0 * col, 80.0 + 30.0 * row));
    }
    let mut t = builder.start();
    let barracks = ObjectId(1);
    t.debug_at(1, crystals(P1, 1000));
    t.at(2, P1, train(&[barracks]));
    t.run_until(50);
    assert_eq!(t.sim().power(P1).ratio(), 0);
    assert_eq!((front(t.sim(), barracks), t.sim().unit_control(P1)), ((0, None), 199));
    assert_eq!(waits(&t), []);
    let generator = DebugCommand::Spawn {
        kind: ObjectKind::Generator,
        owner: Some(P1),
        at: tile(1, 1),
        orientation: Default::default(),
    };
    t.debug_at(51, generator);
    t.run_until(51);
    let status = t.sim().production_status(barracks).unwrap();
    assert!(status.progress > 0, "it starts, at the ratio of 20/30");
    assert_eq!(t.sim().unit_control(P1), UNIT_CONTROL_CAP);
}

/// Determinism: a game with producers that wait replays to the same hashes, and a save made while one waits holds the
/// wait and goes on like the game itself.
#[test]
fn a_game_with_waiting_producers_replays_and_saves() {
    let mut t = three_barracks_and(198);
    let all = [BARRACKS, ObjectId(3), ObjectId(4)];
    t.at(2, P1, train(&all)).at(2, P1, train(&all));
    t.debug_at(31, DebugCommand::Remove { id: ObjectId(8) });
    t.debug_at(90, DebugCommand::Remove { id: ObjectId(9) });
    t.run_until(20);
    let json = serde_json::to_string(t.sim()).unwrap();
    assert_eq!(json.matches("\"blocked\":true").count(), 1, "the waiting Barracks");
    let saved: Sim = serde_json::from_str(&json).unwrap();
    assert_eq!(&saved, t.sim());
    assert_eq!(saved.state_hash(), t.sim().state_hash());
    t.run_until(200);
    let (b2, b3, b4) = (BARRACKS, ObjectId(3), ObjectId(4));
    assert_eq!(
        waits(&t),
        [(2, b4, UC), (82, b2, UC), (82, b3, UC), (111, b4, UC)],
        "the first, as in the Verification, then B4's second after its first came out in tick 110"
    );
    let replayed = t.replay().run();
    assert_eq!(replayed.state_hash(), t.sim().state_hash());
    assert_eq!(&replayed, t.sim());
    let mut resumed = saved;
    for inputs in &t.replay().inputs[20..] {
        resumed.step(inputs);
    }
    assert_eq!(resumed.state_hash(), t.sim().state_hash());
}
