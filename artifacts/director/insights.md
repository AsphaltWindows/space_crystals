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

## Lessons learned (process)
- **Never `cd` in a Bash command in this session.** It moves the session's cwd (it happened three times). Use absolute paths, `git -C`, or a subshell `( cd X && ... )`.
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
- **Never start a Bash command with a bare `cd`**, whether into a worktree or a docs folder (`cd dir && grep …`). The session's cwd persists, and it slipped twice. Use a subshell `( cd $WT && … )`, `git -C`, `--manifest-path`, or absolute paths. If it slips, run `cd /home/iv/dev/space_crystals` at once.
- **Every test gets its own temp directory.** The "flaky under load" replay test was really two tests sharing `sc_replay_test_<pid>`, so whichever finished first deleted the other's folder. Suspect shared state before blaming timing.
- **Bevy 0.20 quirk:** a `Text2d` changed in a frame in which no camera saw any sprite or text is never laid out. Labels that must update anywhere use `NoFrustumCulling`, stay visible, and are emptied rather than hidden (M2-H2 ghost labels).
- **Agents' "readings to confirm":** sort each into design (a question with the built behaviour as the current behaviour) or technical (decide, and note it in the plan). Don't let them pile up in reports.
- **Split a slice** when part of it can start early (J1 ran during wave 4) or needs a quiet machine (J2's perf).
- **Ignored tests rot silently.** `nav_perf::knowledge_rebuilds_256` failed on master from M2-F2 until M2-J2, because `cargo test` skips `#[ignore]`. After a slice that changes game rules (defeat, economy), and at every milestone end, run `cargo test --release --workspace -- --ignored` with `--test-threads=1` on a quiet machine.

- **Quote design text in briefs; don't paraphrase it.** M3-G's brief said "velocity along it zeroed" (per axis) where 0035 says "set velocity to 0" (all of it). The agent built the paraphrase, and had to be resumed to fix it before the merge.
- **Every scripted text replacement asserts it matched exactly once** (`assert s.count(old) == 1`). An unasserted `str.replace` on a brief silently did nothing, and the error only surfaced later.
- **The Designer answers questions within the hour,** often while the slice that raised them is still running. Triage at every hook notice, and fold answers into briefs that haven't launched yet (M3-V took 0169, 0175–0176 and 0183 that way). Slices waiting on a busy area split: M3-L became L1 (vision, now) and L2 (orders, after R).
- **Free slots go to small client or polish slices** (M3-U) when every queued slice waits on one running slice (R).

## M4 plan revision 2 (adopted 2026-11-25)
- `artifacts/director/m4-plan.md` is the planner's revision 2, which folds in 0229–0282. It adds R1 and R2 (code organization), O (combat spike), U (Grenadier), and S1 and S2 (Laser Turrets). The waves and the critical path are in `plan.md`'s M4 entry.
- The common brief, `scratchpad/briefs/m4-common.md`, now binds T29–T54, cites the per-object design paths (0282) and the code-organization rules (T54). Update it whenever the plan's structure changes.
- M4-R runs alone: no other slice beside R1 and R2, since the refactor touches nearly every file. Briefs written after it lands must use its new module layout.
- The next question number is **Q0086**.
