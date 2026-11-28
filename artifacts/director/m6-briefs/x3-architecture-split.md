# Task X3: Split ARCHITECTURE.md into a short core and per-system docs

## Why
`ARCHITECTURE.md` is 712 KB (6,757 lines, 29 `##` sections), and every implementing agent is told to read it first. That is far more than any one task needs, and what an agent reads stays in its context for the rest of its session. The user approved this split, and two rule changes that go into the new core (below).

## Goal
1. **A short core, `ARCHITECTURE.md`, at most 30 KB,** that every agent reads in full. It holds the rules every task needs, written concisely, with no history (which slice or design change did what):
   - the design is the spec, citations, and the comment rule (below);
   - layout, toolchain and commands;
   - the definition of done, with the test policy (below);
   - determinism: the rules, not their history;
   - time, space and units;
   - code conventions;
   - adding a unit or structure: the checklist;
   - tunables: how to add one;
   - "Bevy 0.20: verify, don't recall";
   - **an index of the system docs:** one line each, saying what it covers and when to read it.
2. **Per-system docs in `docs/architecture/`,** one file per remaining `##` section (Object model, Placement and the build area, Tick pipeline, Combat, Vision, Behaviours, Navigation, Movement and collision, Economy, Production, Commands/events/replays, Sim scenarios and tests, Client architecture, QA mode, Replays and golden hashes, Map editor, Computer players, Adding a faction, Map model).
   - **Move their text verbatim.** Don't rewrite or trim it: a cleanup pass comes later.
   - Split a file above about 60 KB at its `###` headings, into one file per group of subsections, so that an agent can read just the part it needs. Client architecture (about 137 KB), Object model and Sim scenarios and tests are the likely ones.
   - Each file starts with a one-line title and a one-line "Read this when …".
3. **Every reference follows.** There are about 218 references of the form `ARCHITECTURE.md > Section` in about 175 files under `crates/` and `docs/`. Point each at the section's new home: `docs/architecture/<file>.md > Section`, or `ARCHITECTURE.md > Section` if the section stays in the core. Change only those references: no other code or comment changes.
4. **The citations test checks them.** Extend `crates/sim/tests/design_citations/` so that every `ARCHITECTURE.md > Section` and `docs/architecture/<file>.md > Section` reference must name an existing heading of that file. This check is a gate, not a report. The design's own citations stay as they are (their heading check stays a report).

## The two rule changes for the core
- **Comments:**
  - Cite the design section that a piece of code implements by its full path (`path/file.md > Section`).
  - Mark a number or rule the design doesn't give `Placeholder (QNNNN point N)`, or `Placeholder (not in the design: why)`.
  - Don't quote the design beyond a few words, and only where its exact wording matters.
  - Don't record history (which slice, design change or plan decision did what): git and the Director's plan hold it.
  - Say briefly what the code does and why.
- **The test policy** (the definition of done):
  - While you work, run the tests for the code you touch (`cargo test -p <crate> <filter>`).
  - At the end, run each of these once: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and the seed sweep (`cargo test --workspace -- --ignored seed_sweep`, about 6 s).
  - The other ignored suites (ai_games, the soaks) and perf A/B runs are for milestone boundaries. Run them only when your brief says so.

## How
- **Budget:** 250 tool calls. If you reach it, stop at a green, committed point and report what is left.
- Do the move with a script that splits the file at its headings. Read only what you rewrite (the core's sections, roughly the first 160 lines and the last 350), not the whole file.
- `CLAUDE.md` names `ARCHITECTURE.md` as the architecture and coding rules. That stays true, so leave it alone.
- Never edit anything under `artifacts/`.

## Acceptance
- `ARCHITECTURE.md` is at most 30 KB, and every other line of the old file is in exactly one file under `docs/architecture/`. A script check: the old file's lines, minus the core's rewritten sections, all appear in the new files.
- No reference to a moved section is left pointing at `ARCHITECTURE.md`, and the citations test's new gate passes.
- fmt, clippy (`-D warnings`), `cargo test --workspace` and the seed sweep pass. No golden moves: you change comments and docs only.

## Report
At most 10 lines:
- the core's size, and the files in `docs/architecture/` with their sizes;
- how many references changed;
- the citations test's new gate;
- anything in the old file that you couldn't place.
