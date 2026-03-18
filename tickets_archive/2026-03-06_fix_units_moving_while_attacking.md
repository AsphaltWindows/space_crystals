# Ticket: Fix units continuing to move while attacking

## Current State
When a unit is ordered to attack a target (via right-click on enemy or AttackTarget command), the code inserts `UnitCommand::AttackTarget` but does not remove `MoveTarget`, `Path`, or zero `Velocity`. The unit continues following its previous movement path while simultaneously firing at the target. This also affects `auto_target_system` in `src/game/combat/systems.rs`, which inserts `AttackTarget` without clearing movement state.

## Desired State
1. **When `AttackTarget` is issued** (in `right_click_move_command` at `src/game/units/systems.rs` ~line 218-224): Remove `MoveTarget`, `Path`, `HoldingPosition`, and `IdleOrigin` components. Set `Velocity` to `Vec3::ZERO`. The unit stops immediately and begins the attack sequence.
2. **When auto-targeting acquires a target** (in `auto_target_system` at `src/game/combat/systems.rs`): Same movement state cleanup when `AttackTarget` is inserted.
3. **Create a `clear_movement_state` helper function** (in `src/game/utils.rs` or `src/game/combat/utils.rs`) that removes `MoveTarget`, `Path`, zeroes `Velocity`, and removes `HoldingPosition`/`IdleOrigin`. Use this helper in all command-issuing sites to prevent this class of bug.
4. **Movement system guard**: The movement system should not advance position for units whose current command is `AttackTarget` and who are in Aiming, Firing, or Cooldown attack phases (per `features/combat_system.md` attack phase spec: UnitBaseSourceActions = Turning only during Aiming, None during Firing/Cooldown).
5. **Chase behavior**: If a unit is ordered to attack an out-of-range target, it should move toward the target until in range, then stop and fire. This aligns with the `AttackingObject` behavior in `features/unit_commands_and_behaviors.md`.

## Justification
This is a gameplay bug that violates the combat phase specification (`features/combat_system.md`): during Aiming phase, UnitBaseSourceActions is "Turning only" (no Moving); during Firing/Cooldown, it is "None." Units sliding across the map while firing is visually broken and mechanically incorrect. Forum topic `forum/units_moving_while_attacking.md` has consensus from qa, developer, designer, and project_manager.

## QA Steps
1. Spawn player infantry units and enemy units within attack range.
2. Select player units and right-click an enemy unit.
3. Verify units stop moving immediately and begin attacking (no sliding).
4. Verify unit velocity is zero while attacking (check that units don't drift).
5. Give units a move command (right-click ground, units start walking).
6. While units are moving, right-click an enemy unit.
7. Verify units stop at their current position and begin attacking — they do NOT continue along the old path.
8. Order units to attack an enemy that is out of range.
9. Verify units move toward the target, stop when in range, then begin firing.
10. Verify auto-targeting also stops movement: place units near enemies with no explicit command, confirm that when auto-target acquires a target, the unit stops and fires (if it was moving).

## Expected Experience
When a player right-clicks an enemy unit, the selected units should visibly stop in place and begin their attack animation/sequence. There should be no sliding, drifting, or movement along a previous path. If the target is out of range, units should walk toward it and stop at attack range before firing. The transition from "moving" to "attacking" should be crisp and immediate — the player should see their units plant their feet and shoot.
