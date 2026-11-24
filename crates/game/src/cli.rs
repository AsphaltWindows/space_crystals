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
mod tests;
