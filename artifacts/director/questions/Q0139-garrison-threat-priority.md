# Q0139 — Does a Garrison with armed units inside count as a threat to enemy target scans?

- **Date**: 2026-11-26T05:14:42-05:00
- **Blocking**: no. M5-N2 built the literal reading: a Garrison never counts as a threat.
- **Docs**: `systems/controls/behaviors.md > TurretAutonomousScanning > Target Selection Priority` (and `> BaseAutoTargeting`, which uses the same rule); `factions/cults/structures/garrison.md`

## Question
When an enemy scans for a target, is a Garrison with armed units inside a "threatening" target, picked before targets that can't attack the scanner?

## Context
- **The rule:** "**Threatening units first**: Prefer ValidTargets whose AttackAttributes.TargetDomain includes this unit's domain ... These are units that have the capability to attack this unit."
- **The Garrison has no AttackAttributes of its own.** Its occupants fire, each "picks its own target ... as the Laser Turret", but they can't be targeted while it stands.
- **As built,** an enemy prefers any armed unit in its scan step over the Garrison that is shooting it. The Laser Turret, which has its own attack, does count as a threat.

## Options
- **(a) As built:** a Garrison is never a threat.
- **(b) A Garrison is a threat while an occupant inside could attack the scanner:** any occupant's weapon, the Militant's stand-in included, reaches the scanner's domain.
- **(c) A Garrison is a threat whenever anyone is inside.**
