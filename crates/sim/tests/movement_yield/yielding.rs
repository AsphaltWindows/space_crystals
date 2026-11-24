//! Who yields (design changes 0022 and 0023): an idle unit of the walker's owner steps aside; an enemy, a
//! moving unit, a unit busy where it stands and one that can't move don't; a shielded one does, at its speed.
//! A save in the middle of a sidestep continues identically.

use super::*;

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

/// The mode of the idle unit of [`idle_unit_in_a_mode`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mode {
    /// None: it moves at its full speed.
    Normal,
    /// Its Riot Shield raised in tick 1 (`factions/gdo/units.md > Peacekeeper > Riot Shield`): half speed.
    Shielded,
    /// A mode it can't move in, stood in for by a save edited in tick 1 to a `Mover::speed_factor` of 0.
    Fixed,
}

/// Unit 1 (player 1) is ordered east through unit 2 (an idle Peacekeeper of player 1, exactly in its way, in `mode`)
/// in tick 2, the game having been saved and loaded in tick 1. Returns unit 2's positions after each tick up to tick
/// 120; the invariants are checked on the way.
fn idle_unit_in_a_mode(mode: Mode) -> Vec<Vec2> {
    let mut t = ScenarioBuilder::new(OPEN).start();
    t.without_auto_targeting();
    t.spawn_unit_at(1, PK, P1, v(300.0, 300.0))
        .spawn_unit_at(1, PK, P1, v(400.0, 300.0));
    if mode == Mode::Shielded {
        t.ability_at(1, P1, &[ObjectId(2)], Ability::RiotShield { raise: true });
    }
    t.run(1);
    let mut save = serde_json::to_value(t.sim()).unwrap();
    if mode == Mode::Fixed {
        let unit = &mut save["objects"][1];
        assert_eq!(unit["id"], 2);
        unit["mover"]["speed_factor"] = 0.0.into();
    }
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
    let idle = sim.object(ObjectId(2)).unwrap();
    assert!(idle.orders.as_ref().unwrap().is_idle());
    let shield = if mode == Mode::Shielded {
        sc_sim::ShieldMode::Raised
    } else {
        sc_sim::ShieldMode::Lowered
    };
    assert_eq!(sc_sim::riot_shield::mode(idle), Some(shield), "a mode, not a task");
    places
}

/// 0023, Verification: "Walk a unit through your own idle Peacekeepers with shields raised: they step aside (at
/// their shielded speed)." and "Through a deployed Artillery Rig: the mover paths around it." The shield stays up
/// all the while.
///
/// TODO(Artillery Rig): the deployed mode doesn't exist in the sim yet. Until it does, it is stood in for by what it
/// will set: the hook for a unit that can't move, `Mover::can_move`, which a speed factor of 0 answers.
#[test]
fn a_shielded_unit_yields_at_its_speed_and_a_unit_that_cant_move_never_yields() {
    let start = v(400.0, 300.0);
    let steps = |places: &[Vec2]| -> Vec<f32> {
        let from = std::iter::once(&start).chain(places);
        from.zip(places).map(|(a, b)| a.distance(*b)).collect()
    };
    let fastest = |steps: &[f32]| steps.iter().copied().fold(0.0_f32, f32::max);
    let unshielded = steps(&idle_unit_in_a_mode(Mode::Normal));
    assert!((fastest(&unshielded) - 4.0).abs() < 1e-3, "{unshielded:?}");

    let shielded = idle_unit_in_a_mode(Mode::Shielded);
    assert_ne!(*shielded.last().unwrap(), start, "it stepped aside");
    assert!((fastest(&steps(&shielded)) - 2.0).abs() < 1e-3, "at its shielded speed");

    let deployed = idle_unit_in_a_mode(Mode::Fixed);
    assert!(deployed.iter().all(|p| *p == start), "a fixed obstacle");
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
