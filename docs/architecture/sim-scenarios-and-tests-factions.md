# Sim scenarios and tests — Scenario tests for the factions' features

Read this when you write or find a sim test of a Cults, Syndicate or GDO feature, upgrades, game end or determinism.

- **Cults construction tests** (slice M5-E) are `crates/sim/tests/cults_construction/` (`flow.rs`: four Recruits in
  80 frames paid once, five from one side spread along the wall (0339), two at twice the rate, one alone, a Shift
  chain, builders pulled off and back by Assist, the right-click; `refunds.rs`: two of three re-ordered and then the
  third (0340), killed, after the start, a queued building its last holder drops (0340), X, the queued building after
  X; `footprint.rs` (slice M5-J1, design changes 0362 and 0372): nine Recruits under a Hall walk off and it starts, a
  site waiting for one walking off its middle, an enemy and an own Recruit not on the order each failing it, refunded
  once, a Recruit holding it queued holding it back until it walks off, and 0372's Verification, a Hall Shift-placed
  over a builder's spot at a Storage; `gate.rs`: the Armory's Storage, Shift-clicks each
  paid, Assist's refusals, a Hall flush against the edge and in a corner taken (slice M5-S4, 0387); `crowds.rs`:
  twenty-four on a Storage; `hall.rs`: a built Hall's lineage, Halls built flush against the edge (70%: 14 Recruits,
  275 ticks each) and in a corner (49%: 9, 392 ticks), gatherers taken off and back), each tick checking a frame per
  builder at work. `sim/spot_work/tests.rs` (test defs): the wall full
  and those waiting further out moving in, oblong ABAC footprints, 2 × 3 and 2 × 4, at four rotations, and a walk
  ended on the footprint looking again (0362); `placement/wall/tests.rs` the wall, its search and the way round;
  `spot_work/tests.rs` the claim and the reach.
- **Repair tests** (slice M5-N8) are `crates/sim/tests/repair/` (`rates`: one to four Recruits on an Armory, linear, a
  crystal every 3 HP, each object by its own terms; `bank`: the bank running out, the queued orders going on, a Repair
  given with no crystals; `agents`: a Tunnel for free at Tiers 1 and 2, a Laser Turret lowered and raised, one left
  neutral; `refusals`: the gate's; `regeneration`: a Hideout to full in 100 s, under AoE fire, none once its Tunnel is
  gone, none for a turret), `sim/repairs/tests.rs` (test defs: a vehicle followed as it drives off, a glider refused by
  its movement), `repair/tests.rs` (the terms, who repairs what, the payment's steps) and `tests/default_order.rs`'s
  Repair rows. In the client: `command_ui/resolution/tests/repair.rs` (the right-click and D as data),
  `tests/commands/recruit_ui/repairing.rs` (a Storage repaired by a right-click, an Armory entered by one and repaired
  by D, D Inactive at full HP) and `tests/commands/agent_ui/repairing.rs` (a turret repaired, a Tunnel entered by a
  right-click and repaired by D, an expansion refused).
- **Beacon Deployment tests** (slice M3-F) are `crates/sim/tests/deploy/` (the walk to the footprint's centre and the
  start on arrival, the 48 frames, the HP from 20 to 120 and damage carried through (0110), Beacon Capacity 10/10 to
  9/10 to 9/15 and a cancel to 11/10 (0112), the cancel's fractions of HP and Shields and its place (0109, 0111), the
  Drone's Shields kept as it starts, the load lost or returned (0114), defeat (0113), a destruction, the footprint
  claimed, auto-targeting taking the footprint, Shift, a save and a replay; in modules by topic: `arrival`, `cancel`,
  `loads`, `beacon`, `saves`) and `crates/sim/tests/deploy_gate/` (one
  Drone per Deploy; the gate on what the player knows, remembered buildings, hidden units and never-explored ground
  (0120); failures on arrival for a structure, a unit of either player, a patch, a Drone walled off, and a Drone blocked
  while overlapping that deploys (0117, 0118); only Cancel on a deploying Beacon (0116); the reservations and
  `Sim::pending_builds` (0125); a fuzz of the gate and the arrival; in modules by topic: `gate`, `arrival`,
  `pending`, `fuzz`). Unit tests: `deploy.rs`'s, `placement/tests/`
  (reservations, `Mode::Known`) and `transform/tests.rs` (the load hashed).
- **Agent-built Tunnel tests** (slice M4-E) are `crates/sim/tests/builds/`, under `check_sites` (each site's worker held
  in it and nowhere else) after every tick: `flow` (the order paid at once, the site's HP and completion, the network
  and an Eject, the site's one command, the load kept, what enemies see and target, a cancel, a destruction, the
  defeat), `price` (0233's verification, the refusal, every refund, a price of 0, and the books: a property test over
  random orders, Shift-queues, cancels and replacements, with a fixed seed list; since slice M4-Z it also plays Tunnel
  1's expansions, its Laser Turrets among them (an Agent's turret Builds until slice M5-Y, design change 0359), and
  balances both banks every tick by `testing::build_order::Books` and `testing::syndicate::SupplyBooks`, each refund
  counted in its own resource by `build_order::refund_in`) and `rules` (the overlap rule by knowledge and by truth, the
  Drillable area, 0240's hidden expansions for the three factions, a unit on the site, the lingering ghosts'
  reservations). Unit tests: `construction/site.rs`'s (the maximum, who builds what).
