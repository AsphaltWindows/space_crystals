# Task B2: After the scan change, the GDO's waves stall on playtest.json (a bug, or a design effect)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a bug phase, and your budget is 100 tool calls, a hard limit.**

## The failure
- **The check:** `cargo test --release -p sc_ai --test ai games::ai_games -- --ignored --nocapture` fails its outcome check at `crates/ai/tests/games.rs` near line 426:
  `--map maps/playtest.json --factions gdo,gdo --ai 1:standard,2:passive: standard beats passive within 15 minutes`.
- **The table:** in the last run's table (`/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/b1_ai_games.txt`), the GDO's standard, as player 1, sends 11 waves at a passive player 2 and has no winner at 20:00. Before the scan change it won at 7:41.
  - In the same mirror, the GDO's rush as player 2 sends 11 waves and fails the same way.
  - The opposite seats still win: player 2's standard at 14:38 (12:28 before), player 1's rush at 5:43.
- **The other mirrors** against a passive player mostly take longer than before. The Cults' standard on playtest.json wins at 14:26, close to the 15-minute limit; it won at 9:32 before.
- **What changed:** `d28cd4cc`. Target scans (idle, AttackMove, Patrol, and AttackMove's reach rule) now measure SightRange from the unit's centre to the target's edge (`World::in_sight_range`, `crates/sim/src/orders/world/scan.rs`). Before, they measured from the unit's edge.
  - So units engage about half their width later.
  - Where a unit's Range plus its half width exceeds its SightRange (the Gunner, the Rocketman, the Twister, and the artillery), an idle unit doesn't fire at a visible enemy just beyond its scan, though its weapon reaches. Q0196 asks the Designer about this.
  - Weapon range, and Hold Position's and turrets' scans, are unchanged.
- **The design:** change `artifacts/designer/changes/0467-scans-from-centre-full-tanks-tech-gates.md` part 1, and `systems/controls/behaviors.md > BaseAutoTargeting > Active during Idle`.

## Goal
1. **Reproduce the failing game on its own,** in release, in a temporary test that calls `play` from `crates/ai/tests/games/harness.rs` with the row's map, factions and profiles. Phase B1 did this for another row, in about 5 s a game. Remove the temporary code before you commit.
2. **Find what the waves do,** from the game, not by guessing. For two of player 1's waves:
   - where they are sent, and where their units end up;
   - what each unit is doing there: moving, attacking (what), idle, or dying (to what);
   - at which tick a wave stops making progress.
   
   Then compare with the same game before the scan change: check out the old `crates/sim/src/orders/world/scan.rs`, `orders/reach.rs` and `vision/sight.rs` from `d28cd4cc^` for the run only, then restore them. Name the first unit that behaves differently, and why.
3. **Decide which of these it is:**
   - **(a) A sim bug:** the sim breaks a design rule. Fix it in the sim, citing the rule.
   - **(b) The computer player's orders don't fit the new reach.** The sim follows the design, but the AI's orders now leave its units where they don't engage, for example an Attack Move that ends where its units scan nothing. Fix the computer player's orders, keeping to its fair knowledge (`docs/architecture/computer-players.md`; design change 0402).
   - **(c) A design effect,** such as Q0196's band. Don't fix it. Report the evidence for the Designer: which unit, where, what it saw, what its weapon reached.
   - If it is more than one of these, say how much each one costs the game.
4. **A regression test** for (a) or (b), at the smallest scale that shows it (a sim scenario, or the computer player's own unit tests), not the whole AI game.
5. **Gate:**
   - fmt, clippy (`-D warnings`), `cargo test --workspace` and the seed sweep;
   - then the whole ai_games run, once at the end, in the background. It stops at its first failed check. If it fails, report the row and why. Then check the table for every other row that a check would refuse: a mirror's standard against passive within 15 minutes, standard against standard ending, and the rush's own checks.

## Don't
- Don't change the scan measure (0467), weapon range, the 15-minute limit, or any of the checks in `games.rs`.
- Don't tune the computer players' numbers (wave sizes, timings, profiles) to get a row past its check. A fix under (b) changes what the AI orders, and only where the new reach breaks it.
- Don't change any golden's setup. If a golden moves, say which and why.
- Don't edit anything under `artifacts/`.

## Report
At most 10 lines, then **Handoff**, at most 4 lines:
- what the waves do, and the first unit that behaves differently;
- (a), (b) or (c), with the evidence;
- the fix and the test, if any;
- the ai_games result, and any other row a check would refuse.
