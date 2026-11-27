//! (slice M5-Mb) The soak sweep (`artifacts/director/m5-plan.md > 7. QA and the playtest`: "`soak_sweep` over all 16
//! pairings and several seeds (ignored). Scripted Cults orders: gathering, group building, training, evacuation by
//! destruction. T65's invariants"): every pairing on every Skirmish Default map ([`super::maps`]), each with every
//! seed of [`SEEDS`] (the game's, `GameSetup::seed`, and the script's choices, [`STREAM`]), in three parts:
//! 1. **The openings** ([`OPENINGS_TIME`]), as [`super::Pairing::play`] plays them, with its checks, but for the Cults,
//!    whose line trains [`MILITANTS`] Militants (`super::openings::cults_training`): gathering, the Armory's group
//!    Build, the Recruits' entry and their training. At a moment the seed picks while it holds units, each Cults
//!    player's Armory is destroyed (`DebugCommand::Remove`, the one way objects leave the game, `Sim::remove_objects`),
//!    and its units are evacuated (`destruction`; `factions/cults/economy.md > Cults Building Mechanics > Evacuation
//!    on Destruction`).
//! 2. **A fight** ([`FIGHT_TIME`], `fight`): each player's extras, debug spawns by their home, sent to meet the next
//!    player's halfway; the Cults' Hand of Heaven driving through, a Sparrow circling a corner first, the Prayer-mobile
//!    casting Suppress and Inspire, a Shrine casting its two abilities, the Disciples channelling Shroud, and the GDO's
//!    Rocketmen locking on. The openings play on, their books balanced.
//! 3. **The end** ([`super::end`]): player 2's buildings removed, and player 1 wins.
//!
//! After every tick, beside `TestSim`'s invariants (among them T65's "no unit overlaps a building", I1, and "reserved
//! Recruits are held in their Armory", R1; and the addendum's, `artifacts/director/m5-plan-addendum.md > M (now
//! W12)`: R1, E1, C1, A1, T1, H1 and B1; and the tread law, S1) and the pairings' own ([`super::check`]), the soak's
//! (`watch`):
//! - **Unit Control** (T65: "the sum of the Cults usages plus the orphans equals the Cults units alive"): for each
//!   Cults player, their Halls' usages and their orphaned points (`Lineages`) are the Unit Control costs of their
//!   units, on the map and held, but (M6-CM) for their illusions, and of their Halls' Recruits in production, and the
//!   HUD's figure (`Sim::army_cap`).
//! - **Held units** (T65: "held units are only in buildings that exist"): a unit held in a building is in one that is
//!   on the map, holds units, and is its owner's; one held at a site, in a site.
//! - **Sites** (T65: "a site's progress is never above its total").
//! - **Gliders** (the addendum: "air units stay within the overshoot bound"): a Glider is never farther off the map
//!   than its overshoot bound (`watch::overshoot_bound`).
//! - **Crushing** (M5-V4): a crushed unit is gone in the tick, removed with no Hit and no Died, crushable, and the
//!   crusher's enemy, by a kind that crushes; the soak counts them.
//! - **Suppress** (M5-N11): a unit that holds its fire doesn't aim, and fires only a shot under way as it was
//!   suppressed (`sc_sim::suppress`: "a shot being fired [...] run[s] on").
//! - **Buffs** (M5-N5, M5-N11): a buff works until its time is up, never less (F1 checks it ends then): one held in a
//!   tick whose time isn't up after the next is held after it, of a unit that is still there and still of a kind its
//!   ability affects.
//!
//! Each game runs on one of [`WORKERS`] threads, and is named in the output and in a failure by its command line and
//! seed. `SOAK_ONLY=<text>` plays only the games whose name contains it.

mod destruction;
mod fight;
mod watch;

use std::any::Any;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Instant;

use sc_sim::testing::TestSim;
use sc_sim::testing::build_order::Books;
use sc_sim::testing::syndicate::SupplyBooks;
use sc_sim::{Faction, Object, PlayerCommand, PlayerId, Rng, Sim, TICKS_PER_SECOND, Tick};

