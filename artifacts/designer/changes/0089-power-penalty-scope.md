# 0089 — Power penalty: producers never slowed; ratio and timing

- **Date**: 2026-11-24T03:30:00-05:00
- **Answers**: Q0025 (points 1, 2 and 4; point 3 still open)
- **Docs**: `factions/gdo/overview.md > GDOResources > Power` (three new bullets)

## What changed and why
1. **Point 1:** the user chose A. Power producers, meaning the Deployment Center and Generators, are **never slowed**. Only consumers are slowed:
   - the Extraction Facility, for plate construction;
   - the Barracks and other consuming producers;
   - Extraction Plates, for mining.

   The player can always build their way out of negative power at full speed, so there's no downward spiral.
2. **Point 2**, already in the docs ("in proportion to total available power versus total required power"): the ratio is generated ÷ consumed, capped at 100%. 30/51 ≈ 59% matches the baseline. Now written explicitly.
3. **Point 4**, follows from the docs: a building's Power counts from the tick it is placed to the tick it is destroyed. GDO buildings are placed complete, so there is no partial state.

## Implementation impact
- Matches the Director's proposals for points 1, 2 and 4. Nothing new is required.

## Verification
1. Build an all-in Barracks early so power is −21 (30 generated, 51 consumed). Peacekeeper production and plate construction and mining run at ≈59% speed.
2. A Generator being built in the DC at the same time builds at full speed.
