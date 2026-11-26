//! Playtest pairings (slice M5-R, `artifacts/director/m5-plan.md > R: Playtest pairings`): every ordered pair of the
//! four factions (`Faction::ALL`: 4 × 4 = 16, every faction having its Starting Conditions since slice M5-B, plan
//! decision T63), on every Skirmish Default map ([`maps`], `core/map_editor.md > Map Types > Skirmish Default`:
//! the built-in scenarios of that type and the maps of `maps/`). Each starts as the client's `--factions a,b` starts
//! it, player 1 playing `a` and player 2 `b`, and is named by that command line in the output and in every failure
//! ([`Pairing::name`]: `--factions gdo,syndicate --scenario dev_skirmish`).
//!
//! A pairing goes in two parts:
//! 1. **The openings** ([`Pairing::play`]): each player plays their faction's scripted opening ([`OPENINGS`],
//!    `openings`) with their own commands through the gate, for the run's game time. After every tick it checks
//!    `TestSim`'s invariants (the movement invariants, V1, K1, O1 and P1: `sc_sim::testing::invariants`) and its own
//!    ([`check`]); that the gate takes every command the players give ([`gate_takes`]); that each player's books
//!    balance, their Space Crystals (`testing::build_order::Books`, while no patch under one of their plates could run
//!    out in the step) and their Supplies (`testing::syndicate::SupplyBooks`); and that nobody is defeated. At the end
//!    of it every player with an opening has gathered.
//! 2. **The end** ([`end`]): player 2's buildings removed by debug commands, player 2 is defeated and player 1 wins
//!    (`core/game_rules.md > Defeat Condition`: "a player is **defeated when they own no buildings**";
//!    `core/game_rules.md > End of Game`: "When only **one player or team** is left undefeated, it **wins**").
//!
//! The default run plays the first map for 30 s of game time; an ignored one plays every map for 3 minutes. Each
//! pairing runs on a thread of its own, named after it, so that a panic's message names it too.

mod openings;

use std::any::Any;
use std::fs;
use std::path::Path;
use std::thread;

use sc_sim::network::Place;
use sc_sim::testing::build_order::Books;
use sc_sim::testing::syndicate::SupplyBooks;
use sc_sim::testing::{self, TestSim};
use sc_sim::{
    Category, Command, ConstructionCommand, DebugCommand, Faction, GameSetup, MapDef, MapType, ObjectId, PlayerCommand,
    PlayerId, Reject, ResourceState, Scenario, Sim, TICKS_PER_SECOND, Tick, scenarios,
};

use openings::{Hook, Opening};

/// Each faction's scripted opening (`openings`), one line a faction. A faction without one plays none, and its player
/// stays idle: the Cults, from slice M5-B (their Starting Conditions) until slice M5-L adds their line here.
const OPENINGS: &[(Faction, Hook)] = &[
    (Faction::Gdo, openings::gdo),
    (Faction::Colonists, openings::colonists),
    (Faction::Syndicate, openings::syndicate),
];

/// The default run's game time, on the first map.
const SHORT: Tick = 30 * TICKS_PER_SECOND;
/// The ignored run's, on every map.
const LONG: Tick = 3 * 60 * TICKS_PER_SECOND;
/// The most ticks the end's removals may take ([`end`]).
const END_TICKS: Tick = 8;

const P1: PlayerId = PlayerId(1);

/// Every pairing on the first Skirmish Default map, `dev_skirmish`, for 30 s of game time.
#[test]
fn every_pairing_plays_the_first_map() {
    let maps = maps();
    assert!(!maps.is_empty(), "there is no Skirmish Default map");
    run(&maps[..1], SHORT);
}

/// Every pairing on every Skirmish Default map, for 3 minutes of game time.
#[test]
#[ignore = "every map for 3 minutes: cargo test -p sc_sim --release --test pairings -- --ignored --nocapture"]
fn every_pairing_plays_every_map() {
    run(&maps(), LONG);
}

/// A Skirmish Default map, as the client starts it.
struct Map {
    /// The client's flag that names it: `--scenario <name>` for a built-in one, `--map maps/<file>` for a file.
    flag: String,
    /// Its game as the client starts it without `--factions`.
    scenario: Scenario,
}

