# Task A2c: The four goldens left ignored by the new scan measure (0467 part 1, finished)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **This is a test phase, and your budget is 80 tool calls, a hard limit.**

## Where things stand
- **Phases A2 and A2b** changed the target scans. Every scan now measures from the unit's centre to the target's edge: `World::in_sight_range` in `crates/sim/src/orders/world/scan.rs`.
- **The work is on branch `worktree-agent-a08d89bc88677e760`,** at `41f15072`, in the worktree `/home/iv/dev/space_crystals/.claude/worktrees/agent-a08d89bc88677e760`. Work there: `cd` into it first. Its build is warm.
- **Four goldens are still ignored** in `crates/sim/tests/golden/main.rs`, because their own expectations fail. Their setups are in `factory.rs`, `air.rs`, `syndicate.rs` and `shroud.rs` there.
  - `golden_gdo_vehicles`: one of player 2's units survives to tick 1600. In a longer game, #12 hits #3 at tick 1653.
  - `golden_cults_air`: two Sparrows survive.
  - `golden_syndicate_vs_gdo`: only the second Tunnel is lost.
  - `golden_cults_shroud`: at tick 265, #6 is hit by #11.
- **So the client's `golden_games_play_back_in_the_client`** (`crates/game/tests/replay.rs`) is ignored too.
- **What the new measure changes:** a unit engages about half its width later than before. Where its Range equals its SightRange (the Gunner, the Rocketman, the Twister), an idle unit no longer fires at a visible enemy that is within its weapon range but beyond SightRange from its centre. A lone unit also waits until its own eye sees the target, and vision tests tile centres.

## Goal
1. **For each of the four, first find why its expectation fails:** which unit no longer engages, or engages later, where, and by which of the effects above (or another one). One or two sentences each, in your report. Find it from the game, for example by stepping the scenario in a test and printing the unit's position, its target and the gap at the tick where it differs. Don't guess.
2. **Then change its setup as little as possible,** so that it plays what it was written to play: move a unit, or change a distance or a tick in its own expectations, where its doc says the expectation is a recorded play. Bless it. Say what you changed and why.
3. **Un-ignore all four, and the client's playback test,** and run them.
4. **Gate:** `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` and the seed sweep. Then commit on the branch.

## Don't
- Don't change the sim or the client: this phase changes golden setups and their expectations only. If a golden can't play what it was written to play without a sim change, stop and report it.
- Don't edit anything under `artifacts/`.

## Report
At most 8 lines, then **Handoff**, at most 3 lines:
- for each golden, the cause and the change;
- the gate's results;
- the commit.