- **Losing Tunnel tests** (slice M4-H) are `crates/sim/tests/losing_tunnels/`, under `check` (no reference dangles,
  each site's worker is held in it, and only a neutral expansion decays) after every tick: `neutral` (the only Tunnel
  falling in combat: the Hideout neutral, seen and remembered by the enemy, the defeat, the Agents neutral, the network
  destroyed, 4 HP a second and the death in the 1600th tick; one of two Tunnels: work lost, the queue refunded, the
  infantry inside kept), `stranded` (the vehicles at the last Tier 2 Tunnel, a site is no exit, the free rebuild with a
  debug-spawned Barracks), `attack` (auto-targeting and the right-click leave it, an explicit Attack hits it) and
  `reclaim` (at completion and never while a site, the enemy's view and the events of the tick, an enemy reclaiming
  and the old owner's record, partial cover blocking cells and taking Tunnel Space) and `saves` (saves at every
  stage, the replay, the decay hashed and saved only while set). Unit tests: `sim/tunnels.rs`'s
  (the decay's arithmetic). In the client, `crates/game/tests/fog_memory/neutral.rs` (the neutral colour, the falling
  HP bar, the record out of sight).
- **Expansion tests** (slice M4-F) are `crates/sim/tests/expansions/`, under O1 after every tick: `placing` (the
  expansion method by knowledge and by truth, 0290's units), `building` (paid at the click, Tunnel Space, 400 ticks
  with the damage kept, the second Hideout training, one operation, Eject meanwhile), `cancelling` (150 of 200 by
  either X, the site gone otherwise) and `hidden` (the enemy sees nothing; a Warp In over it red at once). The test
  kinds of another cost or Tier Requirement are `sim/expansions/tests.rs` (only a unit test can change a def), the
  ExpandMenu's list `defs/syndicate/tests.rs` (against the Tech Tree table, 0291). In the client,
  `command_ui/interface/tests/expansions.rs` (the menu's letters and greying, the placement, the clamp) and
  `crates/game/tests/commands/expansions.rs` (W, the ghost, the click, the bank, the operation and both X's).
- **Vehicle Assembly tests** (slice M6-SV) are `crates/sim/tests/vehicle_assembly/`, under the invariants and the
  held store's checks after every tick: `building` (refused at Tier 1 and without a Chemical Plant, 250 and 480
  frames from 10% of its HP, its armour's share of a blast and its 1% a second), `production` (the Bike into the
  network for 125 in 240 frames, Tunnel Space 2 at the start, held at 0% for it, and the rallies of 0252), `exit` (out
  of Side A facing out in the four orientations, refused at Tier 1, five out one by one by a rally and sent on, five
  ejected with no order stalling after two, a reading for the Director, and back in through the entry path),
  `mortar` (the area rule over a group, a dodge, Attack Ground and ground only, the 40° turret and the hull, a target
  in range far out of the arc fired at once it will stay in the arc at rest, from beside it within its own blast at
  60° and 90°, and one inside the turning circle met out of the arc and given up, readings, and no crushing) and
  `saves`. The 72 × 48 box of plan decision T100
  is `sim/network/tests.rs`'s (a test def). In the client, `command_ui/interface/tests/syndicate.rs` (the VA's Q, W,
  E, R, X and C) and the ExpandMenu's tests above (its A, greyed by tier and by the Chemical Plant).
