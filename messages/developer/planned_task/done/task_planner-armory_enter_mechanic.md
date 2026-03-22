# armory_enter_mechanic

## Metadata
- **From**: task_planner
- **To**: developer

## Content

## Parent Feature

task_splitter-add_cults_armory.md

## Task

Implement the mechanic for Recruits to enter the Cults Armory building.

### Recruit Enter Command for Armory
When a Recruit is ordered to enter an Armory (via right-click on own Armory or explicit Enter command):
1. The Recruit walks to the entrance side (A side — one of the short ends of the 3x2 footprint)
2. Upon reaching the entrance, the Recruit entity is stored in the Armory's `stored_recruits` list
3. The Recruit entity should be hidden (Visibility::Hidden or similar pattern used by tunnel enter)
4. StoredRecruits count increments (visible in info panel)
5. If the Armory already has 10 stored Recruits (ARMORY_INTERNAL_RECRUIT_CAPACITY), the enter command should be rejected/unavailable

### Right-click Resolution
Add to the right-click resolution system (core.rs right_click_move_command):
- Recruit right-clicks own CultsArmory → Enter command (similar to how units enter tunnels)
- Only if stored_recruits.len() < ARMORY_INTERNAL_RECRUIT_CAPACITY

### Behavior System
Create an `entering_armory_behavior_system` (or reuse/extend existing entering behavior patterns):
- Move toward the entrance side position of the Armory
- On arrival: remove from field, add to ArmoryState.stored_recruits, hide entity
- Follow the same patterns as EnteringTunnelBehavior but target the Armory entrance side

### Tests
- Recruit enters Armory and is added to stored_recruits
- StoredRecruits caps at 10 (11th Recruit cannot enter)
- Non-Recruit units cannot enter the Armory

## Technical Context

### New Command Variant

Add `UnitCommand::EnterArmory(Entity)` to the `UnitCommand` enum in `artifacts/developer/src/game/units/types/state/commands.rs` (line 37, after `ConstructBuilding`). This needs a SEPARATE variant from `Enter(Entity)` because `Enter` is gated by `is_syndicate` in `is_available()` (line 65). `EnterArmory` should be gated to Cults units: add `UnitCommand::EnterArmory(_) => true` to `is_available()` (UI will handle visibility gating).

Also add the `EnterArmory` variant to:
- `command_has_indicator()` in `artifacts/developer/src/game/units/types/types.rs` (line 35) — add alongside `Enter(_)`
- `command_indicator_color()` in same file (line 61) — same color as `Enter`
- `command_state_sync_system` in `artifacts/developer/src/game/units/systems/commands.rs` (line 209) — map to `CommandType::Enter` (or a new `CommandType::EnterArmory` if preferred, but reusing `Enter` is simpler)

### New Behavior Marker Component

Create `EnteringArmoryBehavior` in `artifacts/developer/src/game/units/types/state/behavior.rs` (after `EnteringTunnelBehavior`, line ~162). Follow the exact `EnteringTunnelBehavior` pattern:

```rust
#[derive(Component, Clone, Debug)]
pub struct EnteringArmoryBehavior {
    pub target_armory: Entity,
    pub path: Vec<Vec3>,
    pub path_index: usize,
}

impl EnteringArmoryBehavior {
    pub fn new(target_armory: Entity) -> Self {
        Self { target_armory, path: Vec::new(), path_index: 0 }
    }
}
```

### Dispatch System: `enter_armory_dispatch_system`

Create in `artifacts/developer/src/game/units/systems/behaviors.rs` (after `enter_command_dispatch_system`, line ~513). Mirror the tunnel dispatch pattern:

```rust
pub fn enter_armory_dispatch_system(
    mut commands: Commands,
    mut units: Query<
        (Entity, &mut UnitCommand, &ObjectInstance, &Owner),
        (With<Unit>, Without<EnteringArmoryBehavior>),
    >,
    armories: Query<(&ArmoryState, &Owner), With<ArmoryState>>,
) {
    for (entity, mut command, obj, unit_owner) in &mut units {
        let armory_entity = match &*command {
            UnitCommand::EnterArmory(ae) => *ae,
            _ => continue,
        };
        // Only CultsRecruit can enter
        if obj.object_type != ObjectEnum::CultsRecruit {
            *command = UnitCommand::Idle;
            continue;
        }
        // Validate armory exists, is owned by same player, and has capacity
        let (armory_state, armory_owner) = match armories.get(armory_entity) {
            Ok(r) => r,
            Err(_) => { *command = UnitCommand::Idle; continue; }
        };
        if armory_owner.player_number() != unit_owner.player_number() {
            *command = UnitCommand::Idle;
            continue;
        }
        if armory_state.stored_recruits.len() >= ARMORY_INTERNAL_RECRUIT_CAPACITY {
            *command = UnitCommand::Idle;
            continue;
        }
        commands.entity(entity).insert(EnteringArmoryBehavior::new(armory_entity));
    }
}
```

