# Behaviours

Read this when you touch orders and the behaviours that carry them out (attack, move, gather, build, and the rest).

## Behaviours

Slice E (`orders.rs`, `orders/`, phase 5a's loop in `orders/run.rs`; `systems/controls/behaviors.md`). Phase 5a runs one
behaviour per unit and tick: its current order's, or the idle one. A behaviour never moves or shoots anything itself: it
writes the mover's goal and the target the attack engages, and reads `Mover::status` and the snapshot.

- **State** (`orders/state.rs`, the design's BaseBehaviorState): `ActiveOrder::state` (`Behaviour`: `Plain`,
  `Attack`, `Approach`, `AttackMove`, `Patrol`, `Hold`, `Gather`) and `Orders::auto` (`Auto`: what a unit without an order
  does: `Watching`, `Engaging`, `Returning`). Hashed and serialized like all state; every order that starts resets
  `auto`. A state names objects only by id, and the removal path clears them (`Orders::forget`).
- **The snapshot** (`orders/world.rs`): `World` holds a `View` of every object (position, shape, owner, domain,
  elevation) as the last tick left it, the vision of its phase 5f, and a coarse grid of the objects a scan may
  pick. Every unit decides on the same world, whatever its id. `World::pick` is the scan of
  `systems/controls/behaviors.md > BaseAutoTargeting > Target Selection Priority`: among the ValidTargets in reach
  (enemies only: another player's destructible objects that the unit's owner sees and its attack can target),
  threatening ones first, then the least rotation from the unit's facing **in steps of 45°** (design change 0179,
  `world::rotation_step`, a placeholder step: each step reaches up to its upper bound, so a target exactly 45° off is in
  the first), then the smallest gap, then the lowest id. A scan of SightRange measures from the unit's centre, as vision does
  (`systems/controls/behaviors.md > BaseAutoTargeting > Active during Idle`): a target is in reach when the nearest
  point of its shape lies within the unit's SightRange of the unit's centre (its type's, with the ElevationModifier; a
  sight buff widens vision only; `World::in_sight_range`); MinRange and Range stay edge to edge. It skips enemies
  inside the unit's MinRange
  (design change 0182) and picks one it can hit; the caller may leave out more (`pick`'s `admit`: the AttackMove's
  leash). Enemy structures are ValidTargets too (design change 0103), so units attack enemy bases on their own; the
  structures that can't attack come after the targets that shoot back (a Garrison is one that can while a unit inside
  could attack the scanner's domain: design change 0377, `World::with_occupants`). The grid holds candidates by their
  centres, so a scan looks as far as the unit's longest range with the modifier, plus its own half size and the largest
  candidate's
  (`World::candidate_half`): a footprint in reach is found however far its centre lies.
