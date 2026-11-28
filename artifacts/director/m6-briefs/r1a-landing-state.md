# Task R1a: One landing state and one landing part in the def (the data half of the landing refactor, before GS2)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a mechanic phase, a refactor, and your budget is 120 tool calls.** Phase R1b, after you, makes the step one; you make the data one.

## Why
The design defines landing once, in `systems/mechanics/aircraft_landing.md`. The code builds it twice, each time around its first user:
- the Airfield's landing: `crates/sim/src/landing.rs` and `crates/sim/src/sim/landing.rs`;
- the Supply Chopper's: `fly_to_spot` in `crates/sim/src/sim/supply_run.rs`, near line 285.

The rest of M6 lands through the same rules: the Chopper on open ground (GS2), the Pilgrim on a runway (CP), and the Specter at the air doors (SA). This phase makes landing **one capability**, which any aircraft uses through its def. **Behaviour is unchanged.**

## Where things stand
- **A1** (`f23eb029`) changed the return for 0466. The flight home (`orders/air_base.rs`) now ends once the aircraft is down, so an order queued behind it starts on landing. A landed aircraft with no order goes on rearming (`landing::rearm`). Build on this version.
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
- `factions/gdo/structures/airfield.md > Landing slots`.

## The architecture
The core, then the landing and Airfield paragraphs of `docs/architecture/production.md`, and those of `docs/architecture/movement-and-collision.md`. Find them with `grep -n -i 'landing\|airfield\|approach' <file>`.

## Goal
1. **One landing state** on the `Object`, for any aircraft, in place of the `Landing` in `AircraftState` and the one in `ChopperState`:
   - where it lands: a spot on a structure (the Airfield's slots, the Supply Tower's spot, the Supply Delivery Station's centre);
   - its progress: descending, landed, taking off.

   Name it by the code's conventions. Shape it so that a point on the ground (R1b), a runway (R3) and an air door (R6) can be added as new kinds of spot without reshaping it, but don't add them now.
2. **One landing part in the aircraft's def,** holding the landing and take-off times, in place of the times on `AirfieldDef` and `SupplyChopperDef`.
   - If the design gives a time as the base structure's parameter, not the aircraft's, keep it on the base, and say so in your report.
   - Each number keeps its value and its citation.
3. **One take-off,** in place of `landing::take_off` and `supply_run::take_off`.
4. **Keep the two landing steps** (the Airfield's in `sim/landing.rs`, the Chopper's `fly_to_spot` in `sim/supply_run.rs`) as they are, except that both now read and write the one state and the one def part. Phase R1b merges them.
5. **The hash: every golden stays unchanged.** The landing is hashed today inside the aircraft's state (Object tag 14, sub-tag 1, only while set) and inside the Chopper's (Object tag 27). Keep writing it there, in the same order, for those units, even though it now lives in one field. Record any new tag in your report.

## Tests
The existing landing, Airfield, supply and air tests are the spec: they stay green unchanged. If one must change, say why in your report. Add a test only where the new state has a rule of its own (for example, that a unit's landing state is cleared as it takes off).

## Don't
- Don't change any landing rule or number: the slots' layout, the approach, the Dragonfly's vertical landing, the Chopper's schedule, the rearm.
- Don't merge the two landing steps, or build Field landing (both R1b), the runway landing (R3), the air doors' descent (R6), the carrier archetypes (R2), or any order, button or client change beyond what compiling needs.
- Don't edit anything under `artifacts/`.

## Report
At most 8 lines, then **Handoff** for phase R1b, at most 6 lines: the landing state and its def part (types, files), the one take-off, where the two steps read them, and the hash.
