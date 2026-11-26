# M1 movement, pathfinding and collision: technical design

**Status: approved by the Director on 2026-11-23.** This is binding for M1 movement work, alongside `ARCHITECTURE.md`.
- **Decisions:**
  - The proposed gate change is adopted. Order commands drop ids that no longer exist, and reject only when no unit remains or one isn't owned.
  - f32 stays.
  - Paths are hashed state.
- **The gaps in section 2** went to the Designer as questions:

  | Gaps | Question |
  |---|---|
  | G1–G3 | Q0007 |
  | G4–G5 | Q0008 |
  | G6–G7 | Q0009 |
  | G8–G9 | Q0010 |
  | G10–G12, G14 | Q0011 |

  Q0011 is answered for G10 (design change 0038: resources block), G11 (0039: Rugged is impassable for those
  bases) and G12 (0040 to 0042: path knowledge, below); G14 is still open.

  G13, the queue cap, is technical and set to 32. Until a question is answered, implement the default from the table, behind a named constant or switch that cites its question id.
- **The benchmark** cited in 5.8 was a throwaway spike file and isn't in the repo.
- **Slice D (steering and collision) is implemented.** Where the code differs from the first approved text, the
  text below was changed and the change is marked **(D)** with its reason. The code is `crates/sim/src/movement.rs`
  and `movement/`, `paths.rs` and `mover.rs`.
- **Group destinations follow the magic box** (design change 0021, answering G1): slice M replaced the slots of 6.4.
  Its changes are marked **(M)**. The code is `crates/sim/src/orders/destinations.rs`, `nav::reachable_point`, the
  group field of `nav::serve`, and one steering fix the new group shapes exposed (6.3, `movement/collide.rs`).
- **Friendly yielding, the give-up schedule and occupied targets follow the design** (design changes 0022–0028,
  answering G2–G5 and question Q0015): slice Y built 6.6, changed 5.6 and 6.5, and drew the magic box around the
  silhouettes (6.4). Its changes are marked **(Y)**. The code is `crates/sim/src/movement/yielding.rs`,
  `movement/stuck.rs` and `mover.rs`.
- **Shift-queued group orders follow the design** (design changes 0029 and 0030): slice Q builds the magic box of
  a queued order at the units' planned end positions, and confirmed that destinations are resolved at the click
  and kept per unit (6.4). Its changes are marked **(Q)**. The code is `crates/sim/src/orders/destinations.rs`.
- **Object goals and the behaviours are built** (design changes 0031–0034, 0036 and 0037, answering G7–G9; G6 is
  answered by 0033 and waits for the vehicles milestone): slice E gave `MoveGoal::Object` its meaning (3, 5.6, 6.5)
  and built the behaviours that set goals (`ARCHITECTURE.md > Behaviours`). Its changes are marked **(E)**. The code
  is `crates/sim/src/orders/`, `shape.rs`, `paths.rs` (`chase_check`), `movement.rs` and `movement/stuck.rs`.
