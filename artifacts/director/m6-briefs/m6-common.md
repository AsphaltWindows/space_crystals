# M6 (the GDO's and the Syndicate's remaining content): common rules for every M6 slice

These rules apply to every M6 slice. Each slice runs in its own git worktree, and the Director reviews and merges it. Your slice brief adds the current status and anything specific to you.

**Start from M5's rules:** read `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/briefs/m5-common.md`. Every rule there applies to M6 too, except where this file says otherwise. That includes:
- the shared files;
- the tests and test programs;
- perf;
- the scratchpad;
- the design rules;
- commits;
- "Done means";
- the report.

## How you work (from 2026-11-28)
This section overrides m5-common and the rest of this file where they differ.
- **One phase, one agent.** Your brief names your phase:
  - **mechanic:** a new shared behaviour in `crates/sim`, with unit tests. Shape it for every unit the design gives it (your brief lists them), not for the first one, and drive it from def data rather than from a unit's kind;
  - **entity:** a unit or structure assembled from existing mechanics: its def, recipe and panel, the client, the QA screenshots, the golden and the docs;
  - sometimes a refactor that comes first.

  Do your phase only. The Director merges it, then starts the next phase with a fresh agent. Slices run one at a time, so master won't move under you unless the Director says so.
- **Read lean.** Read the core `ARCHITECTURE.md` in full. From `docs/architecture/`, read only the docs your phase touches: the core's index says which. In code, find what you need with `grep -n`, then read just those lines (`sed -n 'A,Bp'`). Never read a whole large file or a log.
- **Tests.** While you work, run the tests for the code you touch, such as `cargo test -p sc_sim --lib <module>::` or `cargo test -p sc_sim --test sim <file>::`, and write unit tests for each new module. At the end, run each of these once: fmt, clippy (`-D warnings`), `cargo test --workspace` and the seed sweep (about 6 s). Don't run ai_games, the soaks or perf A/Bs: the Director runs them between milestones.
- **Comments.** Cite the design section by its full path, and mark placeholders `Placeholder (QNNNN point N)` or `Placeholder (not in the design: why)`. Don't quote the design beyond a few words, and don't record history (which slice or design change did what). Don't rewrite existing comments.
- **Handoff.** End your report with **Handoff**, at most 8 lines: what the next phase needs to know, such as the types, functions and files it builds on, and anything left undone.

## Read first (instead of m5-common's "Read first" 2 and 4)
1. **`ARCHITECTURE.md`**, the core, in full, and the docs in `docs/architecture/` that your phase touches.
2. **The M6 spec, `artifacts/director/m6-plan.md`:**
   - **The adoption note at its top,** with "Answered since adoption": each answered question, and which slice builds it.
   - **Your slice in §B.** Implement it as written, unless your brief says otherwise.
   - **§C, the waves and the shared-registries table.** Your "don't touch" list is every other slice's files.
   - **§D, the technical decisions T84–T101.** The Director has accepted them, unless your brief overrides one.
   - **§A,** for what is in scope.
3. **The design questions for M6:** Q0153–Q0169 and Q0174–Q0179, in `artifacts/director/questions/`. `artifacts/director/plan.md`'s Questions table gives each one's status. Placeholders cite them as in M5: `Placeholder (Q0158 point 2)`, or `Placeholder (not in the design: why)`.
4. **The design sections and change entries** your brief names: 0366–0369 for the content, plus each answer's entry.

## M6 in the ruleset's form (T101): your checklist
M5-X1 made every number the sim reads part of one `Ruleset`, and M5-X2 exports it as JSON data files that the game reloads live. Write M6 in that form from the start:
1. **Numbers live in the defs,** as plain data.
2. **New tunables** go in a field at the end of the right tuning struct in `crates/sim/src/tuning/<system>.rs` (`tunables!`: the field, its doc, its built-in value), read as `rules().tuning.<system>.<field>`.
   - A hot loop reads its tunables once, before the loop (`let tuning = &rules().tuning.movement;`): each `rules()` read is an atomic load and a branch.
   - A `const` context reads the struct's `BUILTIN` (`tuning::movement::BUILTIN.give_up_ticks()`).
   - Never add a new `const` the sim reads, except a technical one (an array size). Each new field keeps its design citation or `Placeholder (...)` doc.
3. **New def fields** go at the end of their struct, with their `NONE` default (T84). They serialize with the def, so X2's files carry them; X2 loads a file as a merge patch over the built-in rules, so they need no `#[serde(default)]`.
4. **Structure is automatic.** `Ruleset::structural_diff` (`crates/sim/src/ruleset/structure.rs`) blanks every number and name of the ruleset's serialization and compares the rest. So every new non-number field is structure without more code: a variant, a flag, an optional part, a list (pairs, carriers, stocks, ammo, batteries, mines, services, abilities, …). A footprint's sizes are structure too (`KEPT`).
   - A non-number field the game reads only as something starts (like `Recipe::requires`) may join `FREE` there, with its reason.
   - Test each new structural field: a ruleset that changes it has a `structural_diff`, and one that changes only its numbers has none.
