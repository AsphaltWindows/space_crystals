# Playtesting

How to start a game of any two factions, play both sides of it or against the computer, and report what you find.
What to try in each milestone is in its own notes:

- [M2](m2.md): a GDO base, an army, a winner.
- [M3](m3.md): the Colonists from the UI.
- [M4](m4.md): the Syndicate from the UI.
- M5: `m5.md`, not written yet (slice M5-M writes it).

## Starting a pairing

The first playtest is **hot-seat**: one person plays both sides, switching with F2. A computer can also play a side
(*Playing against the computer*). A game starts from the command line, `cargo run -- --factions <a>,<b>` with `--map`
or `--scenario`. There is no start screen.

- `--factions <a>,<b>`: player 1 plays `a` in slot 1, player 2 plays `b` in slot 2. The factions are `gdo`,
  `syndicate`, `cults` and `colonists` (any case).
- `--map <file.json>` starts a map file; `--scenario <name>` starts a built-in map, `dev_skirmish` by default.
- `--player 2` starts on player 2's side. `cargo run -- --help` lists every flag.

**Playtest on `maps/playtest.json`.** It has `dev_skirmish`'s terrain: player 1 starts top left, player 2 bottom
right, and a plateau with two ramps lies between them. Each base stands beside an 8-patch crystal line, the reference
field of `factions/cults/economy.md > Economy check`, with room for three Storages on each long side of it, a 1-tile
gap from the line, and a way round both ends. Every faction's Starting Layout faces the line, and the camera starts
with both in view. `dev_skirmish` (`--scenario dev_skirmish`, or its file `maps/dev_skirmish.json`) keeps its own
fields, in the corners.

Every pairing on the playtest map, all 16 (for `dev_skirmish`, put `--scenario dev_skirmish` in place of
`--map maps/playtest.json`):

```sh
cargo run -- --factions gdo,gdo --map maps/playtest.json
cargo run -- --factions gdo,syndicate --map maps/playtest.json
cargo run -- --factions gdo,cults --map maps/playtest.json
cargo run -- --factions gdo,colonists --map maps/playtest.json
cargo run -- --factions syndicate,gdo --map maps/playtest.json
cargo run -- --factions syndicate,syndicate --map maps/playtest.json
cargo run -- --factions syndicate,cults --map maps/playtest.json
cargo run -- --factions syndicate,colonists --map maps/playtest.json
cargo run -- --factions cults,gdo --map maps/playtest.json
cargo run -- --factions cults,syndicate --map maps/playtest.json
cargo run -- --factions cults,cults --map maps/playtest.json
cargo run -- --factions cults,colonists --map maps/playtest.json
cargo run -- --factions colonists,gdo --map maps/playtest.json
cargo run -- --factions colonists,syndicate --map maps/playtest.json
cargo run -- --factions colonists,cults --map maps/playtest.json
cargo run -- --factions colonists,colonists --map maps/playtest.json
```

Two tests keep these working: `crates/game/tests/pairings.rs` starts each from its flags on every map, and
`crates/sim/tests/pairings/` plays each with every faction's scripted opening (on every map in its ignored run), then
removes player 2's buildings and checks that player 1 wins.

## Playing both sides

- **F2** switches to the next player: you see what they see, and select and command their units. Each side keeps its
  own camera: the first F2 to a side shows its base, and F2 back returns to where you were looking. The top bar shows
  whose side you are on.
- **The game doesn't pause** while you switch: the side you left goes on doing what you told it, and its units fight
  back by themselves.
- **F3** turns the fog of war off and on, to see everything.
- **Tab** cycles the types in the selection (Shift + Tab the other way).
- **Debug keys**: F2 and F3 above, G for the grid, P for the units' paths. F2 and F3 work while debug commands are on,
  as they are unless you start with `--no-debug`: don't use it for a hot-seat game.
- **The end**: a player is defeated when they own no buildings, every structure counting
  (`core/game_rules.md > Defeat Condition`), and the last one standing wins. The game stops and shows VICTORY or
  DEFEAT; F2 shows the other side's.

## Playing against the computer

A computer player can take any player's slot: `--ai <n>[:<profile>],...`. It is a playtest tool
(`core/game_rules.md > Computer players (playtest tool)`). Every faction has one: the GDO, the Syndicate, the
Cults and the Colonists.

