//! Gathering (slice M3-H, brought in line with the design answers by slice M3-H3; `factions/colonists/units.md >
//! Drone > Gathering`, `> Patch Occupancy (one Drone per patch)`, `> When a Gather ends`, `> Assigned Drop-off`;
//! `factions/colonists/economy.md > Egress Node > Drop Off`): the loop and its trip time, occupancy, waiting and
//! redirects, Set Drop-off Location, depletion, the Drop Off order, a lost Egress Node, the gate, a rally on a patch,
//! and a game replayed and saved. `TestSim` checks the invariants after every tick. The Verification points of the
//! design answers are `gathering_rules`; the crowd is `gathering_soak.rs`.

use sc_sim::nav::NavClass;
use sc_sim::orders::state::{Behaviour, GatherPhase};
use sc_sim::testing::{ScenarioBuilder, TestSim, dangling_references};
use sc_sim::{
    Ability, Command, DebugCommand, Object, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId,
    ProductionCommand, Reject, Replay, ResourceState, Sim, SimEvent, SpawnAt, SpawnSettings, Target, Tick, TilePos,
    Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const DRONE: ObjectKind = ObjectKind::Drone;
const NODE: ObjectKind = ObjectKind::EgressNode;
const PATCH: ObjectKind = ObjectKind::CrystalPatch;
/// `> Gathering`: CarryCapacity, MiningDuration and DropOffDuration.
const LOAD: u32 = 8;
const MINING: Tick = 32;
const DROP_OFF: Tick = 2;
/// One trip of the reference geometry with DragMovement (`economy/gathering.rs`, module docs).
const TRIP: Tick = 81;

fn flat(w: usize, h: usize) -> ScenarioBuilder {
    ScenarioBuilder::new(&vec![".".repeat(w); h].join("\n"))
}

fn tile(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Tile(TilePos::new(x, y))
}

fn centre(x: i32, y: i32) -> SpawnAt {
    SpawnAt::Point(TilePos::new(x, y).center())
}

fn patch_of(amount: u32) -> SpawnSettings {
    SpawnSettings::CrystalPatch {
        starting_amount: amount,
    }
}

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn object(t: &TestSim, n: u32) -> &Object {
    t.sim().object(id(n)).expect("exists")
}

fn bank(t: &TestSim, player: PlayerId) -> u32 {
    t.sim().player(player).expect("in the game").crystals
}

fn remaining(t: &TestSim, n: u32) -> u32 {
    match object(t, n).resource {
        Some(ResourceState::CrystalPatch { remaining }) => remaining,
        _ => panic!("object {n} is no patch"),
    }
}

/// The order a unit is carrying out.
fn order(t: &TestSim, n: u32) -> Option<OrderKind> {
    let orders = object(t, n).orders.as_ref().expect("a unit");
    orders.current.as_ref().map(|active| active.order.kind)
}

/// The phase of a worker's job, if it has one.
fn phase(t: &TestSim, n: u32) -> Option<GatherPhase> {
    let orders = object(t, n).orders.as_ref().expect("a unit");
    match orders.current.as_ref()?.state {
        Behaviour::Gather(gather) => Some(gather.phase),
        _ => None,
    }
}

fn is_mining(t: &TestSim, n: u32) -> bool {
    matches!(phase(t, n), Some(GatherPhase::Mining { .. }))
}

fn is_waiting(t: &TestSim, n: u32) -> bool {
    matches!(phase(t, n), Some(GatherPhase::Waiting { .. }))
}

fn load(t: &TestSim, n: u32) -> u32 {
    object(t, n).worker.expect("a worker").load
}

fn drop_off(t: &TestSim, n: u32) -> Option<ObjectId> {
    object(t, n).worker.expect("a worker").drop_off
}

fn gather(t: &mut TestSim, tick: Tick, player: PlayerId, drones: &[u32], patch: u32) {
    let units: Vec<ObjectId> = drones.iter().map(|&n| id(n)).collect();
    t.order_at(tick, player, &units, OrderKind::Gather(id(patch)), false);
}

/// The deliveries so far, as `(tick, worker, drop-off, amount)`.
fn deliveries(t: &TestSim) -> Vec<(Tick, u32, u32, u32)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::ResourcesDelivered {
                worker,
                drop_off,
                amount,
            } => Some((*tick, worker.0, drop_off.0, *amount)),
            _ => None,
        })
        .collect()
}

