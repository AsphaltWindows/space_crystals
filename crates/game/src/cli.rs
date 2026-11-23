//! Command-line flags, parsed by hand. `ARCHITECTURE.md > QA mode` documents them for agents.

use std::path::{Path, PathBuf};

use bevy::math::UVec2;
use sc_sim::{PlayerId, Scenario, Tick, scenarios};

use crate::AppConfig;
use crate::platform::DisplayMode;
use crate::qa::{QaConfig, ScreenshotRequest};

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
    pub player: u8,
    pub window_size: UVec2,
    pub fast: bool,
    pub screenshots: Vec<ScreenshotRequest>,
    pub exit_at_tick: Option<Tick>,
    pub dump_state: Option<PathBuf>,
    pub windowed: bool,
    pub headless: bool,
    pub help: bool,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            scenario: None,
            map: None,
            player: 1,
            window_size: DEFAULT_WINDOW_SIZE,
            fast: false,
            screenshots: Vec::new(),
            exit_at_tick: None,
            dump_state: None,
            windowed: false,
            headless: false,
            help: false,
        }
    }
}

pub fn usage() -> String {
    format!(
        "\
Usage: cargo run -- [flags]

Game:
  --scenario <name>           Start a built-in scenario: {names} (default {DEFAULT_SCENARIO})
  --map <file.json>           Start a scenario loaded from a JSON file instead
  --player <n>                Show player n's view (default 1)
  --window-size <W>x<H>       Window or offscreen image size in pixels (default {w}x{h})
  --fast                      Run as many ticks per frame as possible instead of 16 per second

QA (any of these makes the run reproducible: keyboard and mouse are ignored, frames render offscreen):
  --screenshot <tick>:<path>  Save a PNG of exactly that tick (repeatable)
  --exit-at-tick <n>          Exit at tick n, once its screenshots are saved. A run without a window
                              defaults to the last screenshot's tick.
  --dump-state <path>         Write the state as JSON when the run ends
  --windowed                  Show the QA run in a window instead of rendering offscreen
  --headless                  No window and no rendering at all (no screenshots)

  -h, --help                  Show this help",
        names = scenarios::NAMES.join(", "),
        w = DEFAULT_WINDOW_SIZE.x,
        h = DEFAULT_WINDOW_SIZE.y,
    )
}

/// Parses the flags (without the program name).
pub fn parse(args: &[String]) -> Result<CliOptions, String> {
    let mut options = CliOptions::default();
    let mut args = args.iter();
    while let Some(flag) = args.next() {
        let mut value = || args.next().cloned().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--scenario" => options.scenario = Some(value()?),
            "--map" => options.map = Some(value()?.into()),
            "--player" => options.player = parse_player(&value()?)?,
            "--window-size" => options.window_size = parse_size(&value()?)?,
            "--fast" => options.fast = true,
            "--screenshot" => options.screenshots.push(parse_screenshot(&value()?)?),
            "--exit-at-tick" => options.exit_at_tick = Some(parse_tick(&value()?)?),
            "--dump-state" => options.dump_state = Some(value()?.into()),
            "--windowed" => options.windowed = true,
            "--headless" => options.headless = true,
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

/// Loads the scenario and checks the flags against it and each other.
pub fn into_config(options: CliOptions) -> Result<AppConfig, String> {
    let scenario = match (&options.map, &options.scenario) {
        (Some(_), Some(_)) => return Err("--map and --scenario exclude each other".into()),
        (Some(path), None) => load_scenario_file(path)?,
        (None, name) => {
            let name = name.as_deref().unwrap_or(DEFAULT_SCENARIO);
            scenarios::by_name(name)
                .ok_or_else(|| format!("unknown scenario {name:?}; built-in: {}", scenarios::NAMES.join(", ")))?
        }
    };
    let player = PlayerId(options.player);
    if !scenario.players.iter().any(|p| p.id == player) {
        let ids: Vec<String> = scenario.players.iter().map(|p| p.id.0.to_string()).collect();
        return Err(format!(
            "scenario {:?} has no player {} (players: {})",
            scenario.name,
            options.player,
            ids.join(", ")
        ));
    }
    if options.windowed && options.headless {
        return Err("--windowed and --headless exclude each other".into());
    }
    if options.headless && !options.screenshots.is_empty() {
        return Err("--headless renders nothing, so it can't take screenshots".into());
    }
    let mut qa = QaConfig {
        screenshots: options.screenshots,
        exit_at_tick: options.exit_at_tick,
        dump_state: options.dump_state,
    };
    let display = if options.headless {
        DisplayMode::Headless
    } else if qa.is_active() && !options.windowed {
        DisplayMode::Offscreen
    } else {
        DisplayMode::Window
    };
    if display != DisplayMode::Window && qa.exit_at_tick.is_none() {
        // Nothing can close a run without a window: end it after its last screenshot.
        qa.exit_at_tick = qa.screenshots.iter().map(|s| s.tick).max();
        if qa.exit_at_tick.is_none() {
            return Err("a run without a window needs --exit-at-tick".into());
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
    for path in qa.screenshots.iter().map(|s| &s.path).chain(&qa.dump_state) {
        create_parent_dir(path)?;
    }
    Ok(AppConfig {
        scenario,
        player,
        display,
        window_size: options.window_size,
        qa,
        fast: options.fast,
    })
}

/// Reads a `Scenario` from a JSON file (the representation is `sc_sim`'s serde format).
pub fn load_scenario_file(path: &Path) -> Result<Scenario, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("can't read {}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{} is not a scenario: {e}", path.display()))
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
        assert_eq!(config.scenario.name, DEFAULT_SCENARIO);
        assert_eq!(config.player, PlayerId(1));
        assert_eq!(config.display, DisplayMode::Window);
        assert_eq!(config.window_size, DEFAULT_WINDOW_SIZE);
        assert!(!config.qa.is_active() && !config.fast);
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
    fn a_map_file_round_trips_through_json() {
        let path = std::env::temp_dir().join(format!("sc_cli_map_{}.json", std::process::id()));
        let scenario = scenarios::empty(5, 4);
        std::fs::write(&path, serde_json::to_string(&scenario).unwrap()).unwrap();
        let loaded = load_scenario_file(&path);
        std::fs::remove_file(&path).unwrap();
        assert_eq!(loaded.unwrap(), scenario);
    }
}
