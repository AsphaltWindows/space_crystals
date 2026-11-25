# 0296 — Laser Turret: hidden only when lowered, 6 sight always, blocks when raised, rises only when clear

- **Date**: 2026-11-26T11:25:00-05:00
- **Answers**: Q0084 points 1–3 (points 4–9 pending with the user)
- **Docs**: `factions/syndicate/structures/laser_turret.md > Pop-up rules`

## What changed and why
User:
1. **A:** hidden, with underground armor, only while fully lowered.
2. **A:** sight 6 in both forms.
3. **New rule** (not A or B):
   - upright turrets and **raised** pop-ups block like buildings, and a lowered pop-up never blocks;
   - a pop-up **can't rise while any unit stands over it**;
   - the **owner's own idle units** over it are pushed off by the idle push;
   - any other unit (an enemy, or an own unit with an order) keeps it underground until clear.

## Implementation impact
- The rise check is a footprint-clear test.
- The idle push triggers for own idle units on the footprint.
- A raised pop-up is a movement obstacle.

## Verification
- An enemy Guard standing on a hidden turret's tile isn't shot by it.
- Your own idle Agent over it is nudged off when an enemy comes in range, and the turret rises.
