//! Gathering Drones pass through each other (slice M3-H2; design change 0106, `factions/colonists/units.md > Drone >
//! Gathering Drones pass through each other`, `systems/units.md > UnitCollision > Ground Collision > Exception,
//! gathering Drones` and `> Units that already overlap`): 0106's Verification, point by point. The saturated row is
//! `gathering_soak.rs`. "Recruits mining a patch still bump into each other" has no test: the Cults aren't built,
//! and the Drone is the only worker (`WorkerDef::pass_through` is the switch a Recruit won't have).
//!
//! `TestSim` checks invariant I1 after every tick: only two gathering Drones, or two units that already overlapped a
//! tick earlier, may overlap. So every test here also shows that nothing else ever does.

use sc_sim::orders::state::{Behaviour, GatherPhase};
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, SimEvent, SpawnAt, Target, Tick, TilePos, Vec2};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const DRONE: ObjectKind = ObjectKind::Drone;
const NODE: ObjectKind = ObjectKind::EgressNode;
const PATCH: ObjectKind = ObjectKind::CrystalPatch;
/// Half the Drone's Silhouette, 36 × 36 space units.
const HALF: f32 = 18.0;
/// The Drone's speed on the stand-in mover, space units per tick.
const SPEED: f32 = 8.0;

fn flat(w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&vec![".".repeat(w); h].join("\n"))
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn at(x: f32, y: f32) -> SpawnAt {
    SpawnAt::Point(Vec2::new(x, y))
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn gather(t: &mut TestSim, tick: Tick, player: PlayerId, drones: &[u32], patch: u32) {
    let units: Vec<ObjectId> = drones.iter().map(|&n| id(n)).collect();
    t.order_at(tick, player, &units, OrderKind::Gather(id(patch)), false);
}

fn move_to(t: &mut TestSim, tick: Tick, player: PlayerId, drone: u32, to: Vec2) {
    t.order_at(tick, player, &[id(drone)], OrderKind::Move(Target::Point(to)), false);
}

fn phase(t: &TestSim, n: u32) -> Option<GatherPhase> {
    let orders = t.sim().object(id(n)).expect("exists").orders.as_ref().expect("a unit");
    match orders.current.as_ref()?.state {
        Behaviour::Gather(gather) => Some(gather.phase),
        _ => None,
    }
}

fn is_mining(t: &TestSim, n: u32) -> bool {
    matches!(phase(t, n), Some(GatherPhase::Mining { .. }))
}

/// Whether the boxes of the Drones `a` and `b` overlap: strictly, as invariant I1 has it.
fn overlap(t: &TestSim, a: u32, b: u32) -> bool {
    let d = (t.pos(a) - t.pos(b)).abs();
    d.x < 2.0 * HALF && d.y < 2.0 * HALF
}

/// The loads delivered so far by each of `drones`.
fn deliveries(t: &TestSim, drones: &[u32]) -> Vec<u32> {
    let count = |n: u32| {
        let by = |e: &SimEvent| matches!(e, SimEvent::ResourcesDelivered { worker, .. } if *worker == id(n));
        t.events().iter().filter(|(_, e)| by(e)).count() as u32
    };
    drones.iter().map(|&n| count(n)).collect()
}

/// 0106's Verification: "Two players' Drones mining the same row pass through each other." Two patches with an
/// Egress Node of each player south of them, and three Drones of each player on the two patches: six Drones for two
/// patches, so they wait at them together and cross on every trip. Drones of the two players stand inside each
/// other, the row is mined without a break (occupancy is shared: 8 Space Crystals per second for both together),
/// and every Drone of both players keeps delivering.
#[test]
fn two_players_drones_mining_the_same_row_pass_through_each_other() {
    // Ids: patches 1 and 2, the nodes 3 (player 1) and 4 (player 2), Drones 5 to 7 (player 1) and 8 to 10.
    let mut scenario = flat(16, 12)
        .spawn(PATCH, None, tile(7, 2))
        .spawn(PATCH, None, tile(8, 2))
        .spawn(NODE, Some(P1), tile(4, 5))
        .spawn(NODE, Some(P2), tile(10, 5));
    for (x, y) in [(420.0, 240.0), (464.0, 240.0), (420.0, 290.0)] {
        scenario = scenario.spawn(DRONE, Some(P1), at(x, y));
    }
    for (x, y) in [(604.0, 240.0), (560.0, 240.0), (604.0, 290.0)] {
        scenario = scenario.spawn(DRONE, Some(P2), at(x, y));
    }
    let mut t = scenario.start();
    // Each player's Drones to the patch on the other player's side, so that their ways cross.
    gather(&mut t, 1, P1, &[5, 6, 7], 2);
    gather(&mut t, 1, P2, &[8, 9, 10], 1);
    let (mut enemies_overlapped, mut own_overlapped) = (0, 0);
    for _ in 0..1920 {
        t.step();
        for a in 5..=7 {
            enemies_overlapped += (8..=10).filter(|&b| overlap(&t, a, b)).count();
            own_overlapped += (a + 1..=7).filter(|&b| overlap(&t, a, b)).count();
        }
    }
    assert!(
        enemies_overlapped > 100,
        "Drones of the two players stood inside each other: {enemies_overlapped}"
    );
    assert!(own_overlapped > 100, "and a player's own: {own_overlapped}");
    let loads = deliveries(&t, &[5, 6, 7, 8, 9, 10]);
    assert!(
        loads.iter().all(|&n| n >= 12),
        "every Drone keeps delivering: {loads:?}"
    );
    // 2 minutes of 2 patches, 8 Space Crystals per second, less the first walk and the loads on their way.
    let banked = |player| t.sim().player(player).expect("in the game").crystals;
    let (first, second) = (banked(P1), banked(P2));
    assert!(first + second >= 880, "{first} + {second}");
    assert!(first >= 300 && second >= 300, "{first} and {second}");
}

/// A patch at tile (5, 2), an Egress Node of player 1 two grid units south of it, a Drone of player 1 in the strip
/// between them, on the straight way between the two, and a Drone of `owner`'s parked in the strip at `parked`. Ids:
/// patch 1, node 2, the gatherer 3, the parked Drone 4.
fn strip_with_a_parked_drone(owner: PlayerId, parked: Vec2) -> TestSim {
    flat(12, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), at(352.0, 298.0))
        .spawn(DRONE, Some(owner), SpawnAt::Point(parked))
        .start()
}

