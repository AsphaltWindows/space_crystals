# 0368 — GDO rule confirmations: homing Dragonfly and Missile Turret, dodgeable Hornet, AoE drone, Prospector deploy, Bunker sight, First Aid Tent heals own and allied

- **Date**: 2026-11-27T03:05:00-05:00
- **Answers**: —
- **Docs**:
  - `factions/gdo/structures/{missile_turret,bunker,first_aid_tent}.md`;
  - `factions/gdo/units/{dragonfly,hornet,drone_operator,prospector}.md`.

## What changed and why
User, on 0366's "to confirm" readings:
1. The **Dragonfly and the Missile Turret are homing**. The Hornet was not named, so it becomes **DoublyDisjointed**: dodgeable, and it can Attack Ground. The user is asked to confirm.
2. The drone explodes with **area damage**.
3. **Prospector deployment can't be cancelled.**
4. **Bunker occupants see with the Bunker's sight.**
5. The **First Aid Tent heals own and allied infantry.**

## Implementation impact
- The Hornet's attack type changes from 0366's assumption.
- The rest confirms 0366.

## Verification
—
