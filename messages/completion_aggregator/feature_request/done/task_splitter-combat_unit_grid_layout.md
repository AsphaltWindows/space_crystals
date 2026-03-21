# combat-unit-grid-layout

## Metadata
- **From**: task_splitter
- **To**: completion_aggregator

## Content

Added specific grid slot assignments to BasicCombatUnitInterfaceState.

The combat unit command panel now has defined hotkey positions:

```
[Q] Move      [W] Reverse*    [E] HoldPosition
[A] Attack    [S] Patrol      [D] AttackGround*
[Z] —         [X] Stop        [C] —
```

Key design decisions:
- **AttackMove is NOT a separate command** — it is the result of AwaitingTarget[Attack] resolving on ground (left-click ground = AttackMove). No dedicated button needed.
- **W: Reverse** is conditional — only visible if UnitBase has CanReverse = true.
- **D: AttackGround** is conditional — only visible if AttackType has CanTargetGround = true.
- **Z** is reserved for Back/Cancel in AwaitingTarget states (standard slot assignment).
- **C** is currently unused for combat units.

Modified file: `artifacts/designer/design/control_system.md` — BasicCombatUnitInterfaceState DefaultState section.

## QA Instructions

1. Select a combat unit (e.g., Peacekeeper).
2. Verify the command panel shows the correct grid layout:
   - Q = Move, A = Attack, S = Patrol, E = HoldPosition, X = Stop.
3. Verify that W (Reverse) is hidden for units without CanReverse (e.g., LightInfantry).
4. Verify that D (AttackGround) is hidden for units without CanTargetGround.
5. Press A (Attack), then left-click ground — verify an AttackMove command is issued (not a separate AttackMove button).
6. Press A (Attack), then left-click an enemy — verify an Attack command is issued.
7. While in AwaitingTarget (e.g., after pressing A), press Z or Escape — verify it returns to DefaultState.
8. For a unit with CanReverse = true: verify W (Reverse) appears and enters AwaitingTarget[Reverse].
9. For a unit with CanTargetGround = true: verify D (AttackGround) appears and enters AwaitingTarget[AttackGround].
