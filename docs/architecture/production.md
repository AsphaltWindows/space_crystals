# Production

Read this when you touch producers, queues, Airfields, rally points or how units and buildings are made.

## Production

Producing structures, their rally points and the exit doorways (`production.rs`, `production/`, `sim/producers.rs`,
with the gate's rules in `sim/producers/queuing.rs`, the release in `sim/producers/release.rs` and the step in
`sim/producers/step.rs`; slice M2-D2, the army caps at production start slice M3-P;
`factions/gdo/structures/barracks.md`, `core/entities.md > Structure Type > Exit Doorway`; the Colonists' portals, slice
M3-D: `factions/colonists/structures/robo_portal.md`, `factions/colonists/structures/bio_portal.md`, plan decision T19;
their energy doorway, slice M3-D2: design change 0145, `factions/colonists/structures/README.md > Portals > Portal Exits
(energy doorway)`).

- **State**: `Object::production` (`Production`: the BuildQueue, whose head is the unit in production, so the
  limit `Production::queue_limit` (20) includes it, design change 0081; its entries `QueueEntry`s, a unit or a pool building's
  Release, slice M5-F2, an Altar's augmentation order, slice M5-N1b, or a Temple's research, slice M5-N3, a unit's
  written as its kind alone so the
  files and hashes from before are as they were; its
  progress in `RATIO_ONE`s, a Recruitment Hall's in tiles; the rally point; a Hall's completion tick, `completed_at`,
  hashed and saved only while not 0; a pool building's `reserved` units, entry by entry, hashed and saved only while
  any). What a type produces is `StructureDef::production` (`ProductionDef`: its exit rule, `ExitRule`, the
  `Recipe`s: kind, Space Crystals, frames, and for a Hall its `RecruitmentDef`). `Sim::production_status(id)` is what
  the interface shows: the entry, the progress out of its total, `waiting` (finished, held for the doorway) and
  `blocked` (the army cap the unit
  at the front waits for at 0%, design change 0143). `Production::blocked`, the stored mark of that wait, is hashed and
  saved only while true (`#[serde(default, skip_serializing_if)]`, as `Mover::momentum` is while non-zero), so that the
  games without a wait, the goldens among them, keep their hashes.
- **Commands**: `Command::Production { objects, command }` with `ProductionCommand::Train(kind)`,
  `CancelProduction` and `SetRally(Target)`, a pool building's `TrainFrom { kind, from }` and `Release(from)`
  (slice M5-F2), an Altar's `Augment { kind, from, order }` and `ReleaseOf { kind, from }`, of a unit of a type
  and a kind (slice M5-N1b), and a Temple's `Research(tech)` (slice M5-N3); the gate's rules are in *Commands, events
  and replays*.
- **Costs** (plan decision T7): a unit's Space Crystals are paid when it is queued, and only they and room in the
  queue are checked then. Its army cost (Unit Control, Beacon Capacity; see *Economy*) is taken when its production
  starts (design change 0143, which replaces 0082's reservation at queue time, for every faction): no `Reject` or
  `Refusal` is about the army cap. Cancelling refunds in full (`Refund::Full`) and takes the last entry, a unit
  waiting for room included. When a producer leaves the game, destroyed or removed, `Sim::remove_objects` refunds
  its whole queue the same way, a finished unit waiting inside included (design changes 0084, 0085;
  `Sim::refund_production`); of the cap it frees only what its unit in production took.
- **Group production** (design change 0083): a `Train` to several producers serves them by the work they have
  queued, least first (`Production::remaining_work`: remaining production time, a placeholder reading), ties by
  id, each check counting what the ones before it took, until the crystal check fails (0143: the army cap plays no
  part). A full queue is passed over. The others get `SimEvent::ProductionRefused`.
- **The start check** (design change 0143, `sim/producers/step.rs`): in the producers' step, before a unit at 0% at
  the front of its queue advances, its owner's `ArmyCap::fits(cost)` (`cost == 0 || available − used ≥ cost`: a
  unit without a cost always starts, even over the cap, design change 0176 point 3) must hold, with
  the caps worked out once a step for the players who need them and the units that start in the step added as they
  do. Without room the producer stays at 0%, `Production::blocked`, and checks again every tick; the tick the wait
  begins it raises `SimEvent::ProductionWaiting { producer, reason }`, once (placeholder for 0143's notification,
  whose system is TBD). Producers waiting for the same room are served in id order, the step's, lowest first
  (design change 0175). A unit in production is never checked again: it always finishes (Q0041 point 2). At a power
  ratio of 0 a unit that passes makes no progress, so it neither starts nor takes anything, and another producer may
  take the room (design change 0176 point 2).
- **The step** (`Sim::step_production`, see *Tick pipeline*): the start check, then the unit in production advances
  by `PowerSummary::speed` (design change 0089), capped at its total. Trained in tick `t` at full power, a
  Peacekeeper (80 frames) is released in tick `t + 79`; at 30/51 in tick `t + 135`; a Drone or a Sabre Legionnaire
  (160 frames, the Colonists have no Power) in tick `t + 159`. The next one starts from 0 in the tick after a
  release.
