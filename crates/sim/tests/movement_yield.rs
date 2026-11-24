//! Friendly yielding, occupied targets and the give-up schedule (`docs/movement.md > 5.6`, `> 6.5`, `> 6.6`), by
//! the Verification lists of the design changes:
//! - 0022, 0023 and 0027 (`systems/units.md > UnitCollision > Ground Collision > Friendly yielding`): an idle unit
//!   takes one short sidestep for a blocked unit of its owner; enemies, units on Hold, busy units and units that
//!   can't move never do; a unit that arrived doesn't yield to the rest of its own group order.
//! - 0024 and 0026 (`systems/controls/behaviors.md > MovingToLocation > Failure`, `> Occupied target location`):
//!   stuck after 24 ticks, three re-route attempts after waits of 8, 16 and 32 ticks, then give up; arrived when
//!   stuck within 2 grid units of the destination behind units that aren't moving.
//!
//! Every `TestSim` checks the invariants I1–I7 after each tick (`docs/movement.md > 8`): collision stays hard while
//! units sidestep.

use sc_sim::mover::{GIVE_UP_TICKS, MAX_REPATHS, OCCUPIED_ARRIVAL_GU, REPATH_WAITS, STUCK_TICKS};
use sc_sim::testing::invariants::Invariants;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Sim, SimEvent, Target,
    TickInputs, Vec2, gu,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PK: ObjectKind = ObjectKind::Peacekeeper; // 24 × 24 su, 4 su per tick
const GUARD: ObjectKind = ObjectKind::Guard; // 36 × 36 su, 5 su per tick

/// 16 × 12 open tiles: 1024 × 768 su.
const OPEN: &str = "
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
    ................
";

/// A corridor one tile (64 su) wide and 16 long, y from 64 to 128.
const CORRIDOR: &str = "
    ~~~~~~~~~~~~~~~~
    ................
    ~~~~~~~~~~~~~~~~
";

/// A passage three tiles (192 su) wide and 16 long, y from 64 to 256.
const PASSAGE: &str = "
    ~~~~~~~~~~~~~~~~
    ................
    ................
    ................
    ~~~~~~~~~~~~~~~~
";

