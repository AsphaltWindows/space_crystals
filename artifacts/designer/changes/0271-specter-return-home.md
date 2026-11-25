# 0271 — Specter flies home to re-arm by default

- **Date**: 2026-11-25T15:50:00-05:00
- **Answers**: —
- **Docs**: `factions/syndicate/units.md > Specter > Attack` (Out of ammo)

## What changed and why
User: after dropping its single bomb, the Specter's default behavior is to fly back to the nearest own Tier 3 Tunnel, land, and re-arm inside, even with no Vault (it then waits inside unarmed). With no Tier 3 Tunnel, it circles in place.

## Implementation impact
- New default behavior on ammo empty: path to the nearest T3 Tunnel air door, then land (enter the network).
- Interaction with the player's other queued orders: TBD.

## Verification
- A Specter bombs a target, then turns and flies to the nearest Tier 3 Tunnel and lands.
- With no Tier 3 Tunnel it circles.
