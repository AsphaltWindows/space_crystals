# Economy

Read this when you touch banks, income, Supply, gathering rates, costs or recipe timing.

## Economy

The players' banks, power and army caps, the plates' mining, construction, and defeat and the end of the game
(`economy.rs`, `economy/`, `construction.rs`; slices M2-B, M2-D1, M2-F2 and M3-C; `factions/gdo/overview.md >
GDOResources`, `factions/gdo/economy.md`, `factions/colonists/overview.md > ColonistResources`). Production has a
section of its own.

- **Banks** are the only resources stored: `Player::crystals` and `Player::supplies`, and the Colonists' refined
  banks `Player::alloys`, `essence` and `flux` (slice M3-C, plan decision T21; nothing earns or costs them yet). All
  are `u32`, hashed, and `#[serde(default)]` so that older states load. A game starts them at the player's faction's
  Starting Conditions (`Player::starting`, `StartingConditions`, slice M2-F1; see *Sim scenarios and tests*): a GDO or
  Colonists player's at 100 Space Crystals and the rest at 0, a Syndicate player's at 100 Space Crystals and 1 Supply
  (slice M4-B), a Cults player's all at 0 (slice M5-B).
  `DebugCommand::SetResources` sets all five (`Player::set_banks`; the refined ones `serde(default)` in command files
  too), and `Player::banks` reads them as one `Banks`. Pay with
  `Player::spend_crystals`, which refuses when the bank is short and leaves it as it was; credit with
  `Player::credit_crystals`, which saturates (only a debug command fills a bank that far, and input never panics the
  sim). Every refund goes through `economy::Refund` (`Full`, or `Ready`: 75 %
  rounded down). Supplies pay for an Agent's Tunnel (slice M4-E, *Agent-built Tunnels* below: `Player::spend_supplies`,
  `credit_supplies`); nothing else costs them yet.
