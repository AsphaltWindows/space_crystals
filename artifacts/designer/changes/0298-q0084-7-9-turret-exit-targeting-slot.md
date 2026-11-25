# 0298 — Laser Turret: the Agent's exit, targeting, and no slot for Agent-built turrets

- **Date**: 2026-11-26T11:45:00-05:00
- **Answers**: Q0084 points 7–9 (Q0084 complete: 0296–0298)
- **Docs**: `factions/syndicate/structures/laser_turret.md > Construction`, `> Targeting`

## What changed and why
User chose **A** for all three:
7. The Agent exits at the first clear adjacent spot, searched in a fixed order from the side it came from; if none is clear, it waits inside.
8. Targeting works like a unit holding position: BaseAutoTargeting within Range, then the closest. It keeps its target while that target stays valid and in range.
9. An Agent-built turret doesn't use a Tunnel's operation slot.

## Implementation impact
As proposed.

## Verification
—