/// 0106's Verification: "An idle Drone parked in the strip: gathering Drones path around it / bump into it." An
/// enemy's idle Drone never yields, so it stands where it is for good, half across the gatherer's way. The gatherer
/// never enters its box (the test looks itself, and so does I1 after every tick), goes round it on every trip, and
/// keeps delivering: 14 loads a minute, where the free strip gives 16.
#[test]
fn gathering_drones_go_around_an_idle_drone_parked_in_the_strip() {
    let parked = Vec2::new(336.0, 256.0);
    let mut t = strip_with_a_parked_drone(P2, parked);
    gather(&mut t, 1, P1, &[3], 1);
    let mut widest: f32 = 0.0;
    for tick in 1..=960 {
        t.step();
        assert!(!overlap(&t, 3, 4), "tick {tick}\n{}", t.ascii(Some(parked)));
        assert_eq!(t.pos(4), parked, "tick {tick}: nothing moves the parked Drone");
        // Beside the parked Drone: how far the gatherer is from it.
        if (t.pos(3).y - parked.y).abs() < HALF {
            widest = widest.max(t.pos(3).x - parked.x);
        }
    }
    assert!(widest >= 2.0 * HALF, "it passed beside the parked Drone: {widest}");
    let loads = deliveries(&t, &[3]);
    assert!(loads[0] >= 12, "it keeps delivering on the longer way: {loads:?}");
}

