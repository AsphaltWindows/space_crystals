# 0194 — Shadebringer and its own sludge

- **Date**: 2026-11-25T01:40:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Shadebringer > Sludge`

## What changed and why
User: sludge is laid behind the Shadebringer, so laying the trail doesn't slow it. It is not immune, though: driving back over sludge (its own or another's) slows it like any other vehicle.

## Implementation impact
Sludge must appear behind the vehicle (outside its footprint), not under it. There is no immunity flag.

## Verification
A Shadebringer driving straight with Sludge on keeps full speed. When it turns around and crosses its own trail, it slows down.
