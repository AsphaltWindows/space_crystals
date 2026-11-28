# Task SP2c: The Prism's panel, its drawing, docs and QA (phase 2c of M6-SP, the entity phase)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is an entity phase, and your budget is 120 tool calls, a hard limit.**

## Where things stand
- **SP phase 1** (`971ba38a`): the Prism is in production, and it launches (rise, then unfold) with its orders held.
- **SP2a:** the thrust charges (`OrderKind::Thrust`), a sim mechanic. Its handoff: FILLED IN AT LAUNCH.
- **SP2b:** the one-way rule, a sim mechanic. Its handoff: FILLED IN AT LAUNCH.

## The design
- `factions/syndicate/units/prism.md`, in particular the panel (designer placeholder; Q0168):
  - Q Move, R Thrust, X Stop;
  - A Attack only while armed (that's slice SR, not you);
  - Thrust is targeted: a left-click on a point.

## Goal
1. **The panel:** Q Move, R Thrust, X Stop.
   - `PanelCommand::Thrust` is targeted and queueable.
   - R is greyed by its probe when no charge is left (SP2a's handoff says what to call).
   - Remove any client placeholder for the Prism's panel.
2. **The right-click,** in the client's `right_click_orders`: a one-way unit's right-click on an own Tunnel doesn't order it in. SP2b's handoff says what it resolves to.
3. **Drawing:** the unfold (a simple placeholder look is fine), and a thrust, if SP2a's handoff asks for one.
4. **Docs:**
   - regenerate the controls doc: `UPDATE_DOCS=1 cargo test -p space_crystals --test game playtest_docs::controls`;
   - add the Prism to the docs that list units (`grep -rln 'Phantom' docs` shows where).
5. **QA screenshots** into `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/m6sp/`. Look at each yourself:
   - the Prism's panel, with R greyed after three thrusts;
   - a Prism unfolding over its Aerospace Assembly;
   - a thrust in flight.

   The QA mode: find it with `grep -n 'QA mode' ARCHITECTURE.md docs/architecture/*.md`, then read that section only. Check that `cargo run` starts.
6. **Tests:** client tests of the panel, its greying and the right-click.

## Don't
- Don't change the sim beyond what a client test needs. If the sim lacks something, say so in your report.
- Don't edit anything under `artifacts/`.

## Report
At most 10 lines, then **Handoff** for slice SR (the relay), which adds A once the Prism is armed.
