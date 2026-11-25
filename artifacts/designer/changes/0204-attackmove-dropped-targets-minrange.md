# 0204 — AttackMove dropped targets; MinRange back-off; locked Rocketman fires at any distance

- **Date**: 2026-11-25T03:30:00-05:00
- **Answers**: Q0054
- **Docs**: `systems/controls/behaviors.md > AttackMovingToLocation > Algorithm` (dropped target), `> AttackingObject > Target inside MinRange`; `factions/gdo/units.md > Rocketman > Target Lock` (No minimum range when locked)

## What changed and why
User chose A for all three points (for now). The current M3-L2 behaviour is kept and now documented:
1. A target an attack-moving unit dropped (while alive) at the leash is not re-taken for the rest of that leg, unless it comes within weapon range.
2. Backing off from a target inside MinRange goes to the middle of the MinRange–Range band.
3. A unit that can't back off gives up the order with "Target too close".

Also (user): a **locked** Rocketman fires its homing missiles at any distance up to LockRange. The normal attack's MinRange (0203) doesn't apply to locked fire.

## Implementation impact
No change to M3-L2. New for when Target Lock is built: locked fire ignores MinRange.

## Verification
- Attack-move past an enemy that baits at the leash edge. The unit drops it once and walks on, without flickering between the target and its route.
- A Rocketman locked on an adjacent unit keeps firing.
