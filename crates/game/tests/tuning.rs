//! (Slice M5-X2) Games with a data directory (`--tuning`, `space_crystals::data`), in a test program of their own, as
//! the sim's games on other rules are (`crates/sim/tests/rules.rs`): the rules a game reads are the process's (the
//! current generation, `sc_sim::ruleset`), and the `game` program's tests all play at once on the built-in ones. Each
//! test here holds [`one_at_a_time`] for all of its games, so that no two games on different rules step at once.
//!
//! Headless, one tick per update at the normal speed, as in `replay.rs`; the first update only starts the clock, and
//! each update is a sixteenth of a second of the app's real time, so the poll (every half second) looks every 8th
//! update, no sleep. One fight throughout: a Peacekeeper (id 1, player 1) and a Guard (id 2, player 2), 3 tiles apart,
//! attacking each other on their own. On the built-in rules the Guard wins; with the Peacekeeper's Damage at 100
//! (`qa/tuning/peacekeeper_damage/`), its first shot after the change kills the Guard.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use sc_sim::testing::ScenarioBuilder;
use sc_sim::{Command, DebugCommand, ObjectId, ObjectKind, PlayerId, Ruleset, Scenario, SpawnAt, Tick, TilePos};
use space_crystals::command_ui::feedback::TargetFeedback;
use space_crystals::data::{TuningData, export};
use space_crystals::hud::TopBarNote;
use space_crystals::input::InjectedInput;
use space_crystals::platform::DisplayMode;
use space_crystals::playtest_menu::PlaytestMenu;
use space_crystals::replay::load_replay;
use space_crystals::sim_driver::{RecordedInputs, SimRes};
use space_crystals::tuning::ClientTuning;
use space_crystals::{AppConfig, build_app};

const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
/// The Guard (module docs); the Peacekeeper is id 1.
const GUARD: ObjectId = ObjectId(2);

/// The patch that makes the Peacekeeper win (`qa/tuning/peacekeeper_damage/`).
const DAMAGE_100: &str = r#"{"unit": {"attack": {"damage": 100}}}"#;
const PEACEKEEPER_FILE: &str = "defs/gdo/units/peacekeeper.json";

/// Held by each test for all of its games (module docs). A test that failed holding it leaves it to the next.
fn one_at_a_time() -> MutexGuard<'static, ()> {
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
    ONE_AT_A_TIME.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The fight (module docs): player 1's Peacekeeper and player 2's Guard on a flat 16 × 10 map.
fn fight() -> Scenario {
    ScenarioBuilder::new(&"................\n".repeat(10))
        .spawn(
            ObjectKind::Peacekeeper,
            Some(P1),
            SpawnAt::Point(TilePos::new(5, 5).center()),
        )
        .spawn(ObjectKind::Guard, Some(P2), SpawnAt::Point(TilePos::new(8, 5).center()))
        .build()
}

/// A fresh directory of test `test`'s own (each test removes its own when it is done).
fn temp_dir(test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sc_tuning_test_{test}_{}", std::process::id()));
    // Left by an earlier run that had the same process id and failed before cleaning up.
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a temporary directory");
    dir
}

/// Writes `text` to the file at `path` in `dir`, its folders made.
fn write(dir: &Path, path: &str, text: &str) {
    let file = dir.join(path);
    fs::create_dir_all(file.parent().expect("a folder")).expect("its folders");
    fs::write(file, text).expect("the file");
}

/// The fight's game, with the data directory `tuning` if given, recorded to `record` if given.
fn game(tuning: Option<&Path>, record: Option<&Path>) -> App {
    let mut config = AppConfig::new(fight());
    config.display = DisplayMode::Headless;
    config.tuning = tuning.map(Path::to_path_buf);
    config.record = record.map(Path::to_path_buf);
    let mut app = build_app(config);
    app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    app.finish();
    app.cleanup();
    app.update();
    app
}

fn tick(app: &App) -> Tick {
    app.world().resource::<SimRes>().tick()
}

fn hash(app: &App) -> u64 {
    app.world().resource::<SimRes>().state_hash()
}

/// The Peacekeeper's Damage in the rules its game plays by.
fn damage(app: &App) -> f32 {
    let rules = app.world().resource::<SimRes>().rules();
    let attack = (rules.def(ObjectKind::Peacekeeper).unit.as_ref()).and_then(|unit| unit.attack.as_ref());
    attack.expect("an attack").damage
}

