# Task SP2b: One way only, as a sim mechanic (phase 2b of M6-SP)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a small mechanic phase, in `crates/sim` only, and your budget is 60 tool calls, a hard limit.**

## Where things stand
- **SP phase 1** (`971ba38a`) put the Prism into production. Its def carries `one_way`, but nothing reads it yet.
- **SP2a** (`a112ace7`) added the thrust charges, `crate::thrust`, keyed by `UnitDef::thrust`. It did not touch `one_way`. From its handoff:
  - it added `Reject::NoCharges` ("every charge is spent") in `crates/sim/src/sim/gate/reject.rs`, beside `Reject::LacksAbility`;
  - `crates/sim/tests/prism/` exists: `main.rs`, `thrust.rs`, `launch.rs` and `saves.rs`. Add your tests in a new file there.

## The architecture
Read the core, then only the Tunnel Network's paragraphs: `grep -n -i 'network\|transit' docs/architecture/object-model-holding.md`.

## The design
`factions/syndicate/units/prism.md > One way only`: it is "the only unit that can't get back into the Tunnel Network".

## Goal
Shape the rule by the def field, so that any unit with `one_way` gets it. Nothing should test for the Prism's kind.
1. **`NoTransit::OneWay` and `network::lets_in`,** on the ways in only:
   - in `check_enter` (`crates/sim/src/sim/network/entries.rs`): a new `Reject::OneWay` when every unit of the order is one way;
   - at the other ways in, in the same file (`grep -n 'fn ' crates/sim/src/sim/network/entries.rs`);
   - in `Rule::Enter` (`orders/resolution.rs`): a one-way unit's right-click on an own Tunnel resolves as for any unit that can't enter.
   
   The client's `right_click_orders` belongs to SP2c.
2. **Tests,** at unit level and in `crates/sim/tests/prism/` (SP2a made it):
   - a Prism can't enter a Tunnel, nor be ordered in;
   - in a group ordered in, the others enter and the Prism doesn't. What the Prism does instead is what `check_enter` already does with a unit that can't enter; say what that is in your report;
   - a Prism trained with a Null Chamber stays unmanned (phase 1 didn't test this);
   - the X1/X2 tests for `one_way`.
3. **Goldens:** none should move.

## Don't
- No client work.
- Don't change the thrust, nor any other unit's ways in.
- Don't edit anything under `artifacts/`.

## Report
At most 6 lines, then **Handoff** for SP2c, at most 3 lines: the `Reject` and its text, and what the right-click resolves to.
