# 0487 — An empty worker's right-click on a damaged or unfinished drop-off: its own rules decide

- **Date**: 2026-11-28T15:20:00-05:00
- **Answers**: Q0194
- **Docs**: `systems/mechanics/gathering.md > Job rules > Drop Off order` (new paragraph after "Empty Drones")

## What changed and why
Option (a), as built. The gathering doc's "an empty worker's right-click on its drop-off is a plain Move" is the Drone's case: Drones can't repair. The user noted that if a worker can repair, **Repair is the default right-click on a damaged own building**.

So once Drop Off doesn't apply, because the worker carries nothing, the worker's own right-click rules decide:
- an empty Recruit **repairs** a damaged Storage;
- it **assists** an unfinished one;
- otherwise it is a Move.

A carrying worker still delivers.

## Implementation impact
None; this matches the build. The 0484 tests can cover damaged and unfinished Storages too.

## Verification
1. Right-click a damaged Storage with an empty Recruit. It repairs.
2. Right-click an unfinished Storage. It assists.
3. Right-click an undamaged, complete Storage. It walks there.
4. Do the same with a loaded Recruit. It delivers its load.
