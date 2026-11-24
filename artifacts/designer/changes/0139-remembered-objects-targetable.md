# 0139 — Remembered objects can be targeted

- **Date**: 2026-11-24T17:20:00-05:00
- **Answers**: Q0044 (point 1; also settles point 6); Q0017 (point 2)
- **Docs**: `core/vision.md > Explored > Targeting remembered objects` (new); `systems/controls/selection_and_hud.md > Rally points on enemy objects` (remembered enemy structures)

## What changed and why
User: B for patches, and generally **remembered structures/objects can be targeted** (right-click, Attack, Gather, Move to object, rally), though still not selectable (GDO station exception unchanged).
- The order goes to the last-known position. On sight: still there → carries on; gone → ends as for a vanished target (Gather: redirect rules; Attack: stop and take next queued order / idle).
- No leaks: everything treats the remembered object as present until seen otherwise.
- Q0044 pt 6 (Egress Node sight vs reference row) no longer matters: Gather works on remembered patches.
- Rallies: may be set on remembered enemy structures. A rally on an enemy **structure** stays on it when it leaves sight (structures don't move) rather than becoming a ground rally; 0088's ground-rally conversion remains for enemy **units**. (Designer's consequence of the decision; same practical position either way.)

## Implementation impact
Replaces the command-gate assumption "object targets only when currently visible". Right-click on a remembered enemy structure = Attack (not Move to spot).

## Verification
- Scout a field, leave; right-click a greyed patch with Drones: they walk there and mine.
- Right-click a remembered enemy building with Peacekeepers: they walk there and attack if it's still standing; if it was destroyed unseen, they stop on arrival.
- Rally a Barracks on a remembered enemy building: new units go attack it.
