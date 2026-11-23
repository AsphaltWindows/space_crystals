# fix_test_harness_selection_groups

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-core_systems_split_r2.md

## Task

Fix TestHarness::set_selection() in src/shared/testing/harness.rs to populate the Selection resource groups, not just the Selected ECS component. This is the root cause of 2 failing QA integration tests.

### Problem

set_selection() (harness.rs:264) only adds/removes the Selected component on entities. It does NOT update the Selection resource (groups field). When update_command_panel_state (ui/command_panel/state.rs:139) runs, it reads selection.active_type() which returns None because groups is empty. The system then falls through to the 'no units selected' branch and resets ObjectInterfaceState to Default.

### Fix

1. In set_selection(), after adding Selected components, also update the Selection resource.
2. In clear_selection(), ensure Selection resource groups is also cleared (it calls set_selection(&[]) so this should happen automatically).

### Verification

These 2 tests must pass after the fix:
- cargo test --test qa automated_qa_ui_state_queries::step_1_get_interface_state_barracks
- cargo test --test qa automated_qa_ui_state_queries::step_3_get_interface_state_dc_build_menu

Also verify no regressions: cargo test --lib && cargo test --test qa

## Technical Context

### File to modify

**artifacts/developer/src/shared/testing/harness.rs** — `set_selection()` method at line 264.

### Current implementation (line 264-282)

set_selection() removes Selected from all entities, then adds Selected to the specified entities. It never touches the Selection resource.

### Required changes

After adding/removing Selected components, add this logic:

1. For each entity in the input slice, query its ObjectInstance component to get object_type (ObjectEnum).
2. Look up groupable via `object_type.object_type().groupable` — the ObjectEnum::object_type() method returns an ObjectType struct with a `groupable: bool` field.
3. Build a Vec<(Entity, ObjectEnum, bool)> of (entity, object_type, groupable) tuples.
4. Get the Selection resource mutably: `world.resource_mut::<Selection>()`
5. Call `selection.build_from_entities(&tuples)` — this method (defined at shared/types.rs:173) handles all the grouping logic including groupable vs ungroupable, and sets active_group_index.

### Key types and locations

- **Selection** resource: `src/shared/types.rs:138` — fields: `groups: Vec<SelectionGroup>`, `active_group_index: Option<usize>`
- **SelectionGroup**: `src/shared/types.rs:129` — fields: `object_type: ObjectEnum`, `entities: Vec<Entity>`
- **Selection::build_from_entities()**: `src/shared/types.rs:173` — takes `&[(Entity, ObjectEnum, bool)]`, rebuilds groups, sets active_group_index
- **ObjectInstance**: `src/game/types/objects.rs:55` — field: `object_type: ObjectEnum`
- **ObjectEnum::object_type()**: `src/game/types/objects.rs:217` — returns `ObjectType` which has `groupable: bool` field
- **ObjectType**: `src/game/types/objects.rs:15` — struct with `groupable: bool` at field position 4

### Imports already available

All needed types are already imported in harness.rs:
- `use crate::types::*;` (line 3) brings in Selection, SelectionGroup, Selected, ObjectEnum via shared/types.rs
- `use crate::game::types::objects::{ObjectInstance, StructureInstance};` (line 7) brings in ObjectInstance

### Implementation pattern

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

    // Add Selected to the specified entities
    for &e in entities {
        if world.get_entity(e).is_ok() {
            world.entity_mut(e).insert(Selected);
        }
    }

    // Build group data for the Selection resource
    let mut group_data: Vec<(Entity, ObjectEnum, bool)> = Vec::new();
    for &e in entities {
        if let Some(obj) = world.get::<ObjectInstance>(e) {
            let groupable = obj.object_type.object_type().groupable;
            group_data.push((e, obj.object_type, groupable));
        }
    }

    // Update the Selection resource
    let mut selection = world.resource_mut::<Selection>();
    selection.build_from_entities(&group_data);
}
```

Note: world.get::<ObjectInstance>(e) returns Option<&ObjectInstance> without needing a query. This is the simplest approach for a test harness.

### Integration point

update_command_panel_state (ui/command_panel/state.rs:139) reads Res<Selection> and checks selection.groups.is_empty() at line 154. With the fix, groups will be populated and the ownership guard + interface state logic will work correctly.

## Dependencies

None — this is a standalone fix to the test harness. The Selection resource and build_from_entities() method already exist and are well-tested (see shared/types.rs tests starting at line 660+).
