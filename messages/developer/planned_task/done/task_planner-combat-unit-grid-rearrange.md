# combat-unit-grid-rearrange

## Metadata
- **From**: task_planner
- **To**: developer

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

## Technical Context

### Primary file: `artifacts/developer/src/ui/command_panel.rs`

**Grid layout block (lines 142-153)** — This is the exact block to modify:
```rust
ObjectInterfaceState::Default => match (row, col) {
    // Unit commands (only shown when Selection has units)
    (0, 0) => Some(CommandButtonAction::UnitMove),
    (0, 1) if caps.has_attack => Some(CommandButtonAction::UnitAttack),
    (0, 2) if caps.can_target_ground => Some(CommandButtonAction::UnitAttackGround),
    (1, 0) if caps.has_attack => Some(CommandButtonAction::UnitAttackMove),
    (1, 1) => Some(CommandButtonAction::UnitPatrol),
    (1, 2) => Some(CommandButtonAction::UnitHoldPosition),
    (2, 0) => Some(CommandButtonAction::UnitStop),
    (2, 1) if caps.can_reverse => Some(CommandButtonAction::UnitReverse),
    _ => None,
},
```

Replace with:
```rust
ObjectInterfaceState::Default => match (row, col) {
    // Unit commands (only shown when Selection has units)
    (0, 0) => Some(CommandButtonAction::UnitMove),
    (0, 1) if caps.can_reverse => Some(CommandButtonAction::UnitReverse),
    (0, 2) => Some(CommandButtonAction::UnitHoldPosition),
    (1, 0) if caps.has_attack => Some(CommandButtonAction::UnitAttack),
    (1, 1) => Some(CommandButtonAction::UnitPatrol),
    (1, 2) if caps.can_target_ground => Some(CommandButtonAction::UnitAttackGround),
    (2, 1) => Some(CommandButtonAction::UnitStop),
    _ => None,
},
```

Note: Attack is no longer conditional-free — it keeps the `caps.has_attack` guard. UnitAttackMove is completely removed from the grid.

**AwaitingTarget Back button (lines 170-173)** — Already correct. The `(2, 0) => Some(CommandButtonAction::Back)` in the AwaitingTarget match ensures Z shows Back/Cancel. No changes needed here.

**execute_command_action (line ~1282)** — The `CommandButtonAction::UnitAttackMove` handler sets `AwaitingTarget(CommandType::AttackMove)`. Since UnitAttackMove is no longer on any grid slot, this code path becomes dead code for grid buttons. However, **do NOT remove it** — it may still be used by the legacy hotkey system in `artifacts/developer/src/game/units/systems/commands.rs` (line 45: `KeyCode::KeyT` triggers `CommandType::AttackMove`). The legacy hotkeys fire when the panel is hidden.

**is_action_active (line ~2181)** — The `(CommandButtonAction::UnitAttackMove, CommandType::AttackMove)` match arm can stay; it's harmless and maintains symmetry for any future use.

### Tests to update (same file, starting ~line 2420):

1. **`unit_commands_attack_requires_has_attack`** (line 2427): Change grid position from `(0, 1)` to `(1, 0)`
2. **`unit_commands_attack_ground_requires_can_target_ground`** (line 2438): Change grid position from `(0, 2)` to `(1, 2)`
3. **`unit_commands_attack_move_requires_has_attack`** (line 2449): **Delete this entire test** — AttackMove is no longer on the grid
4. **`unit_commands_patrol_always_available`** (line 2460): Position `(1, 1)` — unchanged
5. **`unit_commands_hold_position_always_available`** (line 2467): Change grid position from `(1, 2)` to `(0, 2)`
6. **`unit_commands_stop_always_available`** (line 2474): Change grid position from `(2, 0)` to `(2, 1)`
7. **`unit_commands_reverse_requires_can_reverse`** (line 2481): Change grid position from `(2, 1)` to `(0, 1)`
8. **`all_caps_unit_commands_shows_all_eight_commands`** (line 2544): Change expected count from 8 to 7 (AttackMove removed) and update the comment
9. **`no_caps_unit_commands_shows_only_universal`** (line 2559): Expected count stays 4 but update comment to reflect new positions: Move(0,0), HoldPos(0,2), Patrol(1,1), Stop(2,1)

### Secondary files (read-only verification, no changes needed):

- **`artifacts/developer/src/ui/types.rs`**: `CommandButtonAction` enum, `SelectedUnitCapabilities` struct — no changes
- **`artifacts/developer/src/game/units/types/state/commands.rs`**: `CommandType` enum, `UnitCommand` enum — no changes
- **`artifacts/developer/src/game/units/systems/commands.rs`**: Legacy hotkey system — no changes (AttackMove still accessible via T key when panel hidden)

### Pattern notes:

- The grid uses `(row, col)` as `(u8, u8)` where row 0 = top (Q/W/E), row 1 = middle (A/S/D), row 2 = bottom (Z/X/C)
- Conditional commands use Rust match guards: `if caps.has_attack =>`
- The `_ => None` wildcard at the end catches all unassigned slots
- Grid labels are generated by `grid_button_label()` (line ~2098) — these are driven by the action enum, not the position, so they auto-update

## Dependencies

None. This is a self-contained UI layout change within `command_panel.rs`. The grid layout block and its tests are the only things that change.
