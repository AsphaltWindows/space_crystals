# 0409 — Shielded Peacekeepers follow an explicit Attack's target

- **Date**: 2026-11-26T16:00:00-05:00
- **Answers**: Q0151 part 1
- **Docs**: `factions/gdo/units/peacekeeper.md > Riot Shield (toggled ability)` (Explicit Attack in a mixed selection)

## What changed and why
The user chose (a): "Attack goes to everyone" (0397/0398) applies to shielded Peacekeepers too. The old line that left them out of an explicit Attack was stale and is replaced. A shielded Peacekeeper can't attack, so it treats the Attack as a Move: it follows the enemy with the group, as on a right-click while shielded, and the held Attack resumes when the shield is lowered (existing "Orders are kept" rule).

## Implementation impact
None: matches the M5-S5 build described in Q0151.

## Verification
Select shielded and unshielded Peacekeepers with some other units, A + click an enemy: everyone heads for it, the unshielded ones fire, the shielded ones follow. Lower the shields: they start attacking that target.