/// The ticks whose inputs held a `DebugCommand::SetRules`, in the game as it is.
fn set_rules_ticks(app: &App) -> Vec<Tick> {
    let inputs = &app.world().resource::<RecordedInputs>().ticks;
    let sets = |commands: &[sc_sim::PlayerCommand]| {
        (commands.iter()).any(|c| matches!(c.command, Command::Debug(DebugCommand::SetRules(_))))
    };
    (inputs.iter().enumerate())
        .filter(|(_, inputs)| sets(&inputs.commands))
        .map(|(i, _)| i as Tick + 1)
        .collect()
}

/// What the feedback line shows, if anything.
fn feedback(app: &App) -> Option<&'static str> {
    let feedback = app.world().resource::<TargetFeedback>();
    feedback.since.map(|_| feedback.text)
}

/// The top bar's notes that show, by their text.
fn notes(app: &mut App) -> Vec<String> {
    let mut notes = app.world_mut().query::<(&TopBarNote, &Node, &Children)>();
    let shown: Vec<Entity> = (notes.iter(app.world()))
        .filter(|(_, node, _)| node.display != Display::None)
        .flat_map(|(.., children)| children.iter())
        .collect();
    (shown.into_iter())
        .filter_map(|text| app.world().get::<Text>(text).map(|text| text.0.clone()))
        .collect()
}

/// Whether the Guard is still in the game.
fn guard_lives(app: &App) -> bool {
    app.world().resource::<SimRes>().object(GUARD).is_some()
}

/// The round trip (the slice's acceptance): the exported built-in data, read back unchanged, give the game no other
/// rules, so it plays tick for tick as one without `--tuning`, with no SetRules; the top bar says nothing of tuning.
#[test]
fn an_exported_directory_plays_as_the_built_in_rules() {
    let _one = one_at_a_time();
    let dir = temp_dir("round_trip");
    export(&dir).expect("exported");
    let (mut tuned, mut plain) = (game(Some(&dir), None), game(None, None));
    while tick(&plain) < 120 {
        tuned.update();
        plain.update();
        assert_eq!((tick(&tuned), hash(&tuned)), (tick(&plain), hash(&plain)));
    }
    assert_eq!(set_rules_ticks(&tuned), Vec::<Tick>::new());
    assert_eq!(*tuned.world().resource::<ClientTuning>(), ClientTuning::BUILTIN);
    assert!(
        !notes(&mut tuned).iter().any(|note| note.starts_with("Tuned")),
        "{:?}",
        notes(&mut tuned)
    );
    let _ = fs::remove_dir_all(&dir);
}

