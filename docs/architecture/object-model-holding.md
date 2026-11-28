# Object model — Held units, sites, expansions and Tunnels

Read this when you work on units off the map (the held store), sites, buildings that hold units, expansions, pop-ups, tech prerequisites or the Tunnel Network.

- **Units off the map: the held store** (`network.rs`, `sim/network.rs`; slice M4-C, plan decision T30;
  `factions/syndicate/tunnels.md > Tunnel Network`): a unit inside a Tunnel Network is **not among the objects**. It
  is a `Held { object, place, since, entered }` in `Sim::held`, a second map by id beside `Sim::objects`, with the
  id it had. So every system that walks the objects (movement, collision, vision, targeting, auto-targeting, picking,
  records, the client's visuals, which diff against `Sim::objects()`) passes it by without a filter, and a forgotten
  case fails closed: it can't show, move or hit a unit that isn't there. The one exception to "never a separate store
  keyed by id" above, for that reason. **What must count held units says so**: `Sim::army_cap` (their army cost,
  Tunnel Space), the removal path's reference cleanup (`Sim::forget_references` walks them too, so a held unit never
  names an object that is gone), the navigation grids (`paths::walking_keys`: a held unit's grid is there for the spot
  it comes out at, slice M5-Z2), `Sim::clear_defeated_networks` (a defeated player's go, `Sim::remove_held`, which
  slice M4-H's stranded-unit rule calls too), the state hash and the saves, `testing::dangling_references`, the
  client's control groups (`ControlState::retain_on_map`). `Place` is where: `Network`, or `Site(site)`, a worker
  inside the site it builds (slice M4-E, *Sites* below: not in the network, so no Eject brings it out, and it doesn't
  regenerate), or `Building(building)`, a unit inside a building that holds units (slice M5-F1, *Buildings that hold
  units* below); match it exhaustively and never assume a held unit's place is on the map (a transport's passengers
  will be held at a carrier that may itself be held). `Sim::take_off_map(id, place, entered)` and `Sim::put_on_map(id,
  pos, facing)` are the only ways off and back: the first ends every reference to the unit as a removal does, without
  a death's effects (no `Died`, no `Removed`), and clears the unit's own orders (a Build among them refunded,
  `Sim::drop_orders`), move and attack (design change 0249), its HP, load and modes kept; call it before 5b, as a
  change of kind. `Sim::held()`, `held_unit(id)`, `network(player)`,
  `network_counts(player)` (the EjectMenu's tiles: unreserved units by kind), `reserved()` and `next_out(player,
  kind)` are what the interface reads. Hashed and saved **only while non-empty** (`#[serde(default,
  skip_serializing_if)]`, a tag in `Sim::state_hash`), as a Tunnel's eject queue is (`TunnelState::eject`), so the
  states without a held unit, the goldens among them, are as they were. Inside, a unit regenerates 1% of its MaxHP
  a second (`TunnelDef::regen_per_second`, placeholder, 0249; `crate::regeneration`, the rule of the expansions'
  regeneration too since slice M5-N8) and may leave `TRANSIT_DELAY_TICKS` (32, placeholder, 0245) after it walked in, a
  unit produced inside at once (0246; `Held::may_exit`). (Slice M6-SN) A manned unit takes its turns at its owner's
  Null Chambers there, `Held::service` (*The Null Chamber* below), hashed (tag 2) and saved only while it has one.
  (Slice M6-SB) A unit with a tank that isn't full has it refilled at its owner's Vaults there, its tanks being its
  turn (*The Vault and the Sludge* below).
  **Measured** (`tests/agents_perf.rs`, ignored, release; slice
  M4-M): 4 Tunnels with 4 Agents each on two shared fields and 60 Guards cycling through Enter and Eject, a mean of
  0.043 ms and a p99 of 0.067 ms a whole sim tick; then 40 Grenadiers shelling a Tunnel field, 0.070 and 0.090 ms (load
  average 0.3 to 0.6).
- **Sites** (`construction/site.rs`, `sim/sites.rs`; slice M4-E, plan decision T31; `factions/syndicate/units/agent.md
  > Building > Tunnel Construction Flow`, design change 0234): a structure a worker builds where it stands is, from its
  first frame, the object it will be, with `Object::construction: Option<Construction { progress, builder }>` while it
  is a site: a `Progress` over its def's frames (`SiteDef`, `StructureDef::site`) and `Builder::Held(worker)`, the
  worker held at `Place::Site(site)`. Hashed (a tag) and saved only while set, so the states without a site are as they
  were. `Object::max_hp` gives the ConstructionHP rule's maximum (`SiteDef::start_hp`, 10%, growing with the frames),
  and the HP follow it up with the damage kept (`transform::follow_max`). **`Object::is_operational`** (neither a site
  nor transforming) is the one predicate of a structure doing what its type does: the exits and Enter
  (`network::is_exit`), the drop-offs (the gate, `Sim::nearest_drop_off`, the resolution's facts) and Tunnel Space
  (`gives_tunnel_space`) all ask it, so that a site does nothing but Cancel. A site counts for defeat, blocks and is
  seen as any structure; the worker inside, held, is not. (Slice M4-S1's Laser Turret site of an Agent's went with
  design change 0359: a turret's site is a Tunnel's expansion's, hidden, *The pop-up form*; it holds its fire until
  complete, *Combat > Structures that attack*.) (M5-E) A Cults site is `Builder::Recruits`: nobody is held, and it
  gains a frame for each Recruit building it from outside (*Economy > Cults construction*); its functions are held
  back while it is a site (`sim/transform.rs`, `hold_back`) and start as it completes (`Sim::start_functions`), so that a Hall
  claims and produces from then on (`Production::completed_at`); all but a rally point, which it may take meanwhile
  (slice M5-S6, *Production > Rally points*).
- **Buildings that hold units** (`holding.rs`, `sim/holding.rs`, `sim/holding/evacuation.rs`; slice M5-F1, plan
  decision T60; `factions/cults/structures/armory.md`, `factions/cults/economy.md > Cults Building Mechanics >
  Evacuation on Destruction`, `systems/controls/commands.md > Enter` with design change 0319's TargetObject): **one
  capability, data on the def**, `StructureDef::holds: Option<HoldsDef { entrance, exit, capacity, accepts }>`: the
  side lettered `entrance` is the way in, `exit` the `ExitRule` of the way out (`ExitRule::Side` only, for now; the
  defs test checks it, and that a type that also produces has the same rule in both), `capacity` bounds the pool,
  `accepts` the kinds let in. The Armory is the one so far (B in, C out, 10, Recruits). The units inside are in the
  held store at `Place::Building(building)` (`Sim::held_in`: longest inside first, by `Held::since` then id), so
  everything the store does for a Tunnel's (references, the hash, the saves, a defeat) holds for them too.
  - **In**: M4-C's Enter order, generalized, not forked. The ways in ask `network::lets_in`: `network::admits`, and at a Tunnel no unit whose type is one way (`UnitDef::one_way`, the Prism; `NoTransit::OneWay`, and `Reject::OneWay` when all of an Enter are; `factions/syndicate/units/prism.md > One way only`); the way out asks `network::admits` alone. `network::admits(target, kind)` asks `holding::admits` of a
    building that holds units (placed, owned, `Object::is_operational`: a site refuses, `Reject::UnderConstruction`
    at the gate; design change 0347 point 4: "a building under construction **can't be entered**; a right-click on
    it Assists", *Economy > Cults construction*; `accepts` names the kind, else `NoTransit::NotAccepted`,
    `Reject::NotAccepted` when none of the units is let in). The walk ends at `network::EntryPath` (`outward`, `spot`;
    `stand(half)`, `reached(pos, half)`), `EntryPath::of_object`: a Tunnel's from `Paths::entry_path`, a holder's
    the middle of its entrance side (`holding::entry_path`; 0347 point 1: "a dedicated doorway at the middle of the
    entrance side": a unit goes in once its box is within `Tunnel::entry_reach_su`, 8 su, of that one point, and nowhere
    else along the side, `entering.rs`). A Tunnel's units walk up
    against the wall (M4-C, unchanged); a holder's stop `holding::ENTRY_STANDOFF_SU` (4 su, within the reach) clear
    of it (`EntryPath::standoff`): against it, a Recruit pressed along the wall by the crowd coming round the corner of
    the entrance side slid ever more slowly towards a spot it couldn't get round the corner to, and gave up with the
    one behind it (`entering.rs`, the QA run's game). **The room is judged on
    arrival** (`Sim::room`: capacity less everyone inside, those an order reserves and those on their way out
    included, design change 0347 point 5: "Capacity counts reserved units"), not at the gate: a unit that gets there
    with no room loses its order and idles beside the building (`SimEvent::EnterFailed { full: true }`, the client's
    "No room inside" line; 0347 point 2: "a unit arriving when the building is full has its Enter **end** and idles
    beside the building (the normal Failure rule)"). The right-click resolves an own operational holder that takes
    the unit's kind to Enter (`Aimed::accepts`, `Abilities::kind`; "Own Armory → Enter Armory").
  - **One of a kind out** (slice M5-N3, `HoldCommand::ReleaseOne(kind)`; `factions/cults/structures/README.md >
    Choosing Recruits, Release and Eject All`, the tech buildings' "pressing one sends **one** Recruit of that kind out
    of the exit at once (no queue)"): a staffed building's (`holding::staffed`: it holds units and researches, the
    Temple) free unit of the kind that went in first is marked ready to leave, as an Eject All marks them all
    (`NoHolders` where none is staffed: a pool building releases by its queue; `NoRecruit(kind)` where none has one).
    Several Temples each let one out (**Placeholder**, as Eject All goes to every one of the selection).
  - **Out by Eject All** (`Command::Holding { objects, command: HoldCommand::EjectAll }`, gate `Sim::check_holding`:
    `Reject::NoHolders`, `Reject::NothingInside`): it marks every unit of the pool ready to leave, `Held::leaving`
    (the tick it was marked; hashed and saved only while set), and `Sim::step_holding`, in the economy phase right
    after `Sim::step_network`, lets each building's unit ready first out, in id order of the buildings, through its
    exit doorway (`Sim::let_out`, `Sim::release_held`, a producer's release: `doorway_spot`, `Mover::exit`, one at a
    time, only once the doorway is free; of several doors, the first free one in `Doorway::order`, slice M5-C's rule;
    `SimEvent::LeftBuilding`): "in the order they are ready" (design change 0348 point 4), ties by the longest inside,
    then id. A pool building's trained and released units are made ready the same way (*Production > Pool
    buildings*), so one doorway serves them all in that order. A unit that goes in meanwhile isn't marked, and stays,
    and so do the reserved ones. Out, the unit goes to the building's RallyPoint with the rally's order (0348 point 3:
    "Ejected and released units walk to the RallyPoint"), or straight out to idle without one (`Sim::rally_order`).
    `production::exit_rule` gives a building's doorway its rule: production's, else the holder's.
  - **Out by destruction** (slice M5-F3, design change 0349: "evacuated units **all appear at the same time inside
    its former footprint**, unharmed, spread over it. Units that overlap ignore each other until separated, and idle
    ones push apart"): in `Sim::remove_objects`, `Sim::evacuations` reads each going holder's units first, longest
    inside first (`Sim::held_in`; 0349: "No placement order"), and takes a finished front entry off its queue
    unrefunded (`Sim::finished_training`: "A unit that had finished training [...] comes out **as the trained unit**,
    with no refund"); then `Sim::refund_production` refunds every other entry in full, so that the units they
    reserved come out as Recruits, a Recruit mid-training among them. `Sim::evacuate` puts them all on the map in that
    call, once the building has left the game and every reference to it is gone: each on the centre of the next tile
    of the footprint, row by row from its north-west tile, west to east (`Placement::tiles`), round again past the
    last (`Sim::footprint_spot`; Placeholder, not in the design: how they spread over it), its box moved in to lie
    inside the footprint, a tile whose terrain the unit can't stand on passed over (`nav::box_on_terrain`, the
    terrain alone: the grids still have the footprint until `paths::sync`). The finished training's unit is trained
    there (`Sim::train`, its `KindChanged`), the others are as they were; each faces as it did, idles (0349 point 4)
    and has a `SimEvent::Evacuated` after the building's `Removed`; so before the defeat check of the tick, and a
    defeated player's evacuees turn neutral on the map with its other units. They overlap, which I1 allows in the
    tick they come out (`invariants::overlaps::may_overlap`, *Movement and collision > Collision exceptions*), and
    part by the movement's own rules: overlapping units ignore each other (`space::blocks`), idle ones step apart
    (`movement/separate.rs`), from the movement of the same tick on (ten Recruits in an Armory are apart within 5
    ticks). Neutral ones don't: no unit yields that has no owner (`yielding::can_yield`), so a defeated player's
    stay as they came out. One for which no tile of the footprint is ground it can stand on (buildable terrain it
    can't stand on) leaves the game (`Sim::remove_held`; Placeholder, not in the design).
  - **Measured** (`tests/holding_perf.rs`, ignored, release): 8 Armories with 10 Recruits each cycling through Enter
    and Eject All, a mean of 0.028 ms and a p99 of 0.071 ms a whole sim tick (the economy phase, `step_holding`
    with the rest, 0.001 ms); the tick that removes all 8 with 80 inside, 0.22 ms. `agents_perf` (the Tunnels'
    Enter) and `recruits_perf` ran A/B against master unchanged: the same games, the same timings (load average 3.9).
    Slice M5-F2 (its ready order, the pool producers' step) ran `holding_perf`, `structures_perf` and `recruits_perf`
    A/B against master: the same games, the same timings within noise (load average 2.4; `recruits_perf`'s tick 1, a
    single cold sample, varies with the build, 0.28 to 0.42 ms on either side). Slice M5-F3 (the evacuation inside
    the footprint) ran `holding_perf` and `recruits_perf` A/B against master, interleaved, three rounds each: the
    same games, the same timings within noise (cycling, 0.030 to 0.037 ms a tick on either side; `recruits_perf`,
    0.076 to 0.079 ms), but for the tick that removes the 8 Armories with 80 inside, 0.22 to 0.23 ms before and 0.135
    to 0.141 ms now (its commands phase, where the removal is, 0.14 ms before and 0.04 ms now: no search round the
    footprints) (load average 2.0). Slice M5-Z1 (the doorway's tile test, `Frame::blocks`) ran both A/B against
    master, interleaved, 13 rounds each: the same games, the same timings (medians on either side: cycling 0.030
    ms a tick, the evacuation's tick 0.129 and 0.131 ms; `recruits_perf` at work 0.076 ms, its Movement 0.032 ms)
    (load average 2.7 to 3.8).
  - **The pool** (`Sim::pool`, the Armory's StoredRecruits) is the units inside that no entry of the building's queue
    reserves and that aren't on their way out (slice M5-F2); `Sim::held_in` is everyone inside. The client's Enter
    pointer is slice M5-J1's (*Client architecture*, *The Recruit's panel*). Design change 0319's other holders fit
    the def as it is: the Chop Shop, the Cathedral and the Airstrip have their `HoldsDef` since slice M5-G2 (in at B,
    out at C, 10 Recruits), the Altar (slice M5-N1b), the Temple (slice M5-N3: in at B, out at C, 6 Recruits, its
    staff, *Production > Research*) and the Shrine (slice M5-N5: in at B, out at C one at a time, 8 Recruits, its
    participants, staffed as the Temple's are, `holding::staffed`; they charge its energy, *Combat > Energy*) have
    theirs, and the Gunworks and the Machine Shop (ABAC, as the Shrine) hold no one until their research is designed
    (design change 0328), so a right-click on one with Recruits is a Move.
  - **The Garrison** (slice M5-N2, plan decision T72 as design change 0335 amended it;
    `factions/cults/structures/garrison.md`): `HoldsDef { entrance: B, exit: ExitRule::AtOnce(B), capacity: 4,
    accepts: Recruit, Militant, Gunner, Disciple }`, SymmetryType AAAB: "**the door on side B** (one door, entrance and
    exit)", entered at its middle as every entrance is (F3's `holding::entry_path`; plan decision D22's nearest side
    went with 0335). It produces nothing (`defs_are_consistent`: one door only for a building that doesn't), has no
    queue and no rally point. **`ExitRule::AtOnce(side)`** is a side doorway (`production::Doorway::of`, the doorway
    of `ExitRule::Side`) through which everyone ready leaves together (`factions/cults/structures/garrison.md >
    Placeholder stats > Leaving`: "on **Eject All** they all come out **at once**, ignoring collision with each other
    until they are clear of the footprint"): `Sim::step_holding` hands such a building to **`Sim::let_all_out`**,
    which finds every ready unit's spot first (`Sim::held_spot`, the one `Sim::release_held` serves a unit let out
    alone by: the door's spot, half inside at the middle of side B, as far out as there is room) and only then puts
    them all there
    (`Sim::come_out`), each with its `LeftBuilding`, in the pool's order: all four at one spot, overlapping, in the
    doorway (`Mover::exit`). They walk straight out of the door, idle (no rally point), passing through each other
    (*Movement and collision > Collision exceptions*: units that already overlap), and part once clear, as idle
    units that overlap do (four Militants are clear of the footprint in 3 ticks and apart 29 later,
    `tests/garrison/eject.rs`); a unit of another's still blocks them, so they wait in the doorway behind it. While
    the doorway holds units it let out, the next Eject All waits, as a door does (T62's spots, which plan §N2's
    `ExitRule::AnySide` would have used, are not brought back: 0335). **Its destruction** evacuates everyone alive
    inside its former footprint, `Sim::evacuate` unchanged (*Out by destruction* above). **Its occupants fire**
    (`StructureDef::occupants_fire`, *Combat > Units that fire from inside a building*); held off the map, they can't
    be targeted or hit, a blast over the Garrison included (T72: "protection by construction"), see with the
    Garrison's SightRange (its owner's vision: no source of their own) and count toward Unit Control as on the map
    (`economy/army/cults.rs` counts the held units). A unit comes out with no attack under way
    (`Sim::put_on_map`: T72, "leaving clears it"). Right-click: the four kinds' on an own Garrison in operation is
    an Enter by `RULES`'s one Enter row (an own building in operation that takes the kind in, before N8's Repair, so a
    damaged Garrison is entered, design change 0325; (M5-S2b) but a full one, damaged, is the Recruits' to repair,
    as at every building that holds units, design change 0378: `Aimed::full`), so no row of its own.
- **Expansions and a Tunnel's operation** (`sim/expansions.rs`, `tunnel/operation.rs`; slice M4-F;
  `factions/syndicate/tunnel_menus.md > ExpandMenu`, `factions/syndicate/tunnel_menus.md > AwaitingPlacement
  (Expansion)`, `factions/syndicate/tunnel_menus.md > Expansions under construction`; design changes 0240, 0254, 0290,
  0291): a Tunnel builds an underground expansion in its area as **its one operation**, `TunnelState::operation:
  Option<Operation>` (`Operation::Building(site)`, or `Operation::Upgrading { progress, to }`, *Tunnel upgrades* below),
  hashed (a tag) and saved only while set. The
  expansion is a site as above, `Builder::Tunnel(tunnel)`, no worker held.
  `TunnelCommand::Expand { kind, origin, orientation }`'s gate (`Sim::check_expand`) is first **`Sim::can_expand`** (the
  player's Tunnel in operation, no operation under way `Reject::OperationUnderWay`, the kind's Tier Requirement
  `Reject::TierRequirement`, its TechPrerequisite `Reject::Requires`, *Tech prerequisites* below, then
  the bank), which the ExpandMenu greys its entries by, then the placement by the truth (`Sim::expansion_truth`,
  `Reject::Unplaceable`). Paid at the click, Tunnel Space used and the ConstructionHP rule from the start (0254);
  completion (`Sim::complete_expansion`) ends the operation, and so does a cancel, the site's X or the Tunnel's
  (`TunnelCommand::Cancel`, `Sim::cancel_operation`; 75% back, `Refund::Expansion`), or the site gone any other way
  (`forget_references`, no refund). **The ExpandMenu's one table** is `defs::EXPAND_MENU` (`ExpandEntry::Kind(kind)` or
  `ExpandEntry::Planned(name)`): the nine expansions of the Tech Tree table in its order and the Laser Turret tenth
  (its `Kind` since slice M4-S2: a Tunnel builds it as an expansion, *The pop-up form* below; the only builder of it
  since design change 0359), each holding its slot (0291); the slice that brings one turns its entry into its `Kind`,
  and tests check the list against the design's table and against `StructureDef::expansion`.
  `Sim::operation_progress(tunnel)` gives the operation and its `Progress` for the Tunnel's InfoPanel.
- **The pop-up form** (`popup.rs`, `sim/popups.rs`; slices M4-S2 and M5-Y, plan decision T50;
  `factions/syndicate/structures/laser_turret.md > Pop-up turret`, `factions/syndicate/structures/laser_turret.md >
  Pop-up rules`, `factions/syndicate/structures/laser_turret.md > When its Tunnel is lost`; design changes 0267, 0296,
  0297 and 0358 to 0360). **Every turret is a pop-up** (0360: "There is no upright form any more"): an object of a
  type with `StructureDef::popup` (`PopUpDef`: the rise, idle and retract frames and the underground armour; the Laser
  Turret's 8, 32, 8 and 6 / 6) has **`Object::popup: Option<PopUp>`** (`Lowered { blocked }`, `Rising(Progress)`,
  `Raised { idle }`, `Retracting(Progress)`) from its first tick, lowered (`Sim::new_object`), its site too, so the
  state is hashed (a tag) and saved only for a turret. **Active is derived, never stored**: a turret in operation is
  active while it has a parent Tunnel (`popup::is_active`, `Sim::is_active_turret`: the expansions' rule,
  `tunnel::area::parent`, an own Tunnel in operation whose current area encloses it; a reading: "in operation", as for
  reclaiming, and an upgrading Tunnel's area is its current tier's), so a new Tunnel, an upgrade and a Tunnel's loss
  change it with no event. **An inactive turret is lowered at once** (a reading of "a raised turret retracts at
  once": in the tick, with no retract frames, so that its armour is the underground one from then) and never fires:
  a neutral one (*Losing Tunnels*), and one a map or a debug spawn puts outside its owner's areas. **The cycle** of an
  active one is `popup::step`, once a tick in `Sim::step_popups` (*Tick pipeline*): lowered with no target; rising for
  its frames once phase 5e picked one, then raised, where alone it fires (`popup::may_fire`, which `combat::step`
  asks: it keeps the target it picked and faces it meanwhile); retracting after its idle frames with no target. A
  phase started in tick `t` does its first frame in `t + 1`: a target picked in `t`, raised at the end of `t + 8`, the
  first shot in `t + 13`; the last target in `t`, down at the end of `t + 40`. Every phase runs its full length (0358
  point 2: "a new target during retracting doesn't interrupt it"). **Each reader asks one function**: hiding and the
  underground armour only while fully lowered (`popup::is_lowered`: `Hiding::of`'s arm, which hides an owned turret
  alone, *Vision > Hidden objects*; **`Object::armor()`**, which every hit meets through combat's snapshot), blocking
  (`popup::blocks`, below), Tunnel Space (`economy::army::occupies_tunnel_region`: a cell of the region, so ¼, for
  every expansion in the region, a turret's site from its first frame, 0358 point 5). **Blocking** (0296 point 3, a
  reading): the footprint blocks from the tick the turret starts to rise until it is fully lowered again, so that
  nothing steps under it while it rises; `paths::blocking(object)` is the one test of a live object's footprint (the
  true grids, its owner's known ones, the gate's spawn checks), `paths::remembered_blocking` a record's (a neutral
  turret's record, always lowered, is walked over: `popup::remembered_lowered`), and `step_popups` syncs the true
  grids when a turret starts or stops blocking, as a structure that appears or goes (*Navigation*). **Rising needs a
  clear footprint**: a lowered turret that wants up while a ground unit's box overlaps its footprint stays `Lowered {
  blocked: true }`, and phase 5d of the next tick asks its owner's idle units there to step off
  (`yielding::make_way`, *Movement and collision > Friendly yielding*); an enemy or a unit with an order keeps it
  down. **A Tunnel's turret** is the ExpandMenu's tenth entry (`StructureDef::expansion`, its one way to be built:
  slice M4-S1's `site`, an Agent's Build, went with 0359), placed by `Method::Expansion` with the ground a surface
  structure needs as well (Buildable, flat and Drillable, 0358 point 3), from Tier 1 (0358 point 4), its site lowered
  from its first frame: hidden, walked over, and taking the Tunnel's one operation (0267). Tests: `popup.rs`'s (the
  cycle), `tests/popup/` (`cycle`, `blocking`, `hidden`, `building`, `active`: 0358's and 0360's Verifications),
  `combat/area/tests.rs` (a blast against each armour). **Measured** (`tests/popup_perf.rs`, ignored, release; 12
  pop-ups in three areas on 256 × 256, their pairs rising and retracting 10 ticks apart every 64, under 200 walkers
  crossing them; four interleaved A/B rounds against the same file on a build without pop-ups, load 2.3): 227 changes
  of the true grids and 495 sets of grids rebuilt in 1280 ticks; a whole tick 0.197 ms on average against 0.179 ms
  (+10%), 0.41 ms against 0.35 ms at the 99th percentile, the worst tick (the group orders) unchanged: about 0.1 ms a
  tick in which a pop-up starts or stops blocking. `nav_perf`, `movement_perf` and `structures_perf`, which have no
  turret, are as they were.
- **Tech prerequisites** (`tech.rs`; slice M4-K, plan decision T37; `core/tech_and_research.md > Tech Prerequisites`,
  design changes 0229 and 0260): **one rule for every faction**. A structure type's TechPrerequisite is
  `StructureDef::requires`, a recipe's "Requires" (a unit's in `Produces`) `Recipe::requires`, kinds leaked from the
  defs (`leak::slice`); `tech::recipe_requires` gives both for a recipe, its own and its building's. **What counts**
  (`tech::counts`, `Sim::requirement_met`): a structure of the type, the player's own and in operation
  (`Object::is_operational`: no site, no transformation; a GDO building ready to place is no object yet), anywhere on
  the map, so anywhere in a Tunnel Network; a neutral one is nobody's. **The gate checks it only as the work starts**
  (`Sim::check_requires`, `Reject::Requires(kind)`): a Train as it is queued (`plan_training`: a producer whose recipe
  isn't met is passed over, and with none left the reason is the first unmet kind), a Construct, a Warp In, an Expand
  (`Sim::can_expand`) and a worker's Build as they are ordered (`Sim::check_build`; the worker's arrival doesn't check
  again: `core/tech_and_research.md > Tech Prerequisites > Check Timing`, "checked only at the moment construction is
  **started** (queued at the constructor, ...)", read as the order, which is when it is paid; slice M4-S1), each before
  the bank, so a greyed entry says "Requires ..." rather than the price. Nothing checks again:
  work queued or started goes on when the last one is lost, a worker's Build ordered and paid but short of its site
  among it, and a finished building keeps working and counting (design change 0335: "losing a prerequisite never
  cancels or disables anything already started or finished"; `sim/tech/tests/losing.rs`, every faction's cases)
  (`core/tech_and_research.md > Tech Prerequisites > Losing a Prerequisite`). Today the Hideout's Guard and the Laser
  Turret (slice M4-S1; the ExpandMenu's alone since design change 0359) require the Optics Lab, and the Hideout's
  Grenadier the Chemical Plant (slice M4-U); the Optics Lab and the Chemical Plant are placeholder expansions (2 × 2,
  150 Space Crystals, 400 frames, HP 400, 6 / 24) that only count. The GDO's Career Campus requires a Barracks, and
  the Barracks' Field Engineer the Career Campus (slice M5-N9, `tests/field_engineer/prerequisites.rs`); the Arsenal
  requires a Barracks, and the Barracks' Rocketman the Arsenal (slice M5-K, `tests/rocketman/prerequisites.rs`).
  `sim/tech/tests.rs` tests the rule with test defs (a
  GDO unit and building, a Colonists Warp In, a worker's Build, an expansion), `tests/prerequisites/` with the design's
  own types. The client shows `Reject::Requires` as "Requires a/an <name>" (`feedback::requires_text`), every faction's
  entries alike (*The Syndicate's panels*).
- **Losing Tunnels** (`sim/tunnels.rs`; slices M4-H and M5-Y; `factions/syndicate/tunnels.md > Losing Tunnels`,
  design changes 0254, 0258 and 0360): what a Tunnel leaves when it leaves the game, on the removal path (*Removal
  path*): its unfinished expansions (`built_by`: the sites it builds, `Builder::Tunnel`, slice M4-F) are destroyed with
  no refund; its completed ones, its owner's expansions wholly inside its area (`tunnel::is_expansion`: the
  underground structures and the Laser Turret; a Tunnel still a site has none), become **neutral**, seen by everyone
  from that tick: their unit in production lost, the rest of the queue refunded (`SimEvent::ProductionCancelled`, the
  lost one's with 0), the rally cleared, a turret lowered at once with its target dropped (it never fires while
  neutral, *The pop-up form*); then the units in the owner's network that no remaining Tunnel lets out
  (`network::admits`, the one exit rule: a site is none) are removed (`Sim::remove_held`). A neutral expansion decays:
  **`Object::decay: Option<Progress>`**, hashed (a tag) and saved only while set, steps
  `tuning::tunnel::Tunnel::decay_percent_per_second` (1%, placeholder) of a 100-second decay a tick, and its HP follows the
  share of its MaxHP the decay has left (`decayed`, computed from the whole ticks in 64 bits, so that a full Hideout
  dies in its 1600th tick of decay, the first being its Tunnel's last; a turret loses 3 HP a second of its 300). An
  owned one under its Tunnel regenerates instead (slice M5-N8, *Economy > Expansions' regeneration*).
  **Reclaiming** is `Sim::reclaim(tunnel)`, which a site's completion calls (`Sim::complete_site`) and an upgrade's
  (slice M4-G): every neutral expansion wholly inside the finished Tunnel's area, at or below its tier (the Tier
  Requirement, `ExpansionDef::tier`: 1 for the Hideout and the turret), becomes the Tunnel owner's, stops decaying and
  is hidden again, with no event: a turret is active again. **No turret keeps a Syndicate player in the game** (0360:
  "losing its last Tunnel defeats the Syndicate unless it owns some other structure"): it is neutral with the Tunnel's
  other expansions, so the shared rule (*Economy > Defeat and the end of the game*) holds as it is. Tests:
  `tests/losing_tunnels/` (`neutral`: every expansion kind seen by every player at once, the defeat),
  `tests/popup/active.rs` (the turret's decay, fire and reclaim).
- **Tunnel upgrades** (`sim/upgrades.rs`; slice M4-G; `factions/syndicate/tunnels.md > Cost Scaling > Upgrade
  Costs`, `factions/syndicate/tunnel_area_and_space.md > Tunnel Area Non-Overlap Rule`; design changes 0229, 0231, 0232,
  0237, 0242, 0249, 0283, 0284): `TunnelCommand::Upgrade` takes a Tunnel to the tier above as its one operation,
  `Operation::Upgrading { progress, to }`. **The data** is the tier table's: `TierDef::upgrade: Option<UpgradeDef {
  frames, base, per_tunnel }>` on the tier it reaches (480 frames, placeholder; 2 + 2 per Tunnel, 3 + 3 per Tunnel).
  **One price function**, `Sim::price_to`, counts the owner's other Tunnels whose `TunnelState::reserved_tier` (the
  tier, or the one it upgrades to) is the target or above, never the Tunnel itself: the order pays it
  (`Sim::upgrade_price`, which the panel shows), and the Tunnel's X refunds it as it is then (`Sim::cancel_upgrade`,
  "the current top price"); a Tunnel destroyed mid-upgrade loses it. **The gate**, `Sim::can_upgrade` (the panel's
  Q), as `Sim::can_expand`: the top tier (`Reject::TopTier`), an operation under way, the Supplies, then the truth
  (`Reject::UpgradeBlocked`, never what): the area at the target tier on the map and Drillable (`area::breaches`) and
  clear of every other Tunnel's `area::reserved`, any player's, a site's and another upgrade's target included. The
  owner's lingering ghosts don't block it (a reading: a ghost is no Tunnel, an upgrade no placement), and their builds
  then fail on arrival. **The reserved area**: from the start, the Tunnel holds its target tier's area against every
  placement and upgrade (`Occupant::of`, the truth and its owner's knowledge; the other players judge by the tier they
  see, `Occupant::as_seen`, and records keep only the tier), while transit, exits, Tunnel Space and reclaiming read
  the tier it has (0249). **The frames**: `Sim::step_upgrades` (*Tick pipeline*); completion is `Sim::set_tier` (the
  HP rising with the MaxHP), then `Sim::reclaim`. Events `UpgradeStarted`, `UpgradeCompleted`, `UpgradeCancelled`.
  Tests: `tests/upgrades/` (`costs`, with the books over random orders; `blocked`, every edge; `during`; `completion`)
  and `sim/upgrades.rs`'s (0231's 500/600 to 700/800).
- **Side A's paths** (`network::Paths`, design changes 0225 and 0248): the one geometry of a Tunnel's entry and
  exit, for the sim and the client's drawing (`object_render/tunnel.rs` reads `Tunnel::path_offset_su`): two spots on Side
  A's line, 30 su from its middle, the exit towards Side B and the entry towards Side D, wherever the sides face
  (`sides`). A unit stands on one with its box against the wall (`entry_stand`, `exit_stand`): (M6-SV, plan decision
  T100) its box as it lies there, a vehicle's Silhouette turned with it (slice M5-V1), facing out of Side A as it is
  ejected (`sim/network/ejects.rs`'s `exit_half`, its fit checked on that box, `Sim::check_unit_fits_facing`) and
  facing the wall as it drives in (`orders::enter`, `EntryPath::reached` on `Shape::bound_half`); a square
  Silhouette's is its own, so infantry is as it was. `network::admits(tunnel,
  kind)` is the one tier rule (`factions/syndicate/tunnels.md > Transit Tier Requirements`; the tier a Tunnel has, which
  an upgrade leaves until it completes, 0249). `tunnel::area::parent`
  derives an expansion's parent Tunnel (plan decision T32: the owner's Tunnel whose Tunnel Area, `tunnel::area::of`,
  fully contains it; `Sim::parent_tunnel`).
- **The air doors** (slice M6-SD, plan decision T96; `factions/syndicate/tunnels.md > Tunnel > Look by tier`: "Aircraft
  use these doors, not the Side A paths"): an air unit passes at Tier 3 alone (`network::uses_air_doors`), through two
  roof points, `Paths::air_entry` and `Paths::air_exit`, `Tunnel::air_door_offset_su` from the Tunnel's centre towards
  Side D and Side B, over the entry and the exit path's halves, turning and flipping with it (the client draws them
  there). **In**: its Enter flies it to the entry door (`orders::enter`); once its centre is over it
  (`network::over_door`, `Tunnel::air_door_radius_su`) the door lets it descend for its type's `UnitDef::air_door_ticks`
  (design change 0431: the Phantom's 16), on the map, an air target, and it goes in (`Sim::fly_in`, `Enter::descent`).
  **Out**: an eject queue has two lanes, its ground units out of Side A's exit path and its air units out of the exit
  door (`Sim::eject_air`), each lane in its order and neither waiting for the other; an aircraft appears over the door,
  inside the map (`Map::clamp_box`), its order given at once, and rises for its `air_door_ticks`, holding still
  (`Mover::rise`, the shape of `Mover::takeoff`; hashed and saved only while set, as `Enter::descent` and the doors'
  clocks). Each door lets one aircraft through every `Tunnel::air_door_interval_ticks` (design change 0251's 8;
  `TunnelState::air_out`, `TunnelState::air_in`). `network::door_lift` is how far up out of a door a unit is, which the
  client draws (`object_render/flight.rs`). Tests: `tests/air_doors/`, the golden `syndicate_air`.
- **The Null Chamber** (`network/services.rs`, `sim/network/ejects.rs`; slice M6-SN, plan decision T98;
  `factions/syndicate/structures/null_chamber.md > Null Chamber Exit Buff`, design change 0424): a Tier 2 Tunnel's
  expansion behind a Hideout, its service's numbers its def's (`StructureDef::chamber`, a `ChamberDef`: 4 slots, a 2 s
  turn, a 20 s buff, +2 GU of sight, a 20% reduction, Placeholders; `Ruleset::check` bounds the turn as a recipe).
  **The turns** (`services::step`, in `Sim::step_network` after the regeneration and before the eject queues): while a
  player owns a chamber in operation (not a site; one left neutral serves no one), the manned units in their network
  (`services::manned`: the Syndicate's unit types; satellites, `UnitDef::satellite`, are slice M6-SC's) take their
  turns first come first served, by owner, `Held::since` (the tick it came in; a produced unit's, its production's)
  and id: the first so many whose turn isn't done, the slots of all the player's chambers together, each take a tick
  of it (`Service::chamber`, a `Progress` of the turn's ticks), the others waiting with what of theirs is done. So a
  unit in, or finished, in tick E takes ticks E to E+31 and may leave buffed at E+32, as the transit delay lets it; a
  chamber finished in tick C serves in C. A turn done is kept while the unit stays inside, with no chamber left too
  (design change 0463). **Side by side** (0424, Q0158): each service has a queue of its own, and a field of its own in
  `Service` hashed under a tag of its own (the chamber's 0; the Vault's turn, slice M6-SB, is the unit's tanks); the
  EjectMenu takes the unit inside longest, done or not. **The buff** (`Sim::leave_network`, both lanes: Side A's exit
  path and the air exit door): a unit whose turn is done, out whatever became of its owner's chambers meanwhile
  (design change 0463, answering Q0187: "**A finished turn always buffs**"), holds `BuffKind::NullChamber` from that
  tick for 320 ticks (`buffs::hold`, a restart if it had one): +2 GU in its eye (`vision/sight.rs`'s `range`,
  `services::sight_bonus`; the auto-targeting scan and the AttackMove reach keep the type's SightRange, 0463: "**vision
  only**", though a scan takes only what its owner sees, so a buffed unit alone may engage up to its and its target's
  half extents sooner, `tests/null_chamber/buff.rs`'s reading) and the hits on it × 0.8 before armour
  (`services::damage_factor` in `combat::pre_armor_multiplier`). `buffs::step` drops it as any buff, in the 319th tick
  after the exit's. Detection, the design's first effect, isn't built (0267). The step returns at once while no unit
  waits or no player owns a chamber: `tests/agents_perf.rs` (no chamber) measured the same A/B against master (mean
  0.090 ms a tick with the network, 0.125 ms shelled; load 3). Tests: `tests/null_chamber/`, the golden
  `syndicate_null_chamber`.
- **The Vault and the Sludge** (`patches.rs`, `network/services.rs`, `defs/units/tanks.rs`; slice M6-SB, plan decisions
  T93 and T98; `factions/syndicate/units/shadebringer.md`, `factions/syndicate/structures/vault.md`, design changes
  0369, 0424 and 0425). **The tanks** (`Object::tanks`, a `Tanks`): a unit whose type has tanks (`UnitDef::tanks`, each
  a `TankDef`: what it lays, its capacity, drain and refill, its patches' life and width and their slow, all def
  numbers) has them from the start, full (Placeholder), in fixed point (`RATIO_ONE` to a unit of charge: 100 is 6553600,
  a tick's drain 40960, a tick's refill 20480); the Shadebringer's Sludge (R) and its Cover (F, slice M6-SC, which adds
  its smoke's `TankDef`). **The toggles** (`Ability::Sludge { on }`, `Ability::Cover { on }`, `patches::switch`): the
  mode-switch rule (`modes.rs`: applied in the commands phase, the orders kept, never queued), on with any charge left
  and refused with none (`Reject::TankEmpty`, `check_charged` in `gate/orders.rs`), off by itself once the tank is
  empty, and off as the unit goes into the network (`Tanks::switch_off` in `Sim::take_off_map`). **The trail**
  (`Sim::patches`, a `Patches` of `Patch` discs, kept by the 2 GU cell of their centres, a lookup reading the cells
  round its point; `patches::lay`, after the tethers): every tick a toggle is on, its unit lays a disc as wide as the
  trail (48 su) at its rear edge where 5d left it, or its front while it backs up (`patches::trail_point`), a new one
  each time that point has moved `tuning.patches.spacing` (24 su, Placeholder) from the last, the last laid again until
  then, its life starting over (a parked one holds its spot), and its tank drains. A patch lasts its `TankDef::life`
  (160 ticks) and goes in the step of its `until`. **The slow** (`patches::slow`, first in 5d): a unit of a base sludge
  slows (`SlowDef::bases`: infantry, the non-hover vehicles, Mechs, Drill Units on the surface), on the ground, whose
  centre the last tick left in a sludge patch (its edge too, Placeholder) holds `BuffKind::Sludge` for 33 ticks, the
  tick and 2 s after, its top speeds × 0.6 while it does (`patches::speed_factor`, after the buffs' in
  `Object::movement_ref`): its own layer too; hover units and aircraft never. **The Vault** (`StructureDef::vault`, a
  `VaultDef`: 2 slots; `services::vaults`, the services' first pass): while a player owns a Vault in operation, the
  units in their network with a tank that isn't full take their turns first come first served (`Held::since`, then id):
  the first so many, the Vaults' slots together, each have every tank of theirs filled a tick's refill. The tanks are
  the turn, so it has no field in `Service` (Service sub-tag 1 stays free): over when the unit leaves, however full
  (0424: "half-filled if ejected early"); nothing drains a held unit's tanks; one slot fills both (Placeholder). It runs
  beside the Null Chamber's queue (0424): a unit is served by both at once. **Hashed** only while there is something:
  `Sim::patches` under tag 4 of `Sim::state_hash`, `Object::tanks` under tag 24, each tank under a sub-tag only while it
  isn't full and off, `BuffKind::Sludge` as the number past the Null Chamber's; saved the same way, so no state from
  before moved. Invariant N1 (*Movement and collision > Invariants*). The client draws the trail (`patches_view.rs`),
  the tanks' bars and the slow's mark (*Client architecture > Visuals*), and the toggles' keys (*The Shadebringer's
  panel*). Tests: `tests/sludge/`, the golden `syndicate_sludge`. **Measured** (`tests/movement_perf.rs`, ignored,
  release; three sets of five to eight rounds interleaved with master's build f4651a44, load 4 to 8): the 500 units on
  256 × 256 the same, a mean of 0.71, 0.67 and 0.86 ms a tick against 0.69, 0.71 and 0.87; through four Shadebringers'
  trails (a mean of 109 patches on the ground, 11.7 units slowed) 12 % and 17 % over the same game with the
  Shadebringers' Sludge off (1.03 against 0.92 ms at load 5, 0.81 against 0.69 at load 8), most of it the slowed armies'
  moves (7 % more box tests and 10 % more repath searches, deterministic counters) and the lookups of every unit where
  it stands, 20 to 40 µs a tick timed alone (a filter of the cells with a patch, tried, gained 2 % of the tick, within
  the runs' noise, and was left out). The network case of `tests/agents_perf.rs` the same: 0.080 and 0.094 ms a tick,
  0.110 and 0.129 shelled, against 0.083 and 0.094, 0.111 and 0.131.
- **Carriers** (`carry.rs`, `sim/holding/carriers.rs`, `orders/unload.rs`; slice M6-ST1, plan decision T94;
  `factions/syndicate/units/subterranean_transport.md`, design changes 0367's placeholders, 0419 and 0432): **one
  capability, data on the def**, `UnitDef::carries: Option<CarrierDef { capacity, places }>`, its places and the bases
  it takes with the places a unit of each takes (`Places`); the Subterranean Transport's two types share one, 4 places,
  an infantry unit 1. M6-GS2's Supply Chopper (4 places, a vehicle 4, one kind at a time) and M6-CP's Pilgrim (16
  places, loading only while landed) reuse it: a carrier loads while it is a ground object (`carry::loads`, through
  `combat::object_domain`, whose landed arm makes a landed aircraft one). **The surfaced kind's Domain**:
  `UnitDef::domain: Option<Domain>` overrides its base's, as do `crushes` and `yields`; every reader asks the
  accessors (`UnitDef::domain()`, `crushes()`, `yields()`, `nav_class()`: targeting's `kind_domain`, the movement
  layers and crushing, the scans, the releases and the parking, placement, the popups, the planning grids, the
  sludge's slow, the overlap invariant), so the surfaced Transport is a Ground unit that crushes and turns as a tracked
  vehicle (0417 to 0419), on the ground layer, a Ground attack's target and not an anti-air one's
  (`Reject::CannotHit`), slowed by the sludge, and the burrowed one keeps its DrillUnit base's Underground. **Aboard**,
  a passenger is held at `Place::Carrier(carrier)` (`Sim::passengers`: longest aboard first, by `Held::since` then
  id), so the store's references, hash, saves and defeat hold for it, and it goes wherever its carrier goes, into the
  Tunnel Network too: `Sim::network` lists `Place::Network` alone (the EjectMenu shows the carrier, not its
  passengers), the network's regeneration and services serve that place alone (0432 point 2: no regeneration, no Null
  Chamber turn, no Vault refill), and the Tunnel Space counts every held unit (`Sim::army_cap`). **Boarding**: the
  Enter order on a carrier (`check_enter`'s carrier arm, `network::admits` asking `carry::admits`; the owner's units
  of a base it takes), a walk after it as it drives (`orders::enter`'s carrier goal) and the entries' step
  (`Sim::board`: within Proximity of its shape from any side, if `Sim::room` has the places; else the Enter ends,
  `EnterFailed` with `full`). **Unload All** (`OrderKind::UnloadAll`, F, untargeted and queueable; `check_unload`:
  `LacksAbility`, `NothingInside`, then the type's rule): the order's start stops the carrier, its behaviour never
  completes, and the economy phase's `Sim::step_unloads` (*Tick pipeline*) ends it once the carrier is at rest and
  puts every passenger out (`Sim::unload`): the `i`th of `n` at the first free spot of the rings round the carrier's
  shape (`carry::ring_spots`: rectangles `Holding::unload_gap_su` (4 su) clear of the shape and of each other, spots
  on the corners and along the sides, `Holding::unload_rings` (3) of them), walking round the nearest ring from its
  share of it; four Guards come out on the first ring's corners. A passenger with no free spot stays aboard, for
  the next F (`systems/mechanics/carrying.md > Unloading`). **Death**: the removal path takes a carrier's passengers
  along (`Sim::passengers_of`): on the map each dies with a `SimEvent::Died` where the carrier was
  (`Sim::passengers_die`), held (stranded, a defeat) each is removed with it (`Sim::remove_held`). **No firing
  out** (0432 point 1): a held unit fires nothing; only a building's occupants fire from inside
  (`combat::occupants`). **Hashed and saved** as the store is: `Place::Carrier` is the
  Place's tag 3, `OrderKind::UnloadAll` the order's tag 25, so no state from before moved; invariant U1
  (*Movement and collision > Invariants*). The client: the Transport's panel (Q Move, R Burrow or Surface held for
  slice M6-ST2, F Unload All, X Stop; `templates/syndicate/transport.rs`), the Vehicle Assembly's R live, and the
  passenger pips over the carrier's bars, to its owner (`HpBar::passengers`). Tests: `tests/transport/`, the unit
  tests in `carry/tests.rs` and `sim/holding/carriers/tests.rs` (a pocket of cliffs), the golden
  `syndicate_transport`. Perf, interleaved against master's binary (10 rounds, load 3 to 5): `tests/movement_perf.rs`'s
  500 units 0.637 ms a tick against 0.649, `tests/vehicles_perf.rs`'s 60 vehicles 0.127 against 0.127,
  `tests/attack_move_perf.rs`'s 0.215 and 0.225 against 0.214 and 0.228, and the network case of
  `tests/agents_perf.rs` 0.084 and 0.113 shelled against the same: within the runs' noise.
