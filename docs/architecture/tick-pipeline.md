# Tick pipeline

Read this when you add or reorder a step of `Sim::step`, or need the order in which the tick runs its phases.

## Tick pipeline

`Sim::step(&TickInputs)` advances exactly one tick. It runs fixed phases in this order (`sim/phase.rs`):

1. **Clear events**, so that `Sim::events()` holds only this tick's events. (M5-X1b) Then **the tick's rules**
   (`sim/rules.rs`): those of the last tick's `DebugCommand::SetRules`, the objects following them, or the game's own,
   become the current ones, so that the whole tick reads one generation (*Tunables > Generations*).
2. **Advance the tick** (`tick += 1`).
3. **Snapshot** `pos`/`facing` into `prev_pos`/`prev_facing` for every object (the client interpolates from these),
   and note who owns a structure as the tick starts (`economy::defeat::structure_owners`, for the defeat check).
4. **Commands**: apply each `PlayerCommand` in `TickInputs` order (`apply_command`). Each passes the gate against
   the state the commands before it left, so a cost is paid from the bank as they left it and a Place is checked
   with the footprints they placed (see *Economy > Construction*). (M5-N5) A `Command::Cast` pays its energy and gives
   its buffs here, to the units where the last tick left them (plan decision T76; *Combat > Buffs*); (M5-N11) a
   Prayer-mobile's Inspire and Suppress too, round it and ahead of it as the last tick left it (*Combat > Inspire and
   Suppress*).
