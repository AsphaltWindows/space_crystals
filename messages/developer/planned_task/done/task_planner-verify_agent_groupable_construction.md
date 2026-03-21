# verify-agent-groupable-construction

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-agent-groupable-construction.md

## Task

Verify that Agent's Groupable=false constraint and single-Agent construction enforcement are correctly implemented. Both features appear to already exist in the codebase:

1. **Groupable=false**: Agent has `groupable: false` set in `src/game/types/objects.rs` (line 230). The `Selection::build_from_entities()` method in `src/shared/types.rs` (line 173) already handles ungroupable entities by giving each its own SelectionGroup. Tests exist: `agent_is_ungroupable`, `multi_agent_selection_creates_separate_groups`, `mixed_agents_and_groupable_units_selection`.

2. **Single-Agent construction enforcement**: Implemented in two places:
   - `src/game/world/faction.rs` (line 1483): Rejects BuildTunnel command at placement time if another Agent is already building at the same location.
   - `src/game/units/systems/behaviors.rs` (line 911): Rejects at runtime when an Agent arrives at a build site already under construction by another Agent. The rejected Agent goes idle.
   - Tests exist: `building_tunnel_rejects_second_agent_at_same_location`, `building_tunnel_allows_agent_at_different_location`.

**Your task**: Confirm that all existing tests pass (`cargo test` in `artifacts/developer/`). If any tests fail or implementation gaps are found relative to the feature spec, fix them. If everything passes, mark the task as complete.

## Technical Context

### Files involved (all under `artifacts/developer/`)

1. **`src/game/types/objects.rs`** — Agent's `groupable: false` is set at line 230 in `ObjectEnum::SyndicateAgent`'s `object_type()` implementation. Test at line 1100: `test_syndicate_agent_is_ungroupable`.

2. **`src/shared/types.rs`** — `Selection::build_from_entities()` (line 173) handles ungroupable entities by giving each its own SelectionGroup. Key tests:
   - `agent_is_ungroupable` (line 814) — confirms `groupable: false` on the ObjectType
   - `multi_agent_selection_creates_separate_groups` (line 820) — 3 Agents → 3 separate groups, Tab cycles through all
   - `mixed_agents_and_groupable_units_selection` (line 847) — 2 Peacekeepers + 2 Agents → 1 grouped PK group + 2 individual Agent groups
   - `build_from_entities_preserves_active_group_on_rebuild` (line 873) — verifies active group type is preserved across rebuild

3. **`src/game/world/faction.rs`** (line 1479-1498) — Placement-time rejection: In `AgentMenu(AgentAwaitingPlacement)` handler, checks `existing_builders` query to reject BuildTunnel if another Agent targets the same grid cell (distance < 1.0 on x and z).

4. **`src/game/units/systems/behaviors.rs`** — Runtime rejection in `building_tunnel_behavior_system`:
   - Lines 911-920: Collects all entities in `BuildTunnelPhase::Constructing` into `constructing_locations`
   - Lines 929-944: When Agent B arrives (`MovingToSite` phase, distance < BUILD_ARRIVAL_THRESHOLD), checks if another entity is already constructing within 1.0 distance. If so, rejects: sets Idle, removes `BuildingTunnelBehavior` component.
   - Tests (line 1979+):
     - `building_tunnel_rejects_second_agent_at_same_location` — Agent A constructing, Agent B arrives at same spot → B rejected
     - `building_tunnel_allows_agent_at_different_location` (line 2035) — Agent B at different spot → allowed

### How to verify

Run `cargo test` from `artifacts/developer/`. All tests listed above should pass. If any fail, investigate the specific assertion. If all pass and the implementation matches the design spec in `artifacts/designer/design/syndicate_objects.md` and `artifacts/designer/design/control_system.md`, the task is complete.

### Potential gaps to check
- The placement-time rejection (faction.rs) uses an `existing_builders` query — confirm it queries for `BuildingTunnelBehavior` components with both `MovingToSite` and `Constructing` phases (not just one).
- The runtime rejection (behaviors.rs) only checks entities in `Constructing` phase. An edge case: two Agents both in `MovingToSite` heading to the same location won't be caught until one transitions to `Constructing`. The placement-time guard should prevent this in normal play, but if commands are issued via other paths, both could arrive. This is likely acceptable given the design says "reject or idle."

## Dependencies

None — this is a verification-only task with no code changes expected (unless tests fail).
