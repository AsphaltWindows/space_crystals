# 0399 — The A: Attack button shows whenever any selected group can attack

- **Date**: 2026-11-27T13:20:00-05:00
- **Answers**: —
- **Docs**: `systems/controls/interface.md > CommandPanel > Standard Slot Assignments` (A: Attack, on every unit)

## What changed and why
User: "attack option should be present if there's even 1 group that's able to attack". A: Attack is **shown** on the panel whenever at least one selected group can attack, whichever group is in focus, and it issues to the whole selection (0397, 0398).
- It is hidden only when no selected unit can attack.
- This replaces 0398's "the panel has no A, but the key still works".

## Implementation impact
- The panel's A slot is computed from the whole selection, not the ActiveGroup.

## Verification
- Select Drones and Militants with the Drones in focus: A is shown. Pressing it makes the Militants attack and the Drones move.
- Select Drones alone: there is no A.
