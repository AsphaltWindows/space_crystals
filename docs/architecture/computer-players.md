# Computer players

Read this when you touch `sc_ai`, the computer players or their tests.

## Computer players

A computer player plays one player's slot (slice A1; `core/game_rules.md > Computer players (playtest tool)`): a
playtest tool, started from the command line (`--ai`, see *QA mode*), whose behaviour is the Director's placeholder
(design change 0402, answering Q0147). It lives in `crates/ai`, `sc_ai`, which depends on the sim and never on Bevy.

### The crate

- **`Ai`** plays one player with a `Profile`: `standard`; `rush` (slice A2d: its faction's opening cut short toward
  an early army, and a small first wave, `FactionAi::waves`; then as standard); or `passive` (standard's play, but it
  never attacks). Each faction's module plays its own rush (`Profile::rushes`), its numbers in `tuning::<faction>`
  (`RUSH_WAVE`, and the GDO's `RUSH_OPENING`, the Cults' `RUSH_CREW`). `Ai::think(&sim)` runs once a tick before the
  step: it reads the game as the last tick left it and returns the player's commands for the next tick. They are the
  same `PlayerCommand`s a client sends, through the same gate, and the replay records them. It gives none once its
  player is defeated or the game is over.
- **Layers**:
  - `view`: what the player knows (below);
  - `turn`: one tick's commands. The gate checks each one (`View::validate`) before it is kept. The gate sees the
    game only as the last tick left it, so the turn counts what the tick's commands spend from the bank and the
    footprints they place;
  - `build`: where a building goes, the nearest spot the placement check takes, with a free ring around it;
  - `army`, shared by every faction: the staging point, the waves and the defense. A wave that knows of nothing to
    attack searches the patches near the enemy starts, then the ground around the enemy's start
    (`army::search_points`; slice A2a, for the Cults' Halls away from any patch, and since slice A2d in every game,
    for the Colonists' last Beacons too). The View tells every player's faction (`View::faction`), shown to every
    player at game start;
  - `factions`: one module per faction behind `FactionAi`. It decides what to build and train, which units fight and
    which buildings rally, and may size its army's waves (`FactionAi::waves`; the army layer's own by default).
- **Numbers**: all placeholders, in `sc_ai::tuning` (*Tunables*).
- **A faction's computer player** (slices A2a to A2c): its module, `factions/<faction>.rs`, sets its `HOOK` (`None`
  until then). The command line then accepts `--ai` for its players, and `ai_games` plays it (*Adding a faction*,
  item 20).

### The View

`View` is the crate's only way to read the game: the `Sim` is private to it. It gives fair knowledge (design change
0402: "Its knowledge must go through the same vision and fog layer a player's client uses"):
- the player's own objects, banks, Power, army cap and build area, as they are;
- other players' objects only while the player sees them (`PlayerVision::sees`, as the client draws them). A
  structure out of sight shows as the player remembers it (`vision::LastKnown`), and a unit out of sight never shows
  (`View::known`);
- the map: its terrain, the neutral objects the map placed (known from the start), and every player's start
  (`Start::all`, from the scenario's slots, which the sim doesn't keep);
- the gate's checks a client asks (placements, casts, `View::validate`). They answer as the client's ghosts do: a tile
  the player doesn't see reports only "NotVisible".

### Determinism

A computer player gives the same commands in the same game on every machine, so that its games repeat exactly: the
tests', and a bug's. (A replay's playback runs no computer players: it plays the commands they recorded.) The sim's
rules hold (*Determinism*):
- `BTreeMap`s, ties broken by `ObjectId`, floats compared with `total_cmp`;
- no clock and no randomness of its own;
- `crates/ai/clippy.toml` bans the sim's banned methods and types.

It never panics, whatever the game's state: an assumption that fails skips the action.

### The client

`crates/game/src/ai_players.rs` holds the client's side:
- **The flag**: `--ai <n>[:<profile>],...` (`cli/ai.rs`) adds one `Ai` per player to `AiPlayers`. The command line
  checks each against the game (`sc_ai::can_play`) and refuses `--ai` with `--editor` or `--replay`.
- **Thinking**: in `FixedUpdate`, `think` runs after QA's `--commands` and before `StepSim`, once for each tick the
  sim steps (none while it is paused or held). It writes into `PendingCommands`, and the step records the commands
  with every other.
- **Input**: while the local player is a computer's, the command flow's input (`CommandUiSystems::Input`) doesn't
  run. Selecting and looking still work, F2 still switches the player, and the top bar's note says "(AI)"
  (`hud::TopBarNote::Computer`).
- **Restart** (slice M5-S7): the playtest menu's Restart gives the new game new `Ai`s of the same players and profiles
  (`AiPlayers::start`, as at launch), so a restarted game plays out as a fresh launch does
  (`tests/playtest_menu/restart.rs`: an AI-against-AI game restarted at tick 600 has, 1200 ticks later, a fresh
  launch's hash; and a rush and a passive player stay so, slice A2d).

### Tests

- **The View's fog rule**: `crates/ai/src/view/tests.rs`. **The seam** (a computer player only for a faction that has
  one): `crates/ai/tests/seam.rs`.
- **AI against AI, headless**: `crates/ai/tests/games.rs`, on `games/harness.rs`. After every step the harness checks
  the invariants, that the gate takes every command, and design change 0402's Verification ("an AI doesn't attack a
  building it has never seen or scouted, apart from the known start locations"): each Attack and Attack Move must aim
  at a building the player knew of, a start, a patch, a point of the ground a wave searches around a start
  (`army::search_points`) or a point of its own. The tests:
  - the default run: standard against passive of each faction with a computer player, on `dev_skirmish` for 6
    minutes, every invariant checked; and each faction's rush against its passive the same way (slice A2d), whose
    first wave must leave before standard's would;
  - the Syndicate's growth (slice A2d): on `maps/playtest.json`, explored or not, two crystal hatches staffed by 5:00;
  - 0402's check on a game to a defeat, and on a planted target;
  - determinism: the same game twice gives the same hash, for each faction with a computer player, and with every
    faction's rush;
  - the ignored `ai_games`, `cargo test -p sc_ai --release --test ai games::ai_games -- --ignored --nocapture`. It
    plays every pairing of factions that have a computer player on every Skirmish Default map, standard against
    passive from either slot and against standard, and in each faction's mirror its rush against passive and against
    standard from either slot (slice A2d). Standard must beat its own faction's passive within 15 minutes from either
    slot, standard against standard must end in a defeat, and a rush's first wave must leave before standard's in its
    place. The Colonists' standard must bank under 500 Space Crystals on average from 3:00 to 10:00 in their mirror,
    the Cults' under 1,000 against the GDO's passive. Standard beating another faction's passive within 20 minutes is
    wanted, not required: it lists whether it did. It prints who won, when, the units trained, the waves sent, when
    the first left, each side's mean bank and income from 3:00 to 10:00 (the harness's `Side::bank` and
    `Side::income`; `harness::watch` shows an observer every step) and what `Ai::think` cost, and the rush's first
    waves beside standard's.
- **The client**:
  - `cli/ai/tests.rs`: the flags;
  - `tests/commands/computer.rs`: the input drop;
  - `tests/replay/computers.rs`: a QA run of two computer players is recorded, and the sim and the client both play it
    back to the same hash.
