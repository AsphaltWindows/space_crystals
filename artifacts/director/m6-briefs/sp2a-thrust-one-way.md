# Task SP2a: The thrust and the one-way rule, as sim mechanics (phase 2a of M6-SP)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a mechanic phase, in `crates/sim` only, and your budget is 250 tool calls.**

## Where things stand
**M6-SP phase 1** (`971ba38a`) put the Orbital Command and the Prism into production, with the Prism's launch:
- the air door's rise, then `Mover::unfold` (32 ticks), with its orders held meanwhile (`Mover::launching()`);
- `UnitDef::satellite`, which `services::manned` skips.

The Prism's def already carries `one_way`, `unfold_ticks` and `thrust: Option<ThrustDef>` (3 charges, 3 GU, 16 ticks), but nothing reads `one_way` or `thrust` yet.

## The architecture
The core, then the HoverCraft paragraphs of `docs/architecture/movement-and-collision.md` and the Tunnel Network's in `docs/architecture/object-model-holding.md`. Find them with `grep -n -i 'hovercraft\|clamp' ...` and `grep -n -i 'network\|transit' ...`.

## The design
- `factions/syndicate/units/prism.md`: `> One way only`, `> Movement` (the thrust charges), and its placeholder stats.
- The change entry `artifacts/designer/changes/0434-*.md` (Q0168): Thrust is targeted and queues with Shift.

## Goal
Shape both mechanics by their def fields, so that any unit with a `ThrustDef`, or with `one_way`, gets them. Nothing should test for the Prism's kind.

1. **Thrust charges.**
   - **A new `crates/sim/src/thrust.rs`:** `Thrust { used, burst }`, modelled on `patches::Tanks`:
     - `fresh(kind)` in `Sim::new_object`;
     - the fill-in in `sim/load.rs`;
     - `thrust: _` in `sim/kinds.rs`;
     - **Object hash tag 17**, hashed and saved only while present.
   - **The order:** `OrderKind::Thrust(Vec2)` (**tag 24**) and `Behaviour::Thrust { fired }` (**tag 17**), in `orders/run.rs`. A thrust toward a point always moves the full 3 GU, whatever the distance; Shift queues it.
   - **The gate:** it refuses a unit without a thrust (`Reject::LacksAbility`) and one with no charge left (a new `Reject::NoCharges`).
   - **The burst:** apply it in `movement.rs` after the HoverCraft clamp, since that clamp writes the move into `mover.velocity` and would carry the burst into the drift.
   - **Invariants:** give `testing/invariants.rs`'s I3 and I5/I7 allowances for it.
2. **One way only.** `NoTransit::OneWay` and `network::lets_in`, on the ways in only:
   - in `check_enter`, with `Reject::OneWay` ("can't re-enter") when every unit of the order is one way;
   - in `sim/network/entries.rs`, near lines 133 and 205;
   - and `Rule::Enter` in `orders/resolution.rs`, for a one-way unit at an own Tunnel.

   The client's `right_click_orders` is for SP2b.
3. **Tests,** at unit level and in `crates/sim/tests/prism/` (new):
   - three thrusts in a row, then refused;
   - Shift-queued thrusts run one after another;
   - a thrust given while the Prism launches is held, and runs once it has unfolded;
   - a one-way unit can't enter a Tunnel, nor be ordered in;
   - a Prism trained with a Null Chamber stays unmanned (phase 1 didn't test this);
   - the X1/X2 tests for the new def fields.
4. **Goldens:** none should move. No golden has a Prism.

## Don't
- No client work: the Prism's panel and drawing are SP2b's.
- Don't edit anything under `artifacts/`.

## Report
At most 10 lines, then **Handoff** for SP2b:
- the `Reject`s and their texts;
- what the panel's probe should call to grey R;
- anything the client must draw.
