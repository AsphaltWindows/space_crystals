# Sim scenarios and tests — Scenario tests for the core systems

Read this when you write or find a sim test of movement, behaviours, the economy, construction, structures, the Colonists, shields, gathering or warp in.

- **Movement scenario tests** are `crates/sim/tests/movement_collision/` (passing, head-on, choke, group arrival,
  idle units, repaths, oscillation bounds, save and load in a crowd; in modules by topic: `passing`, `crowds`,
  `paths`), `group_destinations.rs` (the magic box; a
  group crowding its own spots, 0183),
  `queued_group_destinations.rs` (Shift-queued group orders),
  `movement_yield/` (who yields, a group order's units, the give-up schedule, occupied targets; 0169's 3 GU for a
  point and for a structure), `movement_crowds.rs` (blocks meeting head-on, a group through an idle block, dense
  blocks of any shape; the ignored crowd sweep), `movement_soak.rs` (200 units,
  random orders, 2 000 ticks; bounds on the share of failed moves), `movement_stuck.rs` (whose progress a waiting
  unit shares: a unit behind one whose legs begin again finds its way round), `invariants_overlap.rs` (I1's two
  exceptions and nothing more, on edited states), `crushing/` (slice M5-V4: design change 0418's spin, a line of
  enemies driven through, backing up, Guards and own infantry gone round, and the soak with crushers) and the ignored
  `movement_perf.rs`.
