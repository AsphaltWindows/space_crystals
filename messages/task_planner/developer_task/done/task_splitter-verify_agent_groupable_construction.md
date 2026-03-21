# verify-agent-groupable-construction

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-agent-groupable-construction.md

## Task

Verify that Agent's Groupable=false constraint and single-Agent construction enforcement are correctly implemented. Both features appear to already exist in the codebase:

1. **Groupable=false**: Agent has `groupable: false` set in `src/game/types/objects.rs` (line 230). The `Selection::build_from_entities()` method in `src/shared/types.rs` (line 173) already handles ungroupable entities by giving each its own SelectionGroup. Tests exist: `agent_is_ungroupable`, `multiple_agents_selection`, `mixed_agents_and_groupable_units_selection`.

2. **Single-Agent construction enforcement**: Implemented in two places:
   - `src/game/world/faction.rs` (line 1483): Rejects BuildTunnel command at placement time if another Agent is already building at the same location.
   - `src/game/units/systems/behaviors.rs` (line 911): Rejects at runtime when an Agent arrives at a build site already under construction by another Agent. The rejected Agent goes idle.
   - Tests exist: `second_agent_rejected_at_same_site`, `second_agent_allowed_at_different_site`, `second_agent_rejected_at_placement`.

**Your task**: Confirm that all existing tests pass (`cargo test` in `artifacts/developer/`). If any tests fail or implementation gaps are found relative to the feature spec, fix them. If everything passes, mark the task as complete.
