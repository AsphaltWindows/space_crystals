# 0197 — A second Convert press cancels a deferred conversion

- **Date**: 2026-11-25T02:05:00-05:00
- **Answers**: Q0049
- **Docs**: `factions/colonists/units.md > Weapon Conversion > Eligibility`, `> Completion`

## What changed and why
User chose **B**. Pressing R while a unit's conversion is still deferred (it is waiting for Firing/Cooldown to end) cancels the waiting conversion, and the unit keeps its current weapon. "Initiated" (cannot be cancelled) means the 4-frame conversion has actually started.

In a group, R acts on each unit separately:
- converted units switch back;
- units mid-conversion are left out;
- waiting units cancel.

A double-tap can leave the group split between the two weapons, and that is accepted.

## Implementation impact
This changes the M3-R current behaviour, under which the second press left a waiting unit to convert anyway. Now it clears the pending conversion.

## Verification
Have a Legionnaire fire. During its Firing/Cooldown press R, then R again before it converts. It keeps its weapon. Pressing R only once still converts it after the shot.
