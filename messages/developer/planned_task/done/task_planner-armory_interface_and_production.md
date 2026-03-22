# armory_interface_and_production

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-add_cults_armory.md

## Task

Implement the Armory's command panel interface, training production system, and eject mechanic.

### StructureMenuState
Add `StructureMenuState::ArmoryMenu` variant. Wire it up in:
- `update_command_panel_state` (ObjectEnum::CultsArmory → ArmoryMenu)
- The `get_structure_command_grid` match (or equivalent grid function)

### Command Panel Grid (3x3)
Following the design doc ObjectInterfaceState[Armory]:
- Q (0,0): Train Soldier — CommandIssuingTransition. Begins training a Soldier. Available only if StoredRecruits > 0 AND player has sufficient Space Crystals.
- W (0,1): Train Gunner — CommandIssuingTransition. Begins training a Gunner. Available only if StoredRecruits > 0 AND player has sufficient Space Crystals.
- E (0,2): Eject All — CommandIssuingTransition. Immediately ejects all stored Recruits from exit side. Available only if StoredRecruits > 0.
- C (2,2): Set Rally Point — StateOnlyTransition → AwaitingTarget[SetRallyPoint]. Left-click ground/object sets rally point.

### Right-click Resolution (Armory selected)
- Right-click Ground → SetRallyPoint to that location
- Right-click Object → SetRallyPoint to that object
(Same pattern as Barracks/HQ rally point)

### CommandButtonAction variants
Add new action variants for Armory commands:
- ArmoryTrainSoldier, ArmoryTrainGunner, ArmoryEjectAll
Wire into `execute_command_action` with crystal deduction for training.

