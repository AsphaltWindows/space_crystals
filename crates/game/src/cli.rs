//! Command-line flags, parsed by hand. `ARCHITECTURE.md > QA mode` documents them for agents.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use bevy::math::UVec2;
use sc_sim::{MapDef, PlayerId, Scenario, SimConfig, Tick, scenarios};

use crate::editor::actions::EditorAction;
use crate::editor::{EditorConfig, files};
use crate::platform::DisplayMode;
use crate::qa::{QaConfig, ScreenshotRequest};
use crate::{AppConfig, PlaybackSource, replay};

/// The scenario `cargo run` starts without flags.
pub const DEFAULT_SCENARIO: &str = "dev_skirmish";
/// Window (or offscreen image) size without `--window-size`.
pub const DEFAULT_WINDOW_SIZE: UVec2 = UVec2::new(1280, 720);
/// Largest accepted `--window-size` side, a safe GPU texture size.
const MAX_WINDOW_SIDE: u32 = 8192;

/// The flags as given, before the scenario is loaded.
#[derive(Clone, Debug, PartialEq)]
pub struct CliOptions {
    pub scenario: Option<String>,
    pub map: Option<PathBuf>,
    /// `--editor`, with the file to edit if one was given.
    pub editor: bool,
    pub editor_path: Option<PathBuf>,
    pub editor_script: Option<PathBuf>,
    pub player: u8,
    pub window_size: UVec2,
    pub fast: bool,
    /// Debug commands are on unless `--no-debug`.
    pub debug: bool,
    pub screenshots: Vec<ScreenshotRequest>,
    pub exit_at_tick: Option<Tick>,
    pub dump_state: Option<PathBuf>,
    pub commands: Option<PathBuf>,
    pub input_script: Option<PathBuf>,
    pub show_paths: bool,
    /// The fog of war is on unless `--no-fog`.
    pub fog: bool,
    pub windowed: bool,
    pub headless: bool,
    /// `--replay`: the replay file to play back.
    pub replay: Option<PathBuf>,
    /// `--record`: the file to record the game to.
    pub record: Option<PathBuf>,
    /// `--no-record`: games are recorded unless this is given.
    pub no_record: bool,
    pub help: bool,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            scenario: None,
            map: None,
            editor: false,
            editor_path: None,
            editor_script: None,
            player: 1,
            window_size: DEFAULT_WINDOW_SIZE,
            fast: false,
            debug: true,
            screenshots: Vec::new(),
            exit_at_tick: None,
            dump_state: None,
            commands: None,
            input_script: None,
            show_paths: false,
            fog: true,
            windowed: false,
            headless: false,
            replay: None,
            record: None,
            no_record: false,
            help: false,
        }
    }
}

