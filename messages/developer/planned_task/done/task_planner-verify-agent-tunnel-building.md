# verify-agent-tunnel-building

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-agent-tunnel-building.md

## Task

**Verification-only task**: The Agent Tunnel construction behavior is already fully implemented. Verify the existing implementation compiles and all related tests pass. Run `cargo test` in `artifacts/developer/`. If everything passes, this task is complete — no code changes needed.

## Technical Context

This is a **verification-only task** — no files need to change. The developer should:

1. Run `cargo build` in `artifacts/developer/` to confirm compilation
2. Run `cargo test` in `artifacts/developer/` to confirm all tests pass

The implementation spans these files (all already complete):

- **`src/game/units/types/state/behavior.rs`** — `BuildingTunnelBehavior` component, `BuildTunnelPhase` enum (MovingToSite, Constructing)
- **`src/game/units/systems/behaviors.rs`** — `building_tunnel_behavior_system()` (line ~892) handles movement to site, single-agent enforcement, supply cost deduction, tunnel spawning, agent hiding, construction completion, and tunnel destruction emergence. Extensive test coverage starts at line ~1513.
- **`src/game/types/structures.rs`** — `ConstructionHP` component with `hp_fraction()`, `tunnel_construction_cost()` function (line ~692)
- **`src/game/world/faction.rs`** — `construction_hp_tick_system()` (line ~789) ticks progress and scales HP
- **`src/game/units/systems/core.rs`** — `UnitCommand::BuildTunnel(Vec3)` dispatch (line ~1212), `CommandType::BuildTunnel` handling (line ~588)
- **`src/ui/command_panel.rs`** — `CommandButtonAction::AgentBuildTunnel` UI integration

**Note**: The task description references `build_tunnel_behavior_system` but the actual function name is `building_tunnel_behavior_system` — this is just a naming discrepancy in the task description, not a code issue.

If compilation or tests fail, diagnose and fix the issue. The fix scope should be limited to making existing tunnel-building code compile and pass tests.

## Dependencies

None — this is a standalone verification task. All tunnel-building code is already implemented.