fn v(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

fn move_to(p: Vec2) -> OrderKind {
    OrderKind::Move(Target::Point(p))
}

fn ids(range: std::ops::RangeInclusive<u32>) -> Vec<ObjectId> {
    range.map(ObjectId).collect()
}

fn failures(t: &TestSim) -> usize {
    let failed = |(_, e): &&(u32, SimEvent)| matches!(e, SimEvent::MoveFailed { .. });
    t.events().iter().filter(failed).count()
}

fn is_idle(t: &TestSim, id: u32) -> bool {
    let unit = t.sim().object(ObjectId(id)).unwrap();
    unit.orders.as_ref().unwrap().is_idle()
}

/// Steps until none of `units` is moving; the tick. Panics, with the map, after `max_tick`.
#[track_caller]
fn run_until_done(t: &mut TestSim, units: &[ObjectId], max_tick: u32) -> u32 {
    t.run_until(t.sim().tick().max(2));
    while units.iter().any(|id| t.status(id.0) == MoveStatus::Moving) {
        assert!(t.sim().tick() < max_tick, "still moving\n{}", t.ascii(None));
        t.step();
    }
    t.sim().tick()
}

/// The middle of the block of [`walk_through_a_blob_of`].
const BLOB_CENTRE: Vec2 = Vec2::new(512.0, 384.0);
/// The unit that walks through it, and its way.
const WALKER: u32 = 26;
const WALK: (Vec2, Vec2) = (Vec2::new(200.0, 384.0), Vec2::new(824.0, 384.0));

fn blob_place(i: u32) -> Vec2 {
    BLOB_CENTRE + v(28.0 * ((i % 5) as f32 - 2.0), 28.0 * ((i / 5) as f32 - 2.0))
}

/// A block of 25 idle Peacekeepers of `owner`, 28 su apart (4 su between their boxes), around [`BLOB_CENTRE`]: units
/// 1 to 25. Unit 26, of player 1, stands west of it and is ordered straight through it in tick 2.
fn walk_through_a_blob_of(owner: PlayerId) -> TestSim {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting();
    for i in 0..25 {
        t.spawn_unit_at(1, PK, owner, blob_place(i));
    }
    t.spawn_unit_at(1, PK, P1, WALK.0)
        .order_at(2, P1, &[ObjectId(WALKER)], move_to(WALK.1), false);
    t
}

/// 0022, Verification: "Walk a unit through your own idle group: they step aside and it passes." The walker goes
/// through the middle of the block, and the units it meets each take short sidesteps and then stand again.
#[test]
fn a_unit_walks_through_its_own_idle_group() {
    let mut t = walk_through_a_blob_of(P1);
    t.run(2);
    let mut through_the_middle = false;
    // 624 su at 4 su per tick is 156 ticks.
    while t.status(WALKER) == MoveStatus::Moving {
        assert!(t.sim().tick() < 2 + 156 + 60, "still walking\n{}", t.ascii(None));
        t.step();
        let offset = (t.pos(WALKER) - BLOB_CENTRE).abs();
        through_the_middle |= offset.x < 20.0 && offset.y < 30.0;
    }
    assert_eq!((t.pos(WALKER), t.status(WALKER)), (WALK.1, MoveStatus::Arrived));
    assert!(through_the_middle, "it went around\n{}", t.ascii(None));
    assert_eq!(t.mover(WALKER).repaths_left, MAX_REPATHS, "it never had to re-route");
    assert_eq!(failures(&t), 0);
    t.run(20);
    let moved: Vec<f32> = (0..25).map(|i| t.pos(i + 1).distance(blob_place(i))).collect();
    assert!(moved.iter().filter(|d| **d > 0.0).count() >= 3, "{moved:?}");
    // Short sidesteps: nobody ends farther than a few widths from where it stood, and everybody stands again.
    assert!(moved.iter().all(|d| *d <= 72.0), "{moved:?}\n{}", t.ascii(None));
    for id in 1..=25 {
        assert_eq!((t.mover(id).sidestep, t.mover(id).velocity), (None, Vec2::ZERO));
        assert!(is_idle(&t, id));
    }
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
}

/// 0022, Verification: "Do the same through an enemy group: it must path around." Enemies never yield: no unit of
/// the block moves, and the walker goes around it.
#[test]
fn a_unit_paths_around_an_enemy_idle_group() {
    let mut t = walk_through_a_blob_of(P2);
    run_until_done(&mut t, &[ObjectId(WALKER)], 2 + 156 + 120);
    assert_eq!((t.pos(WALKER), t.status(WALKER)), (WALK.1, MoveStatus::Arrived));
    for i in 0..25 {
        assert_eq!(t.pos(i + 1), blob_place(i), "#{}", i + 1);
        assert_eq!(t.mover(i + 1).sidestep, None);
    }
}

/// 0022, Verification: "Two own units meeting in a one-tile corridor with no detour: both give up (no swap)."
/// Moving units never yield, friendly or not, and once both have given up nobody is moving, so neither steps aside.
#[test]
fn two_own_units_head_on_in_a_corridor_both_give_up() {
    let mut t = ScenarioBuilder::new(CORRIDOR).start();
    t.without_auto_targeting();
    let (west, east) = (v(96.0, 96.0), v(928.0, 96.0));
    t.spawn_unit_at(1, GUARD, P1, west)
        .spawn_unit_at(1, GUARD, P1, east)
        .order_at(2, P1, &[ObjectId(1)], move_to(east), false)
        .order_at(2, P1, &[ObjectId(2)], move_to(west), false);
    // They meet after (832 − 36) / (2 × 5) = 80 ticks.
    let done = run_until_done(&mut t, &ids(1..=2), 2 + 80 + GIVE_UP_TICKS + 8);
    assert!(done > 82 + STUCK_TICKS, "they met first");
    assert_eq!((t.status(1), t.status(2)), (MoveStatus::Failed, MoveStatus::Failed));
    let (a, b) = (t.pos(1), t.pos(2));
    assert!(a.x < b.x && b.x - a.x < 64.0, "no swap: {a} and {b}");
    t.run(40);
    assert_eq!((t.pos(1), t.pos(2)), (a, b));
    assert!(is_idle(&t, 1) && is_idle(&t, 2));
}

/// Eight Peacekeepers of player 1 shoulder to shoulder across [`PASSAGE`] at x 500, on Hold Position from tick 2:
/// units 1 to 8. Unit 9, of player 1, comes from the west.
fn hold_wall() -> TestSim {
    let mut t = ScenarioBuilder::new(PASSAGE).start();
    t.without_auto_targeting();
    for i in 0..8 {
        t.spawn_unit_at(1, PK, P1, v(500.0, 76.0 + 24.0 * i as f32));
    }
    t.spawn_unit_at(1, PK, P1, v(100.0, 160.0))
        .order_at(2, P1, &ids(1..=8), OrderKind::Hold, false);
    t
}

/// 0023, Verification: "Through units on Hold Position: they don't budge and the mover paths around or gives up."
/// 0026, Verification: "A unit ordered to a point 3 tiles behind a wall of Hold Position units keeps trying
/// (re-routes) and eventually gives up instead of counting as arrived."
#[test]
fn a_wall_of_hold_units_doesnt_budge_and_a_unit_sent_far_behind_it_gives_up() {
    let mut t = hold_wall();
    // Three tiles behind the wall. The unit meets it at x 476: 216 su short, more than 2 GU.
    t.order_at(2, P1, &[ObjectId(9)], move_to(v(692.0, 160.0)), false)
        .run(2);
    let mut repaths = Vec::new();
    while t.status(9) == MoveStatus::Moving {
        assert!(t.sim().tick() < 2 + 94 + 3 * GIVE_UP_TICKS, "{}", t.ascii(None));
        let left = t.mover(9).repaths_left;
        t.step();
        if t.mover(9).repaths_left < left {
            repaths.push(t.sim().tick());
        }
    }
    assert_eq!(t.status(9), MoveStatus::Failed, "{}", t.ascii(None));
    assert_eq!(
        repaths.len(),
        usize::from(MAX_REPATHS),
        "it tried every re-route: {repaths:?}"
    );
    assert!(
        t.events()
            .contains(&(t.sim().tick(), SimEvent::MoveFailed { unit: ObjectId(9) }))
    );
    t.step();
    assert!(t.pos(9).x <= 476.0 && is_idle(&t, 9));
    for i in 0..8 {
        assert_eq!(t.pos(i + 1), v(500.0, 76.0 + 24.0 * i as f32), "#{} budged", i + 1);
    }
}

/// 0026: the same wall, and a destination less than 2 GU behind the spot where the wall stops the unit. The wall
/// will never move, so the unit counts as arrived, without a re-route attempt and before the stuck clock runs out.
#[test]
fn a_unit_stopped_within_two_grid_units_behind_units_that_wont_move_has_arrived() {
    let mut t = hold_wall();
    let goal = v(600.0, 160.0);
    t.order_at(2, P1, &[ObjectId(9)], move_to(goal), false);
    // It reaches the wall after 376 su, 94 ticks.
    let done = run_until_done(&mut t, &[ObjectId(9)], 2 + 94 + STUCK_TICKS + REPATH_WAITS[0]);
    assert_eq!(t.status(9), MoveStatus::Arrived, "{}", t.ascii(None));
    assert_eq!(t.mover(9).repaths_left, MAX_REPATHS, "no re-route attempt");
    assert!(t.pos(9).distance(goal) <= gu(OCCUPIED_ARRIVAL_GU) && t.pos(9).x <= 476.0);
    assert!(done < 2 + 94 + STUCK_TICKS, "straight away: tick {done}");
    assert_eq!(failures(&t), 0);
    for i in 0..8 {
        assert_eq!(t.pos(i + 1), v(500.0, 76.0 + 24.0 * i as f32));
    }
}

/// 0023, Verification (Who yields): a unit busy with a stationary task doesn't yield. A Peacekeeper attacking from
/// where it stands is walked around and never moves; `movement::yielding`'s unit tests cover each attack phase of a
/// unit without an order.
#[test]
fn a_unit_attacking_from_where_it_stands_doesnt_budge() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting();
    let (attacker, target) = (v(400.0, 300.0), v(400.0, 150.0));
    t.spawn_unit_at(1, PK, P1, attacker)
        .spawn_unit_at(1, GUARD, P2, target)
        .spawn_unit_at(1, PK, P1, v(300.0, 300.0))
        .order_at(2, P1, &[ObjectId(1)], OrderKind::Attack(ObjectId(2)), false)
        .order_at(2, P1, &[ObjectId(3)], move_to(v(520.0, 300.0)), false)
        .run(2);
    let mut attacked = false;
    // 220 su and a detour.
    while t.status(3) == MoveStatus::Moving {
        assert!(t.sim().tick() < 2 + 55 + 30, "{}", t.ascii(None));
        t.step();
        let unit = t.sim().object(ObjectId(1)).unwrap();
        attacked |= unit.attack.as_ref().unwrap().phase.is_some();
        assert!(!is_idle(&t, 1), "the target outlives the test");
        assert_eq!((t.pos(1), t.mover(1).sidestep), (attacker, None));
    }
    assert!(attacked);
    assert_eq!((t.pos(3), t.status(3)), (v(520.0, 300.0), MoveStatus::Arrived));
}

