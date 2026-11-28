# QA mode

Read this when you run or script the game for QA, or add a QA flag.

## QA mode

Pass the flags with `cargo run -- <flags>`; `cargo run -- --help` lists them. The parser is `crates/game/src/cli.rs`.

| Flag | Effect |
|---|---|
| `--scenario <name>` | Starts a built-in scenario (`sc_sim::scenarios::NAMES`) with its default setup. Default `dev_skirmish`. |
| `--map <file.json>` | Plays a map file (a `MapDef`, see Sim scenarios and tests) instead, with the default setup: a GDO player in every slot. Excludes `--scenario`. Exits with code 2, listing the problems, if `Sim::try_new` rejects it. |
| `--factions <f1,f2,...>` | The factions of the players in slots 1, 2, … (`gdo`, `colonists`, `syndicate`, `cults`, any case), for the default `--scenario`, a named one or a `--map`; the slots not named stay GDO (`GameSetup::per_slot`). More factions than slots is an error. Not with `--editor` or `--replay` (a replay records its setup). |
| `--editor [file.json]` | Opens the map editor (see Map editor) on that file, or on a new map. |
| `--editor-script <file.json>` | Applies a JSON list of `EditorAction`s in the editor's first frame. |
| `--player <n>` | Shows player `n`'s view: camera start, top bar. Default 1. In the editor: the slot play-tests start as. |
| `--window-size <W>x<H>` | Window or offscreen image size in physical pixels. Default `1280x720`. |
| `--fast` | Runs as many ticks per frame as fit in 50 ms instead of 16 per second. Screenshots are unchanged. |
| `--no-debug` | Runs the game with debug commands off (the setup's `SimConfig`), as a real match will; they are on by default. |
| `--unexplored` | Starts the map unexplored (`SimConfig::start_explored` off); by default a game and an editor play-test start with every tile explored (*Vision*). Not with `--replay` (a replay records its options). |
| `--screenshot <tick>:<path>` | Saves a PNG of exactly that tick. Repeatable, in any order; creates parent dirs. |
| `--exit-at-tick <n>` | Exits at tick `n`, after its screenshots. Windowless runs default to the last screenshot's, or else to the `--replay`'s end. |
| `--dump-state <path>` | Writes the state as JSON when the run ends (fields below). |
| `--commands <file.json>` | Applies player commands at given ticks: a JSON list of `{"tick", "player", "command"}` (`qa::TimedCommand`; `command` is a `sc_sim::Command` in serde form). Each applies in the step that reaches its tick (1 or later), a tick's commands in file order. Not with `--editor`. |
| `--input-script <file.json>` | Drives the virtual keyboard and mouse (`InjectedInput`) at given ticks: a JSON list of `{"tick", "cursor", "press", "tap", "release"}` (`qa::InputStep`; buttons by name: `MouseLeft`, `ControlLeft`, `Digit1`, `KeyQ`, `F2`, …). The sim waits at each step's tick until it is applied, and a frame more, in which a tapped button is read in full (a tapped click becomes a click in the frame after its step), so that a step acts at its tick at any pace (slice M6-S6a: under `--fast` the sim ran on to the next stop before such a click was read, its order going late). A button held across steps drags and a chord spans steps (Ctrl + 1: `press` Ctrl and `tap` 1, release Ctrl in a later step). One step a frame (slice M5-S7): a tick's steps apply in consecutive frames, so F10 and then a click in the playtest menu, which pauses the game, go at the same tick. Not with `--editor`. |
| `--show-paths` | Starts with the unit path overlay on (P toggles it). |
| `--no-fog` | Starts with the fog of war off: the client shows everything (F3 toggles it). Display only; not with `--no-debug`. |
| `--windowed` | Starts a game, the editor or a playback in a window instead of borderless full screen (*Window*); shows a QA run, which is never full screen, in a window on the user's display instead of rendering offscreen. |
| `--headless` | No window and no renderer: sim, client state and `--dump-state` only. Needs `--exit-at-tick`. |
| `--replay <file.json>` | Plays a recorded game back (see Replays and golden hashes) with its own map, players and game options: not with `--scenario`, `--map`, `--factions`, `--editor`, `--no-debug` or `--commands`. Pauses at the end, so no tick of the other flags may come after it. The other QA flags work with it. |
| `--record <file.json>` | Records the game to that file. A game is recorded to `replays/` anyway; a QA run only with this flag. Not with `--editor` or `--replay`. |
| `--no-record` | Doesn't record the game. |
| `--ai <n>[:<profile>],...` | Computer players play players `n`, … (slice A1, see *Computer players*), each with a profile, `standard` (the default), `rush` (slice A2d) or `passive`: `--ai 2`, `--ai 2:rush`, `--ai 1,2:passive`. Their factions must have one (`sc_ai::factions::with_ai`). Not with `--editor` or `--replay` (a replay records their commands). The client gives a computer's player no command; F2 still switches to it. |
| `--tuning <dir>` | Plays with the data files of `<dir>` (slice M5-X2, *Tunables*, `docs/tuning.md`): read before the first tick, its rules in the first tick's inputs (`DebugCommand::SetRules`), then polled every 0.5 s of real time. A QA run reads the directory as it starts, so the run is reproducible; a file changed during the run applies at a tick that the wall clock decides. Needs debug commands; not with `--editor` or `--replay` (a replay records the rules in its commands). |
| `--export-data <dir>` | Writes the built-in data as files into `<dir>`, creating its folders, and exits: the starting point of a `--tuning` directory. |

Taking a screenshot (no window appears; about one second plus `tick / 16` seconds, or add `--fast`):

```sh
cargo run -- --scenario dev_skirmish --player 2 --screenshot 1:/path/to/p2.png --dump-state /path/to/p2.json
```

- Any of `--screenshot`, `--exit-at-tick`, `--dump-state`, `--commands`, `--input-script` makes a *QA run*. It
  ignores keyboard and mouse, renders at scale factor 1, and renders offscreen: Bevy renders into an image on this
  machine's GPU (Vulkan, Intel Arc), with no window and no Xvfb. The sim waits at each screenshot tick until the PNG
  is saved, so the PNG shows that tick's exact state with no interpolation. Repeated runs give byte-identical PNGs.
- Exit codes: 0 when everything was saved, 1 when a QA step failed (the log says which), 2 for invalid flags.
- Once the game is over the sim steps no more: the screenshots, input steps and exit scheduled after its last tick
  happen at that tick (`qa::stop_at_game_end`, logged).
- The `--dump-state` JSON holds `tick`, `scenario` (the map's name), `players`, `objects` (every field of
  `sc_sim::Object`), the last tick's `events`, `client` (`local_player`, `camera_center` as a view point in su,
  `grid_visible`, `fog_of_war`, `recorded_ticks`, the local player's `selection` and `last_known` records, the
  `recording` file, and in a playback `replay`: its `file`, `end_tick`, number of `checkpoints` and the `desync`, if
  any) and `map` (the terrain).
