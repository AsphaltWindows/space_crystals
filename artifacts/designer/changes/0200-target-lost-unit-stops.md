# 0200 — A target lost into the fog ends the Attack/Follow; the unit stops

- **Date**: 2026-11-25T02:45:00-05:00
- **Answers**: Q0051
- **Docs**: `systems/controls/behaviors.md > AttackingObject > Algorithm` (step 6), `> MovingToObject` (Unit TargetObject: Follow)

## What changed and why
User: when an Attack target goes out of the owner's vision, the unit just stops, as it does when the target is destroyed. Pursuing into the fog is the player's decision. This **replaces 0034's walk to the last seen position**. Follow does the same: a followed unit that leaves vision ends the Follow.
- The behavior completes at once. The unit stops where it is and is idle, **or goes on to its next queued order**.
- While idle, normal auto-targeting applies (IdleLeashDistance), so an enemy reappearing nearby is engaged.
- The fog delay stays per tile (Q0051 option A): no held-target linger and no lingering of other objects.

## Implementation impact
- Remove the MovingToLocation-to-last-seen fallback from AttackingObject; complete the behavior instead.
- FollowingObject: losing sight of the target completes the behavior.
- Rally points on enemy objects are unchanged (`selection_and_hud.md > rally on an enemy object`): a unit produced after the enemy was lost still moves to the rally location.

## Verification
- A Peacekeeper attacks a fleeing enemy that walks out of sight. It stops right there, with no walk into the fog.
- With a Shift-queued Move after the Attack, it goes on to that Move.
- When the enemy steps back into view near the idle Peacekeeper, it is engaged.