/// The same with the enemy's Drone parked dead on the gatherer's line: "bump into it". The gatherer never enters
/// its box and keeps its job, but it doesn't get round for a long time: its target is so close behind the Drone
/// that each step aside turns its lean back the other way (`docs/movement.md > 6.3`, rule 4), until it counts as
/// arrived behind a unit that won't move and tries again (`orders::gather`, a leg that ends short). **This pins the
/// collision, not the steering**: no delivery is asserted, and the steering is slice M3-G's (reported there).
#[test]
fn a_gathering_drone_bumps_into_an_idle_drone_parked_dead_in_its_way() {
    let parked = Vec2::new(352.0, 256.0);
    let mut t = strip_with_a_parked_drone(P2, parked);
    gather(&mut t, 1, P1, &[3], 1);
    for tick in 1..=960 {
        t.step();
        assert!(!overlap(&t, 3, 4), "tick {tick}\n{}", t.ascii(Some(parked)));
        assert_eq!(t.pos(4), parked, "tick {tick}: nothing moves the parked Drone");
        assert!(phase(&t, 3).is_some(), "tick {tick}: a Gather never ends on its own");
    }
}

/// The same with an idle Drone of the gatherer's own player, which does yield (`systems/units.md > UnitCollision >
/// Ground Collision > Friendly yielding`; 0106: "Yielding rules unchanged"): the gatherer bumps into it, it steps
/// aside, and the two never overlap. A Drone without a Gather job is solid for one that has one.
#[test]
fn an_idle_drone_of_the_gatherers_owner_steps_aside_and_is_never_entered() {
    let parked = Vec2::new(352.0, 256.0);
    let mut t = strip_with_a_parked_drone(P1, parked);
    gather(&mut t, 1, P1, &[3], 1);
    for tick in 1..=960 {
        t.step();
        assert!(!overlap(&t, 3, 4), "tick {tick}\n{}", t.ascii(Some(parked)));
    }
    assert_ne!(t.pos(4), parked, "it made way");
    let loads = deliveries(&t, &[3]);
    assert!(loads[0] >= 12, "{loads:?}");
}

/// 0106's Verification: "Select a Drone overlapping another mining Drone and order Move: it drives out without
/// being blocked or shoving the other; once apart, they collide normally" (`systems/units.md > UnitCollision >
/// Units that already overlap`). Two Drones on one patch: the first mines, the second waits inside it.
/// - Ordered to Move, the waiting Drone is no gathering Drone any more and overlaps one: the two ignore each other
///   until they have separated. It turns east and leaves at full speed, and the mining Drone neither moves nor
///   stops mining.
/// - Apart, they collide: sent back through the mining Drone, it goes round it and never enters its box again (I1
///   would fail too: the pair didn't overlap the tick before).
#[test]
fn a_drone_ordered_out_of_a_mining_drone_drives_out_and_collides_once_apart() {
    // Ids: patch 1, node 2, Drones 3 and 4.
    let mut t = flat(14, 10)
        .spawn(PATCH, None, tile(6, 2))
        .spawn(NODE, Some(P1), tile(6, 5))
        .spawn(DRONE, Some(P1), at(416.0, 250.0))
        .spawn(DRONE, Some(P1), at(420.0, 294.0))
        .start();
    gather(&mut t, 1, P1, &[3, 4], 1);
    for _ in 0..12 {
        t.step();
    }
    assert!(is_mining(&t, 3), "{:?}", phase(&t, 3));
    assert!(
        matches!(phase(&t, 4), Some(GatherPhase::Waiting { .. })),
        "{:?}",
        phase(&t, 4)
    );
    let (miner, waiter) = (t.pos(3), t.pos(4));
    assert!(
        (miner - waiter).abs().max_element() < HALF,
        "one inside the other: {miner} and {waiter}"
    );

    // Out to the east.
    let east = waiter + Vec2::new(160.0, 0.0);
    move_to(&mut t, 13, P1, 4, east);
    let (mut left_at, mut apart_at) = (None, None);
    for tick in 13..=22 {
        let before = t.pos(4);
        t.step();
        let step = t.pos(4) - before;
        // It turns on the spot first (the order's tick, and its TurnRate), then never stands again.
        if left_at.is_some() || step != Vec2::ZERO {
            assert_eq!(step, Vec2::new(SPEED, 0.0), "tick {tick}: at full speed, straight out");
            left_at.get_or_insert(tick);
        }
        assert_eq!(t.pos(3), miner, "tick {tick}: the mining Drone isn't shoved");
        assert!(is_mining(&t, 3), "tick {tick}");
        if apart_at.is_none() && !overlap(&t, 3, 4) {
            apart_at = Some(tick);
        }
    }
    let left_at = left_at.expect("it left");
    assert!(left_at <= 15, "{left_at}");
    // 36 space units less the 4 they started apart, at 8 a tick: apart with its fourth step.
    assert_eq!(apart_at, Some(left_at + 3));

    // Back west, through where the mining Drone stands: it is solid now.
    let west = Vec2::new(miner.x - 160.0, miner.y);
    move_to(&mut t, 23, P1, 4, west);
    let mut beside: f32 = 0.0;
    for tick in 23..=44 {
        t.step();
        assert!(!overlap(&t, 3, 4), "tick {tick}\n{}", t.ascii(Some(miner)));
        if is_mining(&t, 3) {
            assert_eq!(t.pos(3), miner, "tick {tick}");
        }
        if (t.pos(4).x - miner.x).abs() < HALF {
            beside = beside.max((t.pos(4).y - miner.y).abs());
        }
    }
    assert!(beside >= 2.0 * HALF, "it went round the mining Drone: {beside}");
    for _ in 0..40 {
        t.step();
    }
    assert_eq!((t.status(4), t.pos(4)), (MoveStatus::Arrived, west));
}