- **Behaviour scenario tests** are `crates/sim/tests/behaviours_attack.rs` (the approach to a gap of Range, the
  chase, a target lost from vision, a target out of reach, a unit blocked behind its owner's attacker giving up),
  `attack_behind_friends/` (slices M3-G2 and M3-G3: the way round a firing line, and the Failure rule with none,
  design change 0223's Verification), `target_lost.rs` (design change 0200: Attack,
  Follow, a queue, AttackMove and idle units as the target leaves vision), `behaviours_idle.rs` (acquiring by priority,
  the leash and the return, Hold, Move doesn't engage, the auto-targeting switch), `behaviours_travel.rs` (Follow,
  static objects, AttackMove groups and the leash, a fight replayed and saved in its middle), `attack_move_leash.rs`
  (design change 0180: the leash measured from the route planned at the start after a fight, a bait at the leash
  taken once, a detour round a blocked route as the new PathReference), `attack_move_reach.rs` (0230: enemies across
  walls passed without a chase, a rejected enemy taken within weapon range, an identical enemy swapped in, judging
  again when the enemy moves a grid unit or the known obstacles change, the paths per unit and tick),
  `attack_move_own_movement.rs` (0243: judging again once the unit has walked a grid unit from where it judged,
  tick by tick and within the bound, none while it stands, a chase that ended beyond the leash, and 0243's
  Verification against a fresh Guard on the same spot), `target_priority.rs` (0179:
  the 45° steps of rotation, the step's bound, ties by id), the unit tests in `orders/tests/` (with test-only defs:
  a unit fighting back beyond its leash, artillery out-ranging it left alone, MinRange: the back-off, the immobile
  unit's failed order, the scans skipping enemies inside it; the steps themselves),
  `behaviours_patrol.rs`, `behaviours_gate.rs` (targets must be seen; a GDO player's remembered station, by its
  record), `behaviours_soak.rs` (120 units fighting under random orders of every kind, then without orders until
  those end; its checks are invariants that survive tuning: no dangling ids, no idle unit in reach of a visible
  enemy (but one whose scan held an enemy that died in that tick), every order but Hold, Patrol and Follow ends, and
  bounded shares of refused orders, failed moves and path searches; the ignored `soak_sweep` runs seeds 1 to 40 and
  fails if any does. An order ends within the slowest unit's walk round the map's four sides for each order a unit
  still has at the last order, its current one and its queue: slice M3-K2 found seed 5's Peacekeepers with three,
  each across both walls, settling 2765 ticks after it at full speed) and the ignored `behaviours_perf.rs` and `attack_move_perf.rs` (the reach rule: 100 Guards past 100 Drones, and past 30 Generators, across a river).
  `riot_shield/` has the Riot Shield's: toggle timing, each raised-shield effect, the gate, a save in the middle
  of a toggle and its replay, the orders kept through a toggle (`orders_kept.rs`, design changes 0150, 0151), and
  the target state (`target.rs`, design changes 0198 and 0202: a second press before a deferred raise begins, fast
  presses ending in their parity, an Attack taken while lowering and carried out once down, none while raising).
  Slice M3-R's:
  `default_order.rs` (the table of unit kinds against targets, design change 0136), `remembered_targets.rs` (design
  change 0139: the gate, each order on a remembered target that stands and that was destroyed unseen, the rally
  point, no leak in lockstep games), `known_from_the_start.rs` (design change 0159's Verification),
  `follow_itself.rs` (design change 0168's Verification and the Shift-queued step) and `production/drone_rally.rs`
  (a Drone rallied on an enemy unit). In the client, `tests/commands/remembered.rs` plays them through the mouse
  with the fog on.
  Measured (release, 500 units on 256 × 256; slice M2-J2, within 7 % of M1's in an interleaved A/B): 0.13 ms a tick
  with everyone idle or on Hold in sight of the enemy, 0.29 ms on average and 0.65 ms at worst in a battle of 250
  against 250. The release build's split into codegen units moves these by a few percent with code that isn't on
  their path: slice M4-S1's battle ran 3% slower with no new work in it until `World::pick` was `#[inline]` (the
  split had left its call in `travel::march` out of line); with one codegen unit on both sides it was within noise.
- **Resource collision tests** are `crates/sim/tests/resources_block.rs` (design change 0038's verification: units
  path around a crystal field, can't walk across a Supply Delivery Station, and walk through the tile of a patch
  removed by a debug command, standing in for depletion; `economy_mining.rs` has the real thing, below).
- **Path knowledge tests** are `crates/sim/tests/path_knowledge/` (the verification points of design changes
  0040 and 0042: a hidden wall of enemy buildings, a scouted building destroyed out of sight, a crystal field never
  seen, a patch gone in the fog; own structures; an unseen structure is open ground for plans and solid for
  units; lazy grids; saves with stale knowledge; in modules by topic: `plans`, `remembered`, `collision`,
  `grids`) and the ignored `nav_perf/` (`knowledge_rebuilds_256`).
- **Economy tests** are `crates/sim/tests/economy_mining.rs` (lumps every 48 ticks and the first 48 ticks after
  placement, the cycles under the power penalty and across a change of ratio, the loads down to the residual, a plate
  gone from a patch with crystals left or from a depleted one, by a debug removal or in combat, after which a unit
  walks through the opened tile, a patch mined out in the fog that another player assumes until it sees the tile
  (design change 0042), and the links in any order) and `economy_resources.rs` (the banks and
  `SetResources`, power and the ticks a building counts in, Unit Control, a mining game replayed and saved).
- **Construction tests** are `crates/sim/tests/construction/` (each building's frames at full power and the plate's
  680 ticks at 30/51 beside an unslowed Deployment Center, the refunds and their events, every refused command changing
  nothing, a group command in id order, a Place in its orientation with power, area and grids following, a refused Place
  for each problem of the check and one refused by an earlier command of its tick, destroyed constructors' refunds by a
  debug removal and in combat, a fuzz of the gate, a save and a replay; in modules by topic: `timings`, `refunds`,
  `gate`, `placing`, `saves`). The clean Barracks opening, which slice
  M2-D1 measured there first, is the economy oracle's.
- **The economy oracle** is `crates/sim/tests/economy_oracle/`, one folder of modules; its measured results are in
  `docs/economy_oracle.md`. It runs **on demand only** (design change 0401, which closed the economy checks; their
  targets stay in the design as reference for playtests): every test is ignored, so the default `cargo test` runs
  none, and its pins are no longer maintained, so a test may fail; run it with `cargo test --release -p sc_sim --test
  sim economy_oracle -- --ignored --nocapture`. What it checked along the way has tests of its own that still run:
  refunds in `construction/refunds.rs` and `production/queue.rs`, the Syndicate's ceiling in `agent_gathering/`,
  lumps, cycles and the power penalty in `economy_mining.rs` and `construction/timings.rs`; every faction's workers
  gather in `pairings/`. **The Colonists** (slices M3-K1 and M3-K4, `colonists.rs` and `curve.rs`): the
  reference line on `testing::colonists::reference_field` (one Robo Portal training Drones without a break, every
  6th deploying) and the GDO's reference line measured the same way (what each delivered or mined over the 21 s
  centred on a time) against both columns of the table of `factions/gdo/economy.md > Economy Targets (GDO vs Colonists)`
  (`main.rs`'s `TABLE`) within ±15% (design change 0108) and its targets 1 to 4, and the saturation curve of the row by
  number of Drones against `factions/colonists/units/drone.md > Patch Occupancy`. Since design change 0236 the table and
  the curve are this measurement (slice M3-K3's), rounded; a figure out of the tolerance or a failing target is a
  question to the Designer, not tuned. **The GDO** (slice M2-J1, `played.rs` and `gdo.rs`): GDO openings on
  `maps/dev_skirmish.json` from its real start, played with `testing::build_order` (player 2 idle), against
  `factions/gdo/economy.md > Opening Economy Baseline`: plates first (the reference line: income at the table's
  times by its plates landing every 25 s, fully plated at about 2:30, the float at 1:30 and
  2:00), Generator first, the fastest clean Barracks (pinned: started in tick 769, ready at the end of 928), the
  all-in Barracks at −21, and the refunds. Income is the average over the 9 s (3 lump cycles) ending at a time; the
  tolerances on the design's trickle figures are named where they are used (a plate's first lump 3 s after it
  lands, a wait for money up to one cycle longer, a bank under a trickle's by less than a lump per plate), while
  costs, build times, lumps and the power penalty are checked exactly. `--nocapture` prints each line's timeline.
  **The Syndicate** (slice M4-L, `syndicate.rs`): the economy check of `factions/syndicate/units/agent.md > Gathering >
  Space Crystals` (design changes 0259 and 0294) on `testing::syndicate`'s layouts: one Agent's trip and the income
  of 2, 3 and 4 Agents (minutes 5 to 10 of a ten-minute game) at 3 to 6 GU within ±15% of the table, a test for each
  row (rows 0 to 2 are theoretical: the ignored `syndicate_table` measures them for the record); the ceiling, no two
  deliveries at a Tunnel less than 64 ticks apart, in every game; two stacked Tunnels at 3 GU doubling the field's
  intake; the opening at 3 GU from the Starting Conditions, its third Agent out within 0:34–0:46, both books
  balanced every tick; and the lowest one-base saturation. **The Cults** (slices M5-L, M5-S2a and M5-S3, `cults.rs`):
  the economy check of `factions/cults/economy.md > Economy check` (design changes 0346 and 0385) on
  `testing::cults`'s layouts, played by a `Cults` line through the gate with the books balanced every tick: every
  target within ±15%, those out of it pinned (`OUT_OF_TOLERANCE`), and the seats of the reference field (8 patches in
  a row, mined from both long sides: about 32). Its two scenarios start from the one starting Storage and add
  Storages as they're needed: One Hall (its 20 Recruits all mining) and Two Halls (a second Hall as early as
  affordable, then a Storage on every site; 32 mining). **The economies compared** (design changes 0381 to 0383 and
  0385, `compared.rs`): both scenarios, the GDO's reference line and the Colonists' to 6:00, each one's income over
  the 160 frames before, total mined, bank and spent at 1:30, 2:48, 4:00, 5:00 and 6:00, with
  bank + spent = starting bank + total mined at each, every figure pinned (`COMPARED`), and whether Two Halls has
  overtaken One Hall by 4:00 (`OVERTAKEN`).
- **Game start tests** are `crates/sim/tests/game_start/` (slice M2-F1: each player's Starting Layout and banks,
  the Skirmish Default restriction and completeness, the layout rules of design change 0097, a layout plate off its
  patch refused by the game's rule, a pre-placed one by the same rule, two players' layouts in each other's way, the
  outdated flag, and `dev_skirmish`'s bases seeing and reaching their fields; in modules by topic: `start`,
  `layout_rules`, `skirmish_default`) and the unit tests of
  `scenario/starting.rs` (the table, and a layout made for other Starting Conditions). `colonists_start.rs` (slice
  M3-C) has the Colonists' start on `dev_skirmish` in either slot, its Drones clear of the node and the portal, the
  validation of every faction combination and of an incomplete layout, and Beacon Capacity (design change 0143):
  seven Drones queued and the seventh waiting for room, group production past the capacity, a fall in capacity, and
  the refined banks. `syndicate_start.rs` (slice M4-B) has the Syndicate's start in either slot (the objects, 100
  Space Crystals and 1 Supply, Tunnel Space 1/11, the camera on the Tunnel), the crystal hatch 1 grid unit from a
  patch (`tunnel::hatch_square`), the nine faction combinations, an incomplete layout, a layout Tunnel by the lake
  or the edge refused (0242), and an expansion over full Tunnel Space removing nothing. `cults_start.rs` (slice M5-B)
  has the Cults' start in either slot (the objects, 0 Space Crystals, the six Recruits of the Hall's lineage, Unit
  Control 6/20, the camera on the Hall), the Hall's seventh Recruit started at once and out of its bottom door at tick
  192, `dev_skirmish`'s layout as the design asks for it (the Hall's area on the map, Recruitable and uncontested; the
  Storage a 1-tile gap from the patches; the Recruits beside it, clear of the doors), the 16 faction combinations, an
  incomplete layout and one with a Recruit too many, a layout's Hall flush against the map's edge at 70% (slice M5-S4,
  0387), and a layout's units belonging to its first Hall. `pairings/`
  (slice M5-R) starts every ordered pair of the four factions, 16, on every Skirmish Default map and plays it to a win
  (*Adding a faction*, item 19); its ignored `soak_sweep` (slice M5-Mb, `pairings/soak.rs`) plays each with four
  seeds: the openings with the Cults' Armory destroyed with Recruits inside (the evacuation and its refunds checked),
  then a fight of debug-spawned extras (the Hand of Heaven crushing, a Sparrow off the map's edge, Suppress, Inspire,
  the Shrine's two abilities, Shroud, Target Lock), checking T65's invariants (each Cults player's usages and orphans
  against their units, held units only in buildings that exist, sites within their totals) and the addendum's and
  later ones (a Glider within its overshoot bound, a crushed unit gone with no Hit, a suppressed unit starting no
  attack, a buff lasting its full time) after every tick: 192 games in about a minute. `playtest_map.rs` (slice M5-P3) has `maps/playtest.json`: generated from its
  editable part and started in the 16 pairings; each slot's field the reference line of `testing::cults::fields`
  moved, and each of its Storage sites holding the starting Storage or free for a Recruit's Build; the Hall at 100%;
  the Deployment Center seeing and reaching every patch, the crystal hatch 3 grid units from the line, and the Drones
  clear of the node and the portal. The Tunnel Area's rules are unit-tested in `placement/tests/tunnel_area.rs`, Tunnel
  Space in `economy/army/tunnel_space.rs`, the geometry in `tunnel/area.rs`; a unit inside the network keeping its
  Tunnel Space in `tests/tunnel_network/enter.rs`.
- **The building armor rule** (design change 0264, `core/entities.md > Structure Type > Building armor rule`):
  `crates/sim/tests/building_armor.rs` holds every faction's buildings, from the defs alone, to "PointArmor n,
  FullArmor n × size", with an `EXCEPTIONS` list (empty; a stale entry fails), and every underground expansion to
  PointArmor 6 (`structures/README.md > Tunnel Expansions > High armor underground`). A new building passes it as
  it is added, or is listed with its reason.
- **Structure tests** are `crates/sim/tests/structures.rs` (each type's stat block, the Barracks in all 16 ways to
  rotate and flip it and its sides, ground units walking around a footprint, PointArmor on hits, a destroyed
  Generator freeing its tiles, debug spawns kept apart but for a plate on its patch, flat ground, the optional
  orientation in files) and `structures_vision.rs` (each type's sight, the ElevationModifier, an enemy Barracks
  remembered in the fog, a plate on a patch in maps and layouts, the targets that shoot back first, idle and on an
  AttackMove, a scan reaching a large footprint by its edge), and the ignored `structures_perf.rs` (see *Tick
  pipeline > Measuring a tick*). The placement tests are listed under *Placement and the build area*.
- **Colonists tests** are `crates/sim/tests/colonists.rs` (slice M3-A: each kind spawned for two owners, its
  structures blocking every ground grid, each kind's sight, the Sabre Legionnaire striking at contact without the
  ElevationModifier, the Rifle Legionnaire's Range with it, the Drone's 8 su per tick and its path around a Robo
  Portal). Their stat blocks against the design are in `defs/colonists/tests.rs`, and `Sim::change_kind` is
  unit-tested in `sim/kinds/tests.rs`.
- **Shields and Weapon Conversion tests** are `crates/sim/tests/shields.rs` (slice M3-B: the design's cases in a
  game, two hits in one tick, the DirectionalArmor on the overflow only, the regeneration from 0 to full in 40 ticks
  and a neutral object's, the dead gone first, the hash, saves and an older save, the Shields of a portal warping
  in following its maximum and regenerating up to it, and a cancel's fraction with no minimum) and `conversion/`
  (the 4 frames, HP and Shields kept; moving or idle; a mixed group; no cancel and no shelter; the orders going on
  with the other weapon; a save and a replay; `mid_attack.rs`: the gate in every attack phase, a Convert waiting for
  the Cooldown's end with the old weapon, alone and in a group in a fight, design change 0153; `target.rs`: the
  target form, design change 0198: 4 Sabres and 6 Rifles, a press on a unit waiting, converting and converted, fast
  presses ending in their parity). The resolution's cases are unit-tested in `combat/damage.rs`, the regeneration
  in `shields.rs`, the conversion's pieces in `conversion/tests.rs`, a change waiting for the Cooldown's end and a
  second press dropping it in `sim/modes/tests.rs` (0153, 0198). In the client: `tests/combat/` (the shield bar),
  `tests/commands/convert.rs` (R: the type and the group switched at the press, a deferred unit's too, the drawn
  form, design change 0198), `tests/commands/riot_shield/` (`groups.rs`: the groups at the press; `target.rs`: 0198's
  and 0202's Verifications through the panel and the mouse) and the InfoPanel's Shields line in `hud/info_panel`.
- **Gathering tests** are `crates/sim/tests/gathering/` (slices M3-H and M3-H3, one folder of modules by topic,
  split by slice M3-K2: one Drone's trip, pinned at 81
  ticks; two Drones arriving together; waiting, the lowest id mining next; an enemy Drone on the patch; Set
  Drop-off Location in the middle of mining and of a return; depletion and the Drones going on at another patch,
  or delivering and idling when no patch is left; the Drop Off order and the idling at the node; a lost Egress
  Node; interrupted mining; every invalid worker command refused and changing nothing; a Drone rallied on a patch;
  a game replayed and saved in its middle), `gathering_rules/` (slice M3-H3, one folder: a test per Verification
  point of design changes 0123 and 0127 to 0141 that `gathering/` doesn't cover, in modules by topic: `ends`,
  `loads`, `waiting`, `reach`, `knowledge`, `drop_offs`, and slice M3-R's `remembered`: a Gather on a patch out of
  sight, scouted or known from the start, one gone unseen, one placed later and unknown) and
  `gathering_soak.rs` (the four starting Drones for 5 minutes; 24 Drones for 5 minutes, every one delivering at
  least every 30 s, the saturated income, nothing lost; the income growing with the Drones up to saturation; 9
  Drones on 3 patches, once the smallest gridlock, with a steady income; ignored: `saturation_curve`, which prints
  the measurements) and `gathering_pass_through.rs` (design change 0106's Verification: two players' Drones on one
  row; an idle Drone parked in the strip, an enemy's and the gatherer's own; a Drone ordered out of a mining Drone,
  and solid once apart; a Drone stopped inside another, not pushed apart).
- **Warp In tests** are `crates/sim/tests/warp_in/` (slice M3-E: each building's WarpInTime and the Train it takes
  from the next tick; the interpolated maxima; damage carried through, taken before or during it (0110); the
  derived footprint, its corners, flips, turns and exit sides (0121); the Robo Portal's ring, blocked by a unit of
  either player, a patch or a structure (0118); every tile Visible, an unseen unit leaking nothing (0119); the
  footprint solid from the first frame and the Beacon's 2 × 2 after a cancel (0109); no drop-off at an Egress Node
  warping in (0116); the cancels' refunds, 75 % rounded down (56, 112, and 187 for the Robo Portal, 250 since
  design change 0167), and HP (0111); Beacon Capacity kept (0112); defeat and destruction, with no
  refund for either portal (0113, 0115); the gate against `Sim::warp_in_check` in every orientation, a save, a
  replay, and a fuzz of the gate; in modules by topic: `timings`, `interpolation`, `footprints`, `under_warp_in`,
  `cancel`, `gate`, `saves`). The arithmetic and the footprint are unit-tested in `transform/tests.rs`. In the
  client: `tests/client/visuals/` (a visual rebuilt and tinted), `tests/selection/panels.rs` (a selected Beacon
  through its Warp In), and the InfoPanel's line in `hud/info_panel` and `activity.rs`.
- **Tunnel Network tests** (slice M4-C) are `crates/sim/tests/tunnel_network/` (`enter.rs`: the walk into the entry
  path, what leaving the map ends, cleared orders, the gate and the tiers with a vehicle, a Tunnel gone or
  unsuitable, the Failure rule, the transit delay of design change 0245 and the regeneration of 0249; `eject.rs`:
  the paths in all 8 orientations (0248), 10 Guards streaming out and a blocked exit spot (0250), the unit inside
  longest, the reservation and a destroyed Tunnel's queue (0251), the gate; `production.rs`: the Hideout's Agent
  into the network, the rallies of 0252 and the shared queue of 0253, a Hideout without a parent; `crowds.rs`: 20
  Guards filing in (0246), entries and exits side by side at two Tunnels, 40 Guards in and out; `saves.rs`: saves at
  every stage and a replay, the store hashed and saved only while it holds a unit), each tick under the held
  store's invariants (`check_held`). `network.rs`'s unit tests have the geometry in every orientation, the delay and
  the regeneration; `sim/network/tests.rs` an air unit refused at every tier (a test def), a unit's id, HP and load
  through `take_off_map` and `put_on_map`, and a defeated player's network. In the client:
  `tests/commands/tunnel_network.rs` (a right-click on an own Tunnel is Enter; units that go in leave the selection
  and stay in their control groups, whose recall selects those on the map).
- **Holding tests** (slice M5-F1) are `crates/sim/tests/holding/` (`entering.rs`: ten Recruits filing in, timed, from
  a block and from every side, the eleventh's Enter failing for want of room, the QA run's ejected ten going round
  to side B and back in, the gate, the right-click, an Enter whose Armory goes, a Recruit walled in against side B
  that gets in only once it can reach the doorway (slice M5-F3), a site its Recruits build that no Enter gets into
  and a right-click on assists; `eject.rs`: Eject All streaming the pool out of side C one at a time, a Recruit that
  goes in during the stream, the gate; `evacuation.rs` (slice M5-F3): design change 0349's Verification, ten
  inside the footprint in the tick, then apart and idle, the same whatever stands round it (a Storage, an enemy, a
  unit in the doorway), a tile no Recruit can stand on passed over, the evacuees of a defeated player; `loads.rs`: a
  load kept inside and out by real gathering, and through an evacuation; `saves.rs`: saves at every stage and a
  replay; `control.rs`: a Hall's Recruit inside an Armory still in its Hall's usage, inside and evacuated, and an
  orphaned point once its Hall is gone), each tick checking `testing::dangling_references`. `holding/tests.rs` has
  the entry path in every orientation, its reach only in the doorway, and the kinds let in; `sim/holding/tests.rs`
  the Eject All mark hashed and saved only while set; `defs/tests/holds.rs` every exit opposite its entrance (0347
  point 1). The ignored `holding_perf.rs` times eight Armories
  cycling, then evacuated (*Object model > Buildings that hold units*).
- **Garrison tests** (slice M5-N2) are `crates/sim/tests/garrison/` (`entering.rs`: the four kinds going in through
  the door on B, the fifth finding it full and idling, Unit Control unchanged inside, the right-click, a site that
  can't be entered and a right-click on it that assists, a damaged Garrison entered, not repaired; `eject.rs`:
  design change 0335's "Eject All releases all four together from side B", through each other, apart once clear,
  units let out at once still blocked by another's, the gate on an empty one; `fire.rs` and `protection.rs`, *Combat >
  Units that fire from inside a building*), each tick checking `testing::dangling_references`; `defs/tests/holds.rs`
  checks the one door and the occupants' fire of every def (`holds_are_consistent`). The ignored `garrisons_perf.rs`
  times ten full Garrisons firing (*Combat > Units that fire from inside a building*).
- **Shroud tests** (slice M5-N4b) are `crates/sim/tests/shroud/` (`absorption.rs`: a Militant shot once from outside
  taking the area minimum 8 ticks later, fire from under the same cloud in full, a grenade judged by where it was
  thrown from, a release without direction, a lowered turret protected and taking its share; `clouds.rs`: two
  Disciples 5 grid units apart sharing a buffer and a third 8 away not, a channel between two clouds releasing both,
  a Stop releasing a joined pair's buffer over both circles (since slice M6-CM as the stopped one's lingering circle
  vanishes); `channel.rs`: damage not ending a channel and a new order ending it, the circle lingering where it was
  as its Disciple walks off, a full pool running dry 711 ticks on and the 10 energy to start again, a queued Shroud
  ending at once below 10; (M6-CM) `linger.rs`: 0448's Verification step 8, a circle lingering 24 ticks after a Stop
  and after its Disciple's death, absorbing a shot meanwhile and releasing it as it vanishes; `state.rs`: clouds
  splitting and merging phase by phase in a replay that reproduces its hash, the clouds saved and loaded and left out
  while there are none), each tick checking the invariants, C1 among them. The
  golden `cults_shroud` is a battle under the clouds, and the ignored `shroud_perf.rs` times eight Disciples over
  sixty units under fire (*Combat > Shroud*).
- **Armory tests** (slice M5-F2) are `crates/sim/tests/armory/` (`training.rs`: Q pays 20 and a Militant comes out
  of side C 32 ticks later toward the rally point, first in first trained, X's refund, the gate, Q to three Armories;
  `release.rs`: design change 0337's Verification, X taking a Release back, a Release queued like any order;
  `eject.rs`: Eject All leaving the reserved units inside, the order units are ready in; `capacity.rs`: 0347's
  Verification, the load a trained unit drops and a released one keeps; `destruction.rs`: an Armory destroyed
  mid-training, and one holding a trained Militant waiting for its doorway, which comes out a Militant, unrefunded,
  with the rest refunded (slice M5-F3); `control.rs`: the Unit Control unchanged through training, and a Militant
  trained of a destroyed
  Hall's Recruit an orphan; `saves.rs`: saves at every stage and a replay), each tick under the invariants, R1 among
  them, and checking `testing::dangling_references`.
  `sim/producers/pool/tests.rs` has a hurt Recruit's damage, a vehicle's full HP, a unit of two Recruits and an
  orphan's point (test defs); `production/tests.rs` the queue's entries. In the client:
  `tests/commands/armory.rs` (0337's and 0348's verifications, Q, X, Eject All and the right-click rally),
  `command_ui/interface/tests/cults.rs` (the panel as data) and `hud/info_panel/activity/tests/armory.rs`. The
  Altar's (slice M5-N1b): `tests/commands/altar.rs` (units right-clicked in, W,Q,Q's order with the InfoPanel's
  queue, R,R's Release alone, X's refund, Eject All) and `command_ui/interface/tests/altar.rs` (the three stages as
  data, their counts from a game, one Altar Tabbed to). The Cathedral's (slice M5-N4a): `tests/commands/cathedral.rs`
  (Recruits right-clicked in, Q,Q's Disciple out with its energy, its panel and InfoPanel, its right-click on an enemy
  a move), `command_ui/interface/tests/cathedral.rs` (the Cathedral's and the Disciple's panels as data, an Attack
  sending the Disciple to its target), `tests/combat/` (the energy bar) and `hud/info_panel/tests.rs` (the energy
  line). The
  Temple's (slice M5-N3): `tests/commands/temple.rs` (Recruits right-clicked in, Q's research "In progress" then
  Level 2 greyed "Requires a Cathedral", an Enhanced Constitution Militant's MaxHP 60 to 80 at once, A/S/D/F's one
  Recruit out of C, Eject All, X's refund), `command_ui/interface/tests/temple.rs` (the grid, the keys following the
  gate, the counts, the group) and `command_ui/resolution/tests/temple.rs` (the Enter, no right-click of its own).