- **standard** (the default) trains its basic army without pause, gathering it at a staging point between its base
  and the map's centre. It attacks in waves on the nearest building of yours it knows of, or else your start. When you
  attack its base, the units at home defend it.
  - **The GDO** opens Plate, Generator, Generator, Barracks. It plates its whole field, builds a second Barracks and
    trains Peacekeepers: a first wave of 8, then waves of 32.
  - **The Syndicate** keeps three Agents on its crystal hatch and one on Supplies at the Supply Delivery Station. That
    one builds a Tunnel toward the staging point whenever the Supplies pay for one, up to six. Its Tunnel expands an
    Optics Lab, and its Hideout then trains Guards: waves of 20.
  - **The Cults** mine with two Recruits on each patch for each Storage a tile across from it, and build another
    Storage once those places are all taken. The Recruits they don't need enter the Armory, which trains Militants,
    and Gunners once a Gunworks stands. They build Recruitment Halls on open Recruitable land near their start, where
    each recruits at nearly full effectiveness: a first wave of 8, then waves of 32.
  - **The Colonists** fill their field with Drones, 3 to a patch, and the Drones their Robo Portal releases deploy
    into Beacons to keep Beacon Capacity ahead of the army. A Bio Portal warps in on the first Beacon and two more as
    the Drones grow. They train Sabre Legionnaires, converting every other one into a Rifle Legionnaire: a first
    wave of 8, then waves of 32.
- **passive** does the same but never attacks: an economy to race.
- It knows only what its fog of war shows, as you do, and gives only the commands you could.
- How it plays is a placeholder, every number in `crates/ai/src/tuning.rs`. Report what plays badly to the Director.

You on player 1, against the GDO on the playtest map:

```sh
cargo run -- --factions gdo,gdo --ai 2 --map maps/playtest.json
cargo run -- --factions syndicate,gdo --ai 2 --map maps/playtest.json
cargo run -- --factions cults,gdo --ai 2 --map maps/playtest.json
cargo run -- --factions colonists,gdo --ai 2 --map maps/playtest.json
```

Against the Syndicate, the computer on player 2, then on player 1 with you on player 2:

```sh
cargo run -- --factions gdo,syndicate --ai 2 --map maps/playtest.json
cargo run -- --factions syndicate,gdo --ai 1 --player 2 --map maps/playtest.json
```

Against the Cults, the computer on player 2, then on player 1 with you on player 2:

```sh
cargo run -- --factions gdo,cults --ai 2 --map maps/playtest.json
cargo run -- --factions cults,gdo --ai 1 --player 2 --map maps/playtest.json
```

Against the Colonists:

```sh
cargo run -- --factions gdo,colonists --ai 2 --map maps/playtest.json
cargo run -- --factions colonists,colonists --ai 2 --map maps/playtest.json
```

- `--ai 2:passive` gives you a passive opponent.
- The computer on player 1 and you on player 2: `cargo run -- --factions gdo,cults --ai 1 --player 2 --map
  maps/playtest.json`.
- F2 switches to the computer's side to watch it. Its top bar says "(AI)", and you can select its units but not
  command them. F3 shows the whole map.

Two computers against each other, to the end (add `--fast` to watch it at full speed):

```sh
cargo run -- --factions gdo,gdo --ai 1,2 --map maps/playtest.json
cargo run -- --factions gdo,gdo --ai 1,2:passive --map maps/playtest.json
cargo run -- --factions syndicate,syndicate --ai 1,2 --map maps/playtest.json
cargo run -- --factions cults,cults --ai 1,2 --map maps/playtest.json
cargo run -- --factions colonists,colonists --ai 1,2 --map maps/playtest.json
cargo run -- --factions colonists,gdo --ai 1,2 --map maps/playtest.json
```

The replay records the computers' commands with yours.

## Replays

- Every game is recorded to `replays/<date>_<time>_<map>.json` (UTC, under the folder you ran `cargo run` from),
  written as the game starts, every 10 seconds of game time, when it ends and when you quit. `--record <file.json>`
  records to that file instead, and `--no-record` turns it off.
- `cargo run -- --replay <file.json>` plays a recording back: Space pauses and resumes, `.` plays one tick, `-` and
  `=` change the speed, F2 switches the player and F3 the fog.
- The top bar shows the tick, in a game and in a replay.

## Reporting a finding

One finding per item, with:

- **what you did, what you expected, and what happened**;
- **the replay file and the tick** where it shows;
- **the tuning directory**, if you used one (from slice M5-X2 on).

Design matters (a rule or a number that plays badly, or that the design doesn't settle) go to the **Designer**
session (`scripts/designer.sh`); bugs (the game doesn't do what the design or the notes say) go to the **Director**
session (`scripts/director.sh`).

```text
### <what happened, in one line>
- Game: cargo run -- --factions <a>,<b> --scenario <map>
- Did: <what you did>
- Expected: <what you expected>
- Happened: <what happened>
- Replay: replays/<file>.json, tick <n>
- Tuning: <directory>, or none
- For: Designer (design) / Director (bug)
```
