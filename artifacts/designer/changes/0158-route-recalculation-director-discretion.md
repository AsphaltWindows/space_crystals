# 0158 — Route recalculation timing is the Director's call

- **Date**: 2026-11-24T19:55:00-05:00
- **Answers**: Q0033 (point 2, revises 0157)
- **Docs**: `systems/controls/behaviors.md > PathKnowledge` (When to recalculate a route; discovery bullet)

## What changed and why
**Replaces 0157's fixed "no re-plan" rule.** The user leaves all route-recalculation timing to the Director: re-planning when a shortcut opens, and whether a newly seen obstacle triggers a re-route on sight or only once the unit is blocked. Goals: no performance issues, and movement that is predictable for the player. User's first instinct, for reference: recalculate only when actually blocked or stuck.

## Implementation impact
Current M2-E behaviour is acceptable; the Director may change it within those goals without a design question.

## Verification
—