/// `> Units that already overlap`: "Nothing is pushed apart." A Drone stopped inside a mining Drone stays where it
/// is, and the two go on ignoring each other for as long as they overlap: nothing moves either while the mining
/// lasts, and then the miner leaves for the node through the Drone that stands in its way, unhindered. Once the two
/// have been apart they collide like any two units (the stopped Drone, idle and its owner's, now steps aside for
/// it), and they never overlap again.
#[test]
fn a_drone_stopped_inside_another_is_not_pushed_apart() {
    let mut t = flat(14, 10)
        .spawn(PATCH, None, tile(6, 2))
        .spawn(NODE, Some(P1), tile(6, 5))
        .spawn(DRONE, Some(P1), at(416.0, 250.0))
        .spawn(DRONE, Some(P1), at(420.0, 294.0))
        .start();
    gather(&mut t, 1, P1, &[3, 4], 1);
    for _ in 0..12 {
        t.step();
    }
    let (miner, waiter) = (t.pos(3), t.pos(4));
    assert!(is_mining(&t, 3) && overlap(&t, 3, 4), "{miner} and {waiter}");
    t.order_at(13, P1, &[id(4)], OrderKind::Stop, false);
    // The rest of the mining: both stand, overlapping.
    while is_mining(&t, 3) {
        t.step();
        assert_eq!((t.pos(3), t.pos(4)), (miner, waiter), "tick {}", t.sim().tick());
    }
    assert!(t.sim().tick() > 30 && phase(&t, 4).is_none());
    // The miner leaves through the stopped Drone: every step it takes while they overlap is a whole one.
    let mut apart = false;
    for _ in 0..600 {
        let before = t.pos(3);
        t.step();
        let tick = t.sim().tick();
        if !apart && overlap(&t, 3, 4) {
            assert_eq!(t.pos(4), waiter, "tick {tick}: not pushed");
            let step = t.pos(3).distance(before);
            assert!(step == 0.0 || step > SPEED - 0.01, "tick {tick}: unhindered, {step}");
        }
        apart |= !overlap(&t, 3, 4);
        assert!(!(apart && overlap(&t, 3, 4)), "tick {tick}\n{}", t.ascii(Some(waiter)));
    }
    assert!(apart);
    let loads = deliveries(&t, &[3]);
    assert!(loads[0] >= 8, "{loads:?}");
}
