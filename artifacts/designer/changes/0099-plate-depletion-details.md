# 0099 — Plate depletion details

- **Date**: 2026-11-24T04:50:00-05:00
- **Answers**: Q0028 (points 2–4; Q0028 is now fully answered)
- **Docs**:
  - `factions/gdo/economy.md > ExtractionPlate`: "Last partial load", "Depleted patch under a plate", "Selecting a covered patch"; the "Paid in lumps" paragraph moved below PowerPenalty
  - `core/map.md > SpaceCrystalsPatch` (exception noted)

## What changed and why
- **Point 2:** the user chose A. When less than a full load is left, the payment takes only what's left, e.g. 5 out of a 1505 patch. After that, the plate mines at the residual rate.
- **Point 3:** already answered by the docs, and now stated explicitly. A depleted patch stays under its plate with 0 crystals, and the plate mines at the residual rate forever. The patch is removed only when the plate is destroyed. This is GDO's exception to "a depleted patch disappears". The general rule is unchanged for other factions.
- **Point 4:** the user chose A. A click on the plate's tile selects the **plate**. The covered patch can't be selected on its own, and the plate's InfoPanel shows the patch's remaining crystals.

## Implementation impact
- Matches the Director's proposals for points 2–4.

## Verification
1. Give a patch 25 crystals in the editor and put a plate on it. The payments are +10, +10, +5, then +1 every cycle.
2. Destroy the plate after depletion. The patch disappears.
3. Click a plate. The plate is selected, and its panel shows the patch's remaining crystals.
