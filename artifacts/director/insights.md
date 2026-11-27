# Director Insights

## Decisions with the user (2026-11-23, first session)
- **Old prototype deleted** (`artifacts/developer/`, 67K LOC). Nothing salvaged as code. Recoverable from git tag `pipeline-archive`. Survey verdict: its failures were architectural (logic tied to frame time, `Transform` as source of truth, tiles as entities, components used as events, untestable time), not caused by Rust/Bevy.
- **Stack: Rust + Bevy 0.20.1** (was 0.17 in the old docs). Considered Godot+C#, TypeScript/browser, Unity. Chose Rust/Bevy because the user will NOT hand-tune visuals (so an editor adds little), wants max agent QA-ability and strict mistake-catching. Godot's editor value = human visual authoring only; agents tune visuals via file edit → render → screenshot, which needs no editor.
- **Bevy upgrade policy**: stay at most one release behind; upgrade at milestone boundaries soon after each release. Only `crates/game` depends on Bevy, so upgrades stay contained.
- **Rust toolchain pinned** per repo in `rust-toolchain.toml` (1.100.0); the user's global default is untouched. Bump together with Bevy.
- **Edition 2024**, Cargo workspace at the repo root: `crates/sim` (package `sc_sim`) and `crates/game` (package `space_crystals`, the binary).
- **Game logic is a plain Rust library with no Bevy dependency** (`sc_sim`). Bevy only draws, handles input and the HUD. Engine is swappable.
- **Opponents: undecided, kept open.** The sim is deterministic (same inputs → same game) for replays, reproducible bugs and possible lockstep later. During development the user controls all sides via debug keys. AI vs online decided later.
- **Art direction (user, 2026-11-23): 2.5D pixel-art sprites like StarCraft 1 / AoE2.** Sent to the Designer as Q0001 (projection, facings, building orientations). Until then: placeholder shapes, straight-down camera, but client conventions are SC1-style (square grid; y-sorted draw order; sprites anchored at ground position).
- **Generated art with user sign-off**: per-asset review page (claude.ai artifact with a db capability: approve/reject/comment, stills + animations at game speed + in-game context screenshots); agents read verdicts back. Tooling (PixelLab MCP for directional pixel sprites; or Blender+blender-mcp+3D generator; image-model API for textures/icons) is chosen at the art milestone — user must create accounts/keys.
- **Max agent QA-ability is a core requirement**: headless deterministic scenario tests, an in-game QA driver (scripted inputs, fast-forward, state/HUD dumps, screenshots), screenshot checks, replays of the user's sessions.

- **Map editor**: built first as a dev tool (M0-C), then designed by the Designer with the user (`core/map_editor.md`: map types, creation settings, generated symmetry, per-faction Starting Layouts, custom presets). It is now a spec'd feature, so editor behaviour questions go to the Designer.

- **Milestone order (user, 2026-11-24):**
  1. finish M3;
  2. M4 Syndicate;
  3. M5 Cults.

  The user wrote "then M4 Cults", read as M5. The user keeps feeding designs to the Designer, so expect incremental change entries mid-milestone.
- **One playtest, when all four factions are done:** GDO, Colonists, Syndicate and Cults. Until then QA is by the agents: walkthroughs, goldens, soaks and screenshots.
- **The playtest's form (user, 2026-11-25):** hot-seat (one person plays both sides, switching with F2), started from the command line. No AI opponent and no start screen for it; either would be a later decision.
- **The art pipeline waits** until the gameplay design is mostly final.

- **Design principle (user, 0223): default behaviour is predictable, not optimal.**
  - No algorithm that optimises behaviour for the situation. Managing bottlenecks is the player's job.
  - Briefs must not ask agents for clever crowd or flow logic. When a design leaves a case open, propose the simplest predictable rule.
  - M3-G2's indefinite wait was removed for this reason.

## Codebase
- Layout and rules: see `ARCHITECTURE.md` at the repo root (the brief every sub-agent reads first).
- `cargo run` = game; `cargo test` = both crates (both are default-members).

