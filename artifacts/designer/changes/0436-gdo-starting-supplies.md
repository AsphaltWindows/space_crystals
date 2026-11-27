# 0436 — The GDO starts with 1 Supply

- **Date**: 2026-11-26T20:05:00-05:00
- **Answers**: — (follow-up to Q0169 / 0435)
- **Docs**: `factions/gdo/economy.md > Starting Conditions`

## What changed and why
With Supply costs now on six GDO units (0435), the user set **starting Supplies to 1** (was 0, a placeholder awaiting Supply costs): enough for one Field Engineer; everything else that costs Supplies waits for the supply loop.

## Implementation impact
Starting bank: 100 Space Crystals, **1 Supply**.

## Verification
New GDO game: the top bar shows Supplies 1; a Field Engineer can be queued once a Career Campus exists; a Prospector cannot until 7 more Supplies arrive.
