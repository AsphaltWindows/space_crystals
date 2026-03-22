# camera_starting_position

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-camera_starting_position.md

## Task

Add a startup system that centers the camera on the local player's primary structure at game start.

**Context:**
- Camera spawns at (0, 40, 25) looking at origin in `setup()` (main.rs ~line 71)
- GDO's Deployment Center spawns at grid (30, 30) in `setup_gdo_game_start` (faction.rs ~line 117-122)
- Syndicate's Tunnel spawns at grid (40, 40) in `setup_syndicate_game_start` (faction.rs ~line 145-152)
- `LocalPlayer` resource (types.rs) identifies which player is local (player 0)
- `SelectedFaction` resource determines which faction the local player chose
- Camera snap centering pattern already exists: set camera Transform x/z to target position, keep y=40, adjust z with offset formula (cam.y * 25.0 / 40.0)

**Implementation:**
1. Create a new startup system `center_camera_on_start` that runs in Startup schedule AFTER all `setup_*_game_start` functions
2. Query the local player's owner ID from `LocalPlayer` resource
3. Query for the primary structure: ObjectEnum::DeploymentCenter or ObjectEnum::Tunnel owned by the local player (use Owner component match)
4. Read that entity's Transform position
5. Set the MainCamera Transform to center on that position using the same snap formula used elsewhere
6. Register the system in mod.rs Startup schedule with appropriate ordering constraint

## Technical Context

### Files to Modify

1. **`src/game/world/faction.rs`** — Add the new `center_camera_on_start` system here (alongside other game-start functions).

2. **`src/game/world/mod.rs`** (lines 68-75) — Register the new system in `FactionPlugin::build()` under the `OnEnter(AppState::InGame)` schedule, with `.after()` constraints on all setup functions.

### Key Types and Resources

- **`LocalPlayer(pub u8)`** — Resource in `src/shared/types.rs:18`. The `.0` field is the player number (always 0 for local).
- **`SelectedFaction(pub FactionEnum)`** — Resource in `src/shared/types.rs:24`. Determines which faction the local player chose.
- **`FactionEnum`** — Enum in `src/shared/types.rs:275` with variants: `GlobalDefenseOrdinance`, `TheSyndicate`, `TheCults`, `Colonists`.
- **`Owner(pub Option<u8>)`** — Component in `src/shared/types.rs:32`. Use `Owner::player(id)` to create. Compare with `Owner::player(local_player.0)`.
- **`ObjectInstance`** — Component in `src/game/types/objects.rs:55`. Has `object_type: ObjectEnum` field.
- **`ObjectEnum::DeploymentCenter`** — GDO's primary structure.
- **`ObjectEnum::Tunnel`** — Syndicate's primary structure.
- **`MainCamera`** — Marker component in `src/shared/types.rs:14`.

### Camera Snap Pattern (copy from existing code)

The camera snap formula is used in two places — follow this exact pattern:

From `src/game/world/resources.rs:738-747` (control group double-tap):
```rust
if let Ok(mut cam_transform) = camera_query.single_mut() {
    let z_offset = cam_transform.translation.y * 25.0 / 40.0;
    cam_transform.translation.x = target.x;
    cam_transform.translation.z = target.z + z_offset;
}
```

From `src/ui/hud.rs:1155-1157` (portrait alt-click):
```rust
let z_offset = cam_transform.translation.y * 25.0 / 40.0;
cam_transform.translation.x = target_transform.translation.x;
cam_transform.translation.z = target_transform.translation.z + z_offset;
```

### System Signature

```rust
fn center_camera_on_start(
    local_player: Res<LocalPlayer>,
    selected_faction: Res<SelectedFaction>,
    structures: Query<(&ObjectInstance, &Owner, &Transform), Without<MainCamera>>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
) {
    // Determine primary structure type based on faction
    let primary_type = match selected_faction.0 {
        FactionEnum::GlobalDefenseOrdinance => ObjectEnum::DeploymentCenter,
        FactionEnum::TheSyndicate => ObjectEnum::Tunnel,
        _ => return, // TheCults and Colonists not yet implemented
    };
    let local_owner = Owner::player(local_player.0);
    
    // Find the primary structure owned by the local player
    for (obj, owner, transform) in structures.iter() {
        if obj.object_type == primary_type && *owner == local_owner {
            if let Ok(mut cam_transform) = camera_query.single_mut() {
                let z_offset = cam_transform.translation.y * 25.0 / 40.0;
                cam_transform.translation.x = transform.translation.x;
                cam_transform.translation.z = transform.translation.z + z_offset;
            }
            return;
        }
    }
}
```

### Registration in mod.rs

In `FactionPlugin::build()`, add to the `OnEnter(AppState::InGame)` block (line 68-75):
```rust
faction::center_camera_on_start
    .after(faction::setup_gdo_game_start)
    .after(faction::setup_syndicate_game_start)
    .after(faction::setup_cults_game_start)
    .after(faction::setup_colonists_game_start),
```

### Important Notes

- The system uses `OnEnter(AppState::InGame)` (NOT `Startup`), matching the existing game-start systems registration pattern in mod.rs lines 68-75.
- `SelectedFaction` and `LocalPlayer` are both already available as resources when OnEnter(InGame) runs.
- The `Without<MainCamera>` on the structures query and `With<MainCamera>` on the camera query disambiguate the Transform access (required by Bevy's query conflict rules).
- `ObjectEnum` must derive or implement `PartialEq` for the comparison — check it does (it's used in matches throughout the codebase, so it should).
- For TheCults and Colonists factions, the system can safely return early (no-op) until their primary structures are defined.

## Dependencies

None — this is a standalone startup system. It depends only on existing resources (`LocalPlayer`, `SelectedFaction`) and entity spawning (`setup_gdo_game_start`, `setup_syndicate_game_start`) which are all guaranteed to exist before this system runs via `.after()` ordering.
