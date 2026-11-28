# Replays and golden hashes

Read this when you touch replay files, saves, the state hash or the golden replays.

## Replays and golden hashes

A replay file is `sc_sim::Replay` as JSON: `scenario` (the map in its file format and the game setup, game options
included), `inputs` (the `TickInputs` of every tick: `inputs[t]` is what the step from tick `t` to `t + 1` applied)
and `checkpoints` (`{"tick", "hash"}`: `Sim::state_hash` at the end of that tick, ascending). The client writes it
with one line per tick (`replay::replay_json`), as the golden files are; a file without `checkpoints` still plays,
unchecked.

- **Checkpoints** (`sc_sim::replay`): `Replay::record_checked` notes one at tick 0 and every `CHECKPOINT_INTERVAL`
  (64) ticks, and `Replay::checkpoint` one at the last tick (the client, whenever it writes the file).
  `Replay::verify(&sim)` compares a sim played back to some tick with that tick's checkpoint, and
  `Replay::run_checked` plays a whole replay that way. A `Desync { tick, expected, actual, previous }` names the first
  checkpoint that differs and the one before it, which matched: the game diverged in between.
- **Recording** (`crates/game/src/replay/record.rs`): every game the client plays is recorded, not the map editor, its
  play-tests or a playback. The file is `--record`'s, else `replays/<UTC date>_<time>_<map>.json` under the working
  directory (git-ignored; `_2`, `_3`, … when taken). It is written when the game starts, every `AUTOSAVE_TICKS` (160)
  ticks, at the end of the game (complete, with a checkpoint of its last tick: `Recorder::is_finished`,
  `core/game_rules.md > End of Game`) and when the app exits (window closed, Ctrl+C, a QA run's end), atomically
  (`files::write_atomically`), so a crash loses a few seconds at most. QA runs record only with `--record`;
  `--no-record` turns recording off. Should the sim be replaced (`SimRes::generation`), the recording ends there.
  Each game keeps its own replay (slice M5-S7, the Director's call): a Restart from the playtest menu writes the
  recording complete to its last tick (`Recorder::close`) and records the new game to a file of its own
  (`RecordingNames`): `replays/<UTC date>_<time>_<map>.json` of its start, or with `--record <stem>.json`
  `<stem>.restart1.json`, `<stem>.restart2.json`, … beside it.
- **Playback** (`--replay`, `crates/game/src/replay/playback.rs`): the game starts from the replay's scenario with its
  game options, and each step applies the recorded inputs of its tick and nothing else. The local player can't give
  commands: the command UI's input set (`CommandUiSystems::Input`) doesn't run, and anything else pending is dropped.
  It pauses at the end, or where its game ended (a recording made outside the client may go on after it). The game
  is checked against the checkpoints at tick 0 and after every step; the first desync
  is logged as an error, fails a QA run (exit code 1) and shows in red in the top bar, and the playback goes on.
  Controls (Game context, placeholder keys): K pauses and resumes, `.` plays one tick and pauses, `-` and `=`
  step through the speeds ×0.5, ×1, ×2, ×4 and max (`playback::SPEEDS`; `--fast` starts at max), F2 switches the
  player and F3 the fog, also when the recorded game had no debug commands (the selection's and the fog's own
  systems allow them while a `Playback` runs). The top bar shows "Replay x2", ", paused"
  or ", end", and "Tick n / end". Screenshots, `--exit-at-tick`, `--dump-state`, `--input-script` (to press the
  playback keys), `--windowed` and `--headless` work with it; a windowless playback ends at the replay's end unless
  told otherwise. To check a replay quickly: `cargo run -- --replay <file> --headless --fast`.
- **Golden hashes** (`crates/sim/tests/golden/`: the games in `units.rs`, `gdo.rs`, `colonists.rs`, `syndicate.rs`,
  `cults.rs`, `garrison.rs`, `altar.rs`, `temple.rs`, `shroud.rs`, `shrine.rs`, `air.rs`, `anti_air.rs`, `vehicles.rs`,
  `hand_of_heaven.rs`, `prayer_mobile.rs`, `syndicate_vehicles.rs`, `drones.rs`, `defense.rs`, `factory.rs`,
  `syndicate_air.rs`, `artillery.rs`, `syndicate_null_chamber.rs`, `mirage.rs`, `airfield.rs`, `syndicate_sludge.rs`,
  `syndicate_transport.rs`, `dragonflies.rs`, `supplies.rs` and `main.rs`, the recording and the comparison in
  `harness.rs`): thirty-six games, a 3v3 duel,
  a 40-unit
  AttackMove battle, a 12-unit group move through a choke, a patrol that meets an enemy, a chase lost in the fog, three
  Peacekeepers that raise their Riot Shields (`Command::Ability`; one turns back half-way), walk up to a Guard under
  fire and lower them to kill it, and `gdo_opening` (slice M2-J1: `dev_skirmish` from its real start, to 2:45; player 1
  plays the fastest clean Barracks and rallies ten Peacekeepers to player 2's crystal field, where they destroy four
  plates, while player 2 plates its field and builds Generators), the Colonists' `colonists_opening` and
  `colonists_vs_gdo` (slice M3-K1, below), the Syndicate's `syndicate_opening` and `syndicate_vs_gdo` (slice
  M4-L, below), the Cults' `cults_opening` and `cults_vs_syndicate` (slice M5-L, below), `cults_garrison` (slice
  M5-N2, below), `cults_altar` (slice M5-N1b, below), `cults_temple` (slice M5-N3, below), `cults_shroud` (slice
  M5-N4b, below), `cults_shrine` (slice M5-N5, below), `cults_air` (slice M5-N7, below), `gdo_anti_air` (slice
  M5-K, below), `cults_vehicles` (slice M5-N6, below), `hand_of_heaven` (slice M5-N10, below),
  `cults_prayer_mobile` (slice M5-N11, below), `syndicate_vehicles` (slice M6-SV, below), `gdo_drones` (slice
  M6-GD, below), `gdo_defense` (slice M6-GR, below), `gdo_vehicles` (slice M6-GF, below), `syndicate_air` (slice
  M6-SD, below), `gdo_artillery` (slice M6-GA, below), `syndicate_null_chamber` (slice M6-SN, below),
  `cults_mirage` (slice M6-CM, below), `gdo_air` (slice M6-GH2, below), `syndicate_sludge` (slice M6-SB, below),
  `syndicate_transport` (slice M6-ST1, below), `gdo_dragonflies` (slice M6-GL, below) and `gdo_supplies` (slice
  M6-GS1, below), are blessed as replays
  with a checkpoint every 32 ticks and at the end, in `crates/sim/tests/golden/<name>.json`. Another, `area_attacks` (slice M4-O), is a unit test
  (`combat/projectiles/golden.rs`), as it plays test throwers (`combat::testing`): grenades in flight, blasts over a
  hidden Hideout and a group, an Attack Ground. Its file is `tests/golden/test_defs/area_attacks.json`, out of the
  client's playback of `tests/golden/*.json`, which knows only the game's types. The test plays each game from its definition, checks that the
  definition still is the blessed one, and compares the checkpoints; a mismatch fails with the first diverging tick
  and the last one that matched. Each game also asserts what it is about, at its end (`expect`: one side of the duel
  is gone, the group passed the choke, …) and along the way (`during`: the shielded Peacekeepers held their fire), so
  a re-bless can't hide a broken game. `gdo_opening`'s players play by script (a `Script`, here two
  `testing::build_order`s), which gives their commands before each step from the game as it is and checks what it
  saw at the end: both players' books balance every step, the patch of a destroyed plate keeps what it held until a
  plate covers it again, player 2's income falls (210 crystals in the 9 s before its first plate fell, 130 in the
  last 9) and it is not defeated. Its commands are recorded like the others', so the file plays back without the
  script; as they follow the game, they are compared after the checkpoints, not as its definition. The Colonists'
  games play `dev_skirmish` with player 1 as the Colonists, by `testing::colonists::skirmish`: in
  `colonists_opening` (to 1:50) player 1 gathers, trains Drones, deploys, warps in a Bio Portal on its first Beacon,
  trains two Sabre Legionnaires and converts the first; in `colonists_vs_gdo` (to 3:00) it trains eight Sabres and
  converts each to a Rifle Legionnaire to wait east of the Bio Portal, while player 2 plays `gdo_opening`'s Barracks
  line and sends its ten Peacekeepers to player 1's field in two waves of AttackMove: the Colonists' Shields take
  hits and units die on both sides. Both players' books balance every step, the Drones' deliveries included, and
  every command the scripts give is one the gate takes. The Syndicate's games play `dev_skirmish` with player 1 as
  the Syndicate, by `testing::syndicate::skirmish`: in `syndicate_opening` (to 2:15) its Agents gather the patch 3 GU
  from the crystal hatch, the Hideout, rallied on it, trains four more, the third of them builds a second Tunnel
  with the starting Supply, is ejected by it and gathers Supplies, and the first Tunnel expands an Optics Lab; in
  `syndicate_vs_gdo` (to 3:07.38, where it ends) six Guards follow behind the Optics Lab, the Hideout's rally point
  cleared so that they stay in the Tunnel Network, four of them ejected by the second Tunnel to hold the top of the
  plateau's west ramp, while player 2 plays the same Barracks line with sixteen Peacekeepers, sent in waves of six to
  the Syndicate's Tunnel nearest them: Guards and Peacekeepers die, the second Tunnel falls, then the first, which
  leaves the Hideout and the Optics Lab neutral, strands the two Guards in the network and defeats player 1; the game
  is over in its last tick, as the client's playback stops where a game ends. Both players' books balance
  every step, and player 1's Supplies too (`testing::syndicate::SupplyBooks`). The Cults' games play `dev_skirmish`
  with player 1 as the Cults, by `testing::cults::skirmish`: in `cults_opening` (to 2:45) its Recruits are dealt to
  the field, and the line builds a second Storage, a second Hall east of the hill (100% Effectiveness) and an Armory,
  and trains four Militants; in `cults_vs_syndicate` (to 2:55, where it ends) it builds the Armory first, then a
  Gunworks, and trains eight Militants and three Gunners (slice M5-N1a's first step) of the Recruits it has no seat
  for, which attack-move in a wave to player 2's Tunnel, while player 2 plays the Syndicate: two Agents, an Optics Lab,
  a Laser Turret from the Tunnel's ExpandMenu (slice M5-Y) and four Guards at its patch. Three Militants die
  before the turret is destroyed, and then the Tunnel falls, which defeats player 2. Both players' books balance
  every step, player 2's Supplies too. In `cults_garrison` (to 0:40, on a test map) three Militants and a Gunner go
  into a Garrison through its door, and four Agents and two Grenadiers attack-move onto it: the four fire from the
  footprint, farther than either, and kill an Agent; the Grenadiers' blasts land on it and hurt no one inside
  (`during`, every tick); one brings it down, and the four come out alive and unhurt and kill the rest (a Storage
  keeps player 1 in the game). In `cults_altar` (to 1:00, on a test map, debug commands on for the Cults' 100 Space
  Crystals) three Militants go into an Altar, which queues one order of each type: they come out at their orders'
  ticks with MaxHP 60, 30 and 50 at full HP (`during`), and kill two Agents; none falls. In
  `cults_temple` (to 0:25, on a test map, debug commands on for the research's 100 Space Crystals) six Recruits staff
  a Temple, which queues Enhanced Constitution Level 1 at tick 80 and completes it 160 frames on, in tick 239, while
  three Enhanced Constitution Militants fight three Agents: every Militant has 80 MaxHP from that tick and 20 HP
  more, its damage kept (`during`); one falls all the same, and the two others kill the Agents. In `cults_shroud` (to
  0:21, on a test map) six Militants hold under three Disciples' circles, and three Guards and a Grenadier attack-move
  onto them: the outer two Disciples, 8 grid units apart, channel Shroud from tick 1; the middle one channels at tick
  67, which joins the two clouds and releases both buffers at once, each over its own circle, stops at 112, its
  circle lingering until 136 (slice M6-CM's linger, which moved this golden and no other), which splits the cloud and
  releases the joint buffer over its three circles, and joins them again at 148; the outer two run dry at 178, their
  circles gone at 202, the middle one at 292, gone at 316. While a
  unit is under a cloud every hit on it is a release, and at each of those ticks every buffer goes at once
  (`during`). Two Guards fall while the clouds are up, and no unit of player 1's; in the open two of the three
  Militants in front fall (all three before the linger), and the rest kill the last Guard and the Grenadier. In
  `cults_mirage` (slice M6-CM, to 1:50.6, on a test map) a Disciple casts Mirage in tick 1 and sends its north and
  east illusions into two Syndicate Guards' fire, which kills them at ticks 113 and 137, each hit on an illusion twice a
  Guard's (`during`); back to 50 energy, the Disciple casts again at 1610, which removes the two left (no death) and
  makes four more, walks into the Guards' fire and dies at 1753, its new illusions removed with it. In
  `cults_shrine` (to 1:28, on a test map) eight
  Recruits go into a Shrine at tick 1 and charge it 2 energy a second once all are in; three Agents attack-move onto
  three Militants at tick 520, and the middle ones of both sides fall together; at tick 680 the Shrine casts Health
  regeneration over the two Militants left, both wounded: each heals 0.125 HP a tick after the tick's hits for 240
  ticks, up to its MaxHP, the Agents in the circle get nothing, and the two kill the Agents; at tick 1072 it casts
  Energy regeneration on a Disciple, whose energy rises 1.5 a second for 320 ticks (`during`, every tick, with the
  Shrine's energy). In `cults_air` (to 1:30, on a test map, debug commands on for the three Sparrows' 225 Space
  Crystals) three Recruits go into an Airstrip, which queues three Sparrows: each takes off in its tick, the Recruit
  inside the longest first, over the runway's B end on its takeoff run (`during`), and circles the rally point; at
  tick 1080 the three attack-move onto a Militant and a Guard behind it (spawned for player 2, the Syndicate: the two
  infantry that hit air), and strafe them: they kill the Militant, which outranges them and takes one with it, and hit
  the Guard, which kills the other two; every hit of the game is a Sparrow's on them or theirs on a Sparrow. In
  `gdo_anti_air` (to 1:15, on a test map, debug commands on for player 1's 450 Space Crystals) the Deployment Center
  constructs an Arsenal and places it as it is ready, the Barracks refusing W until then, and the Barracks trains three
  Rocketmen, each out in its tick (`during`); at tick 820 they walk within range of two Sparrows of player 2's, spawned
  on Hold, circling, and fire normal missiles, which the Sparrows dodge, all but one, whose burst in the air grazes a
  Sparrow's corner for 0.34 HP (re-blessed in slice M5-S8 for design change 0445's bursts of 40 over 32 su); at tick
  1000 the three lock onto one Sparrow and, Shift-queued, onto the other, and each falls to three locked missiles,
  each a whole burst of 19.9 on its centre; every missile of the game is a Rocketman's at a Sparrow, normal before the
  locks and locked after, and no burst, all in the air, hits the Rocketmen under them. In `cults_vehicles` (to 1:15, on
  a test map, debug commands on for the two Seekers' 150 Space Crystals) four Recruits go into a Chop Shop, which queues
  two Seekers, each of the two Recruits inside the longest of those left: each comes out in its tick half inside the
  middle of side C, facing west, the first of its Recruits, the second gone (`during`), and drives to the rally point
  (re-blessed in slice M5-S8: the Chop Shop starts with its default rally point, 3 grid units out of side C, until
  the game sets its own in tick 64, so the checkpoints of ticks 0 and 32 alone moved); at tick 760 the two
  attack-move onto two Militants and two Guards behind them (player 2's, the Syndicate), firing their turrets on the
  move: they kill the Militants, a Guard kills one Seeker, and the other kills both Guards; every hit of the game is a
  Seeker's on them or theirs on a Seeker. In `hand_of_heaven` (to 1:08.75, on a test map, debug commands on for its
  150 Space Crystals) three Recruits go into a Chop Shop, which queues a Hand of Heaven (W) once they are in: it comes
  out in its tick, the Recruit inside the longest, the other two gone (`during`), and drives to the default rally
  point; at tick 620 it is ordered to attack player 2's Deployment Center 20 grid units west, drives into range and
  shells it from 13.8 grid units off, beyond its own sight and the Deployment Center's, which only a Recruit on Hold
  beside it sees: two missiles 150 ticks apart, each of 592.9 (600 less FullArmor 16 times the 0.44 of the footprint
  the circle covers), which player 2 sees nothing of until the first lands (the reveal); in the tick after the second
  launch it is sent back to the rally point and drives off while the missile flies, which then brings the
  Deployment Center down; every hit of the game is the Hand of Heaven's on it.
  In `cults_prayer_mobile` (to 1:54, on an open test map) a Prayer-mobile
  casts Inspire at tick 801, its 75 energy reached, by four Recruits as they set off for player 2's Temple: they walk
  5.5 su a tick for its 10 s and 4.0 after, and punch it every 9 ticks while it lasts, every 12 after; another casts
  Suppress at tick 1601 over a fight between three of player 1's Militants and six of player 2's Recruits charging
  them: the Militants and the Recruits in its arc, of both players, land no hit from 8 ticks after the cast (a punch
  already under way, `BLOW`) to the end of its 4 s, and fight on after. In `syndicate_vehicles` (to 1:33.75, on an open
  test map, debug commands on for player 1's Tier 2 and its 625 Space Crystals) a Tunnel expands a Vehicle Assembly
  behind a Chemical Plant (480 frames), which trains three Assault Bikes into the Tunnel Network: the first, with no
  rally point, is ejected by hand, the other two auto-ejected by a surface rally point in the tick each is finished,
  each out of Side A's exit path, facing north (`during`); at tick 1262 they AttackMove onto four Guards on Hold in a
  block, shell them from beyond the Guards' range, the first round's blast hitting all four, and kill them all by tick
  1476 without a hit on a Bike. In `gdo_drones` (to 1:35, on a test map, debug commands on for player 1's Supply) a
  Barracks trains a Drone Operator for 100 Space Crystals and 1 Supply, and its four drones end each in its own way:
  the first explodes on a group of player 2's Peacekeepers, the second falls past its leash and explodes there, the
  third runs out of battery, and the fourth falls with its operator, which the Peacekeepers kill; between them the
  operator buys each next drone once the last has ended. In `gdo_defense` (to 1:34, debug commands on for player 1's
  Space Crystals) a Deployment Center places a Radar Station, which scans three Sparrows holding to the east, and a
  Defense System, which builds two Missile Turrets; the second takes player 1 to 70 generated for 80 consumed as the
  Sparrows raid the base, so that the turrets fire slowed, banking a share of each tick, and the station's cooldown
  runs slow, until a third Generator placed mid-fight brings both back to full speed; the turrets' homing missiles
  bring the three down without a miss, and the station scans again in tick 1300, its cooldown over at the end of 1297,
  16 ticks later than at full Power. In `gdo_vehicles` (to 1:40, debug commands on for player 1's Space Crystals and
  2 Supplies) a Factory trains two Raptors and two Twisters, which drive out of side B and park round its default
  rally point, the first on it and each other at the free spot nearest it (design change 0451), then attack-move onto
  four Recruits and two Sparrows: the Raptors' missiles kill the Recruits, the Twisters' machine guns, which hit air
  alone, the Sparrows, all six down by 1:34 (by 1:33 before design change 0454, whose repositioning backs a Raptor up
  from a Recruit inside the circles it turns on). In `syndicate_air` (to 1:33.75, on an open test map, debug commands on
  for player 1's Tier 3 and its 725 Space Crystals) a Tunnel expands a Forge behind its Vehicle Assembly while its
  Aerospace Assembly, behind an Optics Lab, trains three Phantoms into the Tunnel Network: the first two, with no rally
  point, are ejected together and come out over the air exit door 8 ticks apart, rising there for 16 ticks, the two at
  once for 8; a surface rally point set then auto-ejects the third in the tick it is finished (`during`: each out over
  the door, its rise begun). They AttackMove onto two of the GDO's Peacekeepers on Hold, whose attacks can't reach them,
  and a Sparrow, which lands the only hit on them, and kill all three with their Universal lasers; sent home, each flies
  to the air entry door and descends into it for 16 ticks, the door letting one down every 8 at most, two at once, and
  all three end inside. In `gdo_artillery` (to 1:43.125, on an open test map, debug commands on for player 1's 250 Space
  Crystals and 5 Supplies) a Factory, with Advanced Mechanics, trains an Artillery Rig, which drives to a point on a
  Move and deploys there on a Shift-Deploy, in tick 708; an Attack given while it deploys is held, and a Shift-Move
  queued behind it dropped as it deploys (`SimEvent::OrderDropped`). Deployed, it shells three Militants on Hold 10 grid
  units off, out of its own sight, which a Peacekeeper on Hold spots: five shells, 104 frames apart, kill the first (its
  Attack) and the third, which it picks by itself; the second, out of the Peacekeeper's sight, it never fires at. It
  undeploys in tick 1400, holds a Move given meanwhile, mobile again 48 frames on, and drives off on it. In
  `syndicate_null_chamber` (to 1:02.6, on an open test map, debug commands on for player 1's Tier 2 and its 200 Space
  Crystals, auto-targeting off) a Tunnel expands a Null Chamber behind its Hideout while five Guards walk in; finished
  at tick 482, it serves them four at a time (none done before it, four 2 s after it, the fifth 2 s later), and the
  five, ejected at 562, come out buffed in the order they went in, each until its exit's tick plus 320. Three of the
  GDO's Peacekeepers on Hold attack a sixth Guard, which stayed out, and two buffed ones, and the six kill them: a hit
  takes 7 HP of a buffed Guard and 9 of the other (`during`: 8 and 6 of them), and each buff ends in the 319th tick
  after its Guard's exit. In `gdo_air` (slice M6-GH2, to 1:40, on an open test map, debug commands on for player 1's
  150 Space Crystals and 2 Supplies) three Hornets are rebased onto an Airfield and land on its first three slots, and
  the Airfield trains a fourth, which appears landed on the last and stays there, with no rally point (`during`: on its
  slot, down). The four lift off together and AttackMove onto twelve of the Cults' Recruits on Hold, which can't hit
  air, and their missiles' bursts hit up to six at once; out of ammo, each flies home by itself, lands on the first
  slot free in id order, rearms its five missiles in 160 ticks and stays landed (`during`: all four home, full and
  idle, the tick before each raid). The second raid kills the rest; sent home (F), all four end landed and rearmed,
  every landing and take-off counted (`expect`). In `syndicate_sludge` (slice M6-SB, to 0:45, on an open test map, a
  debug command on for player 1's Tier 2, auto-targeting off) two Shadebringers lay their trails east side by side, B
  switching its Sludge off in tick 70 and A's tank empty in tick 161; four of the GDO's Peacekeepers walking south
  across them are slowed, each 2.4 su a tick at most while it holds the slow, and arrive after a fifth that crosses
  none; A backs up over its own trail, slowed; B drives into the Tunnel laying, its Sludge switched off as it goes
  in, and A after it, slowed by B's trail; the Vault refills both tanks at once, 5 a second, until both are full
  (`during`: every new patch at its layer's trail point, none on the ground past 10 s). In `syndicate_transport` (slice
  M6-ST1, to 1:34, on an open test map, debug commands for player 1's Tier 2 and its 200 Space Crystals, auto-targeting
  off) a finished Vehicle Assembly trains a Subterranean Transport behind a finished Forge, inside the network in tick
  401; ejected, it takes four Guards aboard one by one, drives through a line of three of the GDO's Peacekeepers on
  Hold, crushing each, and its Unload All, queued after the move, puts the four out round it in one tick; two board it
  again, it drives into the Tunnel with them (`during`: the EjectMenu lists it alone) and is ejected again, loaded, once
  its transit delay is over; six Peacekeepers kill it in tick 1338, and its two passengers die with it, in its tick,
  where it was (`expect`: player 1's Tunnel Space in use 4, then 7 from the Train on, then 2, the two Guards left). In
  `gdo_dragonflies` (slice M6-GL, to 1:46, on an open test map, debug commands on for player 1's 375 Space Crystals and
  4 Supplies) the Deployment Center constructs Propulsion Labs and places it at tick 401, and the Airfield, its W
  refused until then, trains a Dragonfly, which appears landed on the last slot (`during`: down, there); three
  Dragonflies rebased onto it land on the others by their landing approach. The four lift off straight up and AttackMove
  head on at four of the Cults' Sparrows, which hit back: a missile each, three Sparrows down; out of ammo, each flies
  home by itself, lands and rearms in 64 frames (`during`: all four home, full and idle, before each lift-off). A sortie
  lifts them off to circle a point in the north-west; sent home (F), they land from there, every landing and take-off
  counted (`expect`). In `gdo_supplies` (slice M6-GS1, to 2:20, on an open test map, debug commands on for player 1's
  3000 Space Crystals, their starting Supply kept) the GDO's starting Supply trains a Field Engineer; the Deployment
  Center constructs a Supply Tower, placed with its free chopper down on it (`during`), scheduled on a far station: the
  chopper waits, touches down there in the tick after each of its two deliveries and drops the 4 off at home. The
  chopper the Tower trains attaches to a Tower the map placed, scheduled on a near station a Syndicate Agent picks up at
  too, and takes what the Agent hasn't claimed (2, then 3 of a delivery it waited over); a Drone Operator is trained
  with a Supply the runs brought (`expect`: every landing, load and payment, the bank at 12 Supplies). The
  ignored
  `tests/skirmish_soak.rs`
  (slice M3-K2) plays the Colonists' two players for 10 minutes, twice, until player 1's field runs out: the
  movement invariants and the books every tick, no dangling id, no order stuck in one step beyond a bound (a waiting
  Drone re-checks within 16 ticks once its patch is free), the field's 12000 crystals all delivered, no game end, and
  the hash every 64 ticks equal over both runs and the replay
  (`cargo test -p sc_sim --release --test sim skirmish_soak -- --ignored --nocapture` prints what it saw: the longest
  step of each order, each Drone's longest gap between deliveries, the losses). Any change to the sim's rules or
  state changes the hashes:
  when the change is intended, re-bless and commit the files with it:

  ```sh
  SC_BLESS=1 cargo test -p sc_sim golden
  ```

  The suite runs in well under a second in debug. The golden files are replays, so the client plays them:
  `cargo run -- --replay crates/sim/tests/golden/duel_3v3.json` (the camera starts over the map's middle, or where
  the game sets it: the Colonists' games start it over player 1's base).
