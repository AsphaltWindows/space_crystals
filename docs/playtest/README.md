# Playtesting

How to start a game of any two factions, play both sides of it or against the computer, and report what you find.
What to try in each milestone is in its own notes:

- [M2](m2.md): a GDO base, an army, a winner.
- [M3](m3.md): the Colonists from the UI.
- [M4](m4.md): the Syndicate from the UI.
- [M5](m5.md): the Cults, with how to start a playtest of all four factions, what M5 leaves out, and what to try
  with each slice's work (so far the Airstrip and the Sparrow, the Minimap and its alerts, full screen, the playtest
  menu and the map explored from the start, the Arsenal and the Rocketman, the Chop Shop and the Seeker, and
  crushing).

Two lists are generated from the code: [the controls](controls.md), every key and every faction's command panels,
which a test keeps current; and [the placeholders](placeholders.md), every number and rule the code marks as a
placeholder, to regenerate before a playtest (each says how at its top).

## Starting a pairing

The first playtest is **hot-seat**: one person plays both sides, switching with F2. A computer can also play a side
(*Playing against the computer*). A game starts from the command line, `cargo run -- --factions <a>,<b>` with `--map`
or `--scenario`. There is no start screen.

- `--factions <a>,<b>`: player 1 plays `a` in slot 1, player 2 plays `b` in slot 2. The factions are `gdo`,
  `syndicate`, `cults` and `colonists` (any case).
- `--map <file.json>` starts a map file; `--scenario <name>` starts a built-in map, `dev_skirmish` by default.
- `--player 2` starts on player 2's side. `cargo run -- --help` lists every flag.
- **The window**: the game opens in borderless full screen on the monitor it starts on; `--windowed` opens it in a
  window (`--window-size 1600x900` sets its size). **F11** or **Alt+Enter** switches between the two. While the game
  is the active window the cursor stays inside it, so the camera pans with the cursor at the screen edge (within 8
  pixels of it); Alt+Tab to another window frees the cursor, and clicking back into the game holds it again.
- **The map starts explored**: from 0:00 you see its terrain and its crystal patches dimmed, as you would after
  scouting it, but no other player's base, which you still have to find. `--unexplored` starts it black, as before.

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
- **F10** opens the playtest menu and pauses the game: **Resume** (or F10 again) goes on, **Restart** starts the same
  game over from 0:00 (the same map, factions and computer players, the map explored or not as at launch), and
  **Quit** closes the game. Nothing asks you to confirm. While it is open, the game takes no input; F2 and F3 still
  work.
- **Tab** cycles the types in the selection (Shift + Tab the other way).
- **The minimap**, the HUD's bottom-left panel, shows the whole map as the side you are on knows it, with the fog as
  on the map, and the camera's view as a white outline. Left-click it, or drag on it, to move the camera there.
  Right-click it to give the selection's right-click on that ground (a Move; Shift queues it). After A, or any command
  that awaits a target, left-click it to aim at that ground (an Attack Move). It aims at ground only, never at a unit
  or a building, and can't place a building: while you place one, a left-click on it only moves the camera.
- **Alerts**: when a unit or building of yours is hit outside the camera's view, a red ring pings on the minimap at
  that spot, shrinking and fading for 2 seconds (no sound). A fight in one place pings at most once every 10 seconds.
  **Space** centres the camera on the latest ping. The pings are the side's you are on: after F2, the other side's
  hits ping, and Space goes to that side's latest.
- **Debug keys**: F2 and F3 above, G for the grid, P for the units' paths. F2 and F3 work while debug commands are on,
  as they are unless you start with `--no-debug`: don't use it for a hot-seat game.
- **The end**: a player is defeated when they own no buildings, every structure counting
  (`core/game_rules.md > Defeat Condition`), and the last one standing wins. The game stops and shows VICTORY or
  DEFEAT, with the playtest menu under it: Restart or Quit. F2 shows the other side's.

## Playing against the computer

A computer player can take any player's slot: `--ai <n>[:<profile>],...`. It is a playtest tool
(`core/game_rules.md > Computer players (playtest tool)`). Every faction has one: the GDO, the Syndicate, the
Cults and the Colonists.

