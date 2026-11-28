//! Idle units that overlap move apart (design change 0126, answering question Q0045 point 1;
//! `systems/units.md > UnitCollision > Units that already overlap > Idle units push apart`; `movement::separate`):
//! 0126's Verification, the units that are never moved, and stacks of idle units on one point. `TestSim` and the
//! bare sims here check the invariants after every tick: I1 excuses a pair only while it has overlapped since the
//! tick before, and I5/I7 let a unit without a goal move only while it is idle and overlapped another.

use sc_sim::orders::state::{Behaviour, GatherPhase};
use sc_sim::testing::invariants::Invariants;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{ObjectId, ObjectKind, OrderKind, PlayerId, Sim, SpawnAt, Tick, TickInputs, TilePos, Vec2};

const P1: PlayerId = PlayerId(1);
const DRONE: ObjectKind = ObjectKind::Drone;
/// Half the Drone's Silhouette, 36 × 36 space units.
const HALF: f32 = 18.0;

fn id(n: u32) -> ObjectId {
    ObjectId(n)
}

fn is_mining(t: &TestSim, n: u32) -> bool {
    let orders = t.sim().object(id(n)).expect("exists").orders.as_ref().expect("a unit");
    let state = orders.current.as_ref().map(|active| &active.state);
    matches!(state, Some(Behaviour::Gather(gather)) if matches!(gather.phase, GatherPhase::Mining { .. }))
}

fn overlap(t: &TestSim, a: u32, b: u32) -> bool {
    let d = (t.pos(a) - t.pos(b)).abs();
    d.x < 2.0 * HALF && d.y < 2.0 * HALF
}

/// Two Drones of player 1 on one patch (ids: patch 1, node 2, Drones 3 and 4), stepped until Drone 3 mines with
/// Drone 4 waiting inside it and both at rest.
fn one_inside_the_other() -> TestSim {
    let mut t = ScenarioBuilder::new(&vec![".".repeat(14); 10].join("\n"))
        .spawn(ObjectKind::CrystalPatch, None, SpawnAt::Tile(TilePos::new(6, 2)))
        .spawn(ObjectKind::EgressNode, Some(P1), SpawnAt::Tile(TilePos::new(6, 5)))
        .spawn(DRONE, Some(P1), SpawnAt::Point(Vec2::new(416.0, 250.0)))
        .spawn(DRONE, Some(P1), SpawnAt::Point(Vec2::new(420.0, 294.0)))
        .start();
    t.order_at(1, P1, &[id(3), id(4)], OrderKind::Gather(id(1)), false);
    let at_rest = |t: &TestSim| t.mover(3).momentum == Vec2::ZERO && t.mover(4).momentum == Vec2::ZERO;
    while !(is_mining(&t, 3) && at_rest(&t)) {
        t.step();
        assert!(t.sim().tick() < 40, "{}", t.ascii(None));
    }
    assert!(overlap(&t, 3, 4), "{} and {}", t.pos(3), t.pos(4));
    t
}

/// 0126's Verification: "Stop a gathering Drone while it's inside another Drone; if both are idle they drift apart
/// until both are clickable separately." Both are stopped: both move, away from each other, until their boxes no
/// longer overlap, and no farther. Then they stand for good.
#[test]
fn two_idle_drones_inside_each_other_drift_apart() {
    let mut t = one_inside_the_other();
    let (first, second) = (t.pos(3), t.pos(4));
    let now = t.sim().tick();
    t.order_at(now + 1, P1, &[id(3), id(4)], OrderKind::Stop, false);
    let mut ticks = 0;
    while overlap(&t, 3, 4) {
        let gap = (t.pos(3) - t.pos(4)).abs().max_element();
        t.step();
        ticks += 1;
        assert!(
            (t.pos(3) - t.pos(4)).abs().max_element() > gap,
            "tick {}: farther apart every tick",
            t.sim().tick()
        );
        assert!(ticks <= 18, "{}", t.ascii(None));
    }
    let (a, b) = (t.pos(3), t.pos(4));
    assert!(a != first && b != second, "both moved: {a} and {b}");
    // Just until clear: their boxes touch, on the axis they parted along.
    let d = (a - b).abs();
    assert!(
        d.max_element() >= 2.0 * HALF && d.max_element() < 2.0 * HALF + 0.01,
        "{d}"
    );
    for _ in 0..40 {
        t.step();
        assert_eq!((t.pos(3), t.pos(4)), (a, b), "tick {}: they stand", t.sim().tick());
    }
}

