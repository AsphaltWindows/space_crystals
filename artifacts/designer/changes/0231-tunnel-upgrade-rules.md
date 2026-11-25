# 0231 — Tunnel upgrades: in-progress count, refund what was paid, HP, destruction, reserved area

- **Date**: 2026-11-25T08:20:00-05:00
- **Answers**: Q0060 points 3–6 (points 1–2: 0229)
- **Docs**: `factions/syndicate/tunnels.md > Cost Scaling > Upgrade Costs`

## What changed and why
User accepted the Director's proposals:
- **Pt 3:** a Tunnel mid-upgrade already counts toward the next upgrade's cost (two upgrades started together cost 2 and 4).
- **User addition:** Cancel Upgrade refunds exactly what *that* upgrade cost when ordered, never a recalculated price. Paid prices are never adjusted, so start/cancel order can't gain Supplies. My reading of the user's concern; the same applies to Tunnel construction.
- **Pt 4:** on completion, current HP rises with max HP (damage stays).
- **Pt 5:** destroyed mid-upgrade means the Supplies are lost.
- **Pt 6:** the target tier's larger area is reserved from the start of the upgrade.

## Implementation impact
- Each upgrade (and construction) stores its paid price and refunds exactly that.
- The rest matches the Director's proposals.

## Verification
- Start upgrades on Tunnels A then B (pay 2, then 4). Cancel A: refund 2. Cancel B: refund 4.
- A Tunnel at 500/600 HP upgraded to Tier 2 ends at 700/800.
