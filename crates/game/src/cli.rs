//! Command-line flags, parsed by hand. `ARCHITECTURE.md > QA mode` documents them for agents; [`usage`] is the help
//! text.

mod ai;
mod usage;
mod values;

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use bevy::math::UVec2;
use sc_sim::{Faction, GameSetup, MapDef, PlayerId, Scenario, SimConfig, Tick, scenarios};

use crate::ai_players::AiSlot;
use crate::editor::actions::EditorAction;
use crate::editor::{EditorConfig, files};
use crate::platform::DisplayMode;
use crate::qa::{QaConfig, ScreenshotRequest};
use crate::{AppConfig, PlaybackSource, replay};

pub use usage::usage;
use values::{parse_factions, parse_player, parse_screenshot, parse_size, parse_tick};

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
    /// `--factions`: the faction of each slot's player, in slot order (plan decision T24); `None` for the default
    /// setup, a GDO player in every slot.
    pub factions: Option<Vec<Faction>>,
    /// `--ai`: the players computers play, each with its profile.
    pub ai: Vec<AiSlot>,
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
            factions: None,
            ai: Vec::new(),
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
            "--factions" => options.factions = Some(parse_factions(&value()?)?),
            "--ai" => options.ai = ai::parse_ai(&value()?)?,
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

/// Loads the map, or the replay, and checks the flags against it and each other. A game's map gets the default
/// setup (a GDO player in every slot), or the factions of `--factions`, and must be valid; the editor opens invalid
/// maps too, to fix them, and lists their problems. A replay brings its scenario, players and game options included.
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
    if editor.is_some() && !options.ai.is_empty() {
        return Err("--ai needs a game: the editor's play-tests have no computer players".into());
    }
    ai::check(&options.ai, &scenario)?;
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
        ai: options.ai,
    })
}

/// The replay of `--replay`, which must be valid. It brings its own map, game options and commands.
fn playback_source(options: &CliOptions, path: &Path) -> Result<PlaybackSource, String> {
    let conflicts = [
        (options.editor, "--editor"),
        (options.scenario.is_some(), "--scenario"),
        (options.map.is_some(), "--map"),
        (options.factions.is_some(), "--factions"),
        (!options.debug, "--no-debug"),
        (options.commands.is_some(), "--commands"),
        (!options.ai.is_empty(), "--ai"),
    ];
    if let Some((_, flag)) = conflicts.into_iter().find(|(given, _)| *given) {
        return Err(format!(
            "--replay plays the recorded game with its own map, players, game options and commands: not with {flag}"
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

/// The scenario a game plays: the map of `--map`, else of `--scenario`, else the default, with the default setup or
/// with the factions of `--factions` (the slots it doesn't name get the GDO). It must be valid.
fn game_scenario(options: &CliOptions) -> Result<Scenario, String> {
    let (mut scenario, source) = match (&options.map, &options.scenario) {
        (Some(_), Some(_)) => return Err("--map and --scenario exclude each other".into()),
        (Some(path), None) => (Scenario::new(load_map_file(path)?), path.display().to_string()),
        (None, name) => {
            let name = name.as_deref().unwrap_or(DEFAULT_SCENARIO);
            let scenario = scenarios::by_name(name)
                .ok_or_else(|| format!("unknown scenario {name:?}; built-in: {}", scenarios::NAMES.join(", ")))?;
            (scenario, format!("scenario {name:?}"))
        }
    };
    if let Some(factions) = &options.factions {
        // Plan decision T24: `--factions` picks each slot's faction, as the editor's play-test setup does.
        let slots = scenario.map.slots.len();
        if factions.len() > slots {
            return Err(format!(
                "--factions names {} factions, and {source} has {slots} slot(s)",
                factions.len()
            ));
        }
        scenario.setup = GameSetup::per_slot(&scenario.map, factions, scenario.setup.seed);
    }
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
    if options.factions.is_some() {
        return Err("--factions sets up a game: the editor's play-tests have their own setup".into());
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
