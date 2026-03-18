# Task: Supply Tower ObjectInterfaceState Implementation

## Original Ticket
Implement `ObjectInterfaceState[SupplyTower]` with: Q (Build Supply Chopper), X (Cancel Production), C (Set Rally Point), S (Schedule Deliveries), plus right-click rally point setting.

## Technical Context

### Scope Clarification
**Most of the infrastructure for this task is handled by dependencies**:
- `standard_bottom_row_commands` adds: `SetRallyPoint` CommandButtonAction, `CommandType::SetRallyPoint`, AwaitingTarget[SetRallyPoint] handler, generalized `production_rally_point_system`, `rally_point` field on `SupplyTowerState`, and moves `StCancel` from (0,1) to (2,1) + adds `SetRallyPoint` at (2,2)

This task covers **remaining Supply Tower-specific adjustments** after the dependency completes, plus completing the `ScheduleDeliveries` AwaitingTarget flow.

### What Should Already Be Done (by `standard_bottom_row_commands`)
After that task completes, the SupplyTowerMenu grid should be:
```rust
StructureMenuState::SupplyTowerMenu => match (row, col) {
    (0, 0) => Some(CommandButtonAction::StTrain(ObjectEnum::SupplyChopper)),  // Q
    (1, 0) => Some(CommandButtonAction::StScheduleDeliveries),                // S
    (2, 1) if bk_has_queue => Some(CommandButtonAction::StCancel),            // X
    (2, 2) => Some(CommandButtonAction::SetRallyPoint),                       // C
    _ => None,
},
```
And the production right-click rally system should handle SupplyTowerMenu already.

### What This Task Implements/Verifies

#### 1. Verify Grid Slot Correctness
- **File**: `src/ui/command_panel.rs:90-95` (current location, will shift with dependency)
- Confirm Q at (0,0), S at (1,0), X at (2,1), C at (2,2) after dependency completes
- Verify `bk_has_queue` gate correctly hides X when queue empty (existing pattern)

#### 2. Add `CommandType::ScheduleDeliveries` variant
- **File**: `src/game/units/types/state/commands.rs:76-90`
- Add `ScheduleDeliveries` variant to the `CommandType` enum
- Add `CommandType::ScheduleDeliveries => "Schedule Deliveries"` in the `name()` method

#### 3. Complete `StScheduleDeliveries` handler — enter AwaitingTarget mode
- **File**: `src/ui/command_panel.rs:1198-1201` — currently a TODO stub:
  ```rust
  CommandButtonAction::StScheduleDeliveries => {
      // TODO: Enter AwaitingTarget mode for schedule deliveries
      info!("Supply Tower: Schedule Deliveries (awaiting target not yet implemented)");
  }
  ```
- Replace with:
  ```rust
  CommandButtonAction::StScheduleDeliveries => {
      **interface_state = ObjectInterfaceState::AwaitingTarget(CommandType::ScheduleDeliveries);
      info!("Command mode: Schedule Deliveries");
  }
  ```

#### 4. Add AwaitingTarget[ScheduleDeliveries] left-click handler
- **File**: Best placed in `src/game/units/systems/core.rs` alongside the AwaitingTarget[SetRallyPoint] handler (created by dependency), or as a separate system
- When `interface_state == AwaitingTarget(CommandType::ScheduleDeliveries)` and left-click:
  - Raycast/entity check: confirm clicked entity has `ObjectInstance` with `object_enum == ObjectEnum::SupplyDeliveryStation`
  - If valid SDS: set `SupplyTowerState.scheduled_sds = Some(sds_entity)` on the `CommandPanelTarget` entity
  - Reset interface state to `StructureMenu(SupplyTowerMenu)`
  - If clicked entity is not an SDS, reject (no state change, maybe log)
- Query needed: `Query<&ObjectInstance>` to check clicked entity type
- Also need `Query<&mut SupplyTowerState>` for the target tower

