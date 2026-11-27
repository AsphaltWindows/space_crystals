# 0432 — Subterranean Transport: no firing out; passengers not in the network; Enter surfaces it first

- **Date**: 2026-11-26T19:35:00-05:00
- **Answers**: Q0166
- **Docs**: `factions/syndicate/units/subterranean_transport.md > AttackAttributes`, `> Entering the Tunnel Network loaded`; `factions/syndicate/tunnels.md > Tunnel Network` (Regeneration inside)

## What changed and why
- **1 (a)**: passengers **don't fire** out of the Transport.
- **2 (a)**: passengers **aren't in the network** themselves: no regeneration, no Null Chamber turn, they come out unbuffed (the assumption is now confirmed).
- **3 (c)**: an **Enter order given while burrowed** makes the Transport **surface by itself** (48 frames, Surface rules), then drive in through Side A.

## Implementation impact
1, 2 match the placeholders. 3 replaces placeholder (a) ("surface it manually first"): the Enter order now chains Surface → Enter. If surfacing is impossible where it stands (building, untraversable tile, overlapping unit), the order fails as a Surface would.

## Verification
- A loaded Transport next to enemies: no shots from it.
- Wounded passengers ridden into the network for a minute come out still wounded and unbuffed; the Transport itself healed.
- Burrowed Transport, right-click own Tunnel: it surfaces (3 s) then drives into Side A.