/// A game saved in tick 1 with unit 2's `Mover::speed_factor` set to `factor`: the unit's mode, which nothing in the
/// sim sets yet. Unit 1 (player 1) is ordered east through unit 2 (an idle Peacekeeper of player 1, exactly in its
/// way) in tick 2. Returns unit 2's positions after each tick up to tick 120; the invariants are checked on the
/// way.
fn idle_unit_in_a_mode(factor: f32) -> Vec<Vec2> {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting();
    t.spawn_unit_at(1, PK, P1, v(300.0, 300.0))
        .spawn_unit_at(1, PK, P1, v(400.0, 300.0))
        .run(1);
    let mut save = serde_json::to_value(t.sim()).unwrap();
    let unit = &mut save["objects"][1];
    assert_eq!(unit["id"], 2);
    unit["mover"]["speed_factor"] = factor.into();
    let mut sim: Sim = serde_json::from_value(save).unwrap();
    let mut invariants = Invariants::new(&sim);
    let order = PlayerCommand {
        player: P1,
        command: Command::Order {
            units: vec![ObjectId(1)],
            kind: move_to(v(600.0, 300.0)),
            queue: false,
        },
    };
    let mut places = Vec::new();
    for tick in 2..=120 {
        let inputs = TickInputs {
            commands: if tick == 2 { vec![order.clone()] } else { Vec::new() },
        };
        sim.step(&inputs);
        if let Err(violation) = invariants.check(&sim, &inputs) {
            panic!("{violation:?}");
        }
        places.push(sim.object(ObjectId(2)).unwrap().pos);
    }
    let walker = sim.object(ObjectId(1)).unwrap();
    assert_eq!(walker.pos, v(600.0, 300.0), "the walker got past");
    places
}

