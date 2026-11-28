# Object model — Types, capabilities and factions

Read this when you add or change an object, a capability, an `ObjectDef` or a faction's unit and structure types.

## Object model

Units, structures and resources are all `Object`s (`object.rs`, `core/entities.md > Object Instance`). The layout is
"ECS-lite": a plain struct with core fields plus one `Option` per capability.

- **Core fields**: `id`, `kind`, `owner` (`None` means neutral), `pos`, `prev_pos`, `facing`, `prev_facing`, `hp`
  (`None` means indestructible), `shields` (the current Shields of an object whose kind has MaxShields, the
  Colonists' kinds; `None` for the rest; see *Combat > Shields*) and (slice M5-N4a) `energy` (the energy of an object
  whose kind has an energy pool, the Disciple's: a whole number of `energy::ONE`s a point; `None` for the rest; see
  *Combat > Energy*).
- **Capabilities**: today these are `placement` (the tile footprint), `resource` (`ResourceState`), and for units
  `orders` (`Orders`, `orders.rs`: the current order with its behaviour's state, a queue of at most
  `MAX_QUEUED_ORDERS`, and what the unit does on its own while it has none; see *Behaviours*), `mover`
  (`Mover`, `mover.rs`: goal, status, path, the replanning state of `docs/movement.md > 3`, the sidestep of a
  yielding unit and a moving unit's contact step; `mover/` holds the stuck schedule, the two steps and the hash), for units that
  can attack, `attack` (`AttackState`, `combat.rs`: see *Combat*) and, for units that carry a Riot Shield,
  `riot_shield` (`RiotShield`, `riot_shield.rs`: how far up the shield is, the way it goes being the unit's type,
  see *Combat*); for a plate on a patch,
  `extractor` (`Extractor`, `economy/mining.rs`: the patch it mines and its cycle, see *Economy*); for a structure
  that produces units, `production` (`Production`, `production.rs`: its queue, the progress of the unit in
  production and its rally point, see *Production*); for a structure that constructs buildings, `constructor`
  (`Constructor`, `construction.rs`: idle, constructing or holding a building ready to place, see *Economy*); for
  a worker, the Drone or the Agent, `worker` (`Worker`, `economy/gathering.rs`: its load and its AssignedDropOff,
  see *Economy > Gathering*); for an object transforming into its kind (a Beacon warping in a building, a
  Beacon deploying from a Drone), `transform` (`Transform`, `transform.rs`: what it was and where it stood, how far it
  has come, what it paid and the load a deploying Drone carried, see *Economy > Warp In* and *> Beacon Deployment*);
  for a Legionnaire whose form isn't its type, `conversion` (`Conversion`, `conversion.rs`: the form it has, and the
  frames left of the conversion under way, none while it waits for its Cooldown; see *Type and form* below and
  *Combat > Weapon Conversion*); for a Tunnel, `tunnel` (`TunnelState`, `tunnel.rs`: its tier, plan decision T39;
  see *The Syndicate* below); (slice M6-GD) for a unit whose type has a stock, the Drone Operator, `stock` (`Stock`,
  `stock.rs`: the items it holds and the purchase under way) and `drone` (the drone it has out), and for that drone
  `tether` (`Tether`, `tether.rs`: its operator and the tick its battery runs out; see *The Drone Operator* below);
  (slice M6-GR) for a caster waiting out an ability's cooldown, the Radar Station, `cooldown` (a `Progress`,
  `cooldown.rs`; see *Combat > Scan and cooldowns*); (slice M6-GH1) for an aircraft that rearms at an Airfield, the
  Hornet, `aircraft` (`AircraftState`, `aircraft.rs`: its ammo, the Airfield it is assigned to and (slice M6-GH2) its
  `landing` on an Airfield's slot, `landing::Landing`, hashed under tag 14, the landing under its sub-tag 1, and saved
  only while set; see *Production > The Airfield*); (slice M6-CM) for an illusion, `illusion_of` (the
  Disciple whose Mirage made it, `mirage.rs`, hashed under tag 15 and saved only while set; see *Combat > Mirage*);
  (slice M6-SB) for a unit whose type has tanks, the Shadebringer, `tanks` (`Tanks`, `patches.rs`: what each tank has
  used, its toggle and its trail's newest patch, hashed under tag 24 and saved only while set; see *The Vault and the
  Sludge* below); (slice M6-GS1) for a Supply Chopper, `chopper` (`ChopperState`, `supply_run.rs`: its cargo, one kind
  at a time, the Supply Tower it is attached to and its `landing` on a Tower's or a station's spot, hashed under tag
  27), and for a Supply Tower `schedule` (the station whose deliveries it runs, tag 28), each saved only while set; see
  *Production > The Supply Tower* below.
  The rest arrives as more optional fields, each with its own module. Never add them as separate
  stores keyed by id. A system walks the objects in id order and skips those without its capability
  (`let Some(..) = &mut o.resource else { continue };`).