Import `ArmoryState` from `crate::game::types::structures` and `ARMORY_INTERNAL_RECRUIT_CAPACITY` from `crate::game::types::structures::cults_structure_stats`.

### Behavior System: `entering_armory_behavior_system`

Create in `artifacts/developer/src/game/units/systems/behaviors.rs` (after `entering_tunnel_behavior_system`, line ~572). Mirror the tunnel behavior system closely:

```rust
const ARMORY_ARRIVAL_THRESHOLD: f32 = 0.5;

pub fn entering_armory_behavior_system(
    mut commands: Commands,
    mut units: Query<(
        Entity,
        &Transform,
        &mut EnteringArmoryBehavior,
        &mut LocomotionChannel,
        &mut OrientationChannel,
        &mut Visibility,
    )>,
    mut armories: Query<(&Transform, &mut ArmoryState), With<ArmoryState>>,
) {
    for (entity, transform, entering, mut locomotion, mut orientation, mut visibility) in &mut units {
        let (armory_transform, mut armory_state) = match armories.get_mut(entering.target_armory) {
            Ok(r) => r,
            Err(_) => {
                *locomotion = LocomotionChannel::Stopping;
                *orientation = OrientationChannel::Maintaining;
                commands.entity(entity).remove::<EnteringArmoryBehavior>();
                continue;
            }
        };
        // Re-check capacity (may have filled while walking)
        if armory_state.stored_recruits.len() >= ARMORY_INTERNAL_RECRUIT_CAPACITY {
            *locomotion = LocomotionChannel::Stopping;
            *orientation = OrientationChannel::Maintaining;
            commands.entity(entity).remove::<EnteringArmoryBehavior>();
            commands.entity(entity).insert(UnitCommand::Idle);
            continue;
        }
        // Entrance is Side A (approximated as armory center, like tunnel)
        let entrance_pos = armory_transform.translation;
        let distance = Vec3::new(
            transform.translation.x - entrance_pos.x, 0.0,
            transform.translation.z - entrance_pos.z
        ).length();

        if distance < ARMORY_ARRIVAL_THRESHOLD {
            armory_state.stored_recruits.push(entity);
            *visibility = Visibility::Hidden;
            *locomotion = LocomotionChannel::Stationary;
            *orientation = OrientationChannel::Maintaining;
            commands.entity(entity).insert(Velocity(Vec3::ZERO));
            commands.entity(entity).remove::<EnteringArmoryBehavior>();
        } else {
            *locomotion = LocomotionChannel::Moving(vec![entrance_pos]);
            *orientation = OrientationChannel::Turning(entrance_pos);
        }
    }
}
```

Key difference from tunnel enter: instead of inserting `InTunnelNetwork`, the entity is pushed into `armory_state.stored_recruits` and hidden. It is NOT despawned (matches tunnel pattern — entity stays alive but hidden).

### Right-click Resolution

Modify `right_click_move_command` in `artifacts/developer/src/game/units/systems/core.rs`:

1. **Add `Option<&ArmoryState>` to `target_info` query** (line 249). Current query:
   ```rust
   target_info: Query<(Option<&SupplyDeliveryStation>, Option<&SupplyTowerState>, &Owner, Option<&SpaceCrystalPatch>, Option<&TunnelState>), With<ObjectInstance>>,
   ```
   Change to:
   ```rust
   target_info: Query<(Option<&SupplyDeliveryStation>, Option<&SupplyTowerState>, &Owner, Option<&SpaceCrystalPatch>, Option<&TunnelState>, Option<&ArmoryState>), With<ObjectInstance>>,
   ```
   This means ALL destructure sites for `target_info.get()` throughout core.rs must add a 6th field `armory_opt` (or `_`). Search for `target_info.get(` — there are ~10+ call sites.

2. **Add `has_selected_recruits` flag** (after line 324):
   ```rust
   let has_selected_recruits = selected_units.iter().any(|(_, _, _, _, _, _, obj, _, _)| obj.object_type == ObjectEnum::CultsRecruit);
   ```