/// 0023, Verification: "Walk a unit through your own idle Peacekeepers with shields raised: they step aside (at
/// their shielded speed)." and "Through a deployed Artillery Rig: the mover paths around it."
///
/// TODO(Riot Shield, Artillery Rig): neither mode exists in the sim yet. Until they do, the modes are stood in for
/// by what they will set: the Riot Shield's speed factor of 0.5 (`factions/gdo/units.md > Peacekeeper`), and for a
/// unit that can't move the hook `Mover::can_move`, which a speed factor of 0 answers.
#[test]
fn a_slowed_unit_yields_at_its_speed_and_a_unit_that_cant_move_never_yields() {
    let start = v(400.0, 300.0);
    let steps = |places: &[Vec2]| -> Vec<f32> {
        let from = std::iter::once(&start).chain(places);
        from.zip(places).map(|(a, b)| a.distance(*b)).collect()
    };
    let fastest = |steps: &[f32]| steps.iter().copied().fold(0.0_f32, f32::max);
    let unshielded = steps(&idle_unit_in_a_mode(1.0));
    assert!((fastest(&unshielded) - 4.0).abs() < 1e-3, "{unshielded:?}");

    let shielded = idle_unit_in_a_mode(0.5);
    assert_ne!(*shielded.last().unwrap(), start, "it stepped aside");
    assert!((fastest(&steps(&shielded)) - 2.0).abs() < 1e-3, "at its shielded speed");

    let deployed = idle_unit_in_a_mode(0.0);
    assert!(deployed.iter().all(|p| *p == start), "a fixed obstacle");
}

