# Developer Task: BasicCombatUnitInterfaceState

## Current State
The command panel and unit command infrastructure exist but lack two critical behaviors:
1. **Conditional command visibility**: All 7 unit commands are always shown in the grid regardless of unit capabilities
2. **Right-click context resolution**: Right-click in Default mode only issues Move-to-ground; it doesn't detect enemy/friendly entities for automatic Attack vs Move resolution
3. **Attack mode ground click**: Left-clicking ground in Attack command mode silently cancels instead of issuing AttackMove

## Desired State
Implement BasicCombatUnitInterfaceState as the concrete command interface for combat units:

**DefaultState immediate commands** (no target required):
- `HoldPosition`: issues HoldPosition command
- `Stop`: issues Stop command

**DefaultState target commands** (enters command mode):
- `Attack`: enters Attack mode
- `Move`: enters Move mode
- `Patrol`: enters Patrol mode
- `AttackGround`: enters AttackGround mode (only if unit's AttackType has CanTargetGround=true)
- `Reverse`: enters Reverse mode (only if unit's UnitBase has CanReverse=true)

**RightClickResolution** (from DefaultState):
- Cursor over EnemyObject: issues Attack command targeting that object
- Cursor over Ground: issues Move command to that location
- Cursor over FriendlyObject or NeutralObject: issues Move command to that object

**AwaitingTarget[Attack] resolution**:
- Left-click EnemyObject: issues Attack command targeting that object
- Left-click Ground: issues AttackMove command to that location (NOT silent cancel)

**AwaitingTarget[Move] resolution**:
- Left-click Ground: issues Move command to that location
- Left-click any Object: issues Move command to that object

**AwaitingTarget[Patrol] resolution**:
- Left-click Ground: issues Patrol command to that location

**AwaitingTarget[AttackGround] resolution**:
- Left-click Ground: issues AttackGround command to that location

**AwaitingTarget[Reverse] resolution**:
- Left-click Ground: issues Reverse command to that location

**Conditional command availability**:
- AttackGround only appears if the unit's AttackType.CanTargetGround = true
- Reverse only appears if the unit's UnitBase.CanReverse = true

## Technical Context

### Key Files and Their Roles

1. **`src/ui/command_panel.rs`** — Main file to modify
   - `get_grid_slot_action()` (line 35-96): Currently hardcodes all 7 commands for `UnitCommands` state (lines 80-89). Must become conditional based on selected unit capabilities.
   - `execute_command_action()` (line 458-663): Currently handles `UnitAttackGround` and `UnitAttackMove` unconditionally. No `UnitReverse` action exists.
   - `rebuild_command_panel_ui()` (line 185-332): Needs to pass unit capability data to `get_grid_slot_action()`.
   - `update_command_panel_state()` (line 99-182): Currently sets `UnitCommands` without tracking which unit capabilities are active.

2. **`src/ui/types.rs`** — UI types
   - `CommandButtonAction` enum (line 133-166): Missing `UnitReverse` variant. Has `UnitAttackGround`, `UnitAttackMove`.
   - `CommandPanelState::UnitCommands` (line 128): Currently a simple marker. Needs to carry capability info or a parallel resource needs to track it.

3. **`src/game/units/systems.rs`** — Right-click command handling
   - `right_click_move_command()` (line 136-407): This is the MAIN system needing changes:
     - **Right-click Default mode** (line 254-282): Currently only does Move. Must add entity detection for enemy/friendly distinction BEFORE ground raycast.
     - **Left-click Attack mode ground** (line 316-318): Currently just cancels with log message. Must issue `AttackMove` command instead.
     - **Left-click Attack mode entity** (line 176-243): Works correctly — issues `AttackTarget`.
     - Missing: `CommandType::Patrol` doesn't exist as a right-click handler — only left-click (correct per spec).

4. **`src/game/units/types/state/commands.rs`** — Command definitions
   - `UnitCommand::is_available()` (line 26-42): Already correctly gates commands by `has_attack`, `can_target_ground`, `can_reverse`. Use this in the UI.
   - `CommandType` enum (line 52-61): Missing `Reverse` variant — needs adding.
   - `CommandMode` resource (line 46-49): Used to track active command mode.

5. **`src/game/combat/types.rs`** — Attack capability
   - `AttackCapability` component (line 31-44): Check presence for `has_attack`. Access `attack_type` field to get the runtime `AttackType`.

6. **`src/types.rs`** — Identity enums
   - `AttackTypeEnum::can_target_ground()` (line 221): Returns true for `TailDisjointed` and `DoublyDisjointed`. BUT this is on the identity enum, not the runtime `AttackType`. At runtime, derive from `AttackType` variant matching.
   - `UnitBaseEnum` (line 183-193): Spawned as ECS component on units.

7. **`src/game/units/types/movement.rs`** — Unit base data
   - `UnitBaseData.can_reverse` (line 278): Access via `UnitBaseEnum::data().can_reverse`.

### What Needs to Change

#### 1. Add `UnitReverse` to `CommandButtonAction` (`src/ui/types.rs`)
```rust
/// Unit: Set command mode to Reverse
UnitReverse,
```

#### 2. Add `Reverse` variant to `CommandType` (`src/game/units/types/state/commands.rs`)
```rust
pub enum CommandType {
    Default,
    Move,
    Attack,
    AttackGround,
    AttackMove,
    Patrol,
    Reverse,  // NEW
}
```
Update `name()` and `hotkey()` methods.

#### 3. Track unit capabilities for command panel (`src/ui/types.rs` or `src/ui/command_panel.rs`)
Create a resource to track selected unit capabilities:
```rust
#[derive(Resource, Default)]
pub struct SelectedUnitCapabilities {
    pub has_attack: bool,
    pub can_target_ground: bool,
    pub can_reverse: bool,
}
```
Update this in `update_command_panel_state()` by querying selected units' `AttackCapability` (presence + `attack_type`) and `UnitBaseEnum` (`.data().can_reverse`).

#### 4. Make `get_grid_slot_action()` conditional (`src/ui/command_panel.rs`)
Pass `SelectedUnitCapabilities` and filter commands:
- Grid slot (0,2) `UnitAttackGround`: only if `can_target_ground`
- Add new grid slot for `UnitReverse`: only if `can_reverse`
- Suggested layout:
  ```
  [Q] Move    [W] Attack    [E] AtkGround*
  [A] AtkMove [S] Patrol    [D] HoldPos
  [Z] Stop    [X] Reverse*
  ```
  (* = conditional)

#### 5. Fix right-click enemy detection (`src/game/units/systems.rs`)
In `right_click_move_command()`, for right-click in Default mode:
- BEFORE ground raycast (line 245), add entity detection similar to Attack mode (lines 176-200)
- Compare clicked entity's `Owner` against selected unit's `Owner`
- If enemy (different owner, non-neutral): issue `UnitCommand::AttackTarget(entity)`
- If friendly/neutral/none: fall through to existing Move logic
- Use same `SelectionBounds`-based click detection as Attack mode

#### 6. Fix Attack mode ground click (`src/game/units/systems.rs`)
Line 316-318 currently:
```rust
CommandType::Attack => {
    info!("Attack mode: No enemy at click location, returning to default mode");
    command_mode.mode = CommandType::Default;
}
```
Change to issue AttackMove:
```rust
CommandType::Attack => {
    // Attack + ground click = AttackMove
    for (entity, transform, unit_base, _owner) in selected_units.iter() {
        // pathfind and issue UnitCommand::AttackMove(target_pos)
    }
    command_mode.mode = CommandType::Default;
}
```

#### 7. Add Reverse command mode handling (`src/game/units/systems.rs`)
Add `CommandType::Reverse` match arm in `right_click_move_command()` ground click handler (after line 377):
```rust
CommandType::Reverse => {
    for (entity, transform, unit_base, _owner) in selected_units.iter() {
        // pathfind and issue UnitCommand::Reverse(target_pos)
    }
    command_mode.mode = CommandType::Default;
}
```

### Patterns to Follow
- **Command mode flow**: Set `command_mode.mode` in `execute_command_action()`, display mode text in `rebuild_command_panel_ui()`, resolve in `right_click_move_command()`. Existing Attack/Patrol/AttackMove/AttackGround modes follow this pattern exactly.
- **Entity detection**: Reuse the `SelectionBounds`-based raycasting pattern from lines 176-200 of `right_click_move_command()`.
- **Owner comparison**: `Owner` component has `player_number()` method. Same player = friendly, different player = enemy, no player (neutral) = friendly.
- **Capability query**: Query `Option<&AttackCapability>` and `&UnitBaseEnum` on selected units.
- **Active button highlighting**: `is_action_active()` at line 826 already handles matching action to mode — add `UnitReverse` case.

### Relevant Types
- `AttackCapability` — ECS component, presence = has attack, `attack_type` field contains `AttackType` enum
- `AttackType` — Runtime enum: `FullyConnected`, `TailDisjointed{..}`, `HeadDisjointed{..}`, `DoublyDisjointed{..}`. Check `TailDisjointed`/`DoublyDisjointed` for can_target_ground.
- `UnitBaseEnum` — ECS component on units, `.data()` returns `UnitBaseData` with `can_reverse: bool`
- `Owner` — ECS component with `player_number()` for enemy/friendly detection
- `CommandMode` — Resource tracking active command mode
- `CommandType` — Enum of command modes (need to add `Reverse`)
- `UnitCommand` — Enum of actual commands (already has all needed variants including `Reverse(Vec3)`)

## Dependencies
- **`command_panel_and_interface_state_machine`** (soft): The enriched task describes a future `ObjectInterfaceState` refactor that would replace `CommandPanelState`. This task can proceed using the current flat `CommandPanelState` architecture — the refactor would be a follow-up. No blocker.
- **`unit_commands_and_command_state`** (satisfied): All `UnitCommand` variants including `AttackMove` and `Reverse` already exist. `CommandQueue`, `BaseCommandState`, `TurretCommandState` all implemented.
- **`attack_attributes_types_and_targeting`** (soft): `AttackCapability` already has `attack_type` field with runtime `AttackType`. `can_target_ground` derivation works via pattern matching on `AttackType`. No blocker.

## QA Steps
1. [human] Select a combat unit. Verify the command panel shows HoldPosition, Stop, Attack, Move, Patrol. Verify AttackGround appears only if the unit's attack type has CanTargetGround=true. Verify Reverse appears only if the unit's base has CanReverse=true.
2. [auto] Click HoldPosition. Verify HoldPosition command is issued immediately (no target selection).
3. [auto] Click Stop. Verify Stop command is issued immediately.
4. [human] Click Attack. Verify AwaitingTarget[Attack] is entered. Left-click an enemy. Verify Attack command targeting that enemy is issued.
5. [auto] In AwaitingTarget[Attack], left-click ground. Verify AttackMove command to that location is issued.
6. [auto] Click Move. Left-click ground. Verify Move command to that location. Click Move again, left-click a friendly object. Verify Move command to that object.
7. [auto] Click Patrol. Left-click ground. Verify Patrol command to that location.
8. [auto] For a unit with CanTargetGround: click AttackGround. Left-click ground. Verify AttackGround command to that location.
9. [auto] For a unit with CanReverse: click Reverse. Left-click ground. Verify Reverse command to that location.
10. [auto] Right-click an enemy unit. Verify Attack command is issued targeting that enemy.
11. [auto] Right-click ground. Verify Move command to that location.
12. [auto] Right-click a friendly unit. Verify Move command to that object.
13. [auto] Right-click a neutral object. Verify Move command to that object.
14. [auto] For a unit without CanTargetGround: verify AttackGround does not appear in the command panel.
15. [auto] For a unit without CanReverse: verify Reverse does not appear in the command panel.

## Expected Experience
- Combat unit selection shows a clean command panel with all applicable commands.
- Immediate commands (HoldPosition, Stop) execute instantly on click.
- Target commands show visual feedback during target selection.
- Right-clicking feels natural: enemies get attacked, everything else gets moved to.
- AwaitingTarget[Attack] on ground smartly resolves to AttackMove instead of failing.
- Conditional commands only appear when the unit supports them.

## QA Failure (Previous — returned to developer)
- Step 4 [human]: FAIL — Attack enters AwaitingTarget mode correctly, but left-clicking an enemy unit selects that enemy instead of issuing the attack command.

## QA Failure — 2026-03-09
- Step 1 [human]: PASS — correct commands shown, conditional commands hidden
- **Step 4 [human]: FAIL — Same issue persists. Attack mode entered correctly, but left-clicking an enemy unit does not issue the attack command. The click does not register as targeting the enemy object.** This is a repeat of the previous failure — the fix did not resolve it.

## Automated QA Results (Re-run 2026-03-09, pass 2)
- Step 1 [human]: DEFERRED to human review
- Step 2 [auto]: PASS — HoldPosition command issued immediately
- Step 3 [auto]: PASS — Stop command issued immediately
- Step 4 [human]: DEFERRED to human review (previously failed 3x — verify fix)
- Step 5 [auto]: PASS — Attack + ground click issues AttackMove
- Step 6 [auto]: PASS — Move + ground click issues Move; Move + friendly click issues Move
- Step 7 [auto]: PASS — Patrol + ground click issues Patrol
- Step 8 [auto]: SKIPPED — No unit with CanTargetGround available (Peacekeeper lacks it)
- Step 9 [auto]: SKIPPED — No unit with CanReverse available
- Step 10 [auto]: PASS — Right-click enemy issues AttackTarget
- Step 11 [auto]: PASS — Right-click ground issues Move
- Step 12 [auto]: PASS — Right-click friendly issues Move
- Step 13 [auto]: PASS — Right-click neutral issues Move
- Step 14 [auto]: PASS — Peacekeeper SelectedUnitCapabilities.can_target_ground == false
- Step 15 [auto]: PASS — Peacekeeper SelectedUnitCapabilities.can_reverse == false

## QA Failure — 2026-03-09 (3rd failure)
**Step 4 [human]: FAIL — Neither Attack mode left-click on enemy buildings NOR right-click on enemy buildings causes the unit to attack. Both interaction paths fail to register the target.** Note: auto tests pass for right-click enemy (step 10) in headless mode, suggesting the click-to-entity resolution works in ECS but fails in the actual rendered game — likely a raycasting/hit-detection issue with real 3D objects vs test stubs. This is the #2 recurring blocker after Syndicate Agent spawn.

## QA Failure — 2026-03-09 (4th failure)
**Step 4 [human]: FAIL — Still broken. Attack left-click on enemy does not register. No change from previous attempts.**

## QA Failure — 2026-03-09 (automated re-run, 5th attempt)
**ALL [auto] steps: FAIL — Test compilation failed.** Source code has compile errors in `src/game/world/faction.rs` and `src/ui/command_panel.rs`: `TunnelOperation::BuildingExpansion` pattern/initializer missing fields `grid_x`, `grid_z`. These are not related to this task but prevent any test from compiling. Auto steps cannot be verified until the compile errors are fixed.

## Automated QA Results — 2026-03-09 (6th run, compile fix confirmed)
- Step 1 [human]: DEFERRED to human review
- Step 2 [auto]: PASS — HoldPosition command issued immediately
- Step 3 [auto]: PASS — Stop command issued immediately
- Step 4 [human]: DEFERRED to human review (known recurring failure — attack left-click on enemy)
- Step 5 [auto]: PASS — Attack + ground click issues AttackMove
- Step 6 [auto]: PASS — Move + ground/friendly click issues Move
- Step 7 [auto]: PASS — Patrol + ground click issues Patrol
- Step 8 [auto]: SKIPPED — No unit with CanTargetGround available
- Step 9 [auto]: SKIPPED — No unit with CanReverse available
- Step 10 [auto]: PASS — Right-click enemy issues AttackTarget
- Step 11 [auto]: PASS — Right-click ground issues Move
- Step 12 [auto]: PASS — Right-click friendly issues Move
- Step 13 [auto]: PASS — Right-click neutral issues Move
- Step 14 [auto]: PASS — Peacekeeper correctly lacks AttackGround
- Step 15 [auto]: PASS — Peacekeeper correctly lacks Reverse