- **Factory tests** (slice M6-GF) are `crates/sim/tests/factory/`, under the invariants: `buildings` (each of the
  three refused without the one before it, its frames, the Industrial Powerplant's +60 in the Power summary),
  `production` (both vehicles at their cost and time, Unit Control reserved at the start, the Twister's Supply, no
  Prospector or Artillery Rig yet), `exit` (the default rally point in every orientation, five Raptors out of side B
  facing out with no overlap), `parking` (0451 at the Factory, and the stall with the point cleared), `raptor` (a
  running Peacekeeper hit from Range 6 every 32 frames, the turret tracking while the hull drives and turns) with
  `raptor::close_range` (question Q0184's cases: a target inside its turning circle that it repositions from and
  kills, design change 0454, where it gave it up before; Attack Ground refused), `twister` (a Sparrow
  behind it shot down, the turret turned round; an Attack on a Peacekeeper refused); with
  `sim/producers/tests.rs` (T100's exit spot for a 64 × 48 box) and, in the client,
  `crates/game/tests/commands/factory.rs`.
- **Air door tests** (slice M6-SD) are `crates/sim/tests/air_doors/`, under the invariants and the held store's
  checks after every tick: `building` (the Forge and the Aerospace Assembly, each refused a tier lower and without its
  prerequisite, their prices, frames and armour), `production` (the Phantom into the network behind an Optics Lab,
  and its rally's auto-eject by the air exit door, its 16 ticks' rise holding still), `out` (the two lanes: five
  Phantoms one every 8 ticks while Guards walk out of Side A, two rising at once, a rising one hit as an air target),
  `entering` (a right-click's Enter by the air entry door, the descent hit as an air target, the door's rate, an
  order or a lost Tunnel during it), `tiers` (each tier's refusals and right-clicks for a Guard, a Bike and a
  Phantom, an upgrade to Tier 3 admitting none until it completes, the aircraft stranded), `edge` (a Tier 3 Tunnel
  against each edge), `phantom` (its laser on a Peacekeeper and a Sparrow) and `saves` (mid-rise and mid-descent, and
  the replay). A door's offset tuned past the map's edge is `tests/rules.rs`'s, the bound on `air_door_ticks`
  `ruleset/check/tests.rs`'s. In the client, `command_ui/interface/tests/syndicate.rs` (the AA's Q, W, E, X and C,
  the Phantom's F greyed by tier), the ExpandMenu's tests above (its S and F), `object_render/tunnel.rs` (the doors
  drawn where the sim has them) and `object_render/flight.rs` (the rise and the descent drawn).
- **Null Chamber tests** (slice M6-SN) are `crates/sim/tests/null_chamber/`, under the invariants and the held store's
  checks after every tick: `main` (its def's numbers, its turn as long as the transit delay), `building` (refused at
  Tier 1 and without a Hideout, its 200 Space Crystals and 480 frames, no turn while it is a site, a turn in the tick
  it is finished), `turns` (a turn from the tick of the entry, and of a training's end; six Guards in at once with one
  chamber, four done at 2 s and all six at 4 s, and ejected at 3 s four out buffed and two not; the first in served
  whatever their ids; two chambers' eight slots; a turn done kept inside, but no buff once no chamber is left),
  `buff` (a Guard's sight of 5 + 2, and a hit of 10 dealing 7, × 0.8 before its armour of 1; every hit up to the
  319th tick after the exit reduced and none after; a restart, and a second exit without a turn; another player's
  chamber gives none; a Phantom out of the air exit door buffed) and `saves` (turns under way saved at six points and
  replayed; `Held::service` saved and hashed only while set). `buffs/tests.rs` checks that `BuffKind` hashes and
  saves as the abilities did, `ruleset/check/tests.rs` the turn's bound. In the client, the ExpandMenu's tests above
  (its D). (Design change 0463) `turns` has a turn done before the last chamber is lost leaving buffed, and `buff` a
  buffed Guard on AttackMove engaging where an unbuffed one does, its owner seeing 2 GU farther.
- **Sludge tests** (slice M6-SB) are `crates/sim/tests/sludge/`, under the invariants (N1 and F1 among them): `building`
  (the Vault behind a Hideout for 150 in 400 frames, serving once finished; the Shadebringer behind a Vault for 150 in
  320 frames), `toggle` (on with any charge, 10 a second, parked or not, off at empty, refused at none, the orders
  kept), `trail` (48 su behind it, each patch 10 s, at its front while it backs up), `slow` (0.6 in it and 2 s after, a
  Shadebringer over its own trail and braking across another's, hover units and aircraft not, a box in it but its centre
  out not), `vault` (both toggles off into the network, 5 a second, 2 at a time first come first served, with the Null
  Chamber at once, 0424; the EjectMenu's order kept) and `saves` (a save and a replay at eight points; the patches and
  the tanks in the hash and the file only while there are any); with `patches/tests.rs` (the store and its lookups, each
  toggle its own tank). In the client, `command_ui/interface/tests/shadebringer.rs` (R, F, the off key and "Tank
  empty"), `patches_view/tests.rs` (the discs, the fog, the shrink), `crates/game/tests/commands/sludge.rs` (R on and
  off in a game, the trail drawn, the tank bar, the slow's mark) and the Vehicle Assembly's and the ExpandMenu's tests
  (W, and R).
- **Transport tests** (slice M6-ST1) are `crates/sim/tests/transport/`, under the invariants (U1 among them):
  `production` (the Vehicle Assembly's R behind a Forge, 200 in 400 frames, into the network; ejected, boarded),
  `boarding` (four Guards board and a fifth idles beside it; boarding a Transport that drives away; no enemy, no
  vehicle, none while burrowed), `unloading` (four out on the first ring's corners with no overlap, idle; at rest first;
  Shift-queued after a move; refused empty and to a non-carrier; in a packed pocket two out and two aboard until a
  move and another F), `combat` (no shots from a loaded Transport; a
  Peacekeeper may attack it, a Twister may not; it collides with ground units, crushes enemy Recruits, own infantry
  stop it), `death` (three aboard die with it, after it, where it was, their Tunnel Space free; stranded in the network,
  removed with it), `network` (into a Tier 2 Tunnel loaded and out loaded, the EjectMenu listing it alone; a minute
  inside heals and buffs it, not its passengers) and `saves` (a save and a replay with passengers aboard); with
  `carry/tests.rs` (the rings at every facing), `sim/holding/carriers/tests.rs` (a pocket of cliffs: two out, two
  aboard), `ruleset/tests.rs` (`carries` and `domain` are structure) and `tests/default_order.rs` (the right-click
  boards it). In the client, `command_ui/interface/tests/transport.rs` (the panel, F greyed empty),
  `tests/syndicate.rs` (the Vehicle Assembly's R behind a Forge) and `crates/game/tests/combat/main.rs` (the pips, to
  the owner).
- **Landing tests** (slice M6-GH2) are `crates/sim/tests/landing/`, under the invariants: `slots` (four out of ammo
  on the four slots, a landing in 16 ticks holding still, F and the right-click home from anywhere, one down left on
  its slot under an overflight, a right-click on another own Airfield landing it there), `rearm` (2 of 5 in 96 ticks,
  then landed until a Move; an order 5 s in leaves with 2; an Attack Move Shift-queued behind F lifts it off as it
  lands; F alone, full within 10 s and landed; 272 ticks at 30/51 Power), `domains` (an air target landing and
  taking off, a ground one down), `destroyed` (the aircraft on a destroyed Airfield die with it, one taking off
  escapes, one flying home flies on unassigned) and `burst` (design change 0458's: where a moving target stood,
  about 33 on a tank before armour, a clump of infantry own units included, nothing in the air). The unit tests in
  `landing/tests.rs` cover the layout, the claims, the
  rise and the rearm's speed; L1's negative test is in `testing/invariants/landings.rs`.
- **Dragonfly tests** (slice M6-GL) are `crates/sim/tests/dragonfly/`, under the invariants: `training` (the
  Deployment Center's Propulsion Labs behind an Airfield, the Airfield's W refused until it is placed; 175 Space
  Crystals and 4 Supplies, refused with 3 or 174; 320 frames, landed on a free slot and still, or rising straight up
  for its rally point from IdleSpeed), `missile` (one missile along its heading downs a Sparrow, then home by itself,
  IdleSpeed over its slot, 16 to land, 64 to rearm, a Move lifting it straight up; air alone, nothing once spent),
  `approach` (from 20 starts and headings, never stopped in the air; three at once; no slot free: on to the centre),
  `landing` (`tests/landing`'s domains, destroyed and rearm cases for a Dragonfly) and `separation` (two end 48 su
  apart); with `movement/approach.rs`'s unit tests (the straight-in approach; three starts, far, beside and nearly on
  the spot, each at 24 headings) and G1's held case in `testing/invariants/gliders.rs`.
- **Supply Tower tests** (slice M6-GS1) are `crates/sim/tests/supply_tower/`, under the invariants, auto-targeting
  off: `placement` (the Tower behind a Radar Station, its free chopper down on it and attached, the choppers it trains
  over its centre), `runs` (a pick-up takes all a station holds as it touches down, a drop-off pays it in, the
  right-click gives each, the gate's refusals), `schedule` (a scheduled chopper down on its station in the delivery's
  first tick after a hover of at most 24 ticks, the bank up by the delivery; one leaving as soon as it is home when the
  deliveries are too frequent; two Towers on one station taking turns, the closer first), `attachment` (eviction, a
  command breaking the attachment, the domains landing and down, the free repair in 80 ticks and 136 at 50/85 Power),
  `sharing` (an Agent's claim kept from a chopper's pick-up) and `supplies` (design change 0436's Verification: 1
  Supply to start, a Field Engineer once a Career Campus stands; a run's Supplies train a Drone Operator). L1's
  chopper arm's negative test is in `testing/invariants/landings.rs`.
- **Airfield tests** (slice M6-GH1) are `crates/sim/tests/airfield/`, under the invariants: `training` (the
  Hornet's 150 Space Crystals and 2 Supplies, refused with 1, refunded on a cancel; a new Hornet over its Airfield,
  assigned, hovering, or facing its rally point and flying there), `assignment` (the limit of 4 with the queue; a
  Rebase onto another Airfield with room, refused onto a full one, a group's partly; only aircraft, only onto an own
  Airfield; a destroyed Airfield's aircraft unassigned, their return ended, reassigned only by a right-click),
  `ammo` (5 missiles of 30, then the automatic return home; Attack and Attack Ground refused; a Move interrupting the
  return, the ammo staying at 0; Q0182's hit on a target standing still and miss on one that moved; the aim turning
  in place), `braking` (a Hornet's drift to rest by drag alone beside a Drone's braking; the map edge); with
  `movement/drag/tests.rs` (the drift and the approach plan), `aircraft/tests.rs` and, in the client,
  `command_ui/interface/tests/airfield.rs`, `command_ui/resolution/tests/airfield.rs` and `tests/combat/` (the ammo
  pips).
- **Upgrade tests** (slice M4-G) are `crates/sim/tests/upgrades/`, under O1 after every tick: `costs` (2 then 4, 3
  then 6, a Tier 3 Tunnel in the Tier 2 price, two started together, 0232's refunds in either order, and the books
  over random starts and cancels), `blocked` (an own neighbour, an enemy's site and Tunnel by the truth, Mountain and
  Water, every map edge at 3 and 4 tiles, the gate's order, and an own lingering ghost that doesn't block and fails on
  arrival), `during` (the reserved area against the owner's ghost and an enemy's, 0249's vehicle, the X, the Tunnel
  destroyed) and `completion` (480 frames, the HP, Tunnel Space, a reclaim, saves and the replay); 0231's 500/600 is
  `sim/upgrades.rs`'s unit test. In the client, `command_ui/interface/tests/syndicate.rs` (Q and its reasons),
  `hud/info_panel/hint.rs`'s (the price), `hud/info_panel/activity.rs`'s (the line),
  `placement/tunnel_areas/tests.rs` (the reserved area among the own areas the rings count),
  `crates/game/tests/commands/tunnel_upgrade.rs` (Q, the progress, X's refund, the larger area, what blocks it) and
  `crates/game/tests/commands/agent_ui/ghost.rs` (the ghost red over the reserved area, the rings red over it, the
  current area while selected).
- **Game end tests** are `crates/sim/tests/game_end/` (defeat by a removal and in combat, plates counting; no
  defeat without a structure ever owned; the units made neutral and idle; every command of a defeated player
  refused; a 1v1, three players defeated one after the other, and the draws of 0163: two players defeated in one
  tick, and the last two of three; no producer handed over; a save across a defeat and the end, and old saves; in
  modules by topic: `defeat`, `on_defeat`, `end_of_game`, `saves`) and,
  in the client, `crates/game/tests/game_end/` (`outcome`: the banners, Draw for everyone, the stop; `recording`:
  the saved recording, QA's end; `results`: the results screen of 0164 in a three-player game, under the debug
  player switch and in a replay, the draw seen from it, the spectator's fog; `notice`: the defeat notice of 0357,
  every other player told, under the debug player switch) with the unit tests of `hud/results.rs`
  (who leaves, allies included, which the sim can't make yet). The fuzz games of `robustness.rs` give each player a
  base the stream never removes, so that a debug removal doesn't end them; a fight still can (a Grenadier, or a Laser
  Turret in its owner's Tunnel Area, takes a base), and then the tick after the end checks that every command is refused
  and changes nothing, and the stream goes on in a new game (slice M4-M: any seed does, none is hunted for). One seed
  is one sample, which the next kind or rule reshuffles: **the ignored `robustness::seed_sweep`** (slice M5-Z1) plays
  the first 256 seeds' streams and lists every one that panics, with its message (each of the fuzz's own checks names
  what broke and the tick). A slice that changes the sim runs it before it reports: `cargo test -p sc_sim --release
  --test sim robustness::seed_sweep -- --ignored --nocapture`, about 4 s. At M5-Z1 all 256 pass, and 9 of them end a
  game, which the fixed seed no longer does.
- **Determinism tests** compare `Sim::state_hash()`, an FNV-1a 64 hash of the whole state except the events. Run the
  same scenario and inputs twice, or play back a `Replay`, and compare the hashes (`determinism.rs`, with the replay
  checkpoints). Every state type's hash impl destructures it (`hash.rs`), so a new field doesn't compile until it is
  hashed, and `sim/tests/hash.rs` changes each field in turn and checks the hash changes. The golden games (`golden/`) pin the hashes from build to build: see Replays and golden hashes.
