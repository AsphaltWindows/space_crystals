# Ticket: Attack Phase State Machine

## Current State
No attack phase system exists. There is no way to model the sequential phases of an attack or constrain unit actions during each phase.

## Desired State
Implement the 4-phase attack sequence as a state machine:

**AttackPhaseEnum**: Aiming -> Firing -> Cooldown -> Reloading

Each phase has:
- Duration (from AttackAttributes: AimDuration, FiringDuration, CooldownDuration, ReloadDuration)
- Interruptible flag
- Allowed actions per attack source type

**Phase definitions**:

| Phase | Interruptible | UnitBase Source Actions | Turret Source Actions | Turret Source Base Actions |
|-------|---------------|------------------------|-----------------------|---------------------------|
| Aiming | Yes | Turning | Turning | Moving, Turning |
| Firing | No | None | None | Moving, Turning |
| Cooldown | No | None | None | Moving, Turning |
| Reloading | Yes | Turning, Moving | Turning | Moving, Turning |

**Phase behaviors**:
- **Aiming**: Unit/turret rotates to face target. Cancelled if target becomes invalid. Interrupted by new commands (interruptible=true).
- **Firing**: Attack effect occurs. Cannot be interrupted. FullyConnected: unified animation+effect. HeadDisjointed: spawns tracking projectile. TailDisjointed: effect at locked location. DoublyDisjointed: spawns projectile to locked location.
- **Cooldown**: Post-fire unresponsive period. Cannot be interrupted.
- **Reloading**: Main delay between attacks. Can be interrupted by new commands.

**Target location locking** (TailDisjointed and DoublyDisjointed):
- Target location is locked at the end of the Aiming phase.
- Effect applies to whatever units are at that location at the end of Firing (TailDisjointed) or when projectile arrives (DoublyDisjointed).

## Justification
Required by `features/combat_system.md`. The phase system governs when units can act during combat, creating tactical depth through interruptibility windows and attack source differences (turret units can move while attacking, non-turret units are locked).

## QA Steps
1. Verify attack phases execute in order: Aiming -> Firing -> Cooldown -> Reloading.
2. Verify Aiming phase is interruptible: issuing a move command during Aiming cancels the attack.
3. Verify Firing phase is not interruptible: issuing a move command during Firing does not cancel the attack.
4. Verify Cooldown phase is not interruptible.
5. Verify Reloading phase is interruptible.
6. For a UnitBaseSource unit: verify it can only Turn during Aiming, cannot Move or Turn during Firing/Cooldown, and can Move+Turn during Reloading.
7. For a TurretSource unit: verify turret can Turn during Aiming, cannot Turn during Firing/Cooldown, can Turn during Reloading. Verify unit base can Move+Turn during all four phases.
8. Verify Aiming cancels if the target becomes invalid (e.g., target dies or leaves visibility).
9. For TailDisjointed attack: verify target location is locked at end of Aiming; moving the target after lock causes a miss.
10. For DoublyDisjointed attack: verify target location is locked at end of Aiming; projectile travels to locked location regardless of target movement.

## Expected Experience
The attack state machine progresses through phases with correct durations. Interruptible phases respond to new commands; non-interruptible phases ignore them. UnitBaseSource units visibly stop moving during Aiming/Firing/Cooldown. TurretSource units continue moving freely while the turret attacks. Location-locking attacks demonstrate dodge-ability when targets move after the Aiming phase concludes.
