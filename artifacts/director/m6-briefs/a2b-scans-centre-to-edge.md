# Task A2b: Target scans measure from the unit's centre to the target's edge (0467 part 1, finished on A2's branch)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a mechanic phase, and your budget is 120 tool calls, a hard limit.**

## Where things stand
- **A2's commit** is `dfee14f7`, on branch `worktree-agent-a08d89bc88677e760`, in the worktree `/home/iv/dev/space_crystals/.claude/worktrees/agent-a08d89bc88677e760`. Work there: `cd` into it first. Its build is warm.
- **What A2 did:** every target scan now goes through one function, `World::in_sight_range` (`crates/sim/src/orders/world/scan.rs`). It is called by:
  - `Scan::Sight` in `World::within` and `World::pick`;
  - the AttackMove reach rule (`reach::finishes` in `crates/sim/src/orders/reach.rs`).
  
  It measured the scan as the unit's own eye: `Vision::eye_sees_box` asks whether the eye at the unit's centre sees a tile the target's box covers. Vision samples tile centres.
- **That reading was the Director's, and it is stricter than the design asks.** It cut the reach by up to half a tile more than 0467 expects, and broke:
  - 21 tests, which A2 marked ignored. Find them with `grep -rn 'to re-derive for the centre measure\|the Director re-runs it\|awaiting re-derivation' crates`.
  - 7 goldens whose own expectations fail (`crates/sim/tests/golden/main.rs`);
  - the client's `replay::golden_games_play_back_in_the_client`;
  - the standard-beats-passive game in `crates/ai/tests/games.rs`.

  A2 also re-blessed 17 goldens.

## The design
- `systems/controls/behaviors.md > BaseAutoTargeting > Active during Idle`.
- `artifacts/designer/changes/0467-scans-from-centre-full-tanks-tech-gates.md`: part 1, and Verification 1 and 2.
  - Verification 2: "The distance at which a lone unit picks a target equals its SightRange from its centre to the target's edge or centre, as the Director chooses consistently."
  - Its Implementation impact: "Every unit now engages about half its width later than before."
- **The Director's choice: from the unit's centre to the target's edge.**
  - A scan reaches a target when the distance from the scanning unit's centre to the nearest point of the target's shape is at most the SightRange of the unit's type. That is the unbuffed SightRange, with the ElevationModifier as the old `gap_within` read it: `elevation::sight_range(kind.def().sight_range, source, view.elevation)`.
  - Weapon range (Range and MinRange) stays edge to edge.

## Goal
1. **Merge master** into A2's branch, with `git merge master -m "..."`. No conflicts are expected.
2. **`World::in_sight_range`:** measure from the centre to the target's edge, as above, in place of `Vision::eye_sees_box`.
   - Find the shape's helper for the distance from a point: `grep -n 'fn gap\|fn distance' crates/sim/src/` in the shape module.
   - Remove `eye_sees_box` if nothing else uses it. Keep `sight.rs::sees_tile`, which vision's `stamp` shares.
   - A2 made the unit's half in `pick`'s search radius at least half a tile. Make the radius exactly what the new measure needs.
3. **Un-ignore every test A2 ignored,** the 21 and the three listed above, and run them.
   - **A test that still fails because it was built on the edge-to-edge reach:** re-derive it for the new measure. Move the units or change the distances so that it tests what it was written to test. Don't weaken what it asserts.
   - **A golden whose own expectations fail:** change its setup as little as possible, so that it plays what it was written to play, then bless it. Say for each what you changed and why.
   - **The AI game:** if it still fails, find out why (which units idle, and where) and report it. Don't tune the AI.
4. **Tests:** restate A2's tests in `orders/tests/scan.rs` and `sight.rs` for the new measure.
   - **Verification 2:** a lone unit picks a target exactly when its centre is SightRange from the target's edge, swept 1 su at a time.
   - **Verification 1** (`null_chamber/buff.rs`): a buffed and an unbuffed Guard engage at the same gap.
5. **Goldens:** re-bless only those that fail, in your commit. For two of them, find the first tick and the unit whose scan changed, and name them in your report.
6. **Docs:** state the new measure in the paragraphs A2 changed in `docs/architecture/behaviours.md` and `vision.md`.

## Don't
- Don't change weapon range, MinRange or vision itself.
- Don't leave any of A2's ignored tests ignored. If one can't be re-derived within your budget, stop and report it.
- Don't edit anything under `artifacts/`.

## Report
At most 10 lines, then **Handoff**, at most 4 lines:
- the measure;
- the tests re-derived, and how;
- the goldens re-blessed, and those whose setup changed;
- the AI game;
- any design effect that remains, such as units in a second row that stay idle with a visible enemy in weapon range.
