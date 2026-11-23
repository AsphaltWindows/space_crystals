//! Fights from start to finish (`systems/combat.md`, `ARCHITECTURE.md > Determinism`): a Peacekeeper against a
//! Guard is decided the same way on every run, a three-on-three saved mid-fight continues exactly as the running
//! game does, and random brawls keep the state consistent.

use sc_sim::combat::AttackPhase;
use sc_sim::testing::{self, ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, Object, ObjectId, ObjectKind, OrderKind, PlayerId, Rng, Sim, SimEvent, Target, Tick,
    TickInputs, Vec2,
};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// 12 × 8 tiles of open ground.
const MAP: &str = "
    ............
    ............
    ............
    ............
    ............
    ............
    ............
    ............
";

fn ids(ids: &[u32]) -> Vec<ObjectId> {
    ids.iter().map(|&i| ObjectId(i)).collect()
}

fn died(t: &TestSim) -> Vec<(Tick, ObjectId)> {
    t.events()
        .iter()
        .filter_map(|(tick, e)| match e {
            SimEvent::Died { id, .. } => Some((*tick, *id)),
            _ => None,
        })
        .collect()
}

/// A Peacekeeper (id 1, player 1) and a Guard (id 2, player 2), 160 su apart, ordered to attack each other in
/// tick 2.
fn duel() -> TestSim {
    let mut t = ScenarioBuilder::new(MAP).start();
    t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, Vec2::new(300.0, 250.0))
        .spawn_unit_at(1, ObjectKind::Guard, P2, Vec2::new(460.0, 250.0))
        .order_at(2, P1, &ids(&[1]), OrderKind::Attack(ObjectId(2)), false)
        .order_at(2, P2, &ids(&[2]), OrderKind::Attack(ObjectId(1)), false);
    t
}

/// The Guard wins: it takes the Peacekeeper's 50 HP in ten hits of 5, one every 8 ticks (the last in tick 76),
/// while the Peacekeeper lands five hits of 9 in that time, one every 17 ticks. Two runs agree tick by tick, and
/// the replay ends in the same state.
#[test]
fn peacekeeper_against_guard_ends_the_same_way_every_time() {
    let run = || {
        let mut t = duel();
        let hashes: Vec<u64> = (0..120).map(|_| t.step().sim().state_hash()).collect();
        (t, hashes)
    };
    let (first, first_hashes) = run();
    let (second, second_hashes) = run();
    assert_eq!(first_hashes, second_hashes);
    assert_eq!(first.events(), second.events());
    assert_eq!(first.sim(), second.sim());
    assert_eq!(died(&first), [(76, ObjectId(1))]);
    let guard = first.sim().object(ObjectId(2)).unwrap();
    assert_eq!(guard.hp, Some(35.0));
    assert!(
        guard.orders.as_ref().unwrap().is_idle(),
        "its order completed with the kill"
    );
    assert_eq!(first.replay().run().state_hash(), first.sim().state_hash());
}

/// Peacekeepers 1–3 (player 1) against Guards 4–6 (player 2), facing each other in two columns 160 su apart: all
/// in range of each other. The Peacekeepers focus the middle Guard, then the others; each Guard attacks the
/// Peacekeeper opposite, then the others. The same fight as the client's QA scenario.
fn three_on_three() -> TestSim {
    let mut t = ScenarioBuilder::new(MAP).start();
    for y in [200.0, 248.0, 296.0] {
        t.spawn_unit_at(1, ObjectKind::Peacekeeper, P1, Vec2::new(300.0, y));
    }
    for y in [200.0, 248.0, 296.0] {
        t.spawn_unit_at(1, ObjectKind::Guard, P2, Vec2::new(460.0, y));
    }
    let pks = ids(&[1, 2, 3]);
    for (target, queue) in [(5, false), (4, true), (6, true)] {
        t.order_at(2, P1, &pks, OrderKind::Attack(ObjectId(target)), queue);
    }
    for (guard, targets) in [(4, [1, 2, 3]), (5, [2, 1, 3]), (6, [3, 2, 1])] {
        for (i, target) in targets.into_iter().enumerate() {
            t.order_at(2, P2, &ids(&[guard]), OrderKind::Attack(ObjectId(target)), i > 0);
        }
    }
    t
}

/// Saved in tick 11, with the Guards about to fire and the Peacekeepers reloading, the game loads into the same
/// state and runs on tick by tick as the original does, events included, to the end of the fight: the middle Guard
/// falls in tick 38, Peacekeepers 1 and 3 in 76 and the last one in 100.
#[test]
fn a_fight_saved_mid_way_continues_the_same_after_loading() {
    let mut t = three_on_three();
    t.run_until(11);
    let phase = |sim: &Sim, id: u32| sim.object(ObjectId(id)).unwrap().attack.clone().unwrap().phase;
    assert_eq!(
        phase(t.sim(), 4),
        Some(AttackPhase::Firing {
            target: Some(ObjectId(1))
        })
    );
    assert_eq!(phase(t.sim(), 1), Some(AttackPhase::Reloading));

    let saved = serde_json::to_string(t.sim()).unwrap();
    let mut loaded: Sim = serde_json::from_str(&saved).unwrap();
    assert_eq!(&loaded, t.sim());
    assert_eq!(loaded.state_hash(), t.sim().state_hash());
    let mut running = t.sim().clone();
    let mut deaths = Vec::new();
    for _ in 0..150 {
        running.step(&TickInputs::default());
        loaded.step(&TickInputs::default());
        assert_eq!(loaded.state_hash(), running.state_hash(), "tick {}", running.tick());
        assert_eq!(loaded.events(), running.events(), "tick {}", running.tick());
        deaths.extend(running.events().iter().filter_map(|e| match e {
            SimEvent::Died { id, .. } => Some((running.tick(), id.0)),
            _ => None,
        }));
    }
    assert_eq!(deaths, [(38, 5), (76, 1), (76, 3), (100, 2)]);
    let hp = |id: u32| loaded.object(ObjectId(id)).and_then(|o| o.hp);
    assert_eq!((hp(4), hp(6)), (Some(17.0), Some(80.0)));
}