5. **Systems**, each walking the objects in id order, one function per module (`docs/movement.md > 4`):
   - Modes, `Sim::step_modes` (`sim/modes.rs`, beside `Sim::use_ability`, which applies the ability commands and
     switches a toggled unit's type at the press): each unit's form goes a step toward its type (*Object model >
     Type and form*). A Riot Shield on its way up or down goes on one tick (`riot_shield::step`), and a Weapon
     Conversion one frame, or, its frames done, gives the unit its other form, and converts it back at once if its
     type changed meanwhile (`conversion::step`; see *Combat > Weapon Conversion*); a raise of the Riot Shield or a
     conversion that waits for the unit's shot begins in the first Modes step after its Cooldown (design changes 0172
     and 0153). After the commands, so that a toggle or a Convert counts from its own tick; before 5a, so that the
     behaviours, the movement and the combat of the tick all see the form it leaves.
   - Vehicle deploys, `Sim::step_vehicle_deploys` (slice M6-GA, `sim/vehicle_deploy.rs`, `vehicle_deploy.rs`; plan
     decision T44: "Progress and completion in a Sim step right after `step_modes`, before 5a"): for the units before
     the tick's first new id, each whose current order is a vehicle deploy under way takes one frame of it; one whose
     frames are done becomes its pair's other type (`Sim::change_kind`, a `SimEvent::KindChanged`), its deploy order
     ends, and each order of its queue its new type can't carry out is dropped (`vehicle_deploy::can_take`, the gate's
     rule; `SimEvent::OrderDropped`). Before 5a, so that the unit acts as its new type from that tick's first phase,
     its held order first; a deploy that 5a starts takes its first frame at the next tick's step (*Object model >
     Vehicle deploy*). It passes over every object each tick, a check of its current order.
   - 5a `orders::update`: a unit's finished order completes and its next one starts, until one is in progress, so
     legs follow each other without an idle tick; then that order's behaviour runs for the tick, or the idle one
     (see *Behaviours*), and a turret's own after it (slice M5-U1). They write goals (`Mover::go_to`, `stop`), the
     target the attack engages and a turret's LockedTarget (*Combat > Units' turrets*). Everything
     they read of other objects comes from one snapshot taken first (`orders::world::World`): the positions and the
     vision the last tick left.
   - Arrivals, `Sim::step_deployments` (slices M3-F and M4-E, `sim/builds.rs`; see *Economy > Beacon Deployment* and
     *Economy > Agent-built Tunnels*): part of 5a, right after `orders::update`, in id order. Each worker whose walk
     to its site ended this tick deploys there (a Drone: a change of kind) or starts its site there (an Agent: a new
     object, which it leaves the map for, `Sim::start_site`), before 5b, which claims the footprint; or its order fails
     (`SimEvent::DeployFailed`, `SimEvent::BuildFailed`, a Build refunded): the changes of kind, the new objects and
     the events that a behaviour may not make.
   - Spots, `Sim::step_spots` (slice M5-E, `sim/spot_work.rs`; *Economy > Cults construction*): part of 5a, right
     after the deployments, in id order. Each unit whose current order works at a footprint from a spot beside it (a
     group's Build, an Assist, a building's Repair) and has no spot gets one and walks there, round the footprint, a
     unit on its way heading on at each corner (`Sim::head_on`); an order on a structure that no longer suits it ends;
     every `Construction::respot_ticks`, a unit whose walk is over keeps where it stands within reach, or looks again along the
     builders' wall, and one whose walk ended on the footprint itself looks again as far out as needed (slice M5-J1: its
     site waits for it, design change 0362). Before 5b, so that its walks get their paths in the tick.
   - Entries, `Sim::step_entries` (slice M4-C, `sim/network.rs`; `factions/syndicate/tunnels.md > Tunnel Network >
     Walking in`): part of 5a, right after the deployments, in id order. Each unit whose current order is an Enter
     and whose box is within `Tunnel::entry_reach_su` of its Tunnel's entry spot leaves the map for the network
     (`Sim::take_off_map`, `SimEvent::EnteredNetwork`), on its way or at the end of its walk alike: no hold between
     units (design change 0246). An Enter whose Tunnel is gone or no longer suits the unit, or whose walk ended short
     of the path (the normal Failure rule, 0247), ends (`SimEvent::EnterFailed`). Into a building that holds units
     (slice M5-F1, *Object model > Buildings that hold units*) the same, at its entry spot, if its pool has room then
     (`SimEvent::EnteredBuilding`); with none the Enter ends there. (M6-ST1) Onto a carrier on the map (*Object model >
     Carriers*) the same, within Proximity of its shape from any side (`Orders::proximity_su`), if it has room then
     (`Sim::board`, `SimEvent::Boarded`); with none the Enter ends there (`EnterFailed` with `full`). Before 5b to 5f,
     so that no grid, collision, shot or sight of the tick includes a unit that has gone in.
   - Landings, `Sim::step_landings` (slice M6-GH2, `sim/landing.rs`, `landing.rs`; *Production > The Airfield*): part
     of 5a, right after the entries, Airfield by Airfield in id order. Each aircraft flying home to an Airfield (its
     Return to Base, or a Rebase) flies on to the slot it takes (`landing::claims`, derived every tick) and, come to
     rest over it, lands: its mover stopped, its `Landing` begun. Each landing under way takes a frame, down once its
     16 are done. After 5a, which starts the flights and lifts the aircraft off (an order's start ends its landing,
     `landing::take_off`); before 5b to 5f, so that the tick's movement flies each to its slot, and its combat sees
     each as it is left (an air target while landing, a ground one down: `combat::object_domain`).
   - Supply runs (slice M6-GS1, `sim/supply_run.rs`; *Production > The Supply Tower*): the schedules,
     `Sim::step_schedules`, right before 5a, so that a run they start is under way in the tick: per station and
     player, the closest Tower whose attached chopper is ready (idle, down on it) sends it, a Pick Up Supplies there
     and a Drop Off Supplies home, once the station's next delivery is due within the flight's estimate. The flights,
     `Sim::step_supply_runs`, right after the landings: a chopper whose Attach to Tower runs is attached (any other
     chopper landing or down there evicted); each landing under way takes a frame; each chopper flying to a spot flies
     on to it and, come to rest over it, lands, one per spot (the Tower's attached chopper first), a scheduled one once
     the delivery is due by its touch-down.
   - Attacks on a target too close, `Sim::step_too_close` (slice M3-L2, design change 0182): part of 5a, right after
     the deployments, in id order. Each unit that can't move, or can't back off further (design change 0204 point
     3), whose Attack order 5a found with its target inside its MinRange (`AttackOrder::too_close`) ends the order,
     and its next queued one starts in the next tick, with `SimEvent::TargetTooClose`; and so does each whose Attack
     Ground 5a found with its location inside (`AttackGround::too_close`, slice M5-O3), with
     `SimEvent::LocationTooClose`.
   - Launches, `Sim::step_launches` (slice M6-GD, `sim/drones.rs`; *Object model > The Drone Operator*): part of 5a,
     right after the attacks on a target too close, in id order. Each operator whose Launch 5a started takes a drone
     out of its stock and spawns it over itself, tethered (`SimEvent::DroneLaunched`), and the Launch ends. Before
     5b, so that the drone has its grids from its first tick; a new object, it moves and attacks from the next.
   - Clouds, `Clouds::update` (slice M5-N4b, `shroud.rs`; *Combat > Shroud*): the end of 5a, after the attacks on a
     target too close. The clouds follow the channels as 5a leaves them (`orders::update` returns the units that
     channel): a channel started in the tick protects in its 5e, and a cloud that an ended channel, a split or a join
     changed releases its buffer there.
   - 5b `paths::sync`: if the blocking footprints changed, rebuild the true navigation grids and bump
     `nav_version`. Then `Sim::cover_grids` builds the grids of a class whose first unit came into the game, or into
     a producer's queue, since the last tick (*Navigation > Which grids*).
   - 5c `paths::serve`: first each unit's staggered path checks, in the ticks where `(tick + id) % 4 == 0` (after
     its owner's path knowledge changed, line of sight along its remaining path, and a new plan only if that is
     cut; the farthest waypoint in sight becomes its steering target). Then every `PathState::Wanted` becomes a
     `nav::PathRequest`, served by `nav::serve_on` within `Paths::path_budget` (group fields for move groups). A unit that
     gets no path fails its move (`SimEvent::MoveFailed`). Each unit checks and plans on its owner's known grids,
     as of the end of the last tick (see *Navigation > Path knowledge*).
   - 5d `movement::step`: steering and hard collision, the only code that writes a unit's `pos` besides a teleport
     (see Movement and collision). Units move one after another in id order and never come to overlap, gathering
     Drones apart (design change 0106). (M5-V4) A crusher's move passes through its crushable enemies and crushes
     those it overlaps or sweeps: they leave the game at the end of 5d, before 5e, through the removal path, each with
     a `SimEvent::Crushed` (see *Movement and collision > Crushing*). (M6-SB) Before the move, the sludge's slow
     (`patches::slow`): a unit whose centre the last tick left in a sludge patch holds `BuffKind::Sludge`, its top
     speeds × 0.6 for this move (*Object model > The Vault and the Sludge*).
   - Tethers, `Sim::step_tethers` (slice M6-GD, `sim/drones.rs`, `tether.rs`): right after 5d, for the drones before
     the tick's new objects, in id order. A drone whose end came (its operator gone, past its leash where 5d left it,
     its battery spent) leaves the game through the removal path with `SimEvent::DroneFell` and `Died`, and one past
     its leash drops its explosion where it fell. Before 5e, so that a fallen drone attacks nothing in the tick and
     its explosion lands in it.
   - Patches, `patches::lay` (slice M6-SB): right after the tethers, the patches whose time is over go; then each
     unit with a toggle on, in id order, lays its trail where 5d left it (a new patch, or its last again) and drains
     its tank, its toggle off once the tank is empty. Before 5f, whose vision the client shows them by.
   - 5e `combat::step`: attack phases (a unit's turret turning first) and launches, then those of the units that
     fire from inside a building (slice M5-N2, `combat::occupants`), the projectiles due landing in launch order, the
     tick's list of
     hits applied together (the blasts' first; slice M5-N4b: less the hits a cloud absorbs, plus the releases due,
     *Combat > Shroud*), deaths (see *Combat*). The dead
     leave through `Sim::remove_objects`, then `paths::sync` runs again in case a footprint went with them.
   - Returns, `Sim::send_home` (slice M6-GH1, `sim/aircraft.rs`; *Production > The Airfield*): right after 5e's
     removals. Each aircraft whose last round went in 5e (`combat::step`'s `emptied`, in the order they fired) and that
     still has a base gets a Return to Base in place of its orders. After the removals, so that one killed in the tick
     gets none, nor one whose Airfield fell in it (the removal path unassigned it).
   - Pop-ups, `Sim::step_popups` (slice M4-S2, `sim/popups.rs`; *Object model > The pop-up form*): right after 5e
     and its removals. Each turret in operation from before the tick goes one tick through its cycle, in id order: by
     whether it is active now (an inactive one is lowered at once; a Tunnel lost in the tick left its turrets neutral
     and lowered already, on the removal path), the target 5e left it, and, to start rising, no ground unit's box over
     its footprint, where 5d left the units. A turret that starts to rise or is fully lowered changes what blocks:
     `paths::sync` runs again. Before 5f, so that a turret is seen from the tick it starts to rise and hidden from the
     tick it is down.
   - Shields, `shields::step` (slice M3-B, see *Combat > Shields*): every object's Shields regenerate
     `Shields::shield_regen_per_tick`, up to its MaxShields now (`Object::max_shields`). Right after 5e's removals, so that the
     dead regenerate nothing and a hit's tick ends with the regeneration on top of it; before the economy, so that a
     transforming object regenerates up to the maximum its tick began with, which `Sim::step_transforms` then raises,
     its Shields with it.
   - Energy, `energy::step` (slice M5-N4a, plan decision T75, see *Combat > Energy*): right after the Shields, as T75
     has it. Every object's energy, on the map and then held, in id order, regenerates its type's rate a tick, up to
     its MaxEnergy: after 5e's removals, so that the dead regenerate nothing; before the economy, so that a unit it
     trains has its type's start at the end of its first tick. An object created in the tick (`first_new` on)
     regenerates from the next; a unit trained as its building is destroyed (the evacuation, in 5e) gets its pool
     before this step, and ends its tick a tick's regeneration above its start. (M5-N5) An Energy regeneration buff
     adds its rate here (`buffs::energy_bonus`), from the tick of its cast.
   - Buffs, `buffs::step` (slice M5-N5, plan decision T75, see *Combat > Buffs*): right after the energy. Every
     object's buffs, on the map and then held, in id order: a Health regeneration adds its HP, up to the unit's MaxHP
     with its Augmentations, and each buff whose last tick this was ends. After 5e's removals, so that the dead never
     heal and a hit's tick ends with the heal on top of it; an object created in the tick (`first_new` on) has its
     effects from the next. (M5-N11) Inspire and Suppress do nothing here but end: what they change is read where it
     counts, the speed by the movement step and the attack cycle by phase 5e, and Suppress by phase 5e's start of an
     attack (`suppress::holds_fire`), in every tick of the buff, its cast's included (*Combat > Inspire and
     Suppress*); Suppress has no step of its own.
   - Economy, `economy::step` (plan decision T5, see *Economy*): after 5e's removals and their `paths::sync`, so
     that a building destroyed in the tick counts for nothing in it; before 5f, so that what it brings into the game
     is seen in its own tick. In order: every player's power (the structures this tick's commands placed included),
     the constructors (`construction::step`; when a recipe's frames count: *Economy > Recipe timing*), (slice M6-GR)
     the casters' cooldowns (`cooldown::step`: each by its owner's `PowerSummary::speed`, the cast's tick the first,
     dropped once done; *Combat > Scan and cooldowns*), (slice M6-GH2) the aircraft down on an Airfield's slot, which
     rearm a round every RearmTimePerAmmo by its owner's `PowerSummary::speed` (`landing::rearm`, design change 0423;
     the touchdown's tick the first), the plates' mining, the workers' timed work
     (`gathering::step`, slice M3-H: mining takes its load from the patch, a drop-off pays the bank), (slice M6-GS1)
     the Supply Choppers down on a spot (`economy/supply_run.rs`: a pick-up takes all the station holds that no
     pick-up has claimed, a drop-off pays it in, and a chopper down on its own Tower is repaired free by its owner's
     `PowerSummary::speed`).
     `economy::step` returns the patches the workers emptied, and `step` takes them out through
     `Sim::remove_objects` and syncs the grids again, as after 5e's deaths.
   - Expansions' regeneration, `Sim::regenerate_expansions` (slice M5-N8, `sim/expansions.rs`; *Economy >
     Expansions' regeneration*): part of the economy phase, right after `economy::step` and its removals, before the
     transformations, the sites, the upgrades and the decay, so that an expansion completed or reclaimed in the tick
     regenerates from the next one, keeping in this one the damage it was built with or that its decay left it.
   - Transformations, `Sim::step_transforms` (slice M3-E, see *Economy > Warp In*): part of the economy phase, run
     right after the expansions' regeneration, before the producers: each transformation advances one frame,
     its interpolated maxima and its HP follow, and a complete one turns its functions on, so a portal takes a Train
     from the next tick's commands. It changes no footprint: a Warp In and a cancel change theirs in the commands
     phase, before 5b.
   - Sites, `Sim::step_sites` (slice M4-E, `sim/sites.rs`): part of the economy phase, right after the
     transformations and for the same reason. Each site from before the tick does a frame, its HP following its
     maximum, and a complete one works from the next tick's commands, its worker in the network for the network's
     step. A site is a new object, so by the spawn-tick rule one started in tick `t` does its first frame in `t + 1`
     and completes at the end of `t + frames`. A Tunnel that completes reclaims the neutral expansions it covers
     (`Sim::reclaim`, slice M4-H). (Slice M4-S1's Laser Turret's Agent, which came out beside the turret it
     finished, went with design change 0359.)
   - Repairs, `Sim::step_repairs` (slice M5-N8, `sim/repairs.rs`; *Economy > Repair*): right after the sites, whose
     builders its repairers are counted as (`spot_work::at_work`), and after 5e, so that the HP an object lost in the
     tick is what its repairers restore. Each object that repairers are at work on, in id order, regains its HP, its
     owner paying for it; the Repairs that are over end (`Sim::end_work`), their queued orders starting in the next
     tick's 5a. A site completed in the tick has no repairers yet.
   - Upgrades, `Sim::step_upgrades` (slice M4-G, `sim/upgrades.rs`): right after the repairs. Each Tunnel from before
     the tick that upgrades does a frame, in id order: the Tunnel exists already, so an upgrade ordered in tick `t`
     completes at the end of `t + frames - 1`, as a recipe; then it reclaims, before the decay.
   - Decay, `Sim::step_decay` (slice M4-H, `sim/tunnels.rs`): part of the economy phase, right after the sites, so
     that an expansion a Tunnel finished over in the tick is its owner's again first. Each neutral expansion that
     decays loses a tick of it, in id order; one whose HP reaches 0 dies (`SimEvent::Died`, owner `None`) and leaves
     through `Sim::remove_objects`, the grids synced again. An expansion its Tunnel left neutral in the tick (in the
     commands' removals or 5e's deaths) decays from that tick.
   - Producers, `Sim::step_production` (slice M2-D2, see *Production*): part of the economy phase, run right after
     `economy::step` and not inside it, because a finished unit is inserted into the game (`Sim::insert_object`).
     The Recruitment Halls first queue their Recruits by themselves (slice M5-C, `Sim::start_recruits`: their claims
     and lineages are read here, after 5e's deaths, so a Hall lost in the tick frees its tiles to the next Hall in
     it). A unit at the front starts only if its owner's army cap has room for it (design change 0143), the unit in
     production advances, and a finished one is released into its exit doorway, or held while that is occupied.
     Nothing in `economy::step` reads what the producers write, or the other way round, so the order between the
     two is free; after the transformations, so that a Beacon completed in the tick gives its room in it, as the
     deaths of 5e do theirs. After 5d, so a released unit never moves in its own tick; before 5f, so it is seen in
     it.
   - Stocks, `Sim::step_stocks` (slice M6-GD, `stock.rs`): part of the economy phase, right after the producers. Each
     unit's purchase under way advances a frame, whatever its owner's Power, and one whose frames are done adds its
     item to the stock (`SimEvent::StockBought`).
   - The Tunnel Network, `Sim::step_network` (slice M4-C, `sim/network.rs`): part of the economy phase, right after
     the producers. Every held unit regenerates; (slice M6-SB) the units with a tank that isn't full take a tick of
     their Vault turns, and (slice M6-SN) the manned units a tick of their Null Chamber turns
     (`network::services::step`, *The Null Chamber*, *The Vault and the Sludge*); then each Tunnel with an eject
     queue, in id order, lets
     its first unit out onto the exit spot if that is clear of tiles, footprints and units of its layer
     (`Sim::check_unit_fits`, the rule of every unit put on the map), else the queue waits: no other spot, no minimum
     gap (design change 0250), one unit a tick at most. The unit keeps its id (`Sim::put_on_map`,
     `SimEvent::LeftNetwork`; buffed if its turn is done, `Sim::leave_network`) and gets its order through
     `Sim::give_order`: its rally's, or a Move 1 grid unit straight out. After the producers, so that a unit finished
     in the tick with a surface rally comes out in it; the turns before the queues, so that a unit whose turn is done
     in a tick leaves buffed in it; after 5d, so that it never moves in the tick it appears; before 5f, which sees it.
     **Old ids come back** (plan decision T35): nothing after this step may take `first_new` for "everything that
     arrived this tick".
   - Buildings that hold units, `Sim::step_holding` (slice M5-F1, `sim/holding.rs`): part of the economy phase,
     right after the Tunnel Network. Each building with units ready to leave (`Held::leaving`: ejected, or the unit of
     a pool building's finished entry, slice M5-F2), in id order, lets the one ready first out into its exit doorway
     if the doorway is free (`Sim::let_out`: a trained unit as the unit it trains, `SimEvent::UnitProduced`; an
     ejected or released one as it is, `SimEvent::LeftBuilding`), toward the rally point: one a tick at most, and none
     while the last is still in the doorway. After the producers, so that an entry finished in the tick lets its unit
     out in it, and for the same reasons as theirs after 5d, before 5f.
   - Unloads, `Sim::step_unloads` (slice M6-ST1, `sim/holding/carriers.rs`; *Object model > Carriers*): part of the
     economy phase, right after the buildings that hold units. Each carrier from before the tick whose current order is
     an Unload All, in id order, once it is at rest (its momentum braked off since the order stopped it), ends the
     order and puts its passengers on the map round it (`Sim::unload`, `SimEvent::Unloaded`); its queued orders go on
     from the next tick. Here and not in the behaviours, which bring no object onto the map; after 5d, so that the
     carrier has come to rest in the tick and its passengers never move in the tick they appear; before 5f, which sees
     them.
   - Defeat, `economy::defeat::step` (slice M2-F2, plan decision T8, see *Economy > Defeat and the end of the
     game*): the economy phase's last step, run by `step` after the producers, so that every way a structure left
     the game in the tick counts (the commands' removals, 5e's deaths). Before 5f, so that the units a defeat makes
     neutral are nobody's eyes in its own tick.
   - Clouds again, `Clouds::settle` (slice M5-N4b): after the defeat and its networks, the clouds of the channels
     that ended after 5a, by a death in 5e or a defeat's cleared orders. Their buffers wait for the next tick's 5e
     (Placeholder: the design says "released immediately").
   - 5f `Vision::update`: every player's sight from the objects as the tick leaves them, those this tick's commands
     spawned included, then the fog delays and the reveals that end (see Vision), then (slice M6-GR) the scans: those
     cast in the tick stamped, those that end withdrawn (`Vision::step_scans`, *Vision > Scans*). Combat in 5e reads
     the vision of the tick before, and its hits reveal their attackers on top of it.
   - `update_supply_stations` (`core/map.md > SupplyDeliveryStation`).
   - `Vision::remember`: each player's records of the structures and resources in its sight take the tick's
     final state, the deliveries included (see Vision).
   - Rally points, `production::rally::follow`: on the vision the tick leaves, a rally point on an enemy its owner
     no longer sees becomes a ground rally where it was last seen (design change 0088, see *Production*).
   - Path knowledge, last (`PathKnowledge::observe`, `Sim::cover_grids`, `paths::sync_known`): each player stops
     assuming the map resources whose place it now sees, every unit the tick brought has its grids, and a player
     whose known footprints changed gets a new knowledge version and its planning grids rebuilt, at most once per
     player and tick. After everything it follows from: the objects, the vision and its records. Nothing after it
     may change any of them.

- **Spawn tick**: an object that a command creates during tick `t` is first updated by the systems in tick `t + 1`.
  `step` notes `first_new = ObjectId(next_id)` before the commands, and each system walks only
  `objects.range_mut(..first_new)` (ids only grow). So a debug-spawned station with an interval of 960 delivers at
  `t + 960`, just as a scenario station, spawned before tick 1, delivers at tick 960. An order to a unit spawned
  earlier acts in the tick it arrives: a straight move of `d` su ordered in tick `t` arrives in tick
  `t − 1 + ⌈d / speed⌉`.
- **Grids match the state between ticks**: the true grids match the objects (`step` checks it in debug builds),
  and each player's known grids match what it knows (invariant K1). A phase after 5b that adds or removes a
  blocking footprint (a death in combat, a new structure) must call `paths::sync` again; the known grids follow at
  the end of the tick.
- **Measuring a tick**: `Sim::step_observed(inputs, observe)` is `step` calling `observe` with each `Phase` as it
  starts and `Phase::End` when the tick is done (`sim/phase.rs`); `step` passes a no-op, and the sim never reads
  anything back. `TestSim::step_observed` wraps it for the perf tests. Measured (`tests/structures_perf.rs`, ignored,
  release; slice M2-J2: four GDO bases of 40 structures and 400 Peacekeepers fighting through two of them on 256 ×
  256): a whole tick takes 0.28 ms on average, 0.53 ms at the 99th percentile and at most 2.4 ms after the tick of
  the four players' 20 group orders at once, which takes 7.8 ms, nearly all path serving (each group order is pathed
  in its tick, `docs/movement.md > 5.8`). At worst the grid syncs (5b, 5e's, path knowledge) take 0.13 ms with up to
  3 sets rebuilt in a tick, vision 0.17 ms, the economy 0.11 ms and the defeat check's two walks 0.005 ms.
  Measured with Drones (`tests/drones_perf.rs`, ignored, release; slice M3-K2: two Colonists sides of 40 Drones
  gathering on two rows of 8 patches each, and 10 Legionnaires a side fighting between them, on 128 × 48 for 3
  minutes; three runs at load 1.1 to 1.2): a whole tick takes 0.10 ms on average, 0.12 to 0.21 ms at the 99th
  percentile and at most 0.21 ms after tick 1, whose 32 Gathers and two group orders take 0.5 to 0.9 ms; movement
  0.053 ms, vision 0.025 ms, the behaviours 0.013 ms. The waiting Drones' re-check ticks (every 16th) cost 0.003 ms
  more than the others. There is no M2 baseline to compare with, since M2 had no Drones.
- Insert a new phase at a deliberate position, with a comment in `step` saying why there, give it its `Phase`, and
  update this list.
- Commands always run before movement. There is exactly **one** movement pipeline, and all movement goes through it.
  The old prototype had two pipelines, and they raced.
- Damage accumulates as a **list of hits** that is applied in one place. Never store it in a single slot that a second
  hit can overwrite, because the old prototype lost hits that way.
- Lessons from the prototype, for M1 and on:
  - **Movement**: the arrival radius must be at least one tick of travel. Otherwise units overshoot and oscillate.
  - **Pathfinding**: cap repath attempts (`tuning::movement::Movement::max_repaths`). A* needs a correct closed set, a best-cost
    map and deterministic tie-breaking. Overwriting a parent at equal cost made zigzag paths.
  - **Occupancy** is domain-aware. For example, underground expansions don't block surface units.
