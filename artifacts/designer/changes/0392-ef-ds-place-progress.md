# 0392 — Extraction Facility and Defense System: Place button shows progress; Defense System works like the Extraction Facility

- **Date**: 2026-11-27T12:45:00-05:00
- **Answers**: —
- **Docs**: `factions/gdo/economy.md > ExtractionFacility > ObjectInterfaceState[ExtractionFacility]`; `factions/gdo/structures/defense_system.md`

## What changed and why
User: the Defense System "only builds Missile Turrets so it should work more like the Extraction Facility, but for both of those a progress bar on the place button would be nice".
- **Extraction Facility**: while a plate is under construction, **Q: Place Plate** is shown greyed out, with a running progress bar. It lights up when the plate is ready.
- **Defense System**: no build menu; it uses one slot, like the Extraction Facility. This replaces 0366's "constructs like the Deployment Center".
  - Idle: Q Build Missile Turret.
  - Constructing: Q Place Missile Turret (greyed, with progress bar); X Cancel (full refund).
  - Ready: Q enters placement; X Cancel Ready Turret (75%).
- This matches the Deployment Center's Place button (0388).

## Implementation impact
- Client: a progress-bar overlay on Q for both buildings.
- Defense System: its panel changes to the one-slot flow.

## Verification
- Order a plate and stay in the panel: Q shows a filling bar, then lights up.
- The same for a Missile Turret at the Defense System.
