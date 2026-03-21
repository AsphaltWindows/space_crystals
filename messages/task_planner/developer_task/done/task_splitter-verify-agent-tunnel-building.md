# verify-agent-tunnel-building

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-agent-tunnel-building.md

## Task

**Verification-only task**: The Agent Tunnel construction behavior is already fully implemented. All components are in place:

- `BuildingTunnelBehavior` with `BuildTunnelPhase` (MovingToSite → Constructing) in `src/game/units/types/state/behavior.rs`
- `build_tunnel_behavior_system` in `src/game/units/systems/behaviors.rs` (~line 900) handling the full lifecycle:
  - Movement to build site with arrival threshold
  - Single-Agent construction enforcement (no two agents at same location)
  - Supply cost calculation via `tunnel_construction_cost()` with deduction and insufficient-funds check
  - Tunnel spawning via `spawn_tunnel_under_construction()` (10% HP via ConstructionHP)
  - Agent hidden (`Visibility::Hidden`) during construction (untargetable)
  - Construction completion → Agent despawns into tunnel network
  - Tunnel destruction → Agent emerges with `Visibility::Inherited`
- `ConstructionHP` component in `src/game/types/structures.rs` with `hp_fraction()` implementing the 10% + 90% * progress formula
- `construction_hp_tick_system` in `src/game/world/faction.rs` (~line 789) that ticks progress and scales HP
- `tunnel_construction_cost()` in `src/game/types/structures.rs` (~line 692) implementing scaling costs (0, 1, 2, ...)
- `UnitCommand::BuildTunnel(Vec3)` command variant with availability, indicator, and hotkey tests
- `CommandButtonAction::AgentBuildTunnel` UI integration in command_panel.rs
- Extensive test coverage for all scenarios (arrival, construction, destruction, completion, cost deduction, insufficient funds, single-agent enforcement)

**Your task**: Verify the existing implementation compiles and all related tests pass. Run `cargo test` in `artifacts/developer/`. If everything passes, this task is complete — no code changes needed.
