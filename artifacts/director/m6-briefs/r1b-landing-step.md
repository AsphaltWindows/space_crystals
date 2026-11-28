# Task R1b: One landing step for any spot, the landed rules for any unit, and Field landing (the behaviour half of the landing refactor, before GS2)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a mechanic phase, a refactor, and your budget is 120 tool calls, a hard limit.**

## Why
The design defines landing once, in `systems/mechanics/aircraft_landing.md`. The code builds it twice, each time around its first user:
- the Airfield's landing: `crates/sim/src/landing.rs` and `crates/sim/src/sim/landing.rs`;
- the Supply Chopper's: `fly_to_spot` in `crates/sim/src/sim/supply_run.rs`, near line 285.

The rest of M6 lands through the same rules: the Chopper on open ground (GS2), the Pilgrim on a runway (CP), and the Specter at the air doors (SA). This phase makes landing **one capability**, which any aircraft uses through its def. **Behaviour is unchanged.**

## Where things stand
**R1a** (`1013d5d6`) made the landing state and the def's landing part one each, with one take-off. Behaviour and every golden are unchanged. Build on it:
- **The state** (`crates/sim/src/landing.rs`): `Object::landing: Option<landing::Landing>`, holding `at: Spot`, `descent` and `rearm`.
  - `Spot` has one kind, `Spot::Structure(ObjectId)`, plus `Spot::structure()`. The L1 invariant destructures it with an irrefutable `let Spot::Structure(..)`, so the compiler flags it when you add a kind.
  - Taking off is not in the state: the take-off clears it and starts `Mover::rise`, which the air doors share.
  - `orders::world::Unit` holds a copy of the landing, which `orders/run.rs` writes back.
- **The def:** `UnitDef::landing: Option<LandingDef { landing_ticks, takeoff_ticks }>` (`defs/units.rs`), read with `landing::def_of`. The Chopper has one (16/16). The Airfield's aircraft have none: they land in `AirfieldDef::landing_ticks`, which the design gives as the Airfield's own parameter (`airfield.md > Aircraft Support`). If your step needs a unit's landing archetype (`aircraft_landing.md > Landing archetypes`), put it on `LandingDef`.
- **One take-off:** `landing::take_off(kind, unit, base, &mut landing, &mut rise)`, with `lands_on` (the old rules merged) and `lift_off`. `rise_from` takes the take-off time.
- **The two steps** both read `Object::landing` now:
  - `fly_home` and `land_a_frame` in `sim/landing.rs` filter on `o.aircraft`;
  - `fly_to_spot`, `land_a_frame`, the eviction and the spawn in `sim/supply_run.rs` filter on `o.chopper`. The Chopper's schedule lead is `takeoff + landing − 1`.
- **Hold fire:** `aircraft::may_fire` already holds fire for any landed unit.
- **The hash:** the landing is written where it always was, in the same order: inside Object tag 14 (sub-tag 1) for the Airfield's aircraft, and inside tag 27 for the Chopper. **Object tag 36** is for a landing on a unit with neither state; nothing writes it yet.
- **Still from the mechanics audit** (`artifacts/director/x4-mechanics-audit.md` §3 "Landing" and §5 "The landing capability"; read those parts only):
  - **The approach:** `crates/sim/src/movement/approach.rs`, the Gliders', is called only from `sim/landing.rs`, and writes the Glider's `next_speed` again.
  - **Destroyed with its structure:** `Sim::landed_on` (`sim/landing.rs`, called from `sim/removal.rs`) kills every object whose landing names a removed object. That rule is already generic: keep it so.
- **Left by R1a:** `docs/architecture/object-model-types.md` still describes the landing as part of `AircraftState`. Fix it with your docs (Goal 5).

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
2. **The landed rules, for any landed unit:** it holds fire (R1a's `may_fire`), it is a ground target, it is out of the air separation, and it dies with its structure.
3. **Field landing** (`aircraft_landing.md > Field landing`): landing at a point on open ground, a new kind of spot. GS2 uses it to load and unload the Chopper, so build it now, without any order or button.
   - The point must be ground that infantry can walk on, with no building or unit in the way.
   - A blocked point means the nearest free one, found by a bounded search, as `free_spot` in `crates/sim/src/sim/spot_work.rs` does.
   - Field landing is a new kind of `Spot`, hashed with the landing, so that no golden sees it. Keep a `Spot::Structure` landing's bytes as they are, and give the new kind a sub-tag of its own. If it needs `Mover` state of its own, use **`Mover` tag 17**, GS2's reservation, which R1 and R2 now hold.
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
