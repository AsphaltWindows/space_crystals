# X4: the mechanics audit (2026-11-28)

A read-only audit of `crates/sim` and `crates/game` at master `19e9bdfb`:
- the design's ten mechanics docs (`systems/mechanics/`) against the code;
- the hub files every slice edits;
- three facts for the Designer.

The briefs for the refactor phases and the remaining M6 slices cite its sections. Sizes: S is under about 300 lines, M is 300–1,000, L is over 1,000.

## 1. Summary
1. **Tests of a unit's kind are nearly gone.** There are 87 kinds. Outside `defs/`, the tests and scenarios, production code names a specific `ObjectKind` at 5 sites only, plus 8 lines keyed by faction, all legitimate (§6). All 7 `Ability` variants and all 25 `OrderKind`s are gated by def parts.
2. **Per-unit shape now lives in the def parts and in the modules behind them.**
   - 16 of the 25 optional `UnitDef` parts are set by one def file only.
   - 6 of the 28 `Object` capability fields are named for one unit: `riot_shield`, `conversion`, `drone`, `chopper`, `schedule` and `illusion_of`.
   - So are 6 of the 25 `StructureDef` parts: `airfield`, `supply_tower`, `chamber`, `vault`, `popup` and `tunnel`.
3. **Already one data-driven module each:**
   - holding, with evacuation;
   - garrison fire;
   - the vehicle-deploy rows;
   - gathering, the Recruits included;
   - production;
   - construction sites;
   - energy pools;
   - area casts and buffs;
   - repair.
4. **About 3.8k of about 74.6k production lines (5%) are built around their first user, and M6 hits them:**
   - landing and aircraft, 942 lines;
   - the supply runs, 849;
   - the toggles, 601;
   - the patches, 558;
   - the stock and the drone, 469;
   - the carrier, 400.
5. **A new mechanic costs mostly hub edits, not tests of a unit's kind:**
   - a new order kind edits 17 exhaustive matches in 13 files;
   - a new capability edits about 9 places (§3).

## 2. The mechanics docs against the code

### Carrying
- **Archetypes:** the Ground carrier is built (ST1). M6 adds the Landing carrier (GS2), and the Docked carrier with Paradrop (CP).
- **Modules now:** `carry.rs`, `sim/holding/carriers.rs` (`step_unloads`), `orders/unload.rs`, `orders/enter.rs` with `sim/network/entries.rs`, and `Place::Carrier`.
- **One module, but shaped for the Ground carrier:**
  - `loads()` means "is a ground object";
  - Unload All has no target (`ring_spots`);
  - nothing makes a carrier land.
- **The refactor:**
  - `CarrierDef.archetype: Ground { loading_state } | Landing | Docked { structure, side }`, and `paradrop: Option<{ gap_ticks, fall_ticks }>`;
  - `UnloadAll(Option<Vec2>)`;
  - an Enter on a Landing carrier lands it where it is, and it takes off by itself when done;
  - the Chopper carries Supplies or units, one kind at a time.
- **Size M; first slice GS2.**

### Unit Modes: Deploy
- **Archetypes:** the Artillery Rig is built. In M6, SL and ST2 are rows only; GP is targeted, permanent, and deploys into a structure.
- **Modules now:**
  - `vehicle_deploy.rs`, with its parts in `orders/` and `sim/`;
  - the hold in `Orders::give` (`orders.rs:373`, VehicleDeploy only).

  The Drone's deploy into a Beacon (`deploy.rs` with `transform.rs`) follows the design's own rule. One data-driven module, for pairs of unit types in place.
- **The refactor:**
  - `VehicleDeploy { into, frames, targeted }`, where `into` may be a structure;
  - reuse `deploy.rs`'s walk to the site and its placement check on arrival, and `change_kind` into a structure.
- **Size M; first slice GP.**

