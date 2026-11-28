# Commands, events and replays

Read this when you add a `Command`, a `SimEvent` or a gate check, or touch how commands are recorded.

## Commands, events and replays

- All player intent is a `Command`. It is wrapped in a `PlayerCommand { player, command }` and collected in one
  tick's `TickInputs`. The client never mutates sim state. It reads `Sim` and submits commands.
- The sim validates every command (player, ownership, ids, costs, placement, and positions that are finite and on the
  map), and it **ignores invalid ones**. It must never panic on player input, so use checked or saturating arithmetic
  on numbers that come from input. Panics are only for programmer errors, such as an invalid scenario passed to
  `Sim::new` or an out-of-bounds `Map::set_tile`.
- **The command gate** (`sim/gate.rs`, with each command family's checks in its own file, plan decision T28: orders
  and abilities in `sim/gate/orders.rs`, debug commands in `sim/gate/debug.rs`, production in `sim/producers.rs`,
  construction in `sim/construction.rs`): `Sim::validate(&PlayerCommand) -> Result<(), Reject>` checks a command
  against the current state without changing it, and `step` applies a command (`Sim::apply_command`, `sim/commands.rs`)
  only through the same check (`Sim::check`, which hands the application the objects and positions it resolved), so the
  two can't disagree. In order: the player is in the game (`Reject::UnknownPlayer`), the game isn't over (`GameOver`)
  and the player isn't defeated (`Defeated`, see *Economy > Defeat and the end of the game*), a debug command needs
  debug commands on (`DebugDisabled`), then the command's own rules: every id it names exists (`UnknownObject`) and, for
  a player's orders to objects, belongs to that player (`NotOwned`, through `Sim::referenced(id, Some(player))`; debug
  commands and targets may name anyone's objects), positions are finite and on the map (`OffMap`; (M5-H1) an order's
  finite point is clamped onto the map instead, `Map::clamp_point`, so that a click past the edge sends the units to
  the edge: plan decision T81, design change 0045), an order's target
  object is one the issuing player sees or remembers (`NotVisible`, below), spawns fit (`Spawn(SpawnProblem)`) and keep apart
  from other footprints (`Overlaps(id)`, see Object model). A rejected command changes nothing, neither the state
  nor the events, and never panics.
  The one rejection the sim reports is a Place or a Warp In the placement check refuses (plan decision T4): the
  client checked it before sending it, so the game changed in between, and `SimEvent::PlacementRefused` (output,
  outside the state hash) says why (`Sim::report_refusal`). Add another such event only when the UI needs it.