5. **X2's round-trip test** (export, load, same hash) still passes with your kinds and fields in it.
6. **Client feel tunables** (looks, offsets, beams, trails) go in the client's tuning modules (`crates/game/src/tuning/`, `Res<ClientTuning>`), which X2 loads from `client/*.json`.
7. **The playtest docs:**
   - the controls doc's test fails when your panels change: regenerate it with `UPDATE_DOCS=1 cargo test -p space_crystals --test game playtest_docs::controls` and commit it;
   - the placeholders list is regenerated at QA, so don't commit it.

X1's and X2's shapes are summarised below (filled in at launch).

## The computer players
- **The `ai_games` table** (192 rows, about 15 minutes) is checked between milestones, not per slice: don't run it. If your slice changes something every game has, say so in your report.
- **No computer player builds an M6 kind** until A3. Keep it so: the AI crate constructs by kind.

## Goldens
- Name new goldens `gdo_*` and `syndicate_*`.
- The count is in the client test that counts golden games: update it when you add one.

## QA screenshots
Save them in `/tmp/claude-1000/-home-iv-dev-space-crystals/0fe17aed-7223-49e6-819d-e4ea988641e6/scratchpad/m6<slice>/`, for example `m6gk/`, and look at every one yourself before reporting.

## X1's and X2's shapes (filled in at launch)
**M5-X1a, merged** (`8372d899`): the ruleset, read on the built-in rules. Read `ARCHITECTURE.md > Tunables` and `crates/sim/src/ruleset.rs`'s module docs.
- **`Ruleset { defs, bases, factions, tuning }`.** Its `BUILTIN` (generation 0) is made of the kind registry's defs (`ObjectKind::BUILTIN_DEFS`), the bases' (`UnitBase::BUILTIN_DEFS`), `FactionRules::BUILTIN` and `tuning::BUILTIN`.
- **The defs stay `const` literals** in their def modules, listed by `kind_registry!` in `defs.rs`. A new kind is a def module and one registry line at the end.
  - `kind.def()` reads the current rules, and `kind.builtin_def()` the built-in ones.
  - The sim's unit tests can give a kind other numbers (`test_defs`).
- **14 tuning modules** (`air`, `combat`, `construction`, `gathering`, `holding`, `movement`, `orders`, `paths`, `production`, `repair`, `shields`, `spells`, `tunnel`, `vision`).
  - A faction's own rules are its struct in `defs/<faction>.rs` (`GdoRules`, `SyndicateRules`, `CultsRules`, `ColonistsRules`), inside `FactionRules`.
- **The state hash leaves the rules out.** A kind's place in the registry is hashed, so new kinds go last.
- **Hash tags** (`artifacts/director/hash-tags.md`, the Director's registry): new optional state hashes only while present, under a tag of its own, so no golden moves. Your brief names your `Object`, `OrderKind` and `Sim` tags. For `Behaviour`, `Mover`, `Place`, `Held` and `Service`, use the number the registry gives your slice. For any other hash you extend, take its next free number. Either way, list every tag you used in your report.
- **Counts after a merge of master:** a count that both sides changed the same way merges once, with no conflict, and comes out short. After each merge, set every count to the true number: the goldens (the client's playback count in `replay.rs`, `golden/main.rs`'s doc and the architecture doc that lists them), and `Planned::ALL`'s length.

**M5-X1b, merged** (`a8e9f9af`): `DebugCommand::SetRules` replaces a game's rules between two ticks.
- It is refused for a structural change (`Reject::RulesStructural`), a number that isn't finite (`RulesNotFinite`), or a 64th ruleset (`RulesFull`).
- It applies at the next tick's start, with `SimEvent::RulesChanged`. HP and Shields follow new maxima, and saves carry the rules.
- Its tests are `tests/rules.rs`. Don't touch the debug commands, saves or replays.

**M5-X2, merged** (`2b2f275c`): the data files. Read `docs/tuning.md`.
- **What it built:**
  - `--export-data <dir>`;
  - `--tuning <dir>`, which reads the files as merge patches, polled live;
  - F8, which restarts the game on the tuned data;
  - `Ruleset::check` (`Reject::RulesInvalid { field, why }`);
  - the client's feel tunables, in `crates/game/src/tuning/{camera,input,feedback,look}.rs` (`Res<ClientTuning>`).
- **Your kinds' files** follow `kind_registry!`: `defs/<faction>/{units,structures}/<kind>.json`. A new form that shares another kind's def module (as `OperatorDrone` shares `drone_operator.rs`) is one line in the list of `the_files_mirror_the_def_modules_and_cover_every_part` (`crates/game/src/data/tests.rs`).
- **New def fields and tunables** export with no more code. After adding them, run `cargo test -p space_crystals data::` and the `tuning` test program.
- **`Ruleset::check` holds each number to its built-in sign:** positive stays positive, 0 may grow but not go negative, and negative stays negative. A new number that may take either sign, or a new bound (like the SeparationRadius over the Silhouette), goes in `ruleset/check.rs`, with a test.
- **A list in a patch replaces the whole list,** so a patch carries complete list items (a recipe without `supplies` is refused). Write QA patches from an export.
