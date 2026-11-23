# entity-registry-grid-layout

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-declarative-entity-definitions.md

## Task

Replace the hardcoded command panel grid layout match in grid_layout.rs with EntityRegistry lookups.

**Current state:** `get_grid_slot_action()` in `ui/command_panel/grid_layout.rs` is a large match on ObjectInterfaceState mapping (row, col) to CommandButtonAction. Each entity's slots are hardcoded as match arms.

**After this task:** `get_grid_slot_action()` queries the EntityRegistry resource to find command_slots for the given entity/state, then looks up the (row, col) in those slots. The match arms are removed.

**Implementation:**
1. Modify `get_grid_slot_action()` to accept `&EntityRegistry` (or access it as a Bevy resource)
2. Look up the entity's RegistryEntry via ObjectEnum
3. Search its command_slots for matching (state, row, col) -> return the action
4. Remove all per-entity match arms from grid_layout.rs
5. Update all callers to pass the registry

**This depends on all entity migrations being complete** (all entities must have their command_slots populated in the registry before the match arms can be removed).

**Tests:**
- For every ObjectInterfaceState and every (row, col) position: registry-based lookup returns identical CommandButtonAction as the old match-based implementation
- No regression in command panel behavior
- grid_layout.rs is significantly smaller after this change

**Reference:** Architecture doc Phase 6 (Wire Registry Consumers)
