# 0232 — Cancelled Tunnel upgrades refund the current top price (corrects 0231)

- **Date**: 2026-11-25T08:35:00-05:00
- **Answers**: Q0060 (correction to 0231's refund rule)
- **Docs**: `factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs` (Refunds are the current top price)

## What changed and why
I misread the user in 0231. The correct rule: **Cancel Upgrade refunds the price that upgrade would cost if ordered right now, not counting itself**, whichever Tunnel is cancelled.
- Tier 2: 2 + 2 × (other T2+ Tunnels owned).
- Tier 3: 3 + 3 × (other T3 Tunnels owned).
- Example: A and B start Tier 2 together (pay 2, then 4). Cancelling either first refunds 4; cancelling the other then refunds 2.

The Supplies sunk in what the player still has always equal the formula's total, so start/cancel order neither gains nor loses Supplies. **Replaces 0231's "refund exactly what was paid".**

My extension, flagged for the user: Tunnel construction cancels refund the same way (current construction price, not counting itself).

## Implementation impact
- The refund is computed at cancel time from the current counts, not stored per upgrade.
- Destruction mid-upgrade still loses the Supplies.

## Verification
- A and B upgrade together. Cancel B: refund 4. Then cancel A: refund 2.
- In the other order, cancel A first: refund 4, then B: refund 2.
