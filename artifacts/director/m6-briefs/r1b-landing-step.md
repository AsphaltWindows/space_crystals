# Task R1b: One landing step for any spot, the landed rules for any unit, and Field landing (the behaviour half of the landing refactor, before GS2)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a mechanic phase, a refactor, and your budget is 120 tool calls.**

## Why
The design defines landing once, in `systems/mechanics/aircraft_landing.md`. The code builds it twice, each time around its first user:
- the Airfield's landing: `crates/sim/src/landing.rs` and `crates/sim/src/sim/landing.rs`;
- the Supply Chopper's: `fly_to_spot` in `crates/sim/src/sim/supply_run.rs`, near line 285.

The rest of M6 lands through the same rules: the Chopper on open ground (GS2), the Pilgrim on a runway (CP), and the Specter at the air doors (SA). This phase makes landing **one capability**, which any aircraft uses through its def. **Behaviour is unchanged.**

## Where things stand
- **R1a** made the landing state and the landing part of the def one each, with one take-off. Its handoff: FILLED IN AT LAUNCH. Build on it.
- **The mechanics audit,** `artifacts/director/x4-mechanics-audit.md` §2 "Aircraft landing and rearming", §3 "Landing" and §5 "The landing capability". Read those parts only. What it found:
  - **The state:** the `Landing` sits in `AircraftState` for the Airfield's aircraft, and in `ChopperState` for the Chopper.
  - **The landing time** is on `AirfieldDef` for the Airfield's aircraft, and on `SupplyChopperDef` for the Chopper.
  - **Two take-offs:** `landing::take_off` and `supply_run::take_off`.
  - **The approach:** `crates/sim/src/movement/approach.rs`, the Gliders', is called only from `sim/landing.rs` (near 210), and writes the Glider's `next_speed` again.
  - **Hold fire:** `aircraft::may_fire` (`crates/sim/src/aircraft.rs`, near 86) reads `AircraftState` only.
  - **Destroyed with its structure:** `Sim::landed_on` (`sim/landing.rs`, 187–196, called from `sim/removal.rs`, 90–92) kills every object whose landing names a removed object. `landing.rs` (98–105) reads the `Landing` of either state. That rule is already generic: keep it so.

## The design
- `systems/mechanics/aircraft_landing.md`:
  - `> Landed aircraft (every archetype)`;
  - `> Landing archetypes`: its `Vertical` and `Field landing` sections. `Runway` is phase R3's.
  - `> Rearming archetypes > Assigned base`, as built.
- `factions/gdo/units/supply_chopper.md`: its landing lines (`grep -n -i 'land' ...`).
- `factions/gdo/structures/airfield.md > Aircraft Support` (its landing slots).

## The architecture
The core, then the landing and Airfield paragraphs of `docs/architecture/production.md`, and those of `docs/architecture/movement-and-collision.md`. Find them with `grep -n -i 'landing\|airfield\|approach' <file>`.

## Goal
1. **One step** for every landing aircraft, in place of the Airfield's (`sim/landing.rs`) and the Chopper's (`fly_to_spot` in `sim/supply_run.rs`): the claims on spots, the approach, the descent and landing.
   - **The approach** (`movement/approach.rs`) becomes callable for any spot, and shares the Glider's `next_speed` instead of writing it again.
   - **The spots come from their structures,** by their def, not by a test of their kind.
2. **The landed rules, for any landed unit:** it holds fire (`aircraft::may_fire` reads the one landing state), it is a ground target, it is out of the air separation, and it dies with its structure.
3. **Field landing** (`aircraft_landing.md > Field landing`): landing at a point on open ground, a new kind of spot. GS2 uses it to load and unload the Chopper, so build it now, without any order or button.
   - The point must be ground that infantry can walk on, with no building or unit in the way.
   - A blocked point means the nearest free one, found by a bounded search, as `free_spot` in `crates/sim/src/sim/spot_work.rs` does.
   - Field landing is new state: hash it under the landing's tag, only while set, so that no golden sees it. If it needs `Mover` state of its own, use **`Mover` tag 17**, GS2's reservation, which R1 and R2 now hold.
4. **The hash: every golden stays unchanged.**
5. **Docs.** One paragraph on the landing capability, in the architecture doc that fits best. Make the old paragraphs on the Airfield's and the Chopper's landing point to it, rather than repeating it.

## Tests
- **The existing landing, Airfield, supply and air tests stay green unchanged.** If one must change, say why in your report.
- **Add tests of the capability itself:**
  - an aircraft lands on a spot and takes off, in its def's times;
  - both an Airfield aircraft and a landed Chopper hold fire and are ground targets;
  - both die with their structure;
  - Field landing on a free point;
  - Field landing on a blocked point, going to the nearest free one;
  - the take-off from a Field landing.

## Don't
- Don't change any landing rule or number: the slots' layout, the approach, the Dragonfly's vertical landing, the Chopper's schedule, the rearm.
- Don't build the runway landing (R3), the air doors' descent (R6), the carrier archetypes (R2), or any order, button or client change beyond what compiling needs.
- Don't edit anything under `artifacts/`.

## Report
At most 10 lines, then **Handoff** for phase R2 (the carrier archetypes) and slice GS2, at most 8 lines:
- the landing state, its part in the def, and the step's entry points;
- how a carrier asks to land at a point, and how it learns that it has landed;
- the hash tags.