#### 5. Supply Tower production + rally integration
- **File**: `src/game/world/faction.rs:1588-1634` — `supply_tower_production_tick_system`
- Currently spawns chopper at spawn-side exit but does NOT call `issue_rally_command()`
- After production completes (line 1620-1625), add rally point handling:
  ```rust
  // After spawn_supply_chopper...
  if let Some(rally) = &st_state.rally_point {
      // issue_rally_command() for the newly spawned chopper entity
  }
  ```
- **Challenge**: `spawn_supply_chopper` doesn't return the spawned Entity. Either:
  - Modify `spawn_supply_chopper()` at `src/game/utils.rs` to return `Entity`
  - Or capture the entity from `commands.spawn(...)` — the function needs adjustment
- `issue_rally_command` at `src/game/world/faction.rs:299-362` takes the spawned entity + rally target

#### 6. Right-click rally already handled
- The generalized `production_rally_point_system` (from dependency) already handles `SupplyTowerMenu` right-click → sets `SupplyTowerState.rally_point`
- **Verify** it works correctly for Supply Tower after dependency is complete

### Key Files
| File | Purpose |
|------|---------|
| `src/ui/command_panel.rs:90-95` | SupplyTowerMenu grid slot mapping |
| `src/ui/command_panel.rs:1163-1201` | StTrain, StCancel, StScheduleDeliveries handlers |
| `src/ui/command_panel.rs:1704-1708` | StScheduleDeliveries enabled gate (attached_chopper check) |
| `src/game/types/structures.rs:279-297` | `SupplyTowerState` struct |
| `src/game/types/structures.rs:299-325` | `SupplyTowerState` impl (MAX_QUEUE_SIZE, production_cost, try_queue, cancel_last) |
| `src/game/types/structures.rs:56-61` | `RallyTarget` enum |
| `src/game/world/faction.rs:1588-1634` | `supply_tower_production_tick_system` — add rally integration |
| `src/game/world/faction.rs:299-362` | `issue_rally_command()` — used for rally after production |
| `src/game/utils.rs` | `spawn_supply_chopper()` — may need to return Entity |
| `src/game/units/types/state/commands.rs:76-90` | `CommandType` enum — add ScheduleDeliveries |

### Existing Types
- `SupplyTowerState.rally_point: Option<RallyTarget>` — added by `standard_bottom_row_commands` dependency
- `SupplyTowerState.scheduled_sds: Option<Entity>` — already exists at structures.rs:291
- `SupplyTowerState.attached_chopper: Option<Entity>` — already exists at structures.rs:286
- `SupplyTowerState::MAX_QUEUE_SIZE` = 5
- `SupplyTowerState::production_cost(&ObjectEnum::SupplyChopper)` = 100 SC, 160 frames
- `RallyTarget::Location(Vec3) | Object(Entity)` at structures.rs:56-61

### Patterns to Follow
- **Barracks interface task**: Same verify-and-finalize pattern — most infrastructure comes from `standard_bottom_row_commands`
- **AwaitingTarget handler pattern**: See SetRallyPoint left-click handler (from dependency) for how to handle AwaitingTarget click + reset to default
- **StScheduleDeliveries enabled gate** (`command_panel.rs:1704-1708`): Already checks `attached_chopper.is_some()` — S button is disabled when no chopper attached
- **Production + rally pattern**: See `barracks_production_tick_system` at `faction.rs:228-296` for how it calls `issue_rally_command` after spawning

### Tests
- Supply Tower grid returns `StTrain(SupplyChopper)` at (0,0), `StScheduleDeliveries` at (1,0), `StCancel` at (2,1) when queue non-empty, `SetRallyPoint` at (2,2)
- Supply Tower grid returns `None` at (2,1) when queue is empty
- Supply Tower grid returns `None` at old position (0,1) — confirm StCancel moved
- `CommandType::ScheduleDeliveries.name()` returns "Schedule Deliveries"
- StScheduleDeliveries handler transitions to `AwaitingTarget(CommandType::ScheduleDeliveries)`

