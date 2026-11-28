# Task SLb: An order Shift-queued behind a queued mode switch is judged against the type the unit will then be (a mechanic fix, phase 2 of M6-SL)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a small mechanic phase, in `crates/sim` only, and your budget is 50 tool calls, a hard limit.**

## The bug
M6-GA built the vehicle deploy model (`crates/sim/src/vehicle_deploy.rs`, `crates/sim/src/orders/vehicle_deploy.rs`). Its report found one bug:
- An order Shift-queued behind a queued Undeploy is judged at the gate against the deployed type, the unit's type now.
- So a Move Shift-queued behind a queued Undeploy is refused, though the unit will be mobile when the Move runs.

## The design
`systems/mechanics/modes.md > Archetype: Deploy (an order)`:
- "**Queueable**: Shift-Deploy waits until the earlier orders are done";
- "**Orders given while switching**": once the switch finishes, "each order is checked against the new type: one it can't carry out (e.g. Move for a unit that can't move) is **dropped** with the usual unavailable-command feedback, and the next is tried."

So the type an order is judged against is the type the unit will be when the order runs.

## Goal
1. **Reproduce it first,** in a test on the Artillery Rig, GA's pair: deployed, given a Shift-queued Undeploy and then a Shift-queued Move. If the gate already accepts the Move, the bug is gone: say so in your report, and stop after adding the test.
2. **The fix:** judge a Shift-queued order against the type the unit will be after any switch queued before it, with GA's `vehicle_deploy::can_take`. Find where the gate judges queued orders (`grep -n 'can_take' crates/sim/src/sim/gate/*.rs crates/sim/src/vehicle_deploy.rs`).
   - Keep it shaped by the def's mode pairs, not by the Rig's kind, so the Laser Battery's pair gets it too (SLc adds its data).
   - The check when the order runs stays as it is: an order the new type can't carry out is dropped there, with the usual feedback.
3. **Tests:**
   - a Move Shift-queued behind a queued Undeploy is accepted, and runs once the Rig is mobile;
   - a second Deploy Shift-queued behind a queued Deploy is judged against the deployed type, which can only Undeploy, so the gate refuses it;
   - an attack on air Shift-queued behind a queued Deploy is judged against the deployed type's domains;
   - orders given while the switch runs keep their rule, the last one held (GA's tests stay green).
4. **Goldens:** none should move.

## Don't
- Don't change the deploy's timings, the held order, or which orders each type takes.
- Don't touch the surge attack or the energy pool (SLa's), or the Laser Battery's defs (SLc's).
- Don't edit anything under `artifacts/`.

## Report
At most 6 lines, then **Handoff** for SLc, at most 3 lines: where the queued type is judged, and what SLc must test again on the Laser Battery's pair.