/// Every Skirmish Default map: the built-in scenarios of that type in `scenarios::NAMES`' order, then the maps of
/// `maps/` of that type by file name, as the client loads them (`crates/game/src/cli.rs`: a file's map with its
/// default setup).
fn maps() -> Vec<Map> {
    let built_in = (scenarios::NAMES.iter()).filter_map(|&name| {
        let scenario = scenarios::by_name(name)?;
        let flag = format!("--scenario {name}");
        Some(Map { flag, scenario })
    });
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../maps");
    let mut files: Vec<String> = fs::read_dir(&dir)
        .map(|entries| entries.filter_map(|e| e.ok()?.file_name().into_string().ok()).collect())
        .unwrap_or_default();
    files.retain(|name| name.ends_with(".json"));
    files.sort();
    let in_files = files.into_iter().map(|name| {
        let text = fs::read_to_string(dir.join(&name)).unwrap_or_else(|e| panic!("maps/{name}: {e}"));
        let map: MapDef = serde_json::from_str(&text).unwrap_or_else(|e| panic!("maps/{name}: {e}"));
        let flag = format!("--map maps/{name}");
        Map {
            flag,
            scenario: Scenario::new(map),
        }
    });
    (built_in.chain(in_files))
        .filter(|map| map.scenario.map.map_type == MapType::SkirmishDefault)
        .collect()
}

/// Plays every pairing on each of `maps` with `ticks` of openings, each on a thread named after it, prints what each
/// came to, and fails naming every pairing that failed, with its message.
fn run(maps: &[Map], ticks: Tick) {
    let mut pairings = Vec::new();
    for map in maps {
        for &a in Faction::ALL {
            for &b in Faction::ALL {
                pairings.push(Pairing { map, factions: [a, b] });
            }
        }
    }
    assert_eq!(
        pairings.len(),
        16 * maps.len(),
        "the 16 pairings of the four factions on each map"
    );
    let results: Vec<(String, Result<String, String>)> = thread::scope(|s| {
        let threads: Vec<_> = (pairings.iter())
            .map(|pairing| {
                let thread = thread::Builder::new().name(pairing.name());
                let thread = thread.spawn_scoped(s, move || pairing.play(ticks));
                (pairing.name(), thread.expect("a thread for the pairing"))
            })
            .collect();
        (threads.into_iter())
            .map(|(name, thread)| (name, thread.join().map_err(panic_message)))
            .collect()
    });
    for (name, result) in &results {
        match result {
            Ok(came) => println!("{name}: {came}"),
            Err(message) => println!("{name}: FAILED: {message}"),
        }
    }
    let failed: Vec<String> = (results.iter())
        .filter_map(|(name, result)| Some(format!("{name}: {}", result.as_ref().err()?)))
        .collect();
    assert!(
        failed.is_empty(),
        "{} of {} pairings failed:\n{}",
        failed.len(),
        results.len(),
        failed.join("\n")
    );
    println!("{} pairings played", results.len());
}

/// What a panic said.
fn panic_message(payload: Box<dyn Any + Send>) -> String {
    let text = payload.downcast_ref::<&str>().map(|s| (*s).to_owned());
    text.or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic without a message".to_owned())
}

/// One pairing: player 1 plays `factions[0]` and player 2 `factions[1]`, on `map`.
struct Pairing<'m> {
    map: &'m Map,
    factions: [Faction; 2],
}

