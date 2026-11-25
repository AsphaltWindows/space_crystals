# 0251 — Ejection: longest-inside unit first; queue rules; air doors on the roof; Vault/Null Chamber limits noted

- **Date**: 2026-11-25T11:50:00-05:00
- **Answers**: Q0064 points 3, 4, 11
- **Docs**: `factions/syndicate/tunnels.md > EjectMenu`; `factions/syndicate/structures.md > Vault`, `> Null Chamber > Null Chamber Exit Buff`

## What changed and why
User:
- **Pt 3, A:** of the chosen type, the unit inside the network longest comes out. "Healthiest" was rejected because other per-unit factors (Vault re-arming, Null Chamber re-buff) may matter.
- **Pt 4** as proposed: a click reserves a specific unit, the queue can't be cancelled, and a Tunnel destroyed with a queue leaves the queued units in the network.
- **Pt 11:** the air doors are on the roof, over the matching path halves.
- Noted for later design (user): Vault re-arming and the Null Chamber buff should probably have **throughput limits** (units at a time, and time per unit). TBD.

## Implementation impact
As proposed for M4-C. The Vault and Null Chamber limits are not in scope yet.

## Verification
With two Guards inside (one entered 30 s ago, one 5 s ago), ejecting a Guard brings out the one that entered first.
