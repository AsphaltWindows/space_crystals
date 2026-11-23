# Implementation Plan

_Updated 2026-11-23. Spec: `artifacts/designer/design/`. Code rules: `ARCHITECTURE.md`._

Approach: thin vertical slices the user can run and see, each verified by tests and agent-taken screenshots.

## Built
- **Workspace scaffold** (`8ce3444`): `sc_sim` core interface — units and coordinates, ids, libm math, tile map with default presets, object defs and instances, commands, events, scenarios (`dev_skirmish`, `empty`), `Sim` tick pipeline, Supply Delivery Station countdown. Determinism lints in `crates/sim/clippy.toml`. Bevy 0.20.1 client stub.

## In progress
- **ARCHITECTURE.md**: brief for implementing agents (being written by an agent).

## Queued
### M0 — Foundation (next; two agents in parallel, separate worktrees)
- **M0-A sim** (`crates/sim` only): compact ASCII map format for tests; seeded deterministic PRNG; state hash + determinism test; replay type (scenario + per-tick inputs) with a runner; scenario test harness; tests for map, patches, stations and debug commands, including "invalid input never panics".
- **M0-B client** (`crates/game` only): app shell with feature plugins; 16 Hz FixedUpdate driver and interpolation; camera (28 GU wide, viewport between HUD bars, arrow/edge pan, clamped, map start position); map rendering (preset colours, elevation shading, grid toggle; no entity per tile); placeholder patches and stations with cached handles and y-sorting; HUD frame (top bar plus five fixed bottom sections, empty); QA mode (`--scenario`, `--screenshot`, `--exit-at-tick`, `--dump-state`); deterministic client tests.

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

## Design change entries
| Entry | Status | Notes |
|---|---|---|
| — | — | none published yet (last seen: 0) |