## Environment facts
- Fedora 43, X11 session (`DISPLAY=:0.0`), Intel Arc 130V/140V iGPU (laptop), Vulkan ICDs: intel (ANV) + lavapipe (software). 8 cores, 30 GB RAM.
- Missing dev headers: alsa-lib-devel, systemd-devel (libudev), libX11-devel, wayland-devel. → Bevy built with `default-features = false`, no `audio`, no `bevy_gilrs`, no `wayland`. x11 works via dlopen. Audio later needs `sudo dnf install alsa-lib-devel` (ask user).
- Installed: node 22/npm, python3.14 + Pillow, ffmpeg, ImageMagick, vulkaninfo, glxinfo. Not installed: Blender, uv, Xvfb, Godot, dotnet.
- **The scratchpad is on a `/tmp` tmpfs with a 12.6 GB quota** (found 2026-11-26, when M5-N7 hit "Disk quota exceeded" at about 13 GB).
  - Delete merged slices' scratchpad folders after each merge. Keep the briefs, the plan edits and `tools/`.
  - Perf binaries go in the repo's `target/perf/`, never the scratchpad. `m5-common.md` tells agents so.
  - Remove with guarded paths (`"${S:?}"/...`). The safety check refuses an `rm` whose variable could expand to empty.
- Bevy 0.20.1 feature bundles: `default_app`, `default_platform`, `2d_bevy_render`, `ui_bevy_render`, `dev`. `mouse`/`keyboard` input features come transitively via `bevy_winit` → `bevy_window`.
- Bevy release history: 0.17 Sep 2025, 0.18 Jan 2026, 0.19 Jun 2026, 0.20 Oct 2026 (0.20.1 Oct 28). Agents' training knowledge ends ~0.18/0.19 → always verify Bevy APIs against `~/.cargo/registry/src/*/bevy_*-0.20.1/` source.

