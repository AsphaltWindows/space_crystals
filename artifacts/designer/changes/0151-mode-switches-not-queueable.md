# 0151 — Mode switches are not queueable

- **Date**: 2026-11-24T19:05:00-05:00
- **Answers**: Q0042 (point 7)
- **Docs**: `factions/colonists/units.md > Weapon Conversion` (Not queueable); `factions/gdo/units.md > Peacekeeper > Riot Shield` (Not queueable)

## What changed and why
Option A. Convert and the Riot Shield toggle always happen immediately; Shift has no effect and nothing enters the order queue.

## Implementation impact
As proposed.

## Verification
Shift+R on moving Sabres: they convert right away, queue unchanged.
