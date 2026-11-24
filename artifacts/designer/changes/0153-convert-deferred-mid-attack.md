# 0153 — Convert during Firing/Cooldown is deferred

- **Date**: 2026-11-24T19:15:00-05:00
- **Answers**: Q0042 (point 9)
- **Docs**: `factions/colonists/units.md > Weapon Conversion > Eligibility`

## What changed and why
Option B (replaces "rejected during Firing/Cooldown"). A unit in Firing or Cooldown when Convert is pressed converts right after its Cooldown ends, instead of entering Reloading. Leaving such units out would make the group impossible to unify (a second R flips everyone).

## Implementation impact
Per-unit pending-convert flag consumed at Cooldown end. Applies to single units too.

## Verification
10 Sabres fighting, press R once: all 10 end up Rifles; a few convert 1–4 frames after the others.
