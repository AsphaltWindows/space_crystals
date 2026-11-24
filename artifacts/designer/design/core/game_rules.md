# Game Rules

## Defeat Condition
In a normal game, a player is **defeated when they own no buildings**. This applies to every faction. (Custom scenarios may define other objectives.) Faction notes:
- **Every structure counts**, for every faction — including GDO Extraction Plates, Colonists Beacons, and Syndicate Laser Turrets.
- **The Syndicate**: when a Tunnel is destroyed, all of that Tunnel's expansions become neutral (no longer owned by the Syndicate player), but its Laser Turrets stay Syndicate-owned (pop-up turrets become upright). So losing the last Tunnel defeats the Syndicate only if they own no Laser Turrets. See factions/syndicate/tunnels.md > Tunnel Network > Losing Tunnels.

## On Defeat
(user, 2026-11-24)
- **Remaining units become neutral.** When a player is defeated, their remaining units stay on the map but **stop belonging to anyone** (neutral objects, like the Syndicate expansions of a destroyed Tunnel). Their orders are cleared and they stay idle where they are.
- Being neutral, they **never attack** anyone and are **never auto-targeted** (systems/combat.md > ValidTarget; auto-targeting never picks neutral objects). Any player can still destroy them with an explicit Attack order (systems/controls/commands.md > Attack).
- They block movement like other units and never step aside (only own idle units yield), so they stay as obstacles until destroyed.
- A defeated player stays defeated: nothing they had can be commanded or build again.

## End of Game
(user, 2026-11-24) When only **one player or team** is left undefeated, it **wins**. Every player sees Victory or Defeat, the game stops, and the replay is saved.

**Defeated before the end (team games)** (user, 2026-11-24): a player defeated while their team is still playing keeps watching with **their team's vision only** (what their allies see), never a full map reveal, so they can't pass on information their allies couldn't have. They can't command anything.