/// A random brawl: two jittered columns of eight Peacekeepers and eight Guards, 120 su apart, given random attack
/// orders (mostly on the nearest enemy), Stops, Holds and moves, some of them queued, while debug commands teleport
/// and remove units at random. After every tick no reference to a gone object is left
/// (`testing::dangling_references`), an aim is always at the engaged target, and HP stays within (0, MaxHP]. Two
/// runs agree tick by tick, and units die.
#[test]
fn random_brawls_keep_the_state_consistent_and_deterministic() {
    let run = |seed: u64| {
        let mut rng = Rng::from_seed(seed);
        let mut t = ScenarioBuilder::new(MAP).start();
        for i in 0..8 {
            for (kind, owner, x) in [(ObjectKind::Peacekeeper, P1, 310.0), (ObjectKind::Guard, P2, 430.0)] {
                let jitter = Vec2::new(rng.next_f32() * 30.0, rng.next_f32() * 4.0);
                t.spawn_unit_at(1, kind, owner, Vec2::new(x, 100.0 + 40.0 * i as f32) + jitter);
            }
        }
        t.step();
        assert_eq!(t.sim().objects().count(), 16);
        // The brawl begins: everyone attacks the nearest enemy.
        let units: Vec<(ObjectId, PlayerId, Vec2)> =
            t.sim().objects().map(|o| (o.id, o.owner.unwrap(), o.pos)).collect();
        for (id, owner, pos) in units {
            if let Some(enemy) = t.sim().nearest_object(pos, |o| o.owner != Some(owner)) {
                t.order_at(2, owner, &[id], OrderKind::Attack(enemy.id), false);
            }
        }
        let mut hashes = Vec::new();
        let mut deaths = 0;
        for tick in 2..=600 {
            // A command every third tick or so, so that attacks often run their course.
            if rng.below(3) == 0
                && let Some((player, command)) = random_command(&mut rng, t.sim())
            {
                t.at(tick, player, command);
            }
            t.step();
            check_consistency(t.sim());
            hashes.push(t.sim().state_hash());
            deaths += t
                .sim()
                .events()
                .iter()
                .filter(|e| matches!(e, SimEvent::Died { .. }))
                .count();
        }
        (hashes, deaths, t.sim().clone())
    };
    let (hashes, deaths, end) = run(7);
    assert!(deaths >= 6, "{deaths} deaths");
    let (again, _, end_again) = run(7);
    assert_eq!(hashes, again);
    assert_eq!(end, end_again);
}

/// A random command about one of the units that attack: mostly an attack on the nearest enemy.
fn random_command(rng: &mut Rng, sim: &Sim) -> Option<(PlayerId, Command)> {
    let units: Vec<&Object> = sim.objects().filter(|o| o.attack.is_some()).collect();
    let unit = *units.get(rng.below(units.len().max(1) as u32) as usize)?;
    let owner = unit.owner?;
    let enemies: Vec<ObjectId> = units.iter().filter(|o| o.owner != Some(owner)).map(|o| o.id).collect();
    let nearest = sim
        .nearest_object(unit.pos, |o| o.owner != Some(owner) && o.attack.is_some())
        .map(|o| o.id);
    let kind = match rng.below(10) {
        0..=4 => OrderKind::Attack(nearest?),
        5 => OrderKind::Attack(*enemies.get(rng.below(enemies.len().max(1) as u32) as usize)?),
        6 => OrderKind::Stop,
        7 => OrderKind::Hold,
        8 => OrderKind::Move(Target::Point(unit.pos + Vec2::new(rng.next_f32() * 80.0 - 40.0, 30.0))),
        _ => {
            let command = if rng.below(4) == 0 {
                DebugCommand::Remove { id: unit.id }
            } else {
                let offset = Vec2::new(rng.next_f32(), rng.next_f32()) * 120.0 - 60.0;
                DebugCommand::SetPosition {
                    id: unit.id,
                    pos: unit.pos + offset,
                }
            };
            return Some((P1, Command::Debug(command)));
        }
    };
    let queue = rng.below(4) == 0;
    Some((
        owner,
        Command::Order {
            units: vec![unit.id],
            kind,
            queue,
        },
    ))
}

fn check_consistency(sim: &Sim) {
    assert_eq!(testing::dangling_references(sim), []);
    for o in sim.objects() {
        let max_hp = o.def().max_hp.unwrap_or(0.0);
        assert!(o.hp.is_none_or(|hp| hp > 0.0 && hp <= max_hp), "{o:?}");
        if let Some(attack) = &o.attack
            && let Some(AttackPhase::Aiming { target }) = attack.phase
        {
            assert_eq!(attack.target, Some(target), "{o:?}");
        }
    }
}
