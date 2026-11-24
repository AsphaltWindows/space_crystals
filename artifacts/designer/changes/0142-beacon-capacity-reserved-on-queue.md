# 0142 — Beacon Capacity is reserved when queued

- **Date**: 2026-11-24T17:40:00-05:00
- **Answers**: Q0041 (point 1)
- **Docs**: `factions/colonists/overview.md > Beacon Capacity > Reserved when queued` (new)

## What changed and why
Option A. The capacity check happens at queue time and the unit's cost is reserved immediately (counted as used) until produced or cancelled. Matches GDO Unit Control (0082) and the 0112 example.

## Implementation impact
Queue command refused if used (incl. reservations) + cost > available. Cancel releases the reservation.

## Verification
At 8/10, queue two Drones: 10/10 immediately; a third is refused. Cancel one: 9/10.