- **AttackingObject** (`orders/engage.rs`): `close_in` is its core, shared by all fighting. In range (`combat::
  reaches`) the unit stops and phase 5e attacks; out of range it heads for the target with `MoveGoal::Object`,
  starting only while `combat::may_move` holds. An **Attack order** adds its ends: the target destroyed (the removal
  path completes the order); the target **lost from its owner's vision** (design change 0200, which replaced 0034's
  walk to the last seen position): once its owner has seen it while the order ran (`AttackOrder::seen`), the order
  completes at once, as for a destroyed target, and the unit stops where it is (or the next queued order starts);
  the approach failed, or ended as near as the unit gets to a target that hasn't moved.
  **A remembered target** (design change 0139; the snapshot's `World::known`, the owner's `TargetView`) not seen
  since the order began: the unit walks to its last-known position, engaging nothing (`AttackOrder::remembered`),
  attacks it once its owner sees it standing, and stops where its owner sees its place empty, "the way it would for
  a vanished target" (the next queued order starts). Once seen, it is lost as above when the fog covers it again.
  The removal path leaves such an order alone (*Combat > Removal path*).
  **A target inside MinRange** (`engage::too_close`, design change 0182): a unit that can move backs straight away from
  it, to where the gap would be MinRange + `Orders::back_off_gu` (1 grid unit), never beyond Range
  (`AttackOrder::backing_off`; design change 0454, answering Q0179, slice M6-S6a: it replaced Q0054's middle of the
  band, which took a lone Hand of Heaven out of its own sight, so that it never fired), and attacks once the back-off
  ends: it goes on while the target is within Range, until the move ends there or where the unit is blocked (slice M5-O3
  fixed M5-A's back-off, which stopped as soon as the target was out of MinRange). The unit engages nothing meanwhile,
  so that 5e doesn't aim from just past MinRange and hold it there, and engages the target again as the back-off ends,
  or as the target leaves its Range. One that can't move (`Mover::can_move`) fails the order (`AttackOrder::too_close`,
  `Sim::step_too_close`, `SimEvent::TargetTooClose`; the client's feedback line says "Target too close"), and so does
  one whose back-off ended with the target still inside and where it was: it can't back off ("cornered, or blocked
  behind", design change 0204 point 3; slice M5-A, which gave it the feedback). An Attack Ground's location takes the
  same rule through the same code (slice M5-O3: `TooNear` is what the unit backs off from, `BackOff` the order's parts
  the rule works on), and fails with `SimEvent::LocationTooClose`.
  **Can't aim from where it stands** (design change 0454, answering Q0179; slice M6-S6a, `orders/aim.rs`, its numbers
  `Orders::aim_*`, placeholders): a unit in range that can't bring its weapon to bear moves to a spot it can
  (`Pursuit::repositioning`, `AttackOrder::repositioning`, `AttackGround::repositioning`, each hashed only while set),
  and closes in from there; on its way it stops as soon as it can fire, and goes on though its way pass inside MinRange,
  since the spot lies beyond it. An Attack Ground's location takes the same rules (design change 0460, answering Q0184;
  `aim::Mark::point`). Two kinds can't (`aim::stuck`, by the owner's tiles, `World::plan_grid`; units move and yield): a
  hull that aims too whose turn toward the target the tiles cut where it stands and along its spin's creep (a turn the
  creep frees, beside a wall, is left to the spin), and a narrow turret on a base that can't turn in place whose target
  lies inside a circle it turns on or less than the margin beyond its rim (`aim::margin`: `Orders::aim_turn_margin_su`,
  and its weapon's MinRange, since coming round at speed it brakes straight on). The spot (`aim::spot`, within
  `Orders::aim_spot_radius_gu`, by steps of `Orders::aim_spot_step_su`): for the hull, the nearest on rings round the
  unit with room to turn (no solid tile within its half diagonal), in range and beyond MinRange facing the target, its
  straight way there clear (`nav::los`); for the wheeled base, the nearest straight ahead (never back: a vehicle backs
  up only on the Reverse order, `systems/units.md > UnitCollision`), from which the target lies beyond both circles by
  the margin, on to `Orders::aim_spot_far_gu` for an order's own target, which stays in its sight all along the line.
  The wheeled base's approach that ends as near as it gets, in range and out of its arc, repositions too, where it gave
  up before. An auto-acquired target's spot lies within its leash (`aim::Leash`: the IdleOrigin's, the leg's
  PathReference's), and with none there the unit disengages as at its leash. An order's own target with no spot near
  ("it doesn't wait in place") takes the nearest tile's centre within `Orders::aim_spot_far_gu` (16) with room to turn,
  its way there however long (`aim::far`): for the hull, in range and beyond MinRange facing the target; for the wheeled
  base, twice its MinimumTurnRadius and the margin from it, beyond both circles whatever way it faces there, and outside
  the circles it turns on where it stands (its way there no loop, which took the target out of its sight). One hemmed in
  with none there either does as before, the hull turning as far as it can, the wheeled base closing in (a placeholder).
  The way round may take it out of its own sight of the target, which ends the Attack (design change 0200; 0454: "it
  still ends for the usual reasons"). A move to the spot that fails gives the target up (the Failure rule).
- **Idle units** (`engage::idle`): scan their SightRange every tick; the acquired enemy is engaged with `close_in`
  from the IdleOrigin, where the unit stood. Farther than `Orders::idle_leash_gu` (4) from it the unit disengages and walks
  back without scanning; a target destroyed or out of SightRange leaves it idle where it is.
- **Hold Position** (`engage::hold`): scans its weapon's range only, never moves, and disengages the moment the
  target leaves that range. A unit with a turret engages nothing from its base: its turret scans and fires on its own
  (slice M5-U1, *Combat > Units' turrets*).
- **A turret's behaviour** (`orders/turret.rs`, slice M5-U1): runs for every unit with a turret right after its
  base's behaviour, whatever that is, a Move's included; it locks and lets go of the turret's target and never moves
  the unit (*Combat > Units' turrets*).
- **Move to an object** (`orders/travel.rs::approach`; design changes 0032 and 0033): proximity is a gap of at most
  `Orders::proximity_su` (8). A static object completes the order there (also at the nearest point the unit can reach, or
  stuck within 3 GU of it behind units that won't move). **A unit is followed**: the order never completes by
  itself; the follower waits within proximity (or where it was stopped near it, without re-routing) and sets off
  again when the target moves. It ends with a new order, the target's death, a failed move, or an enemy target
  going out of sight: the follower stops where it is (0200). "Leaves the map" (transports, buildings, tunnels)
  is a documented hook in `approach`. A remembered object (0139) is approached at its last-known position, as an
  Attack's. **A Move or Follow whose target is the unit itself is a Stop** (design change 0168): it halts, and the
  order ends at once, as if arrived; a Shift-queued one completes in the tick it starts; the other units of the
  order follow it as usual (a right-click on one of the selected units).
- **AttackMove** (`travel::march`): one `Leg` to the unit's own destination (the magic box). While moving it scans
  its SightRange and engages with `close_in`, by **the reach rule** (design changes 0230 and 0235, `orders/reach.rs`,
  `travel::acquire`): an enemy within weapon range is always taken; one out of range only if the path to a firing
  position (the path phase 5c would serve the chase, `World::chase_path`, walked in quarter-tile steps,
  `reach::judge`) stays within `Orders::attack_move_leash_gu` (6) of its **PathReference** and keeps the target within the
  unit's SightRange, measured as the scan measures it. **The leash limits chasing only** (design change 0180): a
  target in weapon range is always fought, wherever the unit stands (`travel::pursue`); a chase goes on while the
  unit stands within the leash and its judgement stands (`Leg::chase`), and is judged again otherwise. The
  PathReference (`Leg::reference`, `Leg::follow_path`) is the first path the leg's move gets; it is kept after fights
  (the move's path then, the walking path, is `Leg::walking`) and replaced by a new path the unit gets while it walks
  its leg: a re-route round a blockage. The unit lets go, and resumes the move with a new walking path, when the
  target is destroyed, rejected or no longer valid (out of sight: its attack completes, design change 0200).
  - **No memory of targets let go** (0230). A rejection (`Leg::rejected`, `state::Judged`) only saves judging an
    enemy again, and stands only while nothing the judgement depends on has changed: the enemy within
    `Orders::rejudge_move_su` (1 GU, a placeholder) of where it was judged, the owner's `PlayerKnowledge::version` the same,
    the PathReference the same (`Leg::set_reference` clears them). A rejection by the reach rule (a chase the unit
    can't finish, or one that ended beyond the leash) also records where the unit stood (`Judged::from`), and stands
    only while it stands within `Orders::rejudge_self_su` (1 GU, the design's placeholder) of there: design change 0243,
    answering Q0074, "so the same situation always gets the same response" (a unit that rounds a wall judges the
    building it passed over again, as a fresh unit there would). A chase that ends without the target reached is a
    rejection too: given up by the Failure rule or as near as the unit gets (0235), beyond the leash, or out of the
    owner's sight (then where the target was judged, never where it went); the Failure rule's and the fog's keep
    0230's triggers alone (`from` unset), so the units behind their own side in a corridor don't take the target up
    again as they shuffle. Any state that makes a unit pass over an enemy must follow this pattern: a cache of a
    judgement with its inputs, never a list of ids.
  - **The cost bound**: a unit plans at most `JUDGE_PATHS_PER_TICK` (2) judgement paths per tick, its candidates in
    target-priority order, the rest in later ticks (an enemy within weapon range is still taken at once); an enemy
    farther from the PathReference than any firing position could be costs none (`reach::near_route`); and a
    judgement's search gives up beyond `JUDGE_PATH_SIGHTS` (2) SightRanges of path (`nav::find_path_within`), so it
    never floods the map round a long wall. The searches run in phase 5a on their own scratch (`reach::Reach`, kept
    in `NavState`), outside `Paths::path_budget`; `Sim::reach_counters` reports them. The rejections and the chase's
    judgement are hashed and serialized only while set, and so is a judgement's `from`. The own-movement trigger
    costs a path for each grid unit a unit walks with a rejected enemy in its sight, within the same bound
    (`attack_move_perf.rs` measures it).
- **Patrol** (`travel::patrol`): legs to and fro between the position where the Patrol started executing (design
  change 0036) and the destination. A failed leg counts as arrived; `Orders::patrol_failed_legs` (2) in a row end it (design
  change 0037).
- **Gathering** (`orders/gather.rs` with `gather/patch.rs` (the job at its patch), `gather/station.rs` and
  `gather/station/spot.rs` (the job at a Supply Delivery Station), `gather/deliver.rs` (the way back and the line at
  a hatch), `gather/claims.rs`, `gather/search.rs`, `gather/legs.rs` and `gather/site.rs` (the patch or the station as
  the owner knows it, design change 0139), `orders/world/gathering.rs`;
  slices M3-H, M3-H3, M4-D, M4-I and M5-D, plan decisions T17, T34 and T58; `factions/colonists/units/drone.md`, whose rules
  design changes 0123 and 0127 to 0141 settled, answering question Q0037; `factions/syndicate/units/agent.md >
  Gathering`, design changes 0255 and 0256): one behaviour for the Gather and the Drop Off order of **every worker
  type** and both resources, in
  the phases of `GatherPhase`: ToPatch, Waiting, Mining, ToDropOff, AtHatch, DroppingOff, Delivered. What differs
  between the Drone and the Agent is data: the worker's `WorkerDef` and the drop-off's `StructureDef::drop_off`.
  - **Agents** (M4-D): `WorkerDef::shares_patches` takes a type out of the occupancy below altogether (it neither
    occupies, waits for nor claims a patch, and isn't in `World::patch_mined` or `waited_at`, so a Drone and an Agent
    mine one patch together); with `WorkerDef::drop_off` (`DropOffChoice`) `EachReturn(Nearest::StraightLine)` and
    not `Assigned` (the Drone's) a Gather begins with no AssignedDropOff and each return takes the nearest drop-off as
    it starts (design change 0384: "It is chosen **at every return**, like the Recruit's Storage", replacing 0255's
    choice kept for the Gather; there is no Set Drop-off for the type: the gate's `Sim::sets_drop_off`). "Nearest" is
    from the worker's own centre where it finished mining (design change 0375, answering Q0136: "nearest to the Agent
    itself (straight line) when it finishes mining", the earlier id among equals), to the centre of the crystal point,
    whatever the load (`economy::drop_off::nearest`, the one rule of the
    Drone's and the Agent's return leg and of the removal path): a Tunnel's crystal hatch (design change 0292 point 2:
    "the same measure as for crystals"), an Egress Node's own centre.
  - **Recruits** (M5-D, plan decision T58; `factions/cults/units/recruit.md > Gathering`,
    `factions/cults/structures/storage.md`): data again. `shares_patches`, no `pass_through` (design change 0106:
    Recruits collide while they gather), the Storage's `DropOff::Anywhere`, and
    `DropOffChoice::EachReturn` (design changes 0345 and 0375, answering Q0102 and Q0136): as each return starts
    (`state::Gather::node` is `None`) the Recruit takes the own completed Storage nearest to itself, **edge to edge**,
    where it finished mining (`deliver::nearest_to_worker`; 0375 replaces 0345's "edge to edge from its patch"), the
    earlier id among equals, and with none it idles holding its load. One lost on the way is replaced by the removal
    path as any worker's is. Where two Storages stand side by side under a row of patches, a Recruit whose Silhouette
    reaches over both is as near to both and takes the earlier, whichever face its patch is on (the literal reading,
    reported by slice M5-S2a; `docs/economy_oracle.md > The Cults` measures the reference field of design change
    0385, whose Storages stand so on both sides of its row).
    Within what the design says a face holds (4 to 5 Recruits a 2-tile face)
    nothing jams; with more Recruits than lanes they stand head-on and starve one another, by turns since design
    change 0323's contact step, though one that presses on a face's busy lanes from beside it can stand for minutes.
    The design accepts it (design change 0320, answering Q0110: no give-way rule, more Storages are the player's
    answer); `tests/recruit_gathering/jam.rs` pins it (slice M6-S6a).
    **Measured** (`tests/recruit_gathering/`): one Recruit at gaps of 1, 2 and 3 tiles makes a trip in 24, 56 and 88
    ticks (1.33, 0.57 and 0.36 Space Crystals per second; the design's formula gives 1.45, 0.59 and 0.37); 10 on an
    8-patch field with three Storages, 12.9. `tests/recruits_perf.rs` (ignored, release): 60 Recruits on 22 patches
    with four Storages, a mean of 0.062 ms and a p99 of 0.089 ms a whole sim tick (load average 2.7).
  - **Supplies** (M4-I; `factions/syndicate/units/agent.md > Gathering > Supplies`, design change 0256): a Gather on a
    Supply Delivery Station (`Site::Station`, the station as the owner knows it) runs the same phases, for a worker type
    with `WorkerDef::supplies` (the Agent; the gate's `Sim::gatherers_among`, `resolve`'s station rule). `Mining` at a
    station is the pick-up, PickUpDuration long; `economy::gathering::step` tells a station from a patch by the target
    (`economy::stations`). **No occupancy** (point 1): each worker heads for a free spot of its own round the station
    (`station::spot`: the nearest point against a face, or beside a face's end within reach, clear of known
    structures, of the units its owner sees there and of the spots claimed in this tick, `Claims::take_spot`), since
    Agents collide. **Claims on the Supplies** (point 2): a pick-up begins only while the station's Supplies outnumber
    the pick-ups running as 5a began (`World::unclaimed_supplies`) plus those begun in this tick
    (`Claims::take_supply`); the Supply leaves the station as the pick-up ends (`stations::take`), never before, and
    a pick-up stopped earlier takes nothing (0130). **An empty station** (point 3): the worker waits beside it
    (`Waiting`), keeping its Gather, for ever, and picks up as soon as a Supply is free; one that can't get within
    reach sets off again every `Gathering::recheck_ticks`, for a spot free by then (0141's rule). The pick-up that takes the last
    Supply emits `SimEvent::SupplyStationEmpty { station, owner }` once for each player whose workers' Gather names
    the station (the notification, a placeholder reading of "the owner"). **Switching resources** (point 4; design
    change 0292 point 3): `Worker::resource` says what the load is; a worker with crystals sent to a station (or
    with a Supply to a patch) goes straight there, and loses its load as the new pick-up or mining ends, with the new
    load in hand; one stopped before then keeps it. A worker with a Supply sent to a station turns round there and
    delivers it (0128). **The return** goes to the hatch of the load's resource: Side D for Supplies (design change
    0248), with its own one-at-a-time line, so a Tunnel takes a crystal and a Supply delivery in the same ticks; the
    nearest Tunnel is measured to its crystal hatch all the same, and both reaches are 8 su (design change 0292
    points 2 and 1).
  - **A hatch takes one worker at a time** (`defs::DropOff::Hatches`, the Tunnel; `deliver::at_hatch`). DropOffReach
    is measured to the hatch square. A worker within reach is `AtHatch { since }`: it unloads if nobody did as phase
    5a began (`World::hatch_busy`) or has begun since (`Claims::take_hatch`), and it is the first of the line
    (`World::first_in_line`: those within reach as the phase began, by arrival tick, then id). The line and the busy
    hatch are **derived from the workers' orders in the snapshot**, like a patch's occupancy, so they can't outlive
    an order. A hatch that frees at the end of tick `t` is taken in `t + 1`: a saturated Tunnel takes a load every 64
    ticks exactly, 12.5 Space Crystals per second. The leg back heads for the nearest point against the hatch's
    wall within reach of the square (`legs::approach_point` with `beside`), so waiting workers stand side by side
    (three or four fit). **One that isn't within reach doesn't wait: its leg presses on** (slice M4-W2, design
    change 0315: "Until it is within reach it is never 'arrived': it keeps actively moving toward the hatch"). The
    leg is exempt from the occupied-target rule (`Mover::presses_on`, set by `legs::return_leg` on the leg to a
    hatch only; `docs/movement.md > 6.5`, rule 1), so behind standing Agents it stays a unit on its way, and when
    the mover gives it up (stuck with no attempt left, 104 ticks) the next leg begins in the next tick, for ever
    ("It waits as long as it takes; it never gives up"). `Gathering::leg_retry_ticks` (32) is left for a leg that ends out of
    reach with nothing in its way (the end of its path, or no path), and for the Drones' and Recruits' legs back. A
    worker pushed out of reach loses its place. **0315 doesn't end the jam of four or more Agents that come to the
    hatch along the wall** (`docs/playtest/m4.md > Known issues`, `tests/agent_gathering/jam.rs`).
  - **Hard collision at the hatch** (Agents don't pass through each other, 0106: "Drones only"): from 2 GU between
    hatch and patch on, Agents walk round each other and every one delivers (`tests/agent_gathering/`). At 0 and 1
    GU the strip is narrower than two Agents, each works in a lane of its own, and an Agent behind another in its
    lane blocks it head-on for good. Measured there with three and four Agents: one Agent's income. Open with the
    Designer (see the slice's report).
  - **Occupancy is derived, never stored on the patch** (design change 0133 and the Director's choice): a patch is
    taken while a worker of any player mines it (the snapshot's `World::patch_mined`), once a worker with a lower id
    claimed it in this tick (`gather::Claims`, the one thing of 5a that isn't the snapshot, kept for the tick only),
    and, for any other worker, while one waits at it within MiningReach (`World::waited_at`). A worker on its way
    occupies nothing ("Drones travelling to it do not count"); one that sets off for a free patch claims it for the
    tick, so that no other look in that tick picks it too. Of the workers waiting at a patch the lowest id mines it.
  - **Looking round** (`search::search`; 0131, 0141): among the patches the owner knows (seen or remembered, the
    map's from the start, 0159: `World::known_patches_near`, from its records and sight) that no plate it knows of
    covers, within the worker's `WorkerDef::search_radius` (6 GU, a
    placeholder, centre to centre) around the worker, the nearest free one it can reach (`World::reachable`: a tile
    against the footprint is in the unit's component of its owner's planning grid; the grids leave units out). An
    arrival at a taken patch, or a leg that ends short of one (0141), moves the **order itself** on to that patch,
    so `OrderKind::Gather` names the patch the worker heads for, and after a delivery the one it last mined (0132).
    With none free the worker waits, at its patch or as close to it as it got.
  - **Waiting** (`Waiting { since }`, 0132): the worker mines its patch as soon as it frees, and every
    `Gathering::recheck_ticks` (16, a placeholder), in the ticks that are multiples of it, every waiting worker looks round
    again: it goes to a patch that has freed up, or, short of an unreachable one, tries its leg again. One phase for
    all: counted from each worker's own start, the waiters looked round in different ticks and each went for the
    same freed patch (measured: 0.5 Space Crystals per second less with 20 and with 24 Drones).
  - **Patches by knowledge** (design change 0139, slice M3-R): the behaviour sees its patch as its owner does
    (`World::known_patch`: seen, or remembered at its last-known footprint). A Gather on a patch out of sight walks
    to its last-known spot (`legs::approach_point` on the known footprints, `World::known_fixed`), noting it in
    `Gather::spot` (`note_spot`), and mines it once its owner sees it there; one in reach but not yet seen holds a
    tick. A claim names the claimer's owner, and a patch out of sight counts as taken only by its owner's own claims
    and miners, so nothing of an unseen enemy leaks.
  - **A patch that leaves the game** (depleted, 0134; under a plate, 0137), or is seen gone: the removal path keeps
    its gatherers' orders on it and gives them its centre (`Sim::keep_gatherers_on_spots`, `Gather::spot`;
    `Order::outlives_target`, the one rule of *Combat > Removal path*). Those at the patch look round at once, one
    on its way when it gets to the spot, one carrying a load back after its drop-off ("the search radius is centred
    on the field"). With only occupied patches left it waits at the nearest; with none, the Gather ends.
  - **A Gather ends** only then (0123): the worker delivers what it carries (`Gather::ending`), the order completes
    and the queue goes on; orders Shift-queued behind a Gather, a Deploy as much as any, wait for that. **A Drop Off
    order** is only a delivery (0127): it completes at the drop-off, the worker idling there or starting its next
    order. A worker carrying a load that is sent to a patch walks there, doesn't mine, and returns to deliver
    (0128); other orders keep the load (0129).
  - **The return** reads the worker's AssignedDropOff every tick, so `Ability::SetDropOff` redirects it; without
    one as the return starts it takes the owner's nearest, and with none it idles holding its load (0135).
  - **A drop-off lost on the way** (0170, "switches at once") is replaced by the removal path, not looked for by
    the leg (`Sim::reassign_drop_offs`, like `keep_gatherers_on_spots`): a worker heading for it or unloading
    there takes its owner's nearest at the destruction, by the leg's rule, or none; the others forget it and take
    the nearest as their next return starts (0135). The loss is reacted to once, where it happens, and nothing
    checks every tick that the drop-off still stands. The leg turns in the first 5a after the loss, as it did; for
    a death in 5e the assignment shows from that tick's end.
  - **Legs are moves to a point** the behaviour picks, where the worker stands against the nearest open face of
    its target (`legs::approach_point`), and the behaviour measures MiningReach and DropOffReach itself. Not
    `MoveGoal::Object`: that is planned to the point nearest the object's centre, the north side of every patch
    and node. A leg to a patch that ends short counts as arriving there, out of reach (0141); a leg back to the
    drop-off that ends short begins again after `Gathering::leg_retry_ticks` (32, a placeholder) of standing, a back-off;
    but an Agent's leg back to a hatch presses on instead (design change 0315, above).
  - **Gathering Drones pass through each other** (design change 0106; `gather::on_gather_job`, see *Movement and
    collision > Collision exceptions*): a worker of a type with `WorkerDef::pass_through` (the Drone) whose current
    order is a Gather or a Drop Off. They collide with everything else and, having an order, never yield.
- **Not active during Move**: a Move never engages, and neither does the walk back to an IdleOrigin.
- **Unreachable auto-targets** (design change 0181, "for now"): an idle unit doesn't re-acquire a target it couldn't
  get to for `Orders::give_up_retry_ticks` (32). On an AttackMove or a Patrol such a target is rejected by the reach rule
  instead (0235). A target an explicit Attack gave up (`engage::attack`, the Failure rule) waits the same 32 ticks
  before the idle unit takes it again: the Attack leaves it in `Orders::auto` as the target given up.
- **Placeholders** (the design is silent; each is a comment or a named const in `orders/`):
  a 45° step reaches up to its upper bound; a unit backing off
  from a target inside its MinRange went to the middle of the band from MinRange to Range (now MinRange + 1 grid
  unit, design change 0454), and gives the order up when a back-off ends with the target inside and where it was
  (both written into the design since, by design change 0204, the second with the "Target too close" feedback, which
  slice M5-A added); how a unit that can't aim from where it stands finds its spot, and what one with none does
  (design change 0454 leaves it to the Director; `orders/aim.rs`); a queued Attack or Follow
  whose target, a unit, is out of its owner's sight when its turn comes completes at once, as one lost from vision
  (0200).
- **Auto-targeting switch**: `DebugCommand::SetAutoTargeting { on }` (`Sim::auto_targeting`, hashed state) turns
  every scan off or on, for tests and QA runs about movement between hostile units; `TestSim::without_auto_
  targeting()` schedules it for tick 1. Attack orders work either way.
- **Adding a behaviour**: give it its state in `orders/state.rs` (with its hash, and its ids in `targets` and
  `forget`), start it in `run::start`, run it in `run::run` (`orders/run.rs`), and let it read only the `World` and its
  own unit. A behaviour that sends the unit somewhere must remember that it did (`chasing`, `marching`, `moving`):
  `Mover::status` is the outcome of the unit's latest move, whoever asked for it. Never call `Mover::go_to` tick
  after tick: each call is a path request and a fresh set of repaths.