/// The point the group of [`converging`] is ordered to.
const CONVERGE_ON: Vec2 = Vec2::new(390.0, 260.0);

/// Twelve Peacekeepers of player 1, 60 su apart in four columns of three (units 1 to 12), ordered in tick 2 to a
/// point inside their box; unit 13 of player 1 stands apart.
fn converging() -> TestSim {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting();
    for i in 0..12 {
        t.spawn_unit_at(
            1,
            PK,
            P1,
            v(300.0 + 60.0 * (i % 4) as f32, 200.0 + 60.0 * (i / 4) as f32),
        );
    }
    t.spawn_unit_at(1, PK, P1, v(100.0, 260.0))
        .order_at(2, P1, &ids(1..=12), move_to(CONVERGE_ON), false);
    t
}

/// 0027, Verification: "Converge 12 units on a point: they settle into a calm blob with no shuffling at the
/// centre." 0026, Verification: "the outer units stop within about 2 tiles of the point straight away, with no
/// re-route attempts." A unit that has arrived never moves again while the others come in, and the whole group
/// has arrived within a bounded time.
#[test]
fn a_converging_group_settles_without_shuffling_or_re_routing() {
    let mut t = converging();
    t.run(2);
    let mut arrived_at: Vec<Option<Vec2>> = vec![None; 12];
    // The farthest unit starts 108 su from the point: 27 ticks.
    while arrived_at.iter().any(Option::is_none) {
        assert!(t.sim().tick() < 2 + 27 + STUCK_TICKS, "not settled\n{}", t.ascii(None));
        t.step();
        for (i, at) in arrived_at.iter_mut().enumerate() {
            let id = i as u32 + 1;
            match at {
                Some(at) => assert_eq!(t.pos(id), *at, "#{id} moved after it arrived\n{}", t.ascii(None)),
                None if t.status(id) == MoveStatus::Arrived => *at = Some(t.pos(id)),
                None => assert_eq!(t.status(id), MoveStatus::Moving),
            }
        }
    }
    for (i, at) in arrived_at.iter().enumerate() {
        let id = i as u32 + 1;
        assert_eq!(t.mover(id).repaths_left, MAX_REPATHS, "#{id} tried to re-route");
        assert!(at.unwrap().distance(CONVERGE_ON) <= gu(OCCUPIED_ARRIVAL_GU), "#{id}");
    }
    assert_eq!(failures(&t), 0);
    // Calm: nobody moves once every unit has arrived.
    for _ in 0..100 {
        t.step();
        for (i, at) in arrived_at.iter().enumerate() {
            assert_eq!(Some(t.pos(i as u32 + 1)), *at);
        }
    }
}