### Training Production Tick System
Create `armory_training_tick_system`:
- If ArmoryState has active training (training_queue is Some):
  - Increment training_progress each frame
  - When training_progress reaches the training frames for the unit type:
    - Spawn the trained unit (Soldier or Gunner — use stub ObjectEnum variants if they don't exist yet, with placeholder type data similar to CultsRecruit)
    - Spawn at exit side (C side) of the Armory
    - Issue rally command if rally point is set (follow existing pattern from barracks/HQ)
    - Clear training state
- Note: Soldier and Gunner ObjectEnum variants likely don't exist yet. Create stubs: ObjectEnum::CultsSoldier, ObjectEnum::CultsGunner with placeholder stats and spawn functions.

### Eject System
Create `armory_eject_system`:
- When EjectAll is triggered, eject stored Recruits one at a time from exit side (C side) in rapid succession
- Use a cooldown/tick pattern similar to tunnel ejection (EjectionQueue pattern)
- Ejected Recruits return as normal field units (restore Visibility, place at exit side position)

### Info Panel
In the info panel section of command_panel.rs, show:
- StoredRecruits count: "Recruits: X/10"
- Training progress if active: "Training: [UnitName] X%"

### Availability Gating
- Train buttons greyed out when StoredRecruits == 0 or insufficient crystals
- Eject All greyed out when StoredRecruits == 0
- Follow existing `is_command_available` patterns

### Tests
- Train Soldier deducts crystal cost and consumes a stored Recruit
- Train Gunner works similarly
- Eject All returns all Recruits to the field
- Buttons unavailable when StoredRecruits is empty
- Rally point integration works

## Technical Context

### Files to Modify

#### 1. `artifacts/developer/src/shared/types.rs` (~line 334-340)
Add `CultsSoldier` and `CultsGunner` to `ObjectEnum` in the `// Cults Units` section after `CultsRecruit`:
```rust
// Cults Units
CultsRecruit,
CultsSoldier,
CultsGunner,
```

#### 2. `artifacts/developer/src/game/types/objects.rs` (~line 216)
Add `ObjectType` entries for `CultsSoldier` and `CultsGunner` in `object_type()`. Use placeholder stats similar to CultsRecruit:
```rust
ObjectEnum::CultsSoldier => ObjectType { name: "Soldier", size: (1, 1), destructible: true, sight_range: 5, groupable: true },
ObjectEnum::CultsGunner => ObjectType { name: "Gunner", size: (1, 1), destructible: true, sight_range: 6, groupable: true },
```
Also update `is_unit()` (~line 393) to include these new variants.

#### 3. `artifacts/developer/src/game/types/structures.rs`
- **Constants** in `cults_structure_stats` module (~line 468): `SOLDIER_TRAINING_COST`, `SOLDIER_TRAINING_FRAMES`, `GUNNER_TRAINING_COST`, `GUNNER_TRAINING_FRAMES` should already be defined by the `armory_structure` task. If not, add them:
  ```rust
  pub const SOLDIER_TRAINING_COST: u32 = 75;
  pub const SOLDIER_TRAINING_FRAMES: u32 = 160;
  pub const GUNNER_TRAINING_COST: u32 = 100;
  pub const GUNNER_TRAINING_FRAMES: u32 = 200;
  ```
- **`ArmoryState`** (after `RecruitmentCenterState`, ~line 528): Already defined by the `armory_structure` task with fields `stored_recruits: Vec<Entity>`, `training_queue: Option<ObjectEnum>`, `training_progress: u32`, `rally_point: Option<RallyTarget>`.
- Add a helper method `ArmoryState::training_cost(unit_type: &ObjectEnum) -> Option<u32>` to map `CultsSoldier`/`CultsGunner` to their costs.

#### 4. `artifacts/developer/src/ui/types.rs`

**StructureMenuState** (~line 220, after `RecruitmentCenterMenu`):
```rust
/// Armory selected — training + eject menu
ArmoryMenu,
```

**CommandButtonAction** (~line 311, before `RcCancel`):
```rust
/// Armory: Train a Soldier
ArmoryTrainSoldier,
/// Armory: Train a Gunner
ArmoryTrainGunner,
/// Armory: Eject all stored Recruits
ArmoryEjectAll,
```

**ArmoryEjectionQueue component** — NEW. Create an `ArmoryEjectionQueue` component similar to `EjectionQueue` (line 400-406):
```rust
#[derive(Component, Clone, Debug, Default)]
pub struct ArmoryEjectionQueue {
    pub queue: VecDeque<Entity>,
    pub cooldown: u32,
}
```
This is separate from tunnel's `EjectionQueue` since Armory ejects from its own entity, not the tunnel network.

#### 5. `artifacts/developer/src/ui/command_panel.rs`

**Grid layout** — `get_grid_slot_action()` (~line 42): Add after `StructureMenuState::RecruitmentCenterMenu` block (line 127):
```rust
StructureMenuState::ArmoryMenu => match (row, col) {
    (0, 0) => Some(CommandButtonAction::ArmoryTrainSoldier),
    (0, 1) => Some(CommandButtonAction::ArmoryTrainGunner),
    (0, 2) => Some(CommandButtonAction::ArmoryEjectAll),
    (2, 2) => Some(CommandButtonAction::SetRallyPoint),
    _ => None,
},
```
Note: No conditional `bk_has_queue` gating at the grid level — availability is handled in `grid_button_enabled_ext`.

**`update_command_panel_state()`** (~line 304):
- Add `Option<&ArmoryState>` to the `selected_structures` query (line 306). The destructure at line 346 must add an `armory_state` field.
- Add a new match arm after `ObjectEnum::RecruitmentCenter` (line ~428):
```rust
ObjectEnum::CultsArmory => {
    if armory_state.is_some() {
        let in_valid_state = matches!(*interface_state,
            ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu) |
            ObjectInterfaceState::AwaitingTarget(_)
        );
        if target_changed || !in_valid_state {
            *interface_state = ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu);
        }
    }
}
```

**`execute_command_action()`** (~line 1181):
- The function currently has NO access to `CultsPlayerResources`. Add a `cults_players: &mut Query<(&Player, &mut CultsPlayerResources)>` parameter. This also means updating BOTH call sites: the hotkey path (line 884) and the click path (line 1086).
- Alternatively, add an `armory_query: &mut Query<(&Owner, &mut ArmoryState)>` parameter instead of modifying `bk_hq_query`.
- Add new match arms:

```rust
CommandButtonAction::ArmoryTrainSoldier => {
    let Some(target_entity) = panel_target.entity else { return };
    if let Ok((owner, mut armory)) = armory_query.get_mut(target_entity) {
        if armory.stored_recruits.is_empty() || armory.training_queue.is_some() { return; }
        let cost = SOLDIER_TRAINING_COST as i32;
        if let Some(mut res) = find_cults_resources_mut(owner, cults_players) {
            if res.space_crystals >= cost {
                res.space_crystals -= cost;
                armory.stored_recruits.pop(); // consume one recruit
                armory.training_queue = Some(ObjectEnum::CultsSoldier);
                armory.training_progress = 0;
                interface_state.set_changed();
            }
        }
    }
}
CommandButtonAction::ArmoryTrainGunner => {
    // Same pattern as ArmoryTrainSoldier with GUNNER_TRAINING_COST and CultsGunner
}
CommandButtonAction::ArmoryEjectAll => {
    let Some(target_entity) = panel_target.entity else { return };
    if let Ok((_, mut armory)) = armory_query.get_mut(target_entity) {
        // Move all stored_recruits into the ArmoryEjectionQueue
        commands.entity(target_entity).insert(ArmoryEjectionQueue {
            queue: armory.stored_recruits.drain(..).collect(),
            cooldown: 0,
        });
        interface_state.set_changed();
    }
}
```

Add a `find_cults_resources_mut` helper (mirroring `find_syndicate_resources_mut` at line 2237):
```rust
fn find_cults_resources_mut<'a>(
    owner: &Owner,
    players: &'a mut Query<(&Player, &mut CultsPlayerResources)>,
) -> Option<Mut<'a, CultsPlayerResources>> {
    let player_number = owner.player_number()?;
    players.iter_mut()
        .find(|(p, _)| p.player_number == player_number)
        .map(|(_, res)| res)
}
```

**`grid_button_enabled_ext()`** (~line 2178):
Add availability checks for Armory buttons. Needs access to `ArmoryState` — either via query param or by piggybacking on existing params:
```rust
CommandButtonAction::ArmoryTrainSoldier => {
    target_entity
        .and_then(|e| armory_query.get(e).ok())
        .map(|a| !a.stored_recruits.is_empty() && a.training_queue.is_none())
        .unwrap_or(false)
        && cults_sc >= SOLDIER_TRAINING_COST as i32
}
CommandButtonAction::ArmoryTrainGunner => {
    // Same pattern with GUNNER_TRAINING_COST
}
CommandButtonAction::ArmoryEjectAll => {
    target_entity
        .and_then(|e| armory_query.get(e).ok())
        .map(|a| !a.stored_recruits.is_empty())
        .unwrap_or(false)
}
```

**`grid_button_label()`** (~line 2380): Add labels:
```rust
CommandButtonAction::ArmoryTrainSoldier => format!("[{}] Train\nSoldier", hotkey),
CommandButtonAction::ArmoryTrainGunner => format!("[{}] Train\nGunner", hotkey),
CommandButtonAction::ArmoryEjectAll => format!("[{}] Eject\nAll", hotkey),
```

**`is_unit_action()`**: ArmoryTrainSoldier, ArmoryTrainGunner, ArmoryEjectAll are NOT unit actions — they are structure actions. Return `false`.

**`right_click_cancel_target()`** (~line 1098): Add `RallyTargetKind::Armory` variant:
```rust
Some(RallyTargetKind::Armory) => Some(ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu)),
```

**`RallyTargetKind` enum** (~line 1135): Add `Armory` variant.

**Rally right-click resolve callback**: Where `resolve_rally_target` is called (~line 880 in hotkey handler, ~line 1082 in click handler), add logic to return `RallyTargetKind::Armory` when the active selection is a CultsArmory.

#### 6. `artifacts/developer/src/game/world/faction.rs`

**`production_rally_point_system()`** (~line 703):
- Add `StructureMenuState::ArmoryMenu` to the `is_production_menu` match (line 729-735).
- Add `mut armory_query: Query<(Entity, &mut ArmoryState), (With<Selected>, Without<BarracksState>, Without<HeadquartersState>, Without<SupplyTowerState>, Without<RecruitmentCenterState>)>` to parameters.
- Add a match arm for `StructureMenuState::ArmoryMenu` in the rally-setting logic (~line 765). ArmoryState uses `Option<RallyTarget>` (same as BarracksState), so copy the Barracks pattern directly:
```rust
ObjectInterfaceState::StructureMenu(StructureMenuState::ArmoryMenu) => {
    for (entity, mut armory_state) in &mut armory_query {
        armory_state.rally_point = Some(rally_target.clone());
        spawn_or_update_rally_marker(...);
    }
}
```

**`armory_training_tick_system()`** — NEW system:
```rust
pub fn armory_training_tick_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut armory_query: Query<(Entity, &Owner, &Transform, &StructureInstance, &mut ArmoryState)>,
    tiles: Query<(&GridPosition, &TilePreset), With<Tile>>,
    grid: Res<super::types::GridMap>,
    rally_targets: Query<(&Transform, &Owner), With<ObjectInstance>>,
    occupancy: Res<crate::game::units::types::OccupancyMap>,
) {
    for (entity, owner, transform, si, mut armory) in armory_query.iter_mut() {
        let Some(unit_type) = armory.training_queue else { continue; };
        let required_frames = match unit_type {
            ObjectEnum::CultsSoldier => SOLDIER_TRAINING_FRAMES,
            ObjectEnum::CultsGunner => GUNNER_TRAINING_FRAMES,
            _ => continue,
        };
        armory.training_progress += 1;
        if armory.training_progress >= required_frames {
            // Compute exit side (C) position using StructureInstance rotation
            let exit_pos = armory_exit_side_position(transform, si);
            let exit_grid = world_to_grid(exit_pos);
            // Spawn the trained unit
            let unit_entity = match unit_type {
                ObjectEnum::CultsSoldier => spawn_cults_soldier(&mut commands, &mut meshes, &mut materials, exit_grid.x, exit_grid.z, *owner),
                ObjectEnum::CultsGunner => spawn_cults_gunner(&mut commands, &mut meshes, &mut materials, exit_grid.x, exit_grid.z, *owner),
                _ => continue,
            };
            // Issue rally command (follow HQ pattern at faction.rs:570)
            issue_rally_command(&mut commands, unit_entity, &armory.rally_point, owner, exit_grid.x, exit_grid.z, &tiles, &grid, &rally_targets, &occupancy, &UnitBaseEnum::LightInfantry);
            // Clear training state
            armory.training_queue = None;
            armory.training_progress = 0;
        }
    }
}
```
Note: `issue_rally_command` (line 424) takes `&Option<RallyTarget>` — matches ArmoryState.rally_point type.

**`armory_eject_tick_system()`** — NEW system. Follow `ejection_tick_system` pattern (line 1994-2059) but for Armory:
```rust
pub fn armory_eject_tick_system(
    mut commands: Commands,
    mut armory_query: Query<(Entity, &Transform, &StructureInstance, &mut ArmoryEjectionQueue)>,
) {
    for (entity, transform, si, mut ejection_queue) in armory_query.iter_mut() {
        if ejection_queue.cooldown > 0 {
            ejection_queue.cooldown -= 1;
            continue;
        }
        if let Some(recruit_entity) = ejection_queue.queue.pop_front() {
            let exit_pos = armory_exit_side_position(transform, si);
            commands.entity(recruit_entity)
                .insert(Visibility::Inherited)
                .insert(Transform::from_translation(exit_pos))
                .insert(UnitCommand::Move(exit_pos + Vec3::new(0.0, 0.0, -2.0)));
            ejection_queue.cooldown = 8; // 8-frame spacing like tunnel ejection
        } else {
            // Queue empty — remove the component
            commands.entity(entity).remove::<ArmoryEjectionQueue>();
        }
    }
}
```

**Helper: `armory_exit_side_position()`** — computes Side C world position from Armory Transform and StructureInstance (rotation/flip). Follow `tunnel_side_world_position` pattern from `artifacts/developer/src/game/units/utils.rs`. For a 3x2 building, Side C is the short side opposite Side A.

#### 7. `artifacts/developer/src/game/world/mod.rs` (~line 109-123)

Register the new systems in the `FixedUpdate`/`DiagCategory::Construction` set:
```rust
faction::armory_training_tick_system,
faction::armory_eject_tick_system,
```

Also add `ArmoryMenu` to the `production_rally_point_system` in the `Update`/`DiagCategory::Faction` set (already registered, just needs the menu state added).

#### 8. `artifacts/developer/src/game/utils.rs`

Add stub `spawn_cults_soldier()` and `spawn_cults_gunner()` functions. Follow `spawn_cults_recruit()` (line 1043-1074) pattern but with:
- `ObjectEnum::CultsSoldier` / `ObjectEnum::CultsGunner`
- Placeholder mesh/material (similar colors but slightly different)
- Include `LocomotionChannel`, `OrientationChannel`, `Velocity` (unlike the original recruit spawn, which may be missing these — the `armory_enter_mechanic` task adds them)
- Placeholder stats (HP, movement speed, etc.)

#### 9. Info Panel (if applicable)

The info panel for structures is in `artifacts/developer/src/ui/hud.rs`. Search for existing structure info panels (e.g., HQ at ~line 1100+). Add an Armory info section showing:
- "Recruits: X/10" (stored_recruits.len() / ARMORY_INTERNAL_RECRUIT_CAPACITY)
- "Training: [Soldier|Gunner] X%" when training_queue.is_some()

### Key Patterns to Follow

- **HQ production tick** (`faction.rs:495`): Frame-counting production → spawn → rally. The Armory is simpler (no queue, one at a time, no tunnel network).
- **Tunnel ejection** (`faction.rs:1994`): `EjectionQueue` component with 8-frame cooldown. Armory eject uses the same pattern but from stored_recruits (already on field as hidden entities, not in tunnel network).
- **RC cancel** (`command_panel.rs:1371`): Simple cancel pattern for single-item production.
- **Barracks rally** (`faction.rs:766-771`): Direct `RallyTarget` assignment pattern.
- **`find_syndicate_resources_mut`** (`command_panel.rs:2237`): Helper pattern for faction-specific resource lookup. Create equivalent for Cults.
- **Grid button availability**: `grid_button_enabled_ext()` (line 2178) handles complex availability checks per-action.

### Bevy ECS Considerations

- **Query conflict avoidance**: The `execute_command_action` function already has `bk_hq_query` that queries BarracksState/HeadquartersState/RecruitmentCenterState. ArmoryState should use a SEPARATE query (`armory_query`) to avoid borrowing conflicts.
- **System ordering**: `armory_training_tick_system` and `armory_eject_tick_system` run in `FixedUpdate` (like other production systems). No special ordering needed relative to each other.
- **`update_command_panel_state` query expansion**: Adding `Option<&ArmoryState>` to `selected_structures` requires updating the destructure at line 346. All 7 fields become 8.

### Important Notes

- **CultsPlayerResources** is NOT currently accessible in `execute_command_action` or `command_panel_hotkeys`. You MUST add a new query parameter `cults_players: Query<(&Player, &mut CultsPlayerResources)>` to `execute_command_action` and thread it through both call sites.
- **`bk_has_queue`** parameter in `get_grid_slot_action` is already overloaded for multiple cancel buttons. For Armory, training availability is NOT gated at the grid level (all 4 buttons always appear) — gating is done in `grid_button_enabled_ext`.
- **Trained unit Entity**: When training completes, a stored recruit entity was already consumed (popped from stored_recruits) and hidden. The trained unit is a NEW entity spawned fresh. The consumed recruit entity should be despawned.
- **Armory rally_point type**: The `armory_structure` task defines it as `Option<RallyTarget>` (matching Barracks/HQ), NOT `Option<Vec3>` (like RC). This means you can directly use the Barracks rally pattern in `production_rally_point_system`.

## Dependencies

- **`armory_structure` (planned_task)**: Must land first. Defines `ObjectEnum::CultsArmory`, `ArmoryState` component, `ARMORY_INTERNAL_RECRUIT_CAPACITY`, `SOLDIER_TRAINING_COST`, `SOLDIER_TRAINING_FRAMES`, `GUNNER_TRAINING_COST`, `GUNNER_TRAINING_FRAMES`, and `spawn_cults_armory()`. This task references all of these.
- **`armory_enter_mechanic` (planned_task)**: Should land first or concurrently. Defines `UnitCommand::EnterArmory`, `EnteringArmoryBehavior`, and the right-click/behavior systems that populate `ArmoryState.stored_recruits`. Without this, the Armory will have no recruits to train or eject. Also adds `LocomotionChannel`/`OrientationChannel`/`Velocity` to `spawn_cults_recruit()`, which the eject system needs.
