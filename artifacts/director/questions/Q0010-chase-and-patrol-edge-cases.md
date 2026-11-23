# Q0010 — Chasing into fog; Patrol origin and failed legs

- **Date**: 2026-11-23T15:47:30-05:00
- **Blocking**: no. Proceeding on the defaults below. Needed before the behaviours slice of M1.
- **Docs**: `systems/controls/behaviors.md > AttackingObject`, `> Patrolling`, `systems/controls/commands.md > Patrol`

## Question
1. A unit is chasing an attack target, and the target leaves the vision of the unit's owner. What does the chasing unit do?
2. Patrol has an origin. Is it the unit's position when the Patrol is issued, or when the Patrol starts executing? The two differ when the Patrol is Shift-queued behind other orders, and `commands.md > Patrol` and `behaviors.md > Patrolling` step 1 read differently.
3. What happens when a patrol leg can't be completed?

## Options
**Chasing into fog:**
- **A (default)**: go to the target's last seen position, then the order completes.
- **B**: the order completes at once.
- **C**: keep chasing the target's real position. This leaks information.

**Patrol origin:**
- **A (default)**: the position when the Patrol starts executing.
- **B**: the position when it was issued.

**Failed leg:**
- **A (default)**: a failed leg counts as arrived, and the patrol turns around. Two failed legs in a row end the Patrol.
- **B**: the Patrol ends at the first failed leg.
