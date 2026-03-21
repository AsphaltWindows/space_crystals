# supply-tower-placement-attach-chopper

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-supply-tower-interface.md

## Task

Fix the Supply Tower placement code so the free Supply Chopper spawned on placement is properly attached to the tower.

**Current state**: In `faction.rs` (around line 1410-1425), when a SupplyTower is placed via the DeploymentCenter, `spawn_supply_tower()` creates the tower with `SupplyTowerState::default()` (attached_chopper: None) and `spawn_supply_chopper()` creates the chopper with `SupplyChopperState::default()` (attached_tower: None). The two entities are never linked.

**Required fix**: After spawning both entities, set:
- `SupplyTowerState.attached_chopper = Some(chopper_entity)` on the tower
- `SupplyChopperState.attached_tower = Some(tower_entity)` on the chopper

The tower entity is returned by `spawn_supply_tower()` and the chopper entity by `spawn_supply_chopper()` — both return Entity. Use `commands.entity(tower_entity).insert()` or equivalent post-spawn mutation.

**Why this matters**: Without this fix, the S (Schedule Deliveries) button is always disabled because the availability check (`st.attached_chopper.is_some()` in command_panel.rs ~line 2104) always returns false. The design spec says 'one free Supply Chopper spawns and auto-attaches' on placement.

**Tests**: Add a test verifying that after the placement code runs, the tower's `attached_chopper` references the chopper entity and the chopper's `attached_tower` references the tower entity.

## Technical Context

### Files to Modify

1. **`artifacts/developer/src/game/world/faction.rs`** (lines 1410-1425) — The `ObjectEnum::SupplyTower` branch in the building placement system.

### What to Change

At lines 1410-1425, the placement code currently calls `spawn_supply_tower()` and `spawn_supply_chopper()` but discards the returned Entity IDs. The fix:

1. **Capture both entity IDs** — both `spawn_supply_tower()` (line 1411) and `spawn_supply_chopper()` (line 1420) return `Entity`.
2. **Link them with `commands.entity().insert()`** — After both spawns, overwrite the default state components:
   ```rust
   let tower_entity = spawn_supply_tower(...);
   // ... existing spawn_side_offset + spawn_supply_chopper code ...
   let chopper_entity = spawn_supply_chopper(...);
   commands.entity(tower_entity).insert(SupplyTowerState { attached_chopper: Some(chopper_entity), ..default() });
   commands.entity(chopper_entity).insert(SupplyChopperState { attached_tower: Some(tower_entity), ..default() });
   ```

### Key Types

- **`SupplyTowerState`** (`artifacts/developer/src/game/types/structures.rs`:314) — Component with `attached_chopper: Option<Entity>`, plus build queue, landed_chopper, scheduled_sds, rally_point fields. `Default` sets all to None/empty.
- **`SupplyChopperState`** (`artifacts/developer/src/game/types/structures.rs`:378) — Component with `attached_tower: Option<Entity>` and `carried_supplies: u32`. `Default` sets tower to None, supplies to 0.
- Both are `#[derive(Component)]`.

### Established Pattern

The `commands.entity(e).insert(Component)` pattern is used extensively in faction.rs (e.g., lines 331, 346, 352, 472, 1957, 1969). Using `.insert()` on an entity that already has the component replaces it — this is safe and idiomatic.

### Spawn Functions

- `spawn_supply_tower()` (`artifacts/developer/src/game/utils.rs`:817) — returns `Entity`, inserts `SupplyTowerState::default()` at line 863.
- `spawn_supply_chopper()` (`artifacts/developer/src/game/utils.rs`:872) — returns `Entity`, inserts `SupplyChopperState::default()` at line 922.

### Test Guidance

Add a test in the `#[cfg(test)] mod tests` at the bottom of faction.rs (line 1992). Pattern to follow: use `run_system_once` with a closure that calls both spawn functions, then query for the components:

```rust
#[test]
fn supply_tower_placement_links_chopper() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<Assets<Mesh>>();
    app.init_resource::<Assets<StandardMaterial>>();

    let (tower_entity, chopper_entity) = app.world_mut().run_system_once(
        |mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>| {
            let owner = Owner::player(1);
            let tower = spawn_supply_tower(&mut commands, &mut meshes, &mut materials, 32, 32, owner, StructureRotation::R0, false, false);
            let chopper = spawn_supply_chopper(&mut commands, &mut meshes, &mut materials, 33, 32, owner);
            commands.entity(tower).insert(SupplyTowerState { attached_chopper: Some(chopper), ..default() });
            commands.entity(chopper).insert(SupplyChopperState { attached_tower: Some(tower), ..default() });
            (tower, chopper)
        },
    ).unwrap();

    app.world_mut().flush();

    let tower_state = app.world().entity(tower_entity).get::<SupplyTowerState>().unwrap();
    assert_eq!(tower_state.attached_chopper, Some(chopper_entity));

    let chopper_state = app.world().entity(chopper_entity).get::<SupplyChopperState>().unwrap();
    assert_eq!(chopper_state.attached_tower, Some(tower_entity));
}
```

Note: The test harness (`shared/testing/harness.rs`:167) also spawns SupplyTower but does NOT spawn a chopper — the harness is for general test setup and doesn't need updating for this task.

### Note on Production-Spawned Choppers

The production system at faction.rs:1948 also spawns choppers via `spawn_supply_chopper()` without linking them. That is a separate concern (produced choppers are NOT auto-attached per design — only the initial free chopper is).

## Dependencies

None. This is a standalone bug fix to existing placement code. The `spawn_supply_tower()` and `spawn_supply_chopper()` functions already exist and return Entity.