/// 0027, Verification: "Then walk a different own unit through the blob: the blob's units step aside for it." The
/// exception holds only within the group order.
#[test]
fn a_settled_group_steps_aside_for_another_unit_of_its_owner() {
    let mut t = converging();
    run_until_done(&mut t, &ids(1..=12), 100);
    let settled: Vec<Vec2> = (1..=12).map(|id| t.pos(id)).collect();
    let goal = v(700.0, 260.0);
    let from = t.sim().tick() + 1;
    t.order_at(from, P1, &[ObjectId(13)], move_to(goal), false).run(1);
    // 600 su: 150 ticks.
    run_until_done(&mut t, &[ObjectId(13)], from + 150 + 60);
    assert_eq!(
        (t.pos(13), t.status(13)),
        (goal, MoveStatus::Arrived),
        "{}",
        t.ascii(None)
    );
    assert_eq!(t.mover(13).repaths_left, MAX_REPATHS);
    let stepped_aside = (1..=12).filter(|&id| t.pos(id) != settled[id as usize - 1]).count();
    assert!(stepped_aside >= 1, "{}", t.ascii(None));
}

/// 0024, Verification: "Box a unit in completely: it stops trying and its order completes after about 7 s." Eight
/// enemy units stand around it. It is stuck 24 ticks after the order, looks for another route after waits of 8, 16
/// and 32 ticks, and gives up 24 ticks after the third attempt: 104 ticks, 6.5 s.
#[test]
fn a_boxed_in_unit_gives_up_after_about_seven_seconds() {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting();
    let at = v(500.0, 300.0);
    t.spawn_unit_at(1, PK, P1, at);
    for (dx, dy) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
        t.spawn_unit_at(1, PK, P2, at + v(24.0 * dx as f32, 24.0 * dy as f32));
    }
    t.order_at(2, P1, &[ObjectId(1)], move_to(v(900.0, 300.0)), false)
        .run(2);
    let mut repaths = Vec::new();
    while t.status(1) == MoveStatus::Moving {
        assert!(t.sim().tick() < 200, "{}", t.ascii(None));
        let left = t.mover(1).repaths_left;
        t.step();
        if t.mover(1).repaths_left < left {
            repaths.push(t.sim().tick() - 2);
        }
    }
    let waits = REPATH_WAITS;
    let first = STUCK_TICKS + waits[0];
    assert_eq!(repaths, [first, first + waits[1], first + waits[1] + waits[2]]);
    assert_eq!(repaths, [32, 48, 80]);
    let gave_up = t.sim().tick() - 2;
    assert_eq!(gave_up, GIVE_UP_TICKS, "104 ticks: 6.5 s");
    assert_eq!((t.status(1), t.pos(1)), (MoveStatus::Failed, at));
    assert!(
        t.events()
            .contains(&(t.sim().tick(), SimEvent::MoveFailed { unit: ObjectId(1) }))
    );
    t.step();
    assert!(is_idle(&t, 1), "its order completed");
}

/// 0024, Verification: "Block a unit briefly (under 1.5 s): it continues without re-routing." An enemy unit plugs
/// a corridor and is taken away 20 ticks after the walker ran into it.
#[test]
fn a_unit_blocked_for_less_than_the_stuck_time_carries_on_without_re_routing() {
    let mut t = ScenarioBuilder::new(CORRIDOR).start();
    t.without_auto_targeting();
    let goal = v(900.0, 96.0);
    t.spawn_unit_at(1, PK, P1, v(100.0, 96.0))
        .spawn_unit_at(1, GUARD, P2, v(500.0, 96.0))
        .order_at(2, P1, &[ObjectId(1)], move_to(goal), false)
        .run(3);
    // 370 su to the Guard, whose box begins at x 482: the walker finds no way forward from about tick 94 on.
    while t.mover(1).blocked_ticks == 0 {
        assert!(t.sim().tick() < 110);
        t.step();
    }
    let blocked = t.sim().tick();
    assert!(t.pos(1).x > 460.0, "against the Guard: {}", t.pos(1));
    let wait = STUCK_TICKS - 4;
    t.debug_at(blocked + wait, DebugCommand::Remove { id: ObjectId(2) });
    // 430 su more: 108 ticks.
    let done = run_until_done(&mut t, &[ObjectId(1)], blocked + wait + 108 + 4);
    assert_eq!((t.pos(1), t.status(1)), (goal, MoveStatus::Arrived));
    assert_eq!(t.mover(1).repaths_left, MAX_REPATHS, "it never re-routed");
    assert!(done >= blocked + wait + 100, "it waited: tick {done}");
    assert_eq!(failures(&t), 0);
}