/// Steps until `done` holds, at most `limit` ticks, and returns the tick it first held in.
fn run_to(t: &mut TestSim, limit: Tick, what: &str, done: impl Fn(&TestSim) -> bool) -> Tick {
    for _ in 0..limit {
        t.step();
        if done(t) {
            return t.sim().tick();
        }
    }
    panic!("{what}: not within {limit} ticks\n{}", t.ascii(None));
}

/// The first tick in which worker `n` mines.
fn mines_from(t: &mut TestSim, n: u32) -> Tick {
    run_to(t, 400, "it mines", |t| is_mining(t, n))
}

/// The reference geometry of one trip (design change 0107): a patch at tile (5, 2) and an Egress Node 2 grid
/// units south of it, edge to edge, with a Drone of player 1 between them. Ids: patch 1, node 2, Drone 3.
fn one_patch(amount: u32) -> ScenarioBuilder {
    flat(16, 10)
        .spawn_with(patch_of(amount), tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
}

/// Slice H's first acceptance point: one Drone, the patch 2 grid units from the node. It mines for 32 frames, the
/// patch loses 8 as the mining ends, it unloads for 2 frames, and the bank rises by 8 at the end of each drop-off.
/// The trip time with DragMovement (slice M3-G) is pinned here and recorded in `economy/gathering.rs`: 81 ticks.
#[test]
fn one_drone_carries_loads_of_eight_from_the_patch_to_the_node() {
    let mut t = one_patch(1500).start();
    let start = bank(&t, P1);
    gather(&mut t, 1, P1, &[3], 1);
    // The first leg: from rest to within reach of the patch, 73 su, turning north as it sets off.
    let began = mines_from(&mut t, 3);
    assert_eq!(began, 20);
    assert_eq!(drop_off(&t, 3), Some(id(2)), "auto-assigned: the nearest Egress Node");
    assert_eq!(order(&t, 3), Some(OrderKind::Gather(id(1))));
    // `> Gathering`: MiningDuration. The load is taken as the mining ends, in the economy phase.
    t.run_until(began + MINING - 2);
    assert_eq!((load(&t, 3), remaining(&t, 1)), (0, 1500));
    assert_eq!(phase(&t, 3), Some(GatherPhase::Mining { done: MINING - 1 }));
    t.step();
    assert_eq!((load(&t, 3), remaining(&t, 1)), (LOAD, 1500 - LOAD));
    assert_eq!(phase(&t, 3), Some(GatherPhase::ToDropOff));
    assert_eq!(bank(&t, P1), start, "nothing enters the pool before the drop-off");
    // The way back, then DropOffDuration.
    let unloading = run_to(&mut t, 40, "it unloads", |t| {
        matches!(phase(t, 3), Some(GatherPhase::DroppingOff { .. }))
    });
    assert_eq!(
        unloading,
        began + MINING - 1 + 24,
        "24 ticks from the patch to within reach of the node: thrust, top speed never reached, braking"
    );
    assert_eq!((load(&t, 3), bank(&t, P1)), (LOAD, start));
    t.run(DROP_OFF - 1);
    assert_eq!((load(&t, 3), bank(&t, P1)), (0, start + LOAD));
    assert_eq!(deliveries(&t), [(unloading + 1, 3, 2, LOAD)]);
    // And again, for good: a delivery every 81 ticks.
    t.run_until(unloading + 1 + 5 * TRIP);
    let ticks: Vec<Tick> = deliveries(&t).iter().map(|d| d.0).collect();
    assert_eq!(ticks, [76, 157, 238, 319, 400, 481]);
    assert_eq!(bank(&t, P1), start + 6 * LOAD);
    assert_eq!(
        remaining(&t, 1) + load(&t, 3) + 6 * LOAD,
        1500,
        "nothing lost on the way"
    );
    assert_eq!(
        order(&t, 3),
        Some(OrderKind::Gather(id(1))),
        "a Gather never ends on its own"
    );
    assert_eq!(t.sim().returning_drones(id(2)), [id(3)]);
}

/// `> Patch Occupancy (one Drone per patch)`: two Drones reach a patch in the same tick. The lower id mines; the
/// other "goes to the nearest unoccupied patch to itself instead": the nearer of the two free ones.
#[test]
fn of_two_drones_arriving_together_the_lower_id_mines_and_the_other_goes_to_the_nearest_free_patch() {
    // Patches 1 to 3, node 4, Drone 5 south of patch 1 and Drone 6 north of it, as far away.
    let mut t = flat(16, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(PATCH, None, tile(8, 2))
        .spawn(PATCH, None, tile(2, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(DRONE, Some(P1), SpawnAt::Point(Vec2::new(360.0, 32.0)))
        .start();
    gather(&mut t, 1, P1, &[5, 6], 1);
    let arrived = mines_from(&mut t, 5);
    assert_eq!(order(&t, 5), Some(OrderKind::Gather(id(1))));
    // Drone 6 stands a little east of the patch's middle: patch 2, to the east, is the nearer one.
    assert_eq!(
        order(&t, 6),
        Some(OrderKind::Gather(id(2))),
        "redirected in the tick it arrived"
    );
    assert_eq!(phase(&t, 6), Some(GatherPhase::ToPatch));
    let other = mines_from(&mut t, 6);
    // (By then Drone 5 may be on its way to the node: Drone 6 has to turn its momentum round first.)
    assert!(other > arrived, "both mine, each its own patch");
    assert_eq!(order(&t, 5), Some(OrderKind::Gather(id(1))));
    assert_eq!(remaining(&t, 3), 1500, "nobody went to the farther one");
}

/// `> Patch Occupancy`: "If no patch is unoccupied, it waits at the current patch and mines as soon as it frees
/// up". Design change 0133 with the Director's choice: of the Drones waiting there, the lowest id mines next,
/// whoever came first. Design change 0131: a patch its owner has never seen is no patch to go to, re-check after
/// re-check (0132).
#[test]
fn with_no_patch_free_drones_wait_and_the_lowest_id_mines_next() {
    // Patch 1; patch 2, 6 grid units east of it, out of everyone's sight; node 3. Drone 4 comes from the west and
    // arrives last; Drones 5 (south) and 6 (north) arrive together.
    let mut t = flat(16, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(PATCH, None, tile(11, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(1, 2))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(DRONE, Some(P1), centre(5, 0))
        .start();
    assert!(!t.sim().is_visible(P1, object(&t, 2)), "the far patch is in the fog");
    gather(&mut t, 1, P1, &[4, 5, 6], 1);
    let first = mines_from(&mut t, 5);
    assert_eq!(phase(&t, 6), Some(GatherPhase::Waiting { since: first }));
    assert_eq!(
        order(&t, 6),
        Some(OrderKind::Gather(id(1))),
        "it stays: it knows no other patch"
    );
    let third = run_to(&mut t, 30, "Drone 4 waits too", |t| is_waiting(t, 4));
    assert!(third > first);
    // The patch frees as the first mining ends, and the lowest id of the Drones waiting claims it in the next tick.
    assert_eq!(mines_from(&mut t, 4), first + MINING);
    assert!(is_waiting(&t, 6), "it came first, and waits on");
    assert_eq!(load(&t, 5), LOAD);
    assert_eq!(mines_from(&mut t, 6), first + 2 * MINING);
    // The patch gives 8 every 32 frames, its ceiling (`> Patch Occupancy > Consequence`).
    t.run_until(first + 3 * MINING);
    assert_eq!(remaining(&t, 1), 1500 - 3 * LOAD);
    assert_eq!(remaining(&t, 2), 1500);
}

/// `> Patch Occupancy`: "from any player — an enemy Drone mining a patch blocks your Drones too". Design change
/// 0135 (`> Assigned Drop-off > Drop-off gone`): a Drone whose owner has no Egress Node idles holding its load.
#[test]
fn an_enemy_drone_mining_a_patch_blocks_it() {
    // Patch 1, player 1's node 2 and Drone 3 (south, far), player 2's Drone 4 (north, near).
    let mut t = flat(16, 12)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 7))
        .spawn(DRONE, Some(P1), centre(5, 6))
        .spawn(DRONE, Some(P2), centre(5, 0))
        .start();
    gather(&mut t, 1, P1, &[3], 1);
    gather(&mut t, 1, P2, &[4], 1);
    let enemy = mines_from(&mut t, 4);
    run_to(&mut t, 40, "player 1's Drone waits", |t| is_waiting(t, 3));
    assert!(is_mining(&t, 4));
    assert_eq!(
        mines_from(&mut t, 3),
        enemy + MINING,
        "it mines as soon as the patch frees"
    );
    // Player 2 has no Egress Node: its Drone keeps its load and has nothing more to do.
    assert_eq!((order(&t, 4), load(&t, 4), drop_off(&t, 4)), (None, LOAD, None));
    assert_eq!(bank(&t, P2), 100, "its starting bank");
}

/// `> Assigned Drop-off > Set Drop-off Location command`: it "does not interrupt the Drone's current action", and
/// it redirects "a trip already in progress".
#[test]
fn set_drop_off_redirects_the_return_without_interrupting_the_mining() {
    // Patch 1, nodes 2 (near) and 3 (east), Drone 4.
    let mut t = flat(16, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(NODE, Some(P1), tile(10, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .start();
    gather(&mut t, 1, P1, &[4], 1);
    let began = mines_from(&mut t, 4);
    assert_eq!(drop_off(&t, 4), Some(id(2)));
    // In the middle of the mining.
    t.ability_at(began + 10, P1, &[id(4)], Ability::SetDropOff(id(3)));
    t.run_until(began + 10);
    assert_eq!(drop_off(&t, 4), Some(id(3)));
    assert_eq!(
        phase(&t, 4),
        Some(GatherPhase::Mining { done: 11 }),
        "the mining goes on"
    );
    assert_eq!(t.sim().returning_drones(id(3)), [id(4)]);
    assert_eq!(t.sim().returning_drones(id(2)), []);
    run_to(&mut t, 200, "the first delivery", |t| !deliveries(t).is_empty());
    assert_eq!(deliveries(&t)[0].2, 3, "to the new drop-off");
    assert_eq!(load(&t, 4), 0);
    // On the way back with the next load: the trip turns to the other node.
    run_to(&mut t, 200, "it carries the next load", |t| {
        phase(t, 4) == Some(GatherPhase::ToDropOff) && load(t, 4) == LOAD
    });
    t.run(4);
    let now = t.sim().tick();
    t.ability_at(now + 1, P1, &[id(4)], Ability::SetDropOff(id(2)));
    run_to(&mut t, 200, "the second delivery", |t| deliveries(t).len() == 2);
    assert_eq!(deliveries(&t)[1].2, 2);
    assert_eq!(
        order(&t, 4),
        Some(OrderKind::Gather(id(1))),
        "its Gather went on throughout"
    );
    assert_eq!(bank(&t, P1), 100 + 2 * LOAD);
}

/// The Drones' grid.
fn blocked(t: &TestSim, x: i32, y: i32) -> bool {
    let grid = t.sim().nav().grid(NavClass::GroundRugged, 1).expect("the Drone's grid");
    grid.is_blocked(TilePos::new(x, y))
}

/// `core/map.md > SpaceCrystalsPatch`: "When a patch is fully depleted, it disappears from the map", and its tile
/// opens (design change 0038). Design change 0138: the last load is what is left, after the full MiningDuration.
/// Design change 0134: the Drone that mined it looks for another patch where it stands, and goes on there once it
/// has delivered.
#[test]
fn a_depleted_patch_disappears_and_its_drones_go_on_at_another() {
    // Patch 1 holds 20; patch 2 is two tiles east of it; node 3; Drone 4.
    let mut t = flat(16, 10)
        .spawn_with(patch_of(20), tile(5, 2))
        .spawn(PATCH, None, tile(7, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .start();
    gather(&mut t, 1, P1, &[4], 1);
    run_to(&mut t, 400, "two loads delivered", |t| deliveries(t).len() == 2);
    assert_eq!((remaining(&t, 1), bank(&t, P1)), (4, 100 + 2 * LOAD));
    assert!(blocked(&t, 5, 2));
    let began = mines_from(&mut t, 4);
    t.run_until(began + MINING - 2);
    assert_eq!(
        (remaining(&t, 1), load(&t, 4)),
        (4, 0),
        "the full 32 frames for the last 4"
    );
    t.step();
    // The tick the patch is emptied in.
    assert!(t.sim().object(id(1)).is_none(), "the patch is gone");
    assert!(t.sim().events().contains(&SimEvent::Removed(id(1))));
    assert!(!blocked(&t, 5, 2), "its tile is open in the same tick");
    assert_eq!(load(&t, 4), 4, "a partial last load");
    assert_eq!(
        dangling_references(t.sim()),
        [],
        "its Gather names the patch only to say where it was"
    );
    // It looks round as the behaviours next run.
    t.step();
    assert_eq!(order(&t, 4), Some(OrderKind::Gather(id(2))), "on to the other patch");
    assert_eq!(phase(&t, 4), Some(GatherPhase::ToDropOff), "after it has delivered");
    run_to(&mut t, 100, "the last load delivered", |t| deliveries(t).len() == 3);
    assert_eq!(deliveries(&t)[2].3, 4);
    assert_eq!(bank(&t, P1), 100 + 20);
    mines_from(&mut t, 4);
    assert_eq!(order(&t, 4), Some(OrderKind::Gather(id(2))));
}

/// `> When a Gather ends` (design change 0123): with no patch left, the Gather ends, after the Drone has delivered
/// its load. Design change 0134 (its Verification: "Deplete the last patch: Drones arrive, find nothing, deliver
/// any load and idle"): the Drone on its way back to the patch walks on to where it was before it looks round.
#[test]
fn with_no_patch_left_a_drone_delivers_its_load_and_idles() {
    // Patch 1 holds 12, node 2, Drone 3 mines first (south), Drone 4 waits (north).
    let mut t = flat(16, 10)
        .spawn_with(patch_of(12), tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(DRONE, Some(P1), centre(5, 0))
        .start();
    gather(&mut t, 1, P1, &[3, 4], 1);
    // Drone 3 takes 8 and delivers them; Drone 4 takes the last 4 while Drone 3 is on its way back for more.
    let gone = run_to(&mut t, 200, "the patch is mined out", |t| {
        t.sim().object(id(1)).is_none()
    });
    assert_eq!((load(&t, 3), load(&t, 4), bank(&t, P1)), (0, 4, 100 + LOAD));
    assert_eq!(dangling_references(t.sim()), []);
    t.step();
    assert_eq!(
        (order(&t, 4), phase(&t, 4)),
        (Some(OrderKind::Gather(id(1))), Some(GatherPhase::ToDropOff)),
        "it found nothing, and delivers what it carries before its Gather ends"
    );
    assert_eq!(
        (order(&t, 3), phase(&t, 3)),
        (Some(OrderKind::Gather(id(1))), Some(GatherPhase::ToPatch)),
        "on to where the patch was"
    );
    // It arrives, finds nothing, and its Gather ends: it brakes to a stop beside the empty tile.
    let ended = run_to(&mut t, 100, "Drone 3 idles", |t| order(t, 3).is_none());
    assert!(ended > gone + 1);
    let spot = TilePos::new(5, 2).center();
    assert!(
        object(&t, 3).pos.distance(spot) < 64.0,
        "beside the empty tile: {:?}",
        object(&t, 3).pos
    );
    run_to(&mut t, 200, "the last load delivered", |t| deliveries(t).len() == 2);
    t.run(2);
    assert_eq!((order(&t, 3), order(&t, 4)), (None, None), "idle");
    assert_eq!(bank(&t, P1), 100 + 12);
    assert_eq!(t.sim().returning_drones(id(2)), []);
}

/// `> Unit Commands > Drop Off Resources`: a Drone ordered away keeps its load (design change 0129, `> Other orders
/// keep the load`); a Drop Off order is only a delivery, at the drop-off it names, after which the Drone idles there
/// (design change 0127, `> Drop Off order`).
#[test]
fn a_drop_off_order_delivers_the_load_and_the_drone_idles_at_the_node() {
    // Patch 1, nodes 2 (near) and 3 (east), Drone 4.
    let mut t = flat(16, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(NODE, Some(P1), tile(10, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .start();
    gather(&mut t, 1, P1, &[4], 1);
    let loaded = run_to(&mut t, 100, "it carries a load", |t| load(t, 4) == LOAD);
    // Ordered away: it keeps the load, and idles with it.
    let away = OrderKind::Move(Target::Point(Vec2::new(480.0, 224.0)));
    t.order_at(loaded + 1, P1, &[id(4)], away, false);
    run_to(&mut t, 100, "it arrived", |t| order(t, 4).is_none());
    assert_eq!((load(&t, 4), bank(&t, P1)), (LOAD, 100));
    assert_eq!(t.sim().returning_drones(id(2)), [], "it isn't gathering");
    // Drop Off at the other node, which doesn't become its AssignedDropOff.
    let now = t.sim().tick();
    t.order_at(now + 1, P1, &[id(4)], OrderKind::DropOff(id(3)), false);
    run_to(&mut t, 200, "delivered", |t| !deliveries(t).is_empty());
    assert_eq!(deliveries(&t)[0].2, 3);
    assert_eq!(bank(&t, P1), 100 + LOAD);
    t.step();
    assert_eq!(order(&t, 4), None, "the order is complete");
    assert_eq!(drop_off(&t, 4), Some(id(2)));
    // It stays at the node: it doesn't go back to its patch.
    t.run(100);
    assert_eq!((order(&t, 4), deliveries(&t).len()), (None, 1));
    let (at, node) = (object(&t, 4).pos, object(&t, 3).pos);
    assert!(at.distance(node) < 96.0, "beside node 3: {at:?}, the node at {node:?}");
}

/// Design change 0135 (`> Assigned Drop-off > Drop-off gone`): a Drone whose Egress Node is destroyed assigns the
/// nearest one as it starts its return; with none left it idles holding its load, and delivers it when it is sent
/// gathering again (0128: to the patch first, where it turns round without mining).
#[test]
fn drones_of_a_lost_egress_node_assign_another_or_idle_with_their_load() {
    // Patch 1, nodes 2 (near) and 3 (east), Drone 4; and a Beacon, 5, so that player 1 isn't defeated with its
    // last node (`core/game_rules.md > Defeat Condition`).
    let mut t = flat(16, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(NODE, Some(P1), tile(10, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(ObjectKind::Beacon, Some(P1), tile(14, 8))
        .start();
    gather(&mut t, 1, P1, &[4], 1);
    let began = mines_from(&mut t, 4);
    t.debug_at(began + 5, DebugCommand::Remove { id: id(2) });
    t.run_until(began + 5);
    assert_eq!(drop_off(&t, 4), None, "its drop-off is gone");
    assert!(is_mining(&t, 4), "the mining goes on");
    run_to(&mut t, 200, "delivered", |t| !deliveries(t).is_empty());
    assert_eq!(deliveries(&t)[0].2, 3, "at the node left");
    assert_eq!(drop_off(&t, 4), Some(id(3)), "which is its drop-off now");
    // The last node goes while the Drone is on its way to it with the next load.
    run_to(&mut t, 200, "it returns with a load", |t| {
        phase(t, 4) == Some(GatherPhase::ToDropOff) && load(t, 4) == LOAD
    });
    let now = t.sim().tick();
    t.debug_at(now + 3, DebugCommand::Remove { id: id(3) });
    t.run_until(now + 4);
    assert_eq!((order(&t, 4), load(&t, 4), drop_off(&t, 4)), (None, LOAD, None));
    assert_eq!(dangling_references(t.sim()), []);
    t.run(40);
    assert_eq!(object(&t, 4).pos, object(&t, 4).prev_pos, "it stands");
    // A new node, and a new Gather: the old load is delivered at the new node, the nearest.
    let now = t.sim().tick();
    t.debug_at(
        now + 1,
        DebugCommand::Spawn {
            kind: NODE,
            owner: Some(P1),
            at: tile(2, 5),
            orientation: Default::default(),
        },
    );
    gather(&mut t, now + 2, P1, &[4], 1);
    run_to(&mut t, 200, "the kept load delivered", |t| deliveries(t).len() == 2);
    let node = deliveries(&t)[1].2;
    assert_eq!(object(&t, node).kind, NODE);
    assert_eq!(drop_off(&t, 4), Some(id(node)));
    assert_eq!(bank(&t, P1), 100 + 2 * LOAD);
}

/// Design change 0130 (`> Interrupted mining`; its Verification: "Order a mining Drone away at frame 20: it carries
/// nothing; another waiting Drone starts mining that patch immediately"): the Drone that waited mines from that very
/// tick.
#[test]
fn interrupted_mining_collects_nothing_and_frees_the_patch_at_once() {
    // Patch 1, node 2, Drone 3 (south) mines, Drone 4 (north) waits.
    let mut t = one_patch(1500).spawn(DRONE, Some(P1), centre(5, 0)).start();
    gather(&mut t, 1, P1, &[3, 4], 1);
    let began = mines_from(&mut t, 3);
    assert!(is_waiting(&t, 4));
    let away = OrderKind::Move(Target::Point(Vec2::new(800.0, 288.0)));
    t.order_at(began + 20, P1, &[id(3)], away, false);
    t.run_until(began + 20);
    assert_eq!((order(&t, 3), load(&t, 3), remaining(&t, 1)), (Some(away), 0, 1500));
    assert_eq!(phase(&t, 4), Some(GatherPhase::Mining { done: 1 }));
    t.run_until(began + 20 + MINING - 1);
    assert_eq!((load(&t, 4), remaining(&t, 1)), (LOAD, 1500 - LOAD));
}

fn command(player: PlayerId, command: Command) -> PlayerCommand {
    PlayerCommand { player, command }
}

fn order_of(player: PlayerId, units: &[u32], kind: OrderKind) -> PlayerCommand {
    let units = units.iter().map(|&n| id(n)).collect();
    command(
        player,
        Command::Order {
            units,
            kind,
            queue: false,
        },
    )
}

fn set_drop_off(player: PlayerId, units: &[u32], node: u32) -> PlayerCommand {
    let units = units.iter().map(|&n| id(n)).collect();
    let ability = Ability::SetDropOff(id(node));
    command(player, Command::Ability { units, ability })
}

/// The gate (`ARCHITECTURE.md > Commands, events and replays`): every invalid Gather, Drop Off and Set Drop-off is
/// rejected with its reason, and changes nothing.
#[test]
fn the_gate_rejects_invalid_worker_commands_and_they_change_nothing() {
    // Patch 1; patch 2 under player 1's plate 9, powered by its Generator 10; patch 3 far in the fog; player 1's
    // node 4, Drone 5 and Peacekeeper 6; player 2's node 7 and Drone 8.
    let scenario = flat(24, 10)
        .spawn(PATCH, None, tile(5, 2))
        .spawn(PATCH, None, tile(7, 2))
        .spawn(PATCH, None, tile(22, 8))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(DRONE, Some(P1), centre(5, 4))
        .spawn(ObjectKind::Peacekeeper, Some(P1), centre(3, 4))
        .spawn(NODE, Some(P2), tile(9, 5))
        .spawn(DRONE, Some(P2), centre(9, 4))
        .spawn(ObjectKind::ExtractionPlate, Some(P1), tile(7, 2))
        .spawn(ObjectKind::Generator, Some(P1), tile(14, 6))
        .build();
    let mut t = TestSim::new(scenario.clone());
    t.without_auto_targeting();
    let gather_at = |units: &[u32], patch: u32| order_of(P1, units, OrderKind::Gather(id(patch)));
    let drop_at = |units: &[u32], node: u32| order_of(P1, units, OrderKind::DropOff(id(node)));
    let invalid = [
        (gather_at(&[6], 1), Reject::NoWorkers),
        (gather_at(&[5], 4), Reject::NotGatherable(id(4))),
        (gather_at(&[5], 2), Reject::NotGatherable(id(2))),
        (gather_at(&[5], 3), Reject::NotVisible(id(3))),
        (gather_at(&[5], 99), Reject::UnknownObject(id(99))),
        (gather_at(&[8], 1), Reject::NotOwned(id(8))),
        (gather_at(&[], 1), Reject::NoUnits),
        (drop_at(&[6], 4), Reject::NoWorkers),
        (drop_at(&[5], 7), Reject::NotDropOff(id(7))),
        (drop_at(&[5], 1), Reject::NotDropOff(id(1))),
        (drop_at(&[5], 99), Reject::UnknownObject(id(99))),
        (drop_at(&[8], 4), Reject::NotOwned(id(8))),
        // Design change 0140: no Drone of them carries anything.
        (drop_at(&[5, 6], 4), Reject::NothingToDeliver),
        (set_drop_off(P1, &[6], 4), Reject::LacksAbility),
        (set_drop_off(P1, &[5], 7), Reject::NotDropOff(id(7))),
        (set_drop_off(P1, &[5], 5), Reject::NotDropOff(id(5))),
        (set_drop_off(P1, &[5], 99), Reject::UnknownObject(id(99))),
        (set_drop_off(P1, &[8], 4), Reject::NotOwned(id(8))),
        (set_drop_off(P1, &[99], 4), Reject::NoUnits),
    ];
    for (cmd, reject) in &invalid {
        assert_eq!(t.sim().validate(cmd), Err(*reject), "{cmd:?}");
        t.at(2, cmd.player, cmd.command.clone());
    }
    let mut twin = TestSim::new(scenario);
    twin.without_auto_targeting();
    t.run(3);
    twin.run(3);
    assert_eq!(t.sim().state_hash(), twin.sim().state_hash(), "they changed nothing");

    // The valid forms; the units that aren't workers ignore them (a valid Drop Off is `gathering_rules`, for 0140).
    let valid = [gather_at(&[5, 6], 1), set_drop_off(P1, &[5, 6], 4)];
    for cmd in &valid {
        assert_eq!(t.sim().validate(cmd), Ok(()), "{cmd:?}");
    }
    t.at(4, P1, valid[0].command.clone());
    t.run(1);
    assert_eq!(order(&t, 5), Some(OrderKind::Gather(id(1))));
    assert_eq!(order(&t, 6), None, "the Peacekeeper ignored it");
    // The plate mines its patch whatever the Drones do, and the Drone never turns to the plated patch: with the
    // enemy on patch 1 it waits.
    t.at(5, P2, order_of(P2, &[8], OrderKind::Gather(id(1))).command);
    t.run(120);
    assert!(remaining(&t, 2) < 1500, "the plate mined");
    assert_eq!(order(&t, 5), Some(OrderKind::Gather(id(1))));
    assert_eq!(order(&t, 8), Some(OrderKind::Gather(id(1))));
}

/// Question Q0037 point 9 (its proposal): a Drone produced with the rally point on a patch gets a Gather, its
/// default right-click there.
#[test]
fn a_drone_rallied_on_a_patch_gathers_it() {
    // Patch 1, node 2, Robo Portal 3.
    let mut t = flat(16, 14)
        .spawn(PATCH, None, tile(8, 4))
        .spawn(NODE, Some(P1), tile(11, 3))
        .spawn(ObjectKind::RoboPortal, Some(P1), tile(6, 6))
        .start();
    let portal = vec![id(3)];
    let to_portal = |command| Command::Production {
        objects: portal.clone(),
        command,
    };
    t.at(1, P1, to_portal(ProductionCommand::SetRally(Target::Object(id(1)))));
    t.at(1, P1, to_portal(ProductionCommand::Train(DRONE)));
    run_to(&mut t, 200, "the Drone is out", |t| t.sim().object(id(4)).is_some());
    t.step();
    assert_eq!(order(&t, 4), Some(OrderKind::Gather(id(1))));
    run_to(&mut t, 300, "its first delivery", |t| !deliveries(t).is_empty());
    assert_eq!(deliveries(&t)[0].1, 4);
}

/// The plan's determinism acceptance: a game of gathering with a patch mined out, a drop-off set, a node lost and a
/// Drone ordered about replays to the same state, also through JSON; the state serializes and loads as it was, and a
/// state saved in the middle plays on to the same end.
#[test]
fn a_gathering_game_replays_and_round_trips() {
    let mut scenario = flat(16, 12)
        .spawn_with(patch_of(24), tile(4, 2))
        .spawn(PATCH, None, tile(5, 2))
        .spawn(PATCH, None, tile(6, 2))
        .spawn(PATCH, None, tile(7, 2))
        .spawn(NODE, Some(P1), tile(5, 5))
        .spawn(NODE, Some(P1), tile(9, 5))
        .spawn(NODE, Some(P2), tile(5, 9));
    for i in 0..6 {
        scenario = scenario.spawn(
            DRONE,
            Some(P1),
            SpawnAt::Point(Vec2::new(250.0 + 44.0 * i as f32, 290.0)),
        );
    }
    let mut t = scenario.spawn(DRONE, Some(P2), centre(1, 4)).start();
    let drones: Vec<ObjectId> = (8..14).map(ObjectId).collect();
    t.order_at(1, P1, &drones, OrderKind::Gather(id(1)), false);
    t.order_at(2, P2, &[id(14)], OrderKind::Gather(id(2)), false);
    t.ability_at(90, P1, &drones[..3], Ability::SetDropOff(id(6)));
    t.order_at(200, P1, &drones[4..], OrderKind::DropOff(id(6)), false);
    t.order_at(
        260,
        P1,
        &drones[..1],
        OrderKind::Move(Target::Point(Vec2::new(700.0, 500.0))),
        false,
    );
    t.order_at(300, P1, &drones[..1], OrderKind::Gather(id(3)), true);
    t.debug_at(420, DebugCommand::Remove { id: id(5) });
    t.run_until(500);
    let saved: Sim = serde_json::from_str(&serde_json::to_string(t.sim()).unwrap()).unwrap();
    assert_eq!(&saved, t.sim());
    t.run_until(900);
    let sim = t.sim();
    assert!(sim.object(id(1)).is_none(), "the patch of 24 was mined out");
    assert!(deliveries(&t).len() > 10, "{:?}", deliveries(&t));
    assert!(
        deliveries(&t).iter().any(|d| d.1 == 14 && d.2 == 7),
        "player 2 gathers too"
    );
    assert_eq!(dangling_references(sim), []);

    let replayed = t.replay().run();
    assert_eq!(replayed.state_hash(), sim.state_hash());
    assert_eq!(&replayed, sim);
    let json = serde_json::to_string(t.replay()).unwrap();
    let loaded: Replay = serde_json::from_str(&json).unwrap();
    assert_eq!(&loaded, t.replay());
    assert_eq!(loaded.run().state_hash(), sim.state_hash());
    // The save of tick 500 plays on like the game it was taken from: nothing was ordered after it.
    let mut saved = saved;
    for _ in 500..900 {
        saved.step(&Default::default());
    }
    assert_eq!(saved.state_hash(), sim.state_hash());
    assert_eq!(&saved, sim);
}