- **Per-type constants** sit in one `static ObjectDef` per type (`core/entities.md > Object Type`), in the type's own
  def module, named as its design file: `defs/<faction>/{units,structures}/<object>.rs`, and `defs/map/<object>.rs`
  for the map's objects (*Adding a unit or structure*). Reach them through `ObjectKind::def()` or `Object::def()`.
  Every value cites its design section. A faction's own rules, which belong to no one type (its caps and divisors),
  are in its root module, `defs/<faction>.rs`. What a type does differently is a field of its def where it can be:
  slice M4-R1 turned three `match`es on `ObjectKind` into `StructureDef::onto` (the kind a structure is placed onto),
  `StructureDef::seeds_build_area` and `UnitBaseDef::transit` (which `Transit::of` reads). **The defs are plain serde
  data** (plan decision T54, for the data files of playtest; `defs/tests/serde.rs` takes every def through JSON and
  back): their names and lists stay `&'static str` and `&'static [T]`, which the deserializer leaks once
  (`defs/leak.rs`), so the defs stay `Copy` (the Director accepted this for the playtest loader). A unit type also
  has `ObjectDef::unit: Some(UnitDef)` (`defs/units.rs`, `systems/units.md`): its `UnitBase`, whose `static UnitBaseDef`
  holds the base's domain, movement model and flags (turret, crushable, (M5-V4) crushes, …), the attributes of that
  movement model (`MovementAttrs`, `defs/units/movement_attrs.rs` since slice M5-V3), and its turret, attack and
  costs (its attack's types in `defs/units/attack.rs`, a worker's and a repairer's in `defs/units/worker.rs`, split
  off in slice M6-GK). Its box is its silhouette (`ObjectDef::half_extents`). Armour (`ObjectDef::armor`, an `Armor`
  of PointArmor and FullArmor) is the same for units and structures; map resources have `Armor::NONE`. **Struct-update
  defaults** (plan decision T84, slice M6-GK; `defs/defaults.rs`): every def literal ends with `..ObjectDef::NONE`,
  `..UnitDef::NONE` or `..StructureDef::NONE` and lists only what its type has (a literal made from another def, the
  Peacekeeper (Shielded)'s, ends with that def). A `NONE` leaves the optional fields unset (`None`, an empty list, or
  `false` for a flag few types have); its other fields are placeholders every literal sets. So a new optional field is
  one line in its struct and in its `NONE`, plus the literals of the kinds that have it.