## Lessons carried over from the old prototype (technical)
- One update must equal one sim tick in tests; never let tests depend on wall-clock time (prototype's fixed-step systems never ran in tests).
- Cache mesh/material handles; never create assets per indicator/projectile (prototype OOM).
- Repath attempts must be capped; A* closed-set/tie-breaking must be correct (prototype exponential open set; zigzag paths from equal-cost parent overwrites — keep a best-cost map).
- Arrival radius must exceed one tick of travel or units overshoot/oscillate.
- One movement pipeline; commands strictly ordered before movement (two pipelines caused races).
- Damage must accumulate (list of hits), never be a single overwritable slot (second hit in the same frame was lost).
- Fog rendering: one texture updated on change, not per-tile materials.
- Occupancy must be domain-aware (underground expansions don't block surface units).
- Bevy: keep systems small (16-param limit hurt the prototype's UI code); use Resources, not `Local`, for state tests must set; direct `ButtonInput` presses get cleared before game systems run headless — inject input through our own input layer.
- Rust: `f32::abs(x)` when inference fails on `(x - 1.0).abs()`; don't have both `foo.rs` and `foo/mod.rs`.

## Working style agreed with the user
- **This session is for discussion, planning, triage and short checks.** All hands-on work (code, docs, builds, debugging, reviews) goes to background sub-agents with self-contained briefs that return short reports (≤15–20 lines). Never pull build logs or whole files into this session; grep and tail instead.
- **Models**: sub-agents run on Opus by default. Use Fable for the hardest pieces (M1 pathfinding/movement/collision, stubborn bugs); it costs ~2× Opus per token and runs slower. The user may override.
- **The Opus movement trial** (user, 2026-11-26): H1 (gliders) runs on Opus to test whether movement slices need Fable.
  - Opus does the routine work (scenario scripts, QA runs, screenshots) equally well. Any difference is in designing movement rules and diagnosing stuck units.
  - Until H1 there was no side-by-side comparison. In M1, the one recorded comparison, a Fable slice used about 1.5x the tokens of a similar-length Opus slice, at 2x the price.
  - The rule for the trial: a movement problem that resists two serious attempts becomes an `#[ignore]`d test and a precise report, and only that goes to Fable.
  - If H1 goes well, V1 and V2 run on Opus too.
  - **The result (2026-11-26): passed.**
    - H1 met every acceptance point on Opus, in 617 tool uses over 2 h 14 min.
    - One movement problem took a second attempt (six Sparrows converging; CIRCLE_PULL from 1 to 0.5), and nothing went to Fable.
    - Review found only design questions (Q0148).
    - **V1 and V2 run on Opus.** Fable stays for a problem that resists two serious attempts.
- **Ultracode/workflows: off for now.** Suggest a workflow per task when it fits (milestone review with verification, batches of independent units or sprites); the user opts in with "use a workflow".
- **Parallel coding agents use `isolation: "worktree"`**. They seed `target/` with a reflink copy (btrfs; a cold Bevy build takes ~7 min), commit on their own branch, and the Director reviews and merges.
- **Push frequently** (user, 2026-11-25): "Don't forget to commit and push frequently!"
  - Push `master` to `origin` (GitHub, `AsphaltWindows/space_crystals`) after each merge and each bookkeeping commit.
  - Scan the unpushed diff for secret-like strings first.
  - Never push agent worktree branches.
  - Never push the Designer's uncommitted work. A push only sends commits, and the Designer commits its own.
- **Code organization** (user, through 0282, 2026-11-25), quoted: "keep the code **organized and modular** ... Once testing starts we want to experiment and make changes rapidly and accurately ... per-unit and per-structure definitions living in their own modules, data and tunables kept apart from logic, and shared mechanics (deploy model, toggles, AoE) implemented once and reused." New kinds go in their own modules. Design citations use the per-object doc paths (`factions/<f>/units/<unit>.md`).
- **Tuning data** (user decision, 2026-11-25: "Modules now, data files at playtest").
  - **M4-R, now:** one Rust module per unit and per structure, with tunables in their own modules.
  - **At playtest,** after the four factions, hot-reloadable data files come in so numbers can be tuned live.
  - Keep defs as plain, serde-ready data structs, so that step stays cheap.
- **Economy testing is the user's** (user, 2026-11-26): "So much of it has to do with build orders and optimizations and that would be more effectively tested by me."
  - No more economy comparisons or oracle reruns. The oracle stays as an on-demand tool: its tests are ignored, and its pins aren't maintained (M5-P3).
  - The pairings tests still check that every faction's workers gather, and the goldens still catch sim changes.
  - Q0146 sent the last figures to the Designer and asked to close the Economy check.
- **Computer opponents** (user, 2026-11-26): the user wants basic AIs for the next playtest, since timings are hard to test by switching sides. They are the crate `crates/ai` (`sc_ai`). 0402 (Q0147) made them a playtest tool with fair knowledge. A1 built the framework and the GDO (`168dadc0`); A2a–c added the other factions, and A2d (`0111cc8e`) the rush profile and three economies that spend their income.
  - **The reference table since A2d** is `scratchpad/a2d_ai_games.txt`: 192 rows, 13–18 minutes. Compare every later slice's `ai_games` with it, using `strip()`, not S7's 144-row table. A slice that changes the sim and moves rows becomes the new reference once merged.
  - **Known weak spots, left for A3 or the playtest:** the Syndicate's and the Colonists' standard can't beat the GDO's passive in 20 minutes, and the Cults lose to the GDO's standard.
  - **Adding a faction** is one module behind `FactionAi` plus its `HOOK`. Shared-layer changes must keep the GDO's `ai_games` rows identical (dev_skirmish 10:36, 11:25, 7:16; playtest.json 7:41, 12:28, 5:55); the games are deterministic, so any drift shows.
  - **Judge a computer player by its economy, not only its wins.** A2b and A2c each beat their passive mirror, yet left income unused: the Syndicate gathered at one hatch, and the Colonists sat at their Beacon Capacity cap with 882 SC banked at 3:00. A brief for an AI slice should give income or bank targets. Their behaviour is the Director's placeholder (0402), so such calls are the Director's, not the Designer's.
- **Keep building while the user can't playtest** (user, 2026-11-26): "I might not be able to playtest for the next 24 hours so just continue with the milestones work". The user also approved N7 and K ("N7 and K yes").
  - **The Director's reading:** finish M5's remaining slices in dependency order, then M6 = 0366–0369, the GDO's and the Syndicate's remaining content. The user was told and can redirect.
  - Planning spike M6-P drafts M6's plan meanwhile; it reads and writes only, so it runs as a fourth agent without builds.

## Lessons learned (process)
- **Never start a Bash command with a bare `cd`** into another directory, whether a worktree or a docs folder (`cd dir && grep …`). The session's cwd persists, and it slipped sixteen times (the sixteenth, 2026-11-27, `cd <x1a-int> && cargo test --no-run …` to build X1a's perf binary: the harness again made the worktree the primary directory; the fifteenth, 2026-11-27, `cd …/artifacts/designer/design && grep …` for the minimap alert's rule, and the fourteenth into N6's worktree: both were quick skims, so give grep the absolute path; the thirteenth, 2026-11-26, `cd <N2's worktree> && wc … && grep …` while skimming N2's occupants code: the harness then made the worktree the session's primary directory and warned against leaving it, which doesn't apply to the Director, until `cd /home/iv/dev/space_crystals` put it back. Skim a worktree with `git -C $WT …`, `grep … $WT/<path>` or `( cd $WT && … )`; the twelfth, 2026-11-26, `cd <L's worktree> 2>/dev/null; …; cd /home/iv/dev/space_crystals && …` while measuring comment widths: a leftover first `cd` that the same command undid, so the cwd didn't move, but it was still a slip; the eleventh, 2026-11-26, `cd …/artifacts/designer/design && ls systems/controls/; grep …` while finding where 0370 moved U1's turret sections: grep from the repo root with the full path; the tenth, 2026-11-26, `cd …/artifacts/designer/design && for f in …; grep` while listing the headings of the files 0370 split: use `grep … artifacts/designer/design/<path>` from the repo root; the ninth, 2026-11-25, `cd <worktree> && python3 …` while inspecting a golden in M5-B's worktree: use `( cd $WT && … )` or absolute paths; the eighth, 2026-11-25, `cd …/artifacts/designer/design && grep …` while checking question drafts; the seventh a design-path check, 2026-11-25: check paths with `[ -f artifacts/designer/design/$f ]`, never by cd-ing there). Use a subshell `( cd $WT && … )`, `git -C`, `--manifest-path`, or absolute paths. `cd /home/iv/dev/space_crystals && …` is fine. If it slips, run `cd /home/iv/dev/space_crystals` at once.
- **Folding design changes into a running agent with SendMessage works.** M0-D absorbed 0012/0013 and four review findings mid-task. Give exact file:line references and the fix direction.
- **Parallel agents**: give each the other's areas as a "don't touch" list, and carry cross-cutting state (e.g. the debug switch) through a separate type, so the integration merge stays small.
- **Typical cost**: an editor-sized task takes about 1 hour and 150–250k agent tokens; a read-only review takes about 25 minutes and 170k.
- **M1 slice costs**: S (selection, Opus) took 41 min and 350k tokens; D (collision, Fable) 41 min and 520k; W (combat, Opus) 51 min and 210k. Three parallel worktree agents with disjoint areas, each told to merge master before finishing, integrated with no Director-side conflict work.
- **Never `cd` in Bash, not even into `artifacts/director`**: it moves the session's working directory. Use absolute paths or a subshell.
- **Merge pipelines must fail loudly.** `git merge --ff-only X | tail -1 && git worktree remove ... && git branch -D X` took the exit status from `tail`. When master had moved, the merge failed, yet the branch was deleted anyway; the work was recovered from the commit id. Use `set -o pipefail`, or don't pipe the merge.
- **Deferring design changes until they can be verified**: 0033 rotation and 0035 collision response need long or accelerating units to test, so they wait for the vehicles milestone. This is recorded in plan.md.
- **The Bevy default UI font has ASCII glyphs only** ("°" and "×" show as boxes). Use ASCII in UI text until a font is added.
- Agent worktrees (`isolation: "worktree"`) live in `.claude/worktrees/` (gitignored) and showed the last *pushed* commit at creation, then caught up to local master. Briefs tell agents to `git merge --ff-only master` first anyway. Check `git worktree list` after launching.
- `git add -A <dir>` fails for an already-staged deleted directory; pass it only to `git commit -- <paths>`.
- **Tests must not depend on machine speed.** With several agents building in parallel, the load average reaches about 22. Wall-clock assumptions then flake: "one update at Max pace reaches the end" failed under load. So did perf numbers: master's `movement_perf` measured from 0.57 to 3.7 ms mean. Judge a perf change only by interleaved A/B runs against master, and take absolute numbers on a quiet machine.
- **Every test gets its own temp directory.** The "flaky under load" replay test was really two tests sharing `sc_replay_test_<pid>`, so whichever finished first deleted the other's folder. Suspect shared state before blaming timing.
- **Bevy 0.20 quirk:** a `Text2d` changed in a frame in which no camera saw any sprite or text is never laid out. Labels that must update anywhere use `NoFrustumCulling`, stay visible, and are emptied rather than hidden (M2-H2 ghost labels).
- **Agents' "readings to confirm":** sort each into design (a question with the built behaviour as the current behaviour) or technical (decide, and note it in the plan). Don't let them pile up in reports.
- **Split a slice** when part of it can start early (J1 ran during wave 4) or needs a quiet machine (J2's perf).
- **Ignored tests rot silently.** `nav_perf::knowledge_rebuilds_256` failed on master from M2-F2 until M2-J2, because `cargo test` skips `#[ignore]`. After a slice that changes game rules (defeat, economy), and at every milestone end, run `cargo test --release --workspace -- --ignored` with `--test-threads=1` on a quiet machine.

- **The seed sweep** (M5-Z1): `robustness::seed_sweep` (ignored) fuzzes seeds 0–255 against the invariants in about 6 s on the workspace build: `cargo test --workspace -- --ignored seed_sweep`. It is in `m5-common.md`'s checks, and in mine when integrating a slice that adds kinds or rules. It found the F2 bug that the single fixed `SEED` had missed for a wave.
- **Insert a new Built entry just before `\n## In progress`,** never after the previous entry's lines. An entry with a blank line in it (a table or a loose paragraph) ends a `(?:  .*\n)*` match early, so later inserts stranded A1's and N5's tails under N11 (found and moved back 2026-11-27, r151).
- **A split slice's first half is integrated from the Director's own worktree,** `.claude/worktrees/<x>-int`: a branch at its commit with a merge of master, never the agent's busy worktree, and never the scratchpad (the target is 32 GB). X1a was merged so (`4d17cff4`).
- **Launch worktree agents one per message.** Two in one message raced (2026-11-27): the second's `git worktree add` read the first's half-made metadata ("failed to read .git/worktrees/<other>/commondir: Success") and left a stale branch `worktree-agent-<id>`. Delete that branch (after checking `git log master..<branch>` is empty), then relaunch the failed agent alone.
- **Perf A/B tool:** `scratchpad/tools/perfab.sh <cpu> <rounds> <binA> <binB> <tests…>` runs interleaved rounds pinned to one CPU (0–3 are P-cores, 4–7 E-cores) and prints each side's mean figure.
- **Edit plan.md and insights.md through `artifacts/director/tools/plandoc.py`** (asserted exact replacements, table cells, rows). Its docstring gives the cell indexes: a change row's status is cell 1, a question row's is cell 3. Dry-run on a copy first. Three slips in one day (2026-11-26): a status written into the entry cell, a row glued onto the line above, and a second `rep` of a passage the first had already rewritten.
- **Plan-edit scripts are quoted heredocs** (`<<'PYEOF'`): an apostrophe inside a single-quoted Python string is `\'`, one backslash. Writing two broke r74 and r75 (SyntaxError, nothing written). Prefer double-quoted Python strings for text with apostrophes.
- **When 0370-style splits move sections,** short `` `> Section` `` lines under a file header escape a full-path citation search. Check each moved section's name against the lines below any header naming the old file (as for U1's `> AttackingObject` and `> TurretTurning`, fixed 2026-11-26).

- **Quote design text in briefs; don't paraphrase it.** M3-G's brief said "velocity along it zeroed" (per axis) where 0035 says "set velocity to 0" (all of it). The agent built the paraphrase, and had to be resumed to fix it before the merge.
- **Every scripted text replacement asserts it matched exactly once** (`assert s.count(old) == 1`). An unasserted `str.replace` on a brief silently did nothing, and the error only surfaced later.
- **The Designer answers questions within the hour,** often while the slice that raised them is still running. Triage at every hook notice, and fold answers into briefs that haven't launched yet (M3-V took 0169, 0175–0176 and 0183 that way). Slices waiting on a busy area split: M3-L became L1 (vision, now) and L2 (orders, after R).
- **Free slots go to small client or polish slices** (M3-U) when every queued slice waits on one running slice (R).
- **An agent's merge commit with git's default message can't be fixed afterwards:** the permission system refuses `git filter-branch` and other history rewrites, and doing it for the agent would be permission laundering. M5-K's branch went in as one attributed squash commit: built on a `k-squash` branch in the agent's worktree (`git switch -c k-squash master`, `git merge --squash`, `git commit -F`), checked identical to the tested tree with `git diff --quiet`, then fast-forwarded. The squash message lists the branch's commits. `m5-common.md` and the launch prompts now ask for `-m` on every merge. The same squash suits a branch whose merge of master also carries new code, or whose work commit is marked "(wip)": M5-N10 went in so (`eb1e03cf`), since a merge that adds code hides it from `git log -p`.
- **After a conflicted merge, `git commit --no-edit` keeps git's "# Conflicts" lines,** after the attribution lines (its cleanup is "whitespace" when the message isn't edited). Commit with `--cleanup=strip --no-edit`, or `-F` a clean message. My MM2 merge needed an amend, which is allowed on an unpushed commit of my own.

## M4 plan revision 2 (adopted 2026-11-25)
- `artifacts/director/m4-plan.md` is the planner's revision 2, which folds in 0229–0282. It adds R1 and R2 (code organization), O (combat spike), U (Grenadier), and S1 and S2 (Laser Turrets). The waves and the critical path are in `plan.md`'s M4 entry.
- The common brief, `scratchpad/briefs/m4-common.md`, now binds T29–T54, cites the per-object design paths (0282) and the code-organization rules (T54). Update it whenever the plan's structure changes.
- M4-R runs alone: no other slice beside R1 and R2, since the refactor touches nearly every file. Briefs written after it lands must use its new module layout.
- The next question number is **Q0131**.
- **Build timing (2026-11-25, load 13 on 8 cores):** a full check after a change takes about 72 s to compile and link and 41 s to run the 1877 tests; fmt and a warm clippy take seconds. Agents spend 20–35% of a slice in cargo and most of the rest reasoning. rustc 1.100 already links with LLD by default, so mold would gain little; fewer test binaries (116 today) is the bigger lever, best done between waves.
- **When the Designer moves or splits design files** (0370, 2026-11-26), master's `design_citations` test fails at once.
  - **Fix it on master yourself,** by script, before any agent merges master:
    - map each old (file, first section) to its new file, from the new files' headings and bold labels;
    - hand-map the citations of a whole file;
    - re-flow any comment line pushed past 120 columns;
    - check that the words are unchanged and no code line changed.
  - **Warn the running agents first:** they must not fix other files' citations, and in a conflict they take master's path.
  - Citations are often wrapped across two comment lines (`economy.md >` then `/// RecruitmentHall > …`), so the script must follow the wrap.
  - **Re-flowing doc comments:**
    - Never cut a line so that the next begins with a Markdown marker (`>`, `- `, `#`, `1. `). A line starting with `> ` is a quote, and clippy's `doc_lazy_continuation` fails the build.
    - A short citation (`` `> Section` ``) is allowed only in a module whose doc names exactly one file (`design_citations::rule`). A split that makes such a module name two files breaks it, so write the path in full.
    - Master's rustdoc had 121 warning lines before the rewrite, which is the baseline to compare against.
- **After M5-T (`4ecfc18a`), the test programs and perf:**
  - The integration tests are modules of three programs: `sim`, `perf` and `game`. One file's tests: `cargo test -p sc_sim --test sim <module>::`.
  - The guard `test_programs` fails, naming any test file left out.
  - An edit then `--workspace --no-run` takes about 16 s (sim) or 26 s (client), and the full run about 29 s.
  - **After every merge, build master's perf binary:** `cargo test -p sc_sim --release --test perf --no-run` in the main checkout, then copy the executable (cargo 1.100 puts it at `target/release/build/sc_sim/<hash>/out/perf-<hash>`) to `target/perf/<sha>`, named by the last code commit: `git log -1 --format=%h master -- crates Cargo.toml Cargo.lock`. Agents use it as side A of an A/B (`m5-common.md`).
  - **After a slice that changes the profile,** seed the main checkout's `target/` from its worktree (`cp -a --reflink=auto $WT/target/. target/`) before removing the worktree. Agents copy the main cache at launch, so a stale one costs each of them about 14 minutes.
  - `-p sc_sim` and `--workspace` build the sim with different features, so mixing them compiles it twice.
- **M5 planning started early (2026-11-26).** M5-P, a read-only planning spike on Opus (brief `scratchpad/briefs/m5-planning.md`), drafts `scratchpad/m5/m5-plan-draft.md` while M4's last slices run. M5 ends at the user's first playtest, so the draft also plans playtest readiness for all four factions, and the user's "data files at playtest" step. Adopted as `artifacts/director/m5-plan.md` (2026-11-25), with the Director's decisions at its top; Q0092–Q0108 filed from its drafts. The user's product decisions (the opponent, a start screen) come before M5 starts.
- **Check a planning spike's question drafts with a fresh agent before filing** (M6, 2026-11-26). M6-P's 17 drafts were good, but the read-only check M6-Q (brief `scratchpad/briefs/m6-questions-check.md`) changed all 16 it checked:
  - quotes the docs didn't hold, or missed quotes that supported another option;
  - points the docs or older entries already answered, dropped;
  - the arithmetic of what the placeholder numbers mean in play.

  Also check the spike's plan against the milestone it follows: M6-P found M5 had no slice for tracked movement or crushing, which N10 needs.

## Code layout after M4-R (R1 `8eab2290`, R2 `c60b8d0f`)
`ARCHITECTURE.md > Adding a unit or structure` is the guide. Point every brief at it.
- **The sim:**
  - `defs/<faction>/{units,structures}/<object>.rs` and `defs/map/`. Each faction's own rules (caps, divisor) go in `defs/<faction>.rs`.
  - One `kind_registry!` line per kind, and new kinds go last.
  - Behaviour numbers go in `tuning/<system>.rs`, cited or marked "Placeholder (not in the design)".
  - **The placeholder mark form (M5 standard):** `Placeholder (Q0097 point 2)` for an open question's point, and `Placeholder (not in the design: why)` for a missing number. The colon form `Placeholder: Q00NN` is legacy (one left, cleared by M4-Z). Once the answer arrives, the mark becomes a citation of its change entry.
  - Timers use `progress.rs`; toggles use `toggle.rs`.
- **Serde-ready defs.** Names and lists are `&'static`, and `defs/leak.rs` leaks them once on load. Accepted for the playtest loader.
- **Timers not on `Progress` yet,** because their hashed bytes would change: production, construction, the Extraction Facility cycle, Weapon Conversion, the Riot Shield, mining and unloading, and the network's held units. Each moves at the next re-bless that touches it.
- **The client:**
  - `kinds/<faction>/<object>.rs` (label, `Look`, `ShotStyle`, `PanelTemplate`) behind the exhaustive `kinds::of`;
  - panels in `command_ui/interface/templates/<faction>/<object>.rs`;
  - feel tunables in `crates/game/src/tuning/`.
- **Citations.** `crates/sim/tests/design_citations/` fails on any cited path that doesn't exist. The rule is full paths, with a relative `> Section` only inside a module whose doc names exactly one design file.
- **Files** stay under about 500 lines (`ARCHITECTURE.md`). Splitting is a normal part of every slice.

## Lessons from M4 so far
- **A comments-only pass across the repository** (citations) must run while nothing else does, or every slice gets conflicts in its comments.
- **A golden re-bless is fine when you can explain it.** M4-B re-blessed 8 goldens and proved it by re-running with the new starting banks zeroed and getting the old hashes back. Ask for that kind of proof whenever a slice re-blesses.
- **Ask agents to commit in coherent steps,** each building and passing the tests. R1 did, across 245 files, and review was easy.
