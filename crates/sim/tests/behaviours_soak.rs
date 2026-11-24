//! Soak of the behaviours: 120 units of two players fight on random maps under seeded random orders of every kind
//! (Move, Attack, AttackMove, Patrol, Hold, Stop; on points, on enemies, on their own units and on map resources) for
//! 1 500 ticks, and then without new orders until the orders in flight have ended. It checks invariants, which hold
//! whatever the tuning (speeds, ranges, damage, the movement's constants):
//! - after every tick: the movement invariants (`TestSim` checks them), no state refers to an object that died, and
//!   no unit stays idle in reach of a visible enemy while auto-targeting is on ([`idle_in_reach`]);
//! - every order ends, but those that last until the next one ([`running`]);
//! - the failure ratios stay under stated bounds: orders the gate refuses, moves that fail, and path searches per
//!   unit and tick (no behaviour plans afresh tick after tick);
//! - nothing panics, and the run is deterministic.
//!
//! How many shots and deaths there are is up to the tuning: the test prints them, and only asks that there was a
//! fight.

use std::collections::BTreeSet;

use sc_sim::elevation::{Elevation, sight_range};
use sc_sim::orders::Auto;
use sc_sim::testing::{self, ScenarioBuilder, TestSim};
use sc_sim::{
    Command, DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerCommand, PlayerId, Rng, SU_PER_GU, Shape,
    Sim, SimEvent, SpawnAt, Target, TilePos, Vec2, combat,
};

const SIDE: i32 = 40;
const UNITS: u32 = 120;
/// Ticks with random orders.
const TICKS: u32 = 1500;

