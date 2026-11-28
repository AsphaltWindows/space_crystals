# Movement and collision

Read this when you touch unit movement, steering, collision, jams or aircraft flight.

## Movement and collision

The design is `docs/movement.md > 6` (binding; its **(D)** marks are where slice D changed it, **(M)** where the
magic box did, **(Y)** where friendly yielding and the answers to Q0008 did, **(Q)** where queued group orders did, **(G)** where
the hardening of crowds and nav performance did, **(H2)** where gathering Drones passing through each other did,
**(M3-G)** where DragMovement and idle units moving apart did, **(M3-G2)** where a queueing unit going round
the row in its way did, **(M3-G3)** where one with no way round giving up did, **(H1)** where Gliders and the soft
separation of air units did, **(M5-H2)** where a Glider's Move completing on arrival and Gliders pushing apart slowly
did, **(V1)** where Silhouettes turning with the facing did, **(M5-V4)** where crushing did), and the code is
`crates/sim/src/movement.rs` and `movement/`, with the unit's state in
`mover.rs`.

- **One pipeline, sequential commit** (`movement::step`, phase 5d): the step builds one spatial grid per collision
  layer (ground, underground; air units collide with nothing) from every unit, then moves the units in id order and
  updates the grid at each commit. A unit sees earlier units where they now are and later ones where they were.
  The grids are scratch, rebuilt every tick and never hashed.
- **No overlap by construction**: a position is committed only if `World::is_free` holds, the same strict predicates
  as invariants I1 and I2 (touching is free), with the exceptions of `space::blocks` below and no other. Contact positions are assigned from the blocker's edge
  (`space::resting`), never accumulated. Never write `pos` anywhere else, and never loosen the predicate with an
  epsilon.
- **Lane seeking** (`World::lane`, `World::lane_step`; `docs/movement.md > 6.3`): before anything else, a unit
  looks 4 of its lengths ahead for the units it has to pass: oncoming ones (by `Body::heading`, where they are
  going) and idle units that yield to it. Each bars a range of sideways shifts, its lane. The unit steps, forward
  and sideways at once, towards the nearest clear lane within 3 of its widths, or, in front of a dense group,
  towards the middle of the widest gap; two dense groups mesh like combs and spread from the outside in. One idle
  unit alone is no lane. The code and its constants, the `LANE_*`, are in `collide/lanes.rs`. It is deterministic
  by sorted lanes and fixed cell order, and keeps no state but `Mover::avoid_side`. Never let it write a position
  except through `World::clear`.
- **The grid's cells carry marks** (`Body::kinds`, `Layer::near_any`): which heading sectors and whether a yielding
  unit (and, slice M5-V4, a crushable one) were listed in a cell this tick. They only ever say "maybe", and exist so
  that a unit with nobody to pass doesn't look at bodies. Keep them conservative: mark on every insert and whenever a
  body's kinds change.