## Dependencies
- **`2026-03-09_standard_bottom_row_commands.md`** — **MUST complete first**. Provides: `SetRallyPoint` CommandButtonAction + CommandType, AwaitingTarget handler, generalized production right-click rally system, `rally_point` field on SupplyTowerState, and the Supply Tower grid slot rearrangement (StCancel to (2,1), SetRallyPoint at (2,2)). Without this, there's no rally infrastructure to build on.

## QA Steps
1. [auto] Select a Supply Tower — verify the command panel shows Q (Build Supply Chopper), X (Cancel Production), C (Set Rally Point), and S (Schedule Deliveries) in their correct grid positions
2. [human] Press Q with sufficient SC and queue space — verify a Supply Chopper is added to the build queue and 100 SC is deducted
3. [auto] Press Q when queue is full (5 entries) — verify the command is rejected
4. [auto] Press X with a non-empty queue — verify the last entry is removed and fully refunded
5. [auto] Press X with an empty queue — verify nothing happens
6. [auto] Press C — verify the cursor/state changes to AwaitingTarget[SetRallyPoint]
7. [human] While in AwaitingTarget[SetRallyPoint], left-click a ground location — verify rally point is set and state returns to DefaultState
8. [human] Right-click a ground location while Supply Tower is selected — verify rally point is set directly
9. [auto] Press S with an attached chopper — verify state changes to AwaitingTarget[ScheduleDeliveries]
10. [human] While in AwaitingTarget[ScheduleDeliveries], left-click an SDS — verify scheduled deliveries are set and state returns to DefaultState
11. [auto] Press S without an attached chopper — verify the command is unavailable or rejected
12. [human] Produce a Supply Chopper with a rally point set — verify it moves to the rally point after spawning

## Expected Experience
Selecting a Supply Tower shows a command card with four active slots: Q, X, C, and S. The Q/X production commands work identically to the Barracks pattern. The C rally point flow mirrors Barracks — press C, click a location, see a rally marker. The S command is contextual: it only activates when the tower has an attached chopper, and targets SDS objects specifically. Right-clicking ground also sets rally points directly. The interface feels consistent with other GDO production structures while surfacing the Supply Tower's unique delivery scheduling mechanic.

## QA Results — 2026-03-09 (partial, blocked by dependency failures)
- Step 1 [human]: Partial — Q/X/S visible. C visible but rally point doesn't work for Supply Tower (right-click or hotkey). X only cancels queued items, not active production.
- Step 2 [human]: PASS — Supply Chopper queued, 100 SC deducted
- Step 3 [human]: PASS — Queue full rejects command
- Steps 4-5 [human]: PASS — X cancels queued items (same limitation as Barracks)
- Steps 6-8 [human]: BLOCKED — Rally point broken for Supply Tower (standard_bottom_row dependency)
- Steps 9-11 [human]: BLOCKED — Chopper attachment not implemented, S command untestable
- Step 12 [human]: Partial PASS — Chopper spawns but no rally point behavior

## QA Results — 2026-03-09 (re-test, standard_bottom_row dependency fixed)
- Step 1 [human]: PASS — Q/X/C/S visible in correct positions (X conditional on queue)
- Step 2 [human]: PASS — Supply Chopper queued, 100 SC deducted
- Step 3 [auto]: PASS — Queue full rejects command
- Step 4 [auto]: PASS — X removes last entry with refund
- Step 5 [auto]: PASS — X with empty queue does nothing
- Step 6 [auto]: PASS — C enters AwaitingTarget[SetRallyPoint]
- Step 7 [human]: PASS — Left-click ground sets rally point, returns to DefaultState
- Step 8 [human]: PASS — Right-click ground sets rally point directly (previously broken)
- Steps 9-11 [human]: BLOCKED — Chopper attachment not implemented, S command untestable
- **Step 12 [human]: FAIL — Produced Supply Chopper does NOT move to the rally point after spawning. The chopper spawns but ignores the set rally point.**
