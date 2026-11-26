# 0340 — Cults group construction refunds

- **Date**: 2026-11-26T21:25:00-05:00
- **Answers**: Q0098
- **Docs**: `factions/cults/economy.md > Cults Building Mechanics > Group construction refunds` (new)

## What changed and why
User confirmed the default on all four points:
1. Refund only once no Recruit of the group still holds the order.
2. A full refund if every builder dies before the start.
3. Each Shift-queued building is refunded in full when its last holder drops it before its start.
4. After a cancel, the Recruits go on to their queued orders, otherwise idle.

## Implementation impact
As your proposal (a).

## Verification
- Re-order 2 of 3 builders: no refund.
- Re-order the third: full refund.