/// Determinism of yielding: the walk through an own idle group replays to the same state, and a save taken while
/// units are in the middle of their sidesteps (JSON round trip) continues exactly like the game it came from: the
/// sidestep is state, and the tick's requests carry nothing across ticks.
#[test]
fn a_save_in_the_middle_of_a_sidestep_continues_identically() {
    let mut t = walk_through_a_blob_of(P1);
    t.run(2);
    let sidestepping = |t: &TestSim| (1..=25).filter(|&id| t.mover(id).sidestep.is_some()).count();
    while sidestepping(&t) < 2 {
        assert!(t.sim().tick() < 150, "nobody sidesteps\n{}", t.ascii(None));
        t.step();
    }
    let json = serde_json::to_string(t.sim()).unwrap();
    let mut loaded: Sim = serde_json::from_str(&json).unwrap();
    assert_eq!(&loaded, t.sim());
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    for _ in 0..150 {
        t.step();
        loaded.step(&t.replay().inputs[t.sim().tick() as usize - 1]);
        assert_eq!(loaded.state_hash(), t.sim().state_hash(), "tick {}", t.sim().tick());
    }
    assert_eq!(&loaded, t.sim());
    assert_eq!(t.status(WALKER), MoveStatus::Arrived);
    // The same schedule again, and the replay, give the same state.
    let mut again = walk_through_a_blob_of(P1);
    again.run_until(t.sim().tick());
    assert_eq!(again.sim().state_hash(), t.sim().state_hash());
    assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
}

/// A block of 18 Peacekeepers (three columns of six, `spacing` su apart) around `centre`.
fn spawn_block(t: &mut TestSim, owner: PlayerId, centre: Vec2, spacing: f32) {
    for i in 0..18 {
        let offset = v(spacing * ((i / 6) as f32 - 1.0), spacing * ((i % 6) as f32 - 2.5));
        t.spawn_unit_at(1, PK, owner, centre + offset);
    }
}

/// How the moves of units `1..=n` ended: (arrived, failed, the tick the last one ended).
fn outcome(t: &mut TestSim, n: u32) -> (u32, u32, u32) {
    let done = run_until_done(t, &ids(1..=n), 1200);
    let count = |s: MoveStatus| (1..=n).filter(|&id| t.status(id) == s).count() as u32;
    (count(MoveStatus::Arrived), count(MoveStatus::Failed), done)
}

