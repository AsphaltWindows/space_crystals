# supply-tower-placement-attach-chopper

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-supply-tower-interface.md

## Task

Fix the Supply Tower placement code so the free Supply Chopper spawned on placement is properly attached to the tower.

**Current state**: In `faction.rs` (around line 1410-1425), when a SupplyTower is placed via the DeploymentCenter, `spawn_supply_tower()` creates the tower with `SupplyTowerState::default()` (attached_chopper: None) and `spawn_supply_chopper()` creates the chopper with `SupplyChopperState::default()` (attached_tower: None). The two entities are never linked.

**Required fix**: After spawning both entities, set:
- `SupplyTowerState.attached_chopper = Some(chopper_entity)` on the tower
- `SupplyChopperState.attached_tower = Some(tower_entity)` on the chopper

The tower entity is returned by `spawn_supply_tower()` and the chopper entity by `spawn_supply_chopper()` — both return Entity. Use `commands.entity(tower_entity).entry::<SupplyTowerState>().and_modify(|st| st.attached_chopper = Some(chopper_entity))` or equivalent post-spawn mutation.

**Why this matters**: Without this fix, the S (Schedule Deliveries) button is always disabled because the availability check (`st.attached_chopper.is_some()` in command_panel.rs ~line 2104) always returns false. The design spec says 'one free Supply Chopper spawns and auto-attaches' on placement.

**Files to modify**: `artifacts/developer/src/game/world/faction.rs` (the SupplyTower placement branch in the building placement system, around line 1410-1425).

**Tests**: Add a test verifying that after the placement code runs, the tower's `attached_chopper` references the chopper entity and the chopper's `attached_tower` references the tower entity.
