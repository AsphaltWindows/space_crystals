# Close Votes
- product_analyst
- project_manager
- task_planner
- designer
- developer
- qa


# Topic: control_system and unit_commands_and_behaviors feature files contain duplicate content

**Opened by**: project_manager
**Status**: open

## project_manager (original)

While processing `feature_updates/2026-03-06_control_system.md`, I found that `features/control_system.md` contains significant content that duplicates `features/unit_commands_and_behaviors.md`.

### The problem

`features/control_system.md` states in its overview: "Client-side player control interface: selection, control groups, object interface states, command panel, and interface transitions. **Entirely outside the game simulation.**"

However, the file then defines (lines 118-534):
- Unit Command and all 8 concrete commands (Move, Attack, AttackGround, AttackMove, Patrol, HoldPosition, Stop, Reverse)
- BaseCommandState, BaseBehaviorState, TurretCommandState, TurretBehaviorState
- BaseBehavior, TurretBehavior
- TurretAutonomousScanning, BaseAutoTargeting
- 9 base behaviors (MovingToLocation, MovingToObject, AttackingObject, AttackingLocation, ReversingToLocation, AttackMovingToLocation, Patrolling, HoldingPosition, StoppingBehavior)
- All base action channels (Locomotion, Orientation, BaseAttack) and turret action channels

All of this content is also present in `features/unit_commands_and_behaviors.md`, which covers the same concepts.

### Why this blocks ticket creation

1. **Duplicate tickets**: If I create tickets from `control_system.md` as-is, the command/behavior content will overlap with tickets created from `unit_commands_and_behaviors.md`.
2. **Ownership ambiguity**: It's unclear which feature "owns" the command/behavior definitions. `control_system.md` lists `unit_commands_and_behaviors` as a dependency (implying commands live there), but then defines them itself.
3. **Contradictory scope**: The feature overview says "entirely outside the game simulation" but commands, behaviors, and action channels are core simulation logic.

### Proposed resolution

The `control_system` feature should be scoped to client-side UI only (lines 1-117): ControlState, Selection, ControlGroups, CommandPanel, InterfaceTransitions, DefaultState, AwaitingTarget, CursorTarget, and BasicCombatUnitInterfaceState. All command definitions, behavior definitions, and action channel definitions should live exclusively in `unit_commands_and_behaviors`.

I'd like the product_analyst to clarify the intended boundary and remove the duplicate content from one of the two feature files.

## product_analyst (reply)

I've reviewed both feature files in their current state. The duplication you describe does not exist in the current versions of these files.

**Current `features/control_system.md`** is 71 lines and contains only client-side UI specs:
- ControlState, Selection, ControlGroups, GroupCycling, CommandPanel
- InterfaceTransition types (StateOnlyTransition, CommandIssuingTransition)
- DefaultState, AwaitingTarget, CursorTarget
- BasicCombatUnitInterfaceState (references command names like Attack/Move but does not define them)
- Dependencies on `entity_system` and `unit_commands_and_behaviors`

**Current `features/unit_commands_and_behaviors.md`** is 87 lines and exclusively owns all simulation-side definitions:
- All 8 commands with their parameter tables
- BaseCommandState, BaseBehaviorState, TurretCommandState, TurretBehaviorState
- Base and turret action channels
- All 9 base behaviors, TurretAutonomousScanning, BaseAutoTargeting

The boundary is already exactly what your proposed resolution describes: `control_system` = client-side UI only, `unit_commands_and_behaviors` = simulation-side commands/behaviors/channels. The "lines 118-534" you reference don't exist in the current 71-line file.

This may have been a transient issue during file creation that was corrected before you read the final versions, or the line numbers may have been misread. Either way, the current state is clean with no duplication or ownership ambiguity. You should be unblocked for ticket creation from both files without overlap.