/// A 40 × 40 map with about 6 % scattered water and cliff tiles and two walls with gaps.
fn random_map(rng: &mut Rng) -> String {
    let mut tiles = vec![vec!['.'; SIDE as usize]; SIDE as usize];
    for row in &mut tiles {
        for tile in row.iter_mut() {
            *tile = match rng.below(100) {
                0..=3 => '~',
                4..=5 => 'c',
                _ => '.',
            };
        }
    }
    for at in [13, 26] {
        let gap = rng.below(SIDE as u32 - 4) as usize;
        for (i, row) in tiles.iter_mut().enumerate() {
            row[at] = if (gap..gap + 3).contains(&i) { '.' } else { 'M' };
        }
    }
    tiles
        .into_iter()
        .map(|row| row.into_iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

fn random_point(rng: &mut Rng) -> Vec2 {
    Vec2::new(rng.next_f32(), rng.next_f32()) * (SIDE as f32 * SU_PER_GU)
}

/// The ticks after the last order within which every order that ends by itself must have ended: the time the
/// slowest unit takes to walk four sides of the map, more than the longest way around both walls (about three) and
/// the fights on it. Only a run that fails waits this long.
fn settle_ticks() -> u32 {
    let slowest = [ObjectKind::Guard, ObjectKind::Peacekeeper]
        .map(|kind| kind.def().unit.as_ref().expect("a unit").movement.max_speed())
        .into_iter()
        .fold(f32::INFINITY, f32::min);
    (4.0 * SIDE as f32 * SU_PER_GU / slowest) as u32
}

/// What one seed's run came to.
struct Outcome {
    hash: u64,
    deaths: usize,
    shots: usize,
    /// Orders the gate took, of those tried.
    taken: (u32, u32),
    /// The units the taken orders went to: an order to six units counts six.
    ordered: u32,
    /// Moves that failed (`SimEvent::MoveFailed`).
    failed: usize,
    /// A\* searches over the whole run.
    searches: u64,
    /// The times an idle unit acquired an enemy by itself.
    acquired: u32,
    /// The idle units in reach of an enemy whose engagement was checked in the next tick, and those excused because
    /// an enemy their scan took in died in that tick.
    checks: (u32, u32),
    /// The ticks after the last order until every order that ends by itself had ended; `None` if some never did.
    settled: Option<u32>,
    t: TestSim,
}

/// One seed: the units are spawned in tick 1, player 1's in the west and player 2's in the east, with a few map
/// resources between them; then every few ticks one player orders a run of its units, by what the sim holds then.
/// An order the gate refuses (a target out of sight, say) is part of the test. After [`TICKS`] nobody gives an
/// order, and the run goes on until every order that ends by itself has ended, or [`settle_ticks`] have passed.
fn soak(seed: u64) -> Outcome {
    let mut rng = Rng::from_seed(seed);
    let mut t = ScenarioBuilder::new(&random_map(&mut rng)).seed(seed).start();
    spawn(&mut t, &mut rng);
    t.step();
    assert_eq!(t.sim().objects().filter(|o| o.mover.is_some()).count(), UNITS as usize);

    let (mut next_order, mut taken, mut tried, mut ordered, mut searches) = (2, 0, 0, 0, 0_u64);
    let (mut engaging, mut acquired, mut checks) = (BTreeSet::new(), 0, (0, 0));
    let mut in_reach: Vec<(ObjectId, Vec<ObjectId>)> = Vec::new();
    let mut settled = None;
    while t.sim().tick() < TICKS + settle_ticks() {
        let tick = t.sim().tick() + 1;
        if tick <= TICKS && tick >= next_order {
            next_order = tick + 1 + rng.below(8);
            if let Some((player, command)) = random_order(&t, &mut rng) {
                tried += 1;
                let checked = PlayerCommand {
                    player,
                    command: command.clone(),
                };
                if t.sim().validate(&checked).is_ok()
                    && let Command::Order { units, .. } = &command
                {
                    taken += 1;
                    ordered += units.len() as u32;
                }
                t.at(tick, player, command);
            }
        }
        t.step();
        let sim = t.sim();
        searches += u64::from(sim.nav_counters().searches);
        assert_eq!(testing::dangling_references(sim), [], "seed {seed}, tick {tick}");
        for o in sim.objects() {
            let (Some(orders), Some(mover)) = (&o.orders, &o.mover) else {
                continue;
            };
            // A unit without an order has a goal only while it fights or walks back on its own.
            if orders.is_idle() && mover.status == MoveStatus::Moving {
                assert_ne!(orders.auto, Auto::default(), "seed {seed}: {o:?}");
            }
        }
        // An idle unit that ended the last tick in reach of an enemy has engaged one, unless an order came first.
        // Not checked for a unit one of whose scan's enemies died in this tick: the enemy it engaged in 5a may have
        // died in 5e, which leaves it watching again (the removal path), with the next enemy for the next tick.
        let died: BTreeSet<ObjectId> = sim
            .events()
            .iter()
            .filter_map(|e| match e {
                SimEvent::Died { id, .. } => Some(*id),
                _ => None,
            })
            .collect();
        for (id, scanned) in in_reach {
            if scanned.iter().any(|enemy| died.contains(enemy)) {
                checks.1 += 1;
                continue;
            }
            let Some(orders) = sim.object(id).and_then(|o| o.orders.as_ref()) else {
                continue;
            };
            checks.0 += 1;
            assert!(
                !orders.is_idle() || matches!(orders.auto, Auto::Engaging { .. }),
                "seed {seed}, tick {tick}: #{} stays idle in reach of an enemy: {:?}",
                id.0,
                sim.object(id)
            );
        }
        in_reach = idle_in_reach(sim);
        let now: BTreeSet<ObjectId> = sim
            .objects()
            .filter(|o| {
                o.orders
                    .as_ref()
                    .is_some_and(|orders| orders.is_idle() && matches!(orders.auto, Auto::Engaging { .. }))
            })
            .map(|o| o.id)
            .collect();
        acquired += now.difference(&engaging).count() as u32;
        engaging = now;
        if tick >= TICKS && running(sim).is_empty() {
            settled = Some(tick - TICKS);
            break;
        }
    }
    let count = |wanted: fn(&SimEvent) -> bool| t.events().iter().filter(|(_, e)| wanted(e)).count();
    Outcome {
        hash: t.sim().state_hash(),
        deaths: count(|e| matches!(e, SimEvent::Died { .. })),
        shots: count(|e| matches!(e, SimEvent::Attacked { .. })),
        taken: (taken, tried),
        ordered,
        failed: count(|e| matches!(e, SimEvent::MoveFailed { .. })),
        searches,
        acquired,
        checks,
        settled,
        t,
    }
}

/// [`UNITS`] Peacekeepers and Guards in rows, player 1's in the west and player 2's in the east, and six map
/// resources between them, all in tick 1.
fn spawn(t: &mut TestSim, rng: &mut Rng) {
    let mut spawned = 0;
    for cell in 0..20 * 20 {
        let kind = if rng.below(3) == 0 {
            ObjectKind::Guard
        } else {
            ObjectKind::Peacekeeper
        };
        let owner = PlayerId(1 + (cell % 2) as u8);
        // Player 1 on the western third, player 2 on the eastern.
        let column = (cell / 2 % 10) as f32 * 70.0 + if owner.0 == 1 { 80.0 } else { 1760.0 };
        let at = Vec2::new(column, (cell / 20) as f32 * 110.0 + 150.0);
        let jitter = Vec2::new(rng.below(30) as f32, rng.below(30) as f32);
        let spawn = PlayerCommand {
            player: PlayerId(1),
            command: Command::Debug(DebugCommand::Spawn {
                kind,
                owner: Some(owner),
                at: SpawnAt::Point(at + jitter),
                orientation: Default::default(),
            }),
        };
        if spawned < UNITS && t.sim().validate(&spawn).is_ok() {
            t.at(1, spawn.player, spawn.command);
            spawned += 1;
        }
    }
    for _ in 0..6 {
        let kind = [ObjectKind::CrystalPatch, ObjectKind::SupplyDeliveryStation][rng.below(2) as usize];
        let tile = TilePos::new(15 + rng.below(10) as i32, 2 + rng.below(36) as i32);
        t.debug_at(
            1,
            DebugCommand::Spawn {
                kind,
                owner: None,
                at: SpawnAt::Tile(tile),
                orientation: Default::default(),
            },
        );
    }
}

/// A random order of a random player to a run of its units, or `None` if it has none left.
fn random_order(t: &TestSim, rng: &mut Rng) -> Option<(PlayerId, Command)> {
    let player = PlayerId(1 + rng.below(2) as u8);
    let own: Vec<ObjectId> = t
        .sim()
        .objects()
        .filter(|o| o.owner == Some(player) && o.orders.is_some())
        .map(|o| o.id)
        .collect();
    if own.is_empty() {
        return None;
    }
    let all: Vec<ObjectId> = t.sim().objects().map(|o| o.id).collect();
    let count = [1, 1, 2, 6, 15, own.len()][rng.below(6) as usize].min(own.len());
    let first = rng.below((own.len() - count + 1) as u32) as usize;
    let units = own[first..first + count].to_vec();
    // An object target: mostly something the player sees, sometimes anything at all.
    let seen: Vec<ObjectId> = t
        .sim()
        .objects()
        .filter(|o| t.sim().is_visible(player, o))
        .map(|o| o.id)
        .collect();
    let pool = if rng.below(5) == 0 || seen.is_empty() {
        &all
    } else {
        &seen
    };
    let object = pool[rng.below(pool.len() as u32) as usize];
    let point = random_point(rng);
    let kind = match rng.below(12) {
        0 => OrderKind::Stop,
        1 => OrderKind::Hold,
        2 | 3 => OrderKind::Attack(object),
        4 => OrderKind::Move(Target::Object(object)),
        5 | 6 => OrderKind::Patrol(point),
        7..=9 => OrderKind::AttackMove(point),
        _ => OrderKind::Move(Target::Point(point)),
    };
    let queue = rng.below(4) == 0;
    Some((player, Command::Order { units, kind, queue }))
}

/// The idle units that end the tick watching (`systems/controls/behaviors.md > BaseAutoTargeting > Active during
/// Idle`) with an enemy in reach that they may pick: one their attack reaches where both stand, which their owner
/// sees (`combat::can_attack_now`), and which isn't the one they gave up on for the next tick
/// (`orders::GIVE_UP_RETRY_TICKS`). Nobody, while auto-targeting is off. Each comes with the enemies its scan
/// takes in ([`scanned`]): the one it engages in the next tick is among them.
///
/// An idle unit scans its SightRange, which takes in its Range, so in the next tick each of them engages an enemy,
/// unless an order comes first. Not yet in this one: phase 5a decides from where the units stood when the tick
/// began, so an enemy that walked into range in 5d, or a unit whose order ended in this tick, waits for the next.
/// Returning units don't scan (`> Not active during: Move`), and engaging ones are busy.
fn idle_in_reach(sim: &Sim) -> Vec<(ObjectId, Vec<ObjectId>)> {
    let next = sim.tick() + 1;
    sim.objects()
        .filter(|o| {
            let (Some(orders), Some(owner)) = (&o.orders, o.owner) else {
                return false;
            };
            let Auto::Watching { ignored } = orders.auto else {
                return false;
            };
            sim.auto_targeting()
                && orders.is_idle()
                && sim.objects().any(|e| {
                    e.owner.is_some_and(|other| other != owner)
                        && ignored.is_none_or(|(id, until)| id != e.id || next >= until)
                        && combat::can_attack_now(sim.map(), sim.vision(), o, e)
                })
        })
        .map(|o| (o.id, scanned(sim, o)))
        .collect()
}

/// The enemies an idle unit's scan in the next tick takes in, of which it engages the best (`orders::world::World::
/// pick` with the sight scan): the other players' destructible objects whose shapes lie within its SightRange, edge
/// to edge, with the ElevationModifier, where everything stands as the tick ends, which is where phase 5a of the
/// next tick sees them. More than it may pick, as neither sight nor domain is checked: only a death among them
/// excuses the unit.
fn scanned(sim: &Sim, unit: &sc_sim::Object) -> Vec<ObjectId> {
    let (map, shape) = (sim.map(), Shape::of(unit));
    let source = Elevation::of_object(map, unit);
    sim.objects()
        .filter(|e| {
            let reach = sight_range(unit.def().sight_range, source, Elevation::of_object(map, e));
            e.owner.is_some_and(|other| Some(other) != unit.owner)
                && e.hp.is_some()
                && shape.gap(&Shape::of(e)) <= sc_sim::gu(reach)
        })
        .map(|e| e.id)
        .collect()
}

/// The units with an order that hasn't ended, and what it is (`None`: one waits in the queue), but for those that
/// last until the next order: Hold Position, Patrol and a Move to a unit, which follows it (`systems/controls/
/// behaviors.md > MovingToObject`). Every other order ends: it completes, or it fails.
fn running(sim: &Sim) -> Vec<(ObjectId, Option<OrderKind>)> {
    sim.objects()
        .filter_map(|o| {
            let orders = o.orders.as_ref()?;
            match &orders.current {
                Some(current) => {
                    let lasts = match current.order.kind {
                        OrderKind::Hold | OrderKind::Patrol(_) => true,
                        OrderKind::Move(Target::Object(id)) => sim.object(id).is_some_and(|o| o.mover.is_some()),
                        _ => false,
                    };
                    (!lasts).then_some((o.id, Some(current.order.kind)))
                }
                None => (!orders.queue.is_empty()).then_some((o.id, None)),
            }
        })
        .collect()
}

#[test]
fn units_fighting_under_random_orders_keep_the_invariants() {
    let (mut acquired, mut checked) = (0, 0);
    // Seed 1 ran into an AttackMove that never ended (after design change 0168): a unit acquired an enemy at the edge
    // of its owner's sight, its path to it led out of that sight, so it disengaged, walked back and acquired it again,
    // every other tick. A target the unit let go of is now left alone until it has passed it (`orders::travel::march`;
    // `tests/attack_move_leash.rs` has the loop on its own).
    for seed in [1, 2, 3, 4] {
        let run = soak(seed);
        check(seed, &run);
        acquired += run.acquired;
        checked += run.checks.0;
        if seed == 2 {
            assert_eq!(soak(seed).hash, run.hash, "the same seed again");
            assert_eq!(run.t.replay().run().state_hash(), run.hash, "and its replay");
        }
    }
    // Idle units met enemies, so the check that they engage them did check something.
    assert!(acquired > 0 && checked > 0);
}

/// What every seed's run must come to.
fn check(seed: u64, run: &Outcome) {
    let alive = run.t.sim().objects().filter(|o| o.mover.is_some()).count();
    let ticks = run.t.sim().tick();
    println!(
        "seed {seed}: {} shots, {} deaths, {alive} units left; {} enemies acquired by idle units, {} idle units in \
         reach checked, {} excused by a death; {} of {} orders taken, for {} units; {} moves failed; {} path searches \
         in {ticks} ticks; every order ended {:?} ticks after the last",
        run.shots,
        run.deaths,
        run.acquired,
        run.checks.0,
        run.checks.1,
        run.taken.0,
        run.taken.1,
        run.ordered,
        run.failed,
        run.searches,
        run.settled
    );
    // Every order ends, but those that last until the next one.
    assert!(
        run.settled.is_some(),
        "seed {seed}: orders still running: {:?}",
        running(run.t.sim())
    );
    // It was a fight.
    assert!(run.shots > 0 && run.deaths > 0, "seed {seed}");
    // The gate refuses fewer than half of the orders (a target out of sight, say).
    assert!(run.taken.0 * 2 > run.taken.1, "seed {seed}: {:?}", run.taken);
    // Fewer than one in ten of the units ordered fails a move.
    assert!(
        run.failed * 10 < run.ordered as usize,
        "seed {seed}: {} of {} failed",
        run.failed,
        run.ordered
    );
    // No behaviour plans afresh every tick: on average a unit asks for a path far less than once in 10 ticks.
    assert!(
        run.searches < u64::from(UNITS * ticks / 10),
        "seed {seed}: {} searches",
        run.searches
    );
}

/// The soak over seeds 1 to 40, each run on its own, its outcome printed: which seeds fail, and how. Ignored: about
/// ten times the soak's own time. `cargo test -p sc_sim --release --test behaviours_soak -- --ignored --nocapture`.
#[test]
#[ignore]
fn soak_sweep() {
    let mut failed = Vec::new();
    for seed in 1..=40 {
        let outcome = std::panic::catch_unwind(|| check(seed, &soak(seed)));
        if let Err(cause) = outcome {
            let cause = (cause.downcast_ref::<String>().cloned())
                .or_else(|| cause.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default();
            println!("seed {seed} FAILED: {}", cause.lines().next().unwrap_or_default());
            failed.push(seed);
        }
    }
    println!("failed: {failed:?}");
}
