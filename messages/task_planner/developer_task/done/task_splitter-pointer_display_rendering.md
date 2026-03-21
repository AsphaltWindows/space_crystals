# pointer_display_rendering

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-pointer_display_types.md

## Task

Implement the visual rendering system that updates the cursor appearance based on the PointerDisplayType resource.

### What to implement:

1. **Pointer visual entity** — a small UI element or overlay that follows the cursor position and displays the current pointer type. Options:
   - A colored shape/icon near the cursor (simplest: a small colored square or circle)
   - Or change cursor color/shape via Bevy's window cursor API

2. **update_pointer_display system** that runs after `resolve_pointer_display_type`:
   - Reads the `PointerDisplayType` resource
   - Updates the pointer visual entity's appearance:
     - **Inactive**: Muted/grey/transparent appearance
     - **Move**: Default/white/green cursor indicator
     - **Attack**: Red cursor indicator
     - **AttackGround**: Red cursor indicator (same as Attack or slightly different)
     - **Patrol**: Orange/yellow cursor indicator
     - **GatherResources**: Yellow/gold cursor indicator
     - **ReturnResources**: Yellow/gold cursor indicator (same family as Gather)
     - **Enter**: Cyan/blue cursor indicator
   - When ObjectInterfaceState is placement mode: hide the pointer visual entirely (ghost preview replaces it)
   - Position tracks cursor screen position each frame

3. **Spawn/despawn**: The pointer visual entity should be spawned once at startup (or lazily on first need). Despawn is not needed (it persists).

4. **Integration**: Register the system in the UI plugin, ordered after `resolve_pointer_display_type`.

### Key references:
- PointerDisplayType resource: `ui/types.rs` (added by sibling task pointer_display_type_resolution)
- ObjectInterfaceState::is_placement_mode(): `ui/types.rs` line 164
- CursorOverUi: `ui/types.rs` line 22 — consider hiding pointer when cursor is over UI
- Window cursor position: Bevy 0.17 `Window::cursor_position()`
- Existing UI plugin: `ui/mod.rs`
