# 0233 — Agent-built Tunnels pay when the ghost is placed; ghosts count toward the price; construction refunds at the top price

- **Date**: 2026-11-25T08:45:00-05:00
- **Answers**: Q0061 points 1 and 2 (other points pending)
- **Docs**: `factions/syndicate/tunnels.md > Cost Scaling > Construction Cost`, `> Upgrade Costs` (refunds)

## What changed and why
User:
- **Payment:** a Tunnel build order pays its Supplies when it is given, i.e. when the ghost is placed. Payment is not on arrival.
- **Ghosts count toward the price:** lingering Tunnel ghosts, and Tunnels under construction, count as owned Tunnels for later orders. Two orders in a row cost 1 and 2.
- **Refusal:** an order the player can't pay for is refused with feedback.
- **Refunds:** cancelled, replaced or failed orders, and cancelled constructions, refund the **current top price** (as with upgrades, 0232).

This replaces the Director's on-arrival proposal (Q0061 pt 1) and removes the need for pt 6 (can't pay on arrival).

## Implementation impact
- Charge at order time.
- Count ghosts plus sites plus Tunnels for pricing.
- Refund at the current top price on cancel or failure.

## Verification
- With 3 Supplies and 1 Tunnel, order two Tunnels with Shift. Both ghosts appear and 1 + 2 Supplies are deducted at once.
- Cancel one order: 2 come back.
