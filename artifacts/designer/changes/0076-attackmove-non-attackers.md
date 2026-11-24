# 0076 — AttackMove with units that can't attack

- **Date**: 2026-11-24T01:20:00-05:00
- **Answers**: — (closes the TBD left in 0075)
- **Docs**: `systems/controls/commands.md > AttackMove > Units that can't attack` (new); `factions/gdo/units.md > Peacekeeper > Riot Shield` (TBD resolved)

## What changed and why
AttackMove (A + click on ground) is issued to **every** selected unit, including units that can't attack:
- units with no attack at all (e.g. an unarmed Supply Chopper);
- units that can't attack right now (a Peacekeeper with its shield up).

For those units it never finds anything to engage, so it plays out as a plain move to the location. This keeps a mixed army together, e.g. a shield wall advancing alongside the shooters. The user's framing: a unit without an attack simply never attacks during an attack-move, so this is consistent with 0075. Nothing about the order changes; only the engaging part is empty.

## Implementation impact
- Don't filter units out of AttackMove based on attack capability.
- Units that can't attack never stop to fight, so they may arrive ahead of the rest. That's accepted.
- Contrast with explicit Attack on a target (0075): units that can't attack are left out of it.

## Verification
1. Select 4 shielded Peacekeepers and 4 Rocketmen, press A and click ground beyond some enemies. The Rocketmen stop and fight. The Peacekeepers keep walking to the destination and never fire.
2. Select a Supply Chopper with combat units and attack-move. The Chopper flies to the destination.
