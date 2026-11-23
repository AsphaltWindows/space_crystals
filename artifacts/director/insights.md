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

## Lessons learned (process)
- `git add -A <dir>` fails for an already-staged deleted directory; pass it only to `git commit -- <paths>`.
