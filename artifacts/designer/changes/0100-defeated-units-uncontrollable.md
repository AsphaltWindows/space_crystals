# 0100 — A defeated player's units stay but can't be controlled

- **Date**: 2026-11-24T05:00:00-05:00
- **Answers**: Q0029 (point 1; point 2 still open)
- **Docs**: `core/game_rules.md > On Defeat` (new)

## What changed and why
The user chose B. When a player is defeated (they own no buildings), their remaining units **stay on the map**, still owned by that player:
- Their orders are cleared, and **nobody can command them** again: not the defeated player, not their allies.
- They act as idle units. They stand where they are and defend themselves through normal auto-targeting until destroyed.
- A defeated player therefore can't rebuild, since their workers can't be ordered.

## Implementation impact
- This replaces the Director's proposal, which removed the units.
- On defeat: clear every unit's command queue and block all commands to that player's units.
- Their units keep their alliances and hostility as before. That's an assumption, marked TBD in the docs.
- Their units no longer affect anything that depends on owning buildings.

## Verification
1. In a test game, destroy all of one player's buildings while they have Peacekeepers in the field. The Peacekeepers stop and stay.
2. Their owner can't select them to command them.
3. They shoot enemies that come into range.
4. They can be destroyed normally.