impl Pairing<'_> {
    /// The pairing's name: the client's command line that starts it (`docs/playtest/README.md`).
    fn name(&self) -> String {
        let [a, b] = self.factions.map(|faction| format!("{faction:?}").to_lowercase());
        format!("--factions {a},{b} {}", self.map.flag)
    }

    /// The game the client's `--factions` starts (`crates/game/src/cli.rs`, plan decision T24): the map's, with each
    /// named slot's player playing its faction, and the GDO in any other slot.
    fn scenario(&self) -> Scenario {
        let mut scenario = self.map.scenario.clone();
        scenario.setup = GameSetup::per_slot(&scenario.map, &self.factions, scenario.setup.seed);
        scenario
    }

    /// Plays the pairing (module docs) with `ticks` of openings, and says what came of it. Panics at the first check
    /// that fails.
    fn play(&self, ticks: Tick) -> String {
        let scenario = self.scenario();
        let problems = scenario.validate();
        assert!(problems.is_empty(), "the game can't start: {problems:?}");
        assert!(scenario.setup.players.len() >= 2, "a map of one slot");
        // Debug commands are on, for the end's removals.
        let mut t = TestSim::new(scenario);
        let mut sides: Vec<Side> = (t.sim().players().iter())
            .map(|p| Side {
                player: p.id,
                opening: opening(t.sim(), p.id),
                gathered: 0,
            })
            .collect();
        while t.sim().tick() < ticks {
            let tick = t.sim().tick() + 1;
            let commands: Vec<PlayerCommand> = (sides.iter_mut())
                .filter_map(|side| side.opening.as_mut())
                .flat_map(|opening| opening.play(t.sim()))
                .collect();
            gate_takes(t.sim(), &commands);
            let books: Vec<_> = (sides.iter())
                .map(|side| {
                    let paid = side.opening.as_ref().map_or((0, 0), |opening| opening.paid_in(tick));
                    let crystals = followable(t.sim(), side.player).then(|| Books::open(t.sim(), side.player));
                    (crystals, SupplyBooks::open(t.sim(), side.player), paid)
                })
                .collect();
            for c in commands {
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
            let defeated: Vec<PlayerId> = (t.sim().players().iter())
                .filter(|p| p.is_defeated())
                .map(|p| p.id)
                .collect();
            assert!(
                defeated.is_empty(),
                "tick {tick}: {defeated:?} defeated during the openings"
            );
        }
        for side in sides.iter().filter(|side| side.opening.is_some()) {
            assert!(side.gathered > 0, "player {}'s opening gathered nothing", side.player.0);
        }
        let gathered: Vec<String> = (sides.iter())
            .map(|side| format!("player {} {}", side.player.0, side.gathered))
            .collect();
        let (removed, won) = end(&mut t);
        format!(
            "{} s of openings (Space Crystals gathered: {}), then {removed} buildings removed: player 1 won in tick \
             {won}",
            ticks / TICKS_PER_SECOND,
            gathered.join(", ")
        )
    }
}

/// A player of a pairing.
struct Side {
    player: PlayerId,
    /// Their faction's opening, if it has one ([`OPENINGS`]).
    opening: Option<Box<dyn Opening>>,
    /// The Space Crystals their plates mined and their workers delivered, in the steps their books followed.
    gathered: u32,
}

/// `player`'s opening, by their faction's hook; `None` for a faction without one.
fn opening(sim: &Sim, player: PlayerId) -> Option<Box<dyn Opening>> {
    let faction = sim.player(player)?.faction;
    let &(_, hook) = OPENINGS.iter().find(|&&(f, _)| f == faction)?;
    Some(hook(sim, player))
}

/// That the gate takes each of `commands`, as the ones before it in the step leave the game. This sees the game before
/// the step, so a constructor placing its ready building may take its next Construct in the same step, as in the
/// golden games and the soaks.
fn gate_takes(sim: &Sim, commands: &[PlayerCommand]) {
    let placing: Vec<ObjectId> = (commands.iter())
        .filter_map(|c| match &c.command {
            Command::Construction {
                objects,
                command: ConstructionCommand::Place { .. },
            } => Some(objects.clone()),
            _ => None,
        })
        .flatten()
        .collect();
    for c in commands {
        match sim.validate(c) {
            Ok(()) => {}
            Err(Reject::ConstructorBusy(id)) if placing.contains(&id) => {}
            Err(reject) => panic!("tick {}: the gate refuses {c:?}: {reject:?}", sim.tick() + 1),
        }
    }
}

/// Whether `player`'s books can follow the step: no patch under one of their plates holds 10 or less, the most a plate
/// takes in a step, which the step could empty (`Books::close`).
fn followable(sim: &Sim, player: PlayerId) -> bool {
    let left = |patch| match sim.object(patch)?.resource {
        Some(ResourceState::CrystalPatch { remaining }) => Some(remaining),
        _ => None,
    };
    (sim.objects().filter(|o| o.owner == Some(player)))
        .filter_map(|o| left(o.extractor?.patch))
        .all(|left| left > 10)
}