use super::openings::{self, Opening};
use super::{Map, Pairing, Side, check, end, followable, gate_takes, maps, panic_message};
use destruction::Destruction;
use fight::Fight;
use watch::Watch;

/// The seeds each pairing plays with.
const SEEDS: [u64; 4] = [1, 2, 3, 4];
/// The stream of the script's choices (`Rng::new(seed, STREAM)`), apart from the game's own.
const STREAM: u64 = 0x50a4_5eed;
/// The openings' game time.
const OPENINGS_TIME: Tick = 3 * 60 * TICKS_PER_SECOND;
/// The fight's game time: long enough for a Prayer-mobile to cast Suppress (`fight`).
const FIGHT_TIME: Tick = 150 * TICKS_PER_SECOND;
/// The Militants the soak's Cults train (`openings::cults_training`): enough for their Armory to hold Recruits for a
/// while.
const MILITANTS: usize = 6;
/// The most ticks with units inside an Armory that pass before it is destroyed: the seed picks how many.
const SKIP: u32 = 48;
/// The threads the games run on.
const WORKERS: usize = 8;

/// Every pairing on every map with every seed, for the openings, a fight and the end (module docs).
#[test]
#[ignore = "every pairing, map and seed: cargo test -p sc_sim --release --test sim pairings::soak -- --ignored --nocapture"]
fn soak_sweep() {
    let maps = maps();
    assert!(!maps.is_empty(), "there is no Skirmish Default map");
    let games: Vec<(Pairing, u64)> = (maps.iter())
        .flat_map(|map: &Map| {
            (Faction::ALL.iter())
                .flat_map(move |&a| Faction::ALL.iter().map(move |&b| Pairing { map, factions: [a, b] }))
        })
        .flat_map(|pairing| {
            SEEDS.map(|seed| {
                (
                    Pairing {
                        map: pairing.map,
                        factions: pairing.factions,
                    },
                    seed,
                )
            })
        })
        .collect();
    assert_eq!(
        games.len(),
        16 * maps.len() * SEEDS.len(),
        "16 pairings a map, each with every seed"
    );
    let only = std::env::var("SOAK_ONLY").ok();
    let mut games = games;
    games.retain(|(pairing, seed)| {
        only.as_ref()
            .is_none_or(|only| name(pairing, *seed).contains(only.as_str()))
    });
    let started = now();
    let next = AtomicUsize::new(0);
    let results: Mutex<Vec<Played>> = Mutex::new(Vec::new());
    thread::scope(|s| {
        for worker in 0..WORKERS {
            let (games, next, results) = (&games, &next, &results);
            let thread = thread::Builder::new().name(format!("soak worker {worker}"));
            thread
                .spawn_scoped(s, move || {
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some((pairing, seed)) = games.get(i) else { break };
                        let name = name(pairing, *seed);
                        let result = panic::catch_unwind(AssertUnwindSafe(|| soak(pairing, *seed)));
                        let result = result.map_err(|payload: Box<dyn Any + Send>| panic_message(payload));
                        results.lock().expect("the results").push((i, name, result));
                    }
                })
                .expect("a worker thread");
        }
    });
    let mut results = results.into_inner().expect("the results");
    results.sort_by_key(|&(i, _, _)| i);
    let mut total = Tally::default();
    for (_, name, result) in &results {
        match result {
            Ok((came, tally)) => {
                println!("{name}: {came}");
                total.add(tally);
            }
            Err(message) => println!("{name}: FAILED: {message}"),
        }
    }
    let failed: Vec<String> = (results.iter())
        .filter_map(|(_, name, result)| Some(format!("{name}: {}", result.as_ref().err()?)))
        .collect();
    println!(
        "{} games in {:.1} s: {} ticks; {}",
        results.len(),
        started.elapsed().as_secs_f32(),
        total.ticks,
        total.summary()
    );
    assert!(
        failed.is_empty(),
        "{} of {} games failed:\n{}",
        failed.len(),
        results.len(),
        failed.join("\n")
    );
}

