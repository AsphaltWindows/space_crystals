# 0082 — GDO Unit Control is reserved when a unit is queued

- **Date**: 2026-11-24T02:35:00-05:00
- **Answers**: Q0024 (point 2; points 3–7 still open)
- **Docs**: `factions/gdo/overview.md > GDOResources > Unit Control`

## What changed and why
The user chose A, the Director's proposal:
- A GDO unit takes its Unit Control the moment it is queued, at the same time its crystals are paid.
- Queuing is refused if it would take used Unit Control past 200.
- The HUD's "used" figure includes units that are queued or in production.
- Cancelling a unit gives its Unit Control back.

A GDO queue is therefore never blocked by the cap. This deliberately differs from the Syndicate, whose Tunnel Space is checked when production starts.

## Implementation impact
- Check Unit Control at queue time, next to the crystal cost.
- Release it on cancel, and on any refund when the producer is destroyed. Point 4 of Q0024 is still open.
- When a unit dies, release its Unit Control as usual.

## Verification
1. Get Unit Control near 200, e.g. 198 with a Peacekeeper costing 2 (placeholder). One more Peacekeeper can be queued, and the next is refused even though crystals are available.
2. The HUD shows 200/200 as soon as it's queued.
3. Cancel it. The HUD shows 198 again.
