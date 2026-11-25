# 0223 — Attackers blocked by their own side: no special crowd logic

- **Date**: 2026-11-25T06:45:00-05:00
- **Answers**: Q0057
- **Docs**: `systems/controls/behaviors.md > Core principle: predictable, not optimal` (new), `> AttackingObject > Blocked by its own side` (new)

## What changed and why
User principle, now in the docs: **default behavior must be predictable, not optimal**. No algorithms that optimise behavior for the situation. Managing bottlenecks is the player's job, and units differ in how much oversight they need.

Q0057:
- **Pt 1: A** (as now). An attacker blocked within 3 GU and out of range gives up. An explicit Attack completes and the unit is idle; it re-acquires by idle auto-targeting (0181). An AttackMove drops the target for the leg (0204).
- **Pt 2: B.** Farther out, it goes round occupied space if there is a way. If there isn't, the normal Failure rule applies (re-route attempts, give up after ~7 s). **There is no indefinite waiting behind blockers.**
- **Pt 3: A** (as now). A few attempts, with no re-checking for spots opening elsewhere.

## Implementation impact
- **Remove M3-G2's indefinite wait** (the wait in place since M1 slice E). A ranged unit with no way round follows the Failure rule.
- Keep the going-round when a way exists; that is ordinary pathfinding.
- Pt 1 and pt 3 are unchanged.

## Verification
- Four Peacekeepers in a 1-tile corridor attack a building: the front ones fire. The ones behind with no way round stop after about 7 s, idle, and fire by themselves if the target is in range.
- Eight Sabre Legionnaires on a Deployment Center: the blocked ones give up, then re-acquire on their own.
