# 0005 — Elevation follow-ups: underground expansions may sit on slopes; structures get the high-ground modifier

- **Date**: 2026-11-23T13:17:51-05:00
- **Answers**: —
- **Docs**: `core/map.md > Elevation > Structures only on flat terrain` (exception added), `core/entities.md > Placement Validation > Direct Placement` (underground bullet), `core/vision.md > ElevationModifier`

## What changed and why
Two open points from 0004, decided by the user:
1. **Syndicate underground expansions are exempt from the flatness rule.** They may be placed on sloped tiles inside a Tunnel Area. Surface Syndicate structures (Tunnel, Laser Turret) still need flat tiles.
2. **ElevationModifier applies to structures too**, as both source and target. A defensive structure on higher ground gets +1 sight and attack range against lower targets. One on lower ground gets -1 against higher targets, and enemies on higher ground get +1 against it. A surface structure's elevation level is its flat footprint's level. Air units remain exempt.

## Implementation impact
- Confirms the assumption stated in 0004 (underground expansions skip the flatness check).
- The ElevationModifier check now covers ground/underground structures, not just units.
- Still open: whether slopes change movement speed (currently no effect).

## Verification
- A defensive structure on a plateau outranges an identical one in the valley by 1 against units in the valley; the valley one has -1 range against the plateau.
- A Syndicate underground expansion can be placed in a Tunnel Area that covers sloped tiles. A Tunnel or Laser Turret on a slope shows a red ghost / fails validation.
