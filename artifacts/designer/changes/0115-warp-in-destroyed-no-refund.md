# 0115 — Warp In destroyed: no refund

- **Date**: 2026-11-24T13:30:00-05:00
- **Answers**: Q0039 (point 7)
- **Docs**: `factions/colonists/economy.md > Warp In > Vulnerability`

## What changed and why
Option A. A building destroyed mid-Warp-In refunds nothing; Beacon and cost are both lost. Unlike GDO's destroyed producers (0084), the unfinished building itself is the object the attacker killed.

## Implementation impact
No refund path on destruction of a Warp In in progress (cancel keeps its 75%).

## Verification
Destroy a building mid-Warp-In: resource pool unchanged, Beacon gone.
