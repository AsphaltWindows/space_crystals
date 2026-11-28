//! Timings of a whole sim tick with 500 moving units on a 256 × 256 map, against the targets of
//! `docs/movement.md > 5.8`. Ignored by default:
//!
//! ```sh
//! cargo test -p sc_sim --release --test perf movement_perf -- --ignored --nocapture
//! ```
//!
//! The assertions are on the deterministic counters only, never on wall time; `movement_collision/` asserts the
//! same counters in the normal test run. (M6-SB) `perf_500_units_on_256_through_four_trails` is the same game with
//! four Shadebringers' sludge trails across both armies' way, for an A/B against the plain one, and
//! `perf_500_units_on_256_beside_four_shadebringers` the same Shadebringers with their Sludge off: the difference of
//! the two is the trails' cost with what their slow does to both armies' moves, and the first also times the slow's
//! lookups alone, every unit's where it stands.

use std::time::Duration;

use sc_sim::defs::PatchKind;
use sc_sim::testing::{ScenarioBuilder, TestSim};
use sc_sim::{Ability, DebugCommand, MoveStatus, ObjectId, ObjectKind, OrderKind, PlayerId, Rng, Target, Tick, Vec2};

const SIDE: usize = 256;
const UNITS: u32 = 500;

/// Wall-clock time of `work`. Only this test reads a clock: it measures the sim from outside, and nothing in the
/// sim can call it (`ARCHITECTURE.md > Determinism`).
#[expect(
    clippy::disallowed_methods,
    reason = "a perf test has to read the clock; the sim itself never does"
)]
fn timed<T>(work: impl FnOnce() -> T) -> (T, Duration) {
    let start = std::time::Instant::now();
    let result = work();
    (result, start.elapsed())
}

/// 256 × 256 with 5 % scattered cliffs; the two armies' fields are clear.
fn map(rng: &mut Rng) -> String {
    let mut rows = Vec::with_capacity(SIDE);
    for y in 0..SIDE {
        let row: String = (0..SIDE)
            .map(|x| {
                let field = (8..40).contains(&y) && ((8..40).contains(&x) || (216..248).contains(&x));
                if !field && rng.below(100) < 5 { 'c' } else { '.' }
            })
            .collect();
        rows.push(row);
    }
    rows.join("\n")
}

/// 250 units of each player in two blocks, 13 000 su apart, each ordered to the other's field in one group
/// order, so they also meet head-on half way.
fn armies() -> TestSim {
    let mut rng = Rng::from_seed(11);
    let mut t = ScenarioBuilder::new(&map(&mut rng)).start();
    t.check_invariants(false);
    for i in 0..UNITS / 2 {
        let offset = Vec2::new((i % 25) as f32, (i / 25) as f32) * 48.0;
        let kind = if i % 5 == 0 {
            ObjectKind::Guard
        } else {
            ObjectKind::Peacekeeper
        };
        t.spawn_unit_at(1, kind, PlayerId(1), Vec2::new(700.0, 700.0) + offset);
        t.spawn_unit_at(1, kind, PlayerId(2), Vec2::new(14_000.0, 700.0) + offset);
    }
    let (west, east): (Vec<ObjectId>, Vec<ObjectId>) = (1..=UNITS).map(ObjectId).partition(|id| id.0 % 2 == 1);
    let to = |x: f32| OrderKind::Move(Target::Point(Vec2::new(x, 1300.0)));
    t.order_at(2, PlayerId(1), &west, to(14_600.0), false)
        .order_at(2, PlayerId(2), &east, to(1300.0), false);
    t
}

/// (M6-SB) Where the four Shadebringers of [`trails`] lay theirs, x, across both armies' way (y 700 to 1300), from the
/// north: each drives south from y 600 towards y 1500.
const TRAILS_X: [f32; 4] = [3000.0, 6000.0, 9000.0, 12000.0];
/// A wave of them lays its trails for its tanks' 10 s, then the next takes over from where they began; the first
/// comes in tick 20, after the armies' two fields.
const WAVE: Tick = 160;
const FIRST_WAVE: Tick = 20;

/// (M6-SB) Before a tick of [`perf_500_units_on_256_through_four_trails`]: from tick 20, every [`WAVE`] ticks, the
/// Shadebringers out (their tanks empty) leave the game and four new ones come in at the north ends of their lines
/// ([`TRAILS_X`]), which in the tick after switch their Sludge on, if `sludge`, and drive south across both armies'
/// way: four trails on the ground from then on, 48 su wide, each patch lasting 10 s, slowing the Peacekeepers and
/// Guards that cross them.
fn trails(t: &mut TestSim, sludge: bool) {
    let next = t.sim().tick() + 1;
    if next < FIRST_WAVE {
        return;
    }
    let out: Vec<ObjectId> = (t.sim().objects())
        .filter(|o| o.kind == ObjectKind::Shadebringer)
        .map(|o| o.id)
        .collect();
    match (next - FIRST_WAVE) % WAVE {
        0 => {
            for &id in &out {
                t.debug_at(next, DebugCommand::Remove { id });
            }
            for x in TRAILS_X {
                t.spawn_unit_at(next, ObjectKind::Shadebringer, PlayerId(1), Vec2::new(x, 600.0));
            }
        }
        1 => {
            if sludge {
                t.ability_at(next, PlayerId(1), &out, Ability::Sludge { on: true });
            }
            for id in out {
                let x = t.sim().object(id).map_or(0.0, |o| o.pos.x);
                let south = OrderKind::Move(Target::Point(Vec2::new(x, 1500.0)));
                t.order_at(next, PlayerId(1), &[id], south, false);
            }
        }
        _ => {}
    }
}

