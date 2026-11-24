# Q0049: Convert pressed again while a conversion is deferred

- **Date**: 2026-11-24T16:23:35-05:00
- **Blocking**: no. M3-R is built with the current behaviour below.
- **Docs**: `factions/colonists/units.md > Weapon Conversion > Eligibility` (deferred, not rejected; 0153), `> Completion` (cannot be cancelled once initiated).

## Question
A Legionnaire is in its Firing or Cooldown phase when R (Convert) is pressed, so its conversion waits until Cooldown ends (0153). What does a **second press of R** before then do to that unit?

"Once initiated, the conversion runs to completion and cannot be cancelled" covers a conversion that has started. It doesn't say whether a deferred one has been "initiated".

**Current behaviour:** the second press leaves the waiting unit out, so it still converts when its Cooldown ends. Units already converting are also left out, as "cannot be cancelled" says. The Riot Shield is different: there, a second press mid-toggle reverses (0171).

## Context
This came up while building 0153 (M3-R): a pending conversion is stored on the unit and applied when Cooldown ends.

## Options
- A, as now: the second press doesn't touch a waiting unit; it converts after its shot.
- B: the second press cancels the waiting conversion, so the unit stays as it is.