- **The Recruitment Halls** (slice M5-C, plan decision T57; `recruitment.rs`, `sim/recruitment.rs`;
  `factions/cults/structures/recruitment_hall.md`): a Hall's production is the producers', in an automatic mode that its
  `ProductionDef::recruitment` (`RecruitmentDef { area: 10, capacity: 20 }`) switches on. **The claims**
  (`recruitment::Claims`, derived each time, never stored): each Recruitable tile (`TilePreset::recruitable`: every
  default preset but Water) of a Hall's 10 × 10 area centred on its footprint goes to the first completed Hall whose
  area has it, by `Production::completed_at`, and of Halls completed in the same tick the one whose centre is nearer
  the tile, then the lower id (design change 0341 point 3), whoever owns it (0341 point 5); buildings never matter;
  the area may reach past the map's edge (design change 0387, slice M5-S4, in place of 0341 point 1's whole area on
  the map, which the placement check no longer asks): its tiles off the map count among its 100 and are never
  Recruitable (`recruitment::recruitable`), so a Hall flush against an edge has 70% at most, one in a corner 49%; a
  Hall under construction claims nothing (0341 point 4, `Object::is_operational`). **One rule of claiming** (slice
  M5-J2, `recruitment::winners`): each competitor (`Contender`) ranked, every Recruitable tile of the areas to the least by
  (rank, distance² from its centre in half tiles, id); the game's claims rank the completed Halls by `completed_at`
  (`Claims::of`, `claimed_tiles` for a selected Hall: `Sim::claimed_tiles`), and **the placement preview**
  (`recruitment::preview`, `factions/cults/structures/recruitment_hall.md > Recruitment Area display`, design changes
  0350 to 0352) ranks an explicit list of `Claimant`s, a pure function: `Rank::Completed(tick)` first, then
  `Rank::Pending(n)` (own sites and ghosts in their order of placement, 0351), then the Hall placed;
  `Preview::of` gives each tile of the placed area `Claimable`, `Taken { claimer: Own | OwnPending | Other, by }` or
  `NotRecruitable` (known non-Recruitable ground, off the map; ground the player never explored passes, as in
  `Mode::Known`), and `effectiveness_percent`, rounded down (`recruitment::effectiveness_percent`, shared with
  `HallState`). Who the claimants are is the caller's: the client gathers what its player knows
  (`placement::recruitment_areas::known_claimants`). `PendingBuild::group`, its order's `OrderGroup`, gives the order
  the ghosts were placed in; `Sim::pending_without` is public for the display. `completed_at` is the tick
  `Capabilities::fresh` gives the producer, so a spawn's
  insertion tick, 0 for a map's, and the tick `Sim::start_functions` completes a site (slice M5-E). **The start**
  (`Sim::start_recruits`, first in the producers' step): a completed Hall with an owner and an empty queue queues its
  recipe's Recruit, free, while its usage (`recruitment::Lineages`, its Recruit in production counted: design change
  0342 point 3) is below its capacity, BaseUnitControlCapacity × claimed / 100 rounded down (`recruitment::Capacity`,
  design change 0341 point 2: 33 tiles stop at 6), whatever the global Unit Control (design change 0344); a Hall that
  releases its Recruit queues the next at once (`Sim::recruit_next`), so it shows 0% rather than nothing between
  them, and one at capacity starts again from 0% in the tick a unit of its lineage is lost (0342 point 4). **The
  progress**
  gains the claimed tiles each tick, whatever the power, to a total of 192 × 100 (`production::total_of`): 192 ticks
  a Recruit at 100 tiles, 384 at 50, the k-th of a full Hall in tick 192 k (6 → 20 in 2688 ticks). **The release**,
  out of its one exit side, B (design change 0342 point 1), sets the Recruit's lineage to the Hall. **No X** (0342
  point 2: "Recruits can't be cancelled"): a `CancelProduction` finds nothing to cancel at a Hall
  (`Reject::NothingQueued`), and its panel has no X. A `Train` to a Hall is `CannotProduce`.
  `Sim::claims` and `Sim::hall_state` (`HallState`: claimed and area tiles, effectiveness, usage, capacity) are the
  interface's.
- **Pool buildings** (slice M5-F2, plan decisions T61, T70 and T71; `sim/producers/pool.rs`, `production/queue.rs`;
  `factions/cults/structures/armory.md > Training`, `factions/cults/structures/README.md > Choosing Recruits, Release
  and Eject All`; design changes 0332, 0337, 0347 and 0348): a producer that also holds units (`holding::trains`: a
  `StructureDef` with both `holds` and `production`), the Armory first, makes its units of the units inside, one path
  for every such type, so that the Cathedral, the Chop Shop and the Airstrip are data. **An entry**, `QueueEntry::Unit`
  or `QueueEntry::Release` (`frames`: a recipe's, a Release's `Production::release_frames`, 4, the design's "very fast (**4
  frames**, placeholder)"; `cost`: a recipe's Space Crystals, a Release none), **reserves its units when it is
  queued**, `Production::reserved` beside the queue, entry by entry (`QueueEntry::units_reserved`: the unit's Unit
  Control cost, at least 1; 1 for a Release): the free units of its kind (`Sim::free_units`, `RecruitKind`, those
  that entered first), else `Refusal::NoRecruit`. They stay inside, out of the pool (`Sim::pool`), and the capacity
  counts them (0347 point 5), so the capacity alone bounds the queue: no `Production::queue_limit` for a pool building. **The
  front** advances a frame a tick, the Cults having no power, and finished, its first unit is ready to leave
  (`Sim::ready_to_leave`, `Held::leaving`), the entry waiting at 100% while the units ready before it go out first
  (*Object model > Buildings that hold units*). **Out** (`Sim::let_out`), a training's unit is the Recruit changed in
  kind (`Sim::train`, plan decision T15: its id, its lineage): infantry keep its damage (`transform::follow_max`) and
  its Augmentations, vehicles and aircraft come out at full HP without them (design change 0332,
  `factions/cults/augmentation.md > Persistence`), and a load goes (0347 point 3); the other units it reserved leave
  the game into it, held, with no `Died` and their lineages appended, so the Halls' usage stays as it was. The entry
  then leaves the queue, and the next starts in the next tick. A Release's unit comes out unchanged. **X** takes the
  last entry back, refunding its Space Crystals and freeing its units into the pool (`Sim::unreserve`); a pool
  building that leaves the game refunds every entry but a finished one, whose unit comes out as the trained unit
  (design change 0349 point 3, slice M5-F3), its other units evacuated as they are (*Object model > Buildings that
  hold units*). **Invariant R1** (`testing/invariants/reservations.rs`): only a pool building reserves, an entry
  each, each of its units held in its building and reserved once, and only the front's first may be leaving. The
  Gunner's recipe requires a Gunworks (slice M5-G2), checked as it is queued and never again
  (`tests/armory/gunner.rs`: losing the last Gunworks keeps the Gunners queued and in training).