- **Structures** (`defs/structures.rs`, `defs/gdo/`; `factions/gdo/structures/`, `factions/gdo/economy.md`): the
  GDO's Deployment Center, Generator, Barracks, Extraction Facility and Extraction Plate (slice M2-A; their
  `ObjectKind`s follow the M1 ones, because the state hash numbers kinds in declaration order). A structure type has
  `ObjectDef::structure: Some(StructureDef)` (its `SymmetryType`, BuildRadiusExtension and Power, the plate's
  `MiningDef` in `StructureDef::mining`, and a constructor's `Constructs` list of `Recipe`s in
  `StructureDef::constructs`, see *Economy*) and a tile footprint, `ObjectDef::tile_size` as unturned;
  its `Placement` holds it as placed, `w` and `h` swapped by a quarter turn, with the orientation it was placed in
  (`Placement::orientation`). It has HP, armour and a SightRange, and no orders, mover or attack, so behaviours,
  movement and combat pass it by. The Career Campus and the Field Engineer (slice M5-N9, design change 0328's
  placeholders) come last, after the Cults' kinds: the Career Campus one of the Deployment Center's `Constructs`
  (requires a Barracks), which only counts so far (its research is to come), the Field Engineer one of the
  Barracks' recipes (`Recipe::requires` a Career Campus), unarmed infantry that repairs (`UnitDef::repairs`, *Repair*)
  and gathers nothing. The Arsenal and the Rocketman (slice M5-K, design change 0356's placeholders) come after them:
  the Arsenal one of the Deployment Center's `Constructs` (requires a Barracks; Power −20, BuildRadiusExtension 2),
  which only counts so far (its research is to come), the Rocketman one of the Barracks' recipes (`Recipe::requires` an
  Arsenal), LightInfantry with the Peacekeeper's movement at MaxSpeed 3, a DoublyDisjointed Universal missile (Range
  6, MinRange 2) and Target Lock (`UnitDef::target_lock`, *Combat > HeadDisjointed and Target Lock*).
  **Every other GDO kind** (slice M6-GK, design change 0366; plan §B > GK) follows the Rocketman, before slice
  M6-SK's Syndicate kinds: the twelve buildings (the Factory to the Supply Tower), the ten units (the Raptor to the
  Supply Chopper, the deployed Artillery Rig, `ArtilleryRigDeployed`, and the Drone Operator's drone,
  `OperatorDrone`, named "Drone", among them) and the `Mine`, of a category of its own, `Category::Mine` (a
  silhouette, no `UnitDef` and no `StructureDef`, no orders; never on tiles: `paths` and `placement` pass it by; the
  client draws it with the ground objects, and the editor's palette has a group of mines),
  one def module each with its design file's numbers. Nothing makes them yet (plan decision T69): **their recipes are
  held** in `defs::PLANNED_RECIPES` (`defs/gdo.rs`: the producer and the `Recipe`, each with its cost, frames,
  `requires` and, design changes 0435 and 0441, `Recipe::supplies`, the GDO's second cost), out of every
  `constructs`, `production` and `builds`, until the slice that makes each one moves its recipe into its producer's
  def; the editor's debug spawn places any of them. `defs/gdo/tests.rs` checks the kinds against the design folder
  and that nothing makes them; `tests/gdo_kinds.rs` that no player's command constructs, trains, builds or spawns
  one. The deployed Rig doesn't move (`defs/tests.rs`'s `IMMOBILE`: speed 0, no reverse): a Move given it ends
  without moving it.
- **The Drone Operator** (slice M6-GD; `factions/gdo/units/drone_operator.md`, design changes 0421 and 0441; plan
  decisions T89 and T91): the Barracks' R (its recipe out of `PLANNED_RECIPES`, into the Barracks' production),
  requiring a Career Campus, for 100 Space Crystals **and 1 Supply**, the first recipe with a Supply cost
  (`Recipe::supplies`, `QueueEntry::supplies`): both are paid as it is queued, or neither (`Sim::plan_training`
  checks the crystals, then the Supplies: `Reject::InsufficientSupplies`, `Refusal::Supplies` for a passed-over
  producer), and both are refunded on a cancel or the producer's loss. Slices M6-GF, GH1, GM and GS reuse it as
  data, a `Recipe::supplies` and nothing more. **Its stock** (`UnitDef::stock`, a `StockDef` of item, max, start,
  price and frames; `stock.rs`) holds its drones, at most 1, 1 at the start; `Ability::Buy` (its F) buys one for 50
  Space Crystals paid at the press, in 160 frames counted in the economy phase whatever the Power
  (`SimEvent::StockBought`), only once its drone has ended (`Reject::DroneOut`, `StockFull`, `PurchaseUnderWay`).
  **Its launch** (`OrderKind::Launch`, its R; `orders/launch.rs`) spends one, right after 5a: the drone, an
  `OperatorDrone` (a HoverCraft, air), comes out over the operator with a `Tether` (`UnitDef::tether`, a `TetherDef`:
  leash 8 grid units, battery 320 frames; `SimEvent::DroneLaunched`; `Reject::StockEmpty` with none in stock,
  `DroneOut` with one out). While its drone is out the operator holds still (`tether::holds`): `combat::may_move`
  keeps it where it is, and the gate refuses it every order that would move it (`tether::allows`; `Reject::DroneOut`
  when no unit of the command takes it). **The tether's step**, right after 5d (`tether::ends`, `sim/drones.rs`),
  ends a drone, the first that applies (`tether::Fall`): its operator gone (the removal path takes it along: no
  explosion), its Silhouette past its leash from the operator's, edge to edge (`tether::past_leash`, `Shape::gap`:
  its explosion dropped where it fell, `combat::Projectile::dropped`, landing in the tick's 5e), its battery spent
  (no explosion); each with a `SimEvent::DroneFell` and a `Died`. **Its attack is its explosion**: a melee
  FullyConnected area attack of 150 on the ground units within 48 su, whoever owns them, which a tethered attacker
  spends (`phases::Volley::spent`: it leaves with 5e's dead). `Ruleset::check` bounds a stock's frames as a recipe's
  and its start by its max. `tests/drone_operator/` tests it; the golden game is `gdo_drones`.
- **The Colonists** (`defs/colonists/`; `factions/colonists/units/`, `factions/colonists/economy.md`, slice M3-A):
  the Drone, the Beacon, the Egress Node, the Robo Portal, the Bio Portal and the Sabre and Rifle Legionnaires, after
  the M2 kinds. Three def fields came before their slices: `ObjectDef::max_shields` (MaxShields, set for exactly
  the Colonists' kinds; their Shields came with M3-B, see *Combat > Shields*), `UnitDef::beacon_capacity_cost`
  (M3-C) and `StructureDef::warp_in`, the Beacon's Constructs List (M3-E). `warp_in` is not `constructs`, so the
  Beacon is no GDO-style constructor (plan decision T14). Their structures have no Power and no build area. The
  portals produce (slice M3-D) through an energy doorway, out through their A sides, all four of the Robo Portal's
  (slice M3-D2, design change 0145, see *Production*). The Drone, a
  HoverVehicle, moves by DragMovement (slice M3-G, see *Movement and collision > Movement models*): its equilibrium
  is `(ForwardAcceleration + OmniDirectionalAcceleration) / DragRatio` = 8 su per tick (`MovementAttrs::max_speed`).
  It crosses Rugged tiles with double drag (`HOVER_VEHICLE.rugged_terrain`, design change 0039), on the light
  infantry's `(GroundRugged, 1)` grid.
- **The Syndicate** (`defs/syndicate/`; `factions/syndicate/`, slice M4-A): the Guard (M3), then the Agent, the
  Tunnel and the Hideout, after the Colonists' kinds, the Optics Lab and the Chemical Plant (slice M4-K, *Tech
  prerequisites* below), and the Grenadier (slice M4-U), which throws grenades (*Projectiles*), trained at the Hideout
  behind a Chemical Plant. The Agent has its pistol; units enter and leave the Tunnel
  Network at a Tunnel's Side A and the Hideout produces into it (slice M4-C, *Units off the map* below); the rest is
  inert so far but for blocking, sight and being targets. **The Tunnel is one kind with a tier** (plan decision
  T39): `StructureDef::tunnel` holds
  its `TunnelDef`, the rows of `factions/syndicate/tunnels.md > Tiers` (`TierDef`: MaxHP, Tunnel Area Radius, Transit),
  and the object its `TunnelState` (`tunnel.rs`), from Tier 1. `Object::max_hp` reads its tier's row; `tunnel::set_tier`
  changes the tier, the damage kept (`transform::follow_max`, design change 0231 point 4), through `Sim::set_tier`: an
  upgrade's completion (slice M4-G, *Tunnel upgrades* below) and `DebugCommand::SetTier`, which ends an upgrade under
  way, unrefunded. Records carry the tier last seen (`LastKnown::tunnel`, 0237). **The Hideout is an underground
  expansion**, as are the Optics Lab and the Chemical Plant (`StructureDef::underground`, plan decision T32): it needs
  no flat ground (`ObjectDef::needs_flat_ground`, design change 0005), stands for the ElevationModifier at the level
  under its centre, blocks no movement (*Navigation*) and is hidden from the other players (*Vision > Hidden
  objects*). Surface and underground footprints never share a tile (design change 0240): `may_overlap` keeps them
  apart as any two footprints, and so do the placement methods of slices M4-E and M4-F (*Placement*).
- **The Cults** (`defs/cults/`; `factions/cults/`, slice M5-A, plan decision T55): the Recruit, the Militant and the
  Gunner, the Recruitment Hall, the Storage and the Armory, after the Syndicate's kinds, with their numbers and their
  client kinds, and none of the faction's systems: **each system's slice adds its own field** to their defs (the
  Recruit's `worker`, the Storage's `drop_off`, every structure's `site`, the Hall's and the Armory's `production`,
  the Armory's holding; plan §3.2, "one field per slice"), and `attack` and `popup` stay `None` on every Cults
  structure. The Armory's `requires: [Storage]` is data until Recruits build (slice M5-E). **A Cults unit's Unit
  Control cost** is the Recruits it takes (`factions/cults/structures/recruitment_hall.md > Unit Control Tracking`),
  kept in the GDO's `UnitDef::unit_control_cost`, 1 for each infantry unit (`defs/cults/tests.rs` checks every Cults
  unit's), and counted in the Cults' own Unit Control alone (`ArmyResource::CultsUnitControl`, slice M5-C; *Economy*).
  **The lineage**, `Object::lineage: Vec<ObjectId>` (plan decision T56 as the Director amended it for design change
  0319): one entry per Recruit the unit embodies, the Hall that made it, so a Recruit has one and a Seeker will have
  two, perhaps the same Hall twice; hashed (a tag) and saved only while it has an entry, kept whole by
  `Sim::change_kind`, to which a pool building's training adds the lineages of the other Recruits it consumes (slice
  M5-F2, *Production > Pool buildings*), and set by the Hall that releases the Recruit (slice M5-C, *Production > The
  Recruitment Halls*; by the layout at a game's start, M5-B). An entry whose Hall is gone stays: it is an orphaned
  point, and so is each point of a Cults unit's cost that no entry stands for (`recruitment::Lineages::of`: the cost
  less the entries), so a unit without an entry is orphaned whole (design change 0344: "Recruits no Hall produced
  (custom maps, debug) are orphans from the start"); `testing::references` doesn't check an entry as a reference for
  that reason. **Every other Cults kind** (slice M5-G2, plan decision T69: the last slice of M5 that adds a kind)
  follows, in the tech tree's order, then design changes 0361 to 0365's two vehicles: the Gunworks, the Garrison, the
  Chop Shop, the Machine Shop, the Airstrip, the Scrap Hangar, the Altar (`factions/cults/augmentation.md > Altar`),
  the Temple, the Cathedral and the Shrine; the Disciple, the Seeker (design change 0364, once the Jackal), the
  Sparrow, the Hand of Heaven and the Prayer-mobile; one def module each, with its design file's "Placeholder stats",
  its `site` and its `requires`. `defs/cults/tests.rs` lists them and checks the list against the design folder (every
  structure of `factions/cults/structures/README.md > Tech Tree Overview`, every file of `factions/cults/units/`). Each
  of the others' slices adds its behaviour: the Chop Shop, the Airstrip and the Cathedral held 10 Recruits from slice
  M5-G2 (`HoldsDef`, design change 0347 point 5: an editor-placed one could be entered, with no panel yet), and the
  Recruits built none of the six left until their slices (the Construct menus held their slots, *Client architecture
  > The Recruit's panel*); slice M5-N2 made the Garrison live (*Object model > Buildings that hold units > The
  Garrison*), and slice M5-N6 the Chop Shop, the last (*Production > The Chop Shop*). The units' `MovementAttrs` (the Seeker's and the Prayer-mobile's
  `FixedTurnRadius`, the Hand of Heaven's `SpeedTurnRadius`, the Sparrow's `Glider`) are their models' since slices
  M5-H1 (the Glider), M5-V2 (the FixedTurnRadius) and M5-V3 (the SpeedTurnRadius: *Movement and collision > Movement
  models*), and nothing produces those units before their buildings' slices. Their `TurretAttrs` are their
  turrets' since slice M5-U1 (*Combat > Units' turrets*). The wheeled and tracked
  vehicles don't cross Rugged terrain, so every game's set of grids holds a `(Ground, 1)` grid beside
  `(GroundRugged, 1)` (*Navigation*); the Sparrow is the first air unit kind (`nav_class` `None`: it crosses every
  tile). **The Augmentations** (slice M5-F2; `augmentation.rs`, `factions/cults/augmentation.md >
  AugmentationState (per unit instance)`): `Object::augmentation: Augmentation { constitution, metabolism }`, hashed (a
  tag) and saved only while one is set, so the states without one, the goldens among them, are as they were; its
  `RecruitKind` (Plain, Constitution, Metabolism or Both, design change 0337) is what a pool building's orders choose
  by. The Altar gives them (slice M5-N1b, *Production > The Altar*), and `DebugCommand::SetAugmentation` for
  development and QA; their effects are read through accessors since slice M5-N1a (*Augmented stats* below).
  The Gunner is the first
  kind with a MinRange, which M3-L2's code serves (*Behaviours*; M5-A gave a unit that can't back off its "Target too
  close", design change 0204 point 3; M5-O3 took the back-off on to the middle of the band, question Q0054, and gave
  an Attack Ground's location the same rule; design change 0454 brought it back to MinRange + 1 grid unit, slice
  M6-S6a), and the first TailDisjointed attack (slice M5-O3, *Combat > TailDisjointed
  attacks*), which can miss; the Armory trains it for a player who owns a Gunworks (slice M5-G2, `Recipe::requires`,
  *Tech prerequisites*). The Storage was 1/4 on its 2x2 from slice M5-G2 (Q0094 a), and is 1/6 on its 3x2 since
  design change 0380 (slice M5-S2a), so the building armour test has no exception left. The Cults start a game since
  slice M5-B (plan decision T63; `tests/cults_start.rs`, *Sim scenarios and tests*): no faction is "not implemented"
  any more.
