//! The help text of the flags (`--help`, and after a flag error).

use sc_ai::{Profile, factions};
use sc_sim::scenarios;

use super::{DEFAULT_SCENARIO, DEFAULT_WINDOW_SIZE};
use crate::editor::files;
use crate::replay;

pub fn usage() -> String {
    format!(
        "\
Usage: cargo run -- [flags]

Game:
  --scenario <name>           Start a built-in map: {names} (default {DEFAULT_SCENARIO})
  --map <file.json>           Start a map file instead
                              (both with the default setup: a GDO player in every slot, player n in slot n)
  --factions <f1,f2,...>      The factions of the players in slots 1, 2, ...: gdo, syndicate, cults or
                              colonists (any case); gdo for the slots not named
  --player <n>                Show player n's view (default 1)
  --ai <n>[:<profile>],...    Computers play players n, ... (a playtest tool), each with a profile:
                              {profiles} (default {default_profile}); e.g. --ai 2, --ai 2:rush,
                              --ai 1,2:passive.
                              Their factions must have a computer player: {with_ai}. A computer's player
                              takes no commands from the client; F2 still switches the player (needs debug)
  --window-size <W>x<H>       Window or offscreen image size in pixels (default {w}x{h})
  --windowed                  Start in a window instead of borderless full screen. F11 or Alt+Enter
                              switches between the two (placeholder keys); the cursor stays in the
                              window while it is the active window
  --fast                      Run as many ticks per frame as possible instead of 16 per second
  --no-debug                  Reject debug commands, as a real match does (they are on by default)
  --unexplored                Start with the map unexplored. By default every tile starts explored for
                              every player, who remembers the terrain and the neutral objects as at the
                              start (no player's objects; the fog of war stays on)
  In a game, F10 opens the playtest menu and pauses the game: Resume (or F10 again), Restart (the
  same game over from 0:00, as launched), Quit. At the game's end it opens with the result.
  A unit or building of yours hit off screen pings red on the minimap; Space centres the camera on
  the latest ping.

Map editor:
  --editor [file.json]        Edit a map file (created on the first save if it doesn't exist yet).
                              Without a file: a new {nw}x{nh} map, saved as maps/<name>.json.
                              With --scenario <name>: a built-in map, saved as maps/<name>.json.
                              --player <n> chooses the slot that play-tests (F5) start as.
                              Play-tests take debug commands unless --no-debug, and start with the
                              map explored, as a game does, unless --unexplored. No playtest menu.
  --editor-script <file.json> Apply a JSON list of editor actions on the first frame (QA)

Replays (a game is recorded to {replays}/<UTC time>_<map>.json; not the editor and its play-tests):
  --replay <file.json>        Play a recorded game back, with its own map and game options; it pauses at
                              the end. Keys (placeholders): K pauses and resumes, . plays one tick,
                              - and = change the speed (x0.5, x1, x2, x4, max), F2 switches the player, F3
                              the fog; Space jumps to the shown player's latest alert, as in a game.
                              --fast starts at max speed. The QA flags below work with it.
  --record <file.json>        Record the game to this file instead (QA runs record only with this flag)
  --no-record                 Don't record the game
  Each game keeps its own replay: a game restarted from the playtest menu (F10) records to a new
  {replays}/<UTC time>_<map>.json, or with --record game.json to game.restart1.json, game.restart2.json, ...

QA (any of these makes the run reproducible: keyboard and mouse are ignored, frames render offscreen):
  --screenshot <tick>:<path>  Save a PNG of exactly that tick (repeatable)
  --exit-at-tick <n>          Exit at tick n, once its screenshots are saved. A run without a window
                              defaults to the last screenshot's tick, or else to the end of the --replay.
  --dump-state <path>         Write the state as JSON when the run ends
  --commands <file.json>      Apply player commands at given ticks: a JSON list of
                              {{\"tick\": 2, \"player\": 1, \"command\": {{\"Order\": {{...}}}}}}
  --input-script <file.json>  Drive the virtual keyboard and mouse at given ticks: a JSON list of
                              {{\"tick\": 3, \"cursor\": [640, 300], \"press\": [\"MouseLeft\"], \"tap\": [],
                              \"release\": []}} (buttons: MouseLeft, ShiftLeft, Digit1, KeyA, F2, Tab, ...),
                              one step a frame: F10 at a tick, then a click in the menu at the same tick
  --show-paths                Start with the unit path overlay on (P toggles it)
  --no-fog                    Start with the fog of war off: show everything (F3 toggles it; needs debug
                              commands or a --replay, display only)
  --windowed                  Show the QA run in a window of its size instead of rendering offscreen
  --headless                  No window and no rendering at all (no screenshots)

  -h, --help                  Show this help",
        names = scenarios::NAMES.join(", "),
        profiles = profile_names(),
        default_profile = Profile::default(),
        with_ai = factions::with_ai()
            .into_iter()
            .map(factions::name)
            .collect::<Vec<_>>()
            .join(", "),
        replays = replay::REPLAYS_DIR,
        nw = files::NEW_MAP_SIZE.0,
        nh = files::NEW_MAP_SIZE.1,
        w = DEFAULT_WINDOW_SIZE.x,
        h = DEFAULT_WINDOW_SIZE.y,
    )
}

/// The profiles `--ai` takes, as a list: "standard, rush or passive".
fn profile_names() -> String {
    let names: Vec<&str> = Profile::ALL.iter().map(|p| p.name()).collect();
    match names.split_last() {
        Some((last, [])) => (*last).to_owned(),
        Some((last, rest)) => format!("{} or {last}", rest.join(", ")),
        None => String::new(),
    }
}