- **Power and the army caps are derived** from the objects whenever they are asked for, never stored, so they can't
  drift (plan decision T7). `Sim::power(player)` is a `PowerSummary`: `generated` and `consumed`, the HUD's
  used / available (design change 0090). `Sim::army_cap(player)` is an `ArmyCap { resource, used, available }` for
  the player's faction (`economy/army.rs`, plan decision T20; `None` only for a player not in the game): the GDO's
  Unit Control, out of `GdoRules::unit_control_cap`; the Colonists' Beacon Capacity, out of
  min(`ColonistsRules::beacon_capacity_cap`, `ColonistsRules::beacon_capacity_per_building` × their Colonists structures, a Beacon under deployment
  left out until it completes: design change 0112, *Beacon Deployment*); and the Syndicate's **Tunnel Space** (slice
  M4-B, `economy/army/tunnel_space.rs`; `factions/syndicate/tunnel_area_and_space.md > Tunnel Space`, design changes
  0242, 0258 and 0265), out of min(`SyndicateRules::tunnel_space_cap` 200, ⌊the unused tunnel-region cells of the player's Tunnels /
  `SyndicateRules::cells_per_tunnel_space` 4⌋): each Tunnel's region is its Tunnel Area at its current tier less its own footprint
  (`tunnel::area::region`: 48, 84 and 128 cells), every cell counted, on the map or off it, Drillable or not (0242);
  a cell is used when an underground structure of the player's or a neutral one stands on it, cell by cell (0258
  point 4), never another player's. The regions are summed Tunnel by Tunnel, as the formula reads. And the Cults'
  **Unit Control** (slice M5-C, plan decisions T56 and T57; `economy/army/cults.rs`, `recruitment.rs`;
  `factions/cults/structures/recruitment_hall.md > Cults Unit Control HUD Display`): available is the sum of their
  completed Recruitment Halls' capacities, each rounded down (design change 0341 point 2: "Capacity is rounded down
  everywhere", so two Halls of 6.6 give 12); used is their Halls' usage, the entries of every unit's lineage that
  name them and each Hall's Recruit in production (design change 0342 point 3), plus their orphaned points, the
  entries whose Hall is gone and the points of a Cults unit's cost that no entry stands for (design change 0344:
  "Recruits no Hall produced (custom maps, debug) are orphans from the start"), the units held off the map counted
  too. **A unit costs only in its own faction's
  resource** (`ArmyResource::cost_of`; the Director's decision, slice M5-C): another faction's unit, which only a
  debug or a Custom game gives a player, costs 0 in it, both ways (`economy/army.rs`'s
  `another_factions_unit_costs_nothing_in_either_direction`), so the Recruit's and the Peacekeeper's shared
  `unit_control_cost` field never mixes the GDO's and the Cults' Unit Control. Used is the `ArmyResource::cost_of` of
  the player's units plus, for a resource that `reserves`, their producers' units in production (design change 0143,
  for the GDO, the Colonists and the Syndicate: `Production::in_production`, the unit at the front once its progress
  has begun, a finished one waiting inside included; a unit only queued, or waiting at 0% for room, takes nothing).
  The Cults' Unit Control reserves nothing (`ArmyResource::reserves`): a Hall counts its own Recruit in production,
  and a unit trained of Recruits embodies their points (*Production > Pool buildings*). The cap is checked when a
  unit's production would start, never when it is queued (*Production*), but for the Cults', which is for display
  only and makes nothing wait (design change 0344: "Global total is for display only"). Capacity that falls below
  what is used leaves the units and the unit in production alone, and no new unit starts until there is room (design
  changes 0112 and 0143, Q0041 point 2). `Sim::unit_control(player)` is the GDO's used figure alone, which the HUD
  reads. A
  building counts from the tick it is placed until the tick it leaves the game, that one excluded (design change
  0089), because it is among the objects in between. Each call walks the objects once.
- **Speed in fixed point** (plan decision T6): `RATIO_ONE` (2^16) is 100 %. `PowerSummary::ratio()` is
  `min(RATIO_ONE, ⌈generated × 2^16 / consumed⌉)`, and `RATIO_ONE` without consumers. `PowerSummary::speed(kind)` is
  what a structure of that kind advances in a tick: the ratio for a consumer (negative Power), `RATIO_ONE` for any
  other ("power producers are never slowed", design change 0089). A task of `n` frames at full speed is done when its
  progress reaches `n × RATIO_ONE`, and the rest carries over. Rounding the ratio up gives an exact ratio its exact
  tick count: at 30/51 the k-th 48-frame cycle ends in tick ⌈81.6 k⌉.
- **Recipe timing** (the Director's convention, for every recipe: construction, production and any to come):
  progress accrues in the tick in which the command applies, since the commands apply before the economy phase of
  their tick (plan decision T5). A recipe of `N` frames ordered in tick `t` at full speed completes in tick
  `t + N − 1`, and what it makes can be used from tick `t + N`: a ready building placed, a produced unit acting (it
  comes into the game in tick `t + N − 1`, and the spawn-tick rule has it act from the next). So a Generator (240
  frames) ordered in tick `t` is ready at the end of tick `t + 239` and can be placed in `t + 240`, and a Peacekeeper
  (80 frames) trained in tick `t` is released in tick `t + 79`. A constructor or producer that comes into the game
  in tick `t` starts on its recipes in `t + 1` (spawn-tick rule).
- **Mining** (`economy/mining.rs`, `factions/gdo/economy.md > ExtractionPlate`): a structure with a `MiningDef`
  (`StructureDef::mining`: the plate) that stands on a patch has an `Extractor { patch, cycle }`, linked in
  `Sim::insert_object` once both are in the game, whichever comes first; a plate on bare ground (a debug spawn's) or
  without an owner mines nothing. Each tick the cycle advances by the owner's ratio. When it reaches
  `cycle_ticks × RATIO_ONE` (48 frames), the plate pays its owner a whole load: 10 while the patch holds 10, then what
  is left, then the residual 1 forever (design changes 0098, 0099; `SimEvent::PatchDepleted` when the patch reaches
  0, and it stays under the plate). Spawn-tick rule: a plate placed in tick `t` pays first in tick `t + 48`.
- **Gathering** (`economy/gathering.rs`, `orders/gather.rs`, `sim/workers.rs`; slices M3-H and M3-H3, plan
  decision T17; `factions/colonists/units/drone.md > Gathering`, `factions/colonists/economy.md > Egress Node >
  Drop Off`): a unit type with a `WorkerDef` (`UnitDef::worker`: the Drone's CarryCapacity 8, MiningDuration 32,
  DropOffDuration 2, the two reaches of 8 su and its search radius) is a worker, and a structure type with
  `StructureDef::drop_off` (the Egress Node, the Tunnel) takes its loads. `Object::worker` (`Worker`) holds only what outlives
  an order: the load and the AssignedDropOff (the patch a Gather goes back to is the one its order names, design
  change 0132). The job is an order and its behaviour (see *Behaviours > Gathering*), and
  **the timed work is part of the order's state** (`GatherPhase::Mining { done }`, `DroppingOff { done }`), so an
  order that ends takes its unfinished work along and frees the patch at once. `gathering::step` counts the
  frames in the economy phase, by *Recipe timing*: a worker that reaches its patch in tick `t` holds its load
  (`min(8, remaining)`, taken from the patch) at the end of tick `t + 31` and the patch is free in `t + 32`; one
  that reaches its drop-off in tick `t` is paid at the end of `t + 1` (`SimEvent::ResourcesDelivered`). A patch
  emptied this way leaves the map in the same tick, unless a plate stands on it; workers never gather a plated
  patch. `Sim::returning_drones(node)` is the Egress Node's F.
  **Worker types and drop-offs are data** (slice M4-D, plan decision T34): a `WorkerDef` has a `LoadDef` per
  resource (`crystals`, and `supplies`, the Agent's since slice M4-I: 1 a trip, 48 frames to pick up and 48 to unload,
  reaches of 8 su, `WorkerDef::load(Resource)`), `shares_patches`, `drop_off` (`DropOffChoice`: `Assigned`, the
  Drone; `EachReturn(Nearest)`, chosen as each return starts, `StraightLine` for the Agent, design change 0384, and
  `EdgeToEdge` for the Recruit, slice M5-D and design change 0375), `search_radius`
  and `pass_through`; `StructureDef::drop_off` is `None`, `DropOff::Anywhere` (the Egress Node, the Storage) or
  `DropOff::Hatches { crystals, supplies }` (the Tunnel: Sides B and D, design change 0248; `DropOff::hatch`).
  `Worker::resource` (`defs::Resource`, Crystals by default) is what the load is, kept in saves and hashed only while
  it is Supplies; a delivery credits the bank of its resource, and `SimEvent::ResourcesDelivered` says which.
  `economy/stations.rs` is the station's side of a pick-up: the Supply taken as the pick-up ends, and the
  notification of an empty station (slice M4-I, see *Behaviours > Gathering*).
  `economy/drop_off.rs` turns that into a `DropPoint` for a resource (`drop_point_of(object, resource)`: shape,
  centre, side) and has **the hatch square**
  (`hatch_offset`, `hatch_tile`, `hatch_square(tunnel, Resource)`, also `tunnel::hatch_square`): the middle tile of
  the side, of two middle tiles the one farther from Side A, read off `sides()` so that it turns and flips with the
  structure; the client draws the hatches there. The Agent: 50 a load, 64 frames of mining (design change 0294; 80
  before), 64 of unloading, reaches of 8 su; alone it delivers every 128, 134, 156, 176, 198, 220 and 240 ticks at 0
  to 6 GU from hatch to patch, edge to edge (the design's table: 8.0 to 16.0 s; all within 15%).
  **Measured with DragMovement, the rules of slice M3-H3 and the Drone's numbers of design change 0205**
  (`tests/gathering/`, `tests/gathering_soak.rs`; nothing is tuned to the design's table): one Drone with its
  patch 2 GU from the node, edge to edge, delivers every 73 ticks, 1.75 Space Crystals per second (81 ticks and
  1.58 before 0205; 60 ticks on the stand-in mover it had before DragMovement). On the reference field (8 patches
  in a row, the node 2 GU south of the middle), with gathering Drones passing through each other (design change
  0106), nothing jams: 4, 8, 12, 16, 20 and 24 Drones dealt out over the patches give 6.9, 12.7, 18.8, 25.3, 26.1
  and 31.4 per second (before 0205: 6.3, 11.7, 17.2, 23.1, 24.5, 26.2; before M3-H3: 17.9 for 12, 25.6 for 20,
  27.3 for 24; the stand-in: 8.3, 15.1, 22.0, 26.8, 32.0, 32.0; the design since design change 0236: 6.9 for 4,
  about 25 for 16, about 26 for 20, 32 for 24). Three findings for the economy oracle (slices K1 and K3): 24
  Drones dealt out nearly saturate 8
  patches (before 0205 they didn't: the end patches' trips were longer than three Drones' mining covered); a
  Drone sent on from an occupied patch to the nearest free one often finds that taken too when it gets there
  (dealt out, the longest one of 24 went between two deliveries is 393 ticks, within the plan's 480, 30 s; 357 to
  578 across earlier builds); and the waiting Drones' re-check (design change 0132) costs up to 0.6 per second
  below 16 Drones and up to 1.3 over 16 (measured in slice M3-H3): the patch a waiting Drone leaves for is
  usually one whose own Drone is on its way back to it. The 24-Drone soak costs 25.4 µs a tick in release (24.4
  before slice M3-H3's rules and 13 on the stand-in, each measured side by side: a Drone's tick is the rules' step
  and then its own); with the numbers of 0205, 25.8 against 27.0 before them, side by side (slice M3-K3). Under
  hard collision, as slice M3-H first had it, the field gridlocked from 12 Drones on.
- **Removal** (`Sim::remove_objects`): a plate takes its patch along if that is depleted, in the same tick, which
  opens the tile (design change 0038; the caller's grid sync covers it); a patch with crystals left stays, uncovered.
  A plate whose patch is removed (only a debug command does that) loses its extractor. `testing::referenced_ids`
  lists the extractor's patch, so `dangling_references` checks it.
- **Construction** (`construction.rs`, `sim/construction.rs`; slice M2-D1;
  `factions/gdo/structures/deployment_center.md`, `factions/gdo/economy.md > ExtractionFacility`): a structure type
  with a `Constructs` list (`StructureDef::constructs`: each `Recipe` a building, its cost and its frames) is a
  constructor, and its object has a `Constructor`: `Idle`, `Constructing { kind, progress }` or `Ready { kind }`, one
  building at a time and one ready at most (design change 0071). `Command::Construction` drives it (see *Commands,
  events and replays*): Construct pays the cost as it applies, the cancels refund through `Refund` (`Full` while
  constructing, `Ready` once ready), and so does a constructor that leaves the game (design change 0084, in
  `Sim::remove_objects`, for a destruction and a debug removal alike), each refund with a
  `SimEvent::ConstructionCancelled` (the constructor, the building and the refund) for the interface's feedback. Each
  tick `construction::step` advances a construction by its constructor's `PowerSummary::speed`: the Deployment Center, a
  producer, at full speed whatever the power, the Extraction Facility at the ratio (design change 0089); at
  `frames × RATIO_ONE` it is ready (`SimEvent::ConstructionCompleted`), by *Recipe timing*: a Generator ordered in tick
  `t` is ready at the end of tick `t + 239`, a plate at 30/51 at the end of `t + 679`, and a Place can apply from the
  next tick on. Place inserts the ready building where `Sim::placement_check` allows it as the command applies (plan
  decision T4): it counts for power and the build area at once, the true grids take it in phase 5b of its tick and its
  owner's planning grids at the end of it, and it works from the next tick on. `construction::recipes(kind)`,
  `Constructor::{current, ready, frames_done}` and `Sim::validate` are what a client needs for its menus. A Construct
  checks the building's prerequisites as it starts (slice M4-K, *Tech prerequisites*). Not built: ConstructionHP.
  (M6-GR) The Defense System constructs its Missile Turrets as the Extraction Facility its plate
  (`StructureDef::constructs`; `factions/gdo/structures/defense_system.md`, design change 0392): one at a time, 100
  Space Crystals and 240 frames each, at the ratio as a consumer, placed in the GDO build area. Tests:
  `crates/sim/tests/defense_system/` (`building.rs`, `flow.rs`, `turret.rs`).
- **Warp In** (`transform.rs`, `sim/transform.rs`; slice M3-E, plan decisions T15, T16 and T22;
  `factions/colonists/economy.md > Warp In`, design changes 0109 to 0122): a Beacon transforms in place into a
  building of its Constructs List (`StructureDef::warp_in`, `transform::warp_in_recipe`).
  `ConstructionCommand::WarpIn { kind, orientation }` names one Beacon, which pays the cost at once and becomes the
  building through `Sim::change_kind`, on the footprint `transform::warp_in_footprint` derives (see *Placement and
  the build area > Methods*). The object is the building from its first frame, its whole footprint solid (0109),
  and `Object::transform` (`Transform`: what it was and where, its progress of `total` frames, what it paid) runs
  until it is complete. `Sim::step_transforms` advances it by `RATIO_ONE` a tick (the Colonists have no power), by
  *Recipe timing*: ordered in tick `t`, a WarpInTime of `N` frames completes at the end of `t + N − 1`
  (`SimEvent::TransformCompleted`) and the building works from `t + N`. Its maxima interpolate
  (`Object::max_hp`, `Object::max_shields`: "Beacon MaxHP + (Building MaxHP − Beacon MaxHP) × progress"), and its
  HP and Shields rise by what their maxima rise (`transform::follow_max`, 0110: damage stays taken, whenever it was
  taken; the Shields since slice M3-B, which regenerate up to the maximum now).
  Until it is complete it holds its functions back (0116: no production, drop-off or abilities; the gate
  refuses a Train and a drop-off at it; a portal takes a rally point meanwhile since slice M5-S6, design change 0396
  over 0116's "no rally point": *Production > Rally points*), but it counts for defeat (0113) and keeps providing its 5 Beacon
  Capacity (0112). `ConstructionCommand::CancelTransform` turns every transforming object named back into what it
  was, on its old tiles, with the fraction of its HP it had and at least 1, and the fraction of its Shields with no
  minimum (`transform::keep_fraction`, 0111), and
  refunds `Refund::Ready` of what it paid (`SimEvent::TransformCancelled`); a destroyed one refunds nothing (0115).
- **Beacon Deployment** (`deploy.rs`, `orders/build.rs`, `sim/deploy.rs`, `sim/builds.rs`; slice M3-F, plan decisions
  T15 to T17;
  `factions/colonists/economy.md > Beacon Deployment`, design changes 0109 to 0120 and 0125): a Drone walks to a site
  and transforms in place into a Beacon, by the Warp In's machinery. `OrderKind::Deploy { origin }` (the Beacon's
  2 × 2 footprint, `deploy::site`, never turned; Shift queues it) goes to one Drone; the gate checks its site on what
  the player knows (see *Placement and the build area > Methods*). In phase 5a the Drone heads for the footprint's
  centre (`orders::build`, the walk a Build shares); its walk is over once its move ended, or when it found no way
  forward for `Movement::jammed_ticks`
  while overlapping the footprint (0117). Then `Sim::step_deployments` (see *Tick pipeline*) checks the site on the
  truth: if the Drone overlaps the footprint (`deploy::on_site`) and it passes, the Drone becomes the Beacon on the
  footprint, wherever it stood, the same object (0109: its footprint claimed in 5b of that tick), transforming `from`
  the Drone without `from_placement` (`deploy::BEACON_DEPLOYMENT`: 48 frames, no cost), its first frame that tick;
  else the order fails, the Drone idles and its next queued order starts (`SimEvent::DeployFailed`, its problems
  empty if the Drone stopped short). Its HP interpolate from 20 to 120, and its Shields, kept from the Drone, follow
  their maximum (10 to 10), as a Warp In's do; only Cancel Deployment (`CancelTransform`, 0116) applies, and turns it
  back into the Drone at the footprint's centre, with no order and the fractions of its HP and Shields (0111). It
  frees the Drone's 1 Beacon Capacity as it starts and provides the Beacon's 5 only at completion
  (`Object::is_deploying`, `economy::army`; a cancel takes the 1 back even over the cap, 0112). It counts for defeat
  (0113). The Drone's load stays in `Transform::load`, is lost at completion and returns with a cancel (0114).
- **Agent-built Tunnels** (`orders/build.rs`, `sim/builds.rs`, `sim/sites.rs`, `construction/site.rs`; slice M4-E,
  plan decisions T31 and T51; `factions/syndicate/units/agent.md > Building`, design changes 0233 and 0234):
  `OrderKind::Build { kind, origin, orientation }` is the kind-generic worker build (T51). It goes to the one unit
  named that builds `kind` (`WorkerDef::builds`, `construction::site::builds`), which walks to its site as a Drone
  does, and on arrival starts a site there (*Object model > Sites*) or fails. **The price**
  (`factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`; `SiteDef::price`, `SitePrice::SuppliesPerOwned`)
  is `Sim::build_price(player, kind)`: a Supply for each structure of the kind the player has, sites included, and
  for each Build of the kind among their units' orders (the lingering ghosts). It is paid as the order is given
  (`Sim::give_order`, `pay_build`); the gate refuses what the bank can't pay (`Reject::InsufficientSupplies`), counting
  what the orders it replaces give back. **Refunds** are the price once the order has gone, "the current top price",
  "not counting itself" (design change 0284): for an order replaced (`Sim::drop_orders`), gone with its worker
  off the map (`take_off_map`) or out of the game (`remove_objects`; design change 0289 point 1), or failed on
  arrival (`SimEvent::BuildRefunded`), and for a site cancelled (`Sim::cancel_site`, `SimEvent::SiteCancelled`); a
  destroyed site gives nothing back, nor do a defeated player's orders. So **the books** hold: the Supplies spent less
  those refunded equal the formula's total for what the player has, whatever the order of starts and cancels. A site
  completes after `SiteDef::frames` (480), its worker going where `SiteDef::builder_goes` says (into the network);
  cancelled or destroyed, it lets its worker out at the footprint's centre, idle, with its load
  (`Sim::release_builders`, on the removal path).
- **No Agent-built Laser Turrets** (design change 0359, slice M5-Y): an Agent builds Tunnels alone (its
  `WorkerDef::builds`), and the gate refuses its Build of anything else (`Reject::NoWorkers`). Slice M4-S1's turret of
  an Agent's (plan decision T51; design changes 0296 to 0298: its price in Space Crystals at the order, its Agent out
  beside it, no operation slot, its visible site) went with it; a turret is the Tunnel's ExpandMenu's alone (*Object
  model > The pop-up form*). What it brought stays where another builder uses it: the price in the def's resource
  (`construction::site::BuildPrice { resource, amount }`, `SitePrice::Crystals`), a Cults building's, refunded whole by
  an order that goes and 75% of it rounded down by a cancelled site (`BuildPrice::cancelled`, `Refund::Expansion`),
  the gate's `Reject::InsufficientCrystals`, and a worker's Build checked for its TechPrerequisite at the order (*Object
  model > Tech prerequisites*).
- **Cults construction** (`sim/builds/shared.rs`, `crate::spot_work`, `sim/spot_work.rs`, `orders/assist.rs`,
  `sim/sites.rs`, `placement/wall.rs`; slice M5-E, plan decisions T59 and T62; `factions/cults/economy.md > Cults
  Building Mechanics`, design changes 0071, 0072, 0339 and 0340): Recruits build together, from outside. The Recruit's `WorkerDef::builds` (Hall,
  Storage, Armory, and since slice M5-G2 the Gunworks, the Machine Shop and the Scrap Hangar,
  `tests/cults_construction/tech.rs`) and each structure's `SiteDef` (price, frames, `BuilderGoes::Outside`) are data,
  so a further Cults building is a def alone: every Cults structure has its site since slice M5-G2, and its slice adds
  it to `builds`; `WorkerDef::shares_builds` is what makes a worker's Builds shared.
  - **The ticket** (T59, `Ticket`): a Build given to several such workers is one order they each hold, current or
    queued, by its owner, kind, origin and orientation, derived from the orders: one ghost, one structure for the
    price, paid once as it is given (`Sim::give_shared_build`), refunded once when its last holder drops it before its
    site starts (re-ordered, killed, failed together; design change 0340), never after. The gate
    (`Sim::check_shared_build`) takes the whole group, no `Reject::SeveralConstructors` (it stays for Drones and
    Agents), and checks the site with the group left out (`Sim::group_build_check`).
  - **The site** starts at the first holder whose walk ends within `Construction::build_reach_su` of the footprint
    (`Sim::arrive_together`, in the arrivals' step), on the truth check (`Sim::arrival_check_without`), checked twice
    (slice M5-J1, design change 0362): with nobody left out it starts; else with the ticket's holders left out, any
    other unit on it, of any player, or its ground fails the whole ticket, refunded once, and nothing retries (0339);
    else only its own Recruits stand on it, current or queued holders (design change 0372, answering Q0131: "A Recruit
    holding this Build queued ... counts as one of the order's own Recruits"), and it waits, checked again each tick,
    as they walk off to their spots (a walk ended on the footprint looks again, `Sim::step_spots`, so none stands
    still there).
    It starts at 10% HP; every holder's Build becomes `OrderKind::Assist(site)`, queued ones too. It gains a frame a
    tick for each unit on an Assist of it within reach with its walk over (`spot_work::counts`; 0339's Recruit-frames:
    those walking or waiting add nothing), so n Recruits build n times as fast and none stall it, its progress and HP
    kept; complete, `SimEvent::SiteBuilt`, its builders' Assists end and they go on to their queued orders or idle
    (0340). X refunds 75% (`Sim::cancel_site`).
  - **The spots** (T62, 0339, `crate::spot_work`): each worker claims its own spot as it gets the order, in its
    order's state: the part of the builders' wall nearest it (`placement::Wall`, its box `Construction::build_standoff_su` off the
    footprint), or else the nearest free part along the wall either way, clockwise first, a room (its box and the
    standoff) from the others' (`Claim::keeps`); with the wall full, it waits on a row from the third out (row 1 kept
    free, the way round) and doesn't count. It walks round the footprint and the others' rooms: along the row outside
    its spot's, by that row's corners, and in from there (`Sim::leg`); walks press on (`Mover::press_on`). The count
    of those at work and the claim are written over an order and its target, and slice M5-N8's Repair reuses both
    (*Repair* below).
  - **Assist** (`OrderKind::Assist(site)`, `orders/assist.rs`): of an own site built from outside, by its builders
    (`Reject::NotASite`, `Reject::NoWorkers`); the right-click on such a site (`orders::resolution`).
- **Repair** (`crate::repair`, `orders/repair.rs`, `sim/repairs.rs`, `tuning/repair.rs`; slice M5-N8, plan decision
  T83; `factions/cults/units/recruit.md > Repair`, `factions/syndicate/units/agent.md > Repair`, design changes 0321,
  0322, 0325, 0328, 0329 and 0360): one mechanism for every repairer, its rules in `crate::repair`'s docs.
  - **Who repairs what** is data, `UnitDef::repairs` (`RepairDef`, `None` for a type that repairs nothing; a unit's,
    not a worker's, since slice M5-N9: the Field Engineer repairs but gathers nothing): the Recruit and the Field
    Engineer `RepairTargets::OwnBuildingsAndVehicles`, paid; the Agent `RepairTargets::Kinds`, the Tunnel free and
    the Laser Turret paid (`RepairPayment`); the Drone and every other type nothing. A vehicle
    (`repair::is_vehicle`) is a unit of neither infantry base that isn't a glider, by its movement; a building is a
    structure but for the underground expansions, which no one repairs (they regenerate, below).
    `repair::payment(repairer, target)` is the one rule, `repair::repairs` its answer.
  - **The terms** (`repair::terms`): from what built the kind, its site (`SiteDef`: a Cults building's; the Tunnel's
    480 frames at every tier, 0328), its expansion's build (`ExpansionDef`: the Laser Turret's) or else the first
    `Recipe` that makes it in the registry's order (a vehicle's Space Crystals only), shared out by
    `Repair::repair_time_share` and `Repair::repair_cost_share` (half and half, placeholders): MaxHP / frames a tick per repairer. A
    repairer that pays pays Space Crystals, so a price in another resource (the Tunnel's Supplies) is repaired for free
    or not at all (placeholder).
  - **The target** (`repair::can_repair`, `Sim::can_repair`): another object of the repairer's owner's, complete (no
    site, nothing transforming) and below its MaxHP (`repair::damage`, repair's one read of a MaxHP: the one with its
    Augmentations, `Object::max_hp_with`, slice M5-N1a), of a kind its type repairs. The gate (`Sim::check_repair`)
    keeps the units that can, else `Reject::CantRepair(target)`; `Sim::aimed` gives the kind as `Aimed::repair` for
    the right-click (*Commands, events and replays > Default orders*).
  - **The order** (`OrderKind::Repair(target)`, `RepairWork`): at a building the repairer works from its own spot beside
    the footprint (`RepairWork::Spot`: the spots' step, as an Assist's, *Cults construction*), a Tunnel and a lowered
    Laser Turret alike, which nothing of a repair raises; at a vehicle it follows it (`RepairWork::Follow`,
    `travel::approach` within `Orders::proximity_su`, inside `Construction::build_reach_su`).
  - **The step** (`Sim::step_repairs`, *Tick pipeline*): each unit whose current Repair's target it can still repair,
    within reach of its edge with its walk over (`spot_work::at_work`: a footprint, or a vehicle's Silhouette),
    restores its share, so that n repairers give n times the rate; the free ones first, then those that pay, in whole
    crystals as the HP enters each step of the price (`repair::step_of`, ceil(HP × price / MaxHP), and `top_of_step`),
    with no state of its own, a bank short of one stopping the HP at the top of the last step paid. The Repairs end
    (`Sim::end_work`, the queued orders going on): all of a target's at full HP, all its paid ones when the bank runs
    out (those on their way too, placeholder), any whose target no longer suits it (not its owner's: a neutral turret,
    0360; at full HP; no longer complete), and on the removal path any whose target left the game.
  - **Measured** (`tests/repairs_perf.rs`, ignored, release): 40 Recruits repairing 10 Armories from 1 HP, 12 rounds,
    a mean of 0.025 ms and a p99 of 0.09 ms a whole tick, the economy phase's 0.003 ms (load average 1.4 to 4.4).
    `structures_perf` (no repair in it) pays the repairs' and the regeneration's walks of every object: 3 µs more in
    the economy phase, 0.355 to 0.360 ms a whole tick (+1.3%, 8 interleaved rounds against slice M5-U1's build).
- **Expansions' regeneration** (`Sim::regenerate_expansions` in `sim/expansions.rs`, `crate::regeneration`,
  `tuning::tunnel::Tunnel::expansion_regen_per_second`; slice M5-N8; `factions/syndicate/structures/README.md > Tunnel
  Expansions`, design changes 0329 and 0330): "Not repaired; they regenerate". Each complete underground expansion of
  its owner's with a parent Tunnel (`tunnel::area::parent`: one of its owner's in operation whose area contains it)
  regains 1% of its MaxHP a second (placeholder), always on, under fire too; a neutral one decays instead (*Object
  model > Losing Tunnels*); the Laser Turret, which isn't underground, regenerates nothing (Agents repair it,
  placeholder). `regeneration::regenerate` is the one rule of HP regained as a share of MaxHP a second, the network's
  held units' too (`network::Held::regenerate`), up to the MaxHP with a unit's Augmentations (`Object::max_hp_with`).
- **Adding to the economy** (slices M2-D2, M2-F): put the new step into `economy::step` at its numbered place, advance
  progress by `PowerSummary::speed` from the tick the command applies (*Recipe timing*), take costs with
  `spend_crystals` when the command applies, and give refunds through `Refund`; `construction.rs` is the pattern.
- **Defeat and the end of the game** (`economy/defeat.rs`, slice M2-F2, plan decision T8; `core/game_rules.md >
  Defeat Condition`, `core/game_rules.md > On Defeat`, `core/game_rules.md > End of Game`; design changes 0100 to 0102,
  and 0163 and 0164, slice M3-V):
  - **Defeat**: a player is defeated in the tick their count of owned structures drops to 0 (every structure
    counts, plates included): `defeat::step` compares the owners of structures as the tick started with those as it
    ends, so a player who never owned a structure (a sandbox's) is never defeated, and nobody is once the game is
    over. `Player::defeated_at` (hashed, `#[serde(default)]`), `SimEvent::PlayerDefeated`.
  - **On defeat** (0101): the player's remaining objects, units only since they own no structure, become neutral
    (`owner: None`): orders cleared, attack disengaged, mover stopped, and `Mover::repath` (see *Navigation > Path
    knowledge > No owner*). Neutral, they give no vision, never auto-target and are never auto-targeted, never
    yield, and any player may destroy them with an explicit Attack. A raised Riot Shield stays raised, and an
    attack already Firing or in Cooldown runs its course. No producer is ever handed over: a defeated player owns
    none. The gate refuses every command of a defeated player (`Reject::Defeated`), and a debug spawn of an object
    for them, so a defeated player owns nothing for good. Their units in a Tunnel Network are destroyed instead
    (design change 0258 point 5, `Sim::clear_defeated_networks`). A Syndicate player's expansions that a lost Tunnel
    left neutral no longer count, so losing the last Tunnel defeats a player with no other structure (slice M4-H,
    *Object model > Losing Tunnels*; whose stranded-unit rule takes the network units first, in the removal).
  - **Sides**: the sim has no team or alliance data yet (fixed teams are deferred, `core/map_editor.md > Map Types >
    Custom`), so every player is their own side (`defeat::allied`) and `Sim::allies` is empty. Teams go there.
  - **The end**: the defeat that leaves at most one side standing ends the game in its tick: `Sim::outcome()` is
    `Outcome { tick, winners }` (hashed, serialized, `#[serde(default)]`), after the tick's `PlayerDefeated`s comes
    `SimEvent::GameOver { winners }`. The winners are the side left standing, its defeated members included.
    When the last sides fall in the same tick nobody wins: the game is a **draw** (0163, `Outcome::is_draw`, the
    outcome without winners; derived, so the state and its files are as they were), and every player sees Draw,
    those defeated before it too. From then on the gate refuses every command (`Reject::GameOver`). The sim would
    step on if asked; the client stops it (see *Client architecture > Sim driver*).
  - **Leaving for the results screen** (0164) is the client's (`hud/results.rs`, see *Client architecture > HUD*):
    the sim runs on as before, and a player whose side fell while the game goes on is simply no longer shown it.
