# 0103 — Enemy structures are valid automatic targets

- **Date**: 2026-11-24T05:25:00-05:00
- **Answers**: Q0030 (point 1; points 2–3 still open)
- **Docs**: `systems/combat.md > ValidTarget` (opening sentence)

## What changed and why
The user chose A. ValidTarget covers enemy **units and structures**. Structures that can't attack are non-threatening, so threatening targets come first, and that includes defensive structures that can attack. Attack-move and idle units therefore clear out enemy bases on their own.

## Implementation impact
- Matches the Director's proposal and the current M1 behaviour. The doc wording is fixed.

## Verification
1. Attack-move Peacekeepers into an enemy base with a Peacekeeper defending it. They shoot the defender first, then the buildings.
2. Idle units next to an enemy Generator attack it.