- **Oriented Silhouettes** (slice M5-V1, plan decision T78; `docs/movement.md > 3`, `docs/movement.md > 6.3`): a
  Silhouette that isn't square (`ObjectDef::silhouette_turns`: the Seeker, the Prayer-mobile, the Hand of Heaven)
  lies along its unit's facing. `shape.rs` has the geometry: the separating-axis test for overlap and the sweep, the
  gap from corner to edge, the bounding box for tile work (vision, the grids, footprints), and the shape itself
  against blocked tiles where that box isn't clear (`Shape::clear_on`, `Shape::tile_contact`). The movement step
  carries it as `space::Body::oriented`, and `space::overlaps` and `space::contact` take the box arithmetic for two
  boxes, untouched, and the shapes' otherwise; so do invariants I1 and I2, the debug teleport and the spots units
  are put on or kept off. **A turn is tried before the move** (`collide/turn.rs`, `World::turn`): in parts across
  which a corner moves at most `Movement::turn_try_su`, the first blocked part halved `Movement::step_halvings` times and its free part
  kept; a cut turn is a blocked move (momentum zeroed, the blocker asked to yield, the stuck clock running), and the
  unit takes its step in the shape it has (`follow::creep_on`). Where a move meets a turned shape, the go-around
  rules read the face the two meet on (`space::Face`), which for two boxes gives back the coordinates exactly. Never
  test a turned unit against the tiles or another unit by its box alone, and never turn a body outside
  `World::turn` (DragMovement's turns and a portal's doorway go through the safety net in `movement::step`).
  Measured (`tests/vehicles_perf.rs`, 15 Seekers and 15 Hands of Heaven driven through each other, 600 ticks,
  release, load average 4.6 to 7): a whole tick 0.038 ms on average, 0.12 to 0.13 ms at the 99th percentile; the
  movement phase 0.016 and 0.083 ms. `movement_perf` (no turned unit in it) ran A/B against master, interleaved, six
  rounds each: 1.15 ms on average on master's side, 1.08 ms on this one, within the noise of that load. (M5-V3) With
  the Hands of Heaven tracked, three interleaved rounds at a load average of 2.7 to 3.2: a whole tick 0.039 to 0.046
  ms on average, 0.14 to 0.20 ms at the 99th percentile (master's side, which had no tread law, 0.032 and 0.10); with
  30 more Hands of Heaven crossing north of them (`perf_30_vehicles_and_30_more_tracked`), 0.093 ms and 0.24 to 0.25
  ms. The corner choice tests arcs against the nav grid each tick a unit turns. `movement_perf`: 0.485 ms on master's
  side, 0.483 on this one. (M6-GF) With fifteen Raptors in place of the Seekers (`perf_30_vehicles_with_raptors`), ten
  rounds interleaved with master's runs at a load average of 1.1: a whole tick 0.037 to 0.048 ms on average, 0.13 to
  0.18 ms at the 99th percentile, beside master's Seekers' 0.040 to 0.050 and 0.13 to 0.23 in the same rounds.
- **One unit's tick** (`step_unit`, `collide.rs`): turn towards the steering waypoint (at most TurnRate), then move
  one step: a lane step (above), else free; **slide** along tiles (per axis); **follow** a unit moving the same way; **go around** a standing
  or oncoming one (`collide/contact.rs`: a glance along its face, else **the contact step** of design change 0323:
  straight sideways along the blocker's face by the unit's own width, taken over several ticks as `Mover::step_aside`,
  hashed only while set, during which the unit makes no other move; it commits its side for a second; at a standing
  blocker the side is the near one, at a moving one the side the unit leans to, else its Right first, so head-on
  units pass like traffic; a crossing unit is passed behind); else close up or stay. **Which contact step a base
  takes is data**: `UnitBaseDef::contact_step` (`ContactStep::Sideways`, or `Diagonal`, the 45° step as built before
  0323, kept for the WheeledVehicle until its turn and graze is built). Never match on the base in movement. Within one step of the
  goal the unit lands exactly on it. A unit without a goal moves only by its own sidestep (below), by braking off
  its momentum, or off a unit it overlaps (*Movement models*, *Idle units that overlap*): nothing pushes.
- **Friendly yielding** (`yielding.rs`, `docs/movement.md > 6.6`; design changes 0022, 0023, 0027): a moving unit
  asks the idle unit of its owner that blocked its way to step aside. Requests are collected during the step and
  applied after it in the order made (the lowest id that asked wins); the unit then takes one short sidestep
  (`Mover::sidestep`, hashed state) in its own turn from the next tick on, through `World::is_free` like every
  move. `yielding::can_yield` is the one place that says who yields (an owner, no order at all, no goal, not in an
  attack sequence, `Mover::can_move`: the hook for units that can't move in their mode), and `yields_to` to whom
  (its owner's units, but not the group order it arrived with). A blocked sidestep passes the request on, at most
  `Movement::yield_chain_depth` deep. Moving units never yield or swap. A lone unit squarely in the way steps at 45° to the
  mover's way; a unit that is only clipped, or stands in the way of a mover walking between the units of a group,
  steps straight across it, and such a mover asks the units on both sides of its gap two steps before it reaches
  them (`yielding::ask_ahead`): the group parts and isn't shoved along. **A pop-up turret that wants to rise asks the
  same way** (slice M4-S2, `yielding::make_way`; design change 0296 point 3: "The owner's own idle units over it are
  pushed off by the idle push"): after the units have moved, each lowered turret kept down (`popup::is_blocked`) asks
  the units over its footprint that yield to its owner for a sidestep off it, along the axis on which they are out
  soonest, to where their box touches its edge; the movers' requests come first, and it asks again every tick it is
  kept down. No other unit is ever moved for it.
- **Collision exceptions** between units go in `space::blocks(mover, other)` and nowhere else. The
  sweeps, lane seeking and `World::is_free` all ask it, so a unit ignored for a move is ignored for the whole of it.
  It is asked last, after the box test (few units near a move are in its way). Three exceptions (design change 0106,
  slice M3-H2; crushing, slice M5-V4), all read off the bodies each tick, with **no pair state**:
  - **Gathering Drones** (`systems/units.md > UnitCollision > Ground Collision > Exception, gathering Drones`):
    two bodies with `Body::gathers` (`orders::gather::on_gather_job`: a Drone whose current order is a Gather or a
    Drop Off, at every step of the job) don't block each other, whoever owns them. A gathering Drone and any other
    unit block each other as ever, an idle Drone included.
  - **Units that already overlap** (`systems/units.md > Units that already overlap`): two bodies whose boxes overlap as
    the mover's turn begins don't block each other. `space::overlaps` is I1's own predicate, so the pair is ignored
    exactly while I1 would name it, and normal collision is back the moment the boxes are apart (touching is apart).
    Nothing pushes them apart, and either may move any way meanwhile; an idle one moves off by itself (design
    change 0126, below). Only the first exception, an evacuation and the Garrison's door ever bring two boxes to
    overlap (design change 0349, slice M5-F3: everyone inside a destroyed building appears inside its former
    footprint at once; slice M5-N2: an Eject All lets everyone out of the Garrison's door at once, at one spot), so in
    today's game such a pair is two Drones of which one left its job, two evacuees, or two units out of a Garrison;
    the rule itself asks nothing about the units' kinds.
  - **A crusher's crushable enemies** (slice M5-V4, *Crushing* below): while a crusher's move crushes
    (`Body::crushes`, set on the mover's own body for its turn alone), the crushable units of its enemies don't
    block it. The clause reads the mover's flag only: to every other unit's move the crusher blocks as ever.
  A slice that adds a way for two boxes to come to overlap must add it to
  `testing::invariants::overlaps::may_overlap` too, or I1 fails. Crushing doesn't: the crushed are out of the game
  by the end of the tick of contact.
- **Crushing** (slice M5-V4, plan decision T86, design changes 0418, 0419 and 0453; `systems/units.md >
  UnitCollision > Crushing`, `docs/movement.md > 6.13`; `movement/crush.rs`): a unit whose base crushes
  (`UnitBaseDef::crushes`: the TrackedVehicle and the Mech; the DrillUnit only above ground, slice M6-ST1) crushes the
  crushable enemies (`UnitBaseDef::crushable`: Light Infantry; enemies by `economy::defeat::allied`, neutrals
  nobody's) in its way while its move crushes: at any speed (Moving, Reversing or Stopping, 0453) or turning in place
  (0418), unless its attack holds it at rest (`crush::drives`, decided before its move). At its commit, those its
  Silhouette overlaps where it ends or swept in its turn or its move (`crush::victims`) leave the grid at once, so the
  units after it don't meet them and those not yet moved don't move, and `movement::step` returns them: `Sim::step`
  removes them at the end of 5d through the removal path (grids synced, a Disciple's Shroud ended, before 5e), each
  with a `SimEvent::Crushed { id, kind, owner, pos, hiding, crusher }` in place of a `Died`. It isn't damage: no
  `Hit`, no Shields. Never kill a unit in movement any other way.
- **The exit doorway** (`core/entities.md > Structure Type > Exit Doorway`, design change 0080; a portal's energy
  doorway, design change 0145; `production/exit.rs`, `production::ExitDoor`) is the one exception against tiles,
  and the one case in which a unit's box overlaps a structure. A unit a producer released (or a building that holds
  units let out, `Sim::release_held`, slice M5-F1) has `Mover::exit`, the building's id, while its box still
  overlaps that footprint. For that unit alone, `World::door_free` replaces the tile tests of `tiles_free`,
  `is_free_for` and `along_axis`: the building's own tiles don't block it, but for ground under them that the unit
  can't stand on (slice M5-Z1, `Frame::blocks`, `NavGrid::is_terrain_solid`: the building lets the unit overlap the
  building, not that ground, so that the unit never stands there once the building is gone; a debug spawn or a map's
  object may cover any terrain, and a custom preset may be Buildable and not Traversible; design change 0374,
  answering Q0134, `core/entities.md > Structure Type > Exit Doorway` point 6: "never released onto ground under it
  that it couldn't stand on ... Such a building **holds its finished unit** indefinitely"), every other blocked tile
  does (`ExitDoor::box_clear`), and it may only come out (`ExitDoor::allows`: out of a side doorway, never deeper in,
  and inside only within the exit side's span, so it leaves through that side and no other; out of an energy
  doorway, never closer to the footprint's centre). A door whose every spot lies over such ground has none
  (`Sim::doorway_spot`), and its building waits as behind an occupied doorway (fuzz seed 155: a Hall spawned over
  Cliffs had left its Recruit on one). Units still block it and it blocks them, as ever.
  Phase 5d clears `exit` (and `Mover::exit_heading` with it) in the tick the box is clear of the footprint (or the
  building is gone, or a teleport took the unit away), and nothing ever sets it again: a unit that is out can't
  re-enter.
  - **Out of a side doorway** (the Barracks'), with a goal the unit moves by the ordinary rules (lane seeking,
    going around, yielding requests); without one, `doorway::leave` walks it straight out and stops it against the
    exit side (design change 0086), trying again every tick while something is in the way.
  - **Out of a portal's energy doorway** (slice M3-D2, `doorway::emerge`), goal or none, the unit moves **straight
    along `Mover::exit_heading`** (chosen at its release, see *Production*) and nothing else: no lane, no way round,
    no sidestep (`yielding::can_yield` leaves it out), so it never leaves the ray, and the ray leaves through an
    open side. It moves by its own model (one that stops within the tick turns to its heading first, then steps;
    the Drone thrusts along the ray, `Drag::along`), through `World::straight`: as far as tiles allow (the step
    halved up to 8 times) and up to the first unit. Without a path it lands exactly where it comes clear
    (`EnergyDoorway::outside`), a hair short counting (`CLEAR_SU`, 1/64 su), and stands there; with one it keeps
    its speed out of the footprint, and the next step is `follow`'s. It asks its owner's idle units in the way to
    step aside and tries again every tick; with a path its stuck clock runs, so a move that can't begin fails, and
    the unit stays where it got to, "half out at the edge of the footprint, as a normal unit" (0145 point 4).
  **Never set `Mover::exit` anywhere but where a producer releases a unit (`Sim::release_unit`) or a building that
  holds units lets one out (`Sim::release_held`), and never test a doorway unit's tiles anywhere but through
  `World::door_free`.**
- **Held by its attack**: a unit whose attack sequence holds its base (`combat::may_move` or `may_turn` false: see
  *Combat*) stands for the tick and keeps its goal, path and stuck clock (`wait`), like a unit waiting for its path.
- **Stuck and recovery** (`stuck.rs`; the numbers, in `mover/schedule.rs`, are the design's placeholders of changes
  0024 and 0026): progress is the remaining path length; waiting behind a unit that is getting on counts. A unit is
  stuck after `Movement::stuck_ticks` (24) without it (`Mover::is_stuck`); it then tries another route `Movement::max_repaths` times,
  after waits of `Movement::repath_waits` (8, 16, 32; `Mover::recovery_due`), around the tile of a blocker that stands, and
  gives up 24 ticks after the last: `MoveStatus::Failed` and `SimEvent::MoveFailed`, `Movement::give_up_ticks()` (104) after its
  last progress. **Occupied target**: within `Movement::occupied_arrival_gu` (3, design change 0169; 2 under 0026) of its goal
  behind a stationary unit that won't yield to it, the unit has arrived instead, without a repath: when stuck, or
  after `Movement::jammed_ticks` without a way forward if the blocker has no goal. That covers a unit walled out of its
  magic-box spot by its own group's arrivals (design change 0183): arrived within the radius, given up beyond it.
  Two moving units head-on in a passage too narrow to pass both fail. **The crowd sweep** of slice M1-G3 is kept as
  the ignored `movement_crowds.rs::crowd_sweep` (two blocks crossing head-on, 288 layouts, run in release): for
  0169 it went from 43 to 7 failed units of 5 856 for each owner count (Peacekeepers 8 → 0, Guards 20 → 4,
  Peacekeepers against Guards 15 → 3).
- **Whose progress a waiting unit shares** (`stuck::leads`, slice M3-H2): a unit blocked by one that is on its way
  takes over that unit's `last_progress`, but only if that unit **has got on since its leg began**. `Mover::go_to`
  sets a unit's own clock to the tick of the order, and that is no progress: before this rule, a blocked unit whose
  legs began again and again (a behaviour trying again, a player clicking on it) renewed the clock of every unit
  behind it, and none of them ever looked for a way round (`tests/movement_stuck.rs`). It is read off the mover
  (`repaths_left < MAX_REPATHS || last_progress > last_repath`), with no field of its own. A behaviour may begin a
  leg again as often as it likes; it need not stand in between for the sake of the units behind.
- **Object goals** (`MoveGoal::Object { id, reach }`, `docs/movement.md > 5.6`, `docs/movement.md > 6.5`): the path is
  planned to the point nearest the object's centre that the unit can stand on (right at a structure's wall), and kept
  aimed at the object as it moves (`paths/checks.rs::chase_check`, on the stagger tick: straight at it while the unit's
  box has a clear line, else a new plan once it moved 1 GU or 25 % of the distance, at most every 8 ticks, never counted
  against the repath cap). The unit has `Arrived` when the gap between the two shapes is within `reach`, at the end
  of its path if the object hasn't moved, or by the occupied-target rule (its 3 GU measured as that gap). Progress
  towards a moving object is moving forward at all; towards one at rest it is measured as to a point. A unit
  chasing its attack's target behind stationary attackers of its owner on the same target **queues**
  (`stuck::queues_behind`): no path request; it fails as any stuck unit does once its attempts are used up (slice
  M3-G3, design change 0223; until then it never failed).
- **A queueing unit goes round the row** (slice M3-G2, `movement/round.rs`, `docs/movement.md > 6.5` rule 2;
  `systems/units.md > UnitCollision > Ground Collision`: "Moving units must pathfind around occupied space").
  The row is the unit in its way and the units standing for good next to it, nearer to each other than the unit
  is wide. The detour leads straight out of the row's box (grown by the unit's half extents and
  `Movement::row_clearance_su`), round it from corner to corner, to an end of the side that faces the target: the shortest
  way whose legs are clear of blocked tiles and of the row's units. `Mover::go_round` puts its waypoints in front
  of the path, and `Mover::detour_end` (hashed and serialized only while set) keeps the staggered check from
  skipping them. An attempt counts against the leg's `Movement::max_repaths` on the repath schedule, or comes after
  `Movement::jammed_ticks` behind a unit with no goal; it asks for no path. One after a lost race (the unit now in the way
  moved after this unit's last attempt, `Body::moved`) is free. **With no way round, or no attempt left, the
  Failure rule applies** (slice M3-G3, design change 0223: "it does not wait behind the blockers indefinitely"):
  the unit gives up still stuck `Movement::stuck_ticks` after its last attempt, and its behaviour gives the target up. M3-G2
  had it wait and stand there (`stuck::waits_in_queue`, `World::hold`, both gone). Measured
  (`tests/attack_behind_friends/`): four Peacekeepers in a column all fire on a building within 135 ticks of the
  order in all 16 layouts, where one of the four never fired in 5; eight within 255; in a corridor the ones behind
  give up within `Movement::give_up_ticks()` of their last progress, with the 8 ticks' grace (`no_room.rs`). **Never give a
  queueing unit a path request for the row**: no grid shows it. A melee unit behind its owner's attackers is
  within `Movement::occupied_arrival_gu` of its target, so the occupied-target rule comes first and its approach ends there:
  it gives the target up (design change 0223 point 1, as built).
- **Movement models** (`movement/follow.rs`, `movement/drag.rs`, `movement/wheeled.rs`, `movement/tracked.rs`;
  `docs/movement.md > 6.7`, `docs/movement.md > 6.11`, `docs/movement.md > 6.12`): path following, lane seeking, the
  rules after the sweep, yielding and the stuck clock are shared (`follow`), and a model (`movement/model.rs`,
  `Model::of`) replaces only the "direction → facing and velocity" part. Four are built so: the units that stop
  within the tick (every GDO unit and the Legionnaires: the step the rules give is their move), **DragMovement** (the
  Drone, slice M3-G, plan decision T25), (slice M5-V2) **FixedTurnRadiusMovement** and (slice M5-V3)
  **SpeedTurnRadiusMovement** (below). **GliderMovement** (below) replaces the whole tick.
  - **The integration scheme is fixed**: turn (at most TurnRate, towards the step the unit wants; it never stands
    to turn), drag (`v ← v × (1 − DragRatio)`, the ratio doubled while the tile under the unit's centre is Rugged:
    design change 0039), thrust (`v ← v + a`: OmniDirectionalAcceleration any way plus up to ForwardAcceleration
    along the facing, whichever brings `v` nearest to what the unit wants), position (`pos ← pos + v` as far as
    tiles and units allow). Drag is taken on the velocity the tick began with, so the equilibrium is the design's
    MaxSpeed `(Omni + Forward) / DragRatio`: 8 su a tick for the Drone, 4 on Rugged.
  - **What it wants** is the step the shared rules give a unit without momentum, at the **approach speed**
    (`drag::approach_speed`): the highest speed from which braking with the omni-directional thrust still stops
    it within the path that is left. So braking begins when the stopping distance reaches the remaining path
    (35 su before the goal from top speed with the Drone's numbers of design change 0205, 51 before them), and the
    last step lands exactly on the goal. No overshoot.
  - **(M6-GH1) An air HoverCraft brakes by drag alone** (`UnitBaseDef::brakes_by_drag`, the HoverCraft base's;
    `movement::drag::Terms`; `systems/units.md > DragMovement`: "Air units typically have high drag and high
    acceleration, relying on drag alone to stop"): it never thrusts against its motion. Without a goal it drifts,
    `v ← v × (1 − DragRatio)`, and holds still once a tick's thrust takes all its drag leaves (**placeholder**: how it
    comes to rest); its approach plans the same braking (`drag::braking_approach`: no braking thrust, then the tick
    that holds it still), so it still lands on its goal. On its way it thrusts towards the step it wants, which steers
    it. The Hornet (9 su a tick at top speed, DragRatio 0.1) drifts 80 su in 42 ticks where its thrust would stop it
    within 60 su in 21, and begins to slow about 89 su before a goal. The ground hover units (the HoverVehicles, the
    Drone among them) still brake with their thrust. Of the golden games only those with an air HoverCraft moved,
    re-blessed: `gdo_drones` by its OperatorDrones' braking (its falls unchanged: only the drone sent past its leash
    falls there) and `syndicate_air` by its Phantoms' (slice M6-SD's, merged first), from tick 736.
  - **Blocked means from rest** (`systems/units.md > UnitCollision > Collision Response (Un-sticking)`, design
    change 0035): a move that runs into a tile or a unit goes as far as it can, and the unit's **whole velocity is
    then 0**, not only the part into the obstacle. Where the rules' step is a slide along tiles (`Moved::slid`),
    the unit thrusts at the step it asked for, into the tile, and meets it every tick: it creeps along the wall
    at single-tick acceleration (1.2 su for the Drone) until its way is free, then accelerates normally. That
    creep is progress for the stuck clock wherever it gets closer (`movement/follow/tests.rs`). A wall it only
    touches, a unit it passes through (`space::blocks`) and a unit it moves off (0126) are no obstacles. A debug
    teleport zeroes the velocity too.
  - **`Mover::momentum`** is the velocity carried into the next tick: the displacement the unit made, or zero if
    it was blocked. It is hashed and
    serialized **only while it isn't zero**, so states of games without such a unit hash and load as before the
    field existed. `Mover::stop` and `finish` leave it: a unit without a goal that still has some brakes to a
    stop, `Locomotion::Stopping` (within 26 su and 9 ticks from top speed; 42 su and 16 ticks before design change
    0205). **Never zero it to stop a unit**, and
    never read `velocity` for it: `velocity` is the tick's displacement and `stop` clears it.
  - **Going round a standing unit**: a unit with momentum takes the side of the blocker's middle it is on
    (`World::around`, `near_side`), whatever its lean, and a step aside doesn't count as blocked while the unit
    moves (`Moved::aside`): the manoeuvre takes it a dozen ticks, and turning back costs it all its speed.
  - **Kinematic for every model**: a yielding unit's sidestep, the walk out of a side doorway and an idle
    unit's step off a unit it overlaps are plain steps without momentum. A corner is taken at speed: the unit
    swings wide, or meets the wall beyond and starts again from rest. **(M3-D2)** The walk out of a portal's energy
    doorway is the model's own (`Drag::along`: thrust along the heading, the speed carried from tick to tick, and
    without a path braking to land where the unit comes clear).
  - **FixedTurnRadiusMovement** (`movement/wheeled.rs`, `Wheeled::drive`; slice M5-V2, plan decision T79; the
    Seeker and the Prayer-mobile): a signed speed in `Mover::momentum` (backwards on a Reverse leg,
    `Mover::reverses`, hashed only while set), up by the acceleration of the way it drives and down by
    Deceleration; it turns towards the probe's step by at most `|speed| / MinimumTurnRadius` (never in place, and
    not while it stands or brakes to set off the other way: `movement::turn_rate` is 0 for it, so phase 5e's aim
    never turns it), and moves along the heading halfway through the turn. Blocked means from rest, as above, and a
    turn cut where it is pressed is made all the same, the unit backed off by what its corners sweep
    (`World::back_off`, `movement/collide/turn.rs`): it creeps and turns out of contact. Where the probe's step is
    a slide or nothing it heads along the step it asked for. A goal inside its turning circle is reached by a
    forward loop (`Wheeled::steer`; a waypoint on the way inside it is passed). At a blocker it grazes
    (`ContactStep::Graze`: no step aside, a whole step along the face, its side committed); an idle vehicle
    doesn't yield (design change 0353). Ask `MovementAttrs::reverse_speed` for how a model backs up, and
    `UnitDef::can_reverse` whether a unit takes the Reverse order (`OrderKind::Reverse`; the gate keeps only those
    units, `Reject::CannotReverse` if none). Invariant W1: it turns by its speed.
  - **SpeedTurnRadiusMovement** (`movement/tracked.rs`, `Tracked::drive`, and `movement/tracked/corner.rs`; slice
    M5-V3, plan decision T85 as design change 0417 replaced its law; the Hand of Heaven): the tread law, SpinRate =
    2 × MaxSpeed ÷ TrackWidth (`MovementAttrs::spin_rate`) and at the speed `v` at most SpinRate × (1 − |v| ÷
    MaxSpeed): it spins in place at rest and can't turn at MaxSpeed. A signed speed in `Mover::momentum` as
    FixedTurnRadiusMovement's, backing up on a Reverse leg at ReverseMaxSpeed by the same law. **The corner choice**
    (`Tracked::corner_speed`): the speed it takes a turn at, timed analytically from the law for 33 speeds from a
    stop to its top speed, the soonest whose tightest arc is clear of the nav grid's tiles (`Tracked::arc_fits`),
    else a stop and spin; for the turn now and, braking in time, for the corner at its steering waypoint
    (`Tracked::ahead`). A turn cut short backs off by what its corners sweep, at most what the unit moves in the
    tick; at rest it takes a glance off a unit and a cut spin's step as they are, its facing kept, and a spin runs
    the stuck rules. Its contact step (0323) is taken as it is, a turned Silhouette's straight along the face it met;
    an idle one doesn't yield (design change 0440). Phase 5e's aim has its SpinRate, less what its speed takes
    (`movement::turn_rate`, `tracked_turn_noted`). Invariant S1: it turns by the law at its speed.
- **GliderMovement** (`movement/glider.rs`, `Glider::fly`; slice M5-H1, plan decision T81; `docs/movement.md > 6.9`): a
  Glider (the Sparrow) flies instead of following a path, and collides with nothing. Its speed is `Mover::momentum`'s
  length, always between IdleSpeed and MaxSpeed, its turn in a tick at most `MaxCentripetalAcceleration / v`, and its
  Locomotion always Moving. Without a goal it circles its `Mover::orbit` at IdleSpeed on 1 grid unit
  (`tuning::air::Air::idle_circle_gu`); with a point it arrives within that circle and circles the point from then on;
  an Attack or an Attack Ground makes a strafing run (`Mover::strafe`, `Pass`: toward the target, over it,
  on until out of range, back at full speed), set by `orders::engage::close_in` and `orders::attack_ground`. (slice
  M5-N7, design change 0408) **Where a pass ends** is the one range check measured from the glider's edge to the
  target's **centre** (`Shape::distance_to`, `Strafe::range`): the loop is the same size whatever the target, about
  490 su for the Sparrow; firing and every other range check stay edge to edge. (slice M5-N7) **A takeoff run**
  (`Mover::takeoff`, `Takeoff`; hashed and saved only while set): an aircraft its building lets out flies straight
  along the runway, never turning, until its centre is past the runway's end (`Takeoff::is_over`); neither a new leg
  nor a stop ends it, so an order given meanwhile waits for the end (Placeholder (not in the design: a takeoff run
  cut short)). Ask
  `UnitDef::glides` (or a behaviour's `orders::world::Unit::glides`) whether a unit is a Glider. **Never make a Glider
  stand**: `Mover::stop` and `finish` clear its goal, orbit and run, and it circles where it is from its next flight;
  (slice M5-H2, design change 0404) a Move's order completes as it arrives, and the next queued order starts there
  (`orders::run`; with none it circles the point, idle). The
  ground units' back-off from a target inside MinRange is skipped for it. Its base gets no turn budget in phase 5e
  (`movement::turn_rate` is 0 for it): only its turret aims, and (slice M6-GL) a Glider with none, the Dragonfly,
  fires once its flight has its heading within `Combat::facing_tolerance` of its target
  (`combat::turret::Source::Base`). **Its landing is the one exception** (slice M6-GL, `movement/approach.rs`; design
  change 0428 point 5: "it approaches its slot at IdleSpeed, stops over it while it lands (16 frames), and takes off
  straight up, leaving at IdleSpeed"): a leg begun by `Mover::approach_to` (`Mover::approach`, `Approach::Flying`;
  hashed (Mover tag 13) and saved only while set, cleared by `go_to` and `stop`) is flown by `approach::fly` in place of
  `Glider::fly`: MaxSpeed only while the point is ahead, outside its widest turn and beyond `Air::approach_gu` (4 GU, a
  placeholder), else IdleSpeed; straight on while the point is inside the circle its tick-by-tick flight at that speed
  makes (`approach::circles`, the polygon's circumcircle, not the turn's tangent circle) and the tick's turn can't reach
  it; and at IdleSpeed, with the point on the line of the tick's flight (`ON_LINE_SU`), onto it, its leg over
  (`MoveStatus::Arrived`), Stationary and **over** it (`Approach::Over`), its momentum kept for that tick. It holds
  still in the air (`approach::holds`, `approach::hold`: no move, no momentum, Stationary, its goal kept) while over the
  point with its order running (an order that ended without a stop, a removal's, leaves it stale: `approach::stale`
  stops it), on an Airfield's slot (`landing::of`), and rising (`Mover::rise`); its rise's last tick flies it from
  IdleSpeed. **Placeholder (not in the design: how a Glider lines up with its slot)**. G1 allows the hold.
- **Where an object counts as being: `Object::map_pos`** (slice M5-H1; `systems/units.md > Air Collision`: a Glider
  outside the map edge "stays visible and attackable as if it were at the nearest edge tile"). A Glider's `pos` may
  leave the map while it turns or circles (no cap: design change 0354); `map_pos` is `pos` clamped onto the map
  (`Map::clamp_point`), and `Shape::on_map` the shape there. Vision (an object's eye and how others see it,
  `PlayerVision::map_box`), targeting and the combat phases, ranges and chases (`movement::Chased`), the behaviours'
  snapshot (`orders::world::View`) and path serving read `map_pos`. **Read `pos` only for where the object is drawn
  and moves from**; anything that asks where it is in the game asks `map_pos`. For every object on the map the two are
  the same.
- **Air units keep apart softly** (`movement/air.rs`, `air::separate`; slice M5-H1, plan decision T81, design change
  0328; `docs/movement.md > 6.10`): after every unit has moved and yielded in 5d, each pair of air units with a
  SeparationRadius (`UnitDef::separation_radius`: the Sparrow's 48 su, (slice M5-H2) design change 0405, was 32)
  nearer than the larger radius is pushed apart, each by half of what the pair lacks, a unit's whole push in a tick
  capped at `tuning::air::Air::separation_push_su` (2 su, a placeholder). (M5-H2) Each unit takes its own base's share of
  its pushes and of the cap (`UnitBaseDef::separation_strength`): a HoverCraft all of it, a Glider (slice M5-N7) a
  third (design change 0412; a quarter from design change 0405: "Gliders push apart slowly"). The pairs come from a
  `space::Layer` of the air units, in id order, and
  the pushes are summed and applied together; nothing blocks them. A HoverCraft is kept on the map (design change
  0045), a Glider isn't. (Slice M6-GK) So is a HoverCraft's own move (`movement::step`: the edge stops it as an
  obstacle does, design change 0035), the end of its straight path (`paths::serve`, so that it arrives at a corner
  or an edge) and where the gate puts it (`Sim::check_unit_fits`, Blocked), all by `Map::clamp_box`. A thrust's burst
  (`crate::thrust`, the Prism's) is added after that clamp, which writes the unit's own move into its `velocity`: the
  burst goes through `World::shuffle` and, for a HoverCraft, `Map::clamp_box`, but never into `velocity` or
  `momentum`, so that the drift after it is the unit's own; none while the unit rises out of an air door. I3 and
  I5/I7 allow a tick's burst step (`thrust::burst_moved`). A push is no part of the unit's own move: `velocity` and `momentum` stay its movement's, and
  a unit at rest stays at rest. Units spawned in the tick are pushed against but not pushed. (M6-GH2) An aircraft on
  an Airfield's slot, landing or down (`landing::at`), is out of the separation, neither pushed nor pushing, so that it
  stays on its slot while others fly in beside it or over it (**placeholder**; *Production > The Airfield*).
- **Idle units that overlap move apart** (`movement/separate.rs`; design change 0126, which amends 0106's "nothing
  is pushed apart"): in its turn, a unit that yields (`yielding::can_yield`: no command at all) and is at rest
  takes a step off the lowest-id unit its box overlaps, along the axis it is out soonest, at
  `Movement::separate_speed_su` (2 su a tick, a placeholder), no farther than to where the boxes touch. Where two centres
  coincide the lower id goes west or north. Units with a command are never moved (Hold, a Gather, any order).
  The step goes through `World::shuffle`, so it never enters a unit it doesn't already overlap; `n` units stacked
  on one point are apart within `18 × (n − 1)` ticks for Drones.
- **Counters**: `Sim::move_counters()` (sweeps, box tests and lane tests of the latest tick) and `Sim::nav_counters()` are
  output, not state. Tests assert them; `cargo test -p sc_sim --release --test perf movement_perf -- --ignored
  --nocapture` prints timings.
- **Invariants** (`docs/movement.md > 8`, `sc_sim::testing::invariants`): I1 no two units of a layer overlap, except
  (design change 0106, `invariants::overlaps::may_overlap`) two Drones on a Gather job, and two units that already
  overlapped a tick earlier, until they have been apart (a Drone counts as on its job for the tick in which the job
  ended without another order: a patch emptied in the economy phase ends it after the movement), and (design change
  0349, slice M5-F3) a unit evacuated in the tick with any other (`SimEvent::Evacuated`: it wasn't on the map a tick
  earlier), and (slice M5-N2) two units one building's door let out together in the tick (`SimEvent::LeftBuilding`,
  the Garrison's `ExitRule::AtOnce`). Any other overlap fails,
  the first tick it exists (`tests/invariants_overlap.rs` puts units where they must not be and asks the checker); I2 no
  ground box overlaps a blocked tile or leaves the map, except a unit in an exit doorway (`Mover::exit`), which
  overlaps the tiles of the building that released it and no other blocked tile, nor ground under them it can't
  stand on (slice M5-Z1), overlaps that footprint for as
  long as `exit` is set, never moves deeper in (`ExitDoor::allows`), never stands off a portal's open arc
  (**M3-D2**: `EnergyDoorway::margin` at least −1e-3, `ARC_TOLERANCE`), and never comes to be in a doorway by
  moving (`tests/production/portal_stalls.rs` has the energy doorway's negative tests for I1 and I2); I3 a unit moves at most its speed per tick; I4 every float
  is finite (I3's speed is the model's maximum for a unit with momentum: a mode that slows it only weakens its thrust;
  (M5-N3) in a tick a research level completes, after the movement step, the larger of its MaxSpeeds before and after;
  (M5-N11) so too in a tick that changed what its buffs add to it, an Inspire cast or ended; (M6-SB) a unit with
  momentum that the sludge slows brakes from the speed it carried down to its lowered top speeds, the larger of the
  two);
  I5 a unit without a goal doesn't move; I6 repaths per leg are capped and no path request waits longer
  than `MAX_REQUEST_AGE`; I7 an arrived unit never moves again without an order or a move of its behaviour, and a
  unit without a goal is at rest. A leg is one `Mover::go_to`: an order's move, or one a behaviour starts (a chase,
  a Patrol's next leg, the way back to an IdleOrigin). The exceptions to I5 and I7 are a unit taking its sidestep,
  which must have neither a goal nor an order, a unit walking out of an exit doorway, **(M3-G)** a DragMovement
  unit braking off the momentum it had (it must be `Stopping`, its velocity its momentum, and slower than a tick
  before; no other model may ever have momentum), and an idle unit that overlapped another a tick earlier, or was
  evacuated in the tick, or (M5-N6) that an evacuee came out on (`overlaps::landed_on`: a unit in the doorway of the
  building that went, I1's exception 3), moving off it (design changes 0126 and 0349), and (M5-H1) a Glider, which flies and has
  momentum without a goal, and an air unit with a SeparationRadius pushed off another, by at most
  `Air::separation_push_su` ((M5-H2) its base's share of it; which I3 allows it beyond its speed too). (M5-H1) G1
  (`gliders`): every Glider that was in the game a tick earlier flies at a speed between its IdleSpeed and MaxSpeed,
  its velocity its momentum, and its `map_pos` is on the map; one stopped after its flight (its Attack's target died
  in phase 5e) shows no move until its next, and keeps its speed; (slice M6-GL) one held still by its landing (on a
  slot, rising off it, or over its approach's point, `movement::approach`) shows no move and no flight, Stationary,
  with no speed. K1 (`paths`): each player's known grids are
  those a fresh build
  from the state gives. P1 (`projectiles`, slice M4-O, plan decision T42): every projectile in flight lands after the
  tick just played, between finite points. O1 (`operations`, slice M4-F): a Tunnel's operation and its site name each
  other (`TunnelState::operation`, `Builder::Tunnel`); an upgrading Tunnel is in operation, a tier below its target,
  over its target tier's frames (slice M4-G). R1 (`reservations`, slice M5-F2) is under *Production > Pool buildings*.
  (M5-M1) H1 (`hit_points`): every object, on the map or held, has HP exactly when it has a MaxHP, above 0 and at most
  its MaxHP with its Augmentations (`Object::max_hp_with`), and Shields within 0 and its MaxShields; B1 (`banks`): no
  bank wraps below 0 or past its top in a tick (it changes by less than half its range, but by a `SetResources` of the
  tick); A1 (`augmentations`): only an augmentable kind has an Augmentation, and (M5-N3) only a neutral unit keeps
  research, a defeated player's (`Object::kept_research`); (M5-N4a) E1 (`energy`): every object,
  on the map or held, has energy exactly when its kind has a pool and (M6-CM) it is no illusion, at most its maximum;
  (M5-N3) T1 (`research`):
  techs come only from completed research: a player holds a technology only with its previous level, none is both
  completed and queued, nor queued twice or at two structures, and each queued one is offered by its structure's type
  (`StructureDef::research`), at a structure with an owner who has its previous level; (M5-N4b) C1 (`clouds`): every
  cloud member that channels is a live unit channelling Shroud in the channel it names, (M6-CM) every lingering one
  vanishes within `Spells::linger` ticks, at the tick it said the tick before (its time only runs down), each unit
  that channels is in exactly one cloud, a cloud's circles are connected and two clouds never touch, and a buffer is
  never below 0, its timer running while it holds anything and never past due; (M5-N5) F1 (`buffs`): no object, on
  the map or held, holds two buffs of an ability, each ends within its ability's duration from now, and only a kind
  the ability affects holds it; (M6-CM) M1 (`illusions`): an illusion's parent is in the game, on the map or held,
  made before it, no illusion, of its type, a type that casts Mirage; a parent has no more illusions than its Mirage
  makes; an illusion has no lineage; (M6-GH2) L1 (`landings`): an aircraft on a slot, landing or down, is on an
  Airfield in the game, within `Air::landing_reach_su` of one of its slots, rises no more, and rearms exactly while it
  is down and short of ammo; no Airfield has more on it than its 4 slots; a held unit is on none; (M6-GS1) a Supply
  Chopper on a spot is on a Supply Tower or a station in the game, within reach of its spot, alone there, and neither
  rises nor rearms; (M6-SB) N1
  (`tanks`): an object has tanks exactly when its type has one, a tank its type doesn't have is full and off, a toggle
  is on only with a charge left and its trail's newest patch on the ground where it says, a held unit's toggles are
  off, and every patch on the ground is of a kind a type lays, a disc, and not yet gone; (M6-ST1) U1 (`carriers`): a
  passenger is aboard a carrier in the game, on the map or held, not itself, of a type that carries, its own owner's,
  and of a base its hold takes;
  (M5-V2) W1 (`momentum`): a FixedTurnRadiusMovement unit turns in a tick by at most the speed it can have in it (its
  momentum and one tick's acceleration) over its MinimumTurnRadius, never in place; (M5-V3) S1 (`momentum`): a
  SpeedTurnRadiusMovement unit turns in a tick by at most the tread law's rate at the slowest speed it can have in it
  (its momentum less one tick's acceleration or braking). A
  `TestSim` checks them after
  every tick
  and panics with a picture of
  the map (`TestSim::ascii`: units as letters, with a legend). The check is on by default; `check_invariants(false)`
  turns it off for timing runs. A test that steps a bare `Sim` keeps its own `Invariants` and calls `check` after each
  step, as the golden games do (`tests/golden/harness.rs`). The fuzz's games (`tests/robustness.rs`, the seed sweep's)
  check the rules' invariants alone, `Invariants::check_rules` (P1, O1, R1, H1, B1, A1, E1, T1, C1, F1, G1, M1, L1,
  N1, U1): V1 and K1, which
  rebuild the vision and the planning grids, would take the sweep from 5 s to 26 s.
