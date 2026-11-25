# 0237 — Tunnel Areas can't overlap any player's Tunnels: crowding and blocking allowed

- **Date**: 2026-11-25T09:35:00-05:00
- **Answers**: Q0062 point 1 (other points pending)
- **Docs**: `factions/syndicate/tunnels.md > Tunnel Area Non-Overlap Rule`

## What changed and why
User chose **B**: the non-overlap rule applies between all players' Tunnels. In Syndicate-vs-Syndicate games, crowding or blocking an enemy's Tunnels (placement and upgrades) is a valid tactic.
- The build ghost judges known enemy Tunnels at their remembered position and last-seen tier.
- The check on arrival uses the truth, so it can fail on an unseen upgrade.
- Upgrade checks use the truth, and are blocked by any Tunnel's area.

## Implementation impact
This replaces the Director's own-Tunnels-only proposal (M4-E/F). The area overlap check includes other players' Tunnels; the ghost uses remembered tier.

## Verification
- In a two-Syndicate game, place a Tier 1 Tunnel 1 tile from where an enemy Tier 1 Tunnel's Tier 2 area would reach. The enemy can't upgrade that Tunnel.
