# Game Rules

## Defeat Condition
In a normal game, a player is **defeated when they own no buildings**. This applies to every faction. (Custom scenarios may define other objectives.) Faction notes:
- **Every structure counts**, for every faction — including GDO Extraction Plates, Colonists Beacons, and Syndicate Laser Turrets.
- **The Syndicate**: when a Tunnel is destroyed, all of that Tunnel's expansions become neutral (no longer owned by the Syndicate player), but its Laser Turrets stay Syndicate-owned (pop-up turrets become upright). So losing the last Tunnel defeats the Syndicate only if they own no Laser Turrets. See factions/syndicate/tunnels.md > Tunnel Network > Losing Tunnels.

## On Defeat
(user, 2026-11-24)
- **Remaining units stay, but can't be controlled.** When a player is defeated, their remaining units stay on the map, still owned by that player. Their orders are cleared and nobody can command them again (not the defeated player, not their allies). They behave as idle units: they stand where they are and defend themselves through normal auto-targeting (systems/controls/behaviors.md > BaseAutoTargeting) until they are destroyed.
- **No rebuilding**: since they can't be commanded, a defeated player's workers can't build again (a defeated player stays defeated).
- TBD: whether a defeated player's units still count as allies/enemies exactly as before (assumed yes), and what the defeated player can still see (see End of Game).