/// 0126's Verification: "Stop it inside a still-mining Drone: only the stopped (idle) one moves off." The mining
/// Drone has a command: it neither moves nor stops mining, and takes its load to the node afterwards.
#[test]
fn a_drone_stopped_inside_a_mining_drone_moves_off_alone() {
    let mut t = one_inside_the_other();
    let (miner, waiter) = (t.pos(3), t.pos(4));
    let now = t.sim().tick();
    t.order_at(now + 1, P1, &[id(4)], OrderKind::Stop, false);
    let mut ticks = 0;
    while overlap(&t, 3, 4) {
        t.step();
        ticks += 1;
        // Until its mining is done, when it leaves for the node by itself.
        if is_mining(&t, 3) {
            assert_eq!(t.pos(3), miner, "tick {}: the mining Drone isn't moved", t.sim().tick());
            assert_ne!(
                t.pos(4),
                waiter,
                "tick {}: the stopped one is on its way out",
                t.sim().tick()
            );
        }
        assert!(ticks <= 18, "{}", t.ascii(None));
    }
    let off = t.pos(4);
    assert!(off != waiter && off.distance(waiter) <= 36.0, "{waiter} to {off}");
    // Apart, they collide like any two units, and never overlap again (I1, every tick).
    for _ in 0..300 {
        t.step();
        assert!(!overlap(&t, 3, 4), "tick {}\n{}", t.sim().tick(), t.ascii(Some(off)));
    }
    assert!(
        t.sim().player(P1).unwrap().crystals > 100,
        "the miner went on delivering"
    );
}

/// "Units with an active command (incl. Hold Position, mining) are not moved": a Drone on Hold Position inside a
/// mining Drone stays where it is, and so does the miner, for as long as the mining lasts (0106's rule for units
/// with orders: nothing is pushed apart).
#[test]
fn a_unit_with_a_command_is_never_moved_off() {
    let mut t = one_inside_the_other();
    let (miner, waiter) = (t.pos(3), t.pos(4));
    let now = t.sim().tick();
    t.order_at(now + 1, P1, &[id(4)], OrderKind::Hold, false);
    while is_mining(&t, 3) {
        t.step();
        assert_eq!((t.pos(3), t.pos(4)), (miner, waiter), "tick {}", t.sim().tick());
    }
    assert!(t.sim().tick() > now + 10);
    // The miner leaves through it for the node; the Drone on Hold never moves.
    for _ in 0..200 {
        t.step();
        assert_eq!(t.pos(4), waiter, "tick {}: on Hold", t.sim().tick());
    }
}

/// `count` idle units of `kind` of player 1, all put on one point by editing a saved state (no command stacks
/// units: the gate refuses a teleport onto another unit). Returns the sim and the units' half extent.
fn stack(kind: ObjectKind, count: u32, at: Vec2) -> Sim {
    let mut scenario = ScenarioBuilder::new(&vec![".".repeat(16); 12].join("\n"));
    for i in 0..count {
        scenario = scenario.spawn(kind, Some(P1), SpawnAt::Point(Vec2::new(60.0 + 50.0 * i as f32, 60.0)));
    }
    let mut json = serde_json::to_value(scenario.start().sim()).expect("a state serializes");
    for object in json["objects"].as_array_mut().expect("the objects are a list") {
        object["pos"] = serde_json::to_value(at).expect("a position serializes");
        object["prev_pos"] = object["pos"].clone();
    }
    serde_json::from_value(json).expect("the edited state loads")
}

/// The ticks until no two units of `sim` overlap, stepping with the invariants checked; panics after `limit`.
fn separates_within(sim: &mut Sim, half: f32, limit: Tick) -> Tick {
    let mut invariants = Invariants::new(sim);
    let stacked = |sim: &Sim| {
        let at: Vec<Vec2> = sim.objects().map(|o| o.pos).collect();
        at.iter()
            .enumerate()
            .any(|(i, a)| at[i + 1..].iter().any(|b| (*a - *b).abs().max_element() < 2.0 * half))
    };
    for tick in 1..=limit {
        sim.step(&TickInputs::default());
        if let Err(violation) = invariants.check(sim, &TickInputs::default()) {
            // The vision of an edited state is stale; the movement invariants are what this test is about.
            assert!(!violation.invariant.starts_with('I'), "tick {tick}: {violation:?}");
        }
        if !stacked(sim) {
            return tick;
        }
    }
    panic!("still stacked after {limit} ticks");
}

/// "Units never stay stacked indefinitely": 2, 3, 5 and 8 idle Drones on one point come apart within the
/// documented bound of `movement::separate`, 18 ticks for each unit but the first (36 su at 2 su a tick), without
/// ever entering a unit they didn't overlap (I1), and then stand. Peacekeepers too, 24 su wide: 12 ticks each.
#[test]
fn a_stack_of_idle_units_on_one_point_separates_within_the_bound() {
    for (kind, half, each) in [(DRONE, HALF, 18), (ObjectKind::Peacekeeper, 12.0, 12)] {
        for count in [2, 3, 5, 8] {
            let mut sim = stack(kind, count, Vec2::new(500.0, 400.0));
            let ticks = separates_within(&mut sim, half, each * (count - 1));
            assert!(ticks >= each / 2, "{kind:?} × {count}: {ticks} ticks");
            let at: Vec<Vec2> = sim.objects().map(|o| o.pos).collect();
            for _ in 0..30 {
                sim.step(&TickInputs::default());
            }
            let after: Vec<Vec2> = sim.objects().map(|o| o.pos).collect();
            assert_eq!(at, after, "{kind:?} × {count}: apart, they stand");
        }
    }
}