- **`Command::Order { units, kind, queue }`** (`systems/controls/commands.md > Unit Command`, `docs/movement.md > 3`):
  `OrderKind` is `Move(Target)` (a point or an object), `Attack(id)`, `AttackMove(point)`, `Patrol(point)`, `Hold`,
  `Stop`, `AttackGround(point)` (slice M4-O, for the units that can target the ground: `CannotTargetGround` if none
  can; see *Combat > Attack Ground*), `Shroud` (slice M5-N4b, the Disciple's channel: the units whose type casts it,
  `LacksAbility` if none, and of them those with the energy it needs to start, `NotEnoughEnergy` if none; see *Combat
  > Shroud*), (slice M6-GH1, *Production > The Airfield*) for aircraft `Rebase(airfield)` (to an Airfield of the
  player's own in operation, `NotAnAirfield` if not; the aircraft among the units take it, `LacksAbility` if none:
  those assigned to it, and of the others, in the command's order, as many as it has room for, `AirfieldFull` if
  none) and `ReturnToBase` (the aircraft with a base, `NoBase` if none), and for workers `Gather(patch)` and
  `DropOff(node)` (`factions/colonists/units/drone.md > Unit Commands`), and `Thrust(point)` (`crate::thrust`,
  `factions/syndicate/units/prism.md > Movement`: the units whose type has thrusts, `LacksAbility` if none, and of
  them those with a charge left, `NoCharges` if none; a queued one is judged as it is given, and ends at once if it
  starts with no charge left, Placeholder Q0198). (M6-GH1) An Attack and an Attack Ground go
  to the units with ammo left, or whose weapon never runs out (`aircraft::armed`; `OutOfAmmo` if none). The gate
  takes an order whole or not at all, but tolerates stale selections: at most `MAX_ORDER_UNITS`
  ids (`TooManyUnits`), repeated ids count once, ids of objects that are gone and objects that take no orders are
  dropped, and nothing left is `NoUnits`; any id of another player's object rejects it (`NotOwned`). Points must be
  finite and on the map, target objects must exist (`UnknownObject`) and an attack's target be destructible
  (`NotAttackable`); (M6-GF) an Attack goes to the units whose weapon can hit the target's domain (`systems/combat.md
  > ValidTarget` 3), and none is `CannotHit(target)`: Twisters alone on a Peacekeeper. (Slice M6-S6a) By the client's
  own check (`combat::can_hit`, the target's domain as the player knows it, `Sim::known_domain`, as `Sim::aimed` has
  it: a landed aircraft's is `combat::object_domain`'s to say), so that the two can't disagree: the client sends those
  units after the target instead, and finds an Attack none can carry out an invalid target (design change 0461). A
  Gather's target is a Space Crystals patch the player sees or remembers, with no plate on it
  that the player knows of (`NotGatherable`; a remembered one judged by its record), a Drop Off's a drop-off of the
  player's own (`NotDropOff`), and only the workers among the units
  take either (`NoWorkers` if there is none; `sim/workers.rs`), a Drop Off only those that carry a load
  (`NothingToDeliver` if none does, design change 0140). `Deploy { origin }` (slice M3-F, see *Economy >
  Beacon Deployment*) goes to the one Drone among the units (`NoWorkers` if none, `SeveralConstructors` if more: the
  client picks one, design change 0125), with a site allowed as far as the player knows, its pending sites reserved
  (`Unplaceable(PlacementProblem)`, not reported: `Sim::deploy_check`). `Enter(target)` (slice M4-C,
  `systems/controls/commands.md > Enter`, `Sim::check_enter`) names the structure being entered (design change 0319),
  the player's own (`NotOwned`): a Tunnel in operation, and goes to the units its tier lets in (`network::lets_in`: never to a unit whose type is one way, `OneWay` if all
  are; `NoTransit` if none), or a building that holds units, not a site (`UnderConstruction`; slice M5-F1), and goes to
  the units of the kinds it takes in (`NotAccepted` if none); anything else is `NotATunnel`. Any unit of the target's
  owner may enter, whatever its type's faction. `Build { kind, origin, orientation }` (slice
  M4-E, `Sim::check_build`) goes to the one unit among them that builds `kind` (`NoWorkers`, `SeveralConstructors`),
  Shift-queued only to a unit with no Build yet (`ConstructorBusy`: each Agent takes one placement), and is paid for as
  it is given (`InsufficientSupplies`), its site allowed as far as the player knows (`Unplaceable`, not reported:
  `Sim::build_check`). Without `queue` the order replaces the unit's
  queue and current order; with it, the order is
  appended (a full queue drops it). Each order command is one group (`Sim::next_group`); a location order gives
  each unit of a group its own destination (the magic box, see *Navigation*). What each order does is in
  *Behaviours*.
- **`Command::Ability { units, ability }`**: the units use an ability at once, beside their orders: no queue, no
  Shift, no group, and their orders go on. `Ability::RiotShield { raise }` says which way rather than "toggle", so a
  group and a replay mean the same whatever each unit's mode. `Ability::SetDropOff(node)` sets the workers'
  AssignedDropOff to a drop-off of the player's own (`NotDropOff`), and "does not interrupt the Drone's current
  action" (`factions/colonists/units/drone.md > Assigned Drop-off`). `Ability::Convert` flips the Legionnaires'
  target form (see *Combat > Weapon Conversion*). A toggle switches its units' type at the press (*Object model >
  Type and form*, design change 0198), so a second one reaches them whatever their state. The gate tolerates stale
  selections as for orders (`TooManyUnits`, repeats, gone ids, `NotOwned`, `NoUnits`) and drops the units without
  the ability (`Reject::LacksAbility` when none is left); it refuses no toggle for the state its units are in.
- **`Command::Production { objects, command }`** (`factions/gdo/structures/barracks.md >
  ObjectInterfaceState[Barracks]`, see *Production*): the producers are the player's objects among `objects` that
  produce units, found as an order's units are (stale and repeated ids dropped, `NotOwned`); none is
  `Reject::NoProducers`. `Train(kind)` is taken when at least one producer queues the unit, else
  `CannotProduce(kind)`, `InsufficientCrystals` or `QueueFull` (the army cap is no reason: design change 0143 checks
  it when production starts); a pool building takes no `Train` (slice M5-F2) but `TrainFrom { kind, from }`, the same
  at pool buildings alone, each needing as many free units of kind `from` as the unit costs Unit Control
  (`NoRecruit(from)`; one without them is passed over as a full queue is, `Refusal::NoRecruit`) but no free queue
  slot, its capacity bounding its queue; `Release(from)` when a pool building has a free unit of the kind
  (`NoRecruit`, `NoHolders` for none that holds units), free, and `ReleaseOf { kind, from }` alike, of a unit of the
  type (slice M5-N1b); `Augment { kind, from, order }` at an Altar (`AlreadyAugmented`, `CannotAugment`, then as a
  `TrainFrom`, *Production > The Altar*); `Research(tech)` at a research structure (`CannotResearch`, `Researched`,
  `ResearchUnderWay`, `RequiresTech`, `Requires`, `QueueFull`, `InsufficientCrystals`: *Production > Research*);
  `CancelProduction` when a producer other than a
  Recruitment Hall (design change 0342) has something queued, else `NothingQueued`; `SetRally` with a point on the
  map (`OffMap`) or an object the
  player sees or remembers, as for an order's target (`UnknownObject`, `NotVisible`; the producer itself
  always). `Sim::validate` gives the interface the same answers for its buttons.
- **`Command::Construction { objects, command }`** (`construction.rs`, `sim/construction.rs`): the player's constructors
  carry out a `ConstructionCommand`: `Construct(kind)`, `CancelConstruction`, `CancelReady` or
  `Place { origin, orientation }` (the top-left tile of the turned footprint; the orientation optional in files, as for
  a debug spawn). The gate tolerates stale selections as for orders (`TooManyUnits`, repeats, gone ids, the player's
  objects that construct nothing; `NotOwned` for anyone else's, `NoConstructor` when none is left), then goes through
  them in id order: a Construct starts at each one that has the building on its list, is idle and can pay from what the
  ones before it left, and is rejected with the first one's reason if none can (`CannotConstruct(kind)`,
  `ConstructorBusy(id)`, `InsufficientCrystals`); a cancel goes to each one with something to cancel (`NotConstructing`,
  `NothingReady`) and refunds it (`ConstructionCancelled`). A Place names one constructor (`SeveralConstructors`; design
  change 0071) holding a ready building (`NothingReady`), and is rejected with the placement check's first problem
  (`Unplaceable(PlacementProblem)`), which is reported (above). Each object of a command is one constructor; the client
  sends a command to the group the design says (no smart-casting, design change 0071). `WarpIn { kind, orientation }`
  (slice M3-E, see *Economy > Warp In*) names one Beacon of the player's (`NoConstructor`, `SeveralConstructors`),
  not transforming itself (`ConstructorBusy`), a building of its list (`CannotConstruct`) and the bank to pay it
  (`InsufficientCrystals`), and no position: its footprint is derived, and refused like a Place's (`Unplaceable`,
  reported). `CancelTransform` goes to every transforming object of the player's named, in id order
  (`NotTransforming` when none is).
- **Targets must be known** (design change 0139, answering question Q0017 point 2; `core/vision.md > Explored >
  Targeting remembered objects`): an order (Attack, Move to an object, Gather) or a rally point may name an object
  the issuing player sees or remembers, else `Reject::NotVisible`. `Sim::target_view(player, id)` (`targets.rs`) is
  the one view of it: `Visible`, `Remembered { position, footprint }` (a record out of the player's sight: the order
  goes to the last-known position) or `Gone`. Whether the player sees an object is the one visibility query of the
  object's holder (`Vision::sees` through `Sim::is_visible`; `World::sees` for the behaviours' snapshot, the one
  ValidTarget goes by; `targets::Live::seen`), so that whatever slice M3-L adds to what a player sees (the fog's
  delay, 0178; an attacker revealed, 0177) reaches the targets too. It reads only the player's sight and records, so a remembered object
  destroyed in the fog is taken exactly as one that stands, and the gate's verdict leaks nothing. Remembered objects
  stay unselectable (the GDO player's Supply Delivery Station excepted, `core/vision.md > Explored Selectability`).
  The check is on the sim's vision, whatever the client draws (`--no-fog` is display only). A queued order passed
  the gate when it was given; what its behaviour does if the target is out of sight when its turn comes is in
  *Behaviours*.
- **`Command::Tunnel { tunnel, command }`** (slice M4-C, plan decision T36; `TunnelCommand`, to which Expand,
  Upgrade and the cancels come with slices M4-E to M4-G): the player's own Tunnel in operation (`NotOwned`,
  `NotATunnel`). **`Eject { kind }`** (`factions/syndicate/tunnel_menus.md > EjectMenu`): the tier lets the kind out
  (`NoTransit`), and a unit of the kind in the player's network is free to leave: not reserved on a queue already
  (`NotInNetwork`) and past its transit delay (`InTransit`: **a reading**, design change 0245's "For 2 s it can't be
  ejected anywhere" refuses the click rather than queueing the unit). The gate picks the one inside longest
  (`Sim::next_out`, design change 0251, ties by id) and the command reserves it at the end of the Tunnel's queue.
  The queue can't be cancelled, and goes with its Tunnel, its units staying in the network. **`Upgrade`** (slice
  M4-G): `Sim::can_upgrade` (*Tunnel upgrades*); **`Cancel`** ends the Tunnel's operation, an expansion or an
  upgrade (`NoOperation`).
- **`Command::Holding { objects, command }`** (slice M5-F1, `HoldCommand`; *Object model > Buildings that hold
  units*): the player's buildings among `objects` that hold units take it, the player's other objects are ignored
  (`NotOwned` for another's, `NoHolders` if none is left). **`EjectAll`** (`factions/cults/structures/armory.md >
  Eject All`): those whose pool has a unit (`NothingInside` if none has) mark their pool for Eject All.
  **`ReleaseOne(kind)`** (slice M5-N3): the staffed ones among them (`NoHolders` if none) with a free unit of the kind
  (`NoRecruit(kind)` if none has) each mark the one that went in first (*Object model > Buildings that hold units*).
- **`Command::Cast { casters, ability, at }`** (slice M5-N5, plan decision T76; `sim/casts.rs`, *Combat > Buffs*):
  of the player's `casters` that cast the ability (`LacksAbility` if none), in operation (`UnderConstruction` if
  none), the first in id order with the energy it costs (`NotEnoughEnergy` if none; one Shrine casts, Q0118's "each
  press casts from one Shrine", a command naming several being a Placeholder) casts at `at`, on the map (`OffMap`)
  and on a tile Visible to the player now, as the last tick left their vision (`OutOfSight`: "anywhere the player
  currently has vision"; `Sim::sees_point`, which the client's cast cursor and its click use too). `Sim::can_cast` is
  the checks but the point's, what the panel greys Q and W by. "No cooldowns": the Shrine casts again as soon as it
  has the energy. `SimEvent::Cast` says where it went. (M5-N11) A cast round its caster or ahead of it (the
  Prayer-mobile's Inspire and Suppress, *Combat > Inspire and Suppress*) has no point to check: `at` names nothing (the
  client sends the caster's position), the area is the caster's as the last tick left it, and `SimEvent::Cast` carries
  the caster's position and its `facing`, which the client draws Suppress's arc along. (M6-GR) A caster of an ability
  with a cooldown casts only once it is over: the first in id order with the energy and no cooldown casts, and
  `CoolingDown` if every one with the energy waits; a Scan's point needs only be on the map (`AbilityArea::AnyPoint`,
  `OffMap` else, a point that isn't finite among them).
- **Default orders** (`orders/resolution.rs`; `systems/controls/interface.md > RightClickResolution`, design
  changes 0062, 0063, 0074 and 0136): `resolve(Abilities, Aim)` is the one rule for what a unit does against a target
  by default: the ground gives Move; a worker on a patch it can gather Gather, on its owner's drop-off Drop Off while
  it carries a load (else a Move there, 0140); an own Tunnel whose tier lets the unit through Enter (slice M4-C:
  `Abilities::transit` against `Aimed::transit`, `network::passes`), and so does an own building in operation that
  takes the unit's kind in (slice M5-F1: `Aimed::accepts`, `Abilities::kind`); an enemy it can attack (its attack's
  TargetDomain, the target
  destructible, its mode letting it attack) Attack; anything else a Move to the object, which follows a unit (an
  unarmed Drone follows an enemy, a shielded Peacekeeper too). `Sim::aimed(player, id)` is the target as the player
  knows it (`Aimed`: enemy, destructible, domain, gatherable, drop-off, and since slice M5-N8 `repair`, the kind of a
  damaged own object; a remembered one by its record) and `Sim::default_order(unit, target)` the sim's answer; the rally
  points use it (*Production > Rally points*), and the client's right-click calls `resolve` itself on what it shows
  (*Client architecture > Command UI*).
  `tests/default_order.rs` has the table of unit kinds against targets. The rules on an object are data (slice M5-J1):
  `RULES`, a list of `Rule`s tried in order (Gather, GatherSupplies, DropOff, Enter, Assist, Repair, Attack), the
  first that applies (`Rule::order`) giving the order; a new default order is a `Rule` in its place in the list
  (slice M5-N8's Repair is after Enter and Assist, design changes 0322 and 0325: a damaged own building that the unit
  can enter is entered; `Aimed::repair` against `repair::repairs`, *Economy > Repair*). (M5-S2b) **Unless it is
  full**: the Enter row lets a building that holds units through to the Repair row while it has no room at the moment
  of the click (`Aimed::full`: `Sim::room` is 0, everyone inside counted, reserved units included; design change
  0378, answering Q0140: "a damaged enterable building that is **full at the moment of the click** resolves to
  **Repair**"). A unit whose type repairs nothing keeps the Enter, which ends at the door if the building is still
  full (the normal Full-building rule), as does an Enter given while it had room.
- **Debug spawns and teleports of units** keep the unit invariants: a box on blocked ground or across the map's edge
  is `Reject::Blocked`, one overlapping another unit of its layer `Reject::Occupied(id)`, and so is a blocking
  footprint spawned onto a ground unit. Touching is allowed. A teleported unit plans its path again.
- Every new command gets its rules in the gate and tests showing that its invalid forms are rejected without a panic
  and change nothing.
- **Game options** (`SimConfig`) are part of the game setup (`GameSetup::config`, see Sim scenarios and tests): the
  `Sim` keeps and hashes them, and the `Replay` records them with the setup; map files don't hold them. `debug` turns
  debug commands on. `SimConfig::default()` has it off, as a real match will. The client turns it on
  (`AppConfig::new`; off with `--no-debug`), editor play-tests run with it on, and so does `TestSim::new`
  (`TestSim::with_config` or `ScenarioBuilder::config` for other options). `start_explored` (slice M5-S7, design
  change 0416; *Vision*) starts the map explored: off in `SimConfig::default()` and `SimConfig::DEBUG`, so the sim's
  tests, the goldens and the client's test harness (`AppConfig::new`) start unexplored as before, and in a replay
  recorded before it existed (`#[serde(default)]`, hashed only while on); the command line turns it on for every game
  and play-test it launches (`cli::into_config`), `--unexplored` off.
