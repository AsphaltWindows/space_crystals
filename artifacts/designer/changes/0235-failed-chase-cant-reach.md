# 0235 — AttackMove: a chase given up by the Failure rule counts as "can't reach"

- **Date**: 2026-11-25T09:05:00-05:00
- **Answers**: Q0073
- **Docs**: `systems/controls/behaviors.md > AttackMovingToLocation > Algorithm` (step 3), `> AttackingObject > Blocked by its own side` (wording)

## What changed and why
User chose **A**. A chase given up by the Failure rule puts the enemy in the reach rule's rejected set. Example: the way is blocked by own units, which the path check can't see. The enemy is judged again only on 0230's change triggers: the enemy moves meaningfully (placeholder 1 GU), the obstacle knowledge changes, or PathReference is replaced. Within weapon range it is always fought. This ends the corridor cycle.

An unseen swap of identical enemies costs at most one fresh attempt. That is accepted as a one-off, not a different steady behaviour.

0223's "drops the target for the leg (0204)" wording is updated to match 0230.

## Implementation impact
As M3-L4 proposes: the failed chase enters the rejection cache.

## Verification
Four Peacekeepers attack-move through a 1-tile corridor at a building. The rear ones give up once (about 7 s), walk on, and don't turn back while the front one still fires.
