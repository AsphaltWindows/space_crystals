//! What ends a channel and what starts one (`super`; `orders/shroud.rs`): "**What ends the channel** (user, 2026-11-26,
//! corrected): only the Disciple's **death**, a **cancel** command (Stop or any other order), or running **out of
//! energy**. Taking damage does **not** end it"; "drains **5 energy per second**; its **own regeneration continues
//! while channelling** (user, 2026-11-26; Q0113), so the net drain is 4.5/s, about 44 s from full; needs at least **10
//! energy** to start" (`factions/cults/units/disciple.md > Placeholder stats > Shroud numbers`).

use sc_sim::testing::invariants::Invariants;
use sc_sim::{Command, PlayerCommand, Reject, Sim, Target, TickInputs};

use super::*;

/// A point of energy in `Object::energy`'s units, and what the Disciple's 0.5 a second adds in a tick.
const ONE: u32 = sc_sim::energy::ONE;
const A_TICK: u32 = 2048;

fn shroud(queue: bool) -> PlayerCommand {
    PlayerCommand {
        player: P1,
        command: Command::Order {
            units: vec![ObjectId(1)],
            kind: OrderKind::Shroud,
            queue,
        },
    }
}

fn energy(sim: &Sim) -> u32 {
    sim.object(ObjectId(1)).and_then(|o| o.energy).expect("a pool")
}

/// The Disciple (1) under its own cloud, shot by player 2's Militant (2) from outside it, takes the releases' shares
/// and channels on; a Move ends its channel in the tick it is given. The Militant fires every 8 ticks, so a shot lands
/// in each release's tick, before it, and goes with it: a release every 16 ticks, two shots in each.
#[test]
fn damage_doesnt_end_the_channel_and_a_new_order_does() {
    let mut t = quiet(
        flat(20, 14)
            .spawn(DISCIPLE, Some(P1), point(400.0, 400.0))
            .spawn(MILITANT, Some(P2), point(400.0, 620.0))
            .start(),
    );
    order(&mut t, 1, P1, &[1], OrderKind::Shroud);
    order(&mut t, 2, P2, &[2], OrderKind::Attack(ObjectId(1)));
    step_until(&mut t, 100, "three releases on the Disciple", |t| {
        hits_on(t, 1).len() == 3
    });
    assert!(
        hits_on(&t, 1).iter().all(|&(_, by, damage)| (by, damage) == (1, 0.5)),
        "{:?}",
        hits_on(&t, 1)
    );
    let (first, second) = (releases(&t)[0].clone(), releases(&t)[1].clone());
    assert_eq!(
        (second.0 - first.0, first.2, second.2),
        (16, 8.0, 8.0),
        "{:?}",
        releases(&t)
    );
    assert_eq!(clouds(&t), [vec![1]], "it channels on");
    let ended = |t: &TestSim| {
        (t.events().iter())
            .find(|(_, e)| matches!(e, SimEvent::ShroudEnded { .. }))
            .map(|e| e.0)
    };
    assert_eq!(ended(&t), None);
    let moved = t.sim().tick() + 1;
    order(
        &mut t,
        moved,
        P1,
        &[1],
        OrderKind::Move(Target::Point(Vec2::new(400.0, 200.0))),
    );
    t.step();
    assert_eq!(
        (ended(&t), clouds(&t)),
        (Some(moved), vec![]),
        "the Move ended it at once"
    );
}

/// From full, 200 energy (a save edited to it), the channel pays its drain of 20,480 units a tick, its regeneration of
/// 2,048 going on, 711 ticks (`orders/shroud.rs`), and ends in the next, when its energy (2,048 units, 1/32 of a point)
/// can't pay one: 711 ticks, 44.4 s, after it started, "about 44 s from full". Then the gate refuses another below 10
/// energy (`Reject::NotEnoughEnergy`), and takes it from the tick its regeneration reaches 10 exactly, 318 ticks on.
#[test]
fn from_full_a_channel_runs_dry_711_ticks_after_it_starts_and_another_needs_10_energy() {
    let t = flat(20, 10).spawn(DISCIPLE, Some(P1), point(300.0, 300.0)).start();
    let mut json = serde_json::to_value(t.sim()).unwrap();
    json["objects"][0]["energy"] = serde_json::json!(200 * ONE);
    let mut sim: Sim = serde_json::from_value(json).unwrap();
    let mut rules = Invariants::new(&sim);
    let mut step = |sim: &mut Sim, commands: Vec<PlayerCommand>| {
        let inputs = TickInputs { commands };
        sim.step(&inputs);
        rules.check(sim, &inputs).unwrap();
    };
    step(&mut sim, vec![shroud(false)]);
    let start = sim.tick();
    assert_eq!(
        energy(&sim),
        200 * ONE - 20_480 + A_TICK,
        "a tick's drain, and its regeneration"
    );
    let ended = |sim: &Sim| (sim.events().iter()).any(|e| matches!(e, SimEvent::ShroudEnded { .. }));
    while !ended(&sim) {
        assert!(sim.tick() < start + 800, "it runs dry");
        assert_eq!(sim.clouds().list().len(), 1, "tick {}", sim.tick());
        step(&mut sim, Vec::new());
    }
    assert_eq!(sim.tick(), start + 711, "711 ticks after it started");
    assert_eq!(energy(&sim), 2 * A_TICK, "2,048 left, and the tick's regeneration");
    assert!(sim.clouds().is_empty());
    let mut accepted = None;
    while accepted.is_none() {
        let ready = energy(&sim) >= 10 * ONE;
        let expected = if ready { Ok(()) } else { Err(Reject::NotEnoughEnergy) };
        assert_eq!(sim.validate(&shroud(false)), expected, "{} units", energy(&sim));
        if ready {
            accepted = Some(sim.tick());
        } else {
            step(&mut sim, Vec::new());
        }
    }
    assert_eq!(
        (accepted, energy(&sim)),
        (Some(start + 711 + 318), 10 * ONE),
        "10 exactly"
    );
}