- `--windowed` runs turn vsync off: with the display asleep, X11 paces a vsynced window at one frame per second.
- The playtest menu in a QA run: an `--input-script` step taps `F10` at a tick (the game pauses there, so the sim
  holds that tick for good), and a screenshot of the same tick shows the menu, the step being read during the
  screenshot's settling frames. A click on one of its buttons goes at the same tick, its centre from `MenuLayout`
  (at 1280 × 720: x 640, Resume y 263.5, Restart y 308.2, Quit y 352.8; at the game's end Restart y 421.2, Quit y
  465.8). After a Restart the ticks of later steps and screenshots are the new game's.
- Verify visual work yourself. Capture screenshots and look at the PNGs with the Read tool before you report.
- To look at another part of a map, give the slot a different camera start in the editor (or the file's
  `camera_start`) and pass the file with `--map`. In an `--input-script`, pan with middle-button drags (a press, a
  cursor move, a release: the view moves by the pixels dragged, clamped to the map): the arrow keys and the screen
  edge pan by real time, which `--fast` makes too short to rely on.
- To put units on a map in a QA run, spawn them with debug commands in a `--commands` file at tick 1 and order them
  from tick 2. Spawned objects take the ids after the starting objects, in file order (`--dump-state` lists them):

  ```json
  [{"tick": 1, "player": 1, "command": {"Debug": {"Spawn": {"kind": "Peacekeeper", "owner": 1, "at": {"Point": [1216.0, 1376.0]}}}}},
   {"tick": 2, "player": 1, "command": {"Order": {"units": [18], "kind": {"Move": {"Point": [2016.0, 1376.0]}}, "queue": false}}}]
  ```
