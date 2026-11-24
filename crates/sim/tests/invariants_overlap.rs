//! Invariant I1 and its two exceptions (slice M3-H2; design change 0106, `testing::invariants`): **negative tests**.
//! The movement step never brings two units to overlap unless both are gathering Drones, so no game can show the
//! checker failing. These tests put a unit where it must not be, by editing a saved state, and ask the checker:
//! an overlap that neither exception covers is still caught, and each exception covers no more than it says.
//!
//! The edit is passed to the checker as a teleport (`DebugCommand::SetPosition` in the tick's inputs), which is how
//! it tells a unit that was put somewhere from one that moved without a goal (I5). The gate itself refuses a
//! teleport onto another unit, so only an edited state gets a unit there.

use sc_sim::orders::state::{Behaviour, GatherPhase};
use sc_sim::testing::invariants::Invariants;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Sim, SpawnAt, Target, TickInputs,
    TilePos, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const DRONE: ObjectKind = ObjectKind::Drone;
const PK: ObjectKind = ObjectKind::Peacekeeper;

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn pos(sim: &Sim, n: u32) -> Vec2 {
    sim.object(id(n)).expect("exists").pos
}

/// `sim` as saved and loaded again, with unit `n` at `to`.
fn put(sim: &Sim, n: u32, to: Vec2) -> Sim {
    let mut json = serde_json::to_value(sim).expect("a state serializes");
    let objects = json["objects"].as_array_mut().expect("the objects are a list");
    let object = objects
        .iter_mut()
        .find(|o| o["id"] == n)
        .expect("the unit is in the state");
    object["pos"] = serde_json::to_value(to).expect("a position serializes");
    object["prev_pos"] = object["pos"].clone();
    serde_json::from_value(json).expect("the edited state loads")
}

/// The inputs of a tick that teleported unit `n`.
fn teleport(n: u32, to: Vec2) -> TickInputs {
    let command = Command::Debug(DebugCommand::SetPosition { id: id(n), pos: to });
    TickInputs {
        commands: vec![PlayerCommand { player: P1, command }],
    }
}

/// Whether the checker, which last saw the state `before`, finds I1 broken once unit `n` is put at `to`.
fn breaks_i1(checker: &Invariants, before: &Sim, n: u32, to: Vec2) -> bool {
    let after = put(before, n, to);
    let result = checker.clone().check(&after, &teleport(n, to));
    // Other invariants may object to an edited state (its vision is stale); I1 is checked before them.
    matches!(result, Err(violation) if violation.invariant == "I1")
}

fn is_mining(sim: &Sim, n: u32) -> bool {
    let orders = sim.object(id(n)).expect("exists").orders.as_ref().expect("a unit");
    let state = orders.current.as_ref().map(|active| &active.state);
    matches!(state, Some(Behaviour::Gather(gather)) if matches!(gather.phase, GatherPhase::Mining { .. }))
}

/// A patch at tile (6, 2) and an Egress Node of player 1 south of it. Drone 3 of player 1 mines the patch and Drone
/// 4 of player 2 waits for it, inside Drone 3; Drone 5 of player 1 and Peacekeeper 6 stand idle to the east. The
/// state after 36 ticks: the two Drones have come to rest (they brake, slice M3-G), and Drone 3 mines on.
fn field() -> TestSim {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(14); 10].join("\n"))
        .spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(6, 2)))
        .spawn(ObjectKind::EgressNode, Some(P1), SpawnAt::Tile(TilePos::new(6, 5)))
        .spawn(DRONE, Some(P1), SpawnAt::Point(Vec2::new(416.0, 250.0)))
        .spawn(DRONE, Some(P2), SpawnAt::Point(Vec2::new(420.0, 294.0)))
        .spawn(DRONE, Some(P1), SpawnAt::Point(Vec2::new(600.0, 250.0)))
        .spawn(PK, Some(P1), SpawnAt::Point(Vec2::new(700.0, 250.0)))
        .start();
    t.without_auto_targeting();
    t.order_at(1, P1, &[id(3)], OrderKind::Gather(id(1)), false);
    t.order_at(1, P2, &[id(4)], OrderKind::Gather(id(1)), false);
    t.run_until(36);
    assert!(is_mining(t.sim(), 3));
    t
}