### Unit Modes: Toggle
- **Archetypes:** the Riot Shield and Weapon Conversion are built.
- **Modules now:** `toggle.rs` (the shared press rule), plus `riot_shield.rs` and `conversion.rs`, each with its own state, def, Ability and step. That makes two modules.
- **The refactor:** one `ModeToggle { forms, ticks, mid_switch: Reverse | FinishThenBack, between: { speed, damage, attack } }`.
- **Size M; no M6 slice needs it.**

### Aircraft landing and rearming
- **Archetypes:** built are Vertical (the Airfield's slots, the Tower's and the Station's spots, the air doors), the runway take-off (the Sparrow) and the Assigned base. M6 adds Field landing (GS2), the Network base (SA) and Runway landing (CP).
- **Modules now: several, one per user.**
  - `landing.rs` and `sim/landing.rs`: the Airfield. The `Landing` sits in `AircraftState`, and the landing time on `AirfieldDef`.
  - `supply_run.rs`, and `fly_to_spot` in `sim/supply_run.rs:285`: the `Landing` sits in `ChopperState`, and the landing time on `SupplyChopperDef`.
  - `movement/approach.rs`: the Gliders. Its only caller is `sim/landing.rs:210`.
  - `orders/enter.rs` and `Mover::rise`: the air doors.
  - `sim/holding/takeoff.rs` and `movement/glider.rs`: the runway take-off.
  - `aircraft.rs` and `orders/air_base.rs`: the Assigned base.
  - `network/services.rs`: the Vault.
  - There are two `take_off` functions.
- **The refactor:**
  - one `Object::landing { at: Spot(obj, i) | Ground(pos) | Runway(obj), descent }`;
  - the aircraft's own `LandingDef { archetype, land_ticks, takeoff_ticks }`;
  - one step for the claims, the approach, the descent and the take-off, with `approach` callable for any spot;
  - hold fire for any landed unit (`aircraft::may_fire`, `aircraft.rs:86`, now reads `AircraftState` only);
  - `AmmoDef.rearm: Assigned | Network { service }`.
- **Size L; first slice GS2, then SA and CP.**

### Garrisoning
- **Archetypes:** the Garrison structure is built. M6 adds the Bunker (GB), and the Bastion (CB) adds the Garrison vehicle.
- **Modules now: one, data-driven:**
  - `holding.rs` (`HoldsDef`), `sim/holding.rs`;
  - `sim/holding/evacuation.rs`: inside the footprint, for every holding building;
  - `combat/occupants.rs` (`OccupantFire`), and `ExitRule::AtOnce`.
- **For GB:** rows only (`holds`, `occupants_fire`), plus the exception for lowering the shield on entry.
- **For a moving holder, the Bastion** (size M):
  - **The def part:** `holds` and `occupants_fire` are `StructureDef` parts (`holding.rs:48`, `combat/occupants.rs:57`). Make them one `HoldDef` on `ObjectDef`, as `energy` is.
  - **The place:** occupants fire only from `Place::Building` (`combat/occupants.rs:71,125,206,236`), and `Place::Carrier` passengers are left out on purpose (`:72`). Either key the fire on "a holder with occupant fire", or merge the two into `Place::Holder(id)`.
  - **Entry:** a door at the middle of a footprint side (`sim/network/entries.rs:215`). A vehicle needs the carrier's boarding instead: any side, within Proximity, following it as it drives.
  - **Exit:** `ExitRule::AtOnce(side)` needs a footprint side. A vehicle needs `carry::ring_spots`.
  - **Destruction:** evacuation reads the `placement` tiles (`sim/holding/evacuation.rs:83-98`). A vehicle needs spots where it stood, and an `on_destroyed: EvacuateFootprint | EvacuateWhereStood | Die` parameter in place of the carrier's fixed `passengers_die`.
  - **The firing geometry is already generic:** the shooter is built from `Shape::of(holder)` and `holder.pos` (`combat/phases.rs:176-190`), so range from the Silhouette's edge on the move should work. Not checked: whether the threat code (`with_occupants` in `combat/targeting.rs:246`) assumes a building.

### HP restoration
- **Archetypes:** Repair, the docked service (the Chopper on its Tower) and Regeneration are built. M6 adds the Healing area (GB's Tent).
- **Modules now: four separate codes:**
  - `repair.rs`;
  - `regeneration.rs`, the % of MaxHP (`network.rs:142`, `sim/expansions.rs:279`);
  - `step_object` in `buffs.rs:321`, the flat HP/s of the Shrine's buff;
  - `repair_on` in `economy/supply_run.rs:163`, the docked repair.

  Plan T99's new `heal.rs` would be a fifth.
- **The refactor:**
  - one module;
  - `StructureDef::heals: HealDef { radius, rate, who, docked }`, serving both the Tent and the Tower's docked repair;
  - one regenerate function for flat and % rates.
- **Size M; first slice GB.**

### Spawned objects
- **Archetypes:** built are the stock (the drone's: a purchase only once the previous drone has ended), the Piloted object and the Copy. M6 adds the Deployed object (the mine), and a stock that queues up to the free stock (GM).
- **Modules now:** `stock.rs`; `tether.rs` with `sim/drones.rs` and `orders/launch.rs`; `mirage.rs`. The mine is only a def and `Category::Mine`.
  - One module per archetype, but the stock is shaped for the drone: `refusal` (`stock.rs:68`) tests `o.drone`, and refuses while any purchase runs.
- **The refactor:** `StockDef.purchase: QueueUpToFree | AfterPreviousEnded`, with a queued count. Lay Mine and a mine-trigger module are new code.
- **Size S plus the new code; first slice GM.**

### The Drain toggle and Ground fields
- **Archetypes:** built are the Sludge (two tanks), Shroud's cloud and the sludge itself. M6 adds the one shared tank (0468) and smoke (SC).
- **Modules now:**
  - `patches.rs`: fixed `Tanks { sludge, smoke }` fields and a `TankDef` per patch kind, with the numbers read from the Shadebringer (`patches.rs:205`). `Ability::Sludge` and `Cover` are one variant per patch kind.
  - `shroud.rs`: the clouds.
  - The client: `patches_view.rs` and `shroud_view.rs`. So ground fields are built twice.
- **The refactor:**
  - one `DrainPool { capacity, refill, min }`, with `toggles: [{ lays, drain }]` and one `Toggle { i, on }` ability;
  - a `FieldDef` table by `PatchKind`;
  - one client rule for who sees a field.
- **Size S–M; first slice SC.**

### The rest, in one line each
- **Production:** `production.rs` with `sim/producers`, the queue and pool producers, and `ExitRule` Side, Sides, Centre, Network, AtOnce, Runway and Pad. One module; no refactor.
- **Construction:** data-driven.
  - Placed by a constructor: `construction.rs`.
  - The builder inside: `construction/site.rs` with `sim/sites.rs` (`BuilderGoes { Network, Outside }`).
  - The builders outside: `spot_work.rs`.
  - GB needs `builds` taken out of `WorkerDef` (`defs/units/worker.rs:98` puts it beside the mandatory `crystals`), and the Engineer's exit through the door or at the edge.
- **Gathering:** `orders/gather.rs` with `gather/`, and `economy/gathering.rs`, keyed by `WorkerDef`; `economy/mining.rs` for the extractor. One module.
- **Energy pools, casts and timed effects:**
  - The pools: `energy.rs`, generic.
  - The casts: `SpellDef::Cast(AreaAbility)` with `buffs.rs`, generic, one enum variant per ability. The only channel is Shroud, through a spell-named `OrderKind::Shroud`. Mirage is its own Ability.
  - The timed effects: `BuffKind` in `buffs.rs`, with `cooldown.rs` and `suppress.rs`. One module.

## 3. Hubs: what a new mechanic must edit today
- **A new order kind:** 17 exhaustive matches in 13 files, none with a wildcard.
  - In the sim:
    - `orders.rs:252,292`;
    - `orders/destinations.rs:101`;
    - `orders/hash.rs:30` (a hash tag per kind);
    - `orders/run.rs:189,237,304`;
    - `sim/commands.rs:98`;
    - `sim/gate/orders.rs:41`;
    - `testing/invariants.rs:540` and `testing/commands.rs:265`;
    - `tether.rs:91`;
    - `vehicle_deploy.rs:111`.
  - In the client: `command_ui/indicators.rs:154,194`; `resolution/pointer.rs:72`; `systems/outbox.rs:323`.
  - With a button, also:
    - a `PanelCommand` variant and its 4 matches (`commands/flow.rs`, `commands/names.rs`);
    - per new `Reject`, a text and an arm in `feedback.rs`;
    - per new `SimEvent`, an arm in `game/viewer_events.rs:381`.
- **A new Ability:** 5 exhaustive matches in 3 files (`command.rs:186`, `sim/modes.rs:38,68`, `sim/gate/orders.rs:502,528`). Then a `PanelCommand`, a grid block in `templates.rs` and an arm in `conditions.rs`.
- **A new unit type:**
  - a `kind_registry!` row and its def module;
  - making a greyed "planned" entry live edits `planned.rs` in 5 places;
  - a unit with its own panel also needs a template variant and its arms (`templates.rs:53,298`, `resolution.rs:271`).

  How many hubs each slice touched: GL, a unit built from existing parts, 2; GS1 14; ST1 12; SB 11.
- **A new capability:**
  - the `Object` literal (`sim.rs:435`) and the hash destructure (`hash.rs:309`);
  - `UnitDef::NONE` (`defs/defaults.rs:45`);
  - in `sim/kinds.rs`, fresh and `change_kind`; and `sim/load.rs:152-185`;
  - the removal path's forget;
  - the `Unit` borrow (`orders/run.rs:86`, `orders/world.rs:82`);
  - `Abilities` (`resolution.rs:203`) and a grid block in `templates.rs`;
  - an invariant module and its link in the chain (`invariants.rs:35,213`);
  - a step call in `phase.rs`.
- **The hub files:**
  - 7.7k lines and 455k characters, which is 16–39k each;
  - 38.6% of their lines are comments, and 659 lines cite a slice id;
  - the `//!` docs of `planned`, `invariants`, `templates`, `feedback` and `conditions` (39, 33, 22, 62 and 115 lines) are running changelogs that every slice appends to.
- **X4's proposal:**
  1. **An order registry** (L; after M6): an `order_registry!`, like `kind_registry!`, generates `OrderKind::spec() -> &OrderSpec { payload, targets, slots, hash_tag, queueable, holds_orders, tethered_ok, moves_deployed }`. Each order's module implements `trait OrderBehaviour { check, start, run }`, and the 13 sim matches become lookups or trait calls. It leaves the goldens unchanged if the hash tags keep their numbers.
  2. **A capability registry** (M): a `trait Capability { fresh, hash, forget, invariant, on_change_kind }`, listed in one file that `new_object`, load, `change_kind`, removal and the invariants iterate. The hash tag order is kept.
  3. **The client** (S): generate the `PanelCommand` metadata and `game/kinds.rs` from tables, and move the `//!` changelogs into git history. That cuts about 10% of the hubs' characters.

## 4. For the Designer
- **0484: the Recruits already follow all seven rules,** on the Drones' and Agents' code path. There is one Gather behaviour, keyed by `UnitDef::worker`, and the Recruit's parameters are data (`crates/sim/src/defs/cults/units/recruit.rs:64-82`).
  1. **The last load of a patch** takes `remaining.min(capacity)` (`economy/gathering.rs:283`).
  2. **Interrupted mining:** the frames live in the order's state, so a Recruit ordered away collects nothing.
  3. **Other orders keep the load:** `Worker::load` is written only at delivery, at mining and when a Recruit is trained.
  4. **Who sees the load:**
     - the model's load marker shows to anyone who sees the unit (`game/object_render/load.rs:45`);
     - "Carrying" in the HUD (`game/hud/info_panel.rs:245-251`) and the drop-off indicator lines (`game/command_ui/indicators.rs:254-263`) are owner-only.
  5. **Gather while carrying a load:** a loaded worker goes straight to delivery, with no claim (`orders/gather/patch.rs:51`).
  6. **The Drop Off order:** the gate checks it (`sim/workers.rs:82,112`), it delivers and then the order is done (`orders/gather/deliver.rs:61,201`), and the right-click resolves per unit (`game/command_ui/resolution.rs:304-313`).
  7. **When a Gather ends,** it delivers the load first (`orders/gather/patch.rs:142,188`).
  - **So 0484's phase is tests and citations only.**
  - **Open point:** an empty Recruit's right-click on a damaged, completed own Storage gives Repair, and on an unfinished one gives Assist (`orders/resolution.rs:283-285`, following the Recruit's right-click table). The gathering doc says "Empty → plain Move".
- **0483: the sludge differs from the doc.**
  - The client shows a patch whole when the tile under its centre is Visible, and hides it otherwise (`sludge_discs` in `crates/game/src/patches_view.rs:85-93`, which still calls this a placeholder). Nothing is remembered on Explored tiles, and every player gets the same rule.
  - Shroud's cloud is clipped per Visible tile (`crates/game/src/shroud_view.rs`). So a sludge disc that straddles a Visible tile and a fogged one shows all or nothing, while a cloud shows only its Visible part.
  - The sim keeps every patch globally (`Sim::patches`). Smoke isn't drawn yet.
- **0474 is built generically, not by structure type.**
  - **How:** `Sim::landed_on` (`sim/landing.rs:187-196`, called from `sim/removal.rs:90-92`) kills every object whose `landing::at` names the removed object, landing or down (`landing.rs:98-105`). One already taking off has no `Landing`, and escapes. So a Chopper landed on its Tower already dies with it; only the comments still say "Airfield".
  - **Where aircraft land today:**
    - the Airfield's 4 slots: the Hornet and the Dragonfly;
    - the Supply Tower's spot and the Supply Delivery Station's centre: the Supply Chopper (the Station is indestructible).
  - The Tunnels' air doors are entries into the network, not landings, and the Sparrow never lands.

## 5. Refactor candidates, and the order
### The candidates
For each: where it lives today, the shape it should take, its size, and whether the goldens stay unchanged.
- **The landing capability** (L):
  - **Where:** `landing.rs`, `sim/landing.rs`, `supply_run.rs`, `sim/supply_run.rs`, `aircraft.rs`, `movement/approach.rs`.
  - **The shape:** `Object::landing` with a `LandingDef`, one step, and spot providers.
  - **Goldens:** unchanged only if `hash.rs` keeps writing the landing under tags 14 and 27; otherwise re-bless `gdo_air`, `gdo_dragonflies` and `gdo_supplies`.
- **The carrier archetypes** (M): `carry.rs`, `sim/holding/carriers.rs`, `orders/unload.rs`, `orders/enter.rs`. The shape: `archetype` and `paradrop`, with `UnloadAll(Option<Vec2>)`. Goldens unchanged if `UnloadAll(None)` hashes as today.
- **Ammo against its base** (M): `aircraft.rs`, `orders/air_base.rs`, `network/services.rs`. The shape: `AmmoDef.rearm: Assigned | Network`, with the Vault rearming. Goldens unchanged.
- **The Deploy archetype** (M): `vehicle_deploy.rs`, `deploy.rs`, `transform.rs`. The shape: a `targeted` flag, and `into` a structure. Goldens unchanged.
- **Orders held during a process** (S): `orders.rs:373` (VehicleDeploy), `orders/run.rs:80-84` (`Mover::launching`), and `movement/glider.rs` (the runway run). The shape: one `holds_orders()` predicate. Goldens unchanged.
- **The stock purchase** (S): `stock.rs:68`. The shape: a `purchase` rule and a queued count. Goldens unchanged if the count hashes only when non-zero.
- **The builder part** (S): `defs/units/worker.rs:98`. The shape: `UnitDef::builds`, as `repairs` moved in M5-N9. Goldens unchanged.
- **HP restoration** (M): `repair.rs`, `regeneration.rs`, `buffs.rs:321`, `economy/supply_run.rs:163`. The shape: one module, with a `HealDef` for the area and the docked service. Goldens unchanged if the f32 operation order is kept.
- **The drain pool** (S–M; 0468): `patches.rs`, and Sludge and Cover in `command.rs`. The shape: one pool, a drain per toggle, and a `FieldDef` table. Goldens move: 0468 changes behaviour, so re-bless `syndicate_sludge`.
- **Ground fields** (M): `shroud.rs`, `patches.rs`, and the client's `shroud_view.rs` and `patches_view.rs`. The shape: a shared "centre in field" lookup and visibility rule. Goldens unchanged in the sim.
- **The garrison holder** (M): `holding.rs`, `combat/occupants.rs`, `sim/holding/evacuation.rs`. The shape: one `HoldDef` on `ObjectDef` (§2). Goldens unchanged if `Place` hashes as before.
- **The glider approach** (S–M): `movement/approach.rs`, where the Glider's `next_speed` is written again. The shape: any spot, an optional heading, and a shared `next_speed`. Goldens unchanged.
- **The toggle modes** (M): `toggle.rs`, `riot_shield.rs`, `conversion.rs`. The shape: one `ModeToggle`. Goldens unchanged only if the two states hash as today.
- **The order and capability registries** (L and M): §3. Goldens unchanged if the tags are kept.

### Recommended order
- **Before GS2:** the landing capability, then the carrier archetypes.
- **Before GB:** the builder part; the HP restoration module (the Tent's `HealDef`, and the Tower's docked repair); the Bunker's rows.
- **Before GM:** the stock purchase rule.
- **Before GP:** Deploy's `targeted` and into-a-structure, and the held-orders predicate.
- **Before SC:** the drain pool (0468) and the `FieldDef` table; the client's field visibility, if 0483 means per tile.
- **Before SA:** ammo against its base, and the approach at the air doors. Both build on the landing capability.
- **Before CP:** the runway landing (in the landing capability), the Docked carrier and Paradrop (in the carrier), and the held-orders predicate for the take-off run.
- **Before CB:** the garrison holder, after GB.
- **No refactor needed:**
  - SL and ST2 are rows only;
  - SR (the relay) and SP's thrust are specific to one unit by design.
- **After M6:**
  - the toggles;
  - one type for the energy and tank pools;
  - one sim model for the ground fields;
  - the order and capability registries;
  - the `//!` changelogs.

## 6. Legitimate per-kind code (don't refactor)
- **The factions' economy rules:**
  - `economy/army.rs:93-96,244`;
  - the GDO build area (`placement/build_area.rs:133`);
  - the Syndicate's manned units (`network/services.rs:112`).
- **Rules the design ties to one object:**
  - the network's rules on the Tunnel's def (`network.rs:63`);
  - the Null Chamber's service (`network/services.rs:119`);
  - the Supply Delivery Station (`economy/stations.rs:30`);
  - the shape a gone crystal patch had (`orders/gather/legs.rs:34`).
- **The factions' lists in the defs:**
  - the Syndicate's Eject menu and expansions (`defs/syndicate.rs:40-49,95-104`);
  - the Cults' build menu (`defs/cults.rs:120-132`);
  - the GDO's `PLANNED_RECIPES` (`defs/gdo.rs:40-42`).
- **Specific to one unit by design:**
  - the Drone's deploy into a Beacon, and the Relay;
  - the Cults' recruitment and augmentation;
  - the Syndicate's tunnels and network;
  - the Laser Turret's pop-up;
  - the Rocketman's target lock;
  - the Copy.
- **Not covered by the audit:** `crates/ai`, and the client's per-kind code beyond the counts.
