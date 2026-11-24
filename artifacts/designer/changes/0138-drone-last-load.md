# 0138 — Drone's last load takes the remainder

- **Date**: 2026-11-24T17:10:00-05:00
- **Answers**: Q0037 (point 13; Q0037 done)
- **Docs**: `factions/colonists/units.md > Last load of a patch` (new)

## What changed and why
The Director's proposal: full 32 frames, takes the remaining (< 8) crystals; the patch depletes. Mirrors the plate rule (0099).

## Implementation impact
None beyond the proposal.

## Verification
A patch with 3 left: the Drone mines 2 s, carries 3, the patch disappears.
