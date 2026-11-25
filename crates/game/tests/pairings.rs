//! Playtest pairings in the client (slice M5-R, `artifacts/director/m5-plan.md > R: Playtest pairings`): every ordered
//! pair of the factions that have Starting Conditions starts from the start options on every Skirmish Default map, as
//! `cargo run -- --factions a,b --headless --exit-at-tick 64` starts it, with `--scenario <name>` for a built-in map
//! and `--map <file>` for one of `maps/` (`docs/playtest/README.md`).
//!
//! The flags take the client's own path, `cli::parse`, `cli::into_config` and `build_app`, as its `main` does, and the
//! game runs one tick an update (`TimeUpdateStrategy::FixedTimesteps`), never by the clock, until the flags' exit.
//! Then the game is at tick 64, each player plays the faction named for them and owns their Starting Conditions'
//! objects, nobody is defeated, and debug commands are on, so that F2 switches between the players (the hot-seat).
//! Each pairing runs on a thread named after it, so that a failure names it. The sim's test of the pairings, which
//! plays them on, is `crates/sim/tests/pairings/`.

use std::any::Any;
use std::fs;
use std::path::Path;
use std::thread;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use sc_sim::{Faction, MapDef, MapType, StartingConditions, Tick, scenarios};
use space_crystals::build_app;
use space_crystals::cli;
use space_crystals::editor::objects::faction_name;
use space_crystals::platform::DisplayMode;
use space_crystals::sim_driver::SimRes;

/// The flags' exit tick.
const EXIT_AT: Tick = 64;

#[test]
fn every_pairing_starts_from_the_start_options() {
    let factions: Vec<Faction> = (Faction::ALL.iter().copied())
        .filter(|&faction| starting(faction).is_some())
        .collect();
    let mut pairings = Vec::new();
    for map in maps() {
        for &a in &factions {
            for &b in &factions {
                let names = [a, b].map(|faction| faction_name(faction).to_lowercase());
                let mut flags = vec!["--factions".to_owned(), names.join(",")];
                flags.extend([
                    "--headless".to_owned(),
                    "--exit-at-tick".to_owned(),
                    EXIT_AT.to_string(),
                ]);
                flags.extend(map.flags.iter().cloned());
                let name = format!("--factions {} {}", names.join(","), map.shown);
                pairings.push((name, flags, [a, b]));
            }
        }
    }
    assert!(!pairings.is_empty(), "no pairings");
    let failed: Vec<String> = thread::scope(|s| {
        let threads: Vec<_> = (pairings.iter())
            .map(|(name, flags, factions)| {
                let thread = thread::Builder::new().name(name.clone());
                let thread = thread.spawn_scoped(s, move || start(flags, *factions));
                (name, thread.expect("a thread for the pairing"))
            })
            .collect();
        (threads.into_iter())
            .filter_map(|(name, thread)| Some(format!("{name}: {}", panic_message(thread.join().err()?))))
            .collect()
    });
    for (name, _, _) in &pairings {
        println!("{name}");
    }
    assert!(
        failed.is_empty(),
        "{} of {} pairings failed:\n{}",
        failed.len(),
        pairings.len(),
        failed.join("\n")
    );
}

/// `faction`'s Starting Conditions, if it has them.
fn starting(faction: Faction) -> Option<&'static StartingConditions> {
    given(StartingConditions::of(faction))
}

/// What `StartingConditions::of` gave, through `Into<Option<_>>`, so that this reads it as it is now, an `Option` (none
/// for the Cults, until slice M5-B), and as plan decision T63 makes it, total.
fn given<'a>(conditions: impl Into<Option<&'a StartingConditions>>) -> Option<&'a StartingConditions> {
    conditions.into()
}

/// A Skirmish Default map, by the flags that start it.
struct Map {
    /// The flags as the playtest kit gives them, from the repository's root: `--scenario dev_skirmish`, `--map
    /// maps/dev_skirmish.json`.
    shown: String,
    /// The flags as this test gives them, a file by its full path.
    flags: Vec<String>,
}

/// Every Skirmish Default map: the built-in ones (`scenarios::NAMES`), then the files of `maps/` by name.
fn maps() -> Vec<Map> {
    let mut maps: Vec<Map> = (scenarios::NAMES.iter())
        .filter(|&&name| scenarios::by_name(name).is_some_and(|s| s.map.map_type == MapType::SkirmishDefault))
        .map(|&name| Map {
            shown: format!("--scenario {name}"),
            flags: vec!["--scenario".to_owned(), name.to_owned()],
        })
        .collect();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../maps");
    let mut files: Vec<String> = fs::read_dir(&dir)
        .map(|entries| entries.filter_map(|e| e.ok()?.file_name().into_string().ok()).collect())
        .unwrap_or_default();
    files.retain(|name| name.ends_with(".json"));
    files.sort();
    for name in files {
        let path = dir.join(&name);
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("maps/{name}: {e}"));
        let map: MapDef = serde_json::from_str(&text).unwrap_or_else(|e| panic!("maps/{name}: {e}"));
        if map.map_type == MapType::SkirmishDefault {
            maps.push(Map {
                shown: format!("--map maps/{name}"),
                flags: vec!["--map".to_owned(), path.display().to_string()],
            });
        }
    }
    maps
}

/// Starts the game `flags` set up and runs it to their exit (module docs); `factions` are players 1 and 2's.
fn start(flags: &[String], factions: [Faction; 2]) {
    let options = cli::parse(flags).unwrap_or_else(|e| panic!("the flags don't parse: {e}"));
    let config = cli::into_config(options).unwrap_or_else(|e| panic!("the flags start no game: {e}"));
    assert_eq!(config.display, DisplayMode::Headless);
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    let mut exit = None;
    for _ in 0..EXIT_AT + 16 {
        app.update();
        exit = app.should_exit();
        if exit.is_some() {
            break;
        }
    }
    let sim = app.world().resource::<SimRes>();
    assert_eq!(exit, Some(AppExit::Success), "no exit by tick {}", sim.tick());
    assert_eq!(sim.tick(), EXIT_AT, "the exit tick");
    let played: Vec<Faction> = sim.players().iter().map(|p| p.faction).collect();
    assert_eq!(played[..2], factions, "the players' factions");
    assert!(sim.config().debug, "debug commands are off: F2 can't switch players");
    for p in sim.players() {
        assert!(!p.is_defeated(), "player {} is defeated", p.id.0);
        for &(kind, n) in starting(p.faction).map_or(&[][..], |start| start.objects) {
            let owned = (sim.objects())
                .filter(|o| o.owner == Some(p.id) && o.kind == kind)
                .count();
            assert!(owned >= n as usize, "player {} owns {owned} {kind:?}, of {n}", p.id.0);
        }
    }
}

/// What a panic said.
fn panic_message(payload: Box<dyn Any + Send>) -> String {
    let text = payload.downcast_ref::<&str>().map(|s| (*s).to_owned());
    text.or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "a panic without a message".to_owned())
}
