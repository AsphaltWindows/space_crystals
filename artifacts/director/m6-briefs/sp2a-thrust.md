# Task SP2a: The thrust charges, as a sim mechanic (phase 2a of M6-SP)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a mechanic phase, in `crates/sim` only, and your budget is 110 tool calls, a hard limit.**

## Where things stand
**M6-SP phase 1** (`971ba38a`) put the Orbital Command and the Prism into production, with the Prism's launch:
- the air door's rise, then `Mover::unfold` (32 ticks), with its orders held meanwhile (`Mover::launching()`);
- `UnitDef::satellite`, which `services::manned` skips.

The Prism's def already carries `thrust: Option<ThrustDef>` (3 charges, 3 GU, 16 ticks), but nothing reads it yet. The one-way rule (`one_way`) belongs to SP2b, the next phase.

## The architecture
Read the core, then only these paragraphs, found by keyword:
- the HoverCraft's movement and its clamp: `grep -n -i 'hovercraft\|clamp' docs/architecture/movement-and-collision.md`;
- an Object's optional state, its hash and save: `grep -n -i 'optional\|tanks' docs/architecture/object-model-state.md`;
- adding an order, its behaviour and its gate: `grep -n -i 'OrderKind\|Reject\|gate' docs/architecture/behaviours.md docs/architecture/commands-events-replays.md`.

## The design
- `factions/syndicate/units/prism.md`: `> Movement` (the thrust charges, which never come back) and its placeholder stats.
- The change entry `artifacts/designer/changes/0434-prism-panel-thrust.md` (Q0168), with its Verification:
  - Thrust is targeted. It always moves the full 3 GU toward the point, whatever the point's distance.
  - It Shift-queues, and each queued thrust spends its charge when it starts.
  - Orders given during the launch are held (last one wins), as phase 1 built.
- **Q0198 (open, not blocking):** a queued thrust that starts with no charge left. Proceed on this reading:
  - the gate refuses a thrust only when every charge is spent;
  - a queued thrust that starts with no charge left ends at once, without moving, and the queue goes on.
  
  Mark it `Placeholder (Q0198)`.

## Goal
Shape the thrust by its def field, so that any unit with a `ThrustDef` gets it. Nothing should test for the Prism's kind.
1. **A new `crates/sim/src/thrust.rs`:** `Thrust { used, burst }`, modelled on `patches::Tanks`:
   - `fresh(kind)` in `Sim::new_object`;
   - the fill-in in `sim/load.rs`;
   - `thrust: _` in `sim/kinds.rs`;
   - **Object hash tag 17**, hashed and saved only while present.
2. **The order:** `OrderKind::Thrust(Vec2)` (**tag 24**) and `Behaviour::Thrust { fired }` (**tag 17**), in `orders/run.rs`.
3. **The gate:** it refuses a unit without a thrust (`Reject::LacksAbility`), and one whose charges are all spent (a new `Reject::NoCharges`).
4. **The burst:** apply it in `movement.rs` after the HoverCraft clamp. The clamp writes the move into `mover.velocity`, and would carry the burst into the drift.
5. **Invariants:** give `testing/invariants.rs`'s I3 and I5/I7 allowances for the burst.
6. **Tests,** at unit level and in a new `crates/sim/tests/prism/`:
   - a thrust toward a point 10 GU away moves 3 GU toward it, and so does one toward a point 1 GU away. The drift afterwards is the HoverCraft's own;
   - three thrusts in a row, then a refusal with `NoCharges`. A unit without a thrust is refused with `LacksAbility`;
   - three Shift-queued thrusts run one after another. A fourth queued one ends without moving;
   - a thrust given while the Prism launches is held, and runs once it has unfolded;
   - save and load mid-burst, and the hash with and without a charge spent;
   - the X1/X2 tests for the `thrust` def field.
7. **Goldens:** none should move. No golden has a Prism.

## Don't
- No client work: the Prism's panel and drawing belong to SP2c.
- Don't touch `one_way`: it belongs to SP2b.
- Don't edit anything under `artifacts/`.

## Report
At most 8 lines, then **Handoff**, at most 4 lines, for SP2b and SP2c:
- the `Reject`s and their texts;
- what the panel's probe should call to grey R;
- anything the client must draw for a thrust.
