# 0084 — Destroyed GDO producers refund as if cancelled

- **Date**: 2026-11-24T02:55:00-05:00
- **Answers**: Q0024 (point 4; points 5–7 still open)
- **Docs**:
  - `factions/gdo/structures.md > Barracks > Destroyed while producing` (new)
  - `factions/gdo/structures.md > DeploymentCenter > Cancellation` (new bullet)

## What changed and why
The user chose A. When a GDO producer or constructor is destroyed, everything in progress is refunded **as if it had been cancelled**:
- a Barracks queue (and Airfield and other unit queues): full crystal refund, and Unit Control released;
- the DC's building under construction: full refund;
- the DC's ready-to-place building: 75%, rounded down;
- an Extraction Facility's plate: the same rules as the DC.

This matches research, which is already refunded in full on destruction.

## Implementation impact
- On destruction, run the same refund path as the cancel commands for every queued, in-progress or ready item.
- **TBD:** a finished unit waiting for a blocked exit doorway (0080) when the building dies. It's being asked now. Until it's decided, refund it with the queue.

## Verification
1. Queue 5 Peacekeepers (250 crystals) at a Barracks, then destroy the Barracks. Crystals go up by 250, and Unit Control drops by the queued units' cost.
2. Start a Generator at the DC, then destroy the DC. The full cost is refunded.
3. With a ready-to-place Generator at the DC, destroy the DC. 75% is refunded.
