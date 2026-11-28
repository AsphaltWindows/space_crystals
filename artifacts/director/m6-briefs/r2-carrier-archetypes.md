# Task R2: The carrier archetypes, with the Supply Chopper as the first Landing carrier (a mechanic phase before GS2)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a mechanic phase, and your budget is 120 tool calls, a hard limit.**

## Why
The design defines carrying once, in `systems/mechanics/carrying.md`, with three archetypes. The code's carrier (ST1's, for the Subterranean Transport) is shaped for the Ground carrier only:
- `carry::loads()` means "is a ground object";
- Unload All has no target (`carry::ring_spots`);
- nothing makes a carrier land.

The Supply Chopper (GS2) is a Landing carrier, and the Pilgrim (CP) a Docked carrier with Paradrop. This phase shapes the carrier by archetype, in its def, and makes the Chopper the first Landing carrier in the sim.

## Where things stand
- **R1a and R1b** (`1013d5d6`, `9ac5b9ef`) made landing one capability, with Field landing at a point. Land the carrier through it. From R1b's handoff:
  - **The state:** `Object::landing: Option<Landing { at: Spot::Structure(id) | Spot::Field(point), descent, rearm }>` (`crates/sim/src/landing.rs`).
  - **The def:** `UnitDef::landing: Option<LandingDef { landing_ticks, takeoff_ticks }>`, read with `landing::def_of`. A field landing needs a `LandingDef`: the Chopper has one.
  - **The step:** `Sim::step_landings` (`sim/landing.rs`, right after 5a), with `landing::flies_to`, `pads`, `claim`, `Sim::spot_rules`, `landing::take_off` and `lift_off`.
  - **To land at a point,** the carrier's order calls `mover.land_at(point)` in every 5a tick until the unit is down. The landing step takes the call in the same tick, so it is never saved or hashed. A blocked point becomes the nearest free one (`Sim::field_spot`).
  - **To know it has landed:** `landing::down(o)`, and `o.landing.unwrap().at.field()` is `Some(spot)`; in 5a, `orders::world::Unit::landing` gives the same. Any new order lifts it off, or call `landing::lift_off` once the loading or unloading is done.
  - **The hash:** a field spot is sub-tag 1 and the point, inside the landing's hash. `Mover` tag 17 is still free.
  - **Open (Q0202):** a unit landed in the field stays on the air layer, so ground units walk under it. Build on that; don't make it block them.
- **The mechanics audit,** `artifacts/director/x4-mechanics-audit.md` §2 "Carrying" and §5 "The carrier archetypes". Read those parts only.
- **The carrier now:**
  - `crates/sim/src/carry.rs`;
  - `Sim::step_unloads` in `crates/sim/src/sim/holding/carriers.rs`;
  - `crates/sim/src/orders/unload.rs`;
  - `crates/sim/src/orders/enter.rs`, with `crates/sim/src/sim/network/entries.rs`;
  - `Place::Carrier`;
  - `UnitDef::carries`, ST1's `CarrierDef { capacity, places }`, with places counted per base. A capacity of 4 with a vehicle taking 4 places already allows only one kind of passenger at a time.
  - The boarding gate and `Sim::step_unloads` act only on a carrier that `carry::loads`. A landed aircraft counts, through `object_domain`'s landed arm.
  - The passengers come out on `carry::ring_spots`, rings around the carrier's turned shape.
- **The Chopper's cargo** (GS1): Supplies only, an enum with one kind at a time, shaped so that units can be added as a second kind.

## The design
- `systems/mechanics/carrying.md`, whole: it is short.
- `factions/gdo/units/supply_chopper.md`: `> Cargo`, `> Unit Transport` and `> ObjectInterfaceState[SupplyChopper]`.
- `artifacts/designer/changes/0442-supply-chopper-transport.md`: the Chopper as a transport.

## The architecture
The core, then the Carriers paragraphs of `docs/architecture/object-model-state.md`: `grep -n -i 'carrier\|passenger' docs/architecture/object-model-state.md`.

## Goal
1. **The archetype is part of the carrier's def,** and the logic reads it, never the unit's kind:
   - `Ground { loading_state }`, as built for the Transport;
   - `Landing`.

   Leave room for `Docked { structure, side }` and the Paradrop addition (phase R3), but don't add them now.
2. **Who can board** is a parameter of the def, if it isn't already.
   - A one-way unit (`sc_sim::network::one_way(kind)`, SP2b) must never ride a carrier into the Tunnel Network. Today no carrier can take one; keep it so, with a test if a carrier could.
3. **The Landing carrier:**
   1. **Enter:** units right-clicked onto it make it land where it is (Field landing). They walk to it and board instantly on reaching it.
   2. **Load:** an optional input of the def. The carrier flies to an own eligible unit, lands beside it, and the unit walks in.
   3. **Unload All is targeted:** it flies to the point, lands, and everyone comes out around it. Make Unload All carry an optional point, so that the Ground carrier's untargeted form hashes and behaves exactly as now.
   4. **It takes off by itself** once loading or unloading is done.
   5. **The unloading rules** are the same for every archetype:
      - on the move, it stops (or lands) first;
      - the passengers take the nearest free spots;
      - those with no free spot stay aboard.
4. **The Chopper is the first Landing carrier,** in the sim only:
   - its def's carrier row: 4 infantry places, or 1 vehicle;
   - its cargo is Supplies **or** units, never both.

   GS2 adds the refusals between the two kinds, the panel and the client.
5. **Passengers die with the carrier,** landed or flying, as now.

## Tests
- **The Transport's existing carrier tests stay green unchanged:** they are the Ground carrier's spec.
- **The Landing carrier, on the Chopper:**
  - four Peacekeepers Entering a hovering Chopper make it land; they board, and it takes off;
  - a fifth stays outside, idle;
  - a Load on an own unit;
  - a targeted Unload All across the map: it flies, lands, the passengers come out around it, and it takes off;
  - an Unload All on a blocked point lands at the nearest free one;
  - a crowded unload leaves the rest aboard;
  - one vehicle boards alone;
  - shot down while loaded, everyone dies.
- **Goldens:** none may move.
- **Hash tags:** new `Object` state hashes and saves only while present, under **Object tag 29**. Load and the targeted Unload All hash as **`OrderKind` tags 36 and 37** in `orders/hash.rs`, or reuse Unload All's tag with the point written only when set. A new behaviour hashes as **`Behaviour` tag 22**, and new `Mover` state as **`Mover` tag 17** (R1 didn't take it). These were GS2's reservations, and R1 and R2 now hold them.

## Don't
- Don't build the Docked carrier or Paradrop (R3), or the client: no panel, no targeting, no pips (GS2).
- Don't change the Transport's behaviour.
- Don't edit anything under `artifacts/`.

## Report
At most 10 lines, then **Handoff** for GS2 (the Chopper's entity phase) and R3 (the Docked carrier and Paradrop), at most 8 lines:
- the archetype's def fields;
- the orders and their tags;
- what GS2's client must call;
- where R3 adds the Docked carrier.
