# 0172 — Riot Shield raised mid-attack is deferred

- **Date**: 2026-11-24T21:50:00-05:00
- **Answers**: Q0020 (point 3)
- **Docs**: `factions/gdo/units.md > Peacekeeper > Riot Shield` (During the toggle)

## What changed and why
Option A (matches Convert, 0153). Pressed during Firing/Cooldown, the raise waits for the shot to finish and then starts immediately.

## Implementation impact
Pending-toggle flag consumed at Cooldown end.

## Verification
Peacekeepers fighting, press the shield key: every one raises its shield, some a few frames after the others.