/// A stack against the map's corner still comes apart: a unit that can't go the near way (the edge) takes
/// another.
#[test]
fn a_stack_in_a_corner_separates_too() {
    let mut sim = stack(DRONE, 4, Vec2::new(HALF, HALF));
    separates_within(&mut sim, HALF, 18 * 3 * 2);
}

/// **Negative test** of I5/I7's exception for separating units (`docs/movement.md > 8`): it is for a unit without a
/// command that overlapped another. A Drone on Hold Position inside a mining Drone that moves all the same breaks
/// the invariant, and so does an idle Drone that overlaps nobody. The state is edited, as in
/// `invariants_overlap.rs`: the movement step never does this.
#[test]
fn only_an_idle_unit_that_overlapped_another_moves_off() {
    let mut t = one_inside_the_other();
    let now = t.sim().tick();
    t.order_at(now + 1, P1, &[id(4)], OrderKind::Hold, false);
    // Drone 5 stands idle, clear of everything.
    t.spawn_unit_at(now + 1, DRONE, P1, Vec2::new(700.0, 400.0));
    t.run_until(now + 3);
    let before = t.sim().clone();
    t.step();
    let moved = |n: u32| {
        let mut json = serde_json::to_value(t.sim()).expect("a state serializes");
        let objects = json["objects"].as_array_mut().expect("the objects are a list");
        let object = objects
            .iter_mut()
            .find(|o| o["id"] == n)
            .expect("the unit is in the state");
        object["pos"][0] = (object["pos"][0].as_f64().unwrap() + 2.0).into();
        let after: Sim = serde_json::from_value(json).expect("the edited state loads");
        let result = Invariants::new(&before).check(&after, &TickInputs::default());
        result.err().map(|violation| violation.invariant)
    };
    assert_eq!(moved(4), Some("I5/I7"), "on Hold, inside another");
    assert_eq!(moved(5), Some("I5/I7"), "idle, overlapping nobody");
}

/// I5/I7's exception for separating units counts a unit that dies in the tick: two idle Drones of player 1, of 1 HP
/// and no shields each, stacked on one point (the state edited, as in [`stack`]), move apart, and a Peacekeeper of
/// player 2 shoots one of them dead while they still overlap. The other moved off it in the movement of that tick
/// (phase 5d), before the shot (5e) removed it (`systems/units.md > UnitCollision > Units that already overlap > Idle
/// units push apart`).
#[test]
fn an_idle_drone_moves_off_one_that_dies_in_the_same_tick() {
    let at = Vec2::new(300.0, 300.0);
    let scenario = ScenarioBuilder::new(&vec![".".repeat(16); 12].join("\n"))
        .spawn(DRONE, Some(P1), SpawnAt::Point(Vec2::new(100.0, 100.0)))
        .spawn(DRONE, Some(P1), SpawnAt::Point(Vec2::new(150.0, 100.0)))
        .spawn(
            ObjectKind::Peacekeeper,
            Some(PlayerId(2)),
            SpawnAt::Point(at + Vec2::new(90.0, 0.0)),
        );
    let mut json = serde_json::to_value(scenario.start().sim()).expect("a state serializes");
    for object in json["objects"].as_array_mut().expect("the objects are a list") {
        if object["kind"] == "Drone" {
            object["pos"] = serde_json::to_value(at).expect("a position serializes");
            object["prev_pos"] = object["pos"].clone();
            object["hp"] = serde_json::json!(1.0);
            object["shields"] = serde_json::json!(0.0);
        }
    }
    let mut sim: Sim = serde_json::from_value(json).expect("the edited state loads");
    let mut invariants = Invariants::new(&sim);
    for _ in 0..18 {
        let before = sim.clone();
        sim.step(&TickInputs::default());
        if let Err(violation) = invariants.check(&sim, &TickInputs::default()) {
            // The vision of an edited state is stale; the movement invariants are what this test is about.
            assert!(
                !violation.invariant.starts_with('I'),
                "tick {}: {violation:?}",
                sim.tick()
            );
        }
        let alive: Vec<ObjectId> = sim.objects().filter(|o| o.kind == DRONE).map(|o| o.id).collect();
        if let [survivor] = alive[..] {
            let was = |n: ObjectId| before.object(n).expect("both were there").pos;
            let dead = before
                .objects()
                .find(|o| o.kind == DRONE && o.id != survivor)
                .expect("two Drones")
                .id;
            assert!(
                (was(survivor) - was(dead)).abs().max_element() < 2.0 * HALF,
                "they overlapped"
            );
            let moved = sim.object(survivor).expect("it survived");
            assert_ne!(
                moved.pos, moved.prev_pos,
                "it moved off the other in the tick the other died"
            );
            return;
        }
    }
    panic!("no Drone died while they overlapped");
}
