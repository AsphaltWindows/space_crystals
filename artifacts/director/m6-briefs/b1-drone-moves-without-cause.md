# Task B1: In the computer players' games, a Drone moves for no reason the invariants know (a bug)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a bug phase, and your budget is 80 tool calls, a hard limit.**

## The bug
`cargo test --release -p sc_ai --test ai games::ai_games -- --ignored --nocapture` panics:
```
tick 8218: Violation { invariant: "I5/I7", message: "#109 (Drone) moved from [444.54208, 210] to [442.54208, 210] without a goal, a new order or move, a sidestep, a doorway to leave, momentum to brake off, or a unit it overlapped to move off", ...
```
- **The last clean run** was after `2c36a574`, before phase A2: it played all 192 games with no violation. Since then only `d28cd4cc` changed the sim. It made target scans measure from the unit's centre to the target's edge (`World::in_sight_range`, `crates/sim/src/orders/world/scan.rs`), so the games now play differently.
- **The scan change may not be the cause itself:** it may only have led a game into a state that shows an older bug.
- **`ai_games`** (`crates/ai/tests/games.rs`, near line 357) plays its games one after another, and the panic doesn't say which game failed. The harness is `play` in `crates/ai/tests/games/harness.rs`; the panic is near line 196.
- **The invariants** are I5/I7 in `crates/sim/src/testing/invariants.rs`.

## Goal
1. **Find the game:** which map, factions and profiles. Then reproduce it on its own in a quick test, in release.
   - Run the whole matrix at most twice: once to find the game, and once at the end.
   - Run it in the background, and wait for it to finish.
2. **Find the cause:** what moved the Drone at that tick. Which system moved it, why, and what order and state did it have? Look at the ticks before too. Don't guess: print the state.
3. **Decide which is wrong:**
   - **The sim:** if the unit moves when the rules say it shouldn't, fix the cause in the sim, citing the design section the fix follows.
   - **The invariant:** if the design allows the move but the checker doesn't know about it, add the allowance, narrowly, citing the rule.
   - If neither is clear, stop and report what you found.
4. **Write a regression test** with the smallest setup that shows the bug, at unit level or in `crates/sim/tests/`, not in the AI game.
5. **Gate:**
   - fmt, clippy (`-D warnings`), `cargo test --workspace` and the seed sweep;
   - the whole ai_games run, once at the end. Report its rc. Its rows will differ from the old base; that is expected after A2.

## Don't
- Don't change the scans, the computer players or any golden's setup. If a golden moves, say which and why.
- Don't edit anything under `artifacts/`.

## Report
At most 8 lines, then **Handoff**, at most 3 lines:
- the game;
- the cause, and which was wrong, the sim or the invariant;
- the fix and the test;
- the ai_games result.
