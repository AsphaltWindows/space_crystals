# pointer_display_rendering

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Files to Create/Change

1. **`artifacts/developer/src/ui/pointer.rs`** (NEW FILE) — Contains the rendering system and pointer entity marker component.

   Create a new module for clean separation. This keeps the rendering logic out of the already-large `command_panel.rs`.

   **Marker component:**
   ```rust
   #[derive(Component)]
   pub struct PointerIndicator;
   ```

   **Spawn function** (called from `setup_hud` or as an `OnEnter(AppState::InGame)` system):
   ```rust
   pub fn spawn_pointer_indicator(mut commands: Commands, ui_cam: Res<UiCameraEntity>) {
       commands.spawn((
           Node {
               width: Val::Px(16.0),
               height: Val::Px(16.0),
               position_type: PositionType::Absolute,
               ..default()
           },
           BackgroundColor(Color::srgba(0.0, 1.0, 0.0, 0.7)),
           UiTargetCamera(ui_cam.0),
           PointerIndicator,
           DespawnOnExit(AppState::InGame),
       ));
   }
   ```
   Uses `UiTargetCamera(ui_cam.0)` — same pattern as the resource bar (`hud.rs:50`) and HUD panel (`hud.rs:79`). The `UiCameraEntity` resource is set up by `setup_hud` (`hud.rs:33`), so the pointer spawn system must run after it.

   **Main system:**
   ```rust
   pub fn update_pointer_display(
       pointer_type: Res<PointerDisplayType>,
       interface_state: Res<ObjectInterfaceState>,
       cursor_over_ui: Res<CursorOverUi>,
       windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
       mut indicator: Query<(&mut Node, &mut BackgroundColor, &mut Visibility), With<PointerIndicator>>,
   ) {
       // 1. Get cursor screen position from window.cursor_position()
       // 2. If placement mode or cursor_over_ui.0: set Visibility::Hidden, return
       // 3. Set Visibility::Inherited, update Node left/top to cursor position (with offset so indicator doesn't block clicks)
       // 4. Set BackgroundColor based on PointerDisplayType variant
   }
   ```

2. **`artifacts/developer/src/ui/mod.rs`** — Register the new module and systems.

   - Add `mod pointer;` after `mod command_panel;` (line 8)
   - Add the spawn system as an `OnEnter(AppState::InGame)` system, ordered **after** `hud::setup_hud` (since it needs `UiCameraEntity`):
     ```rust
     .add_systems(OnEnter(AppState::InGame),
         pointer::spawn_pointer_indicator
             .after(hud::setup_hud)
             .after(crate::game::world::faction::setup_player_resources))
     ```
   - Add `update_pointer_display` to the Update systems list, ordered after `resolve_pointer_display_type`:
     ```rust
     pointer::update_pointer_display.after(command_panel::resolve_pointer_display_type),
     ```
     Add this inside the existing `.add_systems(Update, (...).in_set(DiagCategory::UiHud))` block (line 31-44).

3. **`artifacts/developer/src/ui/types.rs`** — The `PointerDisplayType` enum is added by the sibling task. If it's not yet present when you implement this task, you need to add it yourself (see Dependencies). The enum definition:
   ```rust
   #[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
   pub enum PointerDisplayType {
       #[default]
       Inactive,
       Move,
       Attack,
       AttackGround,
       Patrol,
       GatherResources,
       ReturnResources,
       Enter,
   }
   ```

### Color Mapping (Recommended)

Create a helper method on `PointerDisplayType`:
```rust
impl PointerDisplayType {
    pub fn indicator_color(&self) -> Color {
        match self {
            Self::Inactive => Color::srgba(0.5, 0.5, 0.5, 0.3),       // muted grey, mostly transparent
            Self::Move => Color::srgba(0.2, 0.9, 0.2, 0.7),           // green
            Self::Attack => Color::srgba(0.9, 0.15, 0.15, 0.7),       // red
            Self::AttackGround => Color::srgba(0.9, 0.3, 0.1, 0.7),   // dark orange-red
            Self::Patrol => Color::srgba(0.9, 0.7, 0.1, 0.7),         // orange-yellow
            Self::GatherResources => Color::srgba(0.9, 0.8, 0.1, 0.7), // gold
            Self::ReturnResources => Color::srgba(0.8, 0.7, 0.2, 0.7), // darker gold
            Self::Enter => Color::srgba(0.1, 0.7, 0.9, 0.7),          // cyan
        }
    }
}
```
Place this in `pointer.rs` (not types.rs) to keep the rendering detail out of the shared types module.