/// A game played: its place in the sweep, its name, and what came of it or why it failed.
type Played = (usize, String, Result<(String, Tally), String>);

/// The wall clock, for the sweep's time in the output. Only tests read a clock: nothing in the sim can
/// (`ARCHITECTURE.md > Determinism`).
#[expect(
    clippy::disallowed_methods,
    reason = "the sweep reports its time; the sim itself never reads a clock"
)]
fn now() -> Instant {
    Instant::now()
}

/// A game's name: its pairing's command line, and its seed.
fn name(pairing: &Pairing, seed: u64) -> String {
    format!("{} (seed {seed})", pairing.name())
}

/// What a game, or the sweep, saw happen.
#[derive(Clone, Debug, Default)]
struct Tally {
    ticks: u64,
    /// Armories destroyed with units inside, the units evacuated, and the trainings refunded.
    destroyed: u32,
    evacuated: u32,
    refunded: u32,
    /// Casts, by ability: Health regeneration, Energy regeneration, Inspire, Suppress, Scan.
    casts: [u32; 5],
    shrouds: u32,
    /// (M6-CM) Mirages cast.
    mirages: u32,
    crushed: u32,
    /// Unit ticks spent holding fire, and buffs that ended as their time was up.
    suppressed: u64,
    ended: u32,
    /// The attacks made and the units that died, in the fight.
    attacks: u32,
    deaths: u32,
    /// Glider ticks off the map, and the farthest off it one went, in space units.
    off_map: u64,
    farthest: f32,
}

impl Tally {
    fn add(&mut self, other: &Tally) {
        self.ticks += other.ticks;
        self.destroyed += other.destroyed;
        self.evacuated += other.evacuated;
        self.refunded += other.refunded;
        for (mine, theirs) in self.casts.iter_mut().zip(other.casts) {
            *mine += theirs;
        }
        self.shrouds += other.shrouds;
        self.mirages += other.mirages;
        self.crushed += other.crushed;
        self.suppressed += other.suppressed;
        self.ended += other.ended;
        self.attacks += other.attacks;
        self.deaths += other.deaths;
        self.off_map += other.off_map;
        self.farthest = self.farthest.max(other.farthest);
    }

    fn summary(&self) -> String {
        let [health, energy, inspire, suppress, scan] = self.casts;
        format!(
            "{} Armories destroyed, {} units evacuated, {} trainings refunded; casts: {health} Health regeneration, \
             {energy} Energy regeneration, {inspire} Inspire, {suppress} Suppress, {scan} Scan; {} Shrouds; {} Mirages; {} buffs ended on time; \
             {} unit ticks suppressed; {} crushed; {} attacks, {} deaths; {} Glider ticks off the map, at most {:.0} su",
            self.destroyed,
            self.evacuated,
            self.refunded,
            self.shrouds,
            self.mirages,
            self.ended,
            self.suppressed,
            self.crushed,
            self.attacks,
            self.deaths,
            self.off_map,
            self.farthest
        )
    }
}

