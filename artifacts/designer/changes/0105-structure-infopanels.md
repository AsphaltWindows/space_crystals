# 0105 — Default InfoPanel for structures

- **Date**: 2026-11-24T05:50:00-05:00
- **Answers**: Q0030 (point 3; Q0030 is now fully answered)
- **Docs**: `core/entities.md > Object Type > InfoPanel` (Default InfoPanel for structures)

## What changed and why
The Director's proposal is accepted. These are the defaults, unless a structure's own section says more. For example, a plate shows the patch's remaining crystals.
- **Own structure:** name, HP, and current activity: construction or production progress, queue length, a ready-to-place building, and the rally point.
- **Another player's structure:** name, owner and HP only. Production information is never shown.

## Implementation impact
- As proposed.

## Verification
1. Select your own Barracks while it's producing. It shows its name, HP, progress, queue length and rally point.
2. Select an enemy Barracks. It shows only its name, owner and HP.