- **standard** (the default) trains its basic army without pause, gathering it at a staging point between its base
  and the map's centre. It attacks in waves on the nearest building of yours it knows of, or else your start; with
  nothing left that it knows of, a wave searches the patches near your start, then the ground around it. When you
  attack its base, the units at home defend it.
  - **The GDO** opens Plate, Generator, Generator, Barracks. It plates its whole field, builds a second Barracks and
    trains Peacekeepers: a first wave of 8, then waves of 32.
  - **The Syndicate** keeps three Agents on each crystal hatch, and one runner for each on Supplies at the Supply
    Delivery Station. A runner builds a Tunnel whenever the Supplies pay for one, up to six: first where its crystal
    hatch takes in patches no other hatch's Agents use (on `maps/playtest.json`, a second hatch on the starting field
    at about 2:00, both staffed by 3:00; on `dev_skirmish` there is no room for one), then toward the staging point.
    Its Tunnels expand a Hideout for each crystal hatch at work and an Optics Lab, and the Hideouts train Guards:
    waves of 20.
  - **The Cults** mine with two Recruits on each patch for each Storage a tile across from it, and build another
    Storage once those places are all taken. The Recruits they don't need enter the Armory, which trains Militants,
    and Gunners once a Gunworks stands. Once the bank passes 600, the field takes no more Recruits, and every second
    one of its miners leaves for the Armory, down to 8, until the bank is under 300. They build Recruitment Halls on
    open Recruitable land near their start, where each recruits at nearly full effectiveness: a first wave of 8, then
    waves of 32.
  - **The Colonists** fill their field with Drones, 3 to a patch, and the Drones their Robo Portals release deploy
    into Beacons to keep Beacon Capacity ahead of the army, two of each portal's next units. A Bio Portal warps in on
    the first Beacon, a second Robo Portal at 8 Drones, and two more Bio Portals as the Drones grow, each a tile clear
    of their other buildings (Beacons aside), so that units always get out of it. They train Sabre Legionnaires,
    converting every other one into a Rifle Legionnaire: a first wave of 8, then waves of 32.
- **rush** cuts its faction's opening short for an early army, sends a first wave of 6 as soon as they stand at its
  staging point, and then plays as standard. On `dev_skirmish`: the GDO builds its Barracks before its second
  Generator and trains Peacekeepers at once (its first wave leaves at about 1:40, standard's at 2:00); the Syndicate
  trains Guards before its second Tunnel (2:05, standard's 4:40); the Cults build their Armory first and send every
  Recruit beyond six into it (1:30, standard's 3:00); the Colonists train Sabres on their first Bio Portal before
  more Drones (2:00, standard's 2:20).
- **passive** does the same as standard but never attacks: an economy to race.
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

- `--ai 2:passive` gives you a passive opponent, and `--ai 2:rush` one that rushes:
  `cargo run -- --factions gdo,cults --ai 2:rush --map maps/playtest.json`.
- The computer on player 1 and you on player 2: `cargo run -- --factions gdo,cults --ai 1 --player 2 --map
  maps/playtest.json`.
- F2 switches to the computer's side to watch it. Its top bar says "(AI)", and you can select its units but not
  command them. F3 shows the whole map.

Two computers against each other, to the end (add `--fast` to watch it at full speed):

```sh
cargo run -- --factions gdo,gdo --ai 1,2 --map maps/playtest.json
cargo run -- --factions gdo,gdo --ai 1,2:passive --map maps/playtest.json
cargo run -- --factions gdo,gdo --ai 1:rush,2 --map maps/playtest.json
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
- Each game keeps its own replay: a game restarted from the playtest menu (F10) is recorded to a new file, with the
  time of its start, or with `--record game.json` to `game.restart1.json`, `game.restart2.json`, … beside it. The
  game before it is written complete first.
- `cargo run -- --replay <file.json>` plays a recording back: K pauses and resumes (Space did before the minimap's
  alerts took it), `.` plays one tick, `-` and `=` change the speed, F2 switches the player and F3 the fog. The pings
  show for the player you watch, and Space jumps to their latest, as in a game.
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
