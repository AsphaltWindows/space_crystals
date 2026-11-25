# 0269 — Bomber: needs a Vault and a Chemical Plant, ground only, re-arms in the network; the Vault works network-wide

- **Date**: 2026-11-25T15:30:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Bomber (name TBD)`; `factions/syndicate/structures.md > Vault` (Network-wide)

## What changed and why
User:
- The Bomber requires a **Vault and a Chemical Plant**, and attacks **ground only**.
- **A Vault works across the entire Tunnel Network**, not just for the Tunnel whose area it is in. The same applies to Shadebringer tank recharging.

My reading of re-arming, flagged to the user: after its single bomb, the Bomber lands in any own Tunnel's air door and is re-armed inside the network while the player owns a Vault anywhere. Re-arm time and Vault throughput are TBD.

## Implementation impact
Concept only. Vault effects check "player owns a Vault" globally, not per Tunnel.

## Verification
—