/// The pairings' own checks after every tick, beside `TestSim`'s invariants: no object names a removed one
/// (`testing::dangling_references`); and the Tunnel Networks' held store (`sc_sim::network`) holds no unit that is on
/// the map too, or named by an object there, or with orders, and each site's builder is held at its site.
fn check(sim: &Sim) {
    let tick = sim.tick();
    assert_eq!(
        testing::dangling_references(sim),
        [],
        "tick {tick}: objects name removed ones"
    );
    for held in sim.held() {
        let unit = held.object.id;
        assert!(
            sim.object(unit).is_none(),
            "tick {tick}: held unit {unit:?} is on the map too"
        );
        let idle = held.object.orders.as_ref().is_none_or(|orders| orders.is_idle());
        assert!(idle, "tick {tick}: held unit {unit:?} has orders");
        let named = sim.objects().find(|o| testing::referenced_ids(o).contains(&unit));
        assert!(
            named.is_none(),
            "tick {tick}: {:?} names held unit {unit:?}",
            named.map(|o| o.id)
        );
    }
    for o in sim.objects() {
        if let Some(builder) = o.construction.and_then(|c| c.builder.unit()) {
            let held = sim.held_unit(builder).is_some_and(|h| h.place == Place::Site(o.id));
            assert!(
                held,
                "tick {tick}: site {:?}'s builder {builder:?} isn't held at it",
                o.id
            );
        }
    }
}

/// The end (module docs, 2), the openings stopped: every building of player 2's removed, one debug command each
/// (`DebugCommand::Remove`), and again in each next tick while they own one, at most [`END_TICKS`] ticks: a Drone
/// that reaches its site in the tick of the removals begins its Beacon after them, a building from its first frame
/// (`core/game_rules.md > Defeat Condition`). On a map of more slots, the GDO's players in them (as `--factions`
/// starts them) go the same way. Then player 1 has won, in the tick of the last removal, and every other player is
/// defeated and owns nothing, on the map or in a Tunnel Network: their units are neutral
/// (`core/game_rules.md > On Defeat`). Returns how many buildings were removed, and the tick the game ended in.
fn end(t: &mut TestSim) -> (usize, Tick) {
    let from = t.sim().tick();
    let mut removed = 0;
    while t.sim().outcome().is_none() {
        let tick = t.sim().tick() + 1;
        assert!(
            tick <= from + END_TICKS,
            "the others still own buildings after {END_TICKS} ticks of removals"
        );
        let theirs: Vec<PlayerCommand> = (t.sim().objects())
            .filter(|o| o.owner.is_some_and(|owner| owner != P1) && o.def().category == Category::Structure)
            .map(|o| PlayerCommand {
                player: P1,
                command: Command::Debug(DebugCommand::Remove { id: o.id }),
            })
            .collect();
        gate_takes(t.sim(), &theirs);
        removed += theirs.len();
        for c in theirs {
            t.at(tick, c.player, c.command);
        }
        t.step();
        check(t.sim());
    }
    let sim = t.sim();
    let outcome = sim.outcome().expect("the game is over");
    assert_eq!(outcome.winners, [P1], "player 1 wins");
    assert_eq!(
        outcome.tick,
        sim.tick(),
        "the game ended in the tick of the last removal"
    );
    for p in sim.players() {
        assert_eq!(p.is_defeated(), p.id != P1, "player {} defeated", p.id.0);
    }
    let owned: Vec<_> = (sim.objects())
        .filter(|o| o.owner.is_some_and(|owner| owner != P1))
        .map(|o| (o.id, o.kind))
        .collect();
    assert_eq!(owned, [], "the defeated own objects");
    let held: Vec<_> = (sim.held())
        .filter(|h| h.object.owner.is_some_and(|owner| owner != P1))
        .map(|h| (h.object.id, h.object.kind))
        .collect();
    assert_eq!(held, [], "the defeated hold units in a Tunnel Network");
    (removed, outcome.tick)
}