### Existing Patterns to Follow

- **UI entity spawning**: See `setup_hud` in `hud.rs:15-80`. Uses `Node` with `PositionType::Absolute`, `BackgroundColor`, `UiTargetCamera`, `DespawnOnExit(AppState::InGame)`.
- **Cursor position**: `window.cursor_position()` returns `Option<Vec2>` in screen/logical coordinates. Used extensively — see `command_panel.rs:189`, `faction.rs:618`, `resources.rs:120`. Position `(0,0)` is top-left of the window.
- **Node positioning**: For absolute positioning, set `left: Val::Px(x)` and `top: Val::Px(y)` on the `Node`. Apply a small offset (e.g., +12px x, +12px y) so the indicator appears to the bottom-right of the cursor and doesn't intercept mouse clicks on entities below.
- **Visibility control**: Use `Visibility::Hidden` / `Visibility::Inherited`. PlacementGhost uses this same pattern (`faction.rs:1137`).
- **CursorOverUi**: `types.rs:22` — `CursorOverUi(pub bool)`. When true, the pointer indicator should be hidden (no game action possible over UI).
- **System ordering in mod.rs**: All HUD systems run in `DiagCategory::UiHud` set (mod.rs:44). The `resolve_pointer_display_type` system (from sibling task) is registered `.after(update_command_panel_state)`. Your system should be `.after(resolve_pointer_display_type)`.

### Bevy 0.17 Considerations

- **No custom cursor icons without assets**: Bevy 0.17's `Window::cursor_options` supports `CursorGrabMode` and visibility, but custom cursor icons require image assets. The colored UI overlay approach (small `Node` with `BackgroundColor`) is the simplest path that doesn't require any image assets.
- **UI node z-ordering**: The pointer indicator spawned as a root UI node with `UiTargetCamera` will render on top of the game world. It renders in spawn order relative to other root UI nodes, but since it's a small overlay it shouldn't conflict with HUD panels.
- **Frame-by-frame tracking**: The system runs every frame and updates `Node.left`/`Node.top` from `cursor_position()`. No interpolation needed.

### Testing Approach

Unit tests should verify:
1. `PointerDisplayType::indicator_color()` returns distinct colors for each variant.
2. The `update_pointer_display` system hides the indicator during placement mode.
3. The system hides the indicator when `CursorOverUi` is true.
4. The system correctly maps `PointerDisplayType` to `BackgroundColor`.

For ECS-level tests, use `TestApp` from `shared/testing/test_app.rs` — but the color mapping function can be tested as a pure function without ECS setup.

## Dependencies

- **Sibling task `pointer_display_type_resolution`** (planned_task: `task_planner-pointer_display_type_resolution`): This task defines the `PointerDisplayType` enum in `ui/types.rs`, initializes it as a resource in `ui/mod.rs`, and creates the `resolve_pointer_display_type` system in `command_panel.rs`. The rendering task reads the `PointerDisplayType` resource that the resolution task writes. If the resolution task has not been implemented yet, you must add the `PointerDisplayType` enum and resource initialization yourself, then the resolution system will be layered on later.
- **`setup_hud` system** (`ui/hud.rs:15`): Must run before `spawn_pointer_indicator` because it creates the `UiCameraEntity` resource that the pointer entity needs for `UiTargetCamera`. Enforce with `.after(hud::setup_hud)` on the OnEnter system.
- **`CursorOverUi` resource** (`ui/types.rs:22`): Already initialized by `HudPlugin`. Read each frame to decide visibility.