- **Crowds and nav performance are hardened** (slice G1; no design rule changed): units seek lanes among the units
  they have to pass (6.3), an idle group parts for a group that walks through it instead of being shoved along
  (6.6), and a group's field is built only as far as the group stands (5.5). Its changes are marked **(G)**. The
  code is `crates/sim/src/movement/collide/lanes.rs` (`World::lane`), `movement/yielding.rs` (`ask_ahead`),
  `movement/space.rs` (the cells' marks), `nav/search.rs` (`Search::field_to`) and `nav/serve.rs`
  (`smooth_shared`).
- **Paths are planned from what the owner knows** (design changes 0040, 0041 and 0042, answering G12;
  `systems/controls/behaviors.md > PathKnowledge`): slice M2-E split the grids into the true ones, which units
  collide on, and one set per player, which its units plan on (5.2, 5.7), and made the staggered check follow the
  owner's knowledge (5.6). Its changes are marked **(K)**. The code is `crates/sim/src/paths.rs` and `paths/`
  (`knowledge.rs` has the rules), `nav/grid.rs` (`NavGrid::restamp`) and `nav/serve.rs` (`serve_on`).
- **DragMovement is built, and idle units that overlap move apart** (slice M3-G; plan decision T25, design
  changes 0035 and 0039 for hover units, and 0126): 6.7 is the model, 6.8 the separation, and I3, I5 and I7 are
  amended in 8. Its changes are marked **(M3-G)**. The code is `crates/sim/src/movement/drag.rs`, `follow.rs`
  (one unit's tick, split from `movement.rs`) and `separate.rs`, `Mover::momentum`, and one steering rule for
  units with momentum in `movement/collide.rs` (`World::around`, the near side).
- **The occupied-arrival radius is 3 GU** (design change 0169, answering Q0032; slice M3-V): 6.5's rule 1, for a
  point and for an object goal alike, and with it a unit walled out of its magic-box spot by its own group (design
  change 0183). Its changes are marked **(V)**. The code is `mover::OCCUPIED_ARRIVAL_GU`.
- **A queueing unit goes round the row in its way** (slice M3-G2, from the M3 playtest: a unit behind its owner's
  firing line never fired): 6.5's rule 2. Its changes are marked **(M3-G2)**. The code is
  `crates/sim/src/movement/round.rs`, `stuck.rs` (`queues_behind`), `Mover::go_round` and `Mover::detour_end`.
- **A queueing unit with no way round gives up** (slice M3-G3; design change 0223, answering Q0057): 6.5's rule
  2, where it waited without limit (since M1 slice E); now the Failure rule applies to it, as to any stuck unit.
  Its changes are marked **(M3-G3)**. The code is `stuck.rs` (`after_move`); M3-G2's wait (`stuck::waits_in_queue`,
  `collide/hold.rs`) is gone.
- **A leg can be exempt from the occupied-target rule** (slice M4-W2; design change 0315, a first step for question
  Q0109): 6.5's rule 1 doesn't apply to an Agent's leg back to its Tunnel's crystal hatch. Its changes are marked
  **(M4-W2)**. The code is `Mover::presses_on`, `stuck.rs` (`target_is_occupied`) and
  `crates/sim/src/orders/gather/legs.rs` (`return_leg`).

Design references are `path/file.md > Section`, with paths relative to `artifacts/designer/design/`.
Every number marked *(placeholder)* becomes a named const tagged with the question that covers it (section 2).

## 1. Requirements from the design

| Topic | Rule | Source |
|---|---|---|
| Time, space | 16 ticks/s; speeds in su/tick; 64 su = 1 tile | `core/scale.md` |
| M1 units | Peacekeeper: LightInfantry, 24×24 su, 4 su/tick (2 with Riot Shield up), Range 4 GU, Sight 5 GU. Guard: HeavyInfantry, 36×36, 5 su/tick, Range 3 GU. Both: TurnRate 180°/tick, infinite acceleration and deceleration, so M1 kinematics are instant | `factions/gdo/units/peacekeeper.md`, `factions/syndicate/units/guard.md` |
| Later range | Silhouettes 24–60 su so far (vehicles TBD, may exceed a tile); speeds 4–8 su/tick; five movement models (TurnRate, FixedTurnRadius, SpeedTurnRadius, Drag, Glider) | `systems/units.md > MovementModel` ff., faction unit files |
| Domains | Ground, Air, Underground per UnitBase | `systems/units.md > UnitBase` |
| Ground collision | Hard, never overlapping; shape is the Silhouette rectangle; **(Y) friendly yielding**: an idle unit takes one short sidestep for a blocked moving unit of its owner (not on Hold, not busy, not if it can't move, not for its own group order); enemies never yield, and moving units never yield or swap; moving units path around occupied space; ground units collide with ground units and structures | `systems/units.md > UnitCollision > Ground Collision` (0022, 0023, 0027) |
| Air collision | None with ground or structures; soft separation among air units (SeparationRadius) | `systems/units.md > Air Collision` |
| Crushing | Moving TrackedVehicle/Mech pass through and kill enemy LightInfantry; not friendly; stationary crushers are solid | `systems/units.md > Crushing` (not M1, but the collision filter must allow it) |
| Sub-tile lanes | A 64 su gap fits two 24 su units side by side, one lane each way | `factions/cults/units/recruit.md` (throughput note) |
| Terrain | Non-Traversible tiles block ground units. RuggedTerrain = true bases (infantry, Mech) cross Rugged tiles. Drill units travel on Drillable tiles | `core/map.md > Tile`, `systems/units.md > UnitBase`, `systems/units.md > DrillUnit` |
| Elevation | Slopes change neither speed nor path cost; cliffs are non-Traversible tiles; corners are shared, so two adjacent Traversible tiles are always connected | `core/map.md > Elevation` |
| Underground | Expansions are walked over by surface units | `factions/syndicate/structures/README.md > Tunnel Expansions` |
| Move | To a location or an object; ends with Stopping, complete when stopped; completes "within proximity" of an object; on repeated failure to progress: stop, complete. **(Y)** Stuck after 24 ticks without getting closer, 3 re-route attempts after waits of 8, 16 and 32 ticks, then give up (0024); an unreachable target becomes the nearest reachable point (0025); stuck within 3 GU of the target behind units that aren't moving is arrived (0026; **(V)** 3 GU since 0169, 2 before) | `systems/controls/command_behaviors.md > MovingToLocation`, `systems/controls/command_behaviors.md > MovingToObject` |
| Chase | AttackingObject approaches until the target is within Range, beyond MinRange and in facing; resumes when it leaves range. Infantry (UnitBaseSource) cannot move during Aiming, Firing, Cooldown | `systems/controls/command_behaviors.md > AttackingObject`, `systems/combat.md > AttackPhase` |
| Range | ±1 GU by elevation level difference, ground and underground only | `core/vision.md > ElevationModifier` |
| AttackMove, Patrol | Leash 6 GU perpendicular to the path, recompute the path on resuming; Patrol swaps legs forever | `systems/controls/command_behaviors.md > AttackMovingToLocation`, `systems/controls/command_behaviors.md > Patrolling` |
| Idle | Auto-acquire in SightRange, leash 4 GU from IdleOrigin, return to it | `systems/controls/behaviors.md > BaseAutoTargeting` |
| Hold, Stop | Hold: Locomotion Stationary always, never completes. Stop: Stopping, then no behaviour | `systems/controls/command_behaviors.md > HoldingPosition`, `systems/controls/command_behaviors.md > StoppingBehavior` |
| Queue | Shift appends, otherwise the command replaces the whole queue; next order starts when the current behaviour completes | `systems/controls/commands.md > Unit Command` |
| Channels | Locomotion (Moving, Reversing, Stopping, Stationary) and Orientation (Turning, Maintaining) are named state; combat and crushing read them | `systems/controls/action_channels.md > BaseActionChannels`, `systems/combat.md > LocomotionOrientationConstraints` |
| Group moves | **(M)** The magic box: a location order (Move, AttackMove, Patrol) to several units keeps their arrangement when the click is outside the bounding box of their silhouettes (**(Y)**, 0028; each goes to the click plus its offset from the group's centre) and converges on the click when it is inside. No formation movement. Selection has no size limit | `systems/controls/commands.md > Group Destinations (Magic Box)`, `systems/controls/selection.md` |

## 2. Design gaps and contradictions

Top five first. Each default is what this design implements until answered.

| # | Question | Proposed default |
|---|---|---|
| G1 | Several units are ordered to one point and cannot all stand on it. Where does each end up? | **(M) Answered by 0021** (Q0007, part 1): the magic box, 6.4. The default proposed here, a compact lattice of slots around the point, was rejected. Units move at their own speed, not the slowest's |
| G2 | **Contradiction with the Director's brief**, which assumes idle units make way. `systems/units.md > Ground Collision` says they don't. Confirm: no pushing or yielding at all, even for own units blocking a chokepoint? | **(Y) Answered by 0022, 0023 and 0027**: idle own units yield with one short sidestep (6.6). The default here was "no pushing" |
| G3 | With hard collision and no yielding, two units meeting head-on in a corridor too narrow to pass (two Guards, 72 su, in one tile) deadlock. Is "both give up and stop" intended? | **(Y) Answered by 0022**: yes, both give up; moving units never yield or swap |
| G4 | "Repeatedly fails to make progress" has no numbers. | **(Y) Answered by 0024** with these numbers: stuck after 24 ticks without progress; at most 3 repaths per order after waits of 8, 16 and 32 ticks; then fail (about 7 s in total). The schedule is 5.6 |
| G5 | Move target that cannot be reached (cliff, water, inside a structure, cut-off area), or whose point is occupied by a unit that won't move. | **(Y) Answered by 0025 and 0026**: go to the reachable point nearest the target and complete there; arrived when stuck within **2 GU** of the goal behind units that aren't moving (6.5, rule 1); **(V)** 3 GU since 0169 |
| G6 | Does the Silhouette rectangle rotate with facing? All specified ones are square. | **(E) Answered by 0033**: it rotates with the facing. Not built: it waits for the vehicles milestone, with oriented collision. Until then boxes are axis-aligned, and `shape.rs` is the one place that changes |
| G7 | How are Range and "within proximity" measured for large targets? | **(E) Answered by 0031 and 0032**: edge to edge, the gap between the two shapes (`shape::Shape::gap`), for Range, MinRange and proximity; proximity is a gap of 8 su. The default here was centre to nearest point |
| G8 | The attack target leaves the attacker's owner's vision during a chase. | **(E) Answered by 0034**, as proposed: go to its last seen position, then the order completes. A Follow of an enemy ends the same way. **(M3-L3) Replaced by 0200** (answering Q0051): the Attack or Follow completes at once, where the unit stands |
| G9 | Patrol: is the origin the position at issue or when a Shift-queued Patrol starts (`systems/controls/commands.md > Patrol` vs `systems/controls/command_behaviors.md > Patrolling` step 1)? What if a leg fails? | **(E) Answered by 0036 and 0037**, as proposed: when it starts executing. A failed leg counts as arrived; two failed legs in a row end the Patrol |
| G10 | Do map resources (patch, station) block ground units? "Structures" doesn't name them. | Yes, their footprints block |
| G11 | RuggedTerrain = false: is a Rugged tile impassable for that base? Implied, never stated. | Yes, impassable |
| G12 | Do units path with knowledge of structures their owner has never seen? | **(K) Answered by 0040, 0041 and 0042**: no. Paths are planned from what the owner knows: terrain, its own structures, others' structures once seen (a wall at the last-known position until seen otherwise) and the map's resources from the start (5.2, 5.6, 5.7). The default here was the true map |
| G13 | Command queue length. Orders queued behind Patrol or Hold never run. | Cap 32 *(technical)*; the rest as written |
| G14 | Underground units: do they collide with each other? Can air cross every tile? | Not M1. Assume hard collision among themselves; air crosses everything inside the map |

Note for the Designer, not a question: `MovingToLocation` describes a Plan of channel pairs with an ExpectedPosition
and "recompute on deviation". Under hard collision units deviate constantly, and replanning on every deviation is the
repath storm the prototype died of. This design keeps the named channels as state but replans only by the rules in 5.6.

## 3. Sim data model

Two capabilities, each `Option` on `Object` with its own module (`ARCHITECTURE.md > Object model`), both fully hashed
and serialized. Paths are **state, not cache**: a save loaded mid-game continues bit-identically.

```rust
// defs.rs: ObjectDef gains `unit: Option<UnitDef>`; one static UnitBaseDef per UnitBase (systems/units.md > UnitBase).
pub struct UnitBaseDef { domain: Domain, rugged_terrain: bool, crushable: bool, can_turn_in_place: bool, can_reverse: bool, .. }
pub struct UnitDef { base: UnitBase, movement: MovementAttrs /* enum per MovementModel; None accel = infinite */ }
pub enum Domain { Ground, Air, Underground }             // also the collision layer
pub enum NavClass { Ground, GroundRugged, Underground }  // passability predicate; air needs none
// size class: u8 = ceil(max(silhouette w, h) / 64 su); 1 for every unit specified so far.

// orders.rs: the command queue and behaviour state (commands.md, behaviors.md)
pub struct Orders { queue: VecDeque<Order> /* cap MAX_QUEUED_ORDERS */, current: Option<ActiveOrder>,
                    auto: Auto /* (E) what the unit does on its own without an order */ }
pub struct ActiveOrder { order: Order, since: Tick, state: Behaviour /* (E) the behaviour's working state */ }
pub struct Order { kind: OrderKind, group: OrderGroup /* units of one command share it */,
                   slot: Option<Vec2> /* (M) this unit's own destination by the magic box (6.4); None: the click */ }
pub enum OrderKind { Move(Target), Attack(ObjectId), AttackMove(Vec2), Patrol(Vec2), Hold, Stop }
pub enum Target { Point(Vec2), Object(ObjectId) }  // the point is the click

// mover.rs: goal in, motion out. Behaviours write `goal` and read `status`; nothing else moves a unit.
pub struct Mover {
    goal: MoveGoal,            // None | Point(Vec2) | Object { id, reach: f32 } (E): reach is a gap between shapes
    status: MoveStatus,        // Idle | Moving | Arrived | Failed | TargetLost
    locomotion: Locomotion, orientation: Orientation,   // the design's channels
    velocity: Vec2, speed_factor: f32,                  // Riot Shield: 0.5
    group: Option<u32>,                                 // (D) the order group of this goal, kept after it ends (6.5)
    path: Vec<Vec2>, next: u16, path_state: PathState,  // PathState: None | Wanted { since: Tick } | Ready
    nav_version: u32, planned_target: Vec2,             // staleness checks
    repaths_left: u8, last_repath: Tick /* (Y) */, avoid_tiles: Vec<TilePos> /* cap 16 */,
    best_remaining: f32, last_progress: Tick, blocked_ticks: u16,
    avoid_side: Side, avoid_until: Tick,
    sidestep: Option<Sidestep>,                         // (Y) the sidestep of a yielding unit (6.6)
}
```

- **Collision shape**: `half = silhouette / 2`, an axis-aligned box at `pos` (G6). Two boxes overlap only if
  `|dx| < hw_a + hw_b && |dy| < hh_a + hh_b`, strictly: touching is free. Contact positions are *assigned*
  (`obstacle edge ∓ half`), never accumulated, so resting units sit at exact values and don't drift.
- **Derived, not hashed** (`#[serde(skip)]`, rebuilt on load and on change, excluded from `PartialEq`): the `NavGrid`s
  and their component labels. **Scratch** (rebuilt every tick, carries nothing across ticks): the spatial grids and
  the A* arrays. A save/load test (section 8) proves neither leaks into behaviour.
- `Sim` gains `next_group: u32` and `nav_version: u32` (hashed). **(K)** And `path_knowledge` (hashed and
  serialized: history): per player, the map resources it still assumes and the version of its knowledge.
  `Mover::nav_version` holds the owner's knowledge version the path was planned on (the true grids' `nav_version`
  for a unit without an owner).
- **Commands**: `Command::Order { units: Vec<ObjectId>, kind, queue: bool }`. Gate: owner check per unit, target
  finite and on the map, at most `MAX_ORDER_UNITS`, duplicates dropped. **Proposed gate change for the Director**:
  ids that no longer exist are *dropped* rather than rejecting the command (a unit can die in the tick the order
  arrives; under lockstep delay this is routine). Reject only if no unit remains or one isn't owned.
- **Spawn and teleport** must keep the invariants: `SpawnProblem::Blocked` / `Occupied` when a unit's box would
  overlap a blocked tile or another unit of its layer, in scenario validation, debug `Spawn` and `SetPosition`.

## 4. Movement in the tick pipeline

Phases 1–4 stay as they are. Order commands (phase 4) only write `Orders` queues and compute group destinations
(6.4); they never move anything. Then, each walking units in id order over `..first_new`:

| Phase | Does | Why here |
|---|---|---|
| 5a `orders::update` | Pops completed orders, runs the behaviour state machine, writes `mover.goal`, reads `mover.status` and last tick's positions and visibility. **(E)** Every behaviour reads one snapshot of the objects, taken before any of them runs (`orders::world::World`) | After commands, so an order acts in its own tick; before movement, so there is one writer of goals |
| 5b `paths::sync` | If a tile-blocking object changed: rebuild the **true** NavGrids and components, bump `nav_version` **(K)** | The movement below must collide with this tick's obstacles |
| 5c `paths::serve` | Runs the staggered path checks (5.6, 6.2), then serves `Wanted` requests within the budget (5.5). **(K)** Each unit on its owner's known grids, as of the end of the last tick | After goals, before steering; a repath a check asks for is served in the same tick |
| 5d `movement::step` | **The one movement pipeline**: steer, resolve collisions, commit `pos`/`facing`/`velocity`/channels, set `status` | Single place that writes `pos` |
| 5e combat | Attack phases, hit list, damage, deaths (other slice) | Shoots from post-move positions |
| 5f vision | Visibility per player (other slice) | Sees final positions |
| end: path knowledge **(K)** | `PathKnowledge::observe`, then `paths::sync_known`: each player whose known footprints changed gets a new knowledge version and its grids rebuilt | After the vision and its records, which it follows from; once per tick, so a player's grids are rebuilt at most once per tick and between ticks they follow from the state alone |

- 5d builds the spatial grids first from **all** units, including this tick's spawns (obstacles, though not updated),
  then moves units one by one, updating the grid at each commit. A unit tests against movers already at their new
  position and the rest at their old one. Sequential commit makes "no overlap" hold by construction, and id order
  makes it deterministic. The grid holds copies (`id, box, velocity, stationary, owner`), so no second borrow of the
  object map is needed.
- `status` set in 5d is read by 5a next tick. A Shift-queued unit that arrives at tick `t` requests, receives and
  follows its next path at `t + 1`, so it never stands idle between waypoints.
- **Events**: `MoveFailed { unit }` only. Indicators and paths are read from state. Per-tick counters (expansions,
  searches, collision queries) are output outside the hash, for tests and the perf overlay (`Sim::nav_counters`,
  `Sim::move_counters`).

## 5. Pathfinding

**5.1 Algorithm.** Two levels. Global: tile A* over static obstacles, then any-angle smoothing. Local: steering and
collision against units (section 6). Rejected: a sub-tile global grid (1M cells on 256², too slow), navmesh (building
changes are costly, hard to verify), hierarchical/portal graphs (unneeded at these sizes, see 5.7; revisit only if
`MAX_MAP_SIDE` grows), flow fields as the *steering* model (tile-quantised directions, poor with hard collision).
A Dijkstra field is used only to mass-produce paths for group orders (5.5).

**5.2 NavGrid.** One per `(NavClass, size class)` in use: a `blocked` bit per tile plus a component label per tile
(flood fill, row-major seeds, so labels are deterministic).
- `Ground`: blocked if not Traversible, or Rugged (G11), or under a blocking footprint. `GroundRugged`: not
  Traversible, or under a footprint. `Underground`: not Drillable. Blocking footprints: surface structures and
  resources (G10), never underground expansions. Slopes never matter.
- **(K) Whose footprints.** The sim keeps the **true grids** (every blocking footprint) and one set of **known
  grids per player** (`systems/controls/behaviors.md > PathKnowledge`): the footprints the player owns, those of
  its vision's records (seen, and not seen gone: `core/vision.md > Explored`), and the map's resources it hasn't
  seen the place of yet, which it assumes to be where the map put them (design change 0042). Planning reads the
  owner's known grids: path requests, the staggered checks, a chase's line of sight, `reachable_point` for
  destinations and object goals. Collision reads the true grids: the movement step (section 6), the spawn and
  teleport checks, invariant I2. A unit without an owner plans on the true grids. A player's grids are built
  lazily, with its first unit that walks on one.
- Size class `k > 1`: a tile is passable if the `k × k` block anchored on it is (true-clearance); M1 only builds `k = 1`.
- Map edge: a unit's box stays inside the map; tiles outside are blocked.

**5.3 A\*.** 8-connected; a diagonal step needs both orthogonal neighbours free, so nothing squeezes between
diagonal obstacles. Integer costs 10 and 14, octile heuristic, all `u32`: no float enters the search.
- **Closed set and best-cost map**: per-tile `g`, `parent`, `stamp`, `closed` arrays with a generation counter
  (no clearing). A node is expanded once; a neighbour is relaxed only on a **strictly** lower `g`, so equal-cost
  parents are never overwritten (the zigzag bug).
- **Tie-break**: heap key `(f, h, tile index)`, neighbours in a fixed order (E, S, W, N, SE, SW, NW, NE).
- **Unreachable goals never search**: if the goal tile is blocked or in another component, first retarget to the
  tile of the unit's component nearest the goal (ring scan outward from the goal, ties by tile index). So no search
  can fail, and none floods the map.
- **Avoid tiles**: a repath treats the unit's `avoid_tiles` as blocked (6.5). Only this search can fail. **(D)**
  When it fails, the unit drops its avoid tiles and plans again in the next tick instead of failing its move: a
  tile is 64 su and a unit 24, so avoid tiles overstate what is blocked and can't prove that there is no way. The
  smoothed path and the steering target also stay out of the avoid tiles (`nav::smooth_avoiding`,
  `nav::segment_avoids`); string-pulling by terrain alone pulled every detour straight again.

**5.4 Smoothing and line of sight.** `nav::los(class, half, a, b)`: walk the tiles the segment crosses
(Amanatides–Woo) and slab-test the segment against each blocked tile in their 3×3 neighbourhoods, inflated by `half`.
Exact for boxes up to one tile. Smoothing is greedy string-pulling: from an anchor, keep the farthest tile centre
with LOS. Waypoints are tile centres, plus the exact start and goal, which leaves `32 − half` su of corner clearance.

**5.5 Requests and budget.** `PathState::Wanted { since }` on the unit; each tick 5c sorts requests by `(since, id)`
and serves them until `PATH_BUDGET` = 32 000 node expansions is spent *(tuning const)*. A job starts while budget
remains and always finishes, so every request is served within bounded time. Units waiting for a path stand still.
- **Group orders**: when at least `GROUP_FIELD_MIN` = 4 requests share an `OrderGroup`, nav class and component, run
  **one** Dijkstra from the group's anchor tile (same code, `h = 0`), and give each member the parent chain from its
  own tile, smoothed, with its own destination (6.4) as the last waypoint. The field is scratch, dropped in the same
  tick. **(M)** The anchor is the click's tile, and only requests whose goal the anchor tile's centre sees (for the
  unit's box) count and share the field; the others plan alone with A\*. With the magic box a destination is the
  unit's place in the group's shape, which can lie behind an obstacle from the click (on the near side of a wall
  the click is beyond, say), and the way through the anchor would be a detour. A converging group has every goal on
  the anchor's tile, and a group moved over open ground sees all of its goals, so both still cost one field.
- **(G) The field reaches only as far as the group stands** (`Search::field_to`). The search from the anchor heads
  for the bounding rectangle of the members' tiles (the heuristic is the octile distance to it, which is
  consistent, so an expanded tile has its final cost and parent) and stops when every member's tile is expanded.
  The paths cost what the whole field's would. A group standing together pays for the tiles between itself and
  the anchor, not for the map: the two 250-unit orders of the perf test cost 9 000 expansions each, where the whole
  field was 62 000, and both now fit into one tick's budget. Units of an AttackMove group that resume together
  after a fight no longer cost a field over the whole map (4.2 ms in a 250 v 250 battle; now 0.7 ms at worst).
  Members scattered all over the map still cost the whole field, the bound of 5.8.
- **(G) The members share their smoothing** (`nav::serve::smooth_shared`). The paths of a field are chains of one
  tree, so from a tile on two members walk the same tiles, and the waypoint that follows a tile's centre depends
  on that chain alone while the member's own goal is out of sight. It is found once per tile for the whole group.
  The waypoints are exactly those of `smooth`.
- The budget counts expansions, never time, so it is deterministic. **(G)** Its semantics are unchanged: a job
  starts while budget remains and always finishes, and a field is one job.
- **(G) Still open**: a member whose goal the anchor can't see plans alone, within the budget. A group ordered into
  cluttered ground, where the anchor sees few of the goals, is therefore pathed over several ticks, not in the
  tick of the order (the last case `nav_perf` prints: 4 of 200 served in the first tick on its bad-case map).

**5.6 Repath policy.** Per order leg, `repaths_left = MAX_REPATHS` (3, design change 0024). **(Y)** Every counted
repath waits `REPATH_WAITS` (8, 16, 32 ticks) since the unit became stuck or since the repath before
(`Mover::last_repath`), whichever is later.

| Trigger | Check | Counts against the cap |
|---|---|---|
| Stuck (6.5) | every tick, cheap | yes; with none left, `Failed` 24 ticks after the last one |
| **(K)** The owner's knowledge version changed (`PlayerKnowledge::version`; `nav_version` for a unit without an owner) | once, on the unit's stagger tick: LOS along the remaining path on the owner's known grid; repath only if a segment is now blocked. This is the re-route on discovery: the unit plans again within 4 ticks of its owner seeing an obstacle on its way | no (bounded by how often the knowledge changes) |
| Chase target moved **(E)** | on the stagger tick (`paths::chase_check`): with a clear line for its box, the unit's path becomes the one leg straight to the target, no search; without one, a new plan once the target is farther than `max(1 GU, 25 % of the distance)` from `planned_target`, at most every 8 ticks | no (rate-limited) |
| Lost LOS to the steering waypoint | on the stagger tick, after being pushed off the path by sidesteps, **(Y)** and only when a stuck repath is due (the schedule below; slice D had 12 ticks without progress) | yes |

Stagger tick: `(tick + id) % 4 == 0`. While a chaser has LOS to its target it steers straight at it with no path.

**(E) Object goals.** A path to an object is planned to the point nearest its centre that the unit can stand on
(`nav::reachable_point`, as for an unreachable point, 6.4): for a structure or a map resource, whose own tiles are
blocked, that is right at its wall, within proximity. `planned_target` notes where the object's centre was. The unit
has `Arrived` when the gap between its shape and the object's is within the goal's `reach` (checked before it
moves); at the end of its path if the object is still where the path was planned to (the nearest point it can
reach: design change 0025); or by rule 1 of 6.5. At the end of a path to an object that has moved on, it plans
again (not counted). An object that is gone ends the move `TargetLost`.

**(Y)** Timing, as design change 0024 words it (`systems/controls/command_behaviors.md > MovingToLocation > Failure`). A
unit is **stuck** when 24 ticks have passed without progress (`Mover::is_stuck`; nothing restarts that clock but
progress). A stuck unit waits 8 ticks and tries another route, waits 16 and tries again, waits 32 and tries a third
time; if it is still stuck 24 ticks after the third attempt, it gives up (`Mover::recovery_due`). So a unit that
never gets on repaths 32, 48 and 80 ticks after its last progress and fails at 104 ticks (`GIVE_UP_TICKS`, 6.5 s:
the design's "about 7 s"). A unit that gets on after a repath is stuck again 24 ticks after its last progress, and
its next wait runs from then. Slice D had repaths at 24, 48 and 72 ticks, each restarting the stuck clock, with the
same 104. A unit blocked for less than 24 ticks never repaths. A unit that lost sight of its waypoint but is still
getting on slides back by itself and keeps its repaths.

**5.7 Dynamic obstacles.** A building placed or removed sets a dirty flag; 5b rebuilds grids and components in full
(65k tiles, about 1 ms, and rare). Units are never in the NavGrid. Stationary units reach the search only
through `avoid_tiles`.

**(K)** With path knowledge one death can change five sets of grids (the true ones and four players'), so a
rebuild had to get cheaper. A grid keeps its terrain's part and is stamped again with the footprints
(`NavGrid::restamp`), and the components are labelled by runs of free tiles with a union-find, which gives the
same labels as the flood fill (first tiles in row-major order). The true grids are rebuilt in 5b, and again after
a death in 5e. Each player's known grids are rebuilt at the end of the tick, when the set of footprints it knows
changed, so at most once per player and tick; a map resource seen for the first time changes nothing (the same
footprint, now remembered instead of assumed). During a tick a player's grids are those of the end of the tick
before: an own structure placed by a command joins them at the end of its tick. **Measured**
(`tests/nav_perf/`, release, 256 × 256): one grid restamped in 0.17 ms on the bad-case map of 5.8 (1.4 ms for
the build it replaces); with 4 players, 412 units (400 on the move) and 160 patches, the tick in which ten
Generators go in sight of all four players rebuilds 5 sets of grids and takes 1.0 ms in all, the worst tick of
the units' path checks after it 1.3 ms, against 0.27 ms for the ticks before.

**5.8 Performance.** Measured with a throwaway benchmark (`scratchpad/m1/bench/astar.rs`: this exact A\* on 256×256,
10 % random obstacles plus six map-high walls with one gap each, a deliberately bad case):

| Case | Expansions | Time (opt 3 / dev profile) |
|---|---|---|
| Open map, corner to corner | 256 | 19 µs / 26 µs |
| Pairs within 30 tiles | 2 200 avg | 160 µs / 190 µs |
| Random pairs, avg path 270 tiles | 15 000 avg, 49 000 max | 1.4 ms / 1.8 ms |
| Full Dijkstra field | 65 536 | 3.0 ms / 3.5 ms |
| Unreachable goal without the component check | 65 531 | 6.1 ms / 7.2 ms |

About 70–120 ns per expansion. Consequences: a 200-unit group order costs one field (3 ms), not 200 searches
(up to 280 ms); the component check is mandatory; a hierarchy is not needed at 256².
**Targets** (release, 500 moving units, 256×256): movement phase ≤ 1 ms; path serving ≤ 3 ms in a normal tick and
≤ 8 ms worst (budget plus one field); a group order of any size is pathed in the tick it is issued; whole sim tick
≤ 10 ms worst, leaving 6× fast-forward. Tests assert the deterministic counters, not wall time.
**Measured after slice D** (`crates/sim/tests/movement_perf.rs`, release, two armies of 250 crossing a 256×256 map):
the whole sim tick takes 0.19 ms on average and 0.64 ms at worst with 450 or more units moving (2.7 sweeps and 5.9
box tests per unit and tick); the tick of a 250-unit group order (one field, 62 000 expansions, 250 smoothed paths)
takes 6 to 10 ms.
**(G) Measured after slice G1** (the same test, and `behaviours_perf.rs`; before → after):

| Case | Before | After |
|---|---|---|
| Group orders of the two armies of 250 | 7.1 ms and 6.1 ms, in two ticks (62 000 expansions each) | 5.9 ms, both in one tick (18 700 expansions) |
| Whole tick, 450 or more of 500 units moving, the armies crossing | 0.365 ms mean, 0.70 ms worst | 0.40 ms mean, 0.79 ms worst |
| 500 units attack-moving, up to the first shot | 0.35 ms mean, 0.41 ms worst | 0.33 ms mean, 0.38 ms worst |
| 250 v 250 AttackMove battle | 0.48 ms mean, 4.2 ms worst (65 700 expansions) | 0.28 ms mean, 0.67 ms worst (940 expansions) |
| The winners going on | 3.5 ms worst | 0.07 ms worst |

Lane seeking (6.3) costs the crossing armies 10 % of their tick: each unit looks through the cells ahead of it for
units coming its way. The cells' marks (6.1) keep that to one look at a few cells for a unit that has nobody to
pass, and the grid's hot path no longer calls `floor`.

## 6. Collision and steering

**6.1 Layers.** One spatial grid per `Domain`, cell = one tile, a unit listed in every cell its box overlaps.
Ground: hard, vs ground units and the NavGrid of its class. Underground: its own layer (G14). Air: no hard
collision; soft separation is a velocity bias inside SeparationRadius (not M1). A filter
`blocks(mover, other) -> bool` is the single place for exceptions (crushing later).
**(H2)** Two exceptions since slice M3-H2 (design change 0106), both read off the two bodies, no pair state: two
Drones on a Gather job don't block each other; and two units whose boxes already overlap as the mover's turn
begins ignore each other until they are apart, with no push-apart (`ARCHITECTURE.md > Movement and collision >
Collision exceptions`).
**(G)** A cell also notes the **kinds** of unit listed in it since the grid was built: the 45° sector of each
one's heading, and whether it yields (`Body::kinds`, `Layer::near_any`). A mark is never taken back within the
tick, so it means "maybe": lane seeking (6.3) asks the marks first and looks at the bodies only where a unit
coming its way, or an idle unit that would make way, may be. And `NavGrid::open_around` says whether a tile and
its eight neighbours are free of solid tiles: there a unit's box and its step are clear of the tiles without a
look at them.

**6.2 Steering target.** The farthest path waypoint with LOS, advanced on the stagger tick; a waypoint is never
required to be touched. This keeps a group's lateral spread through corridors instead of funnelling every unit
through the same tile centres. **(D)** The stagger tick scans forward from the current waypoint (at most 8), and
every tick a waypoint within one step is passed, so a unit rounds a corner without stopping on it.

**6.3 One unit's step.** `step = max_speed × speed_factor`; `d` = vector to the steering target, clamped to `step`.
- **Arrival**: if the distance to the final goal is ≤ `step`, set `pos = goal` exactly and stop. The arrival
  radius is one tick of travel by construction, so overshoot is impossible. A finite-deceleration model starts
  braking at `v² / (2 · decel) + v`. **(M3-G)** DragMovement's `step` is its approach speed (6.7), which falls to
  the distance itself next to the goal: the same rule lands it.
- **(G) Lane seeking** (`World::lane`, `World::lane_step`), before the sweep. Two dense groups that met head-on
  jammed: 14 of 36 Peacekeepers in blocks 40 su apart gave up, 18 with two owners. Both groups are moving, so
  nobody yields (design change 0022), and the rules below only act on contact, one blocker at a time. Now a unit
  looks ahead for the units it has to pass and steps towards a lane between them, as pedestrians do:
  - **Whom it has to pass**: units within 4 of its own lengths ahead (`LANE_LOOK`; 96 su for a Peacekeeper) and
    nearer than the end of its way, that are **oncoming** (heading against its own by more than 120°, by where
    they are going, `Body::heading`, not by their latest step; moving or jammed), or **idle units that yield to
    it** (6.6). The object it heads for doesn't count, nor do units beyond its goal: units converging on one
    point aren't in each other's way.
  - **Lanes**: each such unit bars a range of sideways shifts of the mover, those at which the two boxes would
    meet, plus `LANE_MARGIN_SU` = 2 su of clearance. With the lanes that hang together around the unit:
    a **clear lane** within 3 of its widths (`LANE_REACH`) is taken, the nearer side, and on a tie (4 su) its
    `avoid_side`, so head-on units still pass on the Right. With **none within reach** (a dense group ahead) it
    takes the middle of the widest gap between two neighbouring lanes, a nearer gap before a slightly wider one
    (`LANE_NEAR`).
  - **The step**: forward and sideways at once, at most 45° off its way; if that isn't free, sideways alone,
    which counts as blocked like a step along a face. If neither is free, the rules below apply. The side taken
    becomes its `avoid_side`.
  - **Why dense groups get through**: the units of both groups aim at the gaps of the other, so the groups mesh
    like combs. The gaps are too narrow at first (16 su for 24 su units), but the units on the outside find a
    clear lane and move out, the gaps next to them widen, their neighbours centre themselves again, and the comb
    spreads until every lane is clear. Collision stays hard throughout: every step is committed through
    `World::is_free` like any other.
  - **Deterministic**: the lanes are sorted by their bounds, then by body index (bodies are in id order); the
    cells are visited in a fixed order; every decision compares `f32`s made with `+ − ×` only (the heading test
    is in squares, without a root); nothing is kept between ticks except `avoid_side`, which was state already.
  - **One idle unit alone** in the way is no lane: the unit walks on and asks it to step aside (6.6), as before.
  - **Results** (`movement_crowds.rs`): 18 v 18 Peacekeepers 40 su apart: 0 of 36 fail
    (14 before; 18 with two owners). Over 192 layouts of two blocks (Peacekeepers and Guards, three shapes, gaps
    of 8 to 24 su, four offsets, one and two owners) 1.4 % of the units fail, 8 of 50 at worst, where 22 % did,
    and 40 of 50 at worst. The soak's failed moves fall from 223 to 81 of about 1 080.
  - **(M4-W3) With the contact step of design change 0323** (rule 4 below; lane seeking itself is unchanged): 18 v
    18 Peacekeepers 0 of 36 fail, done in tick 234 where it was 190; over the 192 layouts 2 of 5,856 units fail
    where 7 did (0.03 %; 1 at worst in a layout); 5 × 5 Guard blocks 16 su apart lose none (1). The soak's failed
    moves are 9, 0 and 2 for its three seeds where they were 68, 2 and 56, with 134, 394 and 400 moves arrived
    (181, 506 and 476).
  - **Known limits**: in a passage too narrow for two lanes nothing changes: head-on movers fail there, by
    design. **(G)** Blocks of Guards lost a few units, but never in the crossing: none of the sweep's failures
    has an oncoming unit near it. The crossing reorders a block, and its late units find their places walled
    off by the group's earlier arrivals, which don't make way (0027), with 16 su between two Guards and a Guard
    36 su wide; stopped farther than 2 GU from their places, they gave up (6.5, rule 1). Their re-route now goes
    round the whole group (6.5, rule 3): over the 192 layouts 0.7 % of the units fail, 4 of 50 at worst, and
    Guard blocks of 5 × 5 lose 20 of 800 units where they lost 38 (3 × 6: none, where 5). What is left are places
    the group closes in on every side: no route reaches them, and nor does 0026's placeholder radius of 2 GU.
- Sweep the box along `d` and find the first blocker (earliest contact; tiles before units, then lowest id):
  1. **Nothing**: commit `d`.
  2. **Tile or structure**: slide. **(D)** Both axis components `(d.x, 0)` and `(0, d.y)`, the larger first, each
     as far as tiles and units allow (the usual per-axis resolution; "try one, then the other" lost the second).
  3. **Unit moving the same way** (`velocity · d > 0`): follow. Advance to contact and wait; no detour.
  4. **Unit stationary or oncoming**: go around it. **(D)** Boxes are axis-aligned, so "sideways" is along the
     blocker's face: `d` turned 90° leans into the blocker whenever `d` isn't square to the face, and is then never
     free. `d` splits into a part straight at the face and a part along it, its lean:
     - *Glance* (the lean is at least half of `d`, squared): slide along the face, the lean alone.
     - *Leaning* (the lean is at least 30 % of `d`): go around on the side the unit leans to, by the **contact
       step** that way (below). Never a step back along the face. **(M)** The side taken becomes `avoid_side`. The unit's own step along the face can drop its lean below 30 %, and head-on with
       the other `avoid_side` then stepped it straight back, for good: a unit pressed against an idle Guard by the
       crossing lanes of two magic-box groups failed so.
     - *Head-on*: the **contact step** towards `avoid_side`. `avoid_side` starts **Right** for everyone on every
       leg, so head-on units pass like traffic with no randomness. Switch side only if the step that way can't
       begin and `avoid_until` has passed; a switch commits for 16 ticks (no left-right flutter). A blocker that itself moves along its face is passed *behind*: once `avoid_until` has passed, the
       side switches to the one the blocker comes from. Without this an eastbound unit and a crossing or
       side-stepping one walked side by side to the map's edge.
     - **(M3-G) The near side, for a unit with momentum at a blocker that stands**: it goes round on the side of
       the blocker's middle it is on, the shorter way, whatever its lean; `avoid_side` decides only dead in the
       middle; if that side is shut it takes the other and commits for 16 ticks. Slice M3-H2 found a gatherer
       stepping to and fro in front of an enemy's Drone parked on its line, for good: its target lay so close
       behind the Drone that every step aside turned its lean back across the face. Seen from the blocker's
       middle the side never changes. It is for units with momentum only: one that stops within the tick loses
       a few steps by the lean rule (and the goldens of M1 and M2 hold it to that rule), one with momentum lost
       all its speed at every turn and delivered one load a minute. Now it delivers 9 to 11 where the free
       strip gives 12, wherever the Drone is parked (`tests/gathering_pass_through.rs`).
     - **(M4-W3) The contact step** (design change 0323, `systems/units.md > UnitCollision > Ground Collision`:
       "for **infantry (LightInfantry, HeavyInfantry), hover vehicles (HoverVehicle, HoverCraft) and
       TrackedVehicles**, the step is at **90°** to its heading (straight sideways), and its length is **the
       unit's own width** (the size of its Silhouette across its heading), so it clears the lane it was in";
       `movement/collide/contact.rs`). Until 0323 the step was `d` turned 45°, which got the unit on, else one
       tick's step along the face. Which step a base takes is data on its def (`UnitBaseDef::contact_step`):
       `Sideways` for the bases 0323 names and for the Mech (0325; its turn to the side first isn't built, there
       is no Mech) and for the DrillUnit (0338); `Diagonal`, the step as it was, for the WheeledVehicle until slice
       M5-V2 builds its "turn and graze". The technical decisions, each the simplest
       predictable rule (0223):
       - **Straight sideways is along the blocker's face**, as every sidestep here: for a unit that meets the face
         squarely that is 90° to its heading; the quarter turn of a leaning heading leans into the blocker on one
         side and is never free there. **Its own width** is its box's extent along that face. (Slice M5-V1's
         oriented Silhouettes will measure both across the heading itself.)
       - **Over several ticks** (`Mover::step_aside`: the axis, and the coordinate the step ends on, fixed when it
         begins; hashed and saved only while set). Each tick the unit moves its speed towards that coordinate,
         the last part exactly onto it, and nothing else: no step forward and no lane seeking, even if the way
         ahead comes free meanwhile. An Agent takes 6 ticks over its 36 su, a Peacekeeper 6 over its 24, a Guard 8,
         a Recruit 6. A unit with momentum thrusts towards each part (6.7) and has made the step once it is on the
         coordinate or past it.
       - **It begins only if its first part is wholly free**, as every candidate of this rule; else that side is
         shut, and the sides are tried as they were (Right first, the other once `avoid_until` has passed).
       - **Partly blocked, it ends there**: a part that isn't wholly free is taken as far as tiles and units allow,
         and the step is over. From the next tick the unit steers as ever, and a contact begins a new step.
       - **It is a step aside** like the one it replaces: it doesn't get the unit on, `blocked_ticks` counts its
         ticks (rule 5; not for a unit with momentum that moves), and the stuck clock runs (6.5). The unit still
         in the way ahead, if there is one, stays the unit's blocker for the stuck rules, and is asked to yield
         (6.6).
       - **A step commits its side** for 16 ticks, as a switch of sides always did: it becomes `avoid_side`, and
         until `avoid_until` the unit takes its next step to the same side whatever its lean ("to its right first,
         switching sides after 1 s"). A step of a whole width can leave the unit just short of clear of a blocker
         that stands a little to that side, leaning back the way it came; by its lean it then stepped straight
         back, to and fro for good (a Guard between two of its firing line). A step that began is finished whatever
         `avoid_until` says.
       - **At a blocker that stands, the near side** (the rule below, which slice M3-G made for the units with
         momentum, now for every base that takes the contact step, for the reason given there: seen from the
         blocker's middle the side never changes as the unit steps). `avoid_side`, Right at first, decides dead
         in the middle. At a blocker that moves the sides are chosen as before: the lean, `avoid_side` head-on,
         the pass behind.
       - **Measured against the other choices of side** at a standing blocker (the Agents' 30 layouts; each
         with the commitment): the lean when the unit leans and Right first head-on, as before 0323, leaves the
         hatch locked up at 1 grid unit to its side (1.2, 0.0 and 4.2 Space Crystals per second with 4, 5 and 6
         Agents); so does the lean with the near side head-on (1.3, 0.0, 0.3). The near side when the unit leans
         with Right first head-on takes in 11.2 or more everywhere, and so does the near side throughout (10.7 or
         more). Of the sim's tests as they stood before 0323, 15 failed with the near side throughout and 17 with
         the lean and Right first. Without the commitment, 11 layouts had an Agent a minute without a load where 7
         and 6 do.
       - **What it costs** (`docs/playtest/m4.md > Known issues and accepted behaviour`): units that converge on an
         occupied point, or crowd a doorway or a patch's face, step a whole width round the unit in their way where
         they used to shuffle a step, and settle later and elsewhere; a unit stopped by a standing unit beside the
         thing it heads for goes round on its near side even where that leads away from it.
       - **A worker on its site** (`orders/build.rs`, design change 0117: "stopped by a blocker on the way while
         already overlapping the footprint, it starts deploying from where it stands"): beginning a contact step
         there counts as stopped, since the step would take it off the footprint.
     Every candidate is taken only if it is wholly free.
  5. **All fail**: advance to contact or stay; `blocked_ticks += 1`. **(D)** A whole step along a face (no
     progress) counts as blocked too, so units dancing in a corridor count as stationary after 12 ticks.
- Rotations use constants (`(x, y) → (−y, x)`, and `(x − y, x + y) × FRAC_1_SQRT_2` for the `Diagonal` step), no trigonometry. Facing follows
  the path direction, not the sidestep, turning by at most TurnRate; when the turn completes within the tick (all M1
  units) velocity is `d` itself. **(G)** TurnRate is one budget per tick: the aim in phase 5e turns a unit only by
  what this turn left of it (`movement::Turns`), so a unit that moved never turns more than its TurnRate in a tick. Other movement models replace only this "desired direction → velocity and facing"
  step; path following and collision stay shared.
  **(M3-G)** For a unit with momentum a step aside (along the face, or across to a lane: `Moved::aside`) isn't
  blocked while the unit moves: it needs about a dozen ticks for what the others do in four, and would count as
  jammed, and as arrived behind an occupied target (6.5 rule 1), just as it gets round.
- A unit counts as **stationary** for others when its locomotion is Stationary or `blocked_ticks ≥ 12`.
- **(Y)** A unit with `goal = None` moves only by its own sidestep for a blocked unit of its owner (6.6); nothing
  pushes it. **(M3-G)** And by braking off its momentum (6.7), or off a unit it overlaps (6.8).
- **(D) No overlap by construction, in floats.** A position is committed only if it passes the overlap predicates
  of I1 and I2 against the tiles and every unit where it stands now (`World::is_free`); a candidate that fails by a
  rounding corner case is dropped and the unit stays. The sweep works on the same differences as the predicate
  (`a − b` against `half_a + half_b`), and an assigned contact that rounds to the inside is moved away float by
  float (`space::resting`).

**6.4 Group destinations: the magic box (G1).** **(M)** Design change 0021 replaced the slots first approved here
(`systems/controls/commands.md > Group Destinations (Magic Box)`). At command application (phase 4), a location
order (Move to a point, AttackMove, Patrol) to more than one unit gives each unit its own destination
(`Order::slot`, `orders/destinations.rs`):
- **(Y)** The **box** is the bounding rectangle of the units' full Silhouettes, their collision boxes (design
  change 0028; slice M took their centres). The **group's centre** is the box's centre, so a group arrives centred
  on the click.
- **Click inside the box, edges included: converge.** Every unit goes to the click itself (`slot = None`), and those
  the crowd blocks stop by 6.5 rule 1: **(Y)** within 2 GU (**(V)** 3 GU) of the click behind units that won't
  move, which the members that arrived before them are (they don't yield to their own group order, 6.6). A unit
  shut out farther than that re-routes and gives up. A group in a line has a box as high as its units, so a click
  on or between them converges it.
- **Click outside: keep the arrangement.** Every unit goes to its position plus the vector from the box's centre to
  the click, so the group arrives in the shape it had.
- **No formation movement**: each unit goes at its own speed, and a mixed group strings out.
- **(Q) Resolved at the click** (design change 0030): the destinations are computed once, as the command applies,
  queued or not, and each unit keeps its own in its order (`Order::slot`). A unit starts a queued leg alone, when
  its earlier orders are done, and nothing recomputes its destination then: as a group of one it would converge on
  the click. So the shape appears at every destination, a unit that falls short of a spot still heads for its own
  spot on the next leg, and a member that dies on the way moves nobody's spot. Slice M already worked this way.
- **(Q) Shift-queued orders: the box at the planned ends** (design change 0029, `destinations::planned_end`). For a
  queued order, each unit's position, for the box and for its offset, is where its earlier orders leave it: the
  destination of its last order that moves it, the current one included, if that order ends at a known location
  (a Move to a point or an AttackMove: its own spot, or the click it converges on). Stop and Hold move nobody, so
  the order before them decides. If no order moves the unit, or the last one ends somewhere unknown (an Attack or
  a Move to an object; a Patrol, which never completes), its current position stands in, 0029's fallback. The
  silhouettes are put at these positions (0028). So "far north, then back" brings a line back as the line; and
  after a queued converge the box is one silhouette around the converge point, so the next queued click outside
  it sends every unit to that click, and the blob moves on as a blob. Slice M took the current positions for
  queued orders too. A destination is still made reachable from the unit's current position (its component).
- **Unreachable destinations** (**(Y)** design change 0025 confirms this: the reachable point nearest the target): a
  destination beyond the map's edge is clamped so that the unit's box lies inside the map. A destination the unit
  can't stand on (a blocked tile, a box not clear of one, another component) moves to the nearest point it can stand
  on (`nav::reachable_point`): in rings of tiles outward from it, the point of each tile of the unit's component
  nearest to it where the box is clear, ties by tile index; exact for boxes up to a tile wide. That is 5.3's
  retargeting with points for tile centres, so a group that meets a cliff lines up along its edge instead of piling
  onto tile centres. It never searches. Air units are only clamped to the map.
- A single unit has no box: it goes to the click (`slot = None`), as before.
- **Paths**: the group shares one field anchored on the click's tile; a member whose destination the anchor can't
  see plans alone (5.5).
- The box has no size limit (TBD in the design): a widely scattered selection mostly converges.

**6.5 Stuck detection and recovery.** Progress = remaining path length to the goal. `last_progress` updates when it
beats `best_remaining` by half a step. Following a slower unit is progress. **(E)** Towards an object that is
moving (a unit that moved this tick or the last), progress is moving forward at all: the object may be as fast as
the unit, and the path is re-aimed as it moves, so its length says nothing; towards an object at rest it is the
remaining path length, measured afresh from when it came to rest. A unit is **stuck** after
`STUCK_TICKS` = 24 without it. Then, in order:
1. **(Y) Occupied target** (design change 0026, `systems/controls/command_behaviors.md > MovingToLocation > Occupied
   target location`): the goal is a point within `OCCUPIED_ARRIVAL_GU` = 2 GU (**(V)** 3 GU, design change 0169), and
   the unit in the way is stationary and won't move for this unit → `Arrived`, with no repath. **Yielding comes
   first**: a blocker that yields to this unit (6.6) has been asked to step aside, so the rule is for blockers that
   stay: units that arrived with its own group order, units on Hold, busy units, enemies, units that can't move,
   and a yielding unit that found no room.
   **Timing**: this rule alone doesn't wait for the stuck clock when the blocker has no goal of its own (it will
   never move on): the unit has arrived once it found no way forward for `JAMMED_TICKS` = 12 in a row. Behind a
   stationary unit that has a goal (jammed for now) it applies when the unit is stuck, 8 ticks before the first
   repath. Slice D's stand-in "arrived at any distance behind a unit of its own order group, after a repath" is
   gone: a unit shut out farther than that from its destination re-routes and fails, a unit walled out of its
   magic-box spot by its own group included (**(V)** design change 0183). **(E)** For an object goal the rule is the
   same, the radius measured as the gap between the unit's shape and the object's (design changes 0032 and 0033,
   "Blocked near the target"; **(V)** the same 3 GU, 0169). What `Arrived` then means is the behaviour's: a static
   target completes the order, a followed unit is waited for where the follower stands, until it moves.
   **(M4-W2) Not on a leg that presses on** (`Mover::presses_on`; design change 0315,
   `factions/syndicate/units/agent.md > Gathering > Gathering rules`, "Waiting at the hatch": "an Agent waits only
   once it is within reach of the hatch [...] Until it is within reach it is never 'arrived': it keeps actively
   moving toward the hatch, steering around whatever is in its way [...] and is exempt from MovingToLocation's
   'blocked near the target by standing units counts as arrived' rule [...] It waits as long as it takes; it never
   gives up").
   - **A property of one leg**: the behaviour sets it after `Mover::go_to` (`Mover::press_on`), and the next leg
     begins without it. Only the Gather job's leg back to a hatch sets it (`orders::gather::legs::return_leg`): an
     Agent's leg to its patch, a Drone's and a Recruit's legs, and every other order's moves are as they were. It is
     hashed and saved only while set.
   - **Such a unit goes on by the rules below**: it steers (6.3), looks for another route three times on the
     schedule of rule 3, and fails by rule 4, 104 ticks after its last progress. The behaviour then begins the leg
     again **in the next tick**, for the point nearest to where the Agent stands by then within reach of the hatch,
     for ever. (Until 0315 the leg arrived behind the standing Agents and began again after
     `tuning::gathering::LEG_RETRY_TICKS` = 32 of standing. That wait is left for a leg that reached the end of its
     path out of reach or found no path, a hatch walled off, where a leg in every tick would be a path request in
     every tick; and for the Drones' and the Recruits' legs back.)
   - **To the others it is a unit on its way**, not one at rest: jammed for `JAMMED_TICKS` it counts as stationary
     (6.3), but it has a goal, so a unit behind it re-routes plainly first and avoids its tile only from the second
     attempt (rule 3), and it never yields (6.6).
   - **It doesn't end the jam at the hatch** (`docs/playtest/m4.md > Known issues`; the trace is in
     `crates/sim/tests/agent_gathering/jam.rs`): the Agent that has unloaded is still in a pocket between the wall
     and the loaded Agents that press for its place, and pressing on isn't making way. Where that Agent's own legs
     shuffle it a few space units every second, the Agents blocked by it take over its progress clock (the rule
     above: waiting behind a unit that is getting on), are never stuck, and stand on one leg without limit.
2. **Queueing (E)**: a unit chasing the target its attack engages, blocked by a stationary unit of its owner whose
   attack engages the same target → no path request (`movement::stuck::queues_behind`). The check is a few
   comparisons, so it runs every tick the unit is blocked instead of every 16. **(M3-G3)** It fails as any stuck
   unit does (rule 4) once it has no attempt left; until design change 0223 it never failed.
   **(M3-G2) It goes round the row** (`systems/units.md > UnitCollision > Ground Collision`: "Moving units must
   pathfind around occupied space"). Until slice M3-G2 it only waited, and a unit behind its owner's firing line
   stood out of range for as long as the target lived, with open ground on both sides (`docs/playtest/m3.md`: the
   last of four Peacekeepers on a building). The units of a firing line each stopped in the tick their attack
   reached the target, so they stand a fraction of a step out of line, and the unit behind them stepped to and fro
   in the pocket between two of them: rule 4 (6.3) goes round one blocker and keeps no memory. A repath can't help:
   the row is in no grid, and an avoid tile is a tile wide where the units in the way stand a fraction of a tile
   from the unit, often in its own tile.
   - **The detour** (`movement/round.rs`): the *row* is the unit in the way and every unit standing for good
     (stationary, no goal of its own) that is nearer to one of the row than the moving unit is wide; its *box* is
     their bounding box grown by the unit's half extents and 2 su (`ROW_CLEARANCE_SU`). The detour goes straight
     out of the box by a side and round it, from corner to corner, to an end of the *front*: the side that faces
     the target. Of the ways whose legs are clear of blocked tiles and of the row's own units the shortest is
     taken. Its waypoints go in front of what is left of the path (`Mover::go_round`), and
     `Mover::detour_end` keeps the staggered check (5.6) from skipping them: the tiles show nothing of what they
     lead round.
   - **Attempts**: it counts against the leg's `MAX_REPATHS` and waits like a repath, but asks for no path. It is
     due when the unit is stuck and its wait is over (rule 3's schedule), or already after `JAMMED_TICKS` without
     a way forward behind a unit that has no goal (it stands in range, and won't move on). The second counted
     attempt takes the front's other end. An attempt after **a lost race** is free: the unit now in the way
     moved after this unit's last attempt (`Body::moved`), so it took the place the unit went for. Each lost
     race is a unit that found its place, so there are at most as many as units.
   - **(M3-G3) No way round** (a corridor, a passage the row fills), or no attempt left: the normal **Failure**
     rule applies (design change 0223, answering Q0057; `systems/controls/command_behaviors.md > AttackingObject >
     Blocked by its own side`: "it does not wait behind the blockers indefinitely"). An attempt that finds no way round
     counts all the same. Once none is left the unit gives up, still stuck 24 ticks after its last attempt (rule 4): 84
     ticks after it was blocked behind a unit with no goal (attempts after 12, 28 and 60), 104 (6.5 s) after its last
     progress otherwise (5.6; the grace below adds up to 8). Meanwhile it steers as any blocked unit (6.3),
     sidesteps included. The
     behaviour then gives the target up (`systems/controls/command_behaviors.md > AttackingObject > Failure`): an Attack
     completes, and the unit, idle, takes targets again by itself (`systems/controls/behaviors.md > BaseAutoTargeting >
     Unreachable auto-targets`). An AttackMove leaves the target alone for 32 ticks and walks on, then takes it again
     (`orders::travel::pursue`, as built), so in a corridor it gives up every 116 ticks and its leg never ends while the
     row stands. Slice M3-L4 ends that cycle (design changes 0230 and 0235: the failed chase counts as can't reach, and
     the unit walks on without turning back). Until 0223 the unit waited and stood (M3-G2's `stuck::waits_in_queue` and
     `World::hold`, its step only when the way straight ahead was free, no step aside) for as long as the target lived.
3. **Repath**, for a unit that doesn't queue, if `repaths_left > 0`, when its wait is over (5.6). **(D)** The tile of the stationary unit in the way is added to `avoid_tiles`:
   at once if that unit has no goal; if it is on its way itself (only jammed for now), from the second repath on,
   the first being a plain retry. If the blocker stands in the unit's own tile (the start tile is never avoided),
   the neighbouring tile on the blocker's side is taken; the goal's tile is never taken. **(G)** If that unit
   arrived with this unit's own group order (it won't make way, 0027, and nor will the others that did), the tiles
   of all the units that arrived with that order are added, the nearest first, up to the cap of 16
   (`stuck::avoid_group`): the new route goes round the group and comes at the unit's place from its open side,
   where a route round the one blocker led into the next.
4. Otherwise, 24 ticks after the last repath → `Failed`; the behaviour stops and completes
   (`systems/controls/command_behaviors.md > Failure`).

**(D)** Three refinements, each from a failing scenario:
- **Waiting behind a unit that is getting on is progress.** A unit blocked by one that has a goal takes over that
  unit's `last_progress` if it is later. A queue at a choke isn't stuck while its head moves; units that block each
  other for good share a clock that stands still. Without it, 20 units at a one-tile choke burnt their repaths
  waiting their turn and failed. **(H2)** Only the clock of a unit that has got on since its leg began is taken
  over (`stuck::leads`): a new leg sets a unit's own clock and is no progress, so a blocked unit sent again and
  again no longer keeps those behind it from looking for a way round.
  **(M4-W3) What it lets through, and a correction that wasn't kept.** The first measure on every new path sets
  `best_remaining` afresh, so a unit whose legs begin again and again (a worker trying its patch, 6.5 rule 1's
  pressing leg) makes half a step of "progress" on each by regaining ground it stepped back from; from its first
  such step it leads, and those blocked by it take its clock over and never look for another route (slice M4-W2
  saw five Agents stand so for five minutes). Measured on top of design change 0323's contact step: sharing the
  clock only with a blocker that heads the same way as the unit (waiting *behind* it, not against it) left 8 of
  the Agents' 30 layouts with an Agent a minute without a load where 0323 alone leaves 6, and cost the crowds (5 of
  5,856 units fail in the two-block layouts where 2 do, the soak's failed moves 17, 1 and 10 where 9, 0 and 2). With
  0323 the hatch no longer locks up, so the rule stays as it was.
- **A new path is no progress.** Its first measure only sets `best_remaining`; **(Y)** and nothing else restarts
  the stuck clock (5.6).
- **Grace.** A unit due for a repath or for giving up that touches no unit in that tick (it just stepped around
  one) waits up to 8 ticks to meet its blocker again, so that rules 1 and 3 can see it.

An `Arrived` or `Failed` unit is Stationary and has no goal, so it cannot oscillate; **(Y)** it moves again only by
a sidestep (6.6). A unit reacts to an enemy only on contact, so pathing leaks nothing about unseen units.

**6.6 Friendly yielding. (Y)** Design changes 0022, 0023 and 0027 (`systems/units.md > UnitCollision > Ground
Collision > Friendly yielding`); the code is `movement/yielding.rs`. **(M4-S2)** A lowered pop-up Laser Turret that
wants to rise asks its owner's idle units over its footprint for the same sidestep, off the footprint (design change
0296 point 3, `factions/syndicate/structures/laser_turret.md > Pop-up rules`; `yielding::make_way`); a raised one is
an obstacle like any structure, from the tick it starts to rise (5.7: the grids follow). **(M5-Y)** One outside its
Tunnel's area, a neutral one among them, never rises, so never asks (design change 0360). A moving unit blocked by an
**idle** unit of its own owner asks it to make way, and the idle unit takes **one short sidestep**. Collision stays
hard: the sidestep is a real move, committed through the same predicates as every other (`World::is_free`), in the
unit's own turn of phase 5d.
- **Who yields** (`can_yield`): a unit with an owner and **no active command** (no current order, none queued; Hold
  is a command, so units on Hold never yield), without a goal, not in an attack sequence (Aiming, Firing or
  Cooldown; `AttackState::phase`), and able to move in its current mode (`Mover::can_move`: modes that only slow
  it, such as a raised Riot Shield, don't matter, and it sidesteps at that speed; **the hook** for a deployed or
  deploying Artillery Rig, which will answer `false`). Enemies and nobody's units never yield. Moving units never
  yield or swap, friendly or not: two of them head-on in a passage too narrow to pass both still end `Failed`.
- **Not within the same group order** (0027, `yields_to`): a unit that `Arrived` under a group order doesn't yield
  to members of that order that are still arriving (`Mover::group`; a later order of either clears it). They stop
  against it by 6.5 rule 1. To every other unit of its owner it yields.
- **Requests** (`ask`, `apply`): the mover asks the first unit that blocked its way this tick (`Moved::blocker`),
  whatever it then did about it. Requests are collected while the units move, in id order, and applied when all
  have moved: the first one a unit gets wins (the lowest id that asked), and it sets `Mover::sidestep`, which is
  hashed and serialized state. The sidestep begins in the next tick. The spawn rule and the phase order are
  unchanged.
- **The sidestep** (`Sidestep`, `sidestep`): the unit steps at 45° to the mover's way, half across it and half
  along it, so it leaves the lane and the mover at once. Across the way it goes to the side of the mover's line it
  already stands on; within `YIELD_CENTRED_SU` = 4 su of that line, to the side the mover isn't going round on, so
  the two never chase each other. It goes as far as takes its box out of the mover's lane, at most
  `YIELD_SIDESTEP_WIDTHS` = 1 of its own width *(placeholder, TBD in the design)* and at least 1 su, at its current
  speed, without turning (every M1 unit turns in place; other movement models will need their own). It moves the
  whole step if that is free, else each axis as far as tiles and units allow. It ends when the distance is done or
  after the time the distance takes plus `YIELD_WAIT_TICKS` = 4, and the unit **stays where it got to**: there is
  no way back. A mover still blocked then asks again.
- **Dense idle groups**: a sidestep that another idle unit of the owner blocks passes the request on to it, the
  same way and for the distance still to go, at most `YIELD_CHAIN_DEPTH` = 4 units deep (one tick per link). A unit
  that finds no room stands, counts as stationary, and is an ordinary blocker to the mover (6.5). Chaining was
  chosen over "blocked means ordinary blocker" because a unit can't walk through a tight group of its own without
  it; the depth bounds the cost.
- **(G) Walking through a group.** A group walking through an idle group as wide as itself shoved it ahead, a few
  widths at a time: its units ended 204 su from their places on average and up to 373 su. Three changes:
  - The movers **aim between the idle units** (lane seeking, 6.3): a group of idle units that would yield is a
    row of lanes like an oncoming group, and each mover takes the middle of a gap, or walks around the group if a
    clear lane is within 4/3 of its width (`LANE_AROUND_IDLE`).
  - A unit that is only **clipped** (the mover's line passes beside its box), or that stands in the way of a mover
    walking in such a lane, steps **straight across** the mover's way, with no step along it, as far as its box
    reaches into the mover's lane. The step along the way is what did the shoving. A unit alone and squarely in
    the way still takes the 45° step.
  - The mover asks the idle units it is about to be blocked by, **within two steps**, both sides of its gap at
    once (`yielding::ask_ahead`), instead of one after the other as each stops it. It is still the blocked mover
    that asks, a moment early; a request is applied like any other, the lowest id first.

  Now the idle block parts: its units end 16 su from their places on average and at most 33 su, and all 18 movers
  arrive (16 before). One unit walks through a 5 × 5 block 4 su apart as fast as before (189 ticks; 186 before) and
  moves its units 16 su at most (68 before).
- **Known limits.** A sidestep straight across the mover's way was tried first, by slice Y: a line of movers
  facing an idle group as wide as itself then had nowhere to send it (each unit stepped into its neighbour's lane),
  and a third of them gave up. **(G)** With the movers in the gaps it works: a unit needs only a little to its
  side. An idle unit plugging a passage as narrow as itself has nowhere to step and stays an ordinary blocker.
  Swapping of two moving units is not built (0022 rules it out).

**6.7 DragMovement. (M3-G)** `systems/units.md > DragMovement`, `systems/units.md > HoverVehicle`; plan decision T25;
the code is `movement/drag.rs`. The model of the units that keep their velocity from tick to tick (`Mover::momentum`):
the Drone. It replaces only the "desired direction → velocity and facing" step of 6.3.
- **The integration scheme**, fixed, once per tick and in this order:
  1. **Turn** towards the step the unit wants (below), at most TurnRate. It never stands to turn
     (`systems/combat.md > LocomotionOrientationConstraints[DragMovement]`). Three quarters of the Drone's thrust
     is forward thrust, so it faces the way it pushes: along its path in the open, along a blocker's face while
     it goes round it.
  2. **Drag**: `v ← v × (1 − DragRatio)`, the ratio doubled while the tile under the unit's centre is Rugged
     (`systems/units.md > HoverVehicle > RuggedTerrain`, `systems/units.md > Which tile counts`; design change 0039).
  3. **Thrust**: `v ← v + a`. The thrusts a unit can make are every vector of at most
     OmniDirectionalAcceleration, plus between nothing and ForwardAcceleration along its facing; `a` is the one
     that brings `v` nearest to the velocity it wants. Steps 2 and 3 are `v ← v + a − DragRatio × v`: drag is
     taken on the velocity the tick began with, so thrust and drag balance at `(Omni + Forward) / DragRatio`, the
     design's MaxSpeed: 8 su a tick for the Drone, 4 on Rugged. From rest the speed is `8 × (1 − 0.85ⁿ)` after
     `n` ticks with the Drone's numbers of design change 0205: 1.2, 2.22, 3.09, …, 68 % of 8 after 7 ticks, 96 %
     after 20 and within 1 % after 29 (before 0205, `8 × (1 − 0.9ⁿ)`: 52 %, 88 %, and within 1 % after 44).
  4. **Position**: `pos ← pos + v`, as far as tiles and units allow: the whole move if it is free, else its two
     axis components one after the other (`World::shuffle`). **If the move runs into an obstacle, the blocked
     part is dropped and the velocity is set to 0** (`systems/units.md > UnitCollision > Collision Response
     (Un-sticking)`, design change 0035): the whole velocity, also what the unit had along the obstacle. Otherwise
     the momentum kept is the displacement made.
     - **The creep**: where 6.3's step is a slide along tiles (rule 2, `Moved::slid`), the unit thrusts towards
       the step it asked for, into the tile, and meets it: "running into the wall every frame". It restarts from
       0 every tick and creeps along the wall at single-tick acceleration (1.2 su for the Drone, where it flies
       at 8) while it turns, until its way is free; then it accelerates normally.
     - **Progress**: the creep shortens the remaining path, which is what 6.5 measures, so a unit scraping free
       is never stuck and never gives up (0035's Implementation impact; `movement/follow/tests.rs`: 400 ticks
       along a wall, progress noted at least every 8).
     - **No obstacles**: a wall or a unit the box only touches (touching is free), a unit it passes through
       (`space::blocks`: gathering Drones, units that already overlap) and a unit it moves off (6.8).
     - A debug teleport zeroes the velocity as well.
- **What it wants**: the step that 6.3's rules give a unit without momentum, at the approach speed. The rules
  run first, as a probe (lane seeking, slide, follow, go around), and the thrust steers the velocity towards the
  step they return. If the thrust reaches it, that step is the move as it is. So the collision rules, the
  yielding requests and the stuck clock are the same for every model.
- **Braking and arrival**: the unit brakes with its omni-directional thrust against its motion, facing the way
  it goes. The **approach speed** for a remaining path length `D` is the highest speed `s` such that a step of
  `s` followed by braking in every tick (`v ← v × keep − brake`) stops within `D`; it is built from the goal
  backwards (`drag::approach_speed`), with `+ − × ÷` only. Braking therefore begins when the stopping distance
  reaches the remaining path, and within `brake / keep` of the goal the speed is the distance itself: the unit
  lands exactly on the goal at a speed one tick of braking takes away, and is at rest. The plan uses 15/16 of the
  thrust (`PLANNED_BRAKE`): the reserve absorbs the rounding of positions, which would otherwise push the unit
  past its goal by a rounding error. **Bounds for the Drone** from top speed, with its numbers of design change
  0205: it begins to brake 35 su before its goal (one more step, then 27 su in 10 ticks); stopped without a goal
  it stands within 26 su and 9 ticks. Before 0205: 51 su (43 su in 16 ticks), and 42 su and 16 ticks.
- **Without a goal** (a Stop, an arrival within reach of an object, a failed move, a held base) a unit with
  momentum brakes the same way and moves meanwhile: `Locomotion::Stopping`, slower every tick. `Mover::stop` and
  `finish` leave the momentum to the movement step.
- **Reach**: an object goal, and a worker's leg, end when the unit comes within reach, at the approach speed of
  that distance (about 2.5 su a tick at 8 su); it brakes off the rest while its work has begun, and stops short of
  the object, as planned for the path's end.
- **Corners** are taken at speed: the approach speed looks at the path's whole length, not at its turns. The
  unit swings wide, or meets the wall beyond the corner and starts again from rest. Slowing for corners is
  not built.
- **Kinematic steps**: a yielding unit's sidestep (6.6), the walk out of a side doorway (the Barracks') and the
  step off an overlapped unit (6.8) are plain steps for every model, without momentum.
- **Out of a portal's energy doorway (M3-D2)**: design change 0145 (`factions/colonists/structures/README.md > Portals >
  Portal Exits (energy doorway)`); the code is `movement/doorway.rs::emerge`. The unit appears at the portal's
  centre and moves straight along the heading chosen at its release (`Mover::exit_heading`) until its box is clear
  of the footprint, by its own model: a Drone thrusts along the ray (`Drag::along`), its speed carried from tick
  to tick, and without a path brakes to land exactly where it comes clear; with one it keeps its speed out of the
  footprint and follows its path from there. It never leaves the ray (no lanes, no way round, no sidestep) and
  never comes closer to the centre; it goes up to a unit in its way and no farther (`World::straight`).
- **State**: `Mover::momentum`, serialized and hashed only while it isn't zero, so the states of games without
  such a unit hash as they did (the goldens of M1 and M2 are unchanged).

**6.8 Idle units that overlap move apart. (M3-G)** Design change 0126 (`systems/units.md > UnitCollision > Units
that already overlap > Idle units push apart`); the code is `movement/separate.rs`. A unit that yields (6.6: no
command at all) and is at rest, whose box overlaps another unit's of its layer, moves off it in its own turn:
away from the overlapping unit with the lowest id, along the axis on which it is out soonest, 2 su a tick
(`SEPARATE_SPEED_SU`, *placeholder*), no farther than to where the two boxes touch. Where the two centres
coincide on an axis the lower id goes the negative way and the higher the positive. If it can't move that way
(a tile, a unit it doesn't overlap), it tries the other axis, then the ways back through the unit. Two idle
units both move; a unit with a command (Hold Position, mining, any order) never does. The step is committed
through the same predicates as every move, so it brings no new overlap. `n` units stacked on one point come
apart within `18 × (n − 1)` ticks for Drones, 12 each for Peacekeepers.

## 7. Numbers: f32, not fixed point

**Decision: keep `f32` under the current rules, and keep every discrete decision in the pathfinder in integers.**
- Rust guarantees IEEE-754 results for `+ − × ÷` and `sqrt` at every optimisation level, never contracts to FMA
  and has no fast-math, so x86-64, aarch64 and wasm32 agree bit for bit. The remaining cross-machine risks are
  transcendentals (already routed through `libm`) and NaN payload bits (state must never hold a NaN: validated
  inputs, `normalize_or_zero`, an "all floats finite" invariant in tests).
- Precision is ample: the largest coordinate is 16 384 su, where f32 resolves 0.001 su.
- This design removes the usual float hazards: no epsilons in overlap tests (strict comparisons, assigned contact
  positions), no trig in collision, integer A\* costs and keys, `total_cmp` plus id for every ordering.
- Fixed point would mean replacing `glam::Vec2` across the sim, the client and the file formats, and writing our own
  `sqrt`/`atan2`, for a guarantee we already have. The design's fractional values (0.9 and 0.3 su/tick², 0.15 drag)
  fit both.
- Lockstep needs the same binary-level determinism replays need, nothing more. Insurance: golden hashes committed in
  tests (section 8) and, once a second architecture is available, running that one test there. Excluded: 32-bit x86
  without SSE2 (x87).
- Add `f32::mul_add` to `clippy.toml` (keeps arithmetic uniform and avoids the slow software fallback).

## 8. Test strategy

- **Nav unit tests** (ASCII maps): A\* cost equals a brute-force Dijkstra on seeded random maps; expansions ≤ passable
  tiles (closed set); a straight open-map path smooths to two waypoints (no zigzag); same call twice gives the same
  path; an unreachable goal costs 0 expansions and returns the nearest reachable tile; LOS against hand-drawn corners.
- **Invariants**, checked after every tick by a `TestSim` switch, in every scenario test:
  I1 no two units of a layer overlap (**(H2)** except two Drones on a Gather job, and two units that already
  overlapped a tick earlier, until they are apart: design change 0106); I2 no ground box overlaps a blocked tile or
  leaves the map (except a unit in an exit doorway, `Mover::exit`, on its own building's tiles: it only comes out,
  and **(M3-D2)** out of a portal's energy doorway only within its open arc; **(M5-Z1)** never over ground under the
  footprint that it can't stand on, so that it stands on open ground when the building goes);
  I3 `|pos − prev_pos| ≤ max_speed`; I4 all floats finite; I5 a unit with no goal doesn't move;
  I6 repaths per order ≤ `MAX_REPATHS`, request age bounded; I7 an `Arrived` unit never moves again without an order.
  **(Y)** I5 and I7 allow the one exception of 6.6: a unit without a goal may move while it has a `Mover::sidestep`,
  and only a unit without a goal and without an order may have one.
  **(M3-G)** Amended for DragMovement and for 0126:
  - **I3**: for a DragMovement unit `max_speed` is the model's maximum, `(Omni + Forward) / DragRatio` on open
    ground, without the mover's `speed_factor`: a mode that slows such a unit only weakens its thrust, and the
    speed it has decays. For every other unit it is `max_speed × speed_factor`, as before.
  - **I5 and I7, braking**: a DragMovement unit without a goal may move in a tick if it had momentum a tick
    earlier, and may be other than at rest only if it is `Stopping`, its `velocity` equals its `momentum` (or the
    momentum is zero: it met an obstacle in this tick), and it is **shorter than the momentum a tick earlier**. So it brakes to a stop and nothing else: one that keeps its speed,
    gains speed or moves while claiming to stand fails, and so does a unit of any other model that has momentum
    at all (`tests/movement_drag/`, from the wrong side).
  - **I5 and I7, separating**: a unit without a goal may move, and be other than at rest, if it has no command
    and its box overlapped another unit's a tick earlier (6.8).
  - **I1** is unchanged: DragMovement adds no way for two boxes to come to overlap (every position is committed
    through `World::is_free`), and a separating pair is covered by "already overlapped a tick earlier".
  **(D)** The switch (`TestSim::check_invariants`) is on by default, so every test that uses a `TestSim` runs them;
  `sc_sim::testing::invariants` has the checker for tests that step a bare `Sim`.
- **Scenario tests** (`crates/sim/tests/movement_*.rs`, inline ASCII maps): straight move arrives exactly at
  `ceil(d / speed)` ticks; around a wall; through a one-tile gap; two Peacekeepers pass in a 64 su corridor; two
  Guards head-on in it both end `Failed` within a bound; 20 units through a choke all arrive within
  `2 × path / speed + c`; a group arrives, every unit Stationary, and stays so for 100 ticks; an idle unit in the way
  is passed and never moved; goal on a cliff; enclosed unit; an object spawned on the path forces one repath; chase a
  moving target; three Shift-queued waypoints in order; Patrol for three cycles; Hold; Stop. **(M)** The magic box
  (`crates/sim/tests/group_destinations.rs`, 0021's Verification): a line ordered far away arrives as the same line
  centred on the click; a click inside the box converges with nobody failing; a mixed group strings out, fast units
  first; offset destinations on a cliff or off the map are retargeted; a single unit goes to the click as before.
  **(Y)** Yielding, occupied targets and the schedule (`crates/sim/tests/movement_yield.rs`, the Verification lists
  of 0022–0027): a unit walks through its owner's idle group and around an enemy's; two own units head-on in a
  corridor both give up; Hold units, attacking units and units that can't move don't budge; a slowed unit yields at
  its speed; twelve units converge without shuffling or re-routing and then step aside for another unit; a unit three
  tiles behind a wall of Hold units re-routes and gives up, one within 2 GU arrives; a boxed-in unit repaths at 32,
  48 and 80 ticks and gives up at 104; a unit blocked for 20 ticks never repaths; a save in the middle of a sidestep
  continues identically. The 18-v-18 runs (`movement_crowds.rs`) print their counts. **(Q)** Shift-queued group
  orders (`crates/sim/tests/queued_group_destinations.rs`, the Verification lists of 0029 and 0030): a line queued
  far north and back returns as the same line; a queued converge followed by a far queued click moves the blob as a
  blob; a mixed line queued north and back turns at different times and ends in its line; a unit removed on the way
  leaves the others' spots unchanged. **(G)** Crowds (`crates/sim/tests/movement_crowds.rs`): two blocks of 18 meet
  head-on and at most 2 of 36 give up, with one owner and with two; dense blocks of other shapes and of Guards pass
  with at most a fifth failing; a group walks through its owner's idle block of the same width, which ends within 96
  su of its places (40 su on average) with everyone standing, and an enemy's block never moves. The bounds are loose
  on purpose. `World::lane` has its own unit test. Nav: a group's field costs at most the component; shared smoothing
  gives the waypoints of `smooth`; `open_around` implies `box_clear` and `los`; the cells' marks never miss a heading
  that counts as oncoming.
  **(M3-G)** DragMovement (`crates/sim/tests/movement_drag/`, `movement/drag/tests.rs`): the speed curve from
  rest and its equilibrium; braking within the bound and an exact landing; a Stop in full flight; a Rugged strip
  at 4 su a tick and back to 8; a wall met at speed takes all its velocity, and the creep along a wall is
  progress for 400 ticks (0035); a teleport; rest within
  reach of a patch and a node without touching them or turning back; a save in full flight. Separation
  (`movement_separate.rs`, 0126's Verification): two idle Drones drift apart, one stopped inside a mining Drone
  moves off alone, one on Hold stays, stacks of 2 to 8 on one point and in a corner come apart within the bound.
- **Oscillation**: distance travelled ≤ 1.5 × path length + c; reversals of velocity along the path ≤ a small bound.
- **Determinism**: same run twice, per-tick hashes equal; replay round trip; **save at tick k, load, continue**:
  hashes equal (catches hidden unserialized state); **golden hashes** for three scenarios as committed constants.
- **Soak**: seeded random orders, 200 units, random maps, 2 000 ticks: invariants hold, no panic. **(Y)** Every
  move ends within `GIVE_UP_TICKS` and the grace of its last progress, and of the moves that end at most 60 % fail
  in a seed and 35 % overall (16 % with yielding, 32 % before; **(G)** 7 % with lane seeking: 38, 9 and 34 failed
  moves in the three seeds, where 105, 54 and 64 failed; 8 % with the re-route round the group, 40, 9 and 43,
  within the spread of the seeds: over 64 seeds 2 918 moves failed where 2 850 did, and 63 more arrived).
- **Perf**: an `#[ignore]` test printing ms per tick for 500 units on 256×256, plus CI assertions on the counters.
- **(E) Behaviours** (`crates/sim/tests/behaviours_*.rs`, the Verification lists of 0031–0034, 0036 and 0037): an
  attacker stops when the gap between the edges is its Range; a chase around a corner until the target is
  destroyed; a target lost in the fog ends the order where the attacker stands (**(M3-L3)** design change 0200;
  under 0034 it walked to the last seen position); a target across a moat ends the order
  at the water's edge; a second attacker blocked behind the first in a corridor gives up by the Failure rule
  (**(M3-G3)** design change 0223; it queued for as long as the fight lasted, and `attack_behind_friends/
  no_room.rs` has 0223's Verification: Peacekeepers in a corridor, Sabres on a building); an idle unit
  chases to its leash and returns; Hold never moves; a follower keeps up, waits within 8 su, and waits behind a
  plugged corridor without re-routing; a patch ringed by units is reached within 2 GU; an AttackMove group
  engages and resumes; a Patrol cycles, engages, starts where the order before it ended, turns around after a
  failed leg and ends after two. A soak of 120 units fighting under random orders of every kind keeps the
  invariants, leaves no dangling id and asks for few paths. **(G)** Its checks survive tuning: no unit stays idle
  in reach of a visible enemy, every order but Hold, Patrol and Follow ends once the orders stop, and refused
  orders, failed moves and path searches stay under stated shares. **I5–I7 changed**: a leg is one `Mover::go_to`, an
  order's or a behaviour's, so a unit may also move, and get its repaths back, when a behaviour began a move in
  that tick.
- **QA**: `TestSim::ascii()` renders the map with units as letters for readable failures; a client path overlay
  (waypoints, destinations, status colours) behind a debug key and flag; client replay playback (`--replay <file>`), so a
  failing sim test's `Replay` can be screenshotted with `--screenshot`.

## 9. M1 slicing

| Slice | Content | After | Model |
|---|---|---|---|
| **B Unit foundation** | `UnitBaseDef`/`UnitDef`, Peacekeeper and Guard, `Orders` and `Mover` types with hashing, `Command::Order` with gate and queue rules, spawn/teleport overlap checks, a naive straight-line mover, empty phase stubs 5a–5f in `step`, placeholder unit visuals | — | Opus |
| **C Navigation** | NavGrid, components, A\*, field, LOS, smoothing, request budget (`nav/`, pure and testable alone) | B | **Fable** |
| **D Steering and collision** | Spatial grid, 6.3 step, group slots, stuck and recovery, the invariant harness and movement scenarios; replaces the naive mover | C | **Fable** |
| **S Selection** | Clicks, box, control groups, Tab cycling (client) | B | Opus |
| **H Commands UI** | Command panel, right-click resolution, cursors, indicators, Shift-queue | S | Opus |
| **V Vision** | Visibility per player, ElevationModifier (sim); fog texture (client) | B | Opus |
| **W Combat** | Attack phases, hit list, damage, deaths; range checks per G7 | B | Opus |
| **E Behaviours** | Move-to-object, Attack chase and queueing, AttackMove, Patrol, Hold, Stop, idle auto-target and leashes | D, W, V | **Fable**, or Opus with a Fable review |
| **G QA and hardening** | Debug control of both sides, replay playback, path overlay, soak, perf test, golden hashes | E, H | Opus; Fable reviews movement |

- **Parallel**: after B, four tracks run at once: C → D (movement), S → H (client), V, W. E joins them.
- B creates every shared touch point (`Object` fields, `hash.rs`, the phase stubs, `Command::Order`), so the parallel
  slices each fill their own module and merges stay small.
- Before D starts, G1–G5 should be answered; C needs none of them. W needs G7, E needs G8 and G9.
