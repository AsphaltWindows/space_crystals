# Implementation Plan

_Updated 2026-11-23. Spec: `artifacts/designer/design/`. Code rules: `ARCHITECTURE.md`._

Approach: thin vertical slices the user can run and see, each verified by tests and agent-taken screenshots.

## Built
- **Workspace scaffold** (`8ce3444`): `sc_sim` core interface — units and coordinates, ids, libm math, tile map with default presets, object defs and instances, commands, events, scenarios (`dev_skirmish`, `empty`), `Sim` tick pipeline, Supply Delivery Station countdown. Determinism lints in `crates/sim/clippy.toml`. Bevy 0.20.1 client stub.

- **ARCHITECTURE.md**: the binding brief for implementing agents.
- **M0-A sim foundation** (`1d966e3`, 66 tests): ASCII test maps; PCG32 `Rng`; `Sim::state_hash` (exhaustive, so a new field won't compile until hashed); `Sim` serde; `Replay`; `sc_sim::testing`; versioned JSON scenario files; `ObjectKind::ALL`; `Scenario::validate` and `Sim::try_new`; `PlayerSetup::start_location` (Q0004 placeholder); non-finite positions rejected.
  - Follow-ups: a debug-spawned station delivers at spawn tick + 959, while a scenario station delivers at tick 960, because objects are updated in the tick they spawn. Settle the convention (proposed: objects spawned during a tick start updating on the next tick). Review the whole of M0 once M0-B is merged.

## In progress
- **M0-B client**: agent running in its own worktree (details under M0 below).

## Queued
### M0 — Foundation (next; two agents in parallel, separate worktrees)
- **M0-A sim** (`crates/sim` only): compact ASCII map format for tests; seeded deterministic PRNG; state hash + determinism test; replay type (scenario + per-tick inputs) with a runner; scenario test harness; tests for map, patches, stations and debug commands, including "invalid input never panics". Editor support: readable, versioned scenario files (JSON with tile and elevation rows as strings); `ObjectKind::ALL`; `Scenario::validate` and `Sim::try_new`; per-slot `start_location` (placeholder pending Q0004).
- **M0-B client** (`crates/game` only): app shell with feature plugins; 16 Hz FixedUpdate driver and interpolation; camera (28 GU wide, viewport between HUD bars, arrow/edge pan, clamped, map start position); map rendering (preset colours, elevation shading, grid toggle; no entity per tile); placeholder patches and stations with cached handles and y-sorting; HUD frame (top bar plus five fixed bottom sections, empty); QA mode (`--scenario`, `--screenshot`, `--exit-at-tick`, `--dump-state`); deterministic client tests.

### M0-C — Map editor (user request 2026-11-23; starts once M0-A and M0-B are merged)
An editor mode inside the client (`cargo run -- --editor [maps/x.json]`) that reuses the game's renderer, camera and input layer, so maps look exactly as in game:
- new map with a chosen size;
- paint presets and elevation (set, raise, lower) with brush sizes;
- place, move and delete any object type for any owner (the palette comes from `ObjectKind::ALL`, so new units and buildings appear automatically);
- player slots with start locations and camera starts;
- undo/redo; save and load text files under `maps/`; validation warnings;
- play-test the current map; mirror/symmetry tools.
Agents drive it through the same scripted input actions and QA screenshots. It's a development and content tool; making it a player-facing feature would be a design decision. Start-location semantics wait on Q0004.

### M1 — Units move & fight (sandbox)
Peacekeeper (GDO) and Guard (Syndicate). Pathfinding and hard collision; Move, Stop, Hold, Attack, AttackMove and Patrol commands with Shift-queueing; attack phases and damage; vision and fog of war; selection (clicks, box tiers), control groups, Tab cycling; command panel (BasicCombatUnitInterfaceState), right-click resolution, cursors, command indicators; debug tools to control both sides; replays. A movement design spike may come first. Fable is reserved for pathfinding, movement and collision.

### M2 — GDO base & economy
Deployment Center, Generator, Barracks, Engineering Facility, plates; power; unit control; build area; placement ghost with rotate/flip; production queues and rally points; crystal harvesting; defeat conditions. GDO supplies (Supply Tower, Chopper).

### M3–M5 — Other factions, basics
M3 Colonists (Drone, Beacon, Warp In, Egress Node, portals, Legionnaires, shields). M4 Syndicate (tunnels, Hideout, Agent, Guard economy). M5 Cults (Recruitment Hall, Storage, Recruit, Armory, Soldier, Gunner).

### Later
Vehicles, turrets, air, area damage, research; AI or online opponents (decide with the user); art pipeline (agent-generated pixel-art sprites with per-asset user sign-off); audio (needs `alsa-lib-devel`).

## Open design questions
| Q | Topic | Blocking | Status |
|---|---|---|---|
| Q0001 | Art direction: projection, facings, building orientations, elevation, pixel scale | no | open |
| Q0002 | Input bindings: map clicks, control-group keys, pan inputs, queue modifier | no | open |
| Q0003 | Patch amount, supply delivery defaults, reference map | no | open |
| Q0004 | Map model: player slots, start locations, starting-base layout, pre-placed objects | no | open |
| Q0005 | CommandPanel standard slot positions vs the 4×3 grid | no | open |

## Design change entries
| Entry | Status | Notes |
|---|---|---|
| — | — | none published yet (last seen: 0) |
