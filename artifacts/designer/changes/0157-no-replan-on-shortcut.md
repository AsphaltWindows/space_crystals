# 0157 — No re-plan when a shortcut opens

- **Date**: 2026-11-24T19:50:00-05:00
- **Answers**: Q0033 (point 2)
- **Docs**: `systems/controls/behaviors.md > PathKnowledge`

## What changed and why
Option A (current build). Seeing a remembered obstacle gone doesn't trigger a new path; line-of-sight corner cutting still applies. User principle: avoid recalculating routes unless the unit actually gets blocked or stuck.

## Implementation impact
None for this case. See the follow-up on re-routing at obstacle discovery (designer will confirm whether that should also wait until actually blocked).

## Verification
Destroy a wall the player remembers; a unit routed around it continues around it.
