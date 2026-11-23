# verify_agent_tunnel_building_r1

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-agent_tunnel_building_r1.md

## Task

Verification task: The QA failure reported that Build Tunnel (A) does not produce a ghost placement overlay. This was the same root cause as agent_interface_r1 and agent_groupable_construction_r1 — a missing match arm in manage_placement_ghost (placement.rs). That fix has ALREADY been applied (line 55-58 of placement.rs now handles AgentAwaitingPlacement).

Since the fix landed AFTER this QA rework was filed, verify the full Agent Tunnel Building flow works end-to-end:

1. **Ghost placement**: Select Agent → press A → confirm ghost Tunnel preview appears on cursor with green/red validity tinting
2. **Construction initiation**: Left-click valid location → Agent gets UnitCommand::BuildTunnel + BuildingTunnelBehavior → walks to site
3. **Construction progress**: Tunnel spawns at 10% HP (ConstructionHP), construction completes in ~30 seconds (480 frames at 16fps)
4. **Completion**: Agent enters TunnelNetwork (InTunnelNetwork + Visibility::Hidden), Tunnel becomes operational
5. **Destruction mid-build**: Destroying a partially-built Tunnel — Agent survives (emerges), Supplies not refunded
6. **Cost scaling**: tunnel_construction_cost formula — 2nd Tunnel costs 1 Supply, 3rd costs 2, etc.

If all systems work correctly, this is a pass-through verification. If any step fails, investigate and fix. The underlying systems (BuildingTunnelBehavior, build_tunnel_behavior_system, spawn_tunnel_under_construction, tunnel_construction_cost) have existing tests — run cargo test to confirm they still pass.

Key files: src/game/world/faction/placement.rs, src/game/units/systems/behaviors/construction.rs, src/game/entities/factions/syndicate/buildings/tunnel.rs
