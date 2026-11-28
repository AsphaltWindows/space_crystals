# M5 (the Cults, and the first playtest): common rules for every M5 slice

These rules apply to every M5 slice. Each slice runs in its own git worktree, and the Director reviews and merges it. Your slice brief adds the current status and anything specific to you.

## Read first
1. **`ARCHITECTURE.md`**, all of it. It is binding: object model, tick pipeline, economy, placement, command UI, client architecture (viewer events), code conventions, "Adding a faction", "Adding a unit or structure", Definition of done, Perf, and "Replays and golden hashes".
2. **The M5 spec, `artifacts/director/m5-plan.md`:**
   - **The Director's adoption note** at its top:
     - the user's decisions: the playtest is **hot-seat** (F2), started from the **command line**, with no AI and no start screen;
     - T66 accepted;
     - X1 last;
     - **O3** added for 0304;
     - Q0092–Q0108 filed.
   - **Your slice in §3.3.** Implement it as written, unless your brief says otherwise.
   - **§3.2:** the waves and the ownership table. Your "don't touch" list is every other slice's files.
   - **§4:** the technical decisions T55–T68. The Director has accepted them as written, unless your brief overrides one.
   - **§1.3:** what M2–M4 already built. Reuse it; don't rebuild it.
3. **Earlier conventions,** where your slice extends M2–M4 code:
   - `artifacts/director/m4-plan.md` §4 (T29–T54). **T54, code organization, binds every slice;**
   - `m3-plan.md` §4 (T14–T28);
   - `m2-plan.md` §4 (T1–T13).
4. **The design questions for M5:** `artifacts/director/questions/Q0092-*.md` to `Q0108-*.md`, all open and none blocking. `artifacts/director/plan.md`'s Questions table gives each one's status.
   - Until an answer arrives, build the question's proposal as the current behaviour.
   - Mark it with a doc comment that cites the question and the point, in the form **`Placeholder (Q0097 point 2)`**. Use the same form, `Placeholder (not in the design: why)`, for numbers the design doesn't give.
   - When an answer arrives as a design change entry (`Answers: Q00NN`), the Director tells running slices, and briefs not yet launched get it.
5. **The design sections your slice lists,** in `artifacts/designer/design/` (read-only), and every change entry your brief names, in `artifacts/designer/changes/`. **Renames since the plan was written:**
   - the Soldier is the **Militant** (0302, `factions/cults/units/militant.md`);
   - the Acolyte is the **Disciple** (0305, `factions/cults/units/disciple.md`).

   Use the new names everywhere.
6. **The design principle (0223), `systems/controls/behaviors.md > Core principle: predictable, not optimal`.**
   - No logic that optimises behaviour for the situation.
   - Where a case is open, build the simplest predictable rule and report it.
   - No indefinite waits (M3-G3).
7. **Design paths (0282).** The design docs are one file per object, for example `factions/cults/units/recruit.md`, `factions/cults/structures/armory.md`, `factions/cults/economy.md`.
   - **Cite with full paths** (`path/file.md > Section`). A relative `> Section` is allowed only inside a module whose doc names exactly one design file, and a bare design file name is never allowed.
   - `cargo test -p sc_sim --test sim design_citations` enforces this and fails on any cited path that doesn't exist. Run it before you report.
8. **Code organization (the user, through 0282; T54).** "keep the code organized and modular ... per-unit and per-structure definitions living in their own modules, data and tunables kept apart from logic, and shared mechanics implemented once and reused."
   - A new unit or structure gets its own def module, its own client module and one registry line, by `ARCHITECTURE.md > Adding a unit or structure`.
   - Its numbers live in its def, as plain data with no logic inside.
   - A new behaviour number is a named constant in the right `tuning/<system>.rs` module, with a design citation or `Placeholder (not in the design: why)`.
   - A mechanic used twice is built once. The precedents:
     - timers use `Progress`;
     - toggles use the toggle rule;
     - area damage, the deploy model, held units and exit doorways are shared;
     - the client reads sim events only through `viewer_events`.

## Rules
- **Shared files.** These are shared by parallel slices:
  - `sim.rs::step`, `command.rs`, `sim/gate.rs`, `testing.rs` (`every_command`, `dangling_references`), `object.rs`, `events.rs`, `state_hash`;
  - the kind registries (sim and client), `viewer_events.rs`'s table, and `ARCHITECTURE.md`.

  Keep your edits there small and additive. New kinds go last in `ObjectKind`; on a one-line registry conflict, keep both in landing order. T28 says to split a file near 500 lines before adding to it.
