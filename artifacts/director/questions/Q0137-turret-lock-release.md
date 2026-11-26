# Q0137 — When does a unit's turret let go of its locked target, and does a Move clear it?

- **Date**: 2026-11-26T03:39:38-05:00
- **Blocking**: no — M5-U1 built the reading below, marked Placeholder.
- **Docs**: `systems/controls/commands.md > TurretCommandState`; `systems/controls/behaviors.md > TurretAutonomousScanning`, `> BaseAutoTargeting > Not active during`; `systems/controls/command_behaviors.md > AttackingObject`

## Question
1. **When does a turret drop its LockedTarget,** when the target moves out of the turret's range or arc, or out of sight, and the base isn't engaging it?
2. **Does a Move (or another command that sets no target) clear the turret's lock?** If not, a lock from an explicit Attack, even one on a friendly unit, stays through the Move.

## Context
- **`commands.md > TurretCommandState`:** "When the base behavior does not specify a target, the turret retains its previous TurretCommandState. If the turret has no assigned target, it falls back to autonomous target scanning."
- **AttackingObject** sets "TurretCommandState.LockedTarget = TargetObject" only "When targetable" (in range, beyond MinRange, in the arc). So while the unit approaches, the turret keeps its previous lock, or scans. M5-U1 built that literally.
- **No section says when a lock ends,** other than by a new one.
- **M5-U1's reading (Placeholder):** a turret keeps a lock its base is engaging. It keeps any other lock only while it can fire at it (in range, in its arc, seen by its owner). Otherwise it drops the lock and scans for a new target in the same tick.
- **What retention means for a Move:** a lock set by an explicit Attack, even on a friendly unit, survives a following Move, as long as the turret can still fire at it. An enemy lock kept on the move seems intended ("firing on the move"). A friendly one is probably not.
- **Also built, technical:** a turret turns only toward a lock its owner sees, so its barrel never points at a hidden target.

## Options
- **(a) As built:** a lock lasts while the turret can fire at it, or while the base engages it. Then the turret scans. Commands that set no target leave the lock alone.
- **(b) As (a), except that a Move, Stop or new order clears a lock the base set for an explicit Attack.** The turret then scans for enemies, so a friendly lock never outlives its Attack.
- **(c) Something else,** such as dropping a lock only when its target dies or goes out of sight.
