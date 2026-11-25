# 0208 — Laser Battery: always charging; weaker surges on an empty battery

- **Date**: 2026-11-25T04:25:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Laser Battery > Battery`

## What changed and why
User: the battery is **always charging**, attacking or not. A normal surge draws 2× what charges in one surge interval, so attacking drains a net 1× the charge rate. On an empty battery the unit keeps its normal surge rhythm, but each surge uses only what has charged since the last one (half a normal surge's charge), giving ~63% of normal surge damage by the ^(2/3) rule. This refines 0207's "charges when not attacking".

## Implementation impact
Concept only; battery numbers TBD. Rule: each surge spends min(normal surge charge, current charge), and its damage ∝ spent^(2/3).

## Verification
—