- **Goldens.** There are 20 on master today, and 21 once N6's `cults_vehicles` lands. `crates/sim/tests/golden/main.rs`'s docs say how to add and bless one.
  - Hash a new field only while it is set (non-default), so goldens stay stable unless behaviour changes, and give new fields `#[serde(default)]`.
  - If your slice changes hashed state in an existing golden, re-bless once, after your final `git merge master`, and explain every change. If master moves and the goldens then fail, re-bless again and explain.
- **Tests:**
  - never depend on machine speed: no wall-clock time and no sleeps, and loop until a tick is reached;
  - give every test its own temp directory;
  - drive tests through the gate (`TestSim`, `testing::build_order`, `InjectedInput`) rather than poking state, except where a test is about the state itself;
  - a new `SimEvent` variant needs its row in `viewer_events`' table test (it fails without one).
  - **The test programs (slice M5-T).** The integration tests are modules of three test programs:
    - `sim` (`crates/sim/tests/sim.rs`);
    - `perf` (`crates/sim/tests/perf.rs`, the `*_perf` timings);
    - `game` (`crates/game/tests/game.rs`).

    A new test file needs its `mod` line in its program's root, and the guard test `test_programs` names any file
    left out, with the line to add. Run one file's tests by filter: `cargo test -p sc_sim --test sim holding::`.
    The tests share one process, so none may change the current folder, set an environment variable, exit the
    process or keep global state.
  - `-p sc_sim` and `--workspace` build the sim with different features, so mixing them compiles it twice. Pick one
    for your edit loop.
- **Perf.** If your slice touches movement, nav, combat, vision or the economy:
  - run the relevant ignored release perf tests:
    `cargo test -p sc_sim --release --test perf <name> -- --ignored --nocapture --test-threads=1`;
  - **side A of an A/B is master's prebuilt perf binary.** The Director builds one after each merge, at
    `/home/iv/dev/space_crystals/target/perf/<short sha>`, named by the last master commit that changed code:
    `git log -1 --format=%h master -- crates Cargo.toml Cargo.lock`.
    - Run it as `<file> <name> --ignored --nocapture --test-threads=1`.
    - Side B is your own `cargo test -p sc_sim --release --test perf --no-run` executable, so both sides have the same
      features.
    - If there is no binary for your master, build master yourself;
  - give `uptime`'s load average with the numbers;
  - a perf number taken while other agents build is unreliable, so call a regression only from interleaved A/B runs against master.
- **The scratchpad is a size-limited tmpfs** (about 12.6 GB, shared with every agent). Keep only screenshots, logs and
  small scripts there. Build output, perf binaries and copies of `target/` go in your worktree's `target/`.
- **QA screenshots.** Save them in `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/m5<slice>/`, for example `m5a/`, and look at every one yourself before reporting.
- **The design.** Don't change it. If it is unclear or contradictory and no question covers it, build the most literal reading, mark it in a doc comment, and report it.
- **Dependencies.** No new ones.
- **Commits.** Every commit carries the attribution lines given in your prompt, merge commits included. Always pass `-m` with them to `git merge`: a merge left with git's default message can't be fixed later, since history rewrites are refused. After a merge with conflicts, commit with `git commit --cleanup=strip --no-edit`, or `-F` a clean message: a plain `--no-edit` keeps git's "# Conflicts" lines after the attribution. A merge commit holds only the merge and its conflict resolutions: commit new work, a fix the merge calls for included, separately after it. Don't mark commits "(wip)"; make each one a finished step.

## Done means
1. Commit.
2. Run `git merge master` and resolve any conflicts.
3. Re-bless the goldens if needed.
4. Re-run all checks:
   - `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check` and `cargo test --workspace` are clean;
   - `cargo test -p sc_sim --test sim design_citations` passes;
   - the seed sweep passes: `cargo test --workspace -- --ignored seed_sweep` (about 6 s on the workspace build). It fuzzes 256 seeds of random commands against the sim's invariants. New kinds or rules reshuffle the streams, so a failing seed usually points to a real bug: fix it if it is your slice's, or report it with its seed, invariant and kind;
   - `cargo run` works.

Also:
- doc comments cite the design sections, change entries and questions;
- `ARCHITECTURE.md` is updated for every new convention, as §3.2 lists;
- the slice's Verification and acceptance points each have a test, or a stated reason why not.

## Report (15 lines at most, unless your brief says otherwise)
- branch and commits
- what was built, plus any deviation from the spec and why
- tests added, and how each acceptance point is covered
- golden status, with the reason for each change
- perf numbers with the load average, if your slice has any
- screenshot paths
- open ambiguities, design or technical
