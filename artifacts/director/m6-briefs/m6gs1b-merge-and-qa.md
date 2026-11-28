# Task M6-GS1b: Merge M6-GS1 with master, then its QA (phase 2 of GS1)

Read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m6-common.md`, section "How you work" only. Its rules on phases, reading, tests, comments, budget and handoff apply. **Your budget is 200 tool calls.**

## Where things stand
- **M6-GS1** (the Supply Tower, the Supply Chopper's supply runs and schedule, and the GDO's starting Supply of 1) is complete and committed on branch `worktree-agent-a2003c8186c8cfd52` as `1a3cd8f1`. It is based on master `6b9e071f`, and its worktree is `/home/iv/dev/space_crystals/.claude/worktrees/agent-a2003c8186c8cfd52`.
- **What GS1 built** (its report):
  - **The Tower:** the Deployment Center's R builds it, greyed without a Radar Station. It comes with one free Supply Chopper, landed and attached; Q trains more.
  - **The orders:** Pick Up Supplies, Drop Off Supplies, Attach to Tower (`OrderKind` tags 33–35; `Object` tags 27 `chopper` and 28 `schedule`); the right-click goes through `Rule::SupplyRun`.
  - **The schedule:** one chopper flies to a station at a time.
  - **Repair** at the Tower.
  - **The starting Supply:** the GDO starts with 1 Supply (0436), and the Field Engineer costs 1 (0435). GS1 re-blessed 23 goldens for the bank only, checking that with the Supplies left out of the hash they matched, so play was unchanged. It added the golden `gdo_supplies`.
  - **The client:** panels, pointers, markers, the carried-Supplies pips; the controls doc regenerated, `Planned::SupplyTower` removed.
- **Master has moved since** `6b9e071f`:
  - **M6-SB** (`d37b296d`) was already in GS1's base.
  - **M6-ST1** (`a1cd9299`): the carrier. `UnitDef::domain()` and the other accessors; `Place::Carrier` (Place tag 3); `OrderKind::UnloadAll` (tag 25); the passenger pips in `HpBar`; the golden `syndicate_transport`.
  - **M6-GL** (`4e5e39a9`): Propulsion Labs and the Dragonfly; the landing approach (`movement/approach.rs`, `Mover::approach_to`, Mover tag 13; `glide_home` in `sim/landing.rs`); the golden `gdo_dragonflies`; `Planned::ALL` at 27.
  - **M6-SP phase 1** (`971ba38a`): the Orbital Command and the Prism in production. `Mover::unfold` (Mover tag 12) and `Mover::launching()`; the `UnitDef` fields `satellite`, `one_way`, `unfold_ticks` and `thrust`; `services::manned` skips a satellite; `Planned::ALL` at 25.
  - The Director's and the Designer's artifact commits.

## Goal
1. **In GS1's worktree,** run `git merge master -m "<message with the attribution lines>"`, and resolve every conflict so that **both sides' features stay whole**. A dry run shows 32 conflicted files. Most are places where both sides added an arm, a field or a list entry:
   - the `Aimed` literals (`orders/resolution.rs`, `sim/gate/targets.rs`, the client's `command_ui/resolution/tests.rs`) need both `carrier` and `supply_tower`;
   - the `OrderKind` arms in `orders.rs` and `orders/hash.rs` need both ST1's `UnloadAll` (tag 25) and GS1's three (33–35);
   - `HpBar` and `combat_effects` need both ST1's passenger pips and GS1's supply pips;
   - `Planned::ALL`: master's 25 entries less `SupplyTower`, so 24;
   - **the golden count:** master's 35 plus `gdo_supplies` makes 36, in `crates/game/tests/replay.rs`, `crates/sim/tests/golden/main.rs`'s doc and `ARCHITECTURE.md`;
   - `controls.md`: don't merge it by hand. Take either side, then regenerate it with `UPDATE_DOCS=1 cargo test -p space_crystals --test game playtest_docs::controls`;
   - the rest (defs, gate, commands, tether, testing, the client's command files): keep both sides' additions.

   - **Counts merge silently.** When both sides change a count to the same number, git takes it without a conflict, even if the list now holds a different number of entries. This happened in SP's merge with `Planned::ALL`. Check every count you touch against its list: `Planned::ALL`, the golden count, `OrderKind` and hash-tag lists.

   Commit the merge with `git commit --cleanup=strip --no-edit` (or `-F` with a clean message), so that no "# Conflicts" lines stay in it.
2. **Goldens.** The starting Supply changes the bank of every golden with a GDO player, so master's newer GDO goldens move too (`gdo_dragonflies`, and `syndicate_transport`, whose opponent is the GDO). Re-bless only those that fail. Prove each re-bless is bank-only, as GS1 did: with the players' Supplies left out of the hash, it matches master's blessed hash. No other golden may move.
3. **`Place::Carrier`:** add its arms wherever the compiler asks. GS1's supply-run files read neither `Place` nor a base's domain.
4. **QA screenshots,** into `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/m6gs1/`. Look at each one yourself:
   - the Tower's panel;
   - a chopper landing on a Supply station;
   - the GDO's Supplies rising after a run.

   The QA mode is in `ARCHITECTURE.md > QA mode` (find it with `grep -n '^## QA mode' ARCHITECTURE.md`, then read that section only). Then check that `cargo run` starts the game.
5. **The final gate,** once each: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and the seed sweep (`cargo test --workspace -- --ignored seed_sweep`). Don't run ai_games or perf.

## Don't
- Don't change GS1's or master's behaviour, and don't add features: this phase is the merge and the QA.
- Don't edit anything under `artifacts/`.
- If the permission checker refuses a command as too complex, split it into simple commands. Don't wrap it in a script to get past the check.

## Report
At most 10 lines, then **Handoff**:
- the merge: its commit, and any conflict that wasn't a plain union of both sides;
- the goldens you re-blessed, and the bank-only proof for each;
- the screenshots;
- the gate's results.