/// Plays `pairing` with `seed` (module docs), and says what came of it. Panics at the first check that fails.
fn soak(pairing: &Pairing, seed: u64) -> (String, Tally) {
    let mut scenario = pairing.scenario();
    scenario.setup.seed = seed;
    let problems = scenario.validate();
    assert!(problems.is_empty(), "the game can't start: {problems:?}");
    let mut t = TestSim::new(scenario);
    let mut rng = Rng::new(seed, STREAM);
    let players: Vec<(PlayerId, Faction)> = (t.sim().players().iter()).map(|p| (p.id, p.faction)).collect();
    let mut sides: Vec<Side> = (players.iter())
        .map(|&(player, _)| Side {
            player,
            opening: opening(t.sim(), player),
            gathered: 0,
        })
        .collect();
    let mut destructions: Vec<Destruction> = (players.iter())
        .filter(|&&(_, faction)| faction == Faction::Cults)
        .map(|&(player, _)| Destruction::new(player, rng.below(SKIP)))
        .collect();
    let mut fight = Fight::new(&players, OPENINGS_TIME + 1, &mut rng);
    let mut watch = Watch::default();
    watch.remember(t.sim());
    let mut tally = Tally::default();
    while t.sim().tick() < OPENINGS_TIME + FIGHT_TIME {
        let tick = t.sim().tick() + 1;
        let commands: Vec<PlayerCommand> = (sides.iter_mut())
            .filter_map(|side| side.opening.as_mut())
            .flat_map(|opening| opening.play(t.sim()))
            .collect();
        gate_takes(t.sim(), &commands);
        let mut extra: Vec<PlayerCommand> = (destructions.iter_mut())
            .filter_map(|d| d.before(t.sim(), &commands))
            .collect();
        extra.extend(fight.commands(t.sim()));
        gate_takes(t.sim(), &extra);
        let books: Vec<_> = (sides.iter())
            .map(|side| {
                let paid = side.opening.as_ref().map_or((0, 0), |opening| opening.paid_in(tick));
                let crystals = followable(t.sim(), side.player).then(|| Books::open(t.sim(), side.player));
                (crystals, SupplyBooks::open(t.sim(), side.player), paid)
            })
            .collect();
        for c in commands.into_iter().chain(extra) {
            t.at(tick, c.player, c.command);
        }
        t.step();
        for (side, (crystals, supplies, (paid, paid_supplies))) in sides.iter_mut().zip(books) {
            if let Some(books) = crystals {
                let entry = books.close(t.sim(), paid);
                side.gathered += entry.mined + entry.delivered;
            }
            supplies.close(t.sim(), paid_supplies);
        }
        check(t.sim());
        watch.check(t.sim(), &mut tally);
        for d in &mut destructions {
            d.after(t.sim(), &mut tally);
        }
        fight.after(t.sim(), &mut tally);
        let defeated: Vec<PlayerId> = (t.sim().players().iter())
            .filter(|p| p.is_defeated())
            .map(|p| p.id)
            .collect();
        assert!(defeated.is_empty(), "tick {tick}: {defeated:?} defeated before the end");
    }
    tally.ticks = u64::from(t.sim().tick());
    for side in sides.iter().filter(|side| side.opening.is_some()) {
        assert!(side.gathered > 0, "player {}'s opening gathered nothing", side.player.0);
    }
    let destroyed: Vec<String> = (destructions.iter())
        .map(|d| {
            let removal =
                (d.removal.as_ref()).unwrap_or_else(|| panic!("player {}'s Armory never held a unit", d.player.0));
            format!(
                "player {}'s Armory destroyed in tick {} with {} inside",
                d.player.0,
                removal.tick,
                removal.units.len()
            )
        })
        .collect();
    let gathered: Vec<String> = (sides.iter())
        .map(|side| format!("player {} {}", side.player.0, side.gathered))
        .collect();
    let (removed, won) = end(&mut t);
    let came = format!(
        "Space Crystals gathered: {}; {}{}; then {removed} buildings removed: player 1 won in tick {won}",
        gathered.join(", "),
        (destroyed.iter()).map(|d| format!("{d}; ")).collect::<String>(),
        tally.summary(),
    );
    (came, tally)
}

/// `player`'s opening (`super::opening`), the Cults' training [`MILITANTS`] (module docs).
fn opening(sim: &Sim, player: PlayerId) -> Option<Box<dyn Opening>> {
    match sim.player(player)?.faction {
        Faction::Cults => Some(openings::cults_training(sim, player, MILITANTS)),
        _ => super::opening(sim, player),
    }
}

/// Every object of the game: the map's, and the units held off it.
fn everything(sim: &Sim) -> impl Iterator<Item = &Object> {
    sim.objects().chain(sim.held().map(|held| &held.object))
}