/// A file read before the first tick gives the game its rules in the first tick's inputs; the top bar says so.
#[test]
fn the_directory_is_read_before_the_first_tick() {
    let _one = one_at_a_time();
    let dir = temp_dir("first_tick");
    write(&dir, PEACEKEEPER_FILE, DAMAGE_100);
    write(&dir, "client/feedback.json", r#"{"refusal_ticks": 64}"#);
    let mut app = game(Some(&dir), None);
    app.update();
    assert_eq!(tick(&app), 1);
    assert_eq!(set_rules_ticks(&app), [1]);
    app.update();
    assert_eq!(damage(&app), 100.0, "from the second tick on");
    assert_eq!(app.world().resource::<ClientTuning>().feedback.refusal_ticks, 64);
    assert!(
        notes(&mut app).contains(&"Tuned: 2 files".to_owned()),
        "{:?}",
        notes(&mut app)
    );
    let _ = fs::remove_dir_all(&dir);
}

/// A change mid-game (the slice's acceptance): the poll finds it within half a second, its rules go to the game in
/// the next tick's inputs, and the game plays as the untuned one up to that tick and otherwise from it on: the
/// Peacekeeper's next shot kills the Guard, which wins the untuned fight. The top bar counts the file.
#[test]
fn a_change_mid_game_changes_the_fight_from_its_tick_on() {
    let _one = one_at_a_time();
    let dir = temp_dir("mid_game");
    let (mut tuned, mut plain) = (game(Some(&dir), None), game(None, None));
    while tick(&plain) < 20 {
        tuned.update();
        plain.update();
        assert_eq!(hash(&tuned), hash(&plain), "tick {}", tick(&plain));
    }
    write(&dir, PEACEKEEPER_FILE, DAMAGE_100);
    let mut hashes = Vec::new();
    while set_rules_ticks(&tuned).is_empty() {
        assert!(tick(&tuned) < 30, "the poll looks every half second, 8 ticks");
        tuned.update();
        plain.update();
        hashes.push((tick(&tuned), hash(&tuned), hash(&plain)));
    }
    let set = set_rules_ticks(&tuned)[0];
    assert_eq!(set, tick(&tuned), "in the next tick's inputs");
    for &(at, tuned_hash, plain_hash) in &hashes {
        if at < set {
            assert_eq!(tuned_hash, plain_hash, "tick {at}, before the change");
        }
    }
    assert_eq!(notes(&mut tuned).iter().filter(|n| *n == "Tuned: 1 file").count(), 1);
    while tick(&plain) < 120 {
        tuned.update();
        plain.update();
    }
    assert_ne!(hash(&tuned), hash(&plain));
    assert_eq!(damage(&tuned), 100.0);
    assert!(!guard_lives(&tuned), "the Peacekeeper's shot of 100 kills the Guard");
    assert!(guard_lives(&plain), "the Guard wins on the built-in rules");
    let _ = fs::remove_dir_all(&dir);
}

/// A refused file (the slice's acceptance: a structural patch; a zero speed and a negative cost) keeps the rules as
/// they were and says why in the feedback line, its file and member named, and the game goes on as the untuned one.
#[test]
fn a_refused_file_shows_in_the_feedback_line_and_the_game_goes_on() {
    let _one = one_at_a_time();
    // The Barracks' recipes, a list that a patch gives whole, the first one's cost negative.
    let barracks = || {
        let rules = serde_json::to_value(Ruleset::builtin_ref()).expect("the rules serialize");
        let mut recipes = rules["defs"]["Barracks"]["structure"]["production"]["recipes"].clone();
        recipes[0]["cost"] = serde_json::json!(-50);
        serde_json::json!({"structure": {"production": {"recipes": recipes}}}).to_string()
    };
    let cases = [
        (
            "defs/gdo/structures/barracks.json",
            r#"{"footprint": {"Tiles": {"w": 4}}}"#.to_owned(),
            "defs/gdo/structures/barracks.json: footprint.Tiles.w: structural, which needs a change to the Rust \
             default and a rebuild",
        ),
        (
            PEACEKEEPER_FILE,
            r#"{"unit": {"movement": {"TurnRate": {"max_speed": 0}}}}"#.to_owned(),
            "defs/gdo/units/peacekeeper.json: unit.movement.TurnRate.max_speed: must be positive",
        ),
        (
            "defs/gdo/structures/barracks.json",
            barracks(),
            "defs/gdo/structures/barracks.json: structure.production.recipes[0].cost: must be positive",
        ),
        (
            "tuning/combat.json",
            r#"{"minimum_damage": "high"}"#.to_owned(),
            "tuning/combat.json: minimum_damage: expected a number",
        ),
    ];
    for (i, (path, text, said)) in cases.into_iter().enumerate() {
        let dir = temp_dir(&format!("refused_{i}"));
        let (mut tuned, mut plain) = (game(Some(&dir), None), game(None, None));
        while tick(&plain) < 10 {
            tuned.update();
            plain.update();
        }
        write(&dir, path, &text);
        while tick(&plain) < 60 {
            tuned.update();
            plain.update();
            assert_eq!(hash(&tuned), hash(&plain), "{path}: tick {}", tick(&plain));
            if tick(&plain) == 30 {
                assert_eq!(feedback(&tuned), Some(said));
            }
        }
        assert_eq!(set_rules_ticks(&tuned), Vec::<Tick>::new(), "{path}");
        assert!(tuned.world().resource::<TuningData>().data.tuned.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}

/// A directory refused at the start leaves the game on the built-in rules from its first tick, and says why.
#[test]
fn a_directory_refused_at_the_start_leaves_the_built_in_rules() {
    let _one = one_at_a_time();
    let dir = temp_dir("refused_start");
    write(&dir, "defs/gdo/units/peacekeepr.json", "{}");
    let mut app = game(Some(&dir), None);
    app.update();
    assert_eq!(feedback(&app), Some("defs/gdo/units/peacekeepr.json: unknown file"));
    while tick(&app) < 40 {
        app.update();
    }
    assert_eq!(set_rules_ticks(&app), Vec::<Tick>::new());
    assert_eq!(damage(&app), 10.0);
    let _ = fs::remove_dir_all(&dir);
}

/// A tuned game replays (the slice's acceptance): its recording holds the SetRules of the start and of a change, and
/// plays back with no directory to the same state, checkpoint for checkpoint.
#[test]
fn a_tuned_game_replays_to_the_same_hash() {
    let _one = one_at_a_time();
    let dir = temp_dir("replay");
    let (data, recording) = (dir.join("data"), dir.join("game.json"));
    write(&data, PEACEKEEPER_FILE, r#"{"unit": {"attack": {"damage": 12}}}"#);
    let mut app = game(Some(&data), Some(&recording));
    while tick(&app) < 30 {
        app.update();
    }
    write(&data, PEACEKEEPER_FILE, DAMAGE_100);
    while tick(&app) < 100 {
        app.update();
    }
    assert_eq!(set_rules_ticks(&app).len(), 2, "the start's and the change's");
    app.world_mut().write_message(AppExit::Success);
    app.update();
    let end = (tick(&app), hash(&app));
    let replay = load_replay(&recording).expect("recorded");
    assert_eq!(replay.end_tick(), end.0);
    let played = replay.run_checked().expect("every checkpoint matches");
    assert_eq!((played.tick(), played.state_hash()), end);
    let _ = fs::remove_dir_all(&dir);
}

/// F8 restarts the game as the playtest menu's Restart does, reads the directory again and gives the new game its
/// rules in its first tick; a structural change is refused after F8 too, its file and member named, and the new game
/// plays by the last rules that passed.
#[test]
fn f8_restarts_the_game_with_the_directory_read_again() {
    let _one = one_at_a_time();
    let dir = temp_dir("f8");
    write(&dir, PEACEKEEPER_FILE, DAMAGE_100);
    let mut app = game(Some(&dir), None);
    while tick(&app) < 40 {
        app.update();
    }
    let generation = app.world().resource::<SimRes>().generation();
    write(&dir, PEACEKEEPER_FILE, r#"{"unit": {"attack": {"damage": 20}}}"#);
    app.world_mut().resource_mut::<InjectedInput>().tap_key(KeyCode::F8);
    app.update();
    assert_ne!(app.world().resource::<SimRes>().generation(), generation, "a new game");
    assert_eq!(tick(&app), 1, "its first tick");
    assert_eq!(set_rules_ticks(&app), [1]);
    app.update();
    assert_eq!(damage(&app), 20.0, "the directory read again");
    write(
        &dir,
        "defs/gdo/structures/barracks.json",
        r#"{"footprint": {"Tiles": {"w": 4}}}"#,
    );
    app.world_mut().resource_mut::<InjectedInput>().tap_key(KeyCode::F8);
    app.update();
    assert_eq!(tick(&app), 1);
    assert_eq!(
        feedback(&app),
        Some(
            "defs/gdo/structures/barracks.json: footprint.Tiles.w: structural, which needs a change to the Rust \
             default and a rebuild"
        )
    );
    assert_eq!(set_rules_ticks(&app), [1], "the last rules that passed");
    app.update();
    assert_eq!(damage(&app), 20.0);
    let _ = fs::remove_dir_all(&dir);
}

/// F8 works with the playtest menu open, as the menu's Restart does, and closes it: at the game's end, the result
/// under the menu, a tuning can be tried again at once.
#[test]
fn f8_restarts_the_game_from_the_playtest_menu() {
    let _one = one_at_a_time();
    let dir = temp_dir("f8_menu");
    write(&dir, PEACEKEEPER_FILE, DAMAGE_100);
    let mut app = game(Some(&dir), None);
    while tick(&app) < 20 {
        app.update();
    }
    app.world_mut().resource_mut::<InjectedInput>().tap_key(KeyCode::F10);
    app.update();
    assert_eq!(*app.world().resource::<PlaytestMenu>(), PlaytestMenu::Paused);
    let (paused_at, generation) = (tick(&app), app.world().resource::<SimRes>().generation());
    app.update();
    assert_eq!(tick(&app), paused_at, "paused");
    write(&dir, PEACEKEEPER_FILE, r#"{"unit": {"attack": {"damage": 30}}}"#);
    app.world_mut().resource_mut::<InjectedInput>().tap_key(KeyCode::F8);
    app.update();
    assert_eq!(*app.world().resource::<PlaytestMenu>(), PlaytestMenu::Closed);
    assert_ne!(app.world().resource::<SimRes>().generation(), generation, "a new game");
    assert_eq!((tick(&app), set_rules_ticks(&app)), (1, vec![1]));
    app.update();
    assert_eq!(damage(&app), 30.0, "the directory read again");
    let _ = fs::remove_dir_all(&dir);
}