3. **Add Recruit right-click Armory block** (after the Agent block at line ~660, before the Guard tunnel enter block at line ~662). Pattern:
   ```rust
   // Right-click on own Armory: CultsRecruit Enter
   if is_right_click && command_type == CommandType::Default && has_selected_recruits {
       if let Some(target_entity) = cursor_target.entity {
           if let Ok((_, _, target_owner, _, _, armory_opt)) = target_info.get(target_entity) {
               if let Some(armory_state) = armory_opt {
                   if target_owner.player_number() == Some(local_player.0)
                       && armory_state.stored_recruits.len() < ARMORY_INTERNAL_RECRUIT_CAPACITY
                   {
                       for (entity, _, _, _, attack_state_opt, _, obj, _, mut command_queue) in &mut selected_units {
                           if obj.object_type != ObjectEnum::CultsRecruit { continue; }
                           if let Some(attack_state) = attack_state_opt {
                               if !attack_state.phase.is_interruptible() { continue; }
                           }
                           let mut entity_cmds = commands_ecs.entity(entity);
                           if !shift_held {
                               clear_movement_state_full(&mut entity_cmds);
                           }
                           issue_or_queue_command(&mut entity_cmds, &mut command_queue, UnitCommand::EnterArmory(target_entity), shift_held);
                       }
                       info!("Recruit: Enter Armory");
                       *interface_state = ObjectInterfaceState::Default;
                       return;
                   }
               }
           }
       }
   }
   ```

### System Registration

In `artifacts/developer/src/game/units/mod.rs` (line ~24-26), add the new systems to Phase 2:
```rust
systems::behaviors::enter_armory_dispatch_system,
systems::behaviors::entering_armory_behavior_system
    .after(systems::behaviors::enter_armory_dispatch_system),
```

### Imports Required

- `behaviors.rs`: Add `EnteringArmoryBehavior` to the behavior import block (line 4-9). Add `ArmoryState` import: `use crate::game::types::structures::ArmoryState;`. Add `use crate::game::types::structures::cults_structure_stats::ARMORY_INTERNAL_RECRUIT_CAPACITY;`.
- `core.rs`: Add `ArmoryState` import and `ARMORY_INTERNAL_RECRUIT_CAPACITY` import. Add `EnteringArmoryBehavior` is NOT needed in core.rs (dispatch handles that).
- `commands.rs` (line 209): Add `UnitCommand::EnterArmory(entity) => (CommandType::Enter, None, Some(*entity)),`

### Existing Patterns to Follow

- **`EnteringTunnelBehavior`** (behavior.rs:134-162): Marker component pattern — clone this for `EnteringArmoryBehavior`
- **`enter_command_dispatch_system`** (behaviors.rs:465-513): Dispatch validation pattern — validates target exists, ownership matches, then inserts behavior marker
- **`entering_tunnel_behavior_system`** (behaviors.rs:524-572): Movement + arrival + hide pattern — uses `LocomotionChannel::Moving`, checks distance threshold, sets `Visibility::Hidden` on arrival
- **Right-click Agent block** (core.rs:581-658): Pattern for faction-specific right-click with target type detection and command issuing
- **`has_selected_choppers`/`has_selected_agents`** (core.rs:322-324): Pattern for pre-computing selected unit type flags

### Key Constants

- `ARMORY_INTERNAL_RECRUIT_CAPACITY = 10` — defined in `armory_structure` task at `artifacts/developer/src/game/types/structures.rs` in `cults_structure_stats` module
- `ARMORY_ARRIVAL_THRESHOLD = 0.5` — define locally in behaviors.rs (matches `TUNNEL_ARRIVAL_THRESHOLD`)

### Recruit Spawn Components Note

`spawn_cults_recruit()` (utils.rs:1043-1074) spawns with minimal components — notably has `UnitCommand`, `ObjectInstance(CultsRecruit)`, `Unit`, `Owner`, `UnitBaseEnum`, `GridPosition` but does NOT have `LocomotionChannel`/`OrientationChannel`. The behavior system queries for these channels. Either:
1. Add `LocomotionChannel`/`OrientationChannel` to `spawn_cults_recruit()` (preferred — needed for any movement behavior), OR
2. Make the behavior system's query for channels optional and handle the case

Option 1 is strongly preferred — add `LocomotionChannel::Stationary`, `OrientationChannel::Maintaining`, `Velocity(Vec3::ZERO)` to the recruit spawn (matching other unit spawns like Guard at utils.rs:892-928).

## Dependencies

- **`armory_structure` (planned_task)**: Must land first. Defines `ObjectEnum::CultsArmory`, `ArmoryState` component (with `stored_recruits: Vec<Entity>`), `ARMORY_INTERNAL_RECRUIT_CAPACITY` constant, and `spawn_cults_armory()`. This task references all of these types and constants.
- **No other dependencies.** `CultsRecruit` ObjectEnum variant already exists (shared/types.rs:334). `spawn_cults_recruit()` already exists (utils.rs:1043). The tunnel enter pattern this is modeled on is already fully implemented.
