# 0445 — Rocketman missiles are area damage (normal and locked); Sparrows may dodge normal ones

- **Date**: 2026-11-26T21:25:00-05:00
- **Answers**: Q0173
- **Docs**: `factions/gdo/units/rocketman.md > AttackAttributes (normal attack)`, `> Placeholder stats`

## What changed and why
- **Q0173: option 1, keep** the aiming as is. The user accepts that normal missiles "might miss all the time against sparrows specifically"; against aircraft, Rocketmen use Target Lock (and a spotter against strafers). No sight or lock-time change, no leading. 0356's "sometimes dodges" now reads "almost always dodges".
- **Area damage** (user): the normal missile bursts at its spot (AoE, friendly fire as all area damage), and **locked homing missiles are area damage too**, bursting on the target. With many Sparrows, bursts may catch some.
- **Burst domain** (user confirmed): a missile keeps the domain of what it was fired at (air → hits only air; ground unit or ground → only ground/underground). Empty spot: harmless burst. No "attack air" command (can't aim at empty sky).
- **Numbers** (designer placeholder, user asked for more damage): Damage **40 total**, AoERadius **32 su** (was 20 single-target). Roughly 30 to a 64x40 vehicle square-on, 20 to a Sparrow, 7 to an infantry unit, before armor.

## Implementation impact
- Rocketman: TargetType SingleTarget → AoE (both missiles), Damage 20 → 40, new AoERadius 32 su, domain fixed at fire time.
- The Hand of Heaven (also DoublyDisjointed) is unchanged.

## Verification
- Rocketmen fire at a tight clump of enemy infantry: several take damage per missile; own infantry next to the target take some too.
- A missile fired at a Sparrow that flies on bursts harmlessly; a ground unit under that spot is not hit.
- A locked missile on a Sparrow flying in a group of Sparrows damages the neighbours in the 32 su radius.
