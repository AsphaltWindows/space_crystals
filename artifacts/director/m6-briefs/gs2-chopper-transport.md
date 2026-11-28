# Task GS2: The Supply Chopper carries units: its refusals, panel, drawing, golden and QA (the entity phase)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is an entity phase, and your budget is 120 tool calls, a hard limit.** The mechanics are built: you assemble the Chopper's transport from them.

## Where things stand
- **GS1** built the Chopper's supply loop: its cargo is Supplies, an enum with one kind at a time, shaped so that units can be added as a second kind.
- **R1a and R1b** made landing one capability, with Field landing at a point (`crates/sim/src/landing.rs`, `Sim::step_landings`). Q0202 (open) asks whether ground units pass under a field-landed aircraft; as built, they do.
- **R2** built the carrier archetypes, with the Chopper as the first Landing carrier, in the sim. Its handoff: FILLED IN AT LAUNCH.

## The design
- `factions/gdo/units/supply_chopper.md`: `> Cargo`, `> Unit Transport` and `> ObjectInterfaceState[SupplyChopper]` (its panel).
- `artifacts/designer/changes/0442-supply-chopper-transport.md`: its Verification.
- The unloading rules for every transport (0469, `factions/syndicate/units/subterranean_transport.md > Placeholder stats`): a passenger with no free spot stays aboard, and the pips show to the owner only.

## Goal
1. **The refusals between the two kinds** (0442):
   - the supply commands (Pick Up, Drop Off, Attach) are refused while it carries units;
   - loading units is refused while it carries Supplies.
   
   Each is a gate check with a `Reject` and its text.
2. **The panel,** as `> ObjectInterfaceState[SupplyChopper]` gives it:
   - **F Unload All:** targeted at a point, queueable, available only while it carries units;
   - Pick Up Supplies and Attach to Tower, already built, available only while it carries no units.
   
   Grey each by its probe when the gate would refuse it. There is no Load button.
3. **The client:**
   - **the Chopper's right-click** on an own infantry unit or vehicle is **Load**, "only if the chopper is not carrying Supplies and has room";
   - a unit's right-click on its own Chopper is Enter, if the client doesn't already resolve it through the sim;
   - the targeting for Unload All;
   - the passengers' pips, to the owner only, as the Subterranean Transport's (`grep -rln 'pips' crates/game/src` shows where).
4. **The golden `gdo_airlift`,** made as the other GDO goldens are (`crates/sim/tests/golden/main.rs`): four Peacekeepers board a Chopper, which flies them across the map and unloads them. No other golden may move; if `gdo_supplies` moves, find out why.
5. **Docs:**
   - regenerate the controls doc: `UPDATE_DOCS=1 cargo test -p space_crystals --test game playtest_docs::controls`;
   - add the Chopper's transport to the docs that list transports (`grep -rln 'Subterranean Transport' docs`).
6. **QA screenshots** into `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/m6gs2/`. Look at each yourself:
   - the Chopper landed, with Peacekeepers walking in;
   - the F targeting;
   - the passengers coming out around it, across the map.
   
   The QA mode: `grep -n 'QA mode' ARCHITECTURE.md docs/architecture/*.md`, then read that section only. `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/m6sp/cmds2.json` is a worked example of a QA recipe. Check that `cargo run` starts.

## Tests
- **0442's Verification,** on the real Chopper, where R2's tests don't already cover a point (R2's handoff lists its tests):
  1. four Peacekeepers right-clicked on a hovering Chopper: it lands, they walk in, and it takes off;
  2. a fifth stays outside;
  3. F and a click across the map: it flies there, lands, all four come out around it, and it takes off;
  4. shot down loaded, the passengers die;
  5. an air-only weapon hits it while it lands, but not once it is down.
- **One vehicle:** a Raptor boards alone, and a second unit of any kind is refused.
- **The refusals:** a loaded Chopper refuses Pick Up Supplies, and one carrying Supplies refuses a Load.
- **Client tests** of the panel, its greying and the targeting.

## Don't
- Don't change the carrier archetypes or the landing (R2's, R1's). If they lack something, say so in your report.
- Don't edit anything under `artifacts/`.

## Report
At most 10 lines, then **Handoff** for R3 (the Docked carrier and Paradrop) and CP (the Pilgrim), at most 4 lines: what the panel and the client's targeting give a carrier, and what a new carrier needs to show its pips.