pub fn usage() -> String {
    format!(
        "\
Usage: cargo run -- [flags]

Game:
  --scenario <name>           Start a built-in map: {names} (default {DEFAULT_SCENARIO})
  --map <file.json>           Start a map file instead
                              (both with the default setup: a GDO player in every slot, player n in slot n)
  --player <n>                Show player n's view (default 1)
  --window-size <W>x<H>       Window or offscreen image size in pixels (default {w}x{h})
  --fast                      Run as many ticks per frame as possible instead of 16 per second
  --no-debug                  Reject debug commands, as a real match does (they are on by default)

Map editor:
  --editor [file.json]        Edit a map file (created on the first save if it doesn't exist yet).
                              Without a file: a new {nw}x{nh} map, saved as maps/<name>.json.
                              With --scenario <name>: a built-in map, saved as maps/<name>.json.
                              --player <n> chooses the slot that play-tests (F5) start as.
                              Play-tests take debug commands unless --no-debug.
  --editor-script <file.json> Apply a JSON list of editor actions on the first frame (QA)

Replays (a game is recorded to {replays}/<UTC time>_<map>.json; not the editor and its play-tests):
  --replay <file.json>        Play a recorded game back, with its own map and game options; it pauses at
                              the end. Keys (placeholders): Space pauses and resumes, . plays one tick,
                              - and = change the speed (x0.5, x1, x2, x4, max), F2 switches the player, F3
                              the fog. --fast starts at max speed. The QA flags below work with it.
  --record <file.json>        Record the game to this file instead (QA runs record only with this flag)
  --no-record                 Don't record the game

QA (any of these makes the run reproducible: keyboard and mouse are ignored, frames render offscreen):
  --screenshot <tick>:<path>  Save a PNG of exactly that tick (repeatable)
  --exit-at-tick <n>          Exit at tick n, once its screenshots are saved. A run without a window
                              defaults to the last screenshot's tick, or else to the end of the --replay.
  --dump-state <path>         Write the state as JSON when the run ends
  --commands <file.json>      Apply player commands at given ticks: a JSON list of
                              {{\"tick\": 2, \"player\": 1, \"command\": {{\"Order\": {{...}}}}}}
  --input-script <file.json>  Drive the virtual keyboard and mouse at given ticks: a JSON list of
                              {{\"tick\": 3, \"cursor\": [640, 300], \"press\": [\"MouseLeft\"], \"tap\": [],
                              \"release\": []}} (buttons: MouseLeft, ShiftLeft, Digit1, KeyA, F2, Tab, ...)
  --show-paths                Start with the unit path overlay on (P toggles it)
  --no-fog                    Start with the fog of war off: show everything (F3 toggles it; needs debug
                              commands or a --replay, display only)
  --windowed                  Show the QA run in a window instead of rendering offscreen
  --headless                  No window and no rendering at all (no screenshots)

  -h, --help                  Show this help",
        names = scenarios::NAMES.join(", "),
        replays = replay::REPLAYS_DIR,
        nw = files::NEW_MAP_SIZE.0,
        nh = files::NEW_MAP_SIZE.1,
        w = DEFAULT_WINDOW_SIZE.x,
        h = DEFAULT_WINDOW_SIZE.y,
    )
}

/// Parses the flags (without the program name).
pub fn parse(args: &[String]) -> Result<CliOptions, String> {
    let mut options = CliOptions::default();
    let mut args = args.iter().peekable();
    while let Some(flag) = args.next() {
        if flag == "--editor" {
            options.editor = true;
            options.editor_path = args.next_if(|a| !a.starts_with('-')).map(PathBuf::from);
            continue;
        }
        let mut value = || args.next().cloned().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--scenario" => options.scenario = Some(value()?),
            "--map" => options.map = Some(value()?.into()),
            "--editor-script" => options.editor_script = Some(value()?.into()),
            "--player" => options.player = parse_player(&value()?)?,
            "--window-size" => options.window_size = parse_size(&value()?)?,
            "--fast" => options.fast = true,
            "--no-debug" => options.debug = false,
            "--screenshot" => options.screenshots.push(parse_screenshot(&value()?)?),
            "--exit-at-tick" => options.exit_at_tick = Some(parse_tick(&value()?)?),
            "--dump-state" => options.dump_state = Some(value()?.into()),
            "--commands" => options.commands = Some(value()?.into()),
            "--input-script" => options.input_script = Some(value()?.into()),
            "--show-paths" => options.show_paths = true,
            "--no-fog" => options.fog = false,
            "--windowed" => options.windowed = true,
            "--headless" => options.headless = true,
            "--replay" => options.replay = Some(value()?.into()),
            "--record" => options.record = Some(value()?.into()),
            "--no-record" => options.no_record = true,
            "-h" | "--help" => options.help = true,
            other => return Err(format!("unknown flag {other:?}")),
        }
    }
    Ok(options)
}

fn parse_tick(s: &str) -> Result<Tick, String> {
    s.parse().map_err(|_| format!("{s:?} is not a tick number"))
}

fn parse_player(s: &str) -> Result<u8, String> {
    match s.parse::<u8>() {
        Ok(n) if n >= 1 => Ok(n),
        _ => Err(format!("{s:?} is not a player number (1, 2, …)")),
    }
}

fn parse_size(s: &str) -> Result<UVec2, String> {
    let error = || format!("{s:?} is not a size like 1280x720");
    let (w, h) = s.split_once(['x', 'X']).ok_or_else(error)?;
    let side = |v: &str| v.parse::<u32>().ok().filter(|n| (1..=MAX_WINDOW_SIDE).contains(n));
    Ok(UVec2::new(side(w).ok_or_else(error)?, side(h).ok_or_else(error)?))
}

fn parse_screenshot(s: &str) -> Result<ScreenshotRequest, String> {
    let (tick, path) = s
        .split_once(':')
        .filter(|(_, path)| !path.is_empty())
        .ok_or_else(|| format!("{s:?} is not <tick>:<path>"))?;
    Ok(ScreenshotRequest {
        tick: parse_tick(tick)?,
        path: path.into(),
    })
}

/// Loads the map, or the replay, and checks the flags against it and each other. A game's map gets the default
/// setup (a GDO player in every slot) and must be valid; the editor opens invalid maps too, to fix them, and lists
/// their problems. A replay brings its scenario, game options included.
pub fn into_config(options: CliOptions) -> Result<AppConfig, String> {
    let player = PlayerId(options.player);
    if options.editor_script.is_some() && !options.editor {
        return Err("--editor-script needs --editor".into());
    }
    let playback = match &options.replay {
        Some(path) => Some(playback_source(&options, path)?),
        None => None,
    };
    let (mut scenario, editor) = if let Some(source) = &playback {
        (source.replay.scenario.clone(), None)
    } else if options.editor {
        // Review finding: no player check here. A map without that slot still opens, and play-tests start as its
        // first slot instead.
        let (map, editor) = editor_document(&options)?;
        (Scenario::new(map), Some(editor))
    } else {
        (game_scenario(&options)?, None)
    };
    if editor.is_none() && scenario.setup.player(player).is_none() {
        return Err(format!(
            "map {:?} has no player {} (players 1 to {}, one per slot)",
            scenario.name(),
            options.player,
            scenario.setup.players.len()
        ));
    }
    if playback.is_none() {
        // A replay keeps the recorded game's options: only with them does it play out the same.
        scenario.setup.config = SimConfig { debug: options.debug };
    }
    if options.windowed && options.headless {
        return Err("--windowed and --headless exclude each other".into());
    }
    if options.headless && !options.screenshots.is_empty() {
        return Err("--headless renders nothing, so it can't take screenshots".into());
    }
    if editor.is_some() && options.commands.is_some() {
        return Err("--commands needs a game: the editor's map stays at tick 0".into());
    }
    if !options.fog && !options.debug {
        return Err("--no-fog shows what the player can't see: not with --no-debug".into());
    }
    if editor.is_some() && options.input_script.is_some() {
        return Err("--input-script needs a game: the editor's map stays at tick 0 (use --editor-script)".into());
    }
    let mut qa = QaConfig {
        screenshots: options.screenshots,
        exit_at_tick: options.exit_at_tick,
        dump_state: options.dump_state,
        commands: match &options.commands {
            Some(path) => crate::qa::load_commands(path)?,
            None => Vec::new(),
        },
        show_paths: options.show_paths,
        no_fog: !options.fog,
        input_script: match &options.input_script {
            Some(path) => crate::qa::load_input_script(path)?,
            None => Vec::new(),
        },
    };
    let display = if options.headless {
        DisplayMode::Headless
    } else if qa.is_active() && !options.windowed {
        DisplayMode::Offscreen
    } else {
        DisplayMode::Window
    };
    let replay_end = playback.as_ref().map(|p| p.replay.end_tick());
    if display != DisplayMode::Window && qa.exit_at_tick.is_none() {
        // Nothing can close a run without a window: end it after its last screenshot, or where the replay ends.
        qa.exit_at_tick = qa.screenshots.iter().map(|s| s.tick).max().or(replay_end);
        if qa.exit_at_tick.is_none() {
            return Err("a run without a window needs --exit-at-tick".into());
        }
    }
    if let Some(end) = replay_end {
        let ticks = qa.screenshots.iter().map(|s| s.tick).chain(qa.exit_at_tick);
        if let Some(tick) = ticks.chain(qa.input_script.iter().map(|s| s.tick)).find(|t| *t > end) {
            return Err(format!(
                "the replay pauses at its end, tick {end}, so tick {tick} never comes"
            ));
        }
    }
    if let Some(exit) = qa.exit_at_tick
        && let Some(late) = qa.screenshots.iter().find(|s| s.tick > exit)
    {
        return Err(format!(
            "--screenshot at tick {} comes after --exit-at-tick {exit}",
            late.tick
        ));
    }
    if let Some(editor) = &editor
        && !editor.script.contains(&EditorAction::PlayTest)
    {
        let late = qa
            .screenshots
            .iter()
            .map(|s| s.tick)
            .chain(qa.exit_at_tick)
            .find(|t| *t > 0);
        if let Some(tick) = late {
            return Err(format!(
                "the editor's map stays at tick 0, so tick {tick} never comes: use tick 0, or start a play-test \
                 (PlayTest) in the --editor-script"
            ));
        }
    }
    for path in qa.screenshots.iter().map(|s| &s.path).chain(&qa.dump_state) {
        create_parent_dir(path)?;
    }
    let no_game = editor.is_some() || playback.is_some();
    let record = record_file(
        options.record.as_ref(),
        options.no_record,
        no_game,
        qa.is_active(),
        scenario.name(),
    )?;
    Ok(AppConfig {
        scenario,
        player,
        display,
        window_size: options.window_size,
        qa,
        fast: options.fast,
        editor,
        record,
        playback,
    })
}

/// The replay of `--replay`, which must be valid. It brings its own map, game options and commands.
fn playback_source(options: &CliOptions, path: &Path) -> Result<PlaybackSource, String> {
    let conflicts = [
        (options.editor, "--editor"),
        (options.scenario.is_some(), "--scenario"),
        (options.map.is_some(), "--map"),
        (!options.debug, "--no-debug"),
        (options.commands.is_some(), "--commands"),
    ];
    if let Some((_, flag)) = conflicts.into_iter().find(|(given, _)| *given) {
        return Err(format!(
            "--replay plays the recorded game with its own map, game options and commands: not with {flag}"
        ));
    }
    let replay = replay::load_replay(path)?;
    let problems = replay.scenario.validate();
    if !problems.is_empty() {
        let list: Vec<String> = problems.iter().map(|p| format!("  - {p}")).collect();
        return Err(format!(
            "{} can't be played back: {} problem(s)\n{}",
            path.display(),
            problems.len(),
            list.join("\n")
        ));
    }
    Ok(PlaybackSource {
        file: path.to_path_buf(),
        replay,
    })
}

/// The file the game is recorded to: `--record`'s (`record`), else a new one in `replays/` unless this is a QA run;
/// none in the editor or a replay's playback (`no_game`), or with `--no-record`.
fn record_file(
    record: Option<&PathBuf>,
    no_record: bool,
    no_game: bool,
    qa_run: bool,
    map_name: &str,
) -> Result<Option<PathBuf>, String> {
    match record {
        Some(_) if no_record => Err("--record and --no-record exclude each other".into()),
        Some(_) if no_game => Err("--record records a game: not the editor, nor a replay's playback".into()),
        Some(path) => {
            create_parent_dir(path)?;
            Ok(Some(path.clone()))
        }
        None if no_record || no_game || qa_run => Ok(None),
        None => Ok(Some(replay::recording_path(SystemTime::now(), map_name))),
    }
}

/// The scenario a game plays: the map of `--map`, else of `--scenario`, else the default, with the default setup.
/// It must be valid.
fn game_scenario(options: &CliOptions) -> Result<Scenario, String> {
    let (scenario, source) = match (&options.map, &options.scenario) {
        (Some(_), Some(_)) => return Err("--map and --scenario exclude each other".into()),
        (Some(path), None) => (Scenario::new(load_map_file(path)?), path.display().to_string()),
        (None, name) => {
            let name = name.as_deref().unwrap_or(DEFAULT_SCENARIO);
            let scenario = scenarios::by_name(name)
                .ok_or_else(|| format!("unknown scenario {name:?}; built-in: {}", scenarios::NAMES.join(", ")))?;
            (scenario, format!("scenario {name:?}"))
        }
    };
    let problems = scenario.validate();
    if problems.is_empty() {
        return Ok(scenario);
    }
    let list: Vec<String> = problems.iter().map(|p| format!("  - {p}")).collect();
    Err(format!(
        "{source} can't be played: {} problem(s)\n{}\n(fix them in the map editor: --editor <file.json>)",
        problems.len(),
        list.join("\n")
    ))
}

/// The editor's document and configuration: the file of `--editor <file>` (a new map named after it if it doesn't
/// exist yet), a built-in `--scenario`, or a new map.
fn editor_document(options: &CliOptions) -> Result<(MapDef, EditorConfig), String> {
    if options.map.is_some() {
        return Err("--editor takes the file itself: --editor <file.json>".into());
    }
    let (map, path) = match (&options.editor_path, &options.scenario) {
        (Some(_), Some(_)) => return Err("--editor <file> and --scenario exclude each other".into()),
        (Some(path), None) if path.exists() => (load_map_file(path)?, Some(path.clone())),
        (Some(path), None) => {
            let name = path
                .file_stem()
                .map_or(files::NEW_MAP_NAME.into(), |s| s.to_string_lossy());
            (files::new_map(&name), Some(path.clone()))
        }
        (None, Some(name)) => (
            scenarios::by_name(name)
                .ok_or_else(|| format!("unknown scenario {name:?}; built-in: {}", scenarios::NAMES.join(", ")))?
                .map,
            None,
        ),
        (None, None) => (files::new_map(files::NEW_MAP_NAME), None),
    };
    let script = match &options.editor_script {
        Some(path) => load_editor_script(path)?,
        None => Vec::new(),
    };
    let config = EditorConfig {
        path,
        maps_dir: files::default_maps_dir(),
        script,
    };
    Ok((map, config))
}

/// Reads an `--editor-script`: a JSON list of `EditorAction`s, e.g. `["FitMap", {"Press": [100, 80]}, "Release"]`.
pub fn load_editor_script(path: &Path) -> Result<Vec<EditorAction>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("can't read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{} is not a list of editor actions: {e}", path.display()))
}

/// Reads a map file (`sc_sim`'s versioned JSON map format, [`MapDef`]). The map may still be invalid: check it with
/// `MapDef::validate`.
pub fn load_map_file(path: &Path) -> Result<MapDef, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("can't read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{} is not a map file: {e}", path.display()))
}

