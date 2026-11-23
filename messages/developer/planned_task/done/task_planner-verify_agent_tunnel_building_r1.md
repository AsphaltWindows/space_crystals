# verify_agent_tunnel_building_r1

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

**This is a verification-only task.** The root cause fix (AgentAwaitingPlacement ghost) is already in place. The developer should verify compilation and tests, then mark complete. No code changes expected unless tests fail.

### Verified State (as of task planning):
- **`cargo test --lib`**: All 2103 tests pass, including 216 tunnel-related and 152 building-related tests
- **Integration tests** (`tests/qa/`, `tests/scenarios/`): Have pre-existing compilation errors unrelated to tunnel building (missing imports in guard_unit.rs, scenarios/main.rs). These are NOT in scope for this task.

### Key Files (all already complete — read-only verification):

1. **`src/game/world/faction/placement.rs`** (lines 55-58) — The fix: `AgentAwaitingPlacement` match arm in `manage_placement_ghost()` reads `placement_state.building_type` to spawn the ghost mesh. This was the missing piece that caused the QA failure.

2. **`src/game/units/systems/behaviors/construction.rs`** — `building_tunnel_behavior_system()` (line 94) handles the full lifecycle:
   - `BuildTunnelPhase::MovingToSite`: Agent walks to target, validates placement via `can_worker_place_structure()`, checks single-agent enforcement, deducts supply cost via `tunnel_construction_cost()`, spawns tunnel via `EntityRegistry::spawn()` with `under_construction: true`, overrides `ConstructionHP` with `AGENT_TUNNEL_BUILD_FRAMES`, hides agent
   - `BuildTunnelPhase::Constructing`: Ticks frames, checks if tunnel destroyed mid-build (agent emerges with `Visibility::Inherited`), on completion agent is despawned (enters tunnel network)

3. **`src/game/entities/factions/syndicate/buildings/tunnel.rs`** — `TunnelDef` entity definition, `spawn()` method handles `under_construction` flag, re-exports `tunnel_construction_cost`, `tunnel_t2_upgrade_cost`, `tunnel_t3_upgrade_cost`

4. **`src/ui/command_panel/actions.rs`** (line 595) — `CommandButtonAction::AgentBuildTunnel` handler sets `PlacementState.building_type = Some(ObjectEnum::Tunnel)` and transitions to `AgentMenu(AgentAwaitingPlacement)`

5. **`src/game/types/structures.rs`** — `tunnel_construction_cost(existing_count)` implements the scaling formula, `ConstructionHP::new()` / `hp_fraction()`

### Test Coverage (already passing):
- `building_tunnel_spawned_tunnel_has_construction_hp` — verifies ConstructionHP on spawned tunnel
- `building_tunnel_spawned_tunnel_starts_at_10_percent_hp` — verifies 10% initial HP
- `building_tunnel_rejects_second_agent_at_same_location` — single-agent enforcement
- `building_tunnel_no_player_cancels` — edge case handling
- 216 tunnel UI/grid tests, 152 building behavior tests

### Developer Action:
1. Run `cargo test --lib` — confirm all 2103+ tests pass
2. Run `cargo build` — confirm compilation succeeds
3. Optionally grep for `AgentAwaitingPlacement` in placement.rs to visually confirm the fix is present
4. If everything passes, mark task complete — no code changes needed
5. If any tunnel/building test fails, diagnose and fix within the construction.rs / placement.rs scope

### Note on Integration Test Failures:
The `tests/qa/guard_unit.rs` and `tests/scenarios/main.rs` files have pre-existing compilation errors (missing `guard_type_data`/`guard_attack_data` imports, removed `spawn_peacekeeper`/`spawn_tunnel`/`spawn_headquarters` functions, Bevy API changes). These are NOT related to tunnel building and are OUT OF SCOPE for this task.

## Dependencies

None — this is a standalone verification task. All tunnel-building systems are fully implemented and tested.
