# Task A3: The Recruits follow the gathering job rules (0484): tests and citations only

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a small phase of tests and citations, and your budget is 100 tool calls.**

## Where things stand
The mechanics audit found that the Recruits already follow all seven gathering job rules. They share one code path with the Drones and the Agents: one Gather behaviour, keyed by `UnitDef::worker`. The Recruit's parameters are data in `crates/sim/src/defs/cults/units/recruit.rs` (about lines 64–82).

**So no behaviour changes in this phase.** If a test you write fails, stop and report it; don't change the rules.

Where each rule lives:
1. **The last load of a patch:** `remaining.min(capacity)` in `crates/sim/src/economy/gathering.rs`, near line 283.
2. **Interrupted mining:** the frames live in the order's state (`GatherPhase::Mining`), so a Recruit ordered away collects nothing.
3. **Other orders keep the load:** `Worker::load` is written only at delivery, at mining and when a Recruit is trained.
4. **Who sees the load:**
   - the model's load marker shows to anyone who sees the unit (`crates/game/src/object_render/load.rs`);
   - "Carrying" in the HUD (`crates/game/src/hud/info_panel.rs`, near 245–251) and the drop-off indicator lines (`crates/game/src/command_ui/indicators.rs`, near 254–263) are owner-only.
5. **Gather while carrying a load:** `crates/sim/src/orders/gather/patch.rs`, near 51.
6. **The Drop Off order:**
   - the gate: `crates/sim/src/sim/workers.rs`, near 82 and 112;
   - the behaviour: `crates/sim/src/orders/gather/deliver.rs`, near 61 and 201;
   - the client's right-click, resolved per unit: `crates/game/src/command_ui/resolution.rs`, near 304–313.
7. **When a Gather ends:** `crates/sim/src/orders/gather/patch.rs`, near 142 and 188.

## The design
- `systems/mechanics/gathering.md > Job rules` (its seven subsections) and `> Archetype: Worker round trip`.
- `factions/cults/units/recruit.md > Gathering` and `> Right-Click Resolution:`.
- `artifacts/designer/changes/0484-*.md`, its Verification.

## Goal
1. **Tests,** in `crates/sim/tests/recruit_gathering/rules.rs`; the helpers are in `main.rs` there. First see which of these exist (`grep -n '^fn ' crates/sim/tests/recruit_gathering/*.rs`), then add the missing ones:
   1. 0484 (1): a Recruit ordered away mid-mining carries nothing.
   2. 0484 (2): a loaded Recruit given a Move keeps its 2 crystals, and delivers them on its next trip.
   3. 0484 (3): a right-click on an own Storage delivers the load, then the Recruit idles, or runs its next queued order. `a_drop_off_order_delivers_the_load_at_its_storage` may cover part of this.
   4. Gather while carrying: a loaded Recruit ordered to Gather delivers first.
   5. The last load of a patch: a Recruit takes only what is left.
   6. An empty Recruit's right-click on an own completed, undamaged Storage is a plain Move. Find where the right-click is resolved, and the existing tests of a Recruit's right-click, with `grep -rln 'Recruit' crates/*/tests | xargs grep -ln -i 'right'`.
   7. A mixed group right-clicked on a Storage: the loaded Recruits deliver, the empty ones move.

   Don't test an empty Recruit's right-click on a damaged or unfinished Storage. That case is open as question Q0194.
2. **0484 (4), who sees the load:** a client test that an opponent sees the load marker but not the amount, if no test covers it (`grep -rn -i 'load' crates/game/tests | grep -i 'owner\|opponent\|enemy'`).
3. **Citations.** The job rules moved from the Drone's doc to `systems/mechanics/gathering.md > Job rules`; `drone.md`'s `### Job rules` now points there.
   - Re-point the 23 citations of the seven moved sections, in 11 files. Find them with `grep -rnE 'drone\.md > (Last load of a patch|Interrupted mining|Other orders keep the load|Who sees the load|Gather while carrying a load|Drop Off order|When a Gather ends)' crates docs`. Write `systems/mechanics/gathering.md > <section>`, the subsection's own name.
   - The module docs of `crates/sim/src/orders/gather.rs` (lines 1–8), `crates/sim/src/economy/gathering.rs` (lines 1–3) and `crates/sim/src/defs/units/worker.rs` cite the Drone's doc for the whole job. Cite `systems/mechanics/gathering.md` for the shared rules there. The Drone's own sections stay in its doc, so leave their citations as they are: Gathering, Patch Occupancy, Gathering Drones pass through each other, and Assigned Drop-off.
   - **The relative-citation rule:** a module whose doc cites exactly one file may use relative `` `> Section` `` references. If you add a second file to such a module's doc, make its relative references full, and run `cargo test -p sc_sim --test sim design_citations`.
   - Don't rewrite any other comment.
4. **Goldens:** none may move.

## Don't
- Don't change any gathering behaviour or right-click resolution.
- Don't touch the Drones' or the Agents' parameters.
- Don't edit anything under `artifacts/`.

## Report
At most 8 lines, then **Handoff**, at most 4 lines: the tests added, the citations re-pointed, and anything that failed.