fn create_parent_dir(path: &Path) -> Result<(), String> {
    match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => {
            std::fs::create_dir_all(dir).map_err(|e| format!("can't create {}: {e}", dir.display()))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(str::to_string).collect()
    }

    #[test]
    fn no_flags_start_the_default_game_in_a_window() {
        let config = into_config(parse(&[]).unwrap()).unwrap();
        assert_eq!(config.scenario.name(), DEFAULT_SCENARIO);
        assert_eq!(config.player, PlayerId(1));
        assert_eq!(config.display, DisplayMode::Window);
        assert_eq!(config.window_size, DEFAULT_WINDOW_SIZE);
        assert!(!config.qa.is_active() && !config.fast);
        assert_eq!(config.scenario.setup.config, SimConfig::DEBUG);
        // The default setup: a GDO player in every slot.
        let mut expected = sc_sim::GameSetup::default_for(&config.scenario.map);
        expected.config = SimConfig::DEBUG;
        assert_eq!(config.scenario.setup, expected);
    }

    #[test]
    fn every_flag_parses() {
        let o = parse(&args(
            "--scenario empty --player 2 --window-size 800X600 --fast --screenshot 5:/tmp/a:b.png \
             --screenshot 0:x.png --exit-at-tick 9 --dump-state d.json --windowed",
        ))
        .unwrap();
        assert_eq!(o.scenario.as_deref(), Some("empty"));
        assert_eq!(o.player, 2);
        assert_eq!(o.window_size, UVec2::new(800, 600));
        assert!(o.fast && o.windowed && !o.headless);
        assert_eq!(o.screenshots[0].tick, 5);
        assert_eq!(o.screenshots[0].path, PathBuf::from("/tmp/a:b.png"));
        assert_eq!(o.exit_at_tick, Some(9));
        assert_eq!(o.dump_state, Some(PathBuf::from("d.json")));
        assert!(o.debug);
        let off = into_config(parse(&args("--no-debug")).unwrap()).unwrap();
        assert_eq!(off.scenario.setup.config, SimConfig { debug: false });
    }

    #[test]
    fn bad_flags_are_errors() {
        for bad in [
            "--bogus",
            "--player",
            "--player 0",
            "--window-size 800",
            "--window-size 0x600",
            "--screenshot 5",
            "--screenshot x:a.png",
            "--exit-at-tick -1",
        ] {
            assert!(parse(&args(bad)).is_err(), "{bad}");
        }
    }

    #[test]
    fn qa_flags_render_offscreen_and_windowless_runs_must_end() {
        let config = into_config(parse(&args("--screenshot 3:s.png --screenshot 1:t.png")).unwrap()).unwrap();
        assert_eq!(config.display, DisplayMode::Offscreen);
        assert_eq!(config.qa.exit_at_tick, Some(3));
        let windowed = into_config(parse(&args("--screenshot 3:s.png --windowed")).unwrap()).unwrap();
        assert_eq!(
            (windowed.display, windowed.qa.exit_at_tick),
            (DisplayMode::Window, None)
        );
        let headless = into_config(parse(&args("--headless --exit-at-tick 4")).unwrap()).unwrap();
        assert_eq!(headless.display, DisplayMode::Headless);
        assert!(into_config(parse(&args("--headless")).unwrap()).is_err());
    }

    #[test]
    fn inconsistent_flags_are_errors() {
        for bad in [
            "--scenario nowhere",
            "--player 3",
            "--scenario empty --map m.json",
            "--map /nonexistent/map.json",
            "--headless --windowed --exit-at-tick 1",
            "--headless --screenshot 1:a.png",
            "--screenshot 5:a.png --exit-at-tick 4",
        ] {
            assert!(into_config(parse(&args(bad)).unwrap()).is_err(), "{bad}");
        }
    }

    #[test]
    fn editor_flags_parse() {
        let o = parse(&args("--editor maps/a.json --player 2")).unwrap();
        assert!(o.editor);
        assert_eq!(o.editor_path, Some(PathBuf::from("maps/a.json")));
        assert_eq!(o.player, 2);
        let o = parse(&args("--editor --scenario empty --editor-script s.json")).unwrap();
        assert!(o.editor && o.editor_path.is_none());
        assert_eq!(o.editor_script, Some(PathBuf::from("s.json")));
    }

    #[test]
    fn the_editor_starts_on_a_file_a_new_map_or_a_built_in_scenario() {
        let dir = std::env::temp_dir().join(format!("sc_cli_editor_{}", std::process::id()));
        let missing = dir.join("Frozen Lake.json");
        let editor_args = |path: &Path| vec![String::from("--editor"), path.display().to_string()];
        let config = into_config(parse(&editor_args(&missing)).unwrap()).unwrap();
        assert_eq!(config.editor.unwrap().path, Some(missing));
        assert_eq!(config.scenario.name(), "Frozen Lake");
        let terrain = config.scenario.terrain();
        assert_eq!((terrain.width(), terrain.height()), files::NEW_MAP_SIZE);

        let config = into_config(parse(&args("--editor")).unwrap()).unwrap();
        assert_eq!(config.editor.unwrap().path, None);
        assert_eq!(config.scenario.name(), files::NEW_MAP_NAME);
        let config = into_config(parse(&args("--editor --scenario dev_skirmish")).unwrap()).unwrap();
        assert_eq!(config.scenario.map, scenarios::dev_skirmish_map());

        // An invalid file opens in the editor, to be fixed there; the game refuses it.
        std::fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("bad.json");
        let mut map = scenarios::empty_map(8, 6);
        map.slots[1].start = sc_sim::TilePos::new(50, 50);
        std::fs::write(&bad, serde_json::to_string(&map).unwrap()).unwrap();
        let opened = into_config(parse(&editor_args(&bad)).unwrap()).unwrap();
        assert_eq!(opened.scenario.map, map);
        let map_args = vec![String::from("--map"), bad.display().to_string()];
        let error = into_config(parse(&map_args).unwrap()).unwrap_err();
        assert!(
            error.contains("1 problem(s)") && error.contains("off the map"),
            "{error}"
        );

        // Review finding: `--player` doesn't stop the editor from opening a file, even one without that slot or
        // without slots (play-tests start as its first slot); the game checks the player.
        let mut slotless = map.clone();
        slotless.slots.clear();
        std::fs::write(&bad, serde_json::to_string(&slotless).unwrap()).unwrap();
        let mut with_player = editor_args(&bad);
        with_player.extend(args("--player 2"));
        let opened = into_config(parse(&with_player).unwrap()).unwrap();
        assert_eq!(opened.scenario.map, slotless);
        assert_eq!(opened.player, PlayerId(2));
        let error = into_config(parse(&args("--scenario empty --player 3")).unwrap()).unwrap_err();
        assert!(error.contains("has no player 3"), "{error}");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn editor_scripts_are_json_action_lists() {
        let path = std::env::temp_dir().join(format!("sc_cli_script_{}.json", std::process::id()));
        std::fs::write(
            &path,
            r#"["FitMap", {"SetTool": "Paint"}, {"Press": [100.0, 80.0]}, "Release"]"#,
        )
        .unwrap();
        let script = load_editor_script(&path);
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            script.unwrap(),
            vec![
                EditorAction::FitMap,
                EditorAction::SetTool(crate::editor::tools::Tool::Paint),
                EditorAction::Press(bevy::math::Vec2::new(100.0, 80.0)),
                EditorAction::Release,
            ]
        );
    }

    #[test]
    fn editor_flags_are_checked() {
        for bad in [
            "--editor-script s.json",
            "--editor m.json --scenario empty",
            "--editor --map m.json",
            "--editor --scenario nowhere",
            "--editor --screenshot 5:a.png",
            "--editor --exit-at-tick 3 --headless",
            "--editor --input-script s.json",
        ] {
            assert!(into_config(parse(&args(bad)).unwrap()).is_err(), "{bad}");
        }
        assert!(into_config(parse(&args("--editor --headless --exit-at-tick 0")).unwrap()).is_ok());
    }

    /// Review finding: a file claiming a map larger than the sim's largest side fails cleanly (exit code 2 in
    /// `main`), before the map is allocated.
    #[test]
    fn a_map_file_over_the_largest_side_is_an_error() {
        let path = std::env::temp_dir().join(format!("sc_cli_huge_{}.json", std::process::id()));
        let mut json = serde_json::to_value(scenarios::empty_map(5, 4)).unwrap();
        json["terrain"] = serde_json::json!({ "tiles": [".".repeat(files::MAX_MAP_SIDE as usize + 1)] });
        std::fs::write(&path, json.to_string()).unwrap();
        let game = into_config(parse(&args(&format!("--map {}", path.display()))).unwrap()).err();
        let editor = into_config(parse(&args(&format!("--editor {}", path.display()))).unwrap()).err();
        std::fs::remove_file(&path).unwrap();
        for error in [game, editor] {
            let error = error.expect("refused");
            assert!(error.contains("too big"), "{error}");
        }
    }

    #[test]
    fn games_are_recorded_unless_told_otherwise() {
        let config = into_config(parse(&[]).unwrap()).unwrap();
        let path = config.record.expect("a game is recorded");
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        assert!(
            path.starts_with(replay::REPLAYS_DIR) && name.ends_with("_dev_skirmish.json"),
            "{name}"
        );
        for unrecorded in [
            "--no-record",
            "--headless --exit-at-tick 5",
            "--screenshot 3:s.png",
            "--editor",
        ] {
            let config = into_config(parse(&args(unrecorded)).unwrap()).unwrap();
            assert_eq!(config.record, None, "{unrecorded}");
        }
        let file = std::env::temp_dir()
            .join(format!("sc_cli_record_{}", std::process::id()))
            .join("q.json");
        let qa = format!("--headless --exit-at-tick 5 --record {}", file.display());
        assert_eq!(
            into_config(parse(&args(&qa)).unwrap()).unwrap().record,
            Some(file.clone())
        );
        std::fs::remove_dir_all(file.parent().unwrap()).unwrap();
        for bad in ["--record a.json --no-record", "--editor --record a.json"] {
            assert!(into_config(parse(&args(bad)).unwrap()).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_replay_plays_back_with_its_own_scenario_and_options() {
        let dir = std::env::temp_dir().join(format!("sc_cli_replay_{}", std::process::id()));
        let path = dir.join("game.json");
        let mut scenario = scenarios::empty(8, 6);
        scenario.setup.config = SimConfig { debug: false };
        let mut recorded = sc_sim::Replay::new(scenario);
        let mut sim = recorded.start();
        recorded.record_checked(None, &sim);
        for _ in 0..10 {
            sim.step(&Default::default());
            recorded.record_checked(Some(Default::default()), &sim);
        }
        replay::save_replay(&path, &recorded).unwrap();
        let with = |flags: &str| into_config(parse(&args(&format!("--replay {} {flags}", path.display()))).unwrap());

        let config = with("").unwrap();
        assert_eq!(config.scenario, recorded.scenario, "game options included");
        assert_eq!((config.display, config.record), (DisplayMode::Window, None));
        assert_eq!(config.playback.map(|p| p.replay), Some(recorded.clone()));
        assert_eq!(with("--headless").unwrap().qa.exit_at_tick, Some(10), "the end");
        assert_eq!(with("--screenshot 4:x.png").unwrap().qa.exit_at_tick, Some(4));
        assert_eq!(with("--player 2 --fast --no-fog").unwrap().player, PlayerId(2));
        for bad in [
            "--scenario empty",
            "--map m.json",
            "--editor",
            "--no-debug",
            "--commands c.json",
            "--record r.json",
            "--headless --exit-at-tick 11",
            "--screenshot 11:x.png",
            "--player 3",
        ] {
            assert!(with(bad).is_err(), "{bad}");
        }
        std::fs::write(&path, "{}").unwrap();
        assert!(with("").unwrap_err().contains("is not a replay file"));
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(with("").unwrap_err().contains("can't read"));
    }

    #[test]
    fn a_map_file_round_trips_through_json() {
        let path = std::env::temp_dir().join(format!("sc_cli_map_{}.json", std::process::id()));
        let map = scenarios::empty_map(5, 4);
        std::fs::write(&path, serde_json::to_string(&map).unwrap()).unwrap();
        let loaded = load_map_file(&path);
        // A game setup is no part of a map file.
        std::fs::write(&path, serde_json::to_string(&scenarios::empty(5, 4)).unwrap()).unwrap();
        let scenario = load_map_file(&path);
        std::fs::remove_file(&path).unwrap();
        assert_eq!(loaded.unwrap(), map);
        assert!(scenario.unwrap_err().contains("is not a map file"));
    }
}