/// 0022, Verification: "Re-run the 18-vs-18 head-on": two blocks of 18 Peacekeepers, each ordered onto the other's
/// place, the second block player 1's or player 2's. Every move ends.
///
/// (G) Both blocks are moving, so nobody yields: the units seek lanes (`docs/movement.md > 6.3`). 56 su apart the
/// blocks pass through each other's gaps and every unit arrives. 40 su apart the gaps (16 su) are narrower than
/// a unit: the blocks mesh, spread from the outside in, and pass. Before lane seeking 14 of the 36 units gave up
/// there (18 with two owners); now none does, and the bound leaves room for tuning. The counts are printed for
/// the record (`--nocapture`).
#[test]
fn eighteen_meet_eighteen_head_on() {
    for spacing in [40.0, 56.0] {
        for other in [P1, P2] {
            let mut t = ScenarioBuilder::new(OPEN).start();
            t.without_auto_targeting();
            let (west, east) = (v(200.0, 384.0), v(824.0, 384.0));
            spawn_block(&mut t, P1, west, spacing);
            spawn_block(&mut t, other, east, spacing);
            t.order_at(2, P1, &ids(1..=18), move_to(east), false).order_at(
                2,
                other,
                &ids(19..=36),
                move_to(west),
                false,
            );
            let (arrived, failed, done) = outcome(&mut t, 36);
            println!(
                "18 v 18 head-on, {spacing} su apart, the second block player {}'s: {arrived} arrived, {failed} \
                 failed, done in tick {done}",
                other.0
            );
            assert_eq!(arrived + failed, 36);
            if spacing > 40.0 {
                assert_eq!(failed, 0);
            }
            assert!(failed <= 2, "{failed} of 36 gave up\n{}", t.ascii(None));
            // Nobody takes much longer than the walk itself: 624 su at 4 su per tick is 156 ticks.
            assert!(done <= 2 + 156 + 120, "done in tick {done}");
            assert_eq!(t.replay().run().state_hash(), t.sim().state_hash());
        }
    }
}

/// A group of 18 ordered through a standing block of 18 idle Peacekeepers as wide as itself. An enemy block never
/// moves, and much of the group gives up in front of it; the owner's own block makes way (0022), and the group
/// arrives.
///
/// (G) The idle block **parts** for the group and isn't shoved along: the movers walk between its rows, and the
/// idle units step straight across their way (`docs/movement.md > 6.6`). Before, the block was pushed ahead of the
/// group, its units 204 su on average and up to 373 su; now they end 12 su from their places on average and at
/// most 35 su. The bounds leave room for tuning.
#[test]
fn eighteen_walk_through_eighteen_idle() {
    let mut failed_by_owner = Vec::new();
    for owner in [P1, P2] {
        let mut t = ScenarioBuilder::new(OPEN).start();
        t.without_auto_targeting();
        spawn_block(&mut t, P1, v(200.0, 384.0), 40.0);
        spawn_block(&mut t, owner, v(512.0, 384.0), 40.0);
        t.order_at(2, P1, &ids(1..=18), move_to(v(824.0, 384.0)), false);
        let (arrived, failed, done) = outcome(&mut t, 18);
        println!(
            "18 through 18 idle units of player {}: {arrived} arrived, {failed} failed, done in tick {done}",
            owner.0
        );
        assert_eq!(arrived + failed, 18);
        let moved: Vec<f32> = (0..18)
            .map(|i| {
                let offset = v(40.0 * ((i / 6) as f32 - 1.0), 40.0 * ((i % 6) as f32 - 2.5));
                t.pos(19 + i).distance(v(512.0, 384.0) + offset)
            })
            .collect();
        let (farthest, mean) = (
            moved.iter().fold(0.0_f32, |a, b| a.max(*b)),
            moved.iter().sum::<f32>() / 18.0,
        );
        println!(
            "the idle units of player {} ended {mean:.0} su from their places on average, at most {farthest:.0} su",
            owner.0
        );
        if owner == P1 {
            assert!(
                farthest <= 96.0 && mean <= 40.0,
                "shoved along: {moved:?}\n{}",
                t.ascii(None)
            );
            assert!(moved.iter().any(|d| *d > 0.0), "nobody made way");
            for id in 19..=36 {
                assert_eq!((t.mover(id).sidestep, t.mover(id).velocity), (None, Vec2::ZERO));
            }
        } else {
            assert_eq!(farthest, 0.0, "enemies never yield");
        }
        failed_by_owner.push(failed);
    }
    assert!(
        failed_by_owner[0] <= 2 && failed_by_owner[0] < failed_by_owner[1],
        "{failed_by_owner:?}"
    );
}