#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn perf_500_units_on_256() {
    let mut t = armies();
    measure(&mut t, |_| {});
}

/// (M6-SB) [`perf_500_units_on_256`] through four Shadebringers' trails ([`trails`]): what the patches cost, every
/// unit's lookup of them where it stands (`sc_sim::patches::slow`), the slow of those in them and the laying
/// (`sc_sim::patches::lay`), A/B against the plain game, on master's build and on this one. The lookups are timed
/// alone too, each tick's again from outside the sim (`Patches::covers` for every unit with a mover).
#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn perf_500_units_on_256_through_four_trails() {
    let mut t = armies();
    let (mut patches, mut slowed, mut ticks, mut lookups) = (0_usize, 0_usize, 0_usize, Duration::ZERO);
    measure(&mut t, |t| {
        trails(t, true);
        if t.sim().tick() > FIRST_WAVE + WAVE {
            let sim = t.sim();
            patches += sim.patches().len();
            slowed += sim.objects().filter(|o| sc_sim::patches::slowed(o)).count();
            let (_, took) = timed(|| {
                (sim.objects())
                    .filter(|o| o.mover.is_some() && sim.patches().covers(PatchKind::Sludge, o.pos))
                    .count()
            });
            lookups += took;
            ticks += 1;
        }
    });
    assert!(
        ticks > 0 && patches / ticks > 4 * 20,
        "four trails on the ground: {patches} over {ticks} ticks"
    );
    println!(
        "from tick {}: a mean of {:.0} patches on the ground and {:.1} units slowed; every unit's lookup, timed alone, \
         {:.1} µs a tick",
        FIRST_WAVE + WAVE,
        patches as f64 / ticks as f64,
        slowed as f64 / ticks as f64,
        lookups.as_secs_f64() * 1e6 / ticks as f64
    );
}

/// (M6-SB) [`perf_500_units_on_256_through_four_trails`]'s Shadebringers, their Sludge off: the game it is beside
/// the plain one, for the trails' own cost.
#[test]
#[ignore = "prints timings; run in release with --ignored --nocapture"]
fn perf_500_units_on_256_beside_four_shadebringers() {
    let mut t = armies();
    measure(&mut t, |t| trails(t, false));
    assert!(t.sim().patches().is_empty());
}

/// The game `t` played and timed (module docs): its two group orders' ticks, then every tick with 450 or more units
/// moving, printed; `before` is called before each tick, for the commands of the game's own (untimed).
fn measure(t: &mut TestSim, mut before: impl FnMut(&mut TestSim)) {
    t.step();
    assert_eq!(t.sim().objects().count(), UNITS as usize);

    // The two group orders: one field each (`docs/movement.md > 5.5`). (G) A field reaches only as far as its
    // group stands, so both may fit into the budget of one tick; a field that is more than the budget leaves the
    // second group to the next tick ("budget plus one field" is the worst tick of 5.8).
    let mut fields = 0;
    while fields < 2 {
        before(t);
        let ((), took) = timed(|| {
            t.step();
        });
        let nav = t.sim().nav_counters();
        assert!(nav.fields >= 1 && nav.searches == 0, "{nav:?}");
        assert!(nav.expansions <= nav.fields * (SIDE * SIDE) as u32);
        fields += nav.fields;
        println!(
            "order tick ({} field(s), {} expansions, {} paths): {:.2} ms",
            nav.fields,
            nav.expansions,
            nav.fields * UNITS / 2,
            took.as_secs_f64() * 1e3
        );
    }

    let (mut total, mut worst, mut ticks) = (Duration::ZERO, Duration::ZERO, 0_u32);
    let (mut sweeps, mut pairs, mut steered, mut searches) = (0_u64, 0_u64, 0_u64, 0_u32);
    let mut worst_expansions = 0;
    while ticks < 4000 {
        before(t);
        let ((), took) = timed(|| {
            t.step();
        });
        let moves = t.sim().move_counters();
        // `movement_collision/` asserts the same bounds in the normal test run.
        assert!(moves.unit_sweeps <= 8 * moves.steered && moves.tile_sweeps <= 8 * moves.steered);
        assert!(moves.pair_tests <= 24 * moves.unit_sweeps);
        if moves.steered >= UNITS * 9 / 10 {
            total += took;
            worst = worst.max(took);
            ticks += 1;
            sweeps += u64::from(moves.unit_sweeps + moves.tile_sweeps);
            pairs += u64::from(moves.pair_tests);
            steered += u64::from(moves.steered);
        } else if t.sim().tick() > 100 {
            break;
        }
        let nav = t.sim().nav_counters();
        searches += nav.searches;
        worst_expansions = worst_expansions.max(nav.expansions);
    }
    assert!(ticks > 300, "only {ticks} ticks with 450 or more units moving");
    let per = |n: u64| n as f64 / steered as f64;
    println!(
        "{ticks} ticks with >= 450 of 500 units moving: mean {:.3} ms, worst {:.3} ms per whole sim tick",
        total.as_secs_f64() * 1e3 / f64::from(ticks),
        worst.as_secs_f64() * 1e3
    );
    println!(
        "per moving unit and tick: {:.2} sweeps, {:.2} box tests; {searches} repath searches, worst tick {worst_expansions} expansions",
        per(sweeps),
        per(pairs)
    );
    println!("targets (docs/movement.md > 5.8, release): movement <= 1 ms, whole tick <= 10 ms worst");
    let moving = t.sim().objects().filter_map(|o| o.mover.as_ref());
    println!(
        "at tick {}: {} units still moving",
        t.sim().tick(),
        moving.filter(|m| m.status == MoveStatus::Moving).count()
    );
}