- `Command::Debug(DebugCommand::…)` covers `Spawn`, `Remove`, `SetPosition`, `SetAutoTargeting` (see
  *Behaviours*), `SetResources` (any player's banks, see *Economy*; the gate checks that the player it names is in
  the game, `UnknownPlayer`) and `SetTier` (a Tunnel's tier, `NoSuchTier` for a tier it has no row for or an object
  without tiers; *Object model > The Syndicate*), and `SetAugmentation { id, augmentation }` (slice M5-F2: a unit's on
  the map, as the Altar gives them, slice M5-N1b; *Object model > The Cults*; since slice M5-N1a its HP follows its
  MaxHP, and
  `NotAugmentable` refuses Augmentations to a unit that can't have them, *Object model > Augmented stats*), and
  (M5-X1b) `SetRules(rules)` (the game's rules from the next tick on, *Tunables > Generations*; `RulesStructural` for
  rules of another structure, `RulesNotFinite` for a NaN or an infinity, `RulesFull` when no generation is left), for
  development and QA. `Spawn`
  places a tile-based object in an
  `orientation`
  (`BuildingOrientation`), which files may leave out: unturned is the default and isn't written, so replays from
  before it load unchanged. It bypasses costs and rules, only works while `SimConfig::debug` is on, and
  is never offered in a real match. A teleport sets `prev_pos = pos`, so nothing interpolates across the map.
- **Replay** = the `Scenario` (the map and the game setup, game options included) plus the `TickInputs` of every
  tick, empty ticks included, and state-hash checkpoints: `sc_sim::Replay` (serde) with `start`/`record`/`step`/`run`,
  and `record_checked`/`verify`/`run_checked` for the checkpoints (see Replays and golden hashes, which also covers
  the client's recording and playback). A replay keeps every command whole: `testing::every_command` lists one of
  each kind (every variant of `Command`, `OrderKind`, `Target`, `Ability`, `ProductionCommand`,
  `ConstructionCommand`, `DebugCommand` and `SpawnAt`), and the format tests round-trip it through the sim's serde
  and the client's file; its match stops compiling when a variant is added, so add the new one to it. The client
  must never submit a command with a non-finite number. The sim would reject it anyway, but JSON can't store NaN or
  ±∞, so the replay file would fail to load. Validate at the input layer.
- **Events**: `SimEvent`s (`Spawned`, `Removed`, `SuppliesDelivered`, `MoveFailed`, `Attacked`, `Hit`, `Died`,
  `PatchDepleted`, `ProductionQueued`, `ProductionRefused`, `ProductionCancelled`, `ProductionWaiting`, `UnitProduced`,
  `ConstructionCompleted`, `ConstructionCancelled`, `StructurePlaced`, `PlacementRefused`, `PlayerDefeated`,
  `GameOver`, `KindChanged`, `TransformCompleted`, `TransformCancelled`, `ResourcesDelivered`, `DeployFailed`,
  `TargetTooClose`, `SupplyStationEmpty`, `ProjectileLaunched`, `Blast`, `Missed`, `LocationTooClose`,
  `ShroudStarted`, `ShroudEnded`, `ShroudReleased`, `Crushed`, …) are output
  only, for effects,
  sounds and
  UI. The sim never reads them, and tells each to every player; the client shows a player only what they may
  perceive of it (*Client architecture > Viewer events*). `ProjectileLaunched { shooter, target, from, to, lands_at }`
  and `Blast { at, radius }` (slice M4-O) are a projectile leaving and landing; a blast's damage follows as a `Hit`
  for each object it took, which the client draws nothing of, and `Died` (see *Client architecture > Visuals*,
  Combat effects). A TailDisjointed shot (slice M5-O3) is an `Attacked` on what it struck, its `to` the spot it
  locked, or a `Missed { attacker, target, from, to }` when nothing was there: `target` is what it was aimed at, none
  for the ground. `DeployFailed { drone, origin, problems }` (slice M3-F) is a Deploy that failed on arrival; the
  client's feedback line says why (`command_ui/feedback.rs`). `TargetTooClose { unit, target }` (slice M3-L2, design
  change 0182) is an Attack order that failed because its target is inside the MinRange of a unit that can't move,
  or can't back off further (design change 0204 point 3); the feedback line says "Target too close". So does it for
  `LocationTooClose { unit, at }` (slice M5-O3), the same failure of an Attack Ground at the location `at`.
  `SupplyStationEmpty { station, owner }` (slice M4-I, design change 0256 point 3) is a station whose last Supply was
  picked up, for each player gathering there; the client's notice says so (`hud/notices.rs`).
  `EnteredNetwork { unit, tunnel }` and `LeftNetwork { unit, tunnel }` (slice M4-C) are a unit leaving the map
  for a Tunnel Network and coming back with its id: no `Removed` and no `Spawned` go with them. `EnterFailed { unit,
  target, full }` is an Enter that ended without the unit going in, `full` when a building that holds units had no
  room for it (the feedback line says "No room inside"). `EnteredBuilding { unit, building }`, `LeftBuilding { unit,
  building }` and `Evacuated { unit, building }` (slice M5-F1) are the same for a building that holds units: in, out
  of its doorway by Eject All or a Release, and out inside its former footprint as it is destroyed, after its
  `Removed` (and after the `KindChanged` of a unit whose training was finished, design change 0349).
  `ReleaseQueued { producer, unit }` and `ReleaseCancelled { producer, unit }` (slice M5-F2) are a pool building's
  Release, queued and taken back by X or a refund; its trained unit is a `UnitProduced`. `AugmentQueued { producer,
  unit, order }` and `AugmentCancelled { producer, unit, order, refund }` (slice M5-N1b) are an Altar's order, queued
  and taken back; its unit out augmented is a `UnitProduced`. `BuildFailed {
  worker, kind, origin, orientation, problems }`, `BuildRefunded { worker, kind, refund }`, `SiteStarted { worker,
  site }`, `SiteCompleted { site, worker }`, `SiteCancelled { site, kind, refund }` and `LeftSite { worker, site }`
  (slice M4-E) are a Build that failed on arrival, a Build refunded, and a site's life; `LeftSite` is its worker back
  on the map with its id. `SiteBuilt { site }` (slice M5-E) is a site built from outside complete: no worker comes
  out, its builders were never in.
  `UpgradeStarted { tunnel, to, price }`, `UpgradeCompleted { tunnel, tier }` and `UpgradeCancelled { tunnel, to,
  refund }` (slice M4-G) are a Tunnel upgrade's life; one destroyed mid-upgrade has no event of its own.
  `ProductionWaiting { producer, reason }` (slice M3-P, design change 0143) is a producer whose unit at the front
  began to wait for room under the army cap `reason`, raised once per wait: the placeholder of 0143's notification.
  `Sim::events()` holds only the latest step's events, and `Sim::new` discards the events from the scenario's
  initial spawns. `Hit { attacker, target, damage, shield_damage }` says what a hit took of the target's HP and of
  its Shields (see *Combat > Shields*). `KindChanged { id, from, to }` (`Sim::change_kind`) says that an object
  changed its kind in place (a toggled unit's type at the press, design change 0198).
  The client's response (slice M3-E) reads the objects themselves, so a replay's jump or a loaded game gets it too:
  the object's visual is made anew with the new kind's mesh, size and tag (*Client architecture > Visuals*), it
  stays selected with the same id as its new kind and its marker takes its new shape (*Selection*), and the panels
  re-resolve its type. The Warp In's own interface (its ghost, Cancel on its panel) is slice M3-J1's; a converted
  Legionnaire (slice M3-B) gets the same response and nothing else.
