# 0175 — Several buildings waiting for capacity: fixed internal order

- **Date**: 2026-11-24T22:15:00-05:00
- **Answers**: Q0047 (point 1)
- **Docs**: `factions/gdo/overview.md > Unit Control`; `factions/colonists/overview.md > Beacon Capacity > Reserved when production starts`

## What changed and why
Option A (current build). When room opens for fewer units than are waiting, buildings start in a fixed internal order (lowest id). Applies to all factions.

## Implementation impact
None.

## Verification
Deterministic: replaying the same situation gives the same building first.