/// The rule itself: two units that come to overlap break I1. Two idle units, an idle Drone and a Peacekeeper.
#[test]
fn two_units_that_come_to_overlap_break_i1() {
    let t = field();
    let sim = t.sim();
    let checker = Invariants::new(sim);
    assert!(
        breaks_i1(&checker, sim, 6, pos(sim, 5)),
        "a Peacekeeper on an idle Drone"
    );
    assert!(
        breaks_i1(&checker, sim, 5, pos(sim, 6) + Vec2::new(29.0, 0.0)),
        "by one space unit"
    );
    // Touching is free, as ever.
    assert!(!breaks_i1(&checker, sim, 5, pos(sim, 6) + Vec2::new(30.0, 0.0)));
}

/// Exception 1 is for **two gathering Drones**. A unit that isn't one may not overlap one that is: not an idle
/// Drone, not a Peacekeeper, whoever owns them.
#[test]
fn a_gathering_drone_and_a_unit_that_is_not_one_break_i1() {
    let t = field();
    let sim = t.sim();
    let checker = Invariants::new(sim);
    for gatherer in [3, 4] {
        assert!(
            breaks_i1(&checker, sim, 5, pos(sim, gatherer)),
            "an idle Drone on gathering Drone {gatherer}"
        );
        assert!(
            breaks_i1(&checker, sim, 6, pos(sim, gatherer)),
            "a Peacekeeper on gathering Drone {gatherer}"
        );
    }
    // The control: the two gathering Drones overlap as the state stands, and may overlap any other way.
    assert!(
        checker
            .clone()
            .check(&put(sim, 4, pos(sim, 4)), &TickInputs::default())
            .is_ok()
    );
    assert!(!breaks_i1(&checker, sim, 4, pos(sim, 3)), "one right on the other");
}

/// Exception 1 ends with the job. A Drone that was gathering a tick ago and has another order now was no gathering
/// Drone when it moved: it may not come to overlap one.
#[test]
fn a_drone_that_left_its_job_for_another_order_breaks_i1_by_a_new_overlap() {
    let t = field();
    // Apart first, both still gathering: the checker sees no overlap.
    let apart = pos(t.sim(), 3) + Vec2::new(60.0, 0.0);
    let mut sim = put(t.sim(), 4, apart);
    let mut checker = Invariants::new(&sim);
    // Drone 4 is ordered to Move: it steps east, clear of Drone 3.
    let to = apart + Vec2::new(200.0, 0.0);
    let command = Command::Order {
        units: vec![id(4)],
        kind: OrderKind::Move(Target::Point(to)),
        queue: false,
    };
    let inputs = TickInputs {
        commands: vec![PlayerCommand { player: P2, command }],
    };
    sim.step(&inputs);
    let result = checker.check(&sim, &inputs);
    assert!(
        !matches!(&result, Err(violation) if violation.invariant == "I1"),
        "{result:?}"
    );
    // Put back onto the mining Drone in the very tick its Move began: no exception covers it.
    let checker = Invariants::new(&put(t.sim(), 4, apart));
    assert!(breaks_i1(&checker, &sim, 4, pos(&sim, 3)));
}

/// Exception 2 is for a pair **that already overlapped**, and only until it has separated. Drone 4 is stopped inside
/// the mining Drone 3: no gathering Drone any more, it may stay there and move about in it. Once the two have been
/// apart for a tick, a new overlap between them breaks I1.
#[test]
fn a_pair_that_overlapped_is_excused_only_until_it_has_separated() {
    let mut t = field();
    t.order_at(37, P2, &[id(4)], OrderKind::Stop, false);
    // `TestSim` checks every tick: stopped inside Drone 3, Drone 4 breaks nothing.
    t.run_until(40);
    assert!(is_mining(t.sim(), 3) && !is_mining(t.sim(), 4));
    let sim = t.sim();
    let miner = pos(sim, 3);
    let checker = Invariants::new(sim);
    // Still overlapping, elsewhere in Drone 3: excused, for the pair overlapped the tick before.
    assert!(!breaks_i1(&checker, sim, 4, miner + Vec2::new(-20.0, 10.0)));
    // But not with anyone else: the idle Drone 5 never overlapped it.
    assert!(breaks_i1(&checker, sim, 5, pos(sim, 4)));
    // Apart for a tick, then inside again: caught.
    let apart = put(sim, 4, miner + Vec2::new(36.0, 0.0));
    let checker = Invariants::new(&apart);
    assert!(breaks_i1(&checker, &apart, 4, miner + Vec2::new(35.0, 0.0)));
    assert!(breaks_i1(&checker, &apart, 4, miner));
}
