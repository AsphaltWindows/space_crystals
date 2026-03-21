# combat-unit-grid-rearrange

## Metadata
- **From**: task_splitter
- **To**: task_planner

## Content

## Parent Feature

task_splitter-combat-unit-grid-layout.md

## Task

Rearrange the combat unit DefaultState command panel grid layout to match the updated design spec.

**Current layout** (in `command_panel.rs`, the `ObjectInterfaceState::Default` match for unit commands):
```
[Q] Move      [W] Attack*     [E] AttackGround*
[A] AttackMove* [S] Patrol    [D] HoldPosition
[Z] Stop      [X] Reverse*   [C] —
```

**Required layout**:
```
[Q] Move      [W] Reverse*    [E] HoldPosition
[A] Attack    [S] Patrol      [D] AttackGround*
[Z] —         [X] Stop        [C] —
```

Changes needed:

1. **Rearrange the grid slots** in the `ObjectInterfaceState::Default` match block (in `command_panel.rs` around lines 142-153):
   - (0,0) Q = UnitMove (unchanged)
   - (0,1) W = UnitReverse (conditional: `caps.can_reverse`) — moved from (2,1)
   - (0,2) E = UnitHoldPosition — moved from (1,2)
   - (1,0) A = UnitAttack (conditional: `caps.has_attack`) — moved from (0,1)
   - (1,1) S = UnitPatrol (unchanged)
   - (1,2) D = UnitAttackGround (conditional: `caps.can_target_ground`) — moved from (0,2)
   - (2,0) Z = None (empty, reserved for Back/Cancel in AwaitingTarget)
   - (2,1) X = UnitStop — moved from (2,0)
   - (2,2) C = None (unused)

2. **Remove the AttackMove button**: The UnitAttackMove action should no longer be assigned to any grid slot. AttackMove is produced by AwaitingTarget[Attack] resolving on ground (left-click ground = AttackMove command). Verify this resolution already works in the existing `AwaitingTarget(CommandType::Attack)` handling.

3. **Verify AwaitingTarget cancel**: Ensure Z (2,0) in AwaitingTarget states shows Back/Cancel and returns to DefaultState. This likely already works but verify.

Key files:
- `artifacts/developer/src/ui/command_panel.rs` — main grid layout logic, `execute_command_action()`, AwaitingTarget handling
- `artifacts/developer/src/ui/types.rs` — ObjectInterfaceState, SelectedUnitCapabilities
- `artifacts/developer/src/game/units/types/state/commands.rs` — CommandType, UnitCommand enums
