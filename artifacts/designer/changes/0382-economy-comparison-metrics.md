# 0382 — Economy comparison: income/s, total mined, bank and spent on economy at each checkpoint

- **Date**: 2026-11-27T10:45:00-05:00
- **Answers**: — (extends 0381 / Q0135)
- **Docs**: `factions/cults/economy.md > Economy check` (scenario 2)

## What changed and why
User: compare the economies "in terms of income/s at that point as well as total amount mined, total amount in the bank, total amount spent on economy".

The comparison covers the 4 lines (Cults ideal, Cults realistic, GDO, Colonists) at the 5 checkpoints (1:30, 2:48, 4:00, 5:00, 6:00). At each one, report:
1. **Income/s**: delivered over the preceding 10 s (160 frames), divided by 10.
2. **Total mined**: delivered since 0:00, excluding the starting bank.
3. **Bank**: unspent crystals.
4. **Spent on economy**: cumulative crystals paid for economy items, net of refunds.

**Economy items** (the designer's working list; flag any you find borderline):
- **Cults:** Halls and Storages beyond the starting ones.
- **GDO:** Extraction Facilities and Plates. Generators are reported separately, not counted.
- **Colonists:** Drones and Egress Nodes.

## Implementation impact
- The oracle reports a table of the four metrics per line and checkpoint.
- No tuning yet: report it as a question.

## Verification
- The report has 4 lines × 5 checkpoints × 4 metrics.
- The figures are consistent: total mined + starting bank = bank + everything spent.