- **The Altar** (slice M5-N1b, plan decisions T70 and T71; `factions/cults/augmentation.md > Altar`, design changes
  0332, 0336, 0337 and 0348; `augmentation/orders.rs`, `sim/producers/pool.rs`, `sim/producers/queuing.rs`): a pool
  building as data (`defs/cults/structures/altar.rs`: the four Cults infantry types in at B, 10 of them, out at C), no
  recipe, its entries the orders of `ProductionDef::augment` (`AugmentRecipe { order, cost, frames }`: 25 Space Crystals
  and 48 frames each for Enhanced Constitution and Improved Metabolism, 50 and 96 for Both, placeholders, 0391) and
  Releases. **An order**, `QueueEntry::Augment { order }` (`AugmentOrder`: `fits(kind)`, "missing every Augmentation
  the order applies", and `onto(before)`), is paid and reserves its unit when queued, as a training does: the free
  unit of the type and the kind that went in first (`Sim::free_units_of`, design change 0332), the queue bounded by
  the capacity, the reserved units counted. The gate (`Sim::plan_augment`) refuses `AlreadyAugmented(kind)` for an
  order the kind doesn't fit, `CannotAugment(order)` where no producer has the recipe, and else serves the Altars as a
  training (0083): `QueueFull`, `NoRecruit(from)`, `InsufficientCrystals`. **Out**, the unit gains the Augmentations
  through `augmentation::set` (`Sim::augment`, from `Sim::complete`), its HP following its MaxHP; no level is stamped
  on it (design change 0336: its owner's levels are read whenever its stats are). `ReleaseOf { kind, from }` is the
  Armory's Release of a unit of a type (0337: in every state, a fully augmented unit's only way out but Eject All).
  X and a destroyed Altar refund as at the Armory: the unit under way comes out unaugmented, and one finished and
  waiting for the doorway as the building goes comes out augmented, unrefunded (**Placeholder**, read as 0349 point
  3's training). `Sim::pool_counts_of(building, unit)` counts its free units of a type by kind, its panel's counts.
  Tests: `crates/sim/tests/altar/` (`orders.rs`: one order of each, timings and stats, a hurt unit's damage, first in
  first augmented; `refusals.rs`; `release.rs`: 0337's Release and a fully augmented unit; `cancel.rs`;
  `destruction.rs`; `entering.rs`: the four types and the right-click; `training.rs`: an augmented Recruit trained;
  `saves.rs`), each tick under the invariants.
- **The Cathedral** (slice M5-N4a; `factions/cults/structures/cathedral.md > Training`: "Works like the Armory's
  Training"; design changes 0328, 0332, 0334, 0337 and 0348): the pool path as data, with no code of its own
  (`defs/cults/structures/cathedral.rs`: Recruits in at B, 10 of them counting the reserved, out at C; its one recipe,
  the Disciple, of 1 Recruit, 75 Space Crystals and 320 frames): its training, Release, Eject All, X's refund and
  destruction are the pool building's above. Recruits build it while their player owns a Temple
  (`StructureDef::requires`, `Reject::Requires(Temple)` as the Build is given); losing the Temple later cancels
  nothing. Its Disciple keeps the Recruit's damage and its Augmentations (infantry) and comes out with its type's
  starting energy (*Combat > Energy*). Tests: `crates/sim/tests/cathedral/` (`training.rs`: Q's price and frames, the
  energy to 200 in 300 s, first in first trained, X's refund, the gate's refusals, Q to three Cathedrals; `pool.rs`: a
  Release, Eject All, the capacity, a destroyed Cathedral, the right-click's Enter; `disciple.rs`: the Recruit's damage
  and Augmentations, no weapon, an Attack refused beside a Militant's; `temple.rs`; `saves.rs`) and
  `crates/sim/tests/energy/` (the regeneration exact to the tick, held units', the hash and the saves), each tick under
  the invariants.
- **The Chop Shop** (slice M5-N6; `factions/cults/structures/chop_shop.md`, `factions/cults/units/seeker.md`, design
  changes 0330, 0332, 0337, 0347, 0348 and 0353): the pool path as data (`defs/cults/structures/chop_shop.rs`: Recruits
  in at B, 10 of them counting the reserved, out of side C, `ExitRule::Side(C)`; its one recipe, the Seeker, 75 Space
  Crystals and 320 frames). An entry reserves the Seeker's Unit Control cost, 2 Recruits of the kind
  (`QueueEntry::units_reserved`), so 10 Recruits allow 5 entries and one free Recruit is `NoRecruit`. **Out**, the
  first is the Seeker (`Sim::train`), at full HP with no Augmentation, and the second leaves the game into it, its
  lineage appended: a point of the Seeker's Unit Control at each Recruit's Hall, half of it orphaned once one Hall
  goes. **A vehicle in a side doorway** turns its box with the door: the half of its silhouette facing out
  (`producers::exit_half`) is what `Sim::held_spot`, `Sim::release_unit` and the evacuation's `footprint_spot` place,
  so a Seeker out of a north or south side is 28 su deep along its way out, not 20. It comes out half inside, facing
  out, and the movement's doorway walk drives it straight out (`movement/doorway.rs`, slice M5-V2), to its rally point
  or, with none, to a stop just clear of the doorway; an idle vehicle doesn't yield (0353), so the next Seeker stands
  in the doorway behind it and the queue waits until the player moves the first (**the implementer's reading**, a jam
  for the playtest notes). (M5-S8) **It starts with a rally point** 3 grid units out of side C (*Production > Rally
  points*, design changes 0446 and 0447, answering Q0175), so that stall comes only with the rally point cleared ("A
  cleared vehicle building leaves the stall to the player"); left alone, three Seekers parked between the point and the
  door (the first on it, the others 62 and 126 su behind), and a fourth stopped in the doorway behind them, until
  (M6-GF) design change 0451: the vehicles it trains park round the point (*Production > Parking round a taken rally
  point*), and five leave without the doorway stalling. Recruits build it while their player owns an Armory, and it is the Machine
  Shop's and the Airstrip's TechPrerequisite. Tests: `crates/sim/tests/chop_shop/` (`training.rs`: 0319's NoRecruit,
  the price, the frames and the rally point, the queue of 5, X, Release and Eject All, the right-click; `exit.rs`: the
  four orientations, no rally point (cleared first since slice M5-S8: 0447's Verification), the doorway behind a
  stopped Seeker; `rally.rs` (slice M5-S8): the default rally point in the 16 orientations, a site's from its
  placement through its completion, three Seekers to it with the doorway never stalling, a moved one, the map's edge
  and water, a save with none; `parking.rs` (slice M6-GF): 0451's Verification, five Seekers, five Hands of Heaven
  and five Seekers rallied farther away, each time clustered round the point; `seeker.rs`: full HP and no Augmentation, the Unit
  Control; `combat.rs`: a Militant killed on the move, the DirectionalArmor's three sides, Rugged terrain;
  `destruction.rs`; `normal_game.rs`: an Armory, a Chop Shop and a Seeker from the Cults' start, the Machine Shop and
  the Airstrip no longer greyed; `saves.rs`) and `tests/repair/vehicles.rs` (the Seeker repaired by the real recipe),
  each tick under the invariants. (M5-N11) **Its second recipe**, the Prayer-mobile (125 Space Crystals, 400 frames,
  `requires: &[Temple]`, design change 0444), trains the same way, 2 Recruits of the kind, refused
  `Reject::Requires(Temple)` without a Temple; it comes out with its type's 50 energy (`crates/sim/tests/prayer_mobile/
  production.rs`). The client puts it in the KindMenus' E (`PanelTemplate::trains`), W left to the Hand of Heaven.
- **The Factory** (slice M6-GF; `factions/gdo/structures/factory.md`, `factions/gdo/units/raptor.md`,
  `factions/gdo/units/twister.md`, design changes 0366, 0441, 0446 and 0451): "Production as the Barracks" (its
  queue of 20, the refunds, the rally), out of side B (`ExitRule::Side(B)`, the Barracks' exits), with the default
  rally point of a vehicle building (`default_rally: Some(3)`, *Production > A vehicle building's default rally
  point*). Its recipes, in its `Produces` order, are the Raptor (100 Space Crystals, 240 frames) and the Twister (125
  and 1 Supply, 320 frames: GD's second cost, `Recipe::supplies`), Unit Control 2 each, reserved as production starts;
  the Prospector and the Artillery Rig stay held (`defs::PLANNED_RECIPES`) until slices M6-GP and M6-GA, the client's
  E and R greyed meanwhile. The Deployment Center constructs it behind a Barracks, Advanced Mechanics behind it and
  the Industrial Powerplant (Power +60) behind Advanced Mechanics, each its def module's `RECIPE` in the
  Deployment Center's `constructs`, in the design's order, its TechPrerequisite its own `StructureDef::requires`.
  A vehicle comes out half inside the middle of side B, its box turned out of the door (`producers::exit_half`), and
  drives to the rally point, where (design change 0451) it parks round it if it is taken (below). The Raptor (a
  WheeledVehicle on FixedTurnRadiusMovement, an 80 su minimum turn radius, 10 su a frame forward) fires a homing
  missile from a 60° turret, so it takes slice M6-SV's rule for a narrow turret (`orders::engage`'s `rests_in_arc`,
  *Combat*) and slice M6-S6a's repositioning from a target inside its turning circle (`orders::aim`, *Behaviours*);
  the Twister's 360° machine gun hits air only, and an Attack on a ground unit by Twisters alone is refused
  (`Reject::CannotHit`, *Commands*). Tests: `crates/sim/tests/factory/` (below) and the golden `gdo_vehicles`.
- **The Airfield** (slice M6-GH1; `factions/gdo/structures/airfield.md`, `factions/gdo/units/hornet.md`, design
  changes 0435 and 0441; plan decisions T88 and D34): its recipes are the Hornet, 150 Space Crystals and 2 Supplies
  (`Recipe::supplies`), 320 frames, and (slice M6-GL) the Dragonfly, 175 Space Crystals and 4 Supplies, 320 frames,
  behind Propulsion Labs (its Unit Prerequisite, `Recipe::requires`: `Reject::Requires`), which the Deployment Center
  constructs (200 Space Crystals, 400 frames, behind an Airfield); (slice M6-GH2) the Deployment Center constructs the
  Airfield (250 Space Crystals, 480 frames, behind Advanced Mechanics).
  **Aircraft** (`aircraft.rs`, `Object::aircraft`): a unit whose type has ammo (`UnitDef::ammo`, an `AmmoDef`: its
  max and its RearmTimePerAmmo, the rearm's) comes into the game full (`aircraft::fresh`) and unassigned.
  **Assignment**: the Airfield that trains it is its base (`AircraftState::base`). An Airfield (`StructureDef::airfield`, an
  `AirfieldDef`: capacity 4) holds at most 4, the aircraft in its queue counting, counted from the game and never
  stored (`aircraft::taken`, `aircraft::room`): the production gate refuses one more (`Reject::AirfieldFull`,
  `Refusal::AirfieldFull` for a passed-over producer). `OrderKind::Rebase(airfield)`, an aircraft's right-click on
  another own Airfield, reassigns it as it is given (`Sim::rebase`, from `Sim::give_order`) if that Airfield has
  room, and flies it there (`orders/air_base.rs`); `OrderKind::ReturnToBase`, its F or a right-click on its own
  Airfield, flies it home. Either ends in a landing on one of its slots and a rearm (slice M6-GH2, below). There is
  no manual unassigning: the removal path unassigns an Airfield's aircraft as
  it goes (`Sim::remove_objects`), and nothing reassigns them. **Ammo**: each missile spends a round as it is
  launched (`combat::phases`). With none left the unit can't attack (`aircraft::armed`): it acquires no target, is
  no threat in the others' scans (`combat::threat`), its right-click attacks nothing
  (`orders::resolution::Abilities`), and the gate refuses it an Attack or an Attack Ground (`Reject::OutOfAmmo`).
  **The automatic return**: phase 5e hands back those whose last round went (`combat::step`'s `emptied`), and each
  that still has a base gets a Return to Base in place of its orders (`Sim::send_home`, *Tick pipeline*;
  **placeholder**: the orders queued behind are dropped, as the Specter's), which any order interrupts, the ammo
  staying at 0. **The Hornet** (a HoverCraft, braking by drag alone: *Movement and collision > Movement models*)
  fires 5 DoublyDisjointed missiles, one every 32 frames, each landing where its target stood and (slice M6-GH2,
  design change 0458) bursting there, 45 over 32 su on the ground only (`TargetType::AoE`, `BlastDomain::Attack`:
  each object in the circle hit by its share, *Combat > Area damage*; it was 30 on the one object covering the
  spot), and turns in place to aim (no turret). It comes out landed on a free slot (`ExitRule::Pad`, below). Tests:
  `crates/sim/tests/airfield/` (below). **The Dragonfly** (slice M6-GL; `factions/gdo/units/dragonfly.md`, design
  changes 0366, 0435 and 0457) is a Glider with one HeadDisjointed missile, 90 on air only (SingleTarget, 0457), and
  RearmTimePerAmmo 64: with no turret it fires along its heading (*Movement and collision > Movement models*,
  GliderMovement), its SeparationRadius 48, pushed at a Glider's third. It lands and takes off vertically (below).
  Tests: `crates/sim/tests/dragonfly/` (below).
- **Landing and rearming** (slice M6-GH2; `landing.rs`, `sim/landing.rs`, `orders/air_base.rs`; plan decision T88,
  `factions/gdo/structures/airfield.md > Landing slots`, `factions/gdo/structures/airfield.md > Returning and
  rearming`, design changes 0428 and 0423):
  an Airfield has 4 **slots**, the centres of its footprint's quarters (`AirfieldDef::slot_offset_su`; the layout a
  placeholder), and who is on which is derived every tick, never stored (`landing::claims`): those on it, landing or
  down, and those rising over it keep the free slot nearest them, in id order; then those flying home to it, in id
  order, take the lowest free one. A Return to Base or a Rebase flies the aircraft to its slot, where, come to rest
  (`Air::landing_reach_su`), it **lands** in 16 frames (`AirfieldDef::landing_ticks`), holding still, out of the air
  units' separation (*Movement and collision*); its `AircraftState::landing` is its state from that frame to its
  take-off. Its **domain** is Ground only while it is down (`combat::object_domain`, every reader of a domain asking
  it: the scans, ValidTarget, the blasts, the gate's `CannotHit`, the right-click, the client's too); landing and
  taking off, Air. Down, it **rearms** in the economy phase, a round every RearmTimePerAmmo at its owner's Power's
  speed (`landing::rearm`), and **stays landed**: its flight home ends once it is down and full
  (`orders::air_base::run`). **Any other order lifts it off** as it starts (`landing::take_off`, from
  `orders::world::Unit::take_off`): its landing ends, with the ammo rearmed so far (the round under way lost), and it
  rises for as many frames as it landed in (`Mover::rise`, slice M6-SD's), an air target; an order given while it
  lands rises from as low as it came (`landing::rise_from`, a placeholder). On a slot it holds fire
  (`aircraft::may_fire`, a placeholder). **A destroyed Airfield** takes the aircraft on it, landing or down, along
  (`Sim::landed_on`, on the removal path, each with a `Died`); one taking off escapes, unassigned as the others. L1
  checks it (*Movement and collision > Invariants*). (slice M6-GL) **A Glider**, which can't come to rest, flies its
  **landing approach** to its slot instead (`sim/landing.rs`'s `glide_home`, `Mover::approach_to`;
  *Movement and collision > Movement models*, GliderMovement): it lands once it is over the slot
  (`Mover::is_over`) and within reach, held still from then on; with no slot free it flies to the Airfield's centre
  and circles there (**placeholder**) until one is. Tests: `crates/sim/tests/landing/` (below), (slice M6-GL)
  `crates/sim/tests/dragonfly/`, and the goldens `gdo_air` and `gdo_dragonflies`. The client draws an Airfield's pads
  (`Look::Pads`, `object_render/pads.rs`), a landing aircraft sinking to the ground and one down on it without its
  shadow (`object_render/flight.rs`).
- **The Supply Tower** (slice M6-GS1; `supply_run.rs`, `sim/supply_run.rs`, `orders/supply_run.rs`,
  `economy/supply_run.rs`, `sim/gate/supply_run.rs`, `defs/supply_run.rs`; `factions/gdo/economy.md > SupplyTower`,
  `factions/gdo/units/supply_chopper.md`, design changes 0423, 0435 and 0436): the Deployment Center constructs it
  behind a Radar Station (150 Space Crystals, 320 frames). Placed, it comes with one Supply Chopper
  (`SupplyTowerDef::chopper`), down on it and attached, at no cost (`Sim::spawn_attached_chopper`); one a map or a
  debug spawn places comes alone (a placeholder). It trains more (100 Space Crystals, 160 frames), out over its centre
  in the air, and has a rally point. **A spot** to land on: a Tower's at `SupplyTowerDef::spot_offset_su` from its
  centre, a station's its centre (`supply_run::spot_of`, a placeholder), one chopper on it at a time. The chopper's
  three orders (`orders/supply_run.rs`) fly it there and land it in 16 frames (`SupplyChopperDef::landing_ticks`, GH2's
  `Landing`, its `at` the Tower or the station): **Pick Up Supplies** takes, as it touches down, all the station holds
  that no pick-up has claimed (M4-I's claims: Agents and choppers share a station), and lifts off (a placeholder);
  **Drop Off Supplies** pays its load in at an own Tower (`SimEvent::ResourcesDelivered`) and lifts off unless the
  Tower is its own; **Attach to Tower** attaches it as the order starts, evicting any other chopper landing or down
  there, and lands it for good. Any command breaks the attachment (`Sim::detach`); no other chopper lands on a Tower
  that has one attached (`Reject::TowerTaken`). The right-click (`orders::resolution`, `Rule::SupplyRun`): a station
  gives Pick Up, an own Tower another chopper isn't attached to Drop Off while carrying and Attach otherwise. **Schedule
  Deliveries** (`Command::Schedule`, the Tower's S) sets `Object::schedule`: the chopper attached and down departs once
  the station's next delivery is due within 2L − 1 + F + 4 + F/16 ticks, F the flight's estimate by the drag model
  along the straight line (`movement::flight_ticks`), L the landing's 16: it touches down in the delivery's first tick,
  after a short hover over the station; if the delivery is due already ("too far or too frequent") it departs as soon
  as it is home. One chopper flies to a station at a time, the closest Tower's first. Down on its own Tower a damaged
  chopper is **repaired free**, at the repairers' rate (full in 80 frames, design change 0435), slowed by its owner's
  Power (`PowerSummary::speed`, design change 0423: a reading). It is an air target but down (`landing::of`). The GDO
  starts with 1 Supply (design change 0436). L1 checks the spots (*Movement and collision > Invariants*). Tests:
  `crates/sim/tests/supply_tower/` (below) and the golden `gdo_supplies`.
- **Research** (slice M5-N3, plan decisions T70 and T74 as design changes 0335 to 0338 amend them;
  `core/tech_and_research.md > Research`, `factions/cults/structures/temple.md`; `defs/cults/research.rs`,
  `defs/structures/research.rs`, `sim/producers/research.rs`): the technologies are data (`Tech`, its `TechDef`: name,
  line, level, Space Crystals, frames, the previous level it `requires`, the structures it `requires_structures`; the
  `ResearchLine`s, each an Augmentation's two levels). A structure type researches by `StructureDef::research`
  (`ResearchDef { techs, queue }`: the Temple's four, queue 5) **on its production queue**, as T70 has it, not a second
  one: `QueueEntry::Research { tech }` (serde `{"tech": ...}`), paid when queued, X's full refund of the last entry, a
  destroyed Temple's refund of every entry and the evacuation of its staff, all the queue engine's. **Progress** is one
  `RATIO_ONE` a frame for each unit staffing the building (`Sim::staff`: held and not on its way out; "Linear
  scaling", "Pauses when empty ... (no decay)"), exact in integers: Level 1's 960 frames take 320 with three Recruits;
  a type that holds no one would research at one a frame. **The gate** (`Sim::plan_research`), in T74's order:
  `CannotResearch` where none of the producers offers it, `Researched`, `ResearchUnderWay` (queued or in progress at
  any of the player's structures, `Sim::research_site`), `RequiresTech(previous)` until the previous level has
  **completed** (Q0117), `Requires(Cathedral)` for Level 2, checked as it is queued and never again, then the Temple
  with room in its queue and the least work (0083's order, ties by id), `QueueFull`, `InsufficientCrystals`. A
  Temple has no rally point: `SetRally` leaves research structures out (`NoProducers` for a Temple alone), and its
  staff step out of C and stop (**Placeholder**). **Completion**, in the producers' step (`Sim::complete_research`):
  the technology joins `Player::techs` (a `BTreeSet<Tech>`, hashed under a tag and saved only while non-empty), and
  every unit of the owner's with the line's Augmentation, on the map or held anywhere, follows its MaxHP before and
  after (`augmentation::follow_max`: 0336's "the existing Recruit's MaxHP rises from +20 to +40, and its HP rises by
  20"); speed and weapon follow by themselves, read through the accessors. It comes before `Sim::step_holding`, where
  an Altar's unit is augmented, so a unit augmented in the tick a level completes gets it (T74's order; with fixed
  amounts either order gives the same). Losing the Temple or the Cathedral revokes nothing. `SimEvent::ResearchQueued`,
  `ResearchCancelled { refund }` and `ResearchCompleted` say so. Tests: `crates/sim/tests/research/` (`staffing.rs`:
  960, 480, 320 and 160 frames with 1, 2, 3 and 6 Recruits, the pause without staff; `queue.rs`: the price, X, a
  destroyed Temple, the refusals; `levels.rs`: Level 2 after Level 1 and with a Cathedral, one Temple of a group;
  `retroactive.rs`: 0336's Verification, held units, units augmented later, the same tick as an Altar's order, walking
  units; `doors.rs`: in at B, out of C one at a time, A/S/D/F; `saves.rs`; `neutral.rs`: 0379's Verification), each
  tick under the invariants, T1 among them, whose negative tests are in `tests/invariants_state.rs`.
- **The exit doorways** (`production::ExitDoor`, the one geometry for the release, the movement step and the
  invariants; `ExitDoor::of` reads the building's `ProductionDef::exits`, an `ExitRule`, against its sides as
  placed, `sides`). Two kinds:
  - **A side doorway** (design change 0080; `production::Doorway`, `ExitRule::Side(B)`: the Barracks' one side B):
    a door (`Door`) in the middle of that side, wherever it faces. A finished unit comes into the game in it **half
    inside the building**: its centre on the exit side, facing out, with `Mover::exit` set (see *Movement and
    collision* for the collision exception). When another unit or a blocked tile is right in front of the door it
    stands as far out as there is room, fully inside against the exit side at most (`Sim::doorway_spot`;
    `Door::room_before` counts only what is in front, not what stands behind the door or beyond the building): the
    design's "half inside" would overlap what stands there, and its verification wants the unit to appear all the
    same. (M6-GF, plan decision T100) The tiles it checks are those of the grid its type walks on, by its Silhouette
    (`paths::unit_grid`, `paths::key_of`), not by its box as turned out of the door, which may be a hair longer (a 64
    su one 64.000008 su out of a side facing west) and would name the next size class's grid, one no unit walks on:
    the 64 × 48 Artillery Rig's case, `sim/producers/tests.rs`. **Several exit sides** (slice M5-C,
    `ExitRule::Sides(A)`: the Recruitment Hall's four, until design change
    0342 gave it one, B, so that no type has them now; `core/entities.md > Structure Type > Exit Doorway`: "a side
    whose doorway is occupied counts as blocked, and the building pauses only when every eligible doorway is
    occupied"): a door on each, tried in `Doorway::order`, the side facing the rally point (the greatest cosine with
    the direction from the centre; of two alike, a diagonal, the clockwise one), the bottom one without a rally point,
    then clockwise (Placeholder (not in the design: no building has several exit sides now)); the unit
    takes the first door free of a released unit that has a spot for it. A unit in the doorway is in the door whose
    side it is least inside of (`Doorway::door_at`), which the movement step, the paths and the invariants ask.
  - **A portal's energy doorway** (slice M3-D2, design change 0145; `production::EnergyDoorway`,
    `production/exit/energy.rs`; `ExitRule::Centre(&[A])`: the Robo Portal's and the Bio Portal's A sides, Q0041
    point 5): a finished unit **materializes at the footprint's centre**, facing its heading
    (`Mover::exit_heading`, set and cleared with `exit`, hashed and saved only while set), and walks straight out
    along it (see *Movement and collision > The exit doorway*). **The open directions come from the sides**,
    never from a building's name: a direction is open when the ray from the centre leaves the footprint's box
    through an open side (`EnergyDoorway::margin`: how far out the ray reaches the farthest open side's line less
    the farthest closed side's, at least 0; `is_open`). The arc's two ends, exactly the diagonals of a square
    footprint, are open (**the Director's reading**). The Robo Portal (AAAA) is open all round; the Bio Portal
    (AABB) from its top-left corner's direction clockwise round to its bottom-right one unturned, turning and
    flipping with it. The Heavy and Star Portals' arcs will come from their sides the same way.
  - **A runway** (slice M5-N7, plan decision T71; `production::Runway`, `production/exit/runway.rs`;
    `ExitRule::Runway { from: B, toward: C }`, the Airstrip's; `factions/cults/structures/airstrip.md > Placeholder
    stats`: "a finished aircraft is placed **directly on the runway** and takes off along it toward C"): only for a
    building that holds units (`defs_are_consistent`: `from` and `toward` opposite sides, the holder's exit a side
    doorway on `toward`). Its trained aircraft takes off the moment its training is finished, in the holding step
    (`Sim::step_holding`, `sim/holding/takeoff.rs`: `Sim::takes_off`, `Sim::take_off`), never waiting for the doorway
    ("Aircraft [...] are not covered by this rule", `core/entities.md > Structure Type > Exit Doorway`): it appears
    wholly over the runway against the `from` end, facing along it, at its IdleSpeed, on its takeoff run, with its
    rally's order, or with none circling 2 grid units past the `toward` end (`Mover::orbit`,
    `tuning::air::Air::runway_circle_past_gu`). `exit_rule` gives such a building its holder's exit, so its ejected and
    released units, and the door arrows, use the side doorway as any pool building's.
  - **No doorway: into the Tunnel Network** (slice M4-C, `ExitRule::Network`, the Hideout's, (M6-SV) the
    Vehicle Assembly's and (M6-SD) the Aerospace Assembly's, whose aircraft come out by the air exit door;
    `factions/syndicate/structures/README.md > Rally Point Behavior`, design changes 0252 and 0253;
    `Sim::release_into_network`): the finished unit comes into the game held in its owner's network
    (`SimEvent::UnitProduced` without a `Spawned`), with no transit delay, and nothing ever holds the release up.
    With no rally point, or one on a Tunnel of its owner's, it stays inside. Any other rally is on the surface: the
    unit joins the end of its **parent Tunnel's** eject queue (`tunnel::area::parent`, derived) with its default command
    on the rally target, resolved at production (`Sim::rally_order_of`); a rally on an own underground structure is a
    Move to the point above it, not an object target (0252). Without a parent Tunnel, or one whose tier lets the unit
    out, it stays inside. An entry's order whose target leaves the game meanwhile is dropped (`forget`), and the unit
    then walks straight out and idles.
  - **Onto a slot** (slice M6-GH1's `ExitRule::Pad`, the Airfield's; since slice M6-GH2 `Sim::release_onto_slot`,
    `sim/landing.rs`; design change 0428 point 2: "New aircraft appear **landed on a free slot**; they take off for the
    Airfield's rally point, and with no rally point they stay landed"): the finished aircraft comes into the game down
    and full on the first slot no aircraft takes (*Production > Landing and rearming*), assigned to the Airfield,
    facing its rally point (east without one), with its rally's order, which lifts it off in its first tick. It uses
    no doorway ("Aircraft [...] are not covered by this rule"), nor parks (`Sim::park` is the doorways' only); while no
    slot is free (one rebased elsewhere still on it, or rising from it) it waits inside, finished. It replaced GH1's
    release over the Airfield's centre (D34's placeholder).
  - **Occupied** while a unit the building released is in its doorway (`Mover::exit` is the building's id). While
    it is, the next finished unit is held (progress at its total, `waiting`), the queue behind it doesn't advance,
    and it is released in the first tick the doorway is free (0080; for the portals 0145 point 5, "it waits fully
    inside until the previous unit is out"). So at most one unit is in a doorway, or in each door of a building with
    several exit sides, which waits only while every door is occupied.
- **The heading** (`EnergyDoorway::heading`, 0145 point 3; `release`, `Sim::release_unit`): **towards the rally
  point** (`Rally::point`, where the owner last saw an object) if that direction is open and its way out is free;
  else **the nearest open direction whose way out is free**: of the arc's ends and the open ones of 32 fixed
  headings (`exit::HEADINGS`, every 11.25° from east, exact under the square's eight symmetries), the one at the
  smallest angle from the rally point's direction, ties clockwise first (**the Director's reading**); with every
  way blocked, the nearest open one all the same (the unit stops half out, point 4). With **no rally point**: a
  random one of the open headings whose way out is free, uniformly (all the open ones if none is free). **A way
  out is free** (`EnergyDoorway::way_out_free`) when the unit's box, swept from the centre to one unit-length
  beyond where it comes clear (`outside`), enters no blocked tile of the grid of the unit's own class and size (the
  true grid; the footprint's own tiles don't count, the map's edge does) and no unit of its layer
  (`nav::segment_enters_box`). The release waits while the unit's box at the centre isn't clear, of tiles or of
  units (nearly never: only a unit in the doorway, which occupies it, can be there).
- **The sim's Rng in production** (its first use): the random heading is `Rng::below(n)` among the `n` candidates,
  drawn in `Sim::release_unit` (economy phase, producers in id order) **only when `n` is 2 or more**, and only by a
  portal without a rally point, in the tick it releases. A Barracks never draws, nor does a portal with a rally
  point, so the GDO goldens don't move. The Rng is hashed and saved, so replays and saves draw the same headings.
- **Rally points** (`production/rally.rs`): `Rally::Point` or `Rally::Object { id, at }`. A released unit gets its
  **own default right-click command** on the rally target (design change 0136, `systems/controls/interface.md
  > What a rally point makes a new unit do`: "No per-building lists override this"), at its release, from where it
  appears (out of a portal, the centre: its path starts there), through `Sim::give_order` like a player's:
  `Sim::rally_order` is `Sim::default_order` (`orders/resolution.rs`, the one rule of *Commands > Default orders*) on
  the target as the owner knows it, so a Drone rallied on a patch gathers it, on an enemy unit follows it and on an
  enemy structure moves to it (it is unarmed), a Peacekeeper attacks an enemy, and a rally on the ground is a Move
  there. Without a rally point it walks out of the doorway and stops (design change 0086; out of a portal, along its
  random heading, where it comes clear). The producer itself as the target clears the rally. A rally on an object
  that is removed and that the owner doesn't remember resets to None (`Production::forget`). At the end of each tick
  `rally::follow` (`Sim::follow_targets`) notes where the owner sees the object: an **enemy unit** leaving its sight
  turns the rally into a ground rally at its last-seen position (design change 0088), so an enemy dying in the fog
  changes nothing; a **structure** or a resource stays the rally out of sight, at its last-known position, until the
  owner sees its place without it (design change 0139: "A rally point on an enemy structure stays on it after it
  leaves sight"), and a rally may be set on one the owner only remembers.
- **On an unfinished building** (slice M5-S6, design change 0396: "A building that will produce or release units can
  have its RallyPoint set while it is still under construction or transformation, for every faction"): a type takes
  one if it has one finished (`production::takes_rally`: a `ProductionDef` and no `ResearchDef`, so the Barracks, the
  portals, the Hideout, the Recruitment Hall, the Armory, the Altar, the Cathedral, the Airstrip (slice M5-N7) and the
  Chop Shop (slice M5-N6); not the Temple, the Garrison,
  whose panel has none, nor the Shrine), a site, an expansion's included, or under Warp In. Its functions held back
  (`Object::production` is `None`), it holds a `Production` with the rally alone while one is set, and `None` again
  once it is cleared, so a game that sets none keeps its hashes (the goldens'), but for a vehicle building's site,
  which holds its default one from its placement (below). The gate's `SetRally` takes the player's buildings of such
  a type, finished or not; its other production commands only producers at work
  (`Object::is_operational`), so a Train to the site is `NoProducers`. `Sim::start_functions` keeps the rally as the
  functions start ("The rally point is kept when the building completes"); a cancelled Warp In takes it away with the
  portal. Tests: `tests/cults_construction/rally.rs` (0396's Verification in the sim, a Storage's and a Garrison's
  sites), `tests/warp_in/rally.rs` (a portal kept through completion and lost with a cancel, an Egress Node) and
  `tests/expansions/building.rs` (a Hideout's site).
- **A vehicle building's default rally point** (slice M5-S8; `core/entities.md > Entrance and exit > Exit Doorway`
  point 7, design changes 0446 and 0447, answering Q0175): idle wheeled and tracked vehicles don't step aside, so a
  building that releases them through an exit side "**starts with a RallyPoint 3 grid units straight out from the
  middle of its exit side** (placeholder distance)". A def field, `ProductionDef::default_rally` (grid units, `None`
  for every other producer; `defs_are_consistent`: only with `ExitRule::Side`, for a type that makes a wheeled or
  tracked vehicle), which the Chop Shop sets to 3, so that M6's Factory only sets it too. `production::default_rally`
  places it (`Door::spot` of the type's one exit side, as the placement turns and flips it), and `Sim::new_object`
  sets it as the building's object comes into the game, whichever way (a scenario's spawn, a site, a GDO placement),
  so a site has it from its placement ("it is set at placement, so it exists during construction"):
  `transform::hold_back` keeps a rally the object has, in a `Production` that holds it alone, and
  `Sim::start_functions` keeps it at completion. It is an ordinary rally point, hashed and saved as any (a Chop Shop's
  game moves from its first tick: `cults_vehicles` re-blessed), which the rally commands move or clear (on the
  building itself). **Placeholder (not in the design: a spot off the map, or on ground a vehicle can't stand on)**:
  the point of the map nearest to it (`Map::clamp_point`), one the gate would take, and kept on any ground, as one
  set by hand may be. A saved game's building placed before it existed has none, and loads with none: loading
  builds no object anew. Tests: `tests/chop_shop/rally.rs` and `exit.rs`; in the client,
  `crates/game/tests/commands/chop_shop.rs` (0446's and 0447's Verifications: the marker 3 grid units out of side C,
  a right-click on the Chop Shop clearing it, one turned a quarter) and (M6-GF) `crates/game/tests/commands/
  factory.rs` (the Factory's, 3 grid units out of side B).
- **Parking round a taken rally point** (slice M6-GF, `sim/producers/park.rs`; `core/entities.md > Entrance and exit
  > Exit Doorway` point 7, design change 0451 answering Q0177): "a newly released vehicle whose rally point is
  already taken by parked vehicles **parks at the nearest free spot around the point**". It is the end of one move
  only: the rally Move to a point of a wheeled or tracked vehicle just made and let out of an exit side's doorway, the
  Factory's (`Sim::release_unit`) and the Chop Shop's trained ones (`Sim::let_out`, `Sim::park_out_of`); not
  infantry, an ejected or released unit, a portal's, a Tunnel Network's or an aircraft. `Sim::park` gives that Move
  the spot as the unit's own destination (`orders::Order::slot`, the magic box's field), so it travels and arrives as
  any Move, and a player's Move keeps its own arrival. **Taken**: within the vehicle's reach of the point (the half
  diagonal of its Silhouette) stands a unit that doesn't yield (a wheeled or tracked vehicle, design change 0440:
  `UnitBaseDef::yields`) or lies the destination of another of its owner's vehicles' Moves (its claim, as a
  builder's on its spot: **placeholder**). **Free**: nothing takes it, the vehicle fits there in four facings
  (`Sim::check_unit_fits_facing`), it is off the lane out of the doorway (**placeholder**), and the straight way
  there from the door, the vehicle's width and `Production::parking_lane_margin_su` more each side, meets no taker
  and no claim (**placeholder**: a spot behind the cluster would leave it stopped short in front of it, by
  MovingToLocation's "Occupied target location", in the line toward the door the rule does away with). **The search**:
  rings `Production::parking_step_su` (8) apart out to `Production::parking_radius_gu` (3), each from the point
  straight beyond the point round both ways, the first free point the spot; none, and it drives to the point as
  before (**placeholder**). Nothing new is stored: the slot is the order's, hashed and saved with it. Tests:
  `tests/factory/parking.rs` (five Raptors and five Twisters round the Factory's default point; with it cleared, the
  stall stays the player's, 0447), `tests/chop_shop/parking.rs` (0451's Verification 1 to 3) and the goldens
  `gdo_vehicles` and `cults_vehicles` (re-blessed: its second Seeker parks at the free spot nearest the first).
- **Tests**: `src/production.rs` and `production/exit/tests.rs` (the queue, the Barracks' door in all 8
  orientations and its rule, the room before a door, the Hall's four doors and their order), `recruitment/tests.rs`
  (slice M5-C: the area, the exact capacity, the claims and their order, the lineages' points),
  `recruitment/preview/tests.rs` (slice M5-J2: the preview's claimants in their order, completed before pending,
  0351's placement order, 0341's tie rule as the game's claims split it, the Effectiveness rounded down, off-map tiles,
  known Water and unexplored ground, a Hall flush against the edge at 70% (0387); in the client,
  `placement/recruitment_areas/tests.rs`, the pending Halls' order, the mesh and the label's place, and
  `crates/game/tests/commands/recruitment_areas.rs`, with the fog on: 0350's, 0351's and 0352's Verifications, a Hall
  never seen, the label, a selected Hall's claims and an enemy's none, 0387's at the edge and in the corner),
  `tests/recruitment/` (slice M5-C: a full Hall's 6 → 20 in 2688 ticks then none, 50 tiles' 10 at 384 ticks,
  buildings in the area, Halls sharing tiles and one lost in a tick, a destroyed Hall's Recruits and capacity, a lost
  Recruit restarting a full Hall, orphans, X, a `Train`, the rally on the ground and on a patch, a game replayed and
  saved, and the sides a Recruit leaves by, up to every door occupied), `production/exit/energy/tests.rs` (slice M3-D2:
  the 32 headings' symmetry, the Robo Portal open all round and the Bio Portal's arc in all 8 orientations, a wide
  doorway's arc from corner to corner, the walk out, the choice of heading and its ties and draws, a way out
  blocked by a cliff, by units and by the map's edge), and `tests/production/`: `queue.rs` (times, the power ratio, train and cancel, the limit
  of 20, Unit Control, the gate, group production, refunds, a game replayed and saved), `capacity.rs` (slice M3-P:
  the army caps at production start, design change 0143's and 0112's Verifications with a real deployment, the
  waits and their notification, id order, the refunds of a waiting and of a producing producer, a cancelled wait, a
  producer without power, waits saved and replayed, and slice M3-D's portal over capacity), `rally.rs` (each kind of
  target, clearing, an enemy leaving sight, a remembered structure), `exit.rs` (the verification of design change
  0080, all 8 orientations, a crowd of idle units, a half-blocked exit, a path from inside the footprint, no way
  back in), `stalls.rs` (walled in by a building and by units and freed again, the waiting unit's refund, saves
  in the middle of an exit, and a random game of 3 000 ticks around four doorways), `portals.rs` (slices M3-D and
  M3-D2, design change 0145's Verifications: a Drone's and a Sabre's times and their walk straight out of the
  centre, the heading for a rally point in every direction and on an object, the Bio Portal rallied behind it and
  its arc in all 8 orientations against 9 rallies, random headings by seed, a way walled off, rallies on an enemy,
  a destroyed portal's refund, a replay and a save in the middle of a walk), `portal_stalls.rs` (slice M3-D2: a
  portal ringed by units, its unit half out and the next held at 100% until the first is out, that unit's refund,
  and negative tests of I1 and I2 around the energy doorway) and `half_out.rs` (slice M3-K2a, 0145 point 4: a
  Sabre stuck half out is a normal unit, which auto-targets an enemy in reach, carries out an Attack and is
  attacked; a click picks it: the client's `Selectables::at`).