/// "needs at least **10 energy** to start", for an order queued behind another: the gate takes a Shift-queued Shroud
/// behind one under way while the Disciple has its 50, and the first runs dry 177 ticks on, with 14,336 units left; the
/// queued one starts then, below 10, and ends in the same tick (`orders/shroud.rs`): no second channel, and the
/// Disciple idle.
#[test]
fn a_queued_shroud_that_starts_below_10_energy_ends_at_once() {
    let mut t = flat(20, 10).spawn(DISCIPLE, Some(P1), point(300.0, 300.0)).start();
    order(&mut t, 1, P1, &[1], OrderKind::Shroud);
    t.order_at(1, P1, &[ObjectId(1)], OrderKind::Shroud, true);
    t.step();
    assert_eq!(
        t.sim()
            .object(ObjectId(1))
            .unwrap()
            .orders
            .as_ref()
            .unwrap()
            .queue
            .len(),
        1,
        "queued"
    );
    let ended = |t: &TestSim| {
        (t.events().iter())
            .find(|(_, e)| matches!(e, SimEvent::ShroudEnded { .. }))
            .map(|e| e.0)
    };
    step_until(&mut t, 400, "the first runs dry", |t| ended(t).is_some());
    assert_eq!(ended(&t), Some(178), "177 ticks paid, from tick 1");
    let started: Vec<Tick> = (t.events().iter())
        .filter(|(_, e)| matches!(e, SimEvent::ShroudStarted { .. }))
        .map(|e| e.0)
        .collect();
    assert_eq!(started, [1], "the queued one never channels");
    let orders = t.sim().object(ObjectId(1)).unwrap().orders.clone().unwrap();
    assert!(orders.is_idle(), "{orders:?}");
    assert_eq!(energy(t.sim()), 14_336 + A_TICK);
    assert!(t.sim().clouds().is_empty());
}

/// Design change 0438's Verification: "A Disciple channelling Shroud, Shift+Move: it keeps channelling until out of
/// energy, then moves. A plain right-click ends the channel at once" (`systems/controls/commands.md > Unit Command`: a
/// Shift-queued command is appended to the queue and never replaces the current one). From its 50 the Disciple
/// channels from tick 1; a Move Shift-queued in tick 20 waits behind the channel, which runs dry in tick 178 as it does
/// without one (the test above), the Disciple standing where it was; then the Move starts and takes it to its point.
/// The same game with the Move given without Shift ends the channel in its tick, 20, and the Disciple walks off at once.
#[test]
fn a_shift_queued_move_waits_behind_the_channel_and_a_plain_one_ends_it_at_once() {
    let (from, to) = (Vec2::new(300.0, 300.0), Vec2::new(300.0, 500.0));
    let game = |queue: bool| {
        let mut t = flat(20, 10).spawn(DISCIPLE, Some(P1), point(from.x, from.y)).start();
        order(&mut t, 1, P1, &[1], OrderKind::Shroud);
        t.order_at(20, P1, &[ObjectId(1)], OrderKind::Move(Target::Point(to)), queue);
        t
    };
    let ended = |t: &TestSim| {
        (t.events().iter())
            .find(|(_, e)| matches!(e, SimEvent::ShroudEnded { .. }))
            .map(|e| e.0)
    };

    let mut t = game(true);
    t.run_until(20);
    let orders = t.sim().object(ObjectId(1)).unwrap().orders.clone().unwrap();
    assert_eq!(orders.queue.len(), 1, "queued behind the channel: {orders:?}");
    t.run_until(177);
    assert_eq!((ended(&t), clouds(&t)), (None, vec![vec![1]]), "it channels on");
    assert_eq!(t.pos(1), from, "standing");
    step_until(&mut t, 10, "it runs dry", |t| ended(t).is_some());
    assert_eq!(ended(&t), Some(178), "out of energy, as without the Move");
    step_until(&mut t, 200, "then it moves there", |t| t.pos(1).distance(to) < 1.0);

    let mut t = game(false);
    t.run_until(20);
    assert_eq!((ended(&t), clouds(&t)), (Some(20), vec![]), "the Move ended it at once");
    t.run_until(30);
    assert!(t.pos(1).y > from.y + 20.0, "walking off: {}", t.pos(1));
}
