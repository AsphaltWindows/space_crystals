# test_harness_selection_fix

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-test_suite_cleanup.md

## Task

Fix the TestHarness::set_selection() infrastructure bug and the 2 Category C interface state test failures it causes.

### Problem

TestHarness::set_selection() (src/shared/testing/harness.rs:264) only adds the Selected ECS component to entities. It does NOT update the Selection resource's groups field. When update_command_panel_state runs during test_app.step(), it reads selection.groups to determine active_type(). Since groups is empty, active_type() returns None and the system resets ObjectInterfaceState to Default.

### Fix Required

1. In set_selection(), after adding/removing Selected components, also update the Selection resource.
2. In clear_selection(), ensure Selection resource groups is also cleared.
3. Verify the 2 Category C tests now pass.

### Files
- src/shared/testing/harness.rs -- fix set_selection() and clear_selection()
- tests/qa/automated_qa_ui_state_queries.rs -- verify tests pass (no changes expected)

### Verification
- cargo test --test qa automated_qa_ui_state_queries -- both step_1 and step_3 tests must pass
- cargo test --lib -- no regressions

## Technical Context

### Primary file to modify: `artifacts/developer/src/shared/testing/harness.rs`

**Current set_selection() implementation (line 264-282):**
The method removes `Selected` from all entities, then adds `Selected` to the specified entities. It never touches the `Selection` resource.

**Required change in set_selection():**
After updating the `Selected` ECS markers, build the `Selection` resource groups. The method should:

1. Query `ObjectInstance` from each entity to get its `object_type: ObjectEnum`
2. Look up groupability via `object_type.object_type().groupable` (the `ObjectEnum::object_type()` method returns `ObjectType` which has a `pub groupable: bool` field)
3. Build a list of `(Entity, ObjectEnum, bool)` tuples
4. Call `selection.build_from_entities(&tuples)` on the `Selection` resource

Here is the pattern to follow (using existing Bevy world access style from the same file):

```rust
pub fn set_selection(&mut self, entities: &[Entity]) {
    let world = self.app.world_mut();

    // Remove Selected from all entities that currently have it
    let currently_selected: Vec<Entity> = world
        .query_filtered::<Entity, With<Selected>>()
        .iter(world)
        .collect();
    for e in currently_selected {
        world.entity_mut(e).remove::<Selected>();
    }

    // Add Selected to the specified entities and collect type info for Selection resource
    let mut entity_info: Vec<(Entity, ObjectEnum, bool)> = Vec::new();
    for &e in entities {
        if world.get_entity(e).is_ok() {
            world.entity_mut(e).insert(Selected);
            // Read ObjectInstance to get type info for Selection groups
            if let Some(obj) = world.get::<ObjectInstance>(e) {
                let obj_type = obj.object_type;
                let groupable = obj_type.object_type().groupable;
                entity_info.push((e, obj_type, groupable));
            }
        }
    }

    // Update the Selection resource groups
    let mut selection = world.resource_mut::<Selection>();
    selection.build_from_entities(&entity_info);
}
```

**Required change in clear_selection():**
The current implementation delegates to `set_selection(&[])`, which with the fix above will correctly clear both `Selected` markers AND the `Selection` resource (since `build_from_entities(&[])` clears groups and sets `active_group_index` to `None`). No separate change needed for `clear_selection()`.

### Key types and their locations:

- **`ObjectInstance`** (Component): `artifacts/developer/src/game/types/objects.rs:55` - has `pub object_type: ObjectEnum`
- **`ObjectEnum`**: `artifacts/developer/src/shared/types.rs:316` - enum of all object types
- **`ObjectEnum::object_type()`**: `artifacts/developer/src/game/types/objects.rs:217` - returns `ObjectType` with `pub groupable: bool`
- **`ObjectType`**: `artifacts/developer/src/game/types/objects.rs:14` - has `pub groupable: bool`
- **`Selection`** (Resource): `artifacts/developer/src/shared/types.rs:138` - has `groups: Vec<SelectionGroup>`, `active_group_index: Option<usize>`
- **`Selection::build_from_entities()`**: `artifacts/developer/src/shared/types.rs:173` - takes `&[(Entity, ObjectEnum, bool)]`, builds groups, preserves active group type if possible
- **`Selected`** (Component): marker component in `artifacts/developer/src/shared/types.rs`

### Import needed in harness.rs:

The file already imports `ObjectInstance` at line 7: `use crate::game::types::objects::{ObjectInstance, StructureInstance}`. It also already imports `Selection` indirectly via the `use crate::types::*` at line 3 (which re-exports from `shared/types.rs`). No new imports needed.

### Why the tests fail without this fix:

In `tests/qa/automated_qa_ui_state_queries.rs`:
- **step_1** (line 17): Spawns Barracks, calls `set_selection(&[bk])`, sets `ObjectInterfaceState` to `BarracksMenu`, calls `test_app.step()`. During that step, `update_command_panel_state` (in `artifacts/developer/src/ui/command_panel.rs:325`) reads `selection.active_type()` which returns `None` (groups empty), so `active_is_structure` is `false`, the system falls through to the else branch which counts 0 selected units, and resets state to `Default`.
- **step_3** (line 96): Same pattern with DeploymentCenter and `DcBuildMenu`.

With the fix, `set_selection` will populate `Selection.groups` with a group for the structure entity, so `active_type()` will return `Some(ObjectEnum::Barracks)` or `Some(ObjectEnum::DeploymentCenter)`, and the system will correctly enter the structure menu branch and preserve the interface state.

### Verification commands:

```bash
cd artifacts/developer && cargo test --test qa automated_qa_ui_state_queries
cd artifacts/developer && cargo test --lib
```

## Dependencies

None. This is a test infrastructure fix that does not depend on any other planned tasks. The `Selection::build_from_entities()` method already exists and is well-tested with 8+ unit tests in `shared/types.rs`.
